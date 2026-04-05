use rusqlite::params;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{
    Chapter, EditorMode, PlanningStatus, Scene, SceneStatus, SceneType, StoryTemplate, TemplatePart,
};

use super::AppState;

const BUNDLED_TEMPLATES: &[&str] = &[
    include_str!("../../templates/save-the-cat.json"),
    include_str!("../../templates/three-act-structure.json"),
    include_str!("../../templates/heros-journey.json"),
    include_str!("../../templates/story-circle.json"),
    include_str!("../../templates/seven-point.json"),
];

fn load_bundled_templates() -> Vec<StoryTemplate> {
    BUNDLED_TEMPLATES
        .iter()
        .filter_map(|json| {
            let mut t: StoryTemplate = serde_json::from_str(json).ok()?;
            t.bundled = true;
            Some(t)
        })
        .collect()
}

#[tauri::command]
pub async fn get_bundled_templates() -> Result<Vec<StoryTemplate>, String> {
    Ok(load_bundled_templates())
}

#[tauri::command]
pub async fn get_user_templates(
    project_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<StoryTemplate>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, project_types, structure_json, created_at
             FROM story_templates
             WHERE project_id IS NULL OR project_id = ?1
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let pid = project_id.unwrap_or_default();
    let templates = stmt
        .query_map(params![pid], |row| {
            let project_types_json: String = row.get(3)?;
            let structure_json: String = row.get(4)?;
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                project_types_json,
                structure_json,
                row.get::<_, Option<String>>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(
            |(id, name, description, pt_json, struct_json, created_at)| {
                let project_types: Vec<String> = serde_json::from_str(&pt_json).unwrap_or_default();
                let structure: Vec<TemplatePart> = match serde_json::from_str(&struct_json) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Warning: failed to parse template structure for '{name}': {e}");
                        return None;
                    }
                };
                Some(StoryTemplate {
                    id,
                    name,
                    source: None,
                    description,
                    project_types,
                    structure,
                    bundled: false,
                    created_at,
                })
            },
        )
        .collect();

    Ok(templates)
}

