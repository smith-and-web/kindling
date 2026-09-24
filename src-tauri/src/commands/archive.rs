//! Archive Commands
//!
//! Handles archiving (soft-delete) and restoring chapters and scenes.

use rusqlite::Connection;
use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{Chapter, Scene};

use super::AppState;

#[tauri::command]
pub async fn archive_chapter(chapter_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    archive_unlocked_chapter(&conn, &uuid)
}

fn archive_unlocked_chapter(conn: &Connection, uuid: &Uuid) -> Result<(), String> {
    if db::is_chapter_locked(conn, uuid).map_err(|e| e.to_string())? {
        return Err("Cannot archive a locked chapter".to_string());
    }
    // Archiving the chapter takes its scenes out of the manuscript with it.
    if db::chapter_has_locked_scene(conn, uuid).map_err(|e| e.to_string())? {
        return Err(
            "Cannot archive a chapter containing a locked scene. Unlock the scene first."
                .to_string(),
        );
    }

    db::archive_chapter(conn, uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(conn, uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn archive_scene(scene_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    archive_unlocked_scene(&conn, &uuid)
}

fn archive_unlocked_scene(conn: &Connection, uuid: &Uuid) -> Result<(), String> {
    if db::is_scene_locked(conn, uuid).map_err(|e| e.to_string())? {
        return Err("Cannot archive a locked scene".to_string());
    }

    db::archive_scene(conn, uuid).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_scene_project_id(conn, uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(conn, &project_id).map_err(|e| e.to_string())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Project, SourceType};

    fn fixture() -> (Connection, Project, Chapter, Scene) {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = Project::new("Archive".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "One".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "Kept".into(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();
        (conn, project, chapter, scene)
    }

    fn archived(conn: &Connection, project: &Project) -> (usize, usize) {
        (
            db::get_archived_chapters(conn, &project.id).unwrap().len(),
            db::get_archived_scenes(conn, &project.id).unwrap().len(),
        )
    }

    #[test]
    fn locked_scenes_cannot_be_archived() {
        let (conn, project, chapter, scene) = fixture();
        db::lock_scene(&conn, &scene.id).unwrap();
        assert!(archive_unlocked_scene(&conn, &scene.id)
            .unwrap_err()
            .contains("locked scene"));
        db::unlock_scene(&conn, &scene.id).unwrap();
        db::lock_chapter(&conn, &chapter.id).unwrap();
        assert!(archive_unlocked_scene(&conn, &scene.id).is_err());
        assert_eq!(archived(&conn, &project), (0, 0));

        db::unlock_chapter(&conn, &chapter.id).unwrap();
        archive_unlocked_scene(&conn, &scene.id).unwrap();
        assert_eq!(archived(&conn, &project), (0, 1));
    }

    #[test]
    fn locked_chapters_and_chapters_holding_a_locked_scene_cannot_be_archived() {
        let (conn, project, chapter, scene) = fixture();
        db::lock_chapter(&conn, &chapter.id).unwrap();
        assert!(archive_unlocked_chapter(&conn, &chapter.id)
            .unwrap_err()
            .contains("locked chapter"));
        db::unlock_chapter(&conn, &chapter.id).unwrap();
        db::lock_scene(&conn, &scene.id).unwrap();
        assert!(archive_unlocked_chapter(&conn, &chapter.id)
            .unwrap_err()
            .contains("locked scene"));
        assert_eq!(archived(&conn, &project), (0, 0));

        db::unlock_scene(&conn, &scene.id).unwrap();
        archive_unlocked_chapter(&conn, &chapter.id).unwrap();
        assert_eq!(archived(&conn, &project).0, 1);
    }
}
