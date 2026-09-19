use tauri::State;
use uuid::Uuid;

use crate::{db, models::SessionState};

use super::AppState;

#[tauri::command]
pub async fn get_session_state(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Option<SessionState>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_session_state(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_session_state(
    session: SessionState,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::save_session_state(&conn, &session).map_err(|e| e.to_string())
}
