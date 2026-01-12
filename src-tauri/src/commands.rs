use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::db::{self, initialize_schema};
use crate::models::{Project, Chapter, Scene, Beat, Character, Location};
use crate::parsers::{parse_plottr_file, parse_scrivener_project, parse_markdown_outline};

// ============================================================================
// Application State
// ============================================================================

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(app_data_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Ensure the data directory exists
        std::fs::create_dir_all(&app_data_dir)?;

        let db_path = app_data_dir.join("kindling.db");
        let conn = Connection::open(&db_path)?;

        // Initialize schema
        initialize_schema(&conn)?;

        Ok(Self {
            db: Mutex::new(conn),
        })
    }
}

// ============================================================================
// Import Commands
// ============================================================================

#[tauri::command]
pub async fn import_plottr(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_plottr_file(&path).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Use a transaction for the import
    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

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
pub async fn import_scrivener(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_scrivener_project(&path).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

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

    for character in &parsed.characters {
        db::insert_character(&conn, character).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
    }

    for location in &parsed.locations {
        db::insert_location(&conn, location).map_err(|e| {
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

    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

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

// ============================================================================
// Project Commands
// ============================================================================

#[tauri::command]
pub async fn get_project(id: String, state: State<'_, AppState>) -> Result<Project, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::get_project(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())
}

#[tauri::command]
pub async fn get_recent_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_recent_projects(&conn, 10).map_err(|e| e.to_string())
}

// ============================================================================
// Chapter Commands
// ============================================================================

#[tauri::command]
pub async fn get_chapters(project_id: String, state: State<'_, AppState>) -> Result<Vec<Chapter>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_chapters(&conn, &uuid).map_err(|e| e.to_string())
}

// ============================================================================
// Scene Commands
// ============================================================================

#[tauri::command]
pub async fn get_scenes(chapter_id: String, state: State<'_, AppState>) -> Result<Vec<Scene>, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scenes(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_scene_prose(scene_id: String, prose: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_scene_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())
}

// ============================================================================
// Beat Commands
// ============================================================================

#[tauri::command]
pub async fn get_beats(scene_id: String, state: State<'_, AppState>) -> Result<Vec<Beat>, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_beats(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_beat_prose(beat_id: String, prose: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&beat_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_beat_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())
}

// ============================================================================
// Character Commands
// ============================================================================

#[tauri::command]
pub async fn get_characters(project_id: String, state: State<'_, AppState>) -> Result<Vec<Character>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_characters(&conn, &uuid).map_err(|e| e.to_string())
}

// ============================================================================
// Location Commands
// ============================================================================

#[tauri::command]
pub async fn get_locations(project_id: String, state: State<'_, AppState>) -> Result<Vec<Location>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_locations(&conn, &uuid).map_err(|e| e.to_string())
}
