pub mod audio;
pub mod mistral;
pub mod db;
pub mod export;
pub mod commands;
pub mod app_state;

use app_state::AppState;
use db::Database;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("poptranscribe")
        .join("poptranscribe.db");

    // Create parent directory if needed
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db = Database::new(&db_path).expect("Failed to open database");

    // Load API key from settings
    let api_key = db.get_setting("api_key").ok().flatten().unwrap_or_default();
    let state = AppState::new(db);
    *state.api_key.lock().unwrap() = api_key;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::start_session,
            commands::stop_session,
            commands::get_sessions,
            commands::get_session_detail,
            commands::search_text,
            commands::search_llm,
            commands::rename_speaker,
            commands::export_session,
            commands::get_api_key,
            commands::set_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
