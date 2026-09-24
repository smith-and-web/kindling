//! Lock and Part Commands
//!
//! Handles locking/unlocking chapters and scenes, and toggling Part status.

use rusqlite::Connection;
use tauri::State;
use uuid::Uuid;

use crate::db;

use super::AppState;

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

#[tauri::command]
pub async fn set_chapter_is_part(
    chapter_id: String,
    is_part: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    set_unlocked_chapter_is_part(&conn, &uuid, is_part)
}

fn set_unlocked_chapter_is_part(
    conn: &Connection,
    uuid: &Uuid,
    is_part: bool,
) -> Result<(), String> {
    if db::is_chapter_locked(conn, uuid).map_err(|e| e.to_string())? {
        return Err("Cannot convert a locked chapter. Unlock it first.".to_string());
    }

    db::set_chapter_is_part(conn, uuid, is_part).map_err(|e| e.to_string())?;

    // Update project modified time
    if let Some(project_id) = db::get_chapter_project_id(conn, uuid).map_err(|e| e.to_string())? {
        db::update_project_modified(conn, &project_id).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Chapter, Project, SourceType};

    #[test]
    fn a_locked_chapter_cannot_be_converted_to_a_part() {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = Project::new("Parts".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "One".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let is_part = |conn: &Connection| {
            db::get_chapter_by_id(conn, &chapter.id)
                .unwrap()
                .unwrap()
                .is_part
        };

        db::lock_chapter(&conn, &chapter.id).unwrap();
        assert!(set_unlocked_chapter_is_part(&conn, &chapter.id, true)
            .unwrap_err()
            .contains("locked chapter"));
        assert!(!is_part(&conn));

        db::unlock_chapter(&conn, &chapter.id).unwrap();
        set_unlocked_chapter_is_part(&conn, &chapter.id, true).unwrap();
        assert!(is_part(&conn));
    }
}
