//! The writer's interface-language choice, as the native menu sees it.
//!
//! The webview owns the choice (it needs it synchronously, so it lives in localStorage next
//! to the theme) and mirrors it here with `set_interface_language` on every change and at
//! startup. Rust keeps a copy in `interface-language.json` because the menu is built before
//! the webview loads. A missing, unreadable or unknown value means "follow the system".
use std::{fs, path::Path};
use tauri::AppHandle;

const FILE_NAME: &str = "interface-language.json";

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Stored {
    locale: Option<String>,
}

fn read(path: &Path) -> Option<&'static str> {
    let stored: Stored = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    crate::menu::supported_locale(stored.locale?.as_str())
}

/// Write the choice, returning whether it changed. Writes through a temporary file so a
/// crash mid-write cannot leave a half-written preference.
fn write(path: &Path, locale: Option<&'static str>) -> Result<bool, String> {
    if read(path) == locale && (locale.is_some() || path.exists()) {
        return Ok(false);
    }
    let bytes = serde_json::to_vec_pretty(&Stored {
        locale: locale.map(str::to_string),
    })
    .map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    fs::rename(&temporary, path).map_err(|e| {
        let _ = fs::remove_file(&temporary);
        e.to_string()
    })?;
    Ok(true)
}

/// Validate a locale sent by the webview: `None` follows the system, anything else must name
/// a shipped locale exactly.
fn parse(locale: Option<String>) -> Result<Option<&'static str>, String> {
    match locale {
        None => Ok(None),
        Some(tag) => crate::menu::supported_locale(&tag)
            .map(Some)
            .ok_or_else(|| format!("Unsupported interface language: {tag}")),
    }
}

/// The language chosen in Settings, or `None` to follow the system.
pub fn chosen(app: &AppHandle) -> Option<&'static str> {
    read(&crate::commands::preference_file_path(app, FILE_NAME).ok()?)
}

/// Record the interface language (`null` follows the system) and rebuild the native menu
/// when it changed.
#[tauri::command]
pub async fn set_interface_language(app: AppHandle, locale: Option<String>) -> Result<(), String> {
    let locale = parse(locale)?;
    let path = crate::commands::preference_file_path(&app, FILE_NAME)?;
    if write(&path, locale)? {
        crate::shortcuts::refresh_menu(&app)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_system_and_shipped_locales_only() {
        assert_eq!(parse(None).unwrap(), None);
        assert_eq!(parse(Some("pt-BR".into())).unwrap(), Some("pt-BR"));
        assert!(parse(Some("pt".into())).is_err());
        assert!(parse(Some("system".into())).is_err());
        assert!(parse(Some("xx".into())).is_err());
    }

    #[test]
    fn write_round_trips_and_reports_changes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        assert_eq!(read(&path), None);
        assert!(write(&path, Some("fr")).unwrap());
        assert_eq!(read(&path), Some("fr"));
        assert!(
            !write(&path, Some("fr")).unwrap(),
            "unchanged choice skips the rebuild"
        );
        assert!(write(&path, Some("zh-CN")).unwrap());
        assert_eq!(read(&path), Some("zh-CN"));
        assert!(write(&path, None).unwrap());
        assert_eq!(read(&path), None);
        assert!(!write(&path, None).unwrap());
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn unreadable_or_unknown_files_follow_the_system() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(FILE_NAME);
        fs::write(&path, b"{\"locale\": ").unwrap();
        assert_eq!(read(&path), None);
        fs::write(&path, b"{\"locale\": \"tlh\"}").unwrap();
        assert_eq!(read(&path), None);
        // A first startup on "system" with no file records it, so later startups skip the write.
        let fresh = dir.path().join("fresh.json");
        assert!(write(&fresh, None).unwrap());
        assert!(fresh.exists());
    }
}
