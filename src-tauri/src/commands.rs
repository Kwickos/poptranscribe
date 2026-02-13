use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait};
use tauri::{Emitter, State};
use crate::app_state::{ActiveSession, AppState, SendCapturer};
use crate::audio::capture::{AudioCapturer, CaptureMode};
use crate::db::{Session, Segment};
use crate::mistral::chat::Summary;

/// Detail view for a session, including its segments and optional summary.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionDetail {
    #[serde(flatten)]
    pub session: Session,
    pub segments: Vec<Segment>,
    pub summary: Option<Summary>,
}

// ── Session management ───────────────────────────────────────────────

#[tauri::command]
pub async fn start_session(
    mode: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    // Check that there is no already active session
    {
        let active = state.active_session.lock().map_err(|e| e.to_string())?;
        if active.is_some() {
            return Err("Une session est deja en cours. Arretez-la d'abord.".to_string());
        }
    }

    // Create session in DB
    let session_id = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let title = format!("Reunion {}", chrono::Local::now().format("%d/%m %H:%M"));
        db.create_session(&title, &mode).map_err(|e| e.to_string())?
    };

    // Check API key
    let api_key = {
        let key = state.api_key.lock().map_err(|e| e.to_string())?;
        key.clone()
    };
    if api_key.is_empty() {
        return Err("Cle API Mistral non configuree. Allez dans Parametres.".to_string());
    }

    // Read configured input device from settings
    let device_name = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_setting("input_device").ok().flatten()
    };

    // Start audio capture
    let capture_mode = match mode.as_str() {
        "visio" => CaptureMode::Visio,
        _ => CaptureMode::InPerson,
    };

    let mut capturer = AudioCapturer::new(capture_mode, device_name);
    let receiver = capturer.start().map_err(|e| e.to_string())?;
    let actual_sample_rate = capturer.actual_sample_rate;

    let audio_samples = Arc::new(std::sync::Mutex::new(Vec::<i16>::new()));
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);

    // Clone handles for the background task
    let session_id_clone = session_id.clone();
    let audio_samples_clone = audio_samples.clone();
    let app_clone = app.clone();
    let db_clone = Arc::clone(&state.db);

    // Background task: real-time transcription via WebSocket
    tokio::spawn(async move {
        let sample_rate = actual_sample_rate;
        let stop_rx = stop_rx;

        // Connect to Mistral real-time WebSocket
        let (rt_handle, mut rt_events) = match crate::mistral::realtime::connect_realtime(
            &api_key,
            sample_rate,
        )
        .await
        {
            Ok(conn) => conn,
            Err(e) => {
                eprintln!("[session] Failed to connect realtime transcription: {}", e);
                let _ = app_clone.emit(
                    "session-error",
                    format!("Erreur connexion transcription: {}", e),
                );
                return;
            }
        };

        eprintln!("[session] Real-time transcription connected");

        // Spawn event receiver: forwards WebSocket events to Tauri UI
        let app_events = app_clone.clone();
        let sid_events = session_id_clone.clone();
        let db_events = Arc::clone(&db_clone);
        tokio::spawn(async move {
            while let Some(event) = rt_events.recv().await {
                match event {
                    crate::mistral::realtime::TranscriptionEvent::TextDelta { text } => {
                        let _ = app_events.emit("transcription-delta", &text);
                    }
                    crate::mistral::realtime::TranscriptionEvent::Segment {
                        text,
                        start,
                        end,
                    } => {
                        let segment_id = {
                            if let Ok(db) = db_events.lock() {
                                db.save_segment(
                                    &sid_events, &text, start, end, None, false,
                                )
                                .ok()
                            } else {
                                None
                            }
                        };

                        let segment = serde_json::json!({
                            "id": segment_id.unwrap_or(0),
                            "session_id": sid_events,
                            "text": text,
                            "start_time": start,
                            "end_time": end,
                            "speaker": null,
                            "is_diarized": false
                        });
                        let _ = app_events.emit("transcription-segment", segment);
                    }
                    crate::mistral::realtime::TranscriptionEvent::Error { message } => {
                        eprintln!("[session] Realtime error: {}", message);
                        let _ = app_events.emit("session-error", &message);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Main audio loop: read chunks, accumulate for WAV, send to WebSocket
        loop {
            if *stop_rx.borrow() {
                break;
            }

            match receiver.try_recv() {
                Ok(chunk) => {
                    if !chunk.is_empty() {
                        // Audio level for UI
                        let rms = (chunk.iter()
                            .map(|&s| (s as f64).powi(2))
                            .sum::<f64>()
                            / chunk.len() as f64)
                            .sqrt();
                        let level = ((rms / i16::MAX as f64) * 100.0).min(100.0);
                        let _ = app_clone.emit("audio-level", level as u32);

                        // Accumulate for WAV save
                        if let Ok(mut samples) = audio_samples_clone.lock() {
                            samples.extend_from_slice(&chunk);
                        }

                        // Send to WebSocket for real-time transcription
                        rt_handle.send_audio(chunk);
                    }
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }

        // Signal end of audio to WebSocket
        rt_handle.end_audio();
    });

    // Store active session in state (wrap capturer for Send safety)
    let mut active = state.active_session.lock().map_err(|e| e.to_string())?;
    *active = Some(ActiveSession {
        id: session_id.clone(),
        capturer: SendCapturer(capturer),
        audio_samples,
        sample_rate: actual_sample_rate,
        stop_signal: stop_tx,
    });

    Ok(session_id)
}

#[tauri::command]
pub async fn stop_session(
    session_id: String,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let (samples, sample_rate) = {
        let mut active = state.active_session.lock().map_err(|e| e.to_string())?;

        if let Some(mut session) = active.take() {
            if session.id != session_id {
                // Put it back
                let id = session.id.clone();
                *active = Some(session);
                return Err(format!(
                    "La session active ({}) ne correspond pas a celle demandee ({})",
                    id, session_id
                ));
            }

            // Signal the background task to stop
            let _ = session.stop_signal.send(true);
            // Stop audio capture hardware
            session.capturer.0.stop();

            // Extract accumulated audio
            let samples = session
                .audio_samples
                .lock()
                .map(|s| s.clone())
                .unwrap_or_default();
            let sr = session.sample_rate;

            (samples, sr)
        } else {
            return Err("Aucune session active".to_string());
        }
    };

    // Save full WAV file
    let audio_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("poptranscribe")
        .join("audio");
    std::fs::create_dir_all(&audio_dir).ok();
    let audio_path = audio_dir.join(format!("{}.wav", session_id));

    let duration = samples.len() as f64 / sample_rate as f64;

    crate::audio::store::save_wav(&audio_path, &samples, sample_rate)
        .map_err(|e| format!("Erreur sauvegarde audio: {}", e))?;

    // Update session in DB with audio path and duration
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.update_session_audio_path(
            &session_id,
            audio_path.to_str().unwrap_or(""),
        )
        .map_err(|e| e.to_string())?;
        db.update_session_duration(&session_id, duration)
            .map_err(|e| e.to_string())?;
    }

    // Clone what we need for the background batch reprocessing + summary task
    let api_key = {
        let key = state.api_key.lock().map_err(|e| e.to_string())?;
        key.clone()
    };
    let db_clone = Arc::clone(&state.db);

    // Background task: batch transcription with diarization, then summary
    tokio::spawn(async move {
        match crate::mistral::batch::transcribe_batch(&api_key, &audio_path, true, Some("fr"))
            .await
        {
            Ok(response) => {
                // Clear old live (non-diarized) segments and save diarized ones
                if let Ok(db) = db_clone.lock() {
                    // Remove the live streaming segments so they are replaced by
                    // higher-quality diarized ones
                    let _ = db.clear_live_segments(&session_id);

                    for seg in &response.segments {
                        let _ = db.save_segment(
                            &session_id,
                            &seg.text,
                            seg.start,
                            seg.end,
                            seg.speaker_id.as_deref(),
                            true,
                        );
                    }
                }

                // Build transcript text for summary
                let transcript_text: String = response
                    .segments
                    .iter()
                    .map(|s| {
                        if let Some(ref speaker) = s.speaker_id {
                            format!("{}: {}", speaker, s.text)
                        } else {
                            s.text.clone()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                // Generate AI title + summary
                if !transcript_text.is_empty() {
                    // Title generation (fast, runs first)
                    match crate::mistral::chat::generate_title(&api_key, &transcript_text).await
                    {
                        Ok(title) => {
                            if let Ok(db) = db_clone.lock() {
                                let _ = db.update_session_title(&session_id, &title);
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[session] Erreur generation titre pour {}: {}",
                                session_id, e
                            );
                        }
                    }

                    // Summary generation
                    match crate::mistral::chat::generate_summary(&api_key, &transcript_text).await
                    {
                        Ok(summary) => {
                            if let Ok(summary_json) = serde_json::to_string(&summary) {
                                if let Ok(db) = db_clone.lock() {
                                    let _ = db.save_summary(&session_id, &summary_json);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "[session] Erreur generation resume pour {}: {}",
                                session_id, e
                            );
                        }
                    }
                }

                let _ = app.emit("session-complete", &session_id);
            }
            Err(e) => {
                eprintln!(
                    "[session] Erreur transcription batch pour {}: {}",
                    session_id, e
                );
                let _ = app.emit(
                    "session-error",
                    format!("Erreur de transcription: {}", e),
                );
            }
        }
    });

    Ok(())
}

// ── Data retrieval ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_detail(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<SessionDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let session = db.get_session(&session_id).map_err(|e| e.to_string())?;
    let segments = db.get_segments(&session_id).map_err(|e| e.to_string())?;
    let summary: Option<Summary> = session
        .summary_json
        .as_ref()
        .and_then(|json| serde_json::from_str(json).ok());
    Ok(SessionDetail {
        session,
        segments,
        summary,
    })
}

// ── Search ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn search_text(
    query: String,
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<Segment>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_text(&query, session_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_llm(
    query: String,
    session_id: String,
    live_text: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let segments = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_segments(&session_id).map_err(|e| e.to_string())?
    };
    let api_key = {
        let key = state.api_key.lock().map_err(|e| e.to_string())?;
        key.clone()
    };

    let mut transcript: String = segments
        .iter()
        .map(|s| {
            if let Some(ref speaker) = s.speaker {
                format!("{}: {}", speaker, s.text)
            } else {
                s.text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Append live (in-progress) text from real-time transcription
    if let Some(ref lt) = live_text {
        if !lt.is_empty() {
            if !transcript.is_empty() {
                transcript.push('\n');
            }
            transcript.push_str(lt);
        }
    }

    if transcript.is_empty() {
        return Err("Aucune transcription disponible pour cette session.".to_string());
    }

    crate::mistral::chat::search_transcript(&api_key, &transcript, &query)
        .await
        .map_err(|e| e.to_string())
}

// ── Speaker management ──────────────────────────────────────────────

#[tauri::command]
pub async fn rename_speaker(
    session_id: String,
    old_name: String,
    new_name: String,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rename_speaker(&session_id, &old_name, &new_name)
        .map_err(|e| e.to_string())
}

// ── Export ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn export_session(
    session_id: String,
    format: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    match format.as_str() {
        "markdown" => {
            // Load session detail from DB
            let (session, segments, summary) = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let session = db.get_session(&session_id).map_err(|e| e.to_string())?;
                let segments = db.get_segments(&session_id).map_err(|e| e.to_string())?;
                let summary: Option<Summary> = session
                    .summary_json
                    .as_ref()
                    .and_then(|json| serde_json::from_str(json).ok());
                (session, segments, summary)
            };

            // Generate markdown content
            let md = crate::export::export_markdown(
                &session.title,
                &session.created_at,
                session.duration_secs,
                &segments,
                &summary,
            );

            // Use configured export directory, or default to ~/Documents/poptranscribe/exports/
            let export_dir = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                match db.get_setting("export_dir").ok().flatten() {
                    Some(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
                    _ => dirs::document_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("poptranscribe")
                        .join("exports"),
                }
            };

            // Sanitize title for filename
            let safe_title: String = session
                .title
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
                .collect();
            let filename = format!("{}_{}.md", safe_title, session_id.split('-').next().unwrap_or("export"));
            let file_path = export_dir.join(&filename);

            crate::export::export_to_file(&md, &file_path)
                .map_err(|e| format!("Erreur ecriture fichier: {}", e))?;

            Ok(file_path.to_string_lossy().to_string())
        }
        "pdf" => {
            let (session, segments, summary) = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                let session = db.get_session(&session_id).map_err(|e| e.to_string())?;
                let segments = db.get_segments(&session_id).map_err(|e| e.to_string())?;
                let summary: Option<Summary> = session
                    .summary_json
                    .as_ref()
                    .and_then(|json| serde_json::from_str(json).ok());
                (session, segments, summary)
            };

            let export_dir = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                match db.get_setting("export_dir").ok().flatten() {
                    Some(dir) if !dir.is_empty() => std::path::PathBuf::from(dir),
                    _ => dirs::document_dir()
                        .unwrap_or_else(|| std::path::PathBuf::from("."))
                        .join("poptranscribe")
                        .join("exports"),
                }
            };

            let safe_title: String = session
                .title
                .chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
                .collect();
            let filename = format!("{}_{}.pdf", safe_title, session_id.split('-').next().unwrap_or("export"));
            let file_path = export_dir.join(&filename);

            crate::export::export_pdf(
                &session.title,
                &session.created_at,
                session.duration_secs,
                &segments,
                &summary,
                &file_path,
            )?;

            Ok(file_path.to_string_lossy().to_string())
        }
        other => Err(format!("Export {} pas encore supporte", other)),
    }
}

#[tauri::command]
pub async fn update_session_title(
    session_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_session_title(&session_id, &title).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_session(
    session_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_session(&session_id).map_err(|e| e.to_string())
}

// ── Settings ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_api_key(state: State<'_, AppState>) -> Result<String, String> {
    let key = state.api_key.lock().map_err(|e| e.to_string())?;
    Ok(key.clone())
}

#[tauri::command]
pub async fn set_api_key(key: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut api_key = state.api_key.lock().map_err(|e| e.to_string())?;
    *api_key = key.clone();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting("api_key", &key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(key: String, state: State<'_, AppState>) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_setting(key: String, value: String, state: State<'_, AppState>) -> Result<(), String> {
    // If the key is "api_key", also update the in-memory cache
    if key == "api_key" {
        let mut api_key = state.api_key.lock().map_err(|e| e.to_string())?;
        *api_key = value.clone();
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

// ── Audio devices ────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
pub async fn list_input_devices() -> Result<Vec<AudioDevice>, String> {
    let host = cpal::default_host();
    let default_name = host
        .default_input_device()
        .and_then(|d| d.name().ok());

    let devices = host
        .input_devices()
        .map_err(|e| format!("Impossible de lister les peripheriques audio: {}", e))?;

    let mut result = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            let is_default = default_name.as_deref() == Some(&name);
            result.push(AudioDevice { name, is_default });
        }
    }
    Ok(result)
}

// ── Folder picker ────────────────────────────────────────────────────

#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file()
        .blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}
