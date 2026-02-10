use tauri::State;
use crate::app_state::AppState;
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
pub async fn start_session(mode: String, state: State<'_, AppState>) -> Result<String, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let title = format!("Reunion {}", chrono::Local::now().format("%d/%m %H:%M"));
    let session_id = db.create_session(&title, &mode).map_err(|e| e.to_string())?;
    // Audio capture + streaming will be wired in Task 16
    Ok(session_id)
}

#[tauri::command]
pub async fn stop_session(_session_id: String, _state: State<'_, AppState>) -> Result<(), String> {
    // Will be wired in Task 16
    Ok(())
}

// ── Data retrieval ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_sessions(state: State<'_, AppState>) -> Result<Vec<Session>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_detail(session_id: String, state: State<'_, AppState>) -> Result<SessionDetail, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let session = db.get_session(&session_id).map_err(|e| e.to_string())?;
    let segments = db.get_segments(&session_id).map_err(|e| e.to_string())?;
    let summary: Option<Summary> = session.summary_json.as_ref()
        .and_then(|json| serde_json::from_str(json).ok());
    Ok(SessionDetail { session, segments, summary })
}

// ── Search ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn search_text(query: String, session_id: Option<String>, state: State<'_, AppState>) -> Result<Vec<Segment>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_text(&query, session_id.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_llm(query: String, session_id: String, state: State<'_, AppState>) -> Result<String, String> {
    let segments = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.get_segments(&session_id).map_err(|e| e.to_string())?
    };
    let api_key = {
        let key = state.api_key.lock().map_err(|e| e.to_string())?;
        key.clone()
    };

    let transcript: String = segments.iter()
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
pub async fn rename_speaker(session_id: String, old_name: String, new_name: String, state: State<'_, AppState>) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rename_speaker(&session_id, &old_name, &new_name).map_err(|e| e.to_string())
}

// ── Export ───────────────────────────────────────────────────────────

#[tauri::command]
pub async fn export_session(_session_id: String, _format: String, _state: State<'_, AppState>) -> Result<String, String> {
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
