pub mod audio;
pub mod mistral;
pub mod db;
pub mod export;
pub mod commands;
pub mod app_state;

use app_state::AppState;
use db::Database;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

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
            commands::update_session_title,
            commands::get_api_key,
            commands::set_api_key,
        ])
        .setup(|app| {
            // Build the tray context menu
            let open_item = MenuItemBuilder::with_id("open", "Ouvrir PopTranscribe").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "Quitter").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&open_item])
                .separator()
                .items(&[&quit_item])
                .build()?;

            // Create the system tray icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            // Intercept window close: hide instead of quitting so the app stays in the tray
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
