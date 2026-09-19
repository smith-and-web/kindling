//! Small, isolated support for the export workspace prototype. No public plugin API.
use crate::{db, models::SceneType};
use serde::Serialize;
use std::{io::Write, path::Path};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize)]
pub struct PreviewBlock {
    heading: Option<String>,
    html: String,
}
#[derive(Serialize)]
pub struct PreviewScene {
    id: Uuid,
    title: String,
    synopsis: Option<String>,
    blocks: Vec<PreviewBlock>,
}
#[derive(Serialize)]
pub struct PreviewChapter {
    id: Uuid,
    title: String,
    part: Option<String>,
    scenes: Vec<PreviewScene>,
}

#[tauri::command]
pub fn get_export_prototype_document(
    project_id: String,
    state: State<'_, super::AppState>,
) -> Result<Vec<PreviewChapter>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    let mut part = None;
    for chapter in db::get_chapters(&tx, &id).map_err(|e| e.to_string())? {
        if chapter.archived {
            continue;
        }
        if chapter.is_part {
            part = Some(chapter.title);
            continue;
        }
        let mut scenes = Vec::new();
        for scene in db::get_scenes(&tx, &chapter.id).map_err(|e| e.to_string())? {
            if scene.archived || scene.scene_type != SceneType::Normal {
                continue;
            }
            let beats = db::get_beats(&tx, &scene.id).map_err(|e| e.to_string())?;
            let page = db::writing::uses_page_prose(&scene, &beats);
            let mut blocks = Vec::new();
            if page {
                blocks.push(PreviewBlock {
                    heading: None,
                    html: scene.prose.unwrap_or_default(),
                });
            }
            for beat in beats {
                blocks.push(PreviewBlock {
                    heading: Some(beat.content),
                    html: if page {
                        String::new()
                    } else {
                        beat.prose.unwrap_or_default()
                    },
                });
            }
            scenes.push(PreviewScene {
                id: scene.id,
                title: scene.title,
                synopsis: scene.synopsis,
                blocks,
            });
        }
        result.push(PreviewChapter {
            id: chapter.id,
            title: chapter.title,
            part: part.clone(),
            scenes,
        });
    }
    Ok(result)
}

/// Save only a new HTML preview. Staging plus no-clobber publication preserves existing files.
#[tauri::command]
pub fn save_export_prototype_html(path: String, html: String) -> Result<(), String> {
    let target = Path::new(&path);
    if !target.is_absolute() || target.extension().and_then(|s| s.to_str()) != Some("html") {
        return Err("Choose an absolute filename ending in .html.".into());
    }
    if html.len() > 32 * 1024 * 1024 {
        return Err("Preview exceeds the 32 MB prototype limit.".into());
    }
    let parent = target.parent().ok_or("Choose a destination folder.")?;
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    staged
        .write_all(html.as_bytes())
        .map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist_noclobber(target).map_err(|e| {
        format!("Could not save preview. Choose a new filename; existing files are preserved. {e}")
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preview_never_overwrites_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir
            .path()
            .join("preview.html")
            .to_string_lossy()
            .into_owned();
        save_export_prototype_html(path.clone(), "first".into()).unwrap();
        assert!(save_export_prototype_html(path.clone(), "second".into()).is_err());
        assert_eq!(std::fs::read_to_string(path).unwrap(), "first");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn preview_rejects_non_html_and_relative_paths() {
        assert!(save_export_prototype_html("preview.html".into(), "".into()).is_err());
        let dir = tempfile::tempdir().unwrap();
        assert!(save_export_prototype_html(
            dir.path().join("project.db").to_string_lossy().into_owned(),
            "".into()
        )
        .is_err());
    }
}
