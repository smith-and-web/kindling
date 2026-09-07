//! Searchable prose and atomic, optimistic replacement batches.
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::{
    db,
    models::{EditorMode, PlanningStatus},
};

#[derive(Clone, Serialize)]
pub struct ProseDocument {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub chapter_id: Uuid,
    pub chapter_title: String,
    pub scene_title: String,
    pub beat_title: Option<String>,
    pub prose: String,
    pub locked: bool,
}

fn documents(conn: &Connection, project_id: &Uuid) -> Result<Vec<ProseDocument>, String> {
    let mut result = Vec::new();
    for chapter in db::get_chapters(conn, project_id).map_err(|e| e.to_string())? {
        for scene in db::get_scenes(conn, &chapter.id).map_err(|e| e.to_string())? {
            // Match ScenePanel's editor visibility; hidden planning prose is not a replacement target.
            if scene.planning_status != PlanningStatus::Fixed {
                continue;
            }
            let base = ProseDocument {
                id: scene.id,
                scene_id: scene.id,
                chapter_id: chapter.id,
                chapter_title: chapter.title.clone(),
                scene_title: scene.title.clone(),
                beat_title: None,
                prose: scene.prose.clone().unwrap_or_default(),
                locked: chapter.locked || scene.locked,
            };
            // Only search the active representation; the other is a stale mode-switch copy.
            if scene.editor_mode == EditorMode::Page {
                result.push(base);
            } else {
                let beats = db::get_beats(conn, &scene.id).map_err(|e| e.to_string())?;
                if beats.is_empty() {
                    result.push(base.clone());
                }
                for beat in beats {
                    result.push(ProseDocument {
                        id: beat.id,
                        beat_title: Some(beat.content),
                        prose: beat.prose.unwrap_or_default(),
                        ..base.clone()
                    });
                }
            }
        }
    }
    Ok(result)
}

#[derive(Deserialize)]
pub struct ProseReplacement {
    pub id: Uuid,
    pub expected_prose: String,
    pub prose: String,
}

