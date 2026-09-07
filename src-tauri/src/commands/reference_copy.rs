use super::AppState;
use crate::db::reference_copy::{self, CopyError, CopyPreview, CopyRequest, CopyResult};
use tauri::State;

#[tauri::command]
pub async fn preview_reference_copy(
    request: CopyRequest,
    state: State<'_, AppState>,
) -> Result<CopyPreview, CopyError> {
    let conn = state.db.lock().map_err(|e| CopyError {
        code: "database_error",
        message: e.to_string(),
    })?;
    reference_copy::preview(&conn, &request)
}

#[tauri::command]
pub async fn copy_references_between_projects(
    request: CopyRequest,
    expected_revision: String,
    state: State<'_, AppState>,
) -> Result<CopyResult, CopyError> {
    let conn = state.db.lock().map_err(|e| CopyError {
        code: "database_error",
        message: e.to_string(),
    })?;
    reference_copy::copy(&conn, &request, &expected_revision)
}
