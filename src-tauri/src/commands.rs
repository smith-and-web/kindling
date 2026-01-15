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
    db::update_beat_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())
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
