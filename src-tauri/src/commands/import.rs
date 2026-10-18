//! Import Commands
//!
//! Handles importing projects from external formats (Plottr, Markdown, Longform).

use serde::Serialize;
use tauri::State;

use crate::db;
use crate::models::Project;
use crate::parsers::{
    parse_longform_path, parse_markdown_outline, parse_plottr_file, parse_scrivener_bundle,
    parse_ywriter_file, ParsedLongform, ParsedYWriter,
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
        "novelwriter" => {
            let p =
                crate::parsers::novelwriter::parse_novelwriter_project(std::path::Path::new(&path))
                    .map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: p.project.name,
                chapter_count: p.chapters.len() as i32,
                scene_count: p.scenes.len() as i32,
                beat_count: p.beats.len() as i32,
                character_count: p.characters.len() as i32,
                location_count: p.locations.len() as i32,
            }
        }
        "scrivener" => {
            let parsed =
                parse_scrivener_bundle(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
            ImportPreview {
                project_name: parsed.project.name,
                chapter_count: parsed.chapters.len() as i32,
                scene_count: parsed.scenes.len() as i32,
                beat_count: parsed.beats.len() as i32,
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

    insert_ywriter(&mut conn, &parsed)?;

    Ok(parsed.project)
}

pub(crate) fn insert_ywriter(
    conn: &mut rusqlite::Connection,
    parsed: &ParsedYWriter,
) -> Result<(), String> {
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

    // Insert reference items (yWriter "items")
    for item in &parsed.reference_items {
        db::insert_reference_item(&tx, item).map_err(|e| e.to_string())?;
    }

    // Insert scene references
    for (scene_id, character_id) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&tx, scene_id, character_id).map_err(|e| e.to_string())?;
    }

    for (scene_id, location_id) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&tx, scene_id, location_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())
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

    insert_longform(&mut conn, &parsed)?;

    Ok(parsed.project)
}

pub(crate) fn insert_longform(
    conn: &mut rusqlite::Connection,
    parsed: &ParsedLongform,
) -> Result<(), String> {
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

    for (scene_id, item_id) in &parsed.scene_reference_item_refs {
        db::add_scene_reference_item_ref(&tx, scene_id, item_id).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())
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

    for beat in &parsed.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(parsed.project)
}

#[tauri::command]
pub async fn import_novelwriter(
    path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let parsed =
        crate::parsers::novelwriter::parse_novelwriter_project(std::path::Path::new(&path))
            .map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    insert_novelwriter(&conn, &parsed)?;
    Ok(parsed.project)
}

pub(crate) fn insert_novelwriter(
    conn: &rusqlite::Connection,
    parsed: &crate::parsers::novelwriter::ParsedNovelWriter,
) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    db::insert_project(&tx, &parsed.project).map_err(|e| e.to_string())?;
    for item in &parsed.chapters {
        db::insert_chapter(&tx, item).map_err(|e| e.to_string())?;
    }
    for item in &parsed.scenes {
        db::insert_scene(&tx, item).map_err(|e| e.to_string())?;
    }
    for item in &parsed.beats {
        db::insert_beat(&tx, item).map_err(|e| e.to_string())?;
    }
    for item in &parsed.characters {
        db::insert_character(&tx, item).map_err(|e| e.to_string())?;
    }
    for item in &parsed.locations {
        db::insert_location(&tx, item).map_err(|e| e.to_string())?;
    }
    for item in &parsed.reference_items {
        db::insert_reference_item(&tx, item).map_err(|e| e.to_string())?;
    }
    for (scene, item) in &parsed.scene_character_refs {
        db::add_scene_character_ref(&tx, scene, item).map_err(|e| e.to_string())?;
    }
    for (scene, item) in &parsed.scene_location_refs {
        db::add_scene_location_ref(&tx, scene, item).map_err(|e| e.to_string())?;
    }
    for (scene, item) in &parsed.scene_reference_item_refs {
        db::add_scene_reference_item_ref(&tx, scene, item).map_err(|e| e.to_string())?;
    }
    super::novelwriter_sync::record_baselines(&tx, &parsed.project, parsed)?;
    super::novelwriter_sync::record_split_documents(&tx, &parsed.project, parsed)?;
    tx.commit().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::fs;

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn longform_import_persists_scene_reference_item_links() {
        let temp = tempfile::tempdir().unwrap();
        let index = temp.path().join("Index.md");
        fs::write(
            &index,
            "---\nlongform:\n  format: scenes\n  title: Notes Test\n  scenes: [Opening]\n---\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("Opening.md"),
            "---\ntimelines: ['[[Past]]']\nnotes: ['[[Research]]']\n---\ntimelines:: [[Future]]\ncustom:: [[Motif]]\n\nScene prose.",
        )
        .unwrap();
        fs::create_dir(temp.path().join("timelines")).unwrap();
        fs::write(temp.path().join("timelines/Past.md"), "# Past\n\nBody.").unwrap();

        let parsed = parse_longform_path(&index).unwrap();
        assert!(!parsed.reference_items.is_empty());
        assert!(!parsed.scene_reference_item_refs.is_empty());

        let mut conn = memory_db();
        insert_longform(&mut conn, &parsed).unwrap();

        let stored = db::get_all_reference_items(&conn, &parsed.project.id).unwrap();
        assert_eq!(stored.len(), parsed.reference_items.len());

        let links = db::get_all_scene_reference_item_refs(&conn, &parsed.project.id).unwrap();
        assert_eq!(links.len(), parsed.scene_reference_item_refs.len());
        for (scene_id, item_id) in &parsed.scene_reference_item_refs {
            assert!(links
                .iter()
                .any(|l| &l.scene_id == scene_id && &l.reference_item_id == item_id));
        }
    }

    #[test]
    fn ywriter_import_persists_reference_items() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT><Title>Items Test</Title></PROJECT>
  <ITEMS>
    <ITEM><ID>1</ID><Title>The Key</Title><Desc>A small key.</Desc><Aka>Rust Key</Aka></ITEM>
    <ITEM><ID>2</ID><Title>The Map</Title></ITEM>
  </ITEMS>
</YWRITER7>"#;
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("items.yw7");
        fs::write(&path, xml).unwrap();

        let parsed = parse_ywriter_file(&path).unwrap();
        assert_eq!(parsed.reference_items.len(), 2);

        let mut conn = memory_db();
        insert_ywriter(&mut conn, &parsed).unwrap();

        let stored = db::get_all_reference_items(&conn, &parsed.project.id).unwrap();
        assert_eq!(stored.len(), 2);
        let key = stored.iter().find(|i| i.name == "The Key").unwrap();
        assert_eq!(key.reference_type, "items");
        assert_eq!(
            key.attributes.get("aliases").map(String::as_str),
            Some("Rust Key")
        );
        assert!(stored.iter().any(|i| i.name == "The Map"));
    }
}
