//! CRUD Commands
//!
//! Handles create, read, update, delete operations for all data types.

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::db;
use crate::models::{Beat, Chapter, Character, Location, Project, Scene};

use super::AppState;

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

/// Input type for updating project settings (pen name and genre)
#[derive(serde::Deserialize)]
pub struct ProjectSettingsUpdate {
    pub author_pen_name: Option<String>,
    pub genre: Option<String>,
    pub description: Option<String>,
    pub word_target: Option<i32>,
}

#[tauri::command]
pub async fn update_project_settings(
    project_id: String,
    settings: ProjectSettingsUpdate,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get the existing project
    let mut project = db::get_project(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    // Update the project-specific fields
    project.author_pen_name = settings.author_pen_name;
    project.genre = settings.genre;
    project.description = settings.description;
    project.word_target = settings.word_target;

    // Update modified timestamp
    project.modified_at = chrono::Utc::now().to_rfc3339();

    // Save to database
    db::update_project(&conn, &project).map_err(|e| e.to_string())?;

    Ok(project)
}

/// Delete a project and all its associated data including snapshot files
#[tauri::command]
pub async fn delete_project(
    project_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;

    // Delete snapshot files from disk before deleting from database
    let snapshots_dir: PathBuf = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("snapshots")
        .join(&project_id);

    if snapshots_dir.exists() {
        let _ = fs::remove_dir_all(&snapshots_dir);
    }

    // Delete project from database (cascades to all related tables)
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_project(&conn, &uuid).map_err(|e| e.to_string())?;

    Ok(())
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
    is_part: Option<bool>,
    after_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Chapter, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Determine position based on after_id or append to end
    let position = if let Some(ref after_chapter_id) = after_id {
        let after_uuid = Uuid::parse_str(after_chapter_id).map_err(|e| e.to_string())?;
        let after_chapter = db::get_chapter_by_id(&conn, &after_uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Chapter not found: {}", after_chapter_id))?;

        // Insert after the specified chapter
        let new_position = after_chapter.position + 1;

        // Shift all chapters at or after this position
        db::shift_chapters_after_position(&conn, &project_uuid, new_position)
            .map_err(|e| e.to_string())?;

        new_position
    } else {
        // Append to end
        db::get_max_chapter_position(&conn, &project_uuid).map_err(|e| e.to_string())? + 1
    };

    let chapter = Chapter {
        id: Uuid::new_v4(),
        project_id: project_uuid,
        title,
        position,
        source_id: None,
        archived: false,
        locked: false,
        is_part: is_part.unwrap_or(false),
    };

    db::insert_chapter(&conn, &chapter).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(chapter)
}

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
        is_part: original.is_part,
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

    db::update_scene_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
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
// Beat Commands
// ============================================================================

#[tauri::command]
pub async fn get_beats(scene_id: String, state: State<'_, AppState>) -> Result<Vec<Beat>, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_beats(&conn, &uuid).map_err(|e| e.to_string())
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

    if let Some(scene_id) = scene_id.as_ref() {
        if db::is_scene_locked(&conn, scene_id).map_err(|e| e.to_string())? {
            return Err("Cannot edit beats in a locked scene".to_string());
        }
    }

    db::update_beat_prose(&conn, &uuid, &prose).map_err(|e| e.to_string())?;

    if let Some(scene_id) = scene_id {
        if let Some(project_id) =
            db::get_scene_project_id(&conn, &scene_id).map_err(|e| e.to_string())?
        {
            let _ = db::update_project_modified(&conn, &project_id);
        }
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
