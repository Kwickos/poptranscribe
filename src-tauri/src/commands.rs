use std::sync::Arc;
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

    // Start audio capture
    let capture_mode = match mode.as_str() {
        "visio" => CaptureMode::Visio,
        _ => CaptureMode::InPerson,
    };

    let mut capturer = AudioCapturer::new(capture_mode);
    let receiver = capturer.start().map_err(|e| e.to_string())?;
    let actual_sample_rate = capturer.actual_sample_rate;

    let audio_samples = Arc::new(std::sync::Mutex::new(Vec::<i16>::new()));
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);

    // Clone handles for the background task
    let session_id_clone = session_id.clone();
    let audio_samples_clone = audio_samples.clone();
    let app_clone = app.clone();
    let db_clone = Arc::clone(&state.db);

    // Background task: collect audio chunks, periodically transcribe via streaming API
    tokio::spawn(async move {
        let mut accumulated = Vec::<i16>::new();
        let sample_rate = actual_sample_rate;
        // Transcribe every ~30 seconds of audio
        let chunk_interval = sample_rate as usize * 30;
        let mut chunk_counter = 0u32;
        // Track cumulative time offset for segment timestamps
        let mut time_offset: f64 = 0.0;
        let stop_rx = stop_rx;

        loop {
            // Check stop signal
            if *stop_rx.borrow() {
                break;
            }

            // Collect available audio chunks from the sync receiver
            loop {
                match receiver.try_recv() {
                    Ok(chunk) => {
                        if !chunk.is_empty() {
                            // Compute RMS audio level for the UI meter
                            let rms = (chunk.iter()
                                .map(|&s| (s as f64).powi(2))
                                .sum::<f64>()
                                / chunk.len() as f64)
                                .sqrt();
                            let level = ((rms / i16::MAX as f64) * 100.0).min(100.0);
                            let _ = app_clone.emit("audio-level", level as u32);

                            accumulated.extend_from_slice(&chunk);
                            if let Ok(mut samples) = audio_samples_clone.lock() {
                                samples.extend_from_slice(&chunk);
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Capture has stopped
                        return;
                    }
                }
            }

            // When we have enough audio, send for streaming transcription
            if accumulated.len() >= chunk_interval {
                let samples_to_transcribe = accumulated.clone();
                accumulated.clear();
                chunk_counter += 1;
                let current_offset = time_offset;
                time_offset += samples_to_transcribe.len() as f64 / sample_rate as f64;

                let temp_path = std::env::temp_dir()
                    .join(format!("poptranscribe_chunk_{}.wav", chunk_counter));

                if crate::audio::store::save_wav(&temp_path, &samples_to_transcribe, sample_rate)
                    .is_ok()
                {
                    let api_key = api_key.clone();
                    let app = app_clone.clone();
                    let sid = session_id_clone.clone();
                    let db = Arc::clone(&db_clone);

                    tokio::spawn(async move {
                        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                        let path = temp_path.clone();

                        // Spawn the streaming transcription request
                        tokio::spawn(async move {
                            let _ = crate::mistral::realtime::transcribe_stream(
                                &api_key, &path, Some("fr"), tx,
                            )
                            .await;
                            let _ = std::fs::remove_file(&path);
                        });

                        // Process streaming events as they arrive
                        while let Some(event) = rx.recv().await {
                            if let crate::mistral::realtime::TranscriptionEvent::Segment {
                                text,
                                start,
                                end,
                            } = event
                            {
                                let abs_start = current_offset + start;
                                let abs_end = current_offset + end;

                                // Save live segment to DB for search
                                let segment_id = {
                                    if let Ok(db) = db.lock() {
                                        db.save_segment(
                                            &sid, &text, abs_start, abs_end, None, false,
                                        )
                                        .ok()
                                    } else {
                                        None
                                    }
                                };

                                let segment = serde_json::json!({
                                    "id": segment_id.unwrap_or(0),
                                    "session_id": sid,
                                    "text": text,
                                    "start_time": abs_start,
                                    "end_time": abs_end,
                                    "speaker": null,
                                    "is_diarized": false
                                });
                                let _ = app.emit("transcription-segment", segment);
                            }
                        }
                    });
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        // Process any remaining accumulated audio after stop signal
        if !accumulated.is_empty() {
            chunk_counter += 1;
            let current_offset = time_offset;
            let temp_path = std::env::temp_dir()
                .join(format!("poptranscribe_chunk_{}.wav", chunk_counter));

            if crate::audio::store::save_wav(&temp_path, &accumulated, sample_rate).is_ok() {
                let api_key_final = api_key.clone();
                let app = app_clone.clone();
                let sid = session_id_clone.clone();
                let db = Arc::clone(&db_clone);

                tokio::spawn(async move {
                    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                    let path = temp_path.clone();

                    tokio::spawn(async move {
                        let _ = crate::mistral::realtime::transcribe_stream(
                            &api_key_final, &path, Some("fr"), tx,
                        )
                        .await;
                        let _ = std::fs::remove_file(&path);
                    });

                    while let Some(event) = rx.recv().await {
                        if let crate::mistral::realtime::TranscriptionEvent::Segment {
                            text,
                            start,
                            end,
                        } = event
                        {
                            let abs_start = current_offset + start;
                            let abs_end = current_offset + end;

                            let segment_id = {
                                if let Ok(db) = db.lock() {
                                    db.save_segment(&sid, &text, abs_start, abs_end, None, false)
                                        .ok()
                                } else {
                                    None
                                }
                            };

                            let segment = serde_json::json!({
                                "id": segment_id.unwrap_or(0),
                                "session_id": sid,
                                "text": text,
                                "start_time": abs_start,
                                "end_time": abs_end,
                                "speaker": null,
                                "is_diarized": false
                            });
                            let _ = app.emit("transcription-segment", segment);
                        }
                    }
                });
            }
        }
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
                    let _ = db.clear_segments(&session_id, false);

                    for seg in &response.segments {
                        let _ = db.save_segment(
                            &session_id,
                            &seg.text,
                            seg.start,
                            seg.end,
                            seg.speaker.as_deref(),
                            true,
                        );
                    }
                }

                // Build transcript text for summary
                let transcript_text: String = response
                    .segments
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

                // Generate summary
                if !transcript_text.is_empty() {
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

    let transcript: String = segments
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
    _session_id: String,
    _format: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // Will be implemented in Task 18
    Err("Export not yet implemented".to_string())
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
