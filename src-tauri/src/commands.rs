use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

use crate::db::{self, initialize_schema};
use crate::models::{Beat, Chapter, Character, Location, Project, Scene};
use crate::parsers::{parse_markdown_outline, parse_plottr_file, parse_scrivener_project};

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
pub async fn import_scrivener(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_scrivener_project(&path).map_err(|e| e.to_string())?;

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
pub async fn get_chapters(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Chapter>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_chapters(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_chapter(
    project_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<Chapter, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get next position
    let position =
        db::get_max_chapter_position(&conn, &project_uuid).map_err(|e| e.to_string())? + 1;

    let chapter = Chapter {
        id: Uuid::new_v4(),
        project_id: project_uuid,
        title,
        position,
        source_id: None,
        archived: false,
        locked: false,
    };

    db::insert_chapter(&conn, &chapter).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(chapter)
}

// ============================================================================
// Scene Commands
// ============================================================================

#[tauri::command]
pub async fn get_scenes(
    chapter_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Scene>, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scenes(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_scene(
    chapter_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<Scene, String> {
    let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get next position
    let position = db::get_max_scene_position(&conn, &chapter_uuid).map_err(|e| e.to_string())? + 1;

    let scene = Scene {
        id: Uuid::new_v4(),
        chapter_id: chapter_uuid,
        title,
        synopsis: None,
        prose: None,
        position,
        source_id: None,
        archived: false,
        locked: false,
    };

    db::insert_scene(&conn, &scene).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &chapter_uuid).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(scene)
}

#[tauri::command]
pub async fn save_scene_prose(
    scene_id: String,
    prose: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot edit a locked scene".to_string());
    }

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
pub async fn save_beat_prose(
    beat_id: String,
    prose: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&beat_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the scene_id from the beat and check if it's locked
    let scene_id: Option<Uuid> = conn
        .query_row(
            "SELECT scene_id FROM beats WHERE id = ?1",
            rusqlite::params![uuid.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                Ok(Uuid::parse_str(&id_str).ok())
            },
        )
        .ok()
        .flatten();

    if let Some(scene_id) = scene_id {
        if db::is_scene_locked(&conn, &scene_id).map_err(|e| e.to_string())? {
            return Err("Cannot edit beats in a locked scene".to_string());
        }
    }

    db::update_beat_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_beat(
    scene_id: String,
    content: String,
    state: State<'_, AppState>,
) -> Result<Beat, String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &scene_uuid).map_err(|e| e.to_string())? {
        return Err("Cannot add beats to a locked scene".to_string());
    }

    // Get next position
    let max_pos = db::get_max_beat_position(&conn, &scene_uuid).map_err(|e| e.to_string())?;
    let position = max_pos + 1;

    let beat = Beat::new(scene_uuid, content, position);
    db::insert_beat(&conn, &beat).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_scene_project_id(&conn, &scene_uuid).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(beat)
}

#[tauri::command]
pub async fn save_scene_synopsis(
    scene_id: String,
    synopsis: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot edit a locked scene".to_string());
    }

    db::update_scene_synopsis(&conn, &uuid, synopsis.as_deref()).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

// ============================================================================
// Character Commands
// ============================================================================

#[tauri::command]
pub async fn get_characters(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Character>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_characters(&conn, &uuid).map_err(|e| e.to_string())
}

// ============================================================================
// Location Commands
// ============================================================================

#[tauri::command]
pub async fn get_locations(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Location>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_locations(&conn, &uuid).map_err(|e| e.to_string())
}

// ============================================================================
// Reordering Commands
// ============================================================================

#[tauri::command]
pub async fn reorder_chapters(
    project_id: String,
    chapter_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let chapter_uuids: Vec<Uuid> = chapter_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::reorder_chapters(&conn, &project_uuid, &chapter_uuids).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn reorder_scenes(
    chapter_id: String,
    scene_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let scene_uuids: Vec<Uuid> = scene_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::reorder_scenes(&conn, &chapter_uuid, &scene_uuids).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &chapter_uuid).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn move_scene_to_chapter(
    scene_id: String,
    target_chapter_id: String,
    position: i32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let target_chapter_uuid = Uuid::parse_str(&target_chapter_id).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::move_scene_to_chapter(&conn, &scene_uuid, &target_chapter_uuid, position)
        .map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &target_chapter_uuid).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Delete Commands
// ============================================================================

#[derive(serde::Serialize)]
pub struct ChapterContentCounts {
    pub scene_count: i32,
    pub beat_count: i32,
}

#[tauri::command]
pub async fn get_chapter_content_counts(
    chapter_id: String,
    state: State<'_, AppState>,
) -> Result<ChapterContentCounts, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let (scene_count, beat_count) =
        db::get_chapter_content_counts(&conn, &uuid).map_err(|e| e.to_string())?;
    Ok(ChapterContentCounts {
        scene_count,
        beat_count,
    })
}

#[tauri::command]
pub async fn get_scene_beat_count(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<i32, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_scene_beat_count(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_chapter(chapter_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if chapter is locked
    if db::is_chapter_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot delete a locked chapter".to_string());
    }

    // Get project ID before deleting for updating modified time
    let project_id = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())?;

    db::delete_chapter(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(pid) = project_id {
        db::update_project_modified(&conn, &pid).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_scene(
    scene_id: String,
    chapter_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &scene_uuid).map_err(|e| e.to_string())? {
        return Err("Cannot delete a locked scene".to_string());
    }

    db::delete_scene(&conn, &scene_uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &chapter_uuid).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Re-import Commands
// ============================================================================

#[derive(serde::Serialize)]
pub struct ReimportSummary {
    pub chapters_added: i32,
    pub chapters_updated: i32,
    pub scenes_added: i32,
    pub scenes_updated: i32,
    pub beats_added: i32,
    pub beats_updated: i32,
    pub prose_preserved: i32,
}

// ============================================================================
// Sync Preview Types
// ============================================================================

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncAddition {
    pub id: String,
    pub item_type: String, // "chapter", "scene", "beat"
    pub title: String,
    pub parent_title: Option<String>, // Chapter name for scenes, Scene name for beats
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncChange {
    pub id: String,
    pub item_type: String, // "chapter", "scene", "beat"
    pub field: String,     // "title", "synopsis", "content"
    pub item_title: String,
    pub current_value: String,
    pub new_value: String,
    pub db_id: String, // The database ID to update if accepted
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncPreview {
    pub additions: Vec<SyncAddition>,
    pub changes: Vec<SyncChange>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplySyncRequest {
    pub accepted_change_ids: Vec<String>,
}

#[tauri::command]
pub async fn reimport_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<ReimportSummary, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the existing project to find source path and type
    let project = db::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for reimport".to_string())?;

    // Re-parse the source file based on source type
    let parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::Scrivener => {
            return Err("Scrivener reimport not yet supported".to_string());
        }
        crate::models::SourceType::Markdown => {
            return Err("Markdown reimport not yet supported".to_string());
        }
    };

    let mut summary = ReimportSummary {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
    };

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    // Process chapters
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            // Try to find existing chapter by source_id
            if let Some(existing) = db::find_chapter_by_source_id(&conn, &project_uuid, source_id)
                .map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e.to_string()
            })? {
                // Update existing chapter
                db::update_chapter(
                    &conn,
                    &existing.id,
                    &new_chapter.title,
                    new_chapter.position,
                )
                .map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?;
                summary.chapters_updated += 1;
            } else {
                // Insert new chapter with project's actual UUID
                let chapter_to_insert = Chapter {
                    id: new_chapter.id,
                    project_id: project_uuid,
                    title: new_chapter.title.clone(),
                    position: new_chapter.position,
                    source_id: new_chapter.source_id.clone(),
                    archived: false,
                    locked: false,
                };
                db::insert_chapter(&conn, &chapter_to_insert).map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?;
                summary.chapters_added += 1;
            }
        }
    }

    // Build a map from parsed chapter source_id to our DB chapter
    let db_chapters = db::get_chapters(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let chapter_source_to_db: std::collections::HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to parsed chapter source_id
    let parsed_chapter_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Find the DB chapter this scene belongs to
            let parsed_chapter_source_id = parsed_chapter_id_to_source
                .get(&new_scene.chapter_id)
                .ok_or_else(|| {
                let _ = conn.execute("ROLLBACK", []);
                "Scene references unknown chapter".to_string()
            })?;
            let db_chapter = chapter_source_to_db
                .get(parsed_chapter_source_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Could not find DB chapter for scene".to_string()
                })?;

            // Try to find existing scene by source_id
            if let Some(existing) = db::find_scene_by_source_id(&conn, &db_chapter.id, source_id)
                .map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?
            {
                // Update existing scene (preserving prose!)
                db::update_scene(
                    &conn,
                    &existing.id,
                    &new_scene.title,
                    new_scene.synopsis.as_deref(),
                    new_scene.position,
                )
                .map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?;
                summary.scenes_updated += 1;
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Insert new scene with DB chapter's UUID
                let scene_to_insert = Scene {
                    id: new_scene.id,
                    chapter_id: db_chapter.id,
                    title: new_scene.title.clone(),
                    synopsis: new_scene.synopsis.clone(),
                    prose: None,
                    position: new_scene.position,
                    source_id: new_scene.source_id.clone(),
                    archived: false,
                    locked: false,
                };
                db::insert_scene(&conn, &scene_to_insert).map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?;
                summary.scenes_added += 1;
            }
        }
    }

    // Build scene source_id to DB scene map
    let db_scenes = db::get_all_project_scenes(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let scene_source_to_db: std::collections::HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed scene ID to parsed scene source_id
    let parsed_scene_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // Find the DB scene this beat belongs to
            let parsed_scene_source_id = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Beat references unknown scene".to_string()
                })?;
            let db_scene = scene_source_to_db
                .get(parsed_scene_source_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Could not find DB scene for beat".to_string()
                })?;

            // Try to find existing beat by source_id
            if let Some(existing) = db::find_beat_by_source_id(&conn, &db_scene.id, source_id)
                .map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?
            {
                // Update existing beat (preserving prose!)
                db::update_beat(&conn, &existing.id, &new_beat.content, new_beat.position)
                    .map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                summary.beats_updated += 1;
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Insert new beat with DB scene's UUID
                let beat_to_insert = Beat {
                    id: new_beat.id,
                    scene_id: db_scene.id,
                    content: new_beat.content.clone(),
                    prose: None,
                    position: new_beat.position,
                    source_id: new_beat.source_id.clone(),
                };
                db::insert_beat(&conn, &beat_to_insert).map_err(|e| {
                    let _ = conn.execute("ROLLBACK", []);
                    e.to_string()
                })?;
                summary.beats_added += 1;
            }
        }
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(summary)
}

