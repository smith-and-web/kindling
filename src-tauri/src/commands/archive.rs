//! Archive Commands
//!
//! Handles archiving (soft-delete) and restoring chapters and scenes.

use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{Chapter, Scene};

use super::AppState;

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