#[tauri::command]
pub async fn apply_template(
    project_id: String,
    template_json: String,
    clear_existing: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let template: StoryTemplate =
        serde_json::from_str(&template_json).map_err(|e| e.to_string())?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    let mut next_pos = 0i32;

    if clear_existing.unwrap_or(false) {
        let pid = uuid.to_string();
        let scene_sub = "SELECT id FROM scenes WHERE chapter_id IN (SELECT id FROM chapters WHERE project_id = ?1)";
        for table in &[
            "beats",
            "scene_character_refs",
            "scene_location_refs",
            "scene_reference_item_refs",
            "scene_reference_state",
            "discovery_notes",
            "dismissed_suggestions",
        ] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE scene_id IN ({scene_sub})"),
                params![pid],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.execute(
            "DELETE FROM scenes WHERE chapter_id IN (SELECT id FROM chapters WHERE project_id = ?1)",
            params![pid],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM chapters WHERE project_id = ?1", params![pid])
            .map_err(|e| e.to_string())?;
    } else {
        let existing_chapters = db::get_chapters(&tx, &uuid).map_err(|e| e.to_string())?;
        next_pos = existing_chapters
            .iter()
            .map(|c| c.position)
            .max()
            .unwrap_or(-1)
            + 1;
    }

    for part in &template.structure {
        let part_id = Uuid::new_v4();
        db::insert_chapter(
            &tx,
            &Chapter {
                id: part_id,
                project_id: uuid,
                title: part.title.clone(),
                position: next_pos,
                source_id: None,
                archived: false,
                locked: false,
                is_part: true,
                synopsis: None,
                planning_status: PlanningStatus::Flexible,
            },
        )
        .map_err(|e| e.to_string())?;
        next_pos += 1;

        for chapter in &part.children {
            let chapter_id = Uuid::new_v4();
            db::insert_chapter(
                &tx,
                &Chapter {
                    id: chapter_id,
                    project_id: uuid,
                    title: chapter.title.clone(),
                    position: next_pos,
                    source_id: None,
                    archived: false,
                    locked: false,
                    is_part: false,
                    synopsis: chapter.synopsis.clone(),
                    planning_status: PlanningStatus::Flexible,
                },
            )
            .map_err(|e| e.to_string())?;
            next_pos += 1;

            for (scene_pos, scene) in chapter.scenes.iter().enumerate() {
                db::insert_scene(
                    &tx,
                    &Scene {
                        id: Uuid::new_v4(),
                        chapter_id,
                        title: scene.title.clone(),
                        synopsis: scene.synopsis.clone(),
                        prose: None,
                        position: scene_pos as i32,
                        source_id: None,
                        archived: false,
                        locked: false,
                        scene_type: SceneType::Normal,
                        scene_status: SceneStatus::Draft,
                        planning_status: PlanningStatus::Flexible,
                        editor_mode: EditorMode::Beat,
                    },
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Deserialize)]
pub struct SaveAsTemplateInput {
    pub name: String,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn save_project_as_template(
    project_id: String,
    input: SaveAsTemplateInput,
    state: State<'_, AppState>,
) -> Result<StoryTemplate, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let project = db::get_project(&conn, &uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project not found: {project_id}"))?;

    let chapters = db::get_chapters(&conn, &uuid).map_err(|e| e.to_string())?;
    let active_chapters: Vec<_> = chapters.into_iter().filter(|c| !c.archived).collect();

    let mut parts: Vec<TemplatePart> = Vec::new();
    let mut current_part: Option<TemplatePart> = None;
    let mut orphan_chapters: Vec<crate::models::TemplateChapter> = Vec::new();

    for ch in &active_chapters {
        if ch.is_part {
            if let Some(part) = current_part.take() {
                parts.push(part);
            }
            current_part = Some(TemplatePart {
                title: ch.title.clone(),
                children: Vec::new(),
            });
        } else {
            let scenes = db::get_scenes(&conn, &ch.id).map_err(|e| e.to_string())?;
            let template_scenes: Vec<_> = scenes
                .iter()
                .filter(|s| !s.archived)
                .map(|s| crate::models::TemplateScene {
                    title: s.title.clone(),
                    synopsis: s.synopsis.clone(),
                })
                .collect();

            let template_ch = crate::models::TemplateChapter {
                title: ch.title.clone(),
                synopsis: ch.synopsis.clone(),
                scenes: template_scenes,
            };

            if let Some(ref mut part) = current_part {
                part.children.push(template_ch);
            } else {
                orphan_chapters.push(template_ch);
            }
        }
    }

    if let Some(part) = current_part.take() {
        parts.push(part);
    }

    if !orphan_chapters.is_empty() {
        parts.insert(
            0,
            TemplatePart {
                title: "Ungrouped".to_string(),
                children: orphan_chapters,
            },
        );
    }

    let template_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();
    let project_types_json = serde_json::to_string(&vec![&project.project_type])
        .unwrap_or_else(|_| format!("[\"{}\"]", project.project_type));
    let structure_json = serde_json::to_string(&parts).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO story_templates (id, project_id, name, description, project_types, structure_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            template_id.to_string(),
            project_id,
            input.name,
            input.description,
            project_types_json,
            structure_json,
            now,
        ],
    )
    .map_err(|e| e.to_string())?;

    let template = StoryTemplate {
        id: template_id.to_string(),
        name: input.name,
        source: None,
        description: input.description,
        project_types: vec![project.project_type.clone()],
        structure: parts,
        bundled: false,
        created_at: Some(now),
    };

    Ok(template)
}

#[tauri::command]
pub async fn delete_user_template(
    template_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM story_templates WHERE id = ?1",
        params![template_id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::models::{Project, SourceType};

    fn setup_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::schema::initialize_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_load_bundled_templates() {
        let templates = load_bundled_templates();
        assert_eq!(templates.len(), 5);

        let names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"Save the Cat"));
        assert!(names.contains(&"Three-Act Structure"));
        assert!(names.contains(&"Hero's Journey"));
        assert!(names.contains(&"Story Circle"));
        assert!(names.contains(&"Seven-Point Story Structure"));

        for t in &templates {
            assert!(t.bundled);
            assert!(!t.structure.is_empty());
            assert!(!t.project_types.is_empty());
        }
    }

    #[test]
    fn test_bundled_template_structure_validity() {
        let templates = load_bundled_templates();
        for t in &templates {
            for part in &t.structure {
                assert!(!part.title.is_empty(), "Part title empty in {}", t.name);
                for ch in &part.children {
                    assert!(!ch.title.is_empty(), "Chapter title empty in {}", t.name);
                    for scene in &ch.scenes {
                        assert!(!scene.title.is_empty(), "Scene title empty in {}", t.name);
                    }
                }
            }
        }
    }

    #[test]
    fn test_save_the_cat_has_15_beats() {
        let templates = load_bundled_templates();
        let stc = templates.iter().find(|t| t.name == "Save the Cat").unwrap();
        let total_chapters: usize = stc.structure.iter().map(|p| p.children.len()).sum();
        assert_eq!(total_chapters, 15);
        assert_eq!(stc.structure.len(), 3); // 3 acts
    }

    #[test]
    fn test_apply_template_creates_structure() {
        let conn = setup_db();
        let project = Project::new("Template Test".to_string(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();

        let templates = load_bundled_templates();
        let template = &templates[0]; // Save the Cat

        let tx = conn.unchecked_transaction().unwrap();
        let mut next_pos = 0i32;
        for part in &template.structure {
            let part_id = Uuid::new_v4();
            db::insert_chapter(
                &tx,
                &Chapter {
                    id: part_id,
                    project_id: project.id,
                    title: part.title.clone(),
                    position: next_pos,
                    source_id: None,
                    archived: false,
                    locked: false,
                    is_part: true,
                    synopsis: None,
                    planning_status: PlanningStatus::Flexible,
                },
            )
            .unwrap();
            next_pos += 1;

            for ch in &part.children {
                let ch_id = Uuid::new_v4();
                db::insert_chapter(
                    &tx,
                    &Chapter {
                        id: ch_id,
                        project_id: project.id,
                        title: ch.title.clone(),
                        position: next_pos,
                        source_id: None,
                        archived: false,
                        locked: false,
                        is_part: false,
                        synopsis: ch.synopsis.clone(),
                        planning_status: PlanningStatus::Flexible,
                    },
                )
                .unwrap();
                next_pos += 1;

                for (i, scene) in ch.scenes.iter().enumerate() {
                    db::insert_scene(
                        &tx,
                        &Scene {
                            id: Uuid::new_v4(),
                            chapter_id: ch_id,
                            title: scene.title.clone(),
                            synopsis: scene.synopsis.clone(),
                            prose: None,
                            position: i as i32,
                            source_id: None,
                            archived: false,
                            locked: false,
                            scene_type: SceneType::Normal,
                            scene_status: SceneStatus::Draft,
                            planning_status: PlanningStatus::Flexible,
                            editor_mode: EditorMode::Beat,
                        },
                    )
                    .unwrap();
                }
            }
        }
        tx.commit().unwrap();

        let chapters = db::get_chapters(&conn, &project.id).unwrap();
        let parts: Vec<_> = chapters.iter().filter(|c| c.is_part).collect();
        let non_parts: Vec<_> = chapters.iter().filter(|c| !c.is_part).collect();

        assert_eq!(parts.len(), 3);
        assert_eq!(non_parts.len(), 15);
        assert_eq!(chapters.len(), 18); // 3 acts + 15 sequences
    }

    #[test]
    fn test_user_template_crud() {
        let conn = setup_db();
        let template_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let pt_json = r#"["novel","screenplay"]"#;
        let struct_json =
            r#"[{"title":"Part 1","children":[{"title":"Ch 1","scenes":[{"title":"Sc 1"}]}]}]"#;

        conn.execute(
            "INSERT INTO story_templates (id, project_id, name, description, project_types, structure_json, created_at)
             VALUES (?1, NULL, ?2, ?3, ?4, ?5, ?6)",
            params![template_id, "My Template", "A custom template", pt_json, struct_json, now],
        )
        .unwrap();

        let mut stmt = conn
            .prepare("SELECT id, name FROM story_templates WHERE id = ?1")
            .unwrap();
        let result: Vec<(String, String)> = stmt
            .query_map(params![template_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1, "My Template");

        conn.execute(
            "DELETE FROM story_templates WHERE id = ?1",
            params![template_id],
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM story_templates WHERE id = ?1",
                params![template_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_all_bundled_templates_support_both_project_types() {
        let templates = load_bundled_templates();
        for t in &templates {
            assert!(
                t.project_types.contains(&"novel".to_string()),
                "{} should support novel",
                t.name
            );
            assert!(
                t.project_types.contains(&"screenplay".to_string()),
                "{} should support screenplay",
                t.name
            );
        }
    }
}