#[tauri::command]
pub async fn get_sync_preview(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<SyncPreview, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the existing project to find source path and type
    let project = db::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for sync".to_string())?;

    // Re-parse the source file based on source type
    let parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::Scrivener => {
            return Err("Scrivener sync not yet supported".to_string());
        }
        crate::models::SourceType::Markdown => {
            return Err("Markdown sync not yet supported".to_string());
        }
    };

    let mut preview = SyncPreview {
        additions: Vec::new(),
        changes: Vec::new(),
    };

    // Get existing DB data
    let db_chapters = db::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;
    let chapter_source_to_db: std::collections::HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to chapter for lookups
    let parsed_chapter_map: std::collections::HashMap<Uuid, &Chapter> =
        parsed.chapters.iter().map(|c| (c.id, c)).collect();

    // Process chapters
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            if let Some(existing) = chapter_source_to_db.get(source_id) {
                // Skip locked chapters
                if existing.locked {
                    continue;
                }
                // Check for changes
                if existing.title != new_chapter.title {
                    preview.changes.push(SyncChange {
                        id: format!("chapter-title-{}", existing.id),
                        item_type: "chapter".to_string(),
                        field: "title".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing.title.clone(),
                        new_value: new_chapter.title.clone(),
                        db_id: existing.id.to_string(),
                    });
                }
            } else {
                // New chapter
                preview.additions.push(SyncAddition {
                    id: format!("chapter-{}", source_id),
                    item_type: "chapter".to_string(),
                    title: new_chapter.title.clone(),
                    parent_title: None,
                });
            }
        }
    }

    // Get all scenes for the project
    let db_scenes = db::get_all_project_scenes(&conn, &project_uuid).map_err(|e| e.to_string())?;
    let scene_source_to_db: std::collections::HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed chapter ID to source_id
    let parsed_chapter_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Build map from parsed scene ID to scene for lookups
    let parsed_scene_map: std::collections::HashMap<Uuid, &Scene> =
        parsed.scenes.iter().map(|s| (s.id, s)).collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Get parent chapter name for context
            let parent_chapter_name = parsed_chapter_id_to_source
                .get(&new_scene.chapter_id)
                .and_then(|ch_source_id| chapter_source_to_db.get(ch_source_id))
                .map(|ch| ch.title.clone())
                .or_else(|| {
                    parsed_chapter_map
                        .get(&new_scene.chapter_id)
                        .map(|ch| ch.title.clone())
                });

            if let Some(existing) = scene_source_to_db.get(source_id) {
                // Skip locked scenes (or scenes in locked chapters)
                if existing.locked {
                    continue;
                }
                // Check if parent chapter is locked
                if let Some(ch_source_id) = parsed_chapter_id_to_source.get(&new_scene.chapter_id) {
                    if let Some(ch) = chapter_source_to_db.get(ch_source_id) {
                        if ch.locked {
                            continue;
                        }
                    }
                }
                // Check for title changes
                if existing.title != new_scene.title {
                    preview.changes.push(SyncChange {
                        id: format!("scene-title-{}", existing.id),
                        item_type: "scene".to_string(),
                        field: "title".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing.title.clone(),
                        new_value: new_scene.title.clone(),
                        db_id: existing.id.to_string(),
                    });
                }
                // Check for synopsis changes
                let existing_synopsis = existing.synopsis.clone().unwrap_or_default();
                let new_synopsis = new_scene.synopsis.clone().unwrap_or_default();
                if existing_synopsis != new_synopsis {
                    preview.changes.push(SyncChange {
                        id: format!("scene-synopsis-{}", existing.id),
                        item_type: "scene".to_string(),
                        field: "synopsis".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing_synopsis,
                        new_value: new_synopsis,
                        db_id: existing.id.to_string(),
                    });
                }
            } else {
                // New scene
                preview.additions.push(SyncAddition {
                    id: format!("scene-{}", source_id),
                    item_type: "scene".to_string(),
                    title: new_scene.title.clone(),
                    parent_title: parent_chapter_name,
                });
            }
        }
    }

    // Get all beats for the project
    let db_beats = db::get_all_project_beats(&conn, &project_uuid).map_err(|e| e.to_string())?;
    let beat_source_to_db: std::collections::HashMap<String, &Beat> = db_beats
        .iter()
        .filter_map(|b| b.source_id.as_ref().map(|sid| (sid.clone(), b)))
        .collect();

    // Build map from parsed scene ID to source_id
    let parsed_scene_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // Get parent scene name for context
            let parent_scene_name = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .and_then(|sc_source_id| scene_source_to_db.get(sc_source_id))
                .map(|sc| sc.title.clone())
                .or_else(|| {
                    parsed_scene_map
                        .get(&new_beat.scene_id)
                        .map(|sc| sc.title.clone())
                });

            // Check if parent scene is locked
            if let Some(sc_source_id) = parsed_scene_id_to_source.get(&new_beat.scene_id) {
                if let Some(sc) = scene_source_to_db.get(sc_source_id) {
                    if sc.locked {
                        continue;
                    }
                }
            }

            if let Some(existing) = beat_source_to_db.get(source_id) {
                // Check for content changes
                if existing.content != new_beat.content {
                    preview.changes.push(SyncChange {
                        id: format!("beat-content-{}", existing.id),
                        item_type: "beat".to_string(),
                        field: "content".to_string(),
                        item_title: truncate_string(&existing.content, 50),
                        current_value: existing.content.clone(),
                        new_value: new_beat.content.clone(),
                        db_id: existing.id.to_string(),
                    });
                }
            } else {
                // New beat
                preview.additions.push(SyncAddition {
                    id: format!("beat-{}", source_id),
                    item_type: "beat".to_string(),
                    title: truncate_string(&new_beat.content, 50),
                    parent_title: parent_scene_name,
                });
            }
        }
    }

    Ok(preview)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[tauri::command]
