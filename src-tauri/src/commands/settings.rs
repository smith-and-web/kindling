//! App Settings Commands
//!
//! Handles reading and writing app-wide settings (stored in JSON file).

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::models::AppSettings;

/// Get the path to the settings file
fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    // Ensure the directory exists
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;
    }

    Ok(app_data_dir.join("settings.json"))
}

/// Get app settings
#[tauri::command]
pub async fn get_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    let settings_path = get_settings_path(&app_handle)?;

    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let settings: AppSettings = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        // Return default settings if file doesn't exist
        Ok(AppSettings::default())
    }
}

/// Update app settings
#[tauri::command]
pub async fn update_app_settings(
    app_handle: AppHandle,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let settings_path = get_settings_path(&app_handle)?;

    let contents = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(&settings_path, contents).map_err(|e| e.to_string())?;

    Ok(settings)
}
