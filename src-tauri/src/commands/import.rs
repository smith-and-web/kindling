//! Import Commands
//!
//! Handles importing projects from external formats (Plottr, Markdown, Longform).

use serde::Serialize;
use tauri::State;

use crate::db;
use crate::models::Project;
use crate::parsers::{
    parse_longform_path, parse_markdown_outline, parse_plottr_file, parse_scrivener_bundle,
    parse_ywriter_file,
};

use super::AppState;

/// Preview of an import without inserting into the database
#[derive(Debug, Serialize)]
pub struct ImportPreview {
    pub project_name: String,
    pub chapter_count: i32,
    pub scene_count: i32,
    pub beat_count: i32,
    pub character_count: i32,
    pub location_count: i32,
}

#[tauri::command]
pub async fn preview_import(path: String, format: String) -> Result<ImportPreview, String> {
    let format_lower = format.to_lowercase();
    let preview = match format_lower.as_str() {
        "plottr" => {
            let parsed = parse_plottr_file(&path).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: parsed.beats.len() as i32,
                character_count: parsed.characters.len() as i32,
                location_count: parsed.locations.len() as i32,
            }
        }
        "markdown" => {
            let parsed = parse_markdown_outline(&path).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: parsed.beats.len() as i32,
                character_count: 0,
                location_count: 0,
            }
        }
        "ywriter" => {
            let parsed = parse_ywriter_file(&path).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: parsed.beats.len() as i32,
                character_count: parsed.characters.len() as i32,
                location_count: parsed.locations.len() as i32,
            }
        }
        "longform" => {
            let parsed = parse_longform_path(&path).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: parsed.beats.len() as i32,
                character_count: parsed.characters.len() as i32,
                location_count: parsed.locations.len() as i32,
            }
        }
        "scrivener" => {
            let parsed =
                parse_scrivener_bundle(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: 0,
                character_count: 0,
                location_count: 0,
            }
        }
        _ => return Err(format!("Unknown format: {}", format)),
    };
    Ok(preview)
}

#[tauri::command]
pub async fn import_plottr(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_plottr_file(&path).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Insert project
    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;

    // Insert chapters
    for chapter in &parsed.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    // Insert scenes
    for scene in &parsed.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    // Insert beats
    for beat in &parsed.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    // Insert characters
    for character in &parsed.characters {
        db::insert_character(&tx, character).map_err(|e| e.to_string())?;
    }

    // Insert locations
    for location in &parsed.locations {
        db::insert_location(&tx, location).map_err(|e| e.to_string())?;
    }

    // Insert scene references
    for (scene_id, character_id) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&tx, scene_id, character_id).map_err(|e| e.to_string())?;
    }

    for (scene_id, location_id) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&tx, scene_id, location_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_ywriter(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_ywriter_file(&path).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Insert project
    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;

    // Insert chapters
    for chapter in &parsed.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    // Insert scenes
    for scene in &parsed.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    // Insert beats
    for beat in &parsed.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    // Insert characters
    for character in &parsed.characters {
        db::insert_character(&tx, character).map_err(|e| e.to_string())?;
    }

    // Insert locations
    for location in &parsed.locations {
        db::insert_location(&tx, location).map_err(|e| e.to_string())?;
    }

    // Insert scene references
    for (scene_id, character_id) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&tx, scene_id, character_id).map_err(|e| e.to_string())?;
    }

    for (scene_id, location_id) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&tx, scene_id, location_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_markdown(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_markdown_outline(&path).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;

    for chapter in &parsed.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    for scene in &parsed.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    for beat in &parsed.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_longform(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_longform_path(&path).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;

    for chapter in &parsed.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    for scene in &parsed.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    for beat in &parsed.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    for character in &parsed.characters {
        db::insert_character(&tx, character).map_err(|e| e.to_string())?;
    }

    for location in &parsed.locations {
        db::insert_location(&tx, location).map_err(|e| e.to_string())?;
    }

    for item in &parsed.reference_items {
        db::insert_reference_item(&tx, item).map_err(|e| e.to_string())?;
    }

    for (scene_id, character_id) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&tx, scene_id, character_id).map_err(|e| e.to_string())?;
    }

    for (scene_id, location_id) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&tx, scene_id, location_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_scrivener(path: String, state: State<'_, AppState>) -> Result<Project, String> {
    let parsed = parse_scrivener_bundle(std::path::Path::new(&path)).map_err(|e| e.to_string())?;

    let mut conn = state.db.lock().map_err(|e| e.to_string())?;

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;

    for chapter in &parsed.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    for scene in &parsed.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}