fn replace_batch(
    conn: &Connection,
    project_id: &Uuid,
    changes: &[ProseReplacement],
) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let available = documents(&tx, project_id)?;
    let mut seen = std::collections::HashSet::new();
    for change in changes {
        let doc = available
            .iter()
            .find(|doc| doc.id == change.id)
            .ok_or("Prose is no longer available. Close and reopen Find and Replace.")?;
        if !seen.insert(change.id) || doc.prose != change.expected_prose {
            return Err("Prose changed since searching. Close and reopen Find and Replace.".into());
        }
        if doc.locked {
            return Err("Cannot replace prose in a locked scene or chapter.".into());
        }
        db::writing::save_prose_in_transaction(
            &tx,
            &doc.scene_id,
            chrono::Local::now().date_naive(),
            |tx| {
                if doc.beat_title.is_some() {
                    db::update_beat_prose(tx, &doc.id, &change.prose)
                } else {
                    db::save_scene_page_prose(tx, &doc.id, &change.prose)
                }
            },
        )
        .map_err(|e| e.to_string())?;
    }
    if !changes.is_empty() {
        db::update_project_modified(&tx, project_id).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_search_documents(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ProseDocument>, String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    documents(&conn, &id)
}

#[tauri::command]
pub async fn replace_prose_batch(
    project_id: String,
    changes: Vec<ProseReplacement>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let id = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    replace_batch(&conn, &id, &changes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Beat, Chapter, Project, Scene, SourceType};

    fn fixture() -> (Connection, Uuid, Scene, Beat, Scene) {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = Project::new("Search".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let mut scene = Scene::new(chapter.id, "Beats".into(), None, 0);
        scene.prose = Some("stale page copy".into());
        db::insert_scene(&conn, &scene).unwrap();
        let mut beat = Beat::new(scene.id, "Prompt".into(), 0);
        beat.prose = Some("<p>Find me</p>".into());
        db::insert_beat(&conn, &beat).unwrap();
        let mut page = Scene::new(chapter.id, "Page".into(), None, 1);
        page.editor_mode = EditorMode::Page;
        page.prose = Some("<p>Find me too</p>".into());
        db::insert_scene(&conn, &page).unwrap();
        let mut stale = Beat::new(page.id, "Stale beat".into(), 0);
        stale.prose = Some("stale beat copy".into());
        db::insert_beat(&conn, &stale).unwrap();
        (conn, project.id, scene, beat, page)
    }

    fn change(id: Uuid, expected: &str) -> ProseReplacement {
        ProseReplacement {
            id,
            expected_prose: expected.into(),
            prose: "<p>Replaced</p>".into(),
        }
    }

    #[test]
    fn searches_active_prose_in_manuscript_order_and_scopes_to_project() {
        let (conn, project, scene, beat, page) = fixture();
        let docs = documents(&conn, &project).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].id, beat.id);
        assert_eq!(docs[0].scene_id, scene.id);
        assert_eq!(docs[0].beat_title.as_deref(), Some("Prompt"));
        assert_eq!(docs[1].id, page.id);
        assert!(docs[1].beat_title.is_none());
        assert!(documents(&conn, &Uuid::new_v4()).unwrap().is_empty());
    }

    #[test]
    fn hidden_planning_prose_is_excluded_and_revalidated_before_replacement() {
        let (conn, project, scene, beat, page) = fixture();
        for status in [PlanningStatus::Flexible, PlanningStatus::Undefined] {
            db::update_scene_planning_status(&conn, &page.id, &status).unwrap();
            assert_eq!(documents(&conn, &project).unwrap().len(), 1);
            // A status change after searching must roll back earlier writes in the batch too.
            assert!(replace_batch(
                &conn,
                &project,
                &[
                    change(beat.id, "<p>Find me</p>"),
                    change(page.id, "<p>Find me too</p>")
                ]
            )
            .is_err());
            db::update_scene_planning_status(&conn, &scene.id, &status).unwrap();
            assert!(documents(&conn, &project).unwrap().is_empty());
            assert!(replace_batch(&conn, &project, &[change(beat.id, "<p>Find me</p>")]).is_err());
            db::update_scene_planning_status(&conn, &scene.id, &PlanningStatus::Fixed).unwrap();
            db::update_scene_planning_status(&conn, &page.id, &PlanningStatus::Fixed).unwrap();
            let restored = documents(&conn, &project).unwrap();
            assert_eq!(restored[0].prose, "<p>Find me</p>");
            assert_eq!(restored[1].prose, "<p>Find me too</p>");
        }
    }

    #[test]
    fn includes_scene_prose_when_beat_mode_has_no_beats() {
        let (conn, project, scene, beat, _) = fixture();
        db::delete_beat(&conn, &beat.id).unwrap();
        let docs = documents(&conn, &project).unwrap();
        assert_eq!(docs[0].id, scene.id);
        assert_eq!(docs[0].prose, "stale page copy");
        replace_batch(&conn, &project, &[change(scene.id, "stale page copy")]).unwrap();
        assert_eq!(
            documents(&conn, &project).unwrap()[0].prose,
            "<p>Replaced</p>"
        );
    }

    #[test]
    fn replaces_both_modes_and_supports_optimistic_undo() {
        let (conn, project, _, beat, page) = fixture();
        let changes = [
            change(beat.id, "<p>Find me</p>"),
            change(page.id, "<p>Find me too</p>"),
        ];
        replace_batch(&conn, &project, &changes).unwrap();
        assert!(documents(&conn, &project)
            .unwrap()
            .iter()
            .all(|doc| doc.prose == "<p>Replaced</p>"));
        let writing =
            db::writing::stats(&conn, &project, chrono::Local::now().date_naive()).unwrap();
        assert_eq!((writing.today_words, writing.session_words), (-3, -3));
        let undo = changes
            .into_iter()
            .map(|c| ProseReplacement {
                id: c.id,
                expected_prose: c.prose,
                prose: c.expected_prose,
            })
            .collect::<Vec<_>>();
        replace_batch(&conn, &project, &undo).unwrap();
        let writing =
            db::writing::stats(&conn, &project, chrono::Local::now().date_naive()).unwrap();
        assert_eq!((writing.today_words, writing.session_words), (0, 0));
        assert_eq!(
            db::get_beat(&conn, &beat.id)
                .unwrap()
                .unwrap()
                .prose
                .as_deref(),
            Some("<p>Find me</p>")
        );
        assert_eq!(
            db::get_scene_by_id(&conn, &page.id)
                .unwrap()
                .unwrap()
                .prose
                .as_deref(),
            Some("<p>Find me too</p>")
        );
    }

    #[test]
    fn stale_missing_duplicate_and_foreign_targets_roll_back_the_entire_batch() {
        let (conn, project, _, beat, page) = fixture();
        for invalid in [
            change(page.id, "stale"),
            change(Uuid::new_v4(), ""),
            change(beat.id, "<p>Find me</p>"),
        ] {
            assert!(replace_batch(
                &conn,
                &project,
                &[change(beat.id, "<p>Find me</p>"), invalid]
            )
            .is_err());
            assert_eq!(
                documents(&conn, &project).unwrap()[0].prose,
                "<p>Find me</p>"
            );
        }
        assert!(
            replace_batch(&conn, &Uuid::new_v4(), &[change(beat.id, "<p>Find me</p>")]).is_err()
        );
        replace_batch(&conn, &project, &[]).unwrap();
    }

    #[test]
    fn locked_scenes_and_chapters_are_searchable_but_cannot_be_replaced() {
        let (conn, project, scene, beat, page) = fixture();
        db::lock_scene(&conn, &page.id).unwrap();
        assert!(documents(&conn, &project).unwrap()[1].locked);
        assert!(replace_batch(
            &conn,
            &project,
            &[
                change(beat.id, "<p>Find me</p>"),
                change(page.id, "<p>Find me too</p>")
            ]
        )
        .is_err());
        assert_eq!(
            documents(&conn, &project).unwrap()[0].prose,
            "<p>Find me</p>"
        );
        db::lock_chapter(&conn, &scene.chapter_id).unwrap();
        assert!(documents(&conn, &project).unwrap()[0].locked);
        assert!(replace_batch(&conn, &project, &[change(beat.id, "<p>Find me</p>")]).is_err());
    }

    #[test]
    fn archived_or_mode_switched_prose_cannot_be_replaced() {
        let (conn, project, scene, beat, page) = fixture();
        conn.execute(
            "UPDATE scenes SET archived = 1 WHERE id = ?1",
            [page.id.to_string()],
        )
        .unwrap();
        assert_eq!(documents(&conn, &project).unwrap().len(), 1);
        assert!(replace_batch(&conn, &project, &[change(page.id, "<p>Find me too</p>")]).is_err());
        db::switch_scene_editor_mode(&conn, &scene.id, "page").unwrap();
        assert!(replace_batch(&conn, &project, &[change(beat.id, "<p>Find me</p>")]).is_err());
        conn.execute(
            "UPDATE chapters SET archived = 1 WHERE id = ?1",
            [scene.chapter_id.to_string()],
        )
        .unwrap();
        assert!(documents(&conn, &project).unwrap().is_empty());
    }
}
