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
pub mod detect;
pub mod menu;
pub mod models;
pub mod parsers;

use commands::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init());

    // MCP plugin for QA automation — only in dev builds
    #[cfg(debug_assertions)]
    let builder = builder.plugin(tauri_plugin_mcp::init_with_config(
        tauri_plugin_mcp::PluginConfig::new("Kindling".to_string())
            .start_socket_server(true)
            .socket_path("/tmp/kindling-mcp.sock".into()),
    ));

    builder
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
            commands::import_longform,
            commands::import_scrivener,
            commands::preview_import,
            commands::create_sample_project,
            commands::create_blank_project,
            commands::create_screenplay_project,
            commands::get_page_count_estimate,
            commands::get_project,
            commands::get_recent_projects,
            commands::get_all_projects,
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
            commands::get_references,
            commands::get_scene_reference_items,
            commands::get_scene_reference_state,
            commands::create_reference,
            commands::update_reference,
            commands::delete_reference,
            commands::save_scene_reference_state,
            commands::reclassify_references,
            commands::save_beat_prose,
            commands::delete_beat,
            commands::reorder_beats,
            commands::split_beat,
            commands::rename_beat,
            commands::merge_beats,
            commands::get_discovery_notes,
            commands::create_discovery_note,
            commands::update_discovery_note,
            commands::delete_discovery_note,
            commands::promote_discovery_note_to_beat,
            commands::save_scene_synopsis,
            commands::update_scene_metadata,
            commands::update_scene_planning_status,
            commands::update_chapter_planning_status,
            commands::update_chapter_synopsis,
            commands::save_scene_prose,
            commands::switch_scene_editor_mode,
            commands::save_scene_page_prose,
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
            commands::export_to_longform,
            commands::export_to_docx,
            commands::export_to_epub,
            commands::get_project_word_count,
            commands::generate_treatment,
            commands::preview_scrivener_matches,
            commands::export_to_scrivener,
            // Snapshot commands
            commands::create_snapshot,
            commands::list_snapshots,
            commands::delete_snapshot,
            commands::restore_snapshot,
            commands::preview_snapshot,
            // App settings commands
            commands::get_app_settings,
            commands::update_app_settings,
            // Custom field commands
            commands::get_field_definitions,
            commands::get_all_field_definitions,
            commands::create_field_definition,
            commands::update_field_definition,
            commands::delete_field_definition,
            commands::reorder_field_definitions,
            commands::get_field_values,
            commands::get_field_values_bulk,
            commands::set_field_value,
            commands::clear_field_value,
            // Tag commands
            commands::get_tags,
            commands::create_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::reorder_tags,
            commands::tag_entity,
            commands::untag_entity,
            commands::get_entity_tags,
            commands::bulk_tag,
            commands::bulk_untag,
            commands::get_all_entity_tags,
            commands::filter_entities,
            commands::save_filter,
            commands::get_saved_filters,
            commands::delete_saved_filter,
            // Auto-detect commands
            commands::detect_scene_references,
            commands::detect_all_references,
            commands::dismiss_suggestion,
            // Template commands
            commands::get_bundled_templates,
            commands::get_user_templates,
            commands::apply_template,
            commands::save_project_as_template,
            commands::delete_user_template,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
