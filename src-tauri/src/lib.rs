//! Kindling - Rust Backend
//!
//! This is the Tauri backend for Kindling, a writing application that bridges
//! outline planning and prose writing.
//!
//! # Module Overview
//!
//! - [`commands`]: Tauri IPC command handlers (called from frontend via `invoke()`)
//! - [`db`]: SQLite database schema and query functions
//! - [`models`]: Data structures (Project, Chapter, Scene, Beat, Character, Location)
//! - [`parsers`]: Import parsers for Plottr and Markdown formats
//!
//! # Architecture
//!
//! The frontend communicates with this backend via Tauri's IPC system. Commands
//! are registered in the `run()` function and exposed to the frontend. All data
//! is persisted to a SQLite database in the app's data directory.
//!
//! See `docs/ARCHITECTURE.md` for a full overview of the system.

pub mod commands;
pub mod db;
pub mod menu;
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

            // Set up application menu
            let app_handle = app.handle();
            menu::create_menu(app_handle).expect("Failed to create menu");
            menu::setup_menu_events(app_handle);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::import_plottr,
            commands::import_ywriter,
            commands::import_markdown,
            commands::get_project,
            commands::get_recent_projects,
            commands::update_project_settings,
            commands::delete_project,
            commands::get_chapters,
            commands::create_chapter,
            commands::get_scenes,
            commands::create_scene,
            commands::get_beats,
            commands::create_beat,
            commands::get_characters,
            commands::get_locations,
            commands::save_beat_prose,
            commands::save_scene_synopsis,
            commands::save_scene_prose,
            commands::reorder_chapters,
            commands::reorder_scenes,
            commands::move_scene_to_chapter,
            commands::get_chapter_content_counts,
            commands::get_scene_beat_count,
            commands::delete_chapter,
            commands::delete_scene,
            commands::reimport_project,
            commands::get_sync_preview,
            commands::apply_sync,
            // Rename commands
            commands::rename_chapter,
            commands::rename_scene,
            // Duplicate commands
            commands::duplicate_chapter,
            commands::duplicate_scene,
            // Archive commands
            commands::archive_chapter,
            commands::archive_scene,
            commands::restore_chapter,
            commands::restore_scene,
            commands::get_archived_items,
            // Lock and Part commands
            commands::lock_chapter,
            commands::unlock_chapter,
            commands::lock_scene,
            commands::unlock_scene,
            commands::set_chapter_is_part,
            // Export commands
            commands::export_to_markdown,
            commands::export_to_docx,
            commands::get_project_word_count,
            // Snapshot commands
            commands::create_snapshot,
            commands::list_snapshots,
            commands::delete_snapshot,
            commands::restore_snapshot,
            commands::preview_snapshot,
            // App settings commands
            commands::get_app_settings,
            commands::update_app_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
