//! Shared command definitions, durable keybindings, and native menu synchronization.
use std::{collections::BTreeMap, fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

type Bindings = BTreeMap<String, String>;
#[derive(serde::Deserialize)]
struct Definition {
    id: String,
    binding: String,
}
fn defaults() -> Bindings {
    serde_json::from_str::<Vec<Definition>>(include_str!("../../src/lib/shortcutDefinitions.json"))
        .expect("valid shared shortcut definitions")
        .into_iter()
        .map(|def| (def.id, def.binding))
        .collect()
}

#[derive(Default)]
pub struct ShortcutState {
    suspended: Mutex<bool>,
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    // Match the isolated project data directory in debug QA runs.
    #[cfg(debug_assertions)]
    if let Ok(dir) = std::env::var("KINDLING_DATA_DIR") {
        if !dir.trim().is_empty() {
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            return Ok(PathBuf::from(dir).join("shortcuts.json"));
        }
    }
    Ok(crate::commands::get_settings_path(app)?.with_file_name("shortcuts.json"))
}

fn validate(bindings: &Bindings) -> Result<(), String> {
    let known = defaults();
    if bindings.keys().ne(known.keys()) {
        return Err("Shortcut commands do not match this version of Kindling".into());
    }
    let reserved = [
        "Mod+X",
        "Mod+C",
        "Mod+V",
        "Mod+Shift+V",
        "Mod+Alt+Shift+V",
        "Mod+A",
        "Mod+Z",
        "Mod+Shift+Z",
        "Mod+Y",
        "Mod+M",
        "Mod+H",
        "Mod+Alt+H",
    ];
    let mut used = BTreeMap::new();
    for (id, binding) in bindings {
        if binding.is_empty() {
            continue;
        }
        let key = binding
            .strip_prefix("Mod+")
            .ok_or("Shortcuts require Command/Ctrl")?;
        let key = key.strip_prefix("Alt+").unwrap_or(key);
        let key = key.strip_prefix("Shift+").unwrap_or(key);
        let valid_key = (key.len() == 1
            && key
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()))
            || [
                "Comma",
                "Period",
                "Slash",
                "Backslash",
                "Semicolon",
                "Quote",
                "BracketLeft",
                "BracketRight",
                "Minus",
                "Equal",
                "Backquote",
            ]
            .contains(&key)
            || (1..=24).any(|n| key == format!("F{n}"));
        if !valid_key || reserved.contains(&binding.as_str()) {
            return Err(format!("Invalid or reserved shortcut: {binding}"));
        }
        if let Some(other) = used.insert(binding, id) {
            return Err(format!("Shortcut already used by {other}"));
        }
    }
    Ok(())
}

fn load(app: &AppHandle) -> Result<Bindings, String> {
    let file = path(app)?;
    let mut bindings = defaults();
    if file.exists() {
        let saved: Bindings = serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        for (id, value) in saved {
            if bindings.contains_key(&id) {
                bindings.insert(id, value);
            }
        }
    }
    validate(&bindings)?;
    Ok(bindings)
}

fn apply(app: &AppHandle, bindings: &Bindings, suspended: bool) -> Result<(), String> {
    let empty = Bindings::new();
    crate::menu::create_menu(app, if suspended { &empty } else { bindings })
        .map_err(|e| e.to_string())
}

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    app.manage(ShortcutState::default());
    // A malformed file is reported by get_keyboard_shortcuts in the UI. Keep menus usable.
    let bindings = load(app).unwrap_or_else(|_| defaults());
    apply(app, &bindings, false)
}

#[tauri::command]
pub async fn get_keyboard_shortcuts(app: AppHandle) -> Result<Bindings, String> {
    load(&app)
}

#[tauri::command]
pub async fn set_keyboard_shortcuts(
    app: AppHandle,
    bindings: Bindings,
) -> Result<Bindings, String> {
    validate(&bindings)?;
    let state = app.state::<ShortcutState>();
    let suspended = state.suspended.lock().map_err(|e| e.to_string())?;
    let previous = load(&app).unwrap_or_else(|_| defaults());
    let file = path(&app)?;
    let temporary = file.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(&bindings).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    if let Err(error) = apply(&app, &bindings, *suspended)
        .and_then(|()| fs::rename(&temporary, &file).map_err(|e| e.to_string()))
    {
        let _ = apply(&app, &previous, *suspended);
        let _ = fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(bindings)
}

#[tauri::command]
pub async fn suspend_keyboard_shortcuts(app: AppHandle, suspended: bool) -> Result<(), String> {
    let state = app.state::<ShortcutState>();
    let mut current = state.suspended.lock().map_err(|e| e.to_string())?;
    apply(&app, &load(&app).unwrap_or_else(|_| defaults()), suspended)?;
    *current = suspended;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_valid_and_unique() {
        validate(&defaults()).unwrap();
    }
    #[test]
    fn rejects_conflicts_reserved_keys_and_invalid_input() {
        for binding in [
            "Mod+E",
            "Mod+C",
            "Q",
            "Mod+Shift+Alt+Q",
            "Mod+F25",
            "Mod+Shift+",
            "Mod+Bogus",
        ] {
            let mut bindings = defaults();
            bindings.insert("settings".into(), binding.into());
            assert!(validate(&bindings).is_err(), "{binding}");
        }
    }
    #[test]
    fn accepts_remapping_clearing_and_rejects_unknown_commands() {
        let mut bindings = defaults();
        bindings.insert("settings".into(), "Mod+Alt+Shift+F12".into());
        bindings.insert("quit".into(), "".into());
        validate(&bindings).unwrap();
        bindings.insert("unknown".into(), "".into());
        assert!(validate(&bindings).is_err());
        bindings.remove("unknown");
        bindings.remove("quit");
        assert!(validate(&bindings).is_err());
    }
}
