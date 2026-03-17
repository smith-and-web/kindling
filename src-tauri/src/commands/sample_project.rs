//! Sample Project Command
//!
//! Creates a minimal sample project for first-time users to explore Kindling
//! without importing their own outline.

use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

use crate::db;
use crate::models::{
    Beat, Chapter, Character, EditorMode, Location, PlanningStatus, Project, ReferenceItem, Scene,
    SceneStatus, SceneType, SourceType,
};

use super::AppState;

/// Build and insert a sample project into the database.
/// Returns the created project for the frontend to load.
#[tauri::command]
pub async fn create_sample_project(state: State<'_, AppState>) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();
    let project_id = Uuid::new_v4();

    let project = Project {
        id: project_id,
        name: "Sample Project".to_string(),
        source_type: SourceType::Markdown,
        source_path: None,
        created_at: now.clone(),
        modified_at: now,
        author_pen_name: None,
        genre: Some("Fiction".to_string()),
        description: Some("A sample project to explore Kindling. Try the sidebar, scene panel, beats, and references.".to_string()),
        word_target: None,
        reference_types: Project::default_reference_types(),
        project_type: Project::default_project_type(),
        target_page_count: None,
    };

    let chapter_id = Uuid::new_v4();
    let chapter = Chapter {
        id: chapter_id,
        project_id,
        title: "Chapter 1: The Beginning".to_string(),
        position: 0,
        source_id: None,
        archived: false,
        locked: false,
        is_part: false,
        synopsis: None,
        planning_status: PlanningStatus::Fixed,
    };

    let scene1_id = Uuid::new_v4();
    let scene2_id = Uuid::new_v4();
    let scene3_id = Uuid::new_v4();

    let scenes = vec![
        Scene {
            id: scene1_id,
            chapter_id,
            title: "The Call to Adventure".to_string(),
            synopsis: Some(
                "Our hero receives an unexpected invitation that will change everything."
                    .to_string(),
            ),
            prose: None,
            position: 0,
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Fixed,
            editor_mode: EditorMode::Beat,
        },
        Scene {
            id: scene2_id,
            chapter_id,
            title: "Meeting the Mentor".to_string(),
            synopsis: Some("A wise figure appears with guidance and a crucial object.".to_string()),
            prose: None,
            position: 1,
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Fixed,
            editor_mode: EditorMode::Beat,
        },
        Scene {
            id: scene3_id,
            chapter_id,
            title: "Crossing the Threshold".to_string(),
            synopsis: Some(
                "The hero commits to the journey and leaves the ordinary world behind.".to_string(),
            ),
            prose: None,
            position: 2,
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
            planning_status: PlanningStatus::Fixed,
            editor_mode: EditorMode::Beat,
        },
    ];

    let beat1_1 = Beat {
        id: Uuid::new_v4(),
        scene_id: scene1_id,
        content: "The letter arrives".to_string(),
        prose: Some("The morning post brought something unexpected: a thick envelope with a wax seal. Inside, an invitation to the annual gathering at the old estate—a place they'd heard stories about but never visited.".to_string()),
        position: 0,
        source_id: None,
    };
    let beat1_2 = Beat {
        id: Uuid::new_v4(),
        scene_id: scene1_id,
        content: "The decision".to_string(),
        prose: Some("They could ignore it. Life was comfortable enough. But something in the invitation called to them—a curiosity they hadn't felt in years.".to_string()),
        position: 1,
        source_id: None,
    };
    let beat2_1 = Beat {
        id: Uuid::new_v4(),
        scene_id: scene2_id,
        content: "The stranger at the gate".to_string(),
        prose: Some("An elderly figure waited at the estate gates, leaning on a carved staff. Their eyes held a knowing look, as if they'd been expecting this visitor all along.".to_string()),
        position: 0,
        source_id: None,
    };
    let beat2_2 = Beat {
        id: Uuid::new_v4(),
        scene_id: scene2_id,
        content: "The gift".to_string(),
        prose: Some("From a worn satchel, the mentor produced a small compass. \"This will help when the path grows unclear,\" they said. \"Trust it.\"".to_string()),
        position: 1,
        source_id: None,
    };
    let beat3_1 = Beat {
        id: Uuid::new_v4(),
        scene_id: scene3_id,
        content: "Looking back".to_string(),
        prose: Some("The familiar rooftops faded behind them. Part of them wanted to turn back—to return to the life they knew. But the compass in their pocket hummed with warmth, and they kept walking.".to_string()),
        position: 0,
        source_id: None,
    };

    let character_id = Uuid::new_v4();
    let character = Character {
        id: character_id,
        project_id,
        name: "The Hero".to_string(),
        description: Some("Our protagonist, on the cusp of a great adventure.".to_string()),
        attributes: HashMap::from([("Role".to_string(), "Protagonist".to_string())]),
        source_id: None,
    };

    let location_id = Uuid::new_v4();
    let location = Location {
        id: location_id,
        project_id,
        name: "The Old Estate".to_string(),
        description: Some("A mysterious manor at the edge of town, rarely visited.".to_string()),
        attributes: HashMap::new(),
        source_id: None,
    };

    // Reference item (e.g., objective or custom type)
    let ref_item_id = Uuid::new_v4();
    let reference_item = ReferenceItem {
        id: ref_item_id,
        project_id,
        reference_type: "characters".to_string(),
        name: "The Mentor".to_string(),
        description: Some("A wise guide who appears at key moments.".to_string()),
        attributes: HashMap::new(),
        source_id: None,
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    db::insert_project(&tx, &project).map_err(|e| e.to_string())?;
    db::insert_chapter(&tx, &chapter).map_err(|e| e.to_string())?;

    for scene in &scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    for beat in &[beat1_1, beat1_2, beat2_1, beat2_2, beat3_1] {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    db::insert_character(&tx, &character).map_err(|e| e.to_string())?;
    db::insert_location(&tx, &location).map_err(|e| e.to_string())?;
    db::insert_reference_item(&tx, &reference_item).map_err(|e| e.to_string())?;

    db::add_scene_character_ref(&tx, &scene1_id, &character_id).map_err(|e| e.to_string())?;
    db::add_scene_character_ref(&tx, &scene2_id, &character_id).map_err(|e| e.to_string())?;
    db::add_scene_character_ref(&tx, &scene3_id, &character_id).map_err(|e| e.to_string())?;
    db::add_scene_location_ref(&tx, &scene2_id, &location_id).map_err(|e| e.to_string())?;
    db::add_scene_location_ref(&tx, &scene3_id, &location_id).map_err(|e| e.to_string())?;
    db::add_scene_reference_item_ref(&tx, &scene2_id, &ref_item_id).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(project)
}