pub async fn apply_sync(
    project_id: String,
    accepted_change_ids: Vec<String>,
    accepted_addition_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<ReimportSummary, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the existing project to find source path and type
    let project = db::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for sync".to_string())?;

    // Re-parse the source file based on source type
    let parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::Scrivener => {
            return Err("Scrivener sync not yet supported".to_string());
        }
        crate::models::SourceType::Markdown => {
            return Err("Markdown sync not yet supported".to_string());
        }
    };

    let accepted_set: std::collections::HashSet<String> = accepted_change_ids.into_iter().collect();
    let accepted_additions_set: std::collections::HashSet<String> =
        accepted_addition_ids.into_iter().collect();

    let mut summary = ReimportSummary {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
    };

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| e.to_string())?;

    // Get existing DB data
    let db_chapters = db::get_chapters(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let chapter_source_to_db: std::collections::HashMap<String, Chapter> = db_chapters
        .into_iter()
        .filter_map(|c| c.source_id.clone().map(|sid| (sid, c)))
        .collect();

    // Process chapters - always add new ones, only update if change is accepted
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            if let Some(existing) = chapter_source_to_db.get(source_id) {
                // Check if user accepted the title change
                let change_id = format!("chapter-title-{}", existing.id);
                if accepted_set.contains(&change_id) && existing.title != new_chapter.title {
                    db::update_chapter(
                        &conn,
                        &existing.id,
                        &new_chapter.title,
                        new_chapter.position,
                    )
                    .map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                    summary.chapters_updated += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("chapter-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let chapter_to_insert = Chapter {
                        id: new_chapter.id,
                        project_id: project_uuid,
                        title: new_chapter.title.clone(),
                        position: new_chapter.position,
                        source_id: new_chapter.source_id.clone(),
                        archived: false,
                        locked: false,
                    };
                    db::insert_chapter(&conn, &chapter_to_insert).map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                    summary.chapters_added += 1;
                }
            }
        }
    }

    // Refresh chapter map after inserts
    let db_chapters = db::get_chapters(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let chapter_source_to_db: std::collections::HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to parsed chapter source_id
    let parsed_chapter_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Get existing scenes
    let db_scenes = db::get_all_project_scenes(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let scene_source_to_db: std::collections::HashMap<String, Scene> = db_scenes
        .into_iter()
        .filter_map(|s| s.source_id.clone().map(|sid| (sid, s)))
        .collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Find the DB chapter this scene belongs to
            let parsed_chapter_source_id = parsed_chapter_id_to_source
                .get(&new_scene.chapter_id)
                .ok_or_else(|| {
                let _ = conn.execute("ROLLBACK", []);
                "Scene references unknown chapter".to_string()
            })?;
            let db_chapter = chapter_source_to_db
                .get(parsed_chapter_source_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Could not find DB chapter for scene".to_string()
                })?;

            if let Some(existing) = scene_source_to_db.get(source_id) {
                // Check which changes user accepted
                let mut new_title = existing.title.clone();
                let mut new_synopsis = existing.synopsis.clone();
                let mut updated = false;

                let title_change_id = format!("scene-title-{}", existing.id);
                if accepted_set.contains(&title_change_id) && existing.title != new_scene.title {
                    new_title = new_scene.title.clone();
                    updated = true;
                }

                let synopsis_change_id = format!("scene-synopsis-{}", existing.id);
                if accepted_set.contains(&synopsis_change_id)
                    && existing.synopsis != new_scene.synopsis
                {
                    new_synopsis = new_scene.synopsis.clone();
                    updated = true;
                }

                if updated {
                    db::update_scene(
                        &conn,
                        &existing.id,
                        &new_title,
                        new_synopsis.as_deref(),
                        new_scene.position,
                    )
                    .map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                    summary.scenes_updated += 1;
                }
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("scene-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let scene_to_insert = Scene {
                        id: new_scene.id,
                        chapter_id: db_chapter.id,
                        title: new_scene.title.clone(),
                        synopsis: new_scene.synopsis.clone(),
                        prose: None,
                        position: new_scene.position,
                        source_id: new_scene.source_id.clone(),
                        archived: false,
                        locked: false,
                    };
                    db::insert_scene(&conn, &scene_to_insert).map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                    summary.scenes_added += 1;
                }
            }
        }
    }

    // Refresh scene map after inserts
    let db_scenes = db::get_all_project_scenes(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let scene_source_to_db: std::collections::HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed scene ID to parsed scene source_id
    let parsed_scene_id_to_source: std::collections::HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    // Get existing beats
    let db_beats = db::get_all_project_beats(&conn, &project_uuid).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    let beat_source_to_db: std::collections::HashMap<String, Beat> = db_beats
        .into_iter()
        .filter_map(|b| b.source_id.clone().map(|sid| (sid, b)))
        .collect();

    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // Find the DB scene this beat belongs to
            let parsed_scene_source_id = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Beat references unknown scene".to_string()
                })?;
            let db_scene = scene_source_to_db
                .get(parsed_scene_source_id)
                .ok_or_else(|| {
                    let _ = conn.execute("ROLLBACK", []);
                    "Could not find DB scene for beat".to_string()
                })?;

            if let Some(existing) = beat_source_to_db.get(source_id) {
                // Check if user accepted the content change
                let change_id = format!("beat-content-{}", existing.id);
                if accepted_set.contains(&change_id) && existing.content != new_beat.content {
                    db::update_beat(&conn, &existing.id, &new_beat.content, new_beat.position)
                        .map_err(|e| {
                            let _ = conn.execute("ROLLBACK", []);
                            e.to_string()
                        })?;
                    summary.beats_updated += 1;
                }
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("beat-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let beat_to_insert = Beat {
                        id: new_beat.id,
                        scene_id: db_scene.id,
                        content: new_beat.content.clone(),
                        prose: None,
                        position: new_beat.position,
                        source_id: new_beat.source_id.clone(),
                    };
                    db::insert_beat(&conn, &beat_to_insert).map_err(|e| {
                        let _ = conn.execute("ROLLBACK", []);
                        e.to_string()
                    })?;
                    summary.beats_added += 1;
                }
            }
        }
    }

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(summary)
}

