//! Native open events stay queued until the frontend is ready.
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Emitter, Manager, State};

#[derive(Default)]
pub struct PendingEditorialFiles(Mutex<Vec<String>>);

pub fn supported(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        e.eq_ignore_ascii_case("kindling-review") || e.eq_ignore_ascii_case("kindling-feedback")
    })
}

pub fn enqueue(app: &tauri::AppHandle, paths: impl IntoIterator<Item = PathBuf>) {
    let pending = app.state::<PendingEditorialFiles>();
    if let Ok(mut queue) = pending.0.lock() {
        for path in paths {
            if supported(&path) {
                let path = path.to_string_lossy().to_string();
                if !queue.contains(&path) {
                    queue.push(path);
                }
            }
        }
    }
    let _ = app.emit("editorial-open", ());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[tauri::command]
pub fn take_editorial_open_files(
    state: State<'_, PendingEditorialFiles>,
) -> Result<Vec<String>, String> {
    Ok(std::mem::take(
        &mut *state.0.lock().map_err(|e| e.to_string())?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_editorial_documents_are_routed() {
        assert!(supported(Path::new("/tmp/Review.KINDLING-REVIEW")));
        assert!(supported(Path::new("/tmp/notes.kindling-feedback")));
        assert!(!supported(Path::new("--flag")));
        assert!(!supported(Path::new("review.kindling-review.exe")));
    }
}
