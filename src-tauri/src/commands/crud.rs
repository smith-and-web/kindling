//! CRUD Commands
//!
//! Handles create, read, update, delete operations for all data types.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::db;
use crate::models::{
    Beat, Chapter, Character, DiscoveryNote, Location, PlanningStatus, Project, ReferenceItem,
    Scene, SceneReferenceState, SceneStatus, SceneType, SourceType,
};

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

#[tauri::command]
pub async fn get_all_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_projects(&conn).map_err(|e| e.to_string())
}

/// Input type for updating project settings (pen name and genre)
#[derive(serde::Deserialize)]
pub struct ProjectSettingsUpdate {
    pub author_pen_name: Option<String>,
    pub genre: Option<String>,
    pub description: Option<String>,
    pub word_target: Option<i32>,
    pub reference_types: Option<Vec<String>>,
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
    if let Some(reference_types) = settings.reference_types {
        project.reference_types = reference_types;
    }

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
        synopsis: None,
        planning_status: PlanningStatus::Fixed,
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
        source_id: None,
        archived: false,
        locked: false,
        is_part: original.is_part,
        synopsis: original.synopsis.clone(),
        planning_status: original.planning_status,
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
            scene_type: scene.scene_type,
            scene_status: scene.scene_status,
            planning_status: PlanningStatus::Fixed,
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

    // Blank projects: new scenes default to Undefined; imported projects default to Fixed
    let planning_status = if let Some(project_id) =
        db::get_chapter_project_id(&conn, &chapter_uuid).map_err(|e| e.to_string())?
    {
        if let Some(project) = db::get_project(&conn, &project_id).map_err(|e| e.to_string())? {
            match project.source_type {
                SourceType::Blank => PlanningStatus::Undefined,
                _ => PlanningStatus::Fixed,
            }
        } else {
            PlanningStatus::Fixed
        }
    } else {
        PlanningStatus::Fixed
    };

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
        scene_type: SceneType::Normal,
        scene_status: SceneStatus::Draft,
        planning_status,
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

#[derive(serde::Deserialize)]
pub struct SceneMetadataUpdate {
    pub scene_type: String,
    pub scene_status: String,
}

#[tauri::command]
pub async fn update_scene_metadata(
    scene_id: String,
    metadata: SceneMetadataUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Check if scene is locked
    if db::is_scene_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot edit a locked scene".to_string());
    }

    let scene_type = SceneType::parse(&metadata.scene_type);
    let scene_status = SceneStatus::parse(&metadata.scene_status);