// ============================================================================
// Rename Commands
// ============================================================================

#[tauri::command]
pub async fn rename_chapter(
    chapter_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if chapter is locked
    if db::is_chapter_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot rename a locked chapter".to_string());
    }

    db::rename_chapter(&conn, &uuid, &title).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn rename_scene(
    scene_id: String,
    title: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot rename a locked scene".to_string());
    }

    db::rename_scene(&conn, &uuid, &title).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

// ============================================================================
// Duplicate Commands
// ============================================================================

#[tauri::command]
pub async fn duplicate_chapter(
    chapter_id: String,
    state: State<'_, AppState>,
) -> Result<Chapter, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the original chapter
    let original = db::get_chapter_by_id(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Chapter not found".to_string())?;

    // Get the next position
    let max_pos =
        db::get_max_chapter_position(&conn, &original.project_id).map_err(|e| e.to_string())?;

    // Create new chapter with copy title
    let new_chapter = Chapter {
        id: Uuid::new_v4(),
        project_id: original.project_id,
        title: format!("{} (copy)", original.title),
        position: max_pos + 1,
        source_id: None, // Don't copy source_id
        archived: false,
        locked: false,
    };

    db::insert_chapter(&conn, &new_chapter).map_err(|e| e.to_string())?;

    // Copy all scenes from the original chapter
    let scenes = db::get_scenes(&conn, &uuid).map_err(|e| e.to_string())?;
    for scene in scenes {
        let new_scene = Scene {
            id: Uuid::new_v4(),
            chapter_id: new_chapter.id,
            title: scene.title,
            synopsis: scene.synopsis,
            prose: scene.prose,
            position: scene.position,
            source_id: None,
            archived: false,
            locked: false,
        };
        db::insert_scene(&conn, &new_scene).map_err(|e| e.to_string())?;

        // Copy beats for this scene
        let beats = db::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;
        for beat in beats {
            let new_beat = Beat {
                id: Uuid::new_v4(),
                scene_id: new_scene.id,
                content: beat.content,
                prose: beat.prose,
                position: beat.position,
                source_id: None,
            };
            db::insert_beat(&conn, &new_beat).map_err(|e| e.to_string())?;
        }
    }

    db::update_project_modified(&conn, &original.project_id).map_err(|e| e.to_string())?;

    Ok(new_chapter)
}

