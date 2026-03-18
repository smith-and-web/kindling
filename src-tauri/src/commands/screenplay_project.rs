//! Screenplay Project Command
//!
//! Creates a new screenplay project with a standard 3-act structure:
//! Act I (Setup), Act II (Confrontation), Act III (Resolution),
//! each with one empty sequence.

use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{
    Chapter, EditorMode, PlanningStatus, Project, Scene, SceneStatus, SceneType, SourceType,
};

use super::AppState;

/// Target length presets for screenplay projects
fn target_page_count_from_length(target_length: Option<&str>) -> Option<i32> {
    match target_length {
        Some("short") => Some(30),
        Some("feature") => Some(120),
        Some("long_feature") => Some(150),
        _ => Some(120), // default feature length
    }
}

#[tauri::command]
pub async fn create_screenplay_project(
    name: String,
    target_length: Option<String>,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let project_id = Uuid::new_v4();
    let target_page = target_page_count_from_length(target_length.as_deref());

    let project = Project {
        id: project_id,
        name,
        source_type: SourceType::Blank,
        source_path: None,
        created_at: now.clone(),
        modified_at: now,
        author_pen_name: None,
        genre: None,
        description: None,
        word_target: None,
        reference_types: Project::default_reference_types(),
        project_type: "screenplay".to_string(),
        target_page_count: target_page,
    };

    let acts = [
        "Act I — Setup",
        "Act II — Confrontation",
        "Act III — Resolution",
    ];

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &project).map_err(|e| e.to_string())?;

    for (act_pos, act_title) in acts.iter().enumerate() {
        // Act = Part (is_part: true)
        let act_id = Uuid::new_v4();
        let act_chapter = Chapter {
            id: act_id,
            project_id,
            title: (*act_title).to_string(),
            position: (act_pos * 2) as i32,
            source_id: None,
            archived: false,
            locked: false,
            is_part: true,
            synopsis: None,
            planning_status: PlanningStatus::Undefined,
        };
        db::insert_chapter(&tx, &act_chapter).map_err(|e| e.to_string())?;

        // One empty sequence per act (is_part: false)
        let seq_id = Uuid::new_v4();
        let seq_chapter = Chapter {
            id: seq_id,
            project_id,
            title: "Sequence 1".to_string(),
            position: (act_pos * 2 + 1) as i32,
            source_id: None,
            archived: false,
            locked: false,
            is_part: false,
            synopsis: None,
            planning_status: PlanningStatus::Undefined,
        };
        db::insert_chapter(&tx, &seq_chapter).map_err(|e| e.to_string())?;

        // One placeholder scene in the sequence
        let scene = Scene {
            id: Uuid::new_v4(),
            chapter_id: seq_id,
            title: "INT. LOCATION - DAY".to_string(),
            synopsis: None,
            prose: None,
            position: 0,
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Undefined,
            editor_mode: EditorMode::Beat,
        };
        db::insert_scene(&tx, &scene).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(project)
}

#[derive(Debug, Clone, Serialize)]
pub struct PageCountEstimate {
    pub pages: f32,
    pub words: u32,
    pub target: String,
}

fn count_words_in_html(html: &str) -> usize {
    let text = strip_html_simple(html);
    text.split_whitespace().count()
}

fn strip_html_simple(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}

