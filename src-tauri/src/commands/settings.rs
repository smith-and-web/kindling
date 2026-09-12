//! App Settings Commands
//!
//! Handles reading and writing app-wide settings (stored in JSON file).

use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::models::AppSettings;

/// Get the path to the settings file
pub fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
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

/// Load app settings from the settings file (internal helper)
pub fn load_app_settings(app_handle: &AppHandle) -> Result<AppSettings, String> {
    let settings_path = get_settings_path(app_handle)?;

    if settings_path.exists() {
        let contents = fs::read_to_string(&settings_path).map_err(|e| e.to_string())?;
        let settings: AppSettings = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
        Ok(settings)
    } else {
        // Return default settings if file doesn't exist
        Ok(AppSettings::default())
    }
}

/// Get app settings
#[tauri::command]
pub async fn get_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    load_app_settings(&app_handle)
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

/// Preserve the unreadable file before the user explicitly resets author details.
#[tauri::command]
pub async fn reset_app_settings(app_handle: AppHandle) -> Result<AppSettings, String> {
    reset_settings_file(&get_settings_path(&app_handle)?)
}

fn reset_settings_file(path: &std::path::Path) -> Result<AppSettings, String> {
    // A read failure is not permission to overwrite the original.
    let original = fs::read(path).map_err(|e| e.to_string())?;
    let backup = path.with_file_name(format!("settings-recovery-{}.json", uuid::Uuid::new_v4()));
    use std::io::Write;
    let mut file = fs::File::create_new(backup).map_err(|e| e.to_string())?;
    file.write_all(&original).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    let settings = AppSettings::default();
    fs::write(
        path,
        serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(settings)
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn reset_preserves_malformed_bytes_and_allows_valid_settings_to_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let broken = b"{\"author_name\": ";
        fs::write(&path, broken).unwrap();
        reset_settings_file(&path).unwrap();
        let settings: AppSettings = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(settings.author_name.is_none());
        let backup = fs::read_dir(dir.path())
            .unwrap()
            .map(|p| p.unwrap().path())
            .find(|p| p != &path)
            .unwrap();
        assert_eq!(fs::read(backup).unwrap(), broken);
        assert!(reset_settings_file(&dir.path().join("missing.json")).is_err());
    }
}
