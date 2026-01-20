//! Import Commands
//!
//! Handles importing projects from external formats (Plottr, Markdown).

use tauri::State;

use crate::db;
use crate::models::Project;
use crate::parsers::{parse_markdown_outline, parse_plottr_file};

use super::AppState;

#[tauri::command]
pub async fn import_plottr(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_plottr_file(&path).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Use a transaction for the import
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    // Insert project
    db::insert_project(&conn, &parsed.project).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;

    // Insert chapters
    for chapter in &parsed.chapters {
        db::insert_chapter(&conn, chapter).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    // Insert scenes
    for scene in &parsed.scenes {
        db::insert_scene(&conn, scene).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    // Insert beats
    for beat in &parsed.beats {
        db::insert_beat(&conn, beat).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    // Insert characters
    for character in &parsed.characters {
        db::insert_character(&conn, character).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    // Insert locations
    for location in &parsed.locations {
        db::insert_location(&conn, location).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    // Insert scene references
    for (scene_id, character_id) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&conn, scene_id, character_id).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    for (scene_id, location_id) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&conn, scene_id, location_id).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_markdown(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_markdown_outline(&path).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    db::insert_project(&conn, &parsed.project).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;

    for chapter in &parsed.chapters {
        db::insert_chapter(&conn, chapter).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    for scene in &parsed.scenes {
        db::insert_scene(&conn, scene).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    for beat in &parsed.beats {
        db::insert_beat(&conn, beat).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;

    Ok(parsed.project)
}