#[tauri::command]
pub async fn duplicate_scene(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<Scene, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the original scene
    let original = db::get_scene_by_id(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Scene not found".to_string())?;

    // Get the next position
    let max_pos =
        db::get_max_scene_position(&conn, &original.chapter_id).map_err(|e| e.to_string())?;

    // Create new scene with copy title
    let new_scene = Scene {
        id: Uuid::new_v4(),
        chapter_id: original.chapter_id,
        title: format!("{} (copy)", original.title),
        synopsis: original.synopsis,
        prose: original.prose,
        position: max_pos + 1,
        source_id: None, // Don't copy source_id
        archived: false,
        locked: false,
    };

    db::insert_scene(&conn, &new_scene).map_err(|e| e.to_string())?;

    // Copy beats from the original scene
    let beats = db::get_beats(&conn, &uuid).map_err(|e| e.to_string())?;
    for beat in beats {
        let new_beat = Beat {
            id: Uuid::new_v4(),
            scene_id: new_scene.id,
            content: beat.content,
            prose: beat.prose,
            position: beat.position,
            source_id: None,
        };
        db::insert_beat(&conn, &new_beat).map_err(|e| e.to_string())?;
    }

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &original.chapter_id).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(new_scene)
}

// ============================================================================
// Archive Commands
// ============================================================================

