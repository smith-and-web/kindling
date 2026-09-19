//! Additional screenshot projects. This entire module is absent in release builds.

use std::io::Write;
use std::path::Path;

use rusqlite::Connection;
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::db;
use crate::models::{Beat, Chapter, EditorMode, Project, Scene, SourceType};
use crate::parsers::parse_markdown_outline;

const OUTLINE: &str = "---\ntitle: The Letter — Source Outline\n---\n\n# The Letter\n\n## On the Cliff\n\n- Eleanor reads the sealed letter\n- Thomas arrives with a warning\n\n## The Seventh Step\n\n- A hidden key opens nothing in the lighthouse\n";

/// Adds a screenplay and a syncable outline alongside the user-facing sample.
/// The Markdown source lives beside the active database, including when QA uses
/// KINDLING_DATA_DIR. Each invocation owns a new file and never overwrites one.
#[cfg(debug_assertions)]
#[tauri::command]
pub async fn create_demo_fixture(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let data_dir = conn
        .path()
        .and_then(|p| Path::new(p).parent())
        .ok_or("The demo fixture requires an on-disk app database")?;
    seed_demo_fixture(&conn, data_dir)
}

fn seed_demo_fixture(conn: &Connection, data_dir: &Path) -> Result<Vec<Project>, String> {
    let source_path = data_dir.join(format!("demo-outline-{}.md", Uuid::new_v4()));
    let mut source = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&source_path)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        source
            .write_all(OUTLINE.as_bytes())
            .map_err(|e| e.to_string())?;
        let parsed = parse_markdown_outline(&source_path).map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        let mut screenplay =
            Project::new("The Letter — Screenplay".into(), SourceType::Blank, None);
        screenplay.project_type = "screenplay".into();
        screenplay.target_page_count = Some(120);
        screenplay.author_pen_name = Some("E. M. Hale".into());
        screenplay.genre = Some("Gothic Mystery".into());
        db::insert_project(&tx, &screenplay).map_err(|e| e.to_string())?;
        let act = Chapter::new(screenplay.id, "Act I — Setup".into(), 0).with_is_part(true);
        db::insert_chapter(&tx, &act).map_err(|e| e.to_string())?;
        let sequence = Chapter::new(screenplay.id, "The Letter".into(), 1);
        db::insert_chapter(&tx, &sequence).map_err(|e| e.to_string())?;
        for (position, (title, synopsis, prose)) in [
            ("EXT. KESTREL POINT - DUSK", "Eleanor reads a letter that threatens everything she knows.",
             "<p>Wind tears across the cliff. ELEANOR BLACKWOOD, 18, unfolds a letter beside the lighthouse.</p><p>ELEANOR</p><p>What could possibly be worth all this deception?</p><p>The lighthouse door opens. Thomas steps into the wind.</p>"),
            ("INT. LIGHTHOUSE STAIRWELL - NIGHT", "Thomas watches as Eleanor lifts the seventh step.",
             "<p>A lamp throws their shadows up the spiral stair. Eleanor works her knife under the stone.</p><p>THOMAS</p><p>He'll hear us.</p><p>ELEANOR</p><p>Then let him explain what we find.</p><p>The step lifts. An iron key lies in a fold of oilcloth.</p>"),
        ].iter().enumerate() {
            let scene = Scene {
                editor_mode: EditorMode::Page,
                prose: Some((*prose).into()),
                ..Scene::new(sequence.id, (*title).into(), Some((*synopsis).into()), position as i32)
            };
            db::insert_scene(&tx, &scene).map_err(|e| e.to_string())?;
            db::insert_beat(&tx, &Beat::new(scene.id, (*synopsis).into(), 0)).map_err(|e| e.to_string())?;
        }
        // Import through the real parser to retain the source IDs sync compares.
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
        Ok(vec![screenplay, parsed.project])
    })();
    drop(source);
    if result.is_err() {
        // The transaction rolls back on error; remove only our own new file.
        let _ = std::fs::remove_file(source_path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::super::sync::get_sync_preview_with_connection;
    use super::*;

    #[test]
    fn screenplay_and_source_outline_are_ready_for_screenshots_and_sync() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::new(dir.path().to_path_buf()).unwrap();
        let conn = state.db.lock().unwrap();
        let projects = seed_demo_fixture(&conn, dir.path()).unwrap();
        assert_eq!(projects.len(), 2);
        let screenplay = db::get_project(&conn, &projects[0].id).unwrap().unwrap();
        assert_eq!(screenplay.project_type, "screenplay");
        let scenes = db::get_all_project_scenes(&conn, &screenplay.id).unwrap();
        assert!(scenes.len() >= 2);
        assert!(scenes
            .iter()
            .all(|s| s.title.starts_with("INT.") || s.title.starts_with("EXT.")));
        assert!(scenes.iter().all(|s| s.prose.is_some()));
        let source = db::get_project(&conn, &projects[1].id).unwrap().unwrap();
        assert_eq!(source.source_type, SourceType::Markdown);
        let path = Path::new(source.source_path.as_ref().unwrap());
        assert_eq!(path.parent(), Some(dir.path()));
        assert_eq!(std::fs::read_to_string(path).unwrap(), OUTLINE);
        let preview = get_sync_preview_with_connection(&conn, source.id).unwrap();
        assert!(preview.additions.is_empty());
        assert!(preview.changes.is_empty());
        let edited = format!("{OUTLINE}\n## Beneath the Tower\n\n- Eleanor turns the key\n");
        std::fs::write(path, &edited).unwrap();
        let preview = get_sync_preview_with_connection(&conn, source.id).unwrap();
        assert!(preview
            .additions
            .iter()
            .any(|a| a.title == "Beneath the Tower"));
        let second = seed_demo_fixture(&conn, dir.path()).unwrap();
        assert_ne!(second[1].source_path, source.source_path);
        assert_eq!(std::fs::read_to_string(path).unwrap(), edited);
    }

    #[test]
    fn failure_rolls_back_projects_and_removes_the_new_source_file() {
        let dir = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn.execute_batch("CREATE TRIGGER reject_outline BEFORE INSERT ON projects WHEN NEW.source_type = 'markdown' BEGIN SELECT RAISE(ABORT, 'test failure'); END;").unwrap();
        assert!(seed_demo_fixture(&conn, dir.path()).is_err());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM projects", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
