pub mod commands;
pub mod db;
pub mod models;
pub mod parsers;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Get the app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data directory");

            // Initialize application state with database
            let state =
                AppState::new(app_data_dir).expect("Failed to initialize application state");

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::import_plottr,
            commands::import_scrivener,
            commands::import_markdown,
            commands::get_project,
            commands::get_recent_projects,
            commands::get_chapters,
            commands::create_chapter,
            commands::get_scenes,
            commands::create_scene,
            commands::get_beats,
            commands::get_characters,
            commands::get_locations,
            commands::save_beat_prose,
            commands::save_scene_prose,
            commands::reorder_chapters,
            commands::reorder_scenes,
            commands::move_scene_to_chapter,
            commands::get_chapter_content_counts,
            commands::get_scene_beat_count,
            commands::delete_chapter,
            commands::delete_scene,
            commands::reimport_project,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