#[tauri::command]
pub async fn archive_chapter(chapter_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::archive_chapter(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn archive_scene(scene_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::archive_scene(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn restore_chapter(
    chapter_id: String,
    state: State<'_, AppState>,
) -> Result<Chapter, String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::restore_chapter(&conn, &uuid).map_err(|e| e.to_string())?;

    let chapter = db::get_chapter_by_id(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Chapter not found".to_string())?;

    db::update_project_modified(&conn, &chapter.project_id).map_err(|e| e.to_string())?;

    Ok(chapter)
}

#[tauri::command]
pub async fn restore_scene(scene_id: String, state: State<'_, AppState>) -> Result<Scene, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::restore_scene(&conn, &uuid).map_err(|e| e.to_string())?;

    let scene = db::get_scene_by_id(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Scene not found".to_string())?;

    // Update project modified time
    if let Some(project_id) =
        db::get_chapter_project_id(&conn, &scene.chapter_id).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(scene)
}

#[derive(serde::Serialize)]
pub struct ArchivedItems {
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
}

#[tauri::command]
pub async fn get_archived_items(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<ArchivedItems, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let chapters = db::get_archived_chapters(&conn, &uuid).map_err(|e| e.to_string())?;
    let scenes = db::get_archived_scenes(&conn, &uuid).map_err(|e| e.to_string())?;

    Ok(ArchivedItems { chapters, scenes })
}

// ============================================================================
// Lock Commands
// ============================================================================

#[tauri::command]
pub async fn lock_chapter(chapter_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::lock_chapter(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn unlock_chapter(chapter_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::unlock_chapter(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn lock_scene(scene_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::lock_scene(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn unlock_scene(scene_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::unlock_scene(&conn, &uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}