    db::update_scene_metadata(&conn, &uuid, &scene_type, &scene_status)
        .map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_scene_planning_status(
    scene_id: String,
    planning_status: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let status = PlanningStatus::parse(&planning_status);

    db::update_scene_planning_status(&conn, &uuid, &status).map_err(|e| e.to_string())?;

    if let Some(project_id) = db::get_scene_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_chapter_planning_status(
    chapter_id: String,
    planning_status: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let status = PlanningStatus::parse(&planning_status);

    db::update_chapter_planning_status(&conn, &uuid, &status).map_err(|e| e.to_string())?;

    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn update_chapter_synopsis(
    chapter_id: String,
    synopsis: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    if db::is_chapter_locked(&conn, &uuid).map_err(|e| e.to_string())? {
        return Err("Cannot edit a locked chapter".to_string());
    }

    db::update_chapter_synopsis(&conn, &uuid, synopsis.as_deref()).map_err(|e| e.to_string())?;

    if let Some(project_id) = db::get_chapter_project_id(&conn, &uuid).map_err(|e| e.to_string())? {
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
        source_id: None,
        archived: false,
        locked: false,
        scene_type: original.scene_type,
        scene_status: original.scene_status,
        planning_status: original.planning_status,
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

#[tauri::command]
pub async fn delete_beat(beat_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let beat_uuid = Uuid::parse_str(&beat_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let beat = db::get_beat(&conn, &beat_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Beat not found".to_string())?;

    if db::is_scene_locked(&conn, &beat.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot delete beats in a locked scene".to_string());
    }

    // If beat has prose, merge into previous beat
    if let Some(ref prose) = beat.prose {
        let beats = db::get_beats(&conn, &beat.scene_id).map_err(|e| e.to_string())?;
        if let Some(prev_idx) = beats
            .iter()
            .position(|b| b.id == beat_uuid)
            .and_then(|i| i.checked_sub(1))
        {
            let prev = &beats[prev_idx];
            let merged = match &prev.prose {
                Some(p) => format!("{}<p></p>{}", p, prose),
                None => prose.clone(),
            };
            db::update_beat_prose(&conn, &prev.id, &merged).map_err(|e| e.to_string())?;
        }
    }

    db::delete_beat(&conn, &beat_uuid).map_err(|e| e.to_string())?;

    // Rebase positions of beats after the deleted one
    let beats = db::get_beats(&conn, &beat.scene_id).map_err(|e| e.to_string())?;
    for (i, b) in beats.iter().enumerate() {
        if b.position != i as i32 {
            db::update_beat_position(&conn, &b.id, i as i32).map_err(|e| e.to_string())?;
        }
    }

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &beat.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn reorder_beats(
    scene_id: String,
    beat_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    if db::is_scene_locked(&conn, &scene_uuid).map_err(|e| e.to_string())? {
        return Err("Cannot reorder beats in a locked scene".to_string());
    }

    let existing_beats = db::get_beats(&conn, &scene_uuid).map_err(|e| e.to_string())?;
    let existing_ids: std::collections::HashSet<_> = existing_beats.iter().map(|b| b.id).collect();

    if beat_ids.len() != existing_beats.len() {
        return Err("Beat order must include all beats in the scene".to_string());
    }

    for (position, id_str) in beat_ids.iter().enumerate() {
        let beat_uuid = Uuid::parse_str(id_str).map_err(|e| e.to_string())?;
        if !existing_ids.contains(&beat_uuid) {
            return Err(format!("Beat {} does not belong to this scene", id_str));
        }
        db::update_beat_position(&conn, &beat_uuid, position as i32).map_err(|e| e.to_string())?;
    }

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &scene_uuid).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

/// Find the character offset of the start of the nth <p> tag (0-indexed) in HTML.
fn find_paragraph_offset(html: &str, paragraph_index: u32) -> Option<usize> {
    let mut count = 0u32;
    let mut search_start = 0;
    while let Some(start) = html[search_start..].find("<p") {
        let abs_start = search_start + start;
        if count == paragraph_index {
            return Some(abs_start);
        }
        count += 1;
        search_start = abs_start + 1;
    }
    None
}

#[tauri::command]
pub async fn split_beat(
    beat_id: String,
    split_at: Option<usize>,
    split_before_paragraph: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Beat, String> {
    let beat_uuid = Uuid::parse_str(&beat_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let beat = db::get_beat(&conn, &beat_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Beat not found".to_string())?;

    if db::is_scene_locked(&conn, &beat.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot split beats in a locked scene".to_string());
    }

    let prose = beat.prose.as_deref().unwrap_or("");
    if prose.trim().is_empty() {
        return Err("Cannot split beat with no prose".to_string());
    }
    let len = prose.chars().count();

    let split_pos = match (split_at, split_before_paragraph) {
        (Some(pos), None) => pos,
        (None, Some(para_idx)) => find_paragraph_offset(prose, para_idx)
            .ok_or_else(|| "Paragraph not found for split".to_string())?,
        _ => return Err("Provide either split_at or split_before_paragraph".to_string()),
    };

    if split_pos == 0 || split_pos >= len {
        return Err("Split position must be between 0 and prose length".to_string());
    }

    let prose_before: String = prose.chars().take(split_pos).collect();
    let prose_after: String = prose.chars().skip(split_pos).collect();
    let prose_after = prose_after.trim();

    if prose_after.is_empty() {
        return Err("Nothing to split off - split position is at end of content".to_string());
    }

    db::update_beat_prose(&conn, &beat.id, prose_before.trim()).map_err(|e| e.to_string())?;

    let new_position = beat.position + 1;
    db::shift_beat_positions(&conn, &beat.scene_id, new_position).map_err(|e| e.to_string())?;

    let new_beat = Beat {
        id: Uuid::new_v4(),
        scene_id: beat.scene_id,
        content: String::new(),
        prose: Some(prose_after.to_string()),
        position: new_position,
        source_id: None,
    };
    db::insert_beat(&conn, &new_beat).map_err(|e| e.to_string())?;

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &beat.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(new_beat)
}

#[tauri::command]
pub async fn merge_beats(
    first_beat_id: String,
    second_beat_id: String,
    state: State<'_, AppState>,
) -> Result<Beat, String> {
    let first_uuid = Uuid::parse_str(&first_beat_id).map_err(|e| e.to_string())?;
    let second_uuid = Uuid::parse_str(&second_beat_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let first = db::get_beat(&conn, &first_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "First beat not found".to_string())?;
    let second = db::get_beat(&conn, &second_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Second beat not found".to_string())?;

    if first.scene_id != second.scene_id {
        return Err("Beats must be in the same scene".to_string());
    }
    if first.position + 1 != second.position {
        return Err("Beats must be adjacent (merge second into first)".to_string());
    }
    if db::is_scene_locked(&conn, &first.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot merge beats in a locked scene".to_string());
    }

    let merged_content = if second.content.is_empty() {
        first.content.clone()
    } else if first.content.is_empty() {
        second.content.clone()
    } else {
        format!("{} / {}", first.content, second.content)
    };

    let merged_prose = match (&first.prose, &second.prose) {
        (Some(a), Some(b)) => format!("{}<p></p>{}", a, b),
        (Some(a), None) => a.clone(),
        (None, Some(b)) => b.clone(),
        (None, None) => String::new(),
    };

    db::update_beat(&conn, &first.id, &merged_content, first.position)
        .map_err(|e| e.to_string())?;
    db::update_beat_prose(&conn, &first.id, &merged_prose).map_err(|e| e.to_string())?;
    db::delete_beat(&conn, &second.id).map_err(|e| e.to_string())?;

    let beats = db::get_beats(&conn, &first.scene_id).map_err(|e| e.to_string())?;
    for (i, b) in beats.iter().enumerate() {
        if b.position != i as i32 {
            db::update_beat_position(&conn, &b.id, i as i32).map_err(|e| e.to_string())?;
        }
    }

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &first.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    let mut result = first.clone();
    result.content = merged_content;
    result.prose = if merged_prose.is_empty() {
        None
    } else {
        Some(merged_prose)
    };
    Ok(result)
}

// ============================================================================
// Discovery Note Commands
// ============================================================================

#[tauri::command]
pub async fn get_discovery_notes(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveryNote>, String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_discovery_notes(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_discovery_note(
    scene_id: String,
    content: String,
    tags: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<DiscoveryNote, String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    if db::is_scene_locked(&conn, &scene_uuid).map_err(|e| e.to_string())? {
        return Err("Cannot add discovery notes to a locked scene".to_string());
    }

    let max_pos: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), -1) FROM discovery_notes WHERE scene_id = ?1",
            rusqlite::params![scene_uuid.to_string()],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    let note = DiscoveryNote {
        id: Uuid::new_v4(),
        scene_id: scene_uuid,
        content: content.trim().to_string(),
        tags: tags.unwrap_or_default(),
        position: max_pos + 1,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    db::insert_discovery_note(&conn, &note).map_err(|e| e.to_string())?;

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &scene_uuid).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(note)
}

#[tauri::command]
pub async fn update_discovery_note(
    note_id: String,
    content: String,
    tags: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<DiscoveryNote, String> {
    let note_uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let note = db::get_discovery_note(&conn, &note_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Discovery note not found".to_string())?;

    if db::is_scene_locked(&conn, &note.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot edit discovery notes in a locked scene".to_string());
    }

    let tags = tags.unwrap_or_else(|| note.tags.clone());
    db::update_discovery_note(&conn, &note_uuid, content.trim(), &tags)
        .map_err(|e| e.to_string())?;

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &note.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    let mut updated = note;
    updated.content = content.trim().to_string();
    updated.tags = tags;
    Ok(updated)
}

#[tauri::command]
pub async fn delete_discovery_note(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let note_uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let note = db::get_discovery_note(&conn, &note_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Discovery note not found".to_string())?;

    if db::is_scene_locked(&conn, &note.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot delete discovery notes in a locked scene".to_string());
    }

    db::delete_discovery_note(&conn, &note_uuid).map_err(|e| e.to_string())?;

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &note.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn promote_discovery_note_to_beat(
    note_id: String,
    state: State<'_, AppState>,
) -> Result<Beat, String> {
    let note_uuid = Uuid::parse_str(&note_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let note = db::get_discovery_note(&conn, &note_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Discovery note not found".to_string())?;

    if db::is_scene_locked(&conn, &note.scene_id).map_err(|e| e.to_string())? {
        return Err("Cannot promote notes in a locked scene".to_string());
    }

    let max_pos = db::get_max_beat_position(&conn, &note.scene_id).map_err(|e| e.to_string())?;
    let beat = Beat::new(note.scene_id, note.content.clone(), max_pos + 1);
    db::insert_beat(&conn, &beat).map_err(|e| e.to_string())?;
    db::delete_discovery_note(&conn, &note_uuid).map_err(|e| e.to_string())?;

    if let Some(project_id) =
        db::get_scene_project_id(&conn, &note.scene_id).map_err(|e| e.to_string())?
    {
        let _ = db::update_project_modified(&conn, &project_id);
    }

    Ok(beat)
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
// Reference Commands
// ============================================================================

#[derive(serde::Deserialize)]
pub struct ReferenceUpsert {
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

fn character_to_reference(character: Character) -> ReferenceItem {
    ReferenceItem {
        id: character.id,
        project_id: character.project_id,
        reference_type: "characters".to_string(),
        name: character.name,
        description: character.description,
        attributes: character.attributes,
        source_id: character.source_id,
    }
}

fn location_to_reference(location: Location) -> ReferenceItem {
    ReferenceItem {
        id: location.id,
        project_id: location.project_id,
        reference_type: "locations".to_string(),
        name: location.name,
        description: location.description,
        attributes: location.attributes,
        source_id: location.source_id,
    }
}

#[tauri::command]
pub async fn get_references(
    project_id: String,
    reference_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReferenceItem>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    match reference_type.as_str() {
        "characters" => db::get_characters(&conn, &uuid)
            .map(|items| items.into_iter().map(character_to_reference).collect())
            .map_err(|e| e.to_string()),
        "locations" => db::get_locations(&conn, &uuid)
            .map(|items| items.into_iter().map(location_to_reference).collect())
            .map_err(|e| e.to_string()),
        _ => db::get_reference_items(&conn, &uuid, &reference_type).map_err(|e| e.to_string()),
    }
}

#[tauri::command]
pub async fn get_scene_reference_items(
    scene_id: String,
    reference_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReferenceItem>, String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::get_scene_reference_items(&conn, &scene_uuid, &reference_type).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct SceneReferenceStateUpdate {
    pub reference_id: String,
    pub position: i32,
    pub expanded: bool,
}

#[tauri::command]
pub async fn get_scene_reference_state(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SceneReferenceState>, String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::cleanup_scene_reference_state(&conn, &scene_uuid).map_err(|e| e.to_string())?;

    let mut states =
        db::get_scene_reference_states(&conn, &scene_uuid).map_err(|e| e.to_string())?;
    if states.is_empty() {
        states = db::build_default_scene_reference_state(&conn, &scene_uuid)
            .map_err(|e| e.to_string())?;
    }

    Ok(states)
}

#[tauri::command]
pub async fn save_scene_reference_state(
    scene_id: String,
    reference_type: String,
    states: Vec<SceneReferenceStateUpdate>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    db::delete_scene_reference_states_for_type(&tx, &scene_uuid, &reference_type)
        .map_err(|e| e.to_string())?;

    let mut reference_ids = Vec::new();
    for state_update in &states {
        let reference_id =
            Uuid::parse_str(&state_update.reference_id).map_err(|e| e.to_string())?;
        reference_ids.push(reference_id);
        let state = SceneReferenceState {
            scene_id: scene_uuid,
            reference_type: reference_type.clone(),
            reference_id,
            position: state_update.position,
            expanded: state_update.expanded,
        };
        db::insert_scene_reference_state(&tx, &state).map_err(|e| e.to_string())?;
    }

    match reference_type.as_str() {
        "characters" => {
            db::clear_scene_character_refs(&tx, &scene_uuid).map_err(|e| e.to_string())?;
            for reference_id in reference_ids {
                db::add_scene_character_ref(&tx, &scene_uuid, &reference_id)
                    .map_err(|e| e.to_string())?;
            }
        }
        "locations" => {
            db::clear_scene_location_refs(&tx, &scene_uuid).map_err(|e| e.to_string())?;
            for reference_id in reference_ids {
                db::add_scene_location_ref(&tx, &scene_uuid, &reference_id)
                    .map_err(|e| e.to_string())?;
            }
        }
        _ => {
            db::clear_scene_reference_item_refs_for_type(&tx, &scene_uuid, &reference_type)
                .map_err(|e| e.to_string())?;
            for reference_id in reference_ids {
                db::add_scene_reference_item_ref(&tx, &scene_uuid, &reference_id)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    if let Some(project_id) =
        db::get_scene_project_id(&tx, &scene_uuid).map_err(|e| e.to_string())?
    {
        db::update_project_modified(&tx, &project_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct ReferenceReclassification {
    pub reference_id: String,
    pub new_type: String,
}

#[tauri::command]
pub async fn reclassify_references(
    project_id: String,
    changes: Vec<ReferenceReclassification>,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let result: Result<Project, String> = (|| {
        for change in &changes {
            let reference_uuid =
                Uuid::parse_str(&change.reference_id).map_err(|e| e.to_string())?;
            let target_type = change.new_type.trim().to_lowercase();
            if target_type.is_empty() {
                continue;
            }

            let current_character =
                db::get_character_by_id(&tx, &reference_uuid).map_err(|e| e.to_string())?;
            let current_location =
                db::get_location_by_id(&tx, &reference_uuid).map_err(|e| e.to_string())?;
            let current_reference_item =
                db::get_reference_item_by_id(&tx, &reference_uuid).map_err(|e| e.to_string())?;

            let (current_type, scene_states) = if current_character.is_some() {
                let states = db::get_scene_reference_states_for_reference(
                    &conn,
                    "characters",
                    &reference_uuid,
                )
                .map_err(|e| e.to_string())?;
                ("characters".to_string(), states)
            } else if current_location.is_some() {
                let states = db::get_scene_reference_states_for_reference(
                    &conn,
                    "locations",
                    &reference_uuid,
                )
                .map_err(|e| e.to_string())?;
                ("locations".to_string(), states)
            } else if let Some(item) = &current_reference_item {
                let states = db::get_scene_reference_states_for_reference(
                    &conn,
                    &item.reference_type,
                    &reference_uuid,
                )
                .map_err(|e| e.to_string())?;
                (item.reference_type.clone(), states)
            } else {
                continue;
            };

            if current_type == target_type {
                continue;
            }

            let scene_ids = if current_type == "characters" {
                db::get_scene_ids_for_character(&tx, &reference_uuid).map_err(|e| e.to_string())?
            } else if current_type == "locations" {
                db::get_scene_ids_for_location(&tx, &reference_uuid).map_err(|e| e.to_string())?
            } else {
                db::get_scene_ids_for_reference_item(&tx, &reference_uuid)
                    .map_err(|e| e.to_string())?
            };

            if !scene_states.is_empty() {
                db::delete_scene_reference_states_for_reference(
                    &conn,
                    &current_type,
                    &reference_uuid,
                )
                .map_err(|e| e.to_string())?;
                for state in &scene_states {
                    let max_position = db::get_scene_reference_state_max_position(
                        &conn,
                        &state.scene_id,
                        &target_type,
                    )
                    .map_err(|e| e.to_string())?
                    .unwrap_or(-1);
                    let next_state = SceneReferenceState {
                        scene_id: state.scene_id,
                        reference_type: target_type.clone(),
                        reference_id: reference_uuid,
                        position: max_position + 1,
                        expanded: state.expanded,
                    };
                    db::insert_scene_reference_state(&tx, &next_state)
                        .map_err(|e| e.to_string())?;
                }
            }

            match current_type.as_str() {
                "characters" => {
                    let character = current_character.expect("Character missing");
                    match target_type.as_str() {
                        "locations" => {
                            let location = Location {
                                id: character.id,
                                project_id: character.project_id,
                                name: character.name,
                                description: character.description,
                                attributes: character.attributes,
                                source_id: character.source_id,
                            };
                            db::insert_location(&tx, &location).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_location_ref(&tx, scene_id, &location.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_character(&tx, &character.id).map_err(|e| e.to_string())?;
                        }
                        "items" | "objectives" | "organizations" => {
                            let item = ReferenceItem {
                                id: character.id,
                                project_id: character.project_id,
                                reference_type: target_type.clone(),
                                name: character.name,
                                description: character.description,
                                attributes: character.attributes,
                                source_id: character.source_id,
                            };
                            db::insert_reference_item(&tx, &item).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_reference_item_ref(&tx, scene_id, &item.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_character(&tx, &character.id).map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                    db::delete_scene_character_refs_for_character(&tx, &reference_uuid)
                        .map_err(|e| e.to_string())?;
                }
                "locations" => {
                    let location = current_location.expect("Location missing");
                    match target_type.as_str() {
                        "characters" => {
                            let character = Character {
                                id: location.id,
                                project_id: location.project_id,
                                name: location.name,
                                description: location.description,
                                attributes: location.attributes,
                                source_id: location.source_id,
                            };
                            db::insert_character(&tx, &character).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_character_ref(&tx, scene_id, &character.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_location(&tx, &location.id).map_err(|e| e.to_string())?;
                        }
                        "items" | "objectives" | "organizations" => {
                            let item = ReferenceItem {
                                id: location.id,
                                project_id: location.project_id,
                                reference_type: target_type.clone(),
                                name: location.name,
                                description: location.description,
                                attributes: location.attributes,
                                source_id: location.source_id,
                            };
                            db::insert_reference_item(&tx, &item).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_reference_item_ref(&tx, scene_id, &item.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_location(&tx, &location.id).map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                    db::delete_scene_location_refs_for_location(&tx, &reference_uuid)
                        .map_err(|e| e.to_string())?;
                }
                _ => {
                    let item = current_reference_item.expect("Reference item missing");
                    match target_type.as_str() {
                        "characters" => {
                            let character = Character {
                                id: item.id,
                                project_id: item.project_id,
                                name: item.name,
                                description: item.description,
                                attributes: item.attributes,
                                source_id: item.source_id,
                            };
                            db::insert_character(&tx, &character).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_character_ref(&tx, scene_id, &character.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_reference_item(&tx, &item.id).map_err(|e| e.to_string())?;
                            db::delete_scene_reference_item_refs_for_item(&tx, &reference_uuid)
                                .map_err(|e| e.to_string())?;
                        }
                        "locations" => {
                            let location = Location {
                                id: item.id,
                                project_id: item.project_id,
                                name: item.name,
                                description: item.description,
                                attributes: item.attributes,
                                source_id: item.source_id,
                            };
                            db::insert_location(&tx, &location).map_err(|e| e.to_string())?;
                            for scene_id in &scene_ids {
                                db::add_scene_location_ref(&tx, scene_id, &location.id)
                                    .map_err(|e| e.to_string())?;
                            }
                            db::delete_reference_item(&tx, &item.id).map_err(|e| e.to_string())?;
                            db::delete_scene_reference_item_refs_for_item(&tx, &reference_uuid)
                                .map_err(|e| e.to_string())?;
                        }
                        "items" | "objectives" | "organizations" => {
                            db::update_reference_item_type(&tx, &item.id, &target_type)
                                .map_err(|e| e.to_string())?;
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut project = db::get_project(&tx, &project_uuid)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Project not found".to_string())?;

        let mut types = HashSet::new();
        let characters = db::get_characters(&tx, &project_uuid).map_err(|e| e.to_string())?;
        if !characters.is_empty() {
            types.insert("characters".to_string());
        }
        let locations = db::get_locations(&tx, &project_uuid).map_err(|e| e.to_string())?;
        if !locations.is_empty() {
            types.insert("locations".to_string());
        }
        let reference_items =
            db::get_all_reference_items(&tx, &project_uuid).map_err(|e| e.to_string())?;
        for item in reference_items {
            types.insert(item.reference_type);
        }
        if types.is_empty() {
            types.extend(Project::default_reference_types());
        }
        project.reference_types = types.into_iter().collect();

        db::update_project(&tx, &project).map_err(|e| e.to_string())?;
        db::update_project_modified(&tx, &project_uuid).map_err(|e| e.to_string())?;

        Ok(project)
    })();

    match result {
        Ok(project) => {
            tx.commit().map_err(|e| e.to_string())?;
            Ok(project)
        }
        Err(e) => Err(e),
    }
}

#[tauri::command]
pub async fn create_reference(
    project_id: String,
    reference_type: String,
    reference: ReferenceUpsert,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let attributes = reference.attributes.unwrap_or_default();

    match reference_type.as_str() {
        "characters" => {
            let character =
                Character::new(project_uuid, reference.name, reference.description, None)
                    .with_attributes(attributes);
            db::insert_character(&conn, &character).map_err(|e| e.to_string())?;
        }
        "locations" => {
            let location = Location::new(project_uuid, reference.name, reference.description, None)
                .with_attributes(attributes);
            db::insert_location(&conn, &location).map_err(|e| e.to_string())?;
        }
        _ => {
            let item = ReferenceItem::new(
                project_uuid,
                reference_type,
                reference.name,
                reference.description,
                None,
            )
            .with_attributes(attributes);
            db::insert_reference_item(&conn, &item).map_err(|e| e.to_string())?;
        }
    }

    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn update_reference(
    reference_id: String,
    reference_type: String,
    reference: ReferenceUpsert,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let reference_uuid = Uuid::parse_str(&reference_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let attributes = reference.attributes.unwrap_or_default();

    let project_id = match reference_type.as_str() {
        "characters" => {
            db::update_character(
                &conn,
                &reference_uuid,
                &reference.name,
                reference.description.as_deref(),
                &attributes,
            )
            .map_err(|e| e.to_string())?;
            db::get_character_project_id(&conn, &reference_uuid).map_err(|e| e.to_string())?
        }
        "locations" => {
            db::update_location(
                &conn,
                &reference_uuid,
                &reference.name,
                reference.description.as_deref(),
                &attributes,
            )
            .map_err(|e| e.to_string())?;
            db::get_location_project_id(&conn, &reference_uuid).map_err(|e| e.to_string())?
        }
        _ => {
            db::update_reference_item(
                &conn,
                &reference_uuid,
                &reference.name,
                reference.description.as_deref(),
                &attributes,
            )
            .map_err(|e| e.to_string())?;
            db::get_reference_item_project_id(&conn, &reference_uuid).map_err(|e| e.to_string())?
        }
    };

    if let Some(project_id) = project_id {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_reference(
    reference_id: String,
    reference_type: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let reference_uuid = Uuid::parse_str(&reference_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let project_id = match reference_type.as_str() {
        "characters" => {
            let project_id =
                db::get_character_project_id(&conn, &reference_uuid).map_err(|e| e.to_string())?;
            db::delete_character(&conn, &reference_uuid).map_err(|e| e.to_string())?;
            project_id
        }
        "locations" => {
            let project_id =
                db::get_location_project_id(&conn, &reference_uuid).map_err(|e| e.to_string())?;
            db::delete_location(&conn, &reference_uuid).map_err(|e| e.to_string())?;
            project_id
        }
        _ => {
            let project_id = db::get_reference_item_project_id(&conn, &reference_uuid)
                .map_err(|e| e.to_string())?;
            db::delete_reference_item(&conn, &reference_uuid).map_err(|e| e.to_string())?;
            project_id
        }
    };

    db::delete_scene_reference_states_for_reference(&conn, &reference_type, &reference_uuid)
        .map_err(|e| e.to_string())?;

    if let Some(project_id) = project_id {
        db::update_project_modified(&conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
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
