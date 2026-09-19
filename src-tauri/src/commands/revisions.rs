use super::AppState;
use crate::db::revisions::{self, ReviewData, ReviewDraft, RevisionOverview, SceneReview};
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn get_scene_review(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<SceneReview, String> {
    let id = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    revisions::load(&conn, &id)
}

#[tauri::command]
pub async fn save_scene_review(
    expected: SceneReview,
    data: ReviewData,
    next: Option<ReviewDraft>,
    state: State<'_, AppState>,
) -> Result<SceneReview, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    revisions::save(&conn, &expected, &data, next.as_ref())
}

#[tauri::command]
pub async fn get_revision_overview(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<RevisionOverview>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    revisions::overview(&conn, &id)
}
