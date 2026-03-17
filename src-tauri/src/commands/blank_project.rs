//! Blank Project Command
//!
//! Creates a new empty project from scratch (no import source).
//! Starts with one chapter and one Undefined scene as a starting point.

use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{
    Chapter, EditorMode, PlanningStatus, Project, Scene, SceneStatus, SceneType, SourceType,
};

use super::AppState;

#[tauri::command]
pub async fn create_blank_project(
    name: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let project_id = Uuid::new_v4();
    let chapter_id = Uuid::new_v4();

    let project = Project {
        id: project_id,
        name,
        source_type: SourceType::Blank,
        source_path: None,
        created_at: now.clone(),
        modified_at: now,
        author_pen_name: None,
        genre: None,
        description: None,
        word_target: None,
        reference_types: Project::default_reference_types(),
        project_type: Project::default_project_type(),
        target_page_count: None,
    };

    let chapter = Chapter {
        id: chapter_id,
        project_id,
        title: "Chapter 1".to_string(),
        position: 0,
        source_id: None,
        archived: false,
        locked: false,
        is_part: false,
        synopsis: None,
        planning_status: PlanningStatus::Undefined,
    };

    let scene = Scene {
        id: Uuid::new_v4(),
        chapter_id,
        title: "Scene 1".to_string(),
        synopsis: None,
        prose: None,
        position: 0,
        source_id: None,
        archived: false,
        locked: false,
        scene_type: SceneType::Normal,
        scene_status: SceneStatus::Draft,
        planning_status: PlanningStatus::Undefined,
        editor_mode: EditorMode::Beat,
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &project).map_err(|e| e.to_string())?;
    db::insert_chapter(&tx, &chapter).map_err(|e| e.to_string())?;
    db::insert_scene(&tx, &scene).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(project)
}
