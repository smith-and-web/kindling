use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::models::ReferenceSuggestion;
use crate::{db, detect};

use super::AppState;

#[tauri::command]
pub async fn detect_scene_references(
    scene_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ReferenceSuggestion>, String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let project_id = db::get_scene_project_id(&conn, &scene_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Scene not found".to_string())?;

    detect::detect_references(&conn, &project_id, &scene_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn detect_all_references(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<HashMap<String, Vec<ReferenceSuggestion>>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let result = detect::detect_all_references(&conn, &uuid).map_err(|e| e.to_string())?;

    Ok(result
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect())
}

#[tauri::command]
pub async fn dismiss_suggestion(
    scene_id: String,
    reference_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;
    let ref_uuid = Uuid::parse_str(&reference_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::dismiss_suggestion(&conn, &scene_uuid, &ref_uuid).map_err(|e| e.to_string())
}