/// Estimate page count for a project (words / 250, industry standard)
#[tauri::command]
pub async fn get_page_count_estimate(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<PageCountEstimate, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let chapters = db::get_chapters(&conn, &uuid).map_err(|e| e.to_string())?;
    let mut total_words: usize = 0;

    for chapter in chapters.iter().filter(|c| !c.archived) {
        let scenes = db::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;
        for scene in scenes.iter().filter(|s| !s.archived) {
            if let Some(ref prose) = scene.prose {
                if !prose.is_empty() {
                    total_words += count_words_in_html(prose);
                }
            }
            let beats = db::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;
            for beat in &beats {
                if let Some(ref prose) = beat.prose {
                    if !prose.is_empty() {
                        total_words += count_words_in_html(prose);
                    }
                }
            }
        }
    }

    let pages = total_words as f32 / 250.0;
    let target = db::get_project(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .and_then(|p| p.target_page_count)
        .map(|p| format!("{} pages", p))
        .unwrap_or_else(|| "—".to_string());

    Ok(PageCountEstimate {
        pages,
        words: total_words as u32,
        target,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{Beat, SourceType};

    fn setup_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::schema::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_target_page_count_feature() {
        assert_eq!(target_page_count_from_length(Some("feature")), Some(120));
    }

    #[test]
    fn test_target_page_count_short() {
        assert_eq!(target_page_count_from_length(Some("short")), Some(30));
    }

    #[test]
    fn test_target_page_count_long_feature() {
        assert_eq!(
            target_page_count_from_length(Some("long_feature")),
            Some(150)
        );
    }

    #[test]
    fn test_target_page_count_none_defaults_to_feature() {
        assert_eq!(target_page_count_from_length(None), Some(120));
    }

    #[test]
    fn test_target_page_count_unknown_defaults_to_feature() {
        assert_eq!(target_page_count_from_length(Some("other")), Some(120));
    }

    #[test]
    fn test_strip_html_simple_tags() {
        assert_eq!(strip_html_simple("<p>Hello</p>"), "Hello");
    }

    #[test]
    fn test_strip_html_nested() {
        assert_eq!(
            strip_html_simple("<div><p><strong>Bold</strong> text</p></div>"),
            "Bold text"
        );
    }

    #[test]
    fn test_strip_html_no_tags() {
        assert_eq!(strip_html_simple("plain text"), "plain text");
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html_simple(""), "");
    }

    #[test]
    fn test_count_words_basic() {
        assert_eq!(count_words_in_html("<p>one two three</p>"), 3);
    }

    #[test]
    fn test_count_words_empty_html() {
        assert_eq!(count_words_in_html("<p></p>"), 0);
    }

    #[test]
    fn test_count_words_multiple_paragraphs() {
        // Adjacent closing/opening tags merge words; real TipTap output has newlines between <p>s
        assert_eq!(
            count_words_in_html("<p>Hello world.</p>\n<p>Another line here.</p>"),
            5
        );
    }

    #[test]
    fn test_count_words_complex_formatting() {
        assert_eq!(
            count_words_in_html("<p><strong>Bold</strong> and <em>italic</em> text</p>"),
            4
        );
    }

    #[test]
    fn test_screenplay_project_structure() {
        let conn = setup_db();
        let now = chrono::Utc::now().to_rfc3339();
        let project_id = Uuid::new_v4();

        let project = Project {
            id: project_id,
            name: "Test Screenplay".to_string(),
            source_type: SourceType::Blank,
            source_path: None,
            created_at: now.clone(),
            modified_at: now,
            author_pen_name: None,
            genre: None,
            description: None,
            word_target: None,
            reference_types: Project::default_reference_types(),
            project_type: "screenplay".to_string(),
            target_page_count: Some(120),
        };

        db::insert_project(&conn, &project).unwrap();

        let acts = [
            "Act I — Setup",
            "Act II — Confrontation",
            "Act III — Resolution",
        ];
        for (i, title) in acts.iter().enumerate() {
            let act_id = Uuid::new_v4();
            db::insert_chapter(
                &conn,
                &Chapter {
                    id: act_id,
                    project_id,
                    title: title.to_string(),
                    position: (i * 2) as i32,
                    source_id: None,
                    archived: false,
                    locked: false,
                    is_part: true,
                    synopsis: None,
                    planning_status: PlanningStatus::Undefined,
                },
            )
            .unwrap();
            let seq_id = Uuid::new_v4();
            db::insert_chapter(
                &conn,
                &Chapter {
                    id: seq_id,
                    project_id,
                    title: "Sequence 1".to_string(),
                    position: (i * 2 + 1) as i32,
                    source_id: None,
                    archived: false,
                    locked: false,
                    is_part: false,
                    synopsis: None,
                    planning_status: PlanningStatus::Undefined,
                },
            )
            .unwrap();
            db::insert_scene(
                &conn,
                &Scene {
                    id: Uuid::new_v4(),
                    chapter_id: seq_id,
                    title: "INT. LOCATION - DAY".to_string(),
                    synopsis: None,
                    prose: None,
                    position: 0,
                    source_id: None,
                    archived: false,
                    locked: false,
                    scene_type: SceneType::Normal,
                    scene_status: SceneStatus::Draft,
                    planning_status: PlanningStatus::Undefined,
                    editor_mode: EditorMode::Beat,
                },
            )
            .unwrap();
        }

        let retrieved = db::get_project(&conn, &project_id).unwrap().unwrap();
        assert_eq!(retrieved.project_type, "screenplay");
        assert_eq!(retrieved.target_page_count, Some(120));

        let chapters = db::get_chapters(&conn, &project_id).unwrap();
        assert_eq!(chapters.len(), 6); // 3 acts + 3 sequences
        let parts: Vec<_> = chapters.iter().filter(|c| c.is_part).collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[0].title.starts_with("Act I"));
        assert!(parts[1].title.starts_with("Act II"));
        assert!(parts[2].title.starts_with("Act III"));

        let sequences: Vec<_> = chapters.iter().filter(|c| !c.is_part).collect();
        assert_eq!(sequences.len(), 3);
    }

    #[test]
    fn test_page_count_estimation_with_prose() {
        let conn = setup_db();
        let now = chrono::Utc::now().to_rfc3339();
        let project_id = Uuid::new_v4();

        let project = Project {
            id: project_id,
            name: "Word Count Test".to_string(),
            source_type: SourceType::Blank,
            source_path: None,
            created_at: now.clone(),
            modified_at: now,
            author_pen_name: None,
            genre: None,
            description: None,
            word_target: None,
            reference_types: Project::default_reference_types(),
            project_type: "screenplay".to_string(),
            target_page_count: Some(120),
        };
        db::insert_project(&conn, &project).unwrap();

        let chapter_id = Uuid::new_v4();
        db::insert_chapter(
            &conn,
            &Chapter {
                id: chapter_id,
                project_id,
                title: "Chapter".to_string(),
                position: 0,
                source_id: None,
                archived: false,
                locked: false,
                is_part: false,
                synopsis: None,
                planning_status: PlanningStatus::Undefined,
            },
        )
        .unwrap();

        let scene_id = Uuid::new_v4();
        db::insert_scene(
            &conn,
            &Scene {
                id: scene_id,
                chapter_id,
                title: "Test Scene".to_string(),
                synopsis: None,
                prose: Some("<p>Scene level prose with five words.</p>".to_string()),
                position: 0,
                source_id: None,
                archived: false,
                locked: false,
                scene_type: SceneType::Normal,
                scene_status: SceneStatus::Draft,
                planning_status: PlanningStatus::Undefined,
                editor_mode: EditorMode::Beat,
            },
        )
        .unwrap();

        let beat_id = Uuid::new_v4();
        db::insert_beat(
            &conn,
            &Beat {
                id: beat_id,
                scene_id,
                content: "Beat 1".to_string(),
                prose: Some("<p>Beat prose with four words.</p>".to_string()),
                position: 0,
                source_id: None,
            },
        )
        .unwrap();

        // scene: 6 words ("Scene level prose with five words")
        // beat: 5 words ("Beat prose with four words")
        // Total = 11 words, pages = 11/250 = 0.044
        let chapters = db::get_chapters(&conn, &project_id).unwrap();
        let mut total_words: usize = 0;
        for chapter in chapters.iter().filter(|c| !c.archived) {
            let scenes = db::get_scenes(&conn, &chapter.id).unwrap();
            for scene in scenes.iter().filter(|s| !s.archived) {
                if let Some(ref prose) = scene.prose {
                    if !prose.is_empty() {
                        total_words += count_words_in_html(prose);
                    }
                }
                let beats = db::get_beats(&conn, &scene.id).unwrap();
                for beat in &beats {
                    if let Some(ref prose) = beat.prose {
                        if !prose.is_empty() {
                            total_words += count_words_in_html(prose);
                        }
                    }
                }
            }
        }
        assert_eq!(total_words, 11);
        let pages = total_words as f32 / 250.0;
        assert!((pages - 0.044).abs() < 0.01);
    }

    #[test]
    fn test_page_count_excludes_archived() {
        let conn = setup_db();
        let now = chrono::Utc::now().to_rfc3339();
        let project_id = Uuid::new_v4();

        let project = Project {
            id: project_id,
            name: "Archive Test".to_string(),
            source_type: SourceType::Blank,
            source_path: None,
            created_at: now.clone(),
            modified_at: now,
            author_pen_name: None,
            genre: None,
            description: None,
            word_target: None,
            reference_types: Project::default_reference_types(),
            project_type: "screenplay".to_string(),
            target_page_count: None,
        };
        db::insert_project(&conn, &project).unwrap();

        let chapter_id = Uuid::new_v4();
        db::insert_chapter(
            &conn,
            &Chapter {
                id: chapter_id,
                project_id,
                title: "Ch".to_string(),
                position: 0,
                source_id: None,
                archived: true,
                locked: false,
                is_part: false,
                synopsis: None,
                planning_status: PlanningStatus::Undefined,
            },
        )
        .unwrap();

        let scene_id = Uuid::new_v4();
        db::insert_scene(
            &conn,
            &Scene {
                id: scene_id,
                chapter_id,
                title: "Archived Scene".to_string(),
                synopsis: None,
                prose: Some("<p>This should not be counted</p>".to_string()),
                position: 0,
                source_id: None,
                archived: false,
                locked: false,
                scene_type: SceneType::Normal,
                scene_status: SceneStatus::Draft,
                planning_status: PlanningStatus::Undefined,
                editor_mode: EditorMode::Beat,
            },
        )
        .unwrap();

        let chapters = db::get_chapters(&conn, &project_id).unwrap();
        let mut total_words: usize = 0;
        for chapter in chapters.iter().filter(|c| !c.archived) {
            let scenes = db::get_scenes(&conn, &chapter.id).unwrap();
            for scene in scenes.iter().filter(|s| !s.archived) {
                if let Some(ref prose) = scene.prose {
                    total_words += count_words_in_html(prose);
                }
            }
        }
        assert_eq!(total_words, 0);
    }

    #[test]
    fn test_novel_project_type_default() {
        let project = Project::new("Novel Test".to_string(), SourceType::Blank, None);
        assert_eq!(project.project_type, "novel");
        assert_eq!(project.target_page_count, None);
    }

    #[test]
    fn test_project_type_persists_in_db() {
        let conn = setup_db();
        let project = Project {
            project_type: "screenplay".to_string(),
            target_page_count: Some(90),
            ..Project::new("DB Type Test".to_string(), SourceType::Blank, None)
        };
        db::insert_project(&conn, &project).unwrap();
        let retrieved = db::get_project(&conn, &project.id).unwrap().unwrap();
        assert_eq!(retrieved.project_type, "screenplay");
        assert_eq!(retrieved.target_page_count, Some(90));
    }
}
