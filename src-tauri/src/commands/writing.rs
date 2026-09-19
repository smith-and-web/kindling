use chrono::Local;
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::db::writing::{self, WritingStats};

#[tauri::command]
pub async fn get_writing_stats(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<WritingStats, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    writing::stats(&conn, &id, Local::now().date_naive()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_daily_writing_goal(
    project_id: String,
    goal: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !(0..=1_000_000).contains(&goal) {
        return Err("Daily goal must be a whole number between 0 and 1,000,000".into());
    }
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    writing::set_goal(&conn, &id.to_string(), goal, Local::now().date_naive())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_writing_session(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    writing::reset_session(&conn, &id.to_string()).map_err(|e| e.to_string())
}
