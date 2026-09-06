//! Reproducible first-time-user sample, shared with documentation screenshots.

use super::AppState;
use crate::db;
use crate::models::{
    Beat, Chapter, Character, EditorMode, FieldDefinition, Location, PlanningStatus, Project,
    ReferenceItem, SavedFilter, Scene, SceneStatus, SceneType, SourceType, Tag,
};
use rusqlite::Connection;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn create_sample_project(state: State<'_, AppState>) -> Result<Project, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    seed_sample_project(&conn).map_err(|e| e.to_string())
}

/// Keep the complete fixture atomic, including fields, tags and scene links.
fn seed_sample_project(conn: &Connection) -> rusqlite::Result<Project> {
    let tx = conn.unchecked_transaction()?;
    let mut project = Project::new("The Letter".into(), SourceType::Blank, None);
    project.author_pen_name = Some("E. M. Hale".into());
    project.genre = Some("Gothic Mystery".into());
    project.word_target = Some(90000);
    project.reference_types.push("items".into());
    project.description = Some("A lighthouse keeper's daughter learns that the man who raised her is not her father, and that the tower he tends was built to hide something.".into());
    db::insert_project(&tx, &project)?;

    let eleanor = Character {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Eleanor Blackwood".into(), description: Some("Eighteen, raised at Kestrel Point since she was two. Reads the sea better than she reads people. Carries a pocket knife her father gave her and has just learned he is not her father.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_character(&tx, &eleanor)?;
    let silas = Character {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Silas Blackwood".into(), description: Some("Keeper of Kestrel Point for sixteen years. Precise, kind, and entirely unwilling to answer a direct question. Whatever he is protecting, he has protected it longer than Eleanor has been alive.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_character(&tx, &silas)?;
    let thomas = Character {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Thomas Ward".into(), description: Some("The relief keeper from Wrackham, two years older than Eleanor and a poor liar. He knows part of it. He does not know he knows.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_character(&tx, &thomas)?;
    let margaret = Character {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Margaret Wren".into(), description: Some("M.W. Signed the letter. Nobody in Wrackham will say the name aloud, and the parish register has had a page cut out of it.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_character(&tx, &margaret)?;
    let lighthouse = Location {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Kestrel Point".into(), description: Some("Ninety-one feet of grey stone on a headland that the Admiralty chart still marks as unsurveyed. Eighty-eight steps to the lamp room. The seventh one is loose.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_location(&tx, &lighthouse)?;
    let cliff = Location {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "The Cliff Path".into(), description: Some("A quarter mile of chalk and gorse between the light and the town, with a forty-foot drop on the seaward side and no rail.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_location(&tx, &cliff)?;
    let harbour = Location {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "Wrackham Harbour".into(), description: Some("Six fishing boats, one chandlery, one church with a damaged register. Everyone here remembers Margaret Wren and no one will say so.".into()),
        attributes: HashMap::new(), source_id: None,
    };
    db::insert_location(&tx, &harbour)?;
    let sealed_letter = ReferenceItem {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "The Sealed Letter".into(), description: Some("Cream laid paper, elegant hand, no postmark. Left inside Eleanor's copy of Bowditch. Signed M.W.".into()),
        attributes: HashMap::new(), source_id: None,
        reference_type: "items".into(),
    };
    db::insert_reference_item(&tx, &sealed_letter)?;
    let seventh_step_key = ReferenceItem {
        id: uuid::Uuid::new_v4(), project_id: project.id,
        name: "The Key Beneath the Seventh Step".into(), description: Some("Iron, four inches, too large for any door in the lighthouse. It does not fit the lamp room, the oil store, or the keeper's quarters.".into()),
        attributes: HashMap::new(), source_id: None,
        reference_type: "items".into(),
    };
    db::insert_reference_item(&tx, &seventh_step_key)?;

    // Stored spelling follows the fixture contract; the UI also accepts its older
    // `multiselect` spelling so existing projects continue to work.
    let field_specs = [
        (
            "Role",
            "select",
            Some(r#"["protagonist","antagonist","supporting"]"#),
        ),
        ("Age", "number", None),
        ("Wants", "text", None),
        (
            "Traits",
            "multi_select",
            Some(r#"["observant","resolute","loyal","secretive","resourceful"]"#),
        ),
    ];
    let character_values = [
        (
            &eleanor,
            [
                "protagonist",
                "18",
                "The truth about her parents",
                r#"["observant","resolute"]"#,
            ],
        ),
        (
            &thomas,
            [
                "supporting",
                "20",
                "To be trusted with something that matters",
                r#"["loyal","resourceful"]"#,
            ],
        ),
        (
            &margaret,
            [
                "supporting",
                "43",
                "To bring Eleanor safely home",
                r#"["resourceful","secretive"]"#,
            ],
        ),
        (
            &silas,
            [
                "antagonist",
                "54",
                "For Eleanor to stop asking",
                r#"["loyal","secretive"]"#,
            ],
        ),
    ];
    for (position, (name, kind, options)) in field_specs.iter().enumerate() {
        let mut def = FieldDefinition::new(
            project.id,
            "character".into(),
            (*name).into(),
            (*kind).into(),
            position as i32,
        );
        def.options = options.map(str::to_string);
        db::create_field_definition(&tx, &def)?;
        for (character, values) in &character_values {
            db::set_field_value(&tx, &def.id, &character.id, Some(values[position]))?;
        }
    }

    let role = Tag::new(project.id, "role".into(), Some("#D97706".into()), None, 0);
    let status = Tag::new(project.id, "status".into(), None, None, 1);
    db::create_tag(&tx, &role)?;
    db::create_tag(&tx, &status)?;
    let protagonist = Tag::new(
        project.id,
        "protagonist".into(),
        Some("#D97706".into()),
        Some(role.id),
        0,
    );
    let antagonist = Tag::new(project.id, "antagonist".into(), None, Some(role.id), 1);
    let supporting = Tag::new(project.id, "supporting".into(), None, Some(role.id), 2);
    let active = Tag::new(
        project.id,
        "active".into(),
        Some("#15803D".into()),
        Some(status.id),
        0,
    );
    let deceased = Tag::new(project.id, "deceased".into(), None, Some(status.id), 1);
    let unknown = Tag::new(project.id, "unknown".into(), None, Some(status.id), 2);
    for tag in [
        &protagonist,
        &antagonist,
        &supporting,
        &active,
        &deceased,
        &unknown,
    ] {
        db::create_tag(&tx, tag)?;
    }
    for (character, role_tag, status_tag) in [
        (&eleanor, &protagonist, &active),
        (&thomas, &supporting, &active),
        (&margaret, &supporting, &unknown),
        (&silas, &antagonist, &active),
    ] {
        db::tag_entity(&tx, &role_tag.id, "character", &character.id)?;
        db::tag_entity(&tx, &status_tag.id, "character", &character.id)?;
    }
    db::save_filter(
        &tx,
        &SavedFilter::new(
            project.id,
            "Active protagonists".into(),
            "character".into(),
            serde_json::json!({"tags": [protagonist.id, active.id], "operator": "AND"}).to_string(),
            0,
        ),
    )?;

    let chapter = Chapter {
        synopsis: Some("Eleanor reads the letter, confronts Thomas, and finds the key.".into()),
        ..Chapter::new(project.id, "The Letter".into(), 0)
    };
    db::insert_chapter(&tx, &chapter)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(chapter.id, "On the Cliff".into(), Some("Eleanor discovers a mysterious letter revealing her father may not be who he claims.".into()), 0)
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(&tx, &Beat { prose: Some("<p>The lighthouse keeper's daughter stood at the edge of the cliff, her <em>heart pounding</em> against her ribs like a caged bird desperate for freedom. The wind whipped her dark hair across her face, carrying with it the salt-spray of a restless sea.</p><p><strong>Eleanor Blackwood</strong> had always known this day would come. The letter in her pocket—crumpled now from being read a dozen times—confirmed what she'd long suspected: <em>her father was not who he claimed to be.</em></p><p>She pulled the paper out once more, squinting at the elegant handwriting in the fading light:</p><blockquote><p>My dearest Eleanor,</p><p>If you are reading this, I have failed in my duty to protect you. The lighthouse holds secrets older than the stone it stands upon. Find the key beneath the seventh step. Trust no one—especially not the man who calls himself your father.</p><p>Your true guardian,</p><p>M.W.</p></blockquote><p>\"What secrets?\" she whispered to the churning waves below. <em>What could possibly be worth all this deception?</em></p>".into()), ..Beat::new(scene.id, "Eleanor reads the mysterious letter on the cliff".into(), 0) })?;
    db::insert_beat(&tx, &Beat { prose: Some("<p>The door to the lighthouse creaked open behind her. Eleanor spun around, her hand instinctively reaching for the pocket knife she always carried—a gift from the man she'd called \"Papa\" for eighteen years.</p><p>It was only Thomas, the relief keeper, with his coat buttoned wrong and his hair full of weather. He stopped when he saw her face.</p><p>\"You've found it, then,\" he said.</p><p>Not <em>what's wrong</em>. Not <em>Eleanor, you're white as the lamp glass</em>. He had gone straight to <em>found it</em>, and in the two seconds it took him to understand what he had said, she watched him try to take it back.</p><p>\"Found what, Thomas?\"</p><p>He looked at the sea. Everyone in Wrackham looked at the sea when they wanted to stop talking.</p><p>She unfolded the letter between them. \"M.W. means Margaret Wren, doesn't it? And what has Silas Blackwood been keeping from me?\" Thomas reached for the paper, then let his hand fall.</p>".into()), ..Beat::new(scene.id, "Thomas arrives and Eleanor confronts him".into(), 1) })?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "She counts the stairs on the way up and the number is wrong".into(),
                2,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &thomas.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &cliff.id)?;
    db::add_scene_reference_item_ref(&tx, &scene.id, &sealed_letter.id)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(
            chapter.id,
            "The Seventh Step".into(),
            Some(
                "The step lifts out. What is underneath it does not fit any lock in the building."
                    .into(),
            ),
            1,
        )
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(&tx, &Beat { prose: Some("<p>It came up without protest. That was the part that frightened her—not the hollow beneath, not the oilcloth bundle, but the clean grey line where the stone had been lifted and set back, lifted and set back, for longer than she had been alive.</p><p>Someone had been coming here. Someone had been coming here the whole time.</p>".into()), ..Beat::new(scene.id, "The seventh step lifts out cleanly, as though it has been lifted often".into(), 0) })?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "The key fits nothing: not the lamp room, not the oil store, not the quarters"
                    .into(),
                1,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "Silas calls up the stairwell and she puts the stone back before answering".into(),
                2,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "She wraps the key in her handkerchief before Silas reaches the landing".into(),
                3,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "A draught beneath the wall hints at a passage absent from the plans".into(),
                4,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &lighthouse.id)?;
    db::add_scene_reference_item_ref(&tx, &scene.id, &seventh_step_key.id)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(chapter.id, "Supper with Silas".into(), Some("Eleanor tests a harmless question and discovers how carefully Silas has rehearsed his answer.".into()), 2)
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(&tx, &Beat { prose: Some("<p>Silas laid three places, though Thomas had already gone. Eleanor watched the empty chair while he told her the staircase had never needed repair.</p>".into()), ..Beat::new(scene.id, "She asks when the seventh step was repaired".into(), 0) })?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &silas.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &lighthouse.id)?;
    let chapter = Chapter {
        synopsis: Some(
            "Wrackham closes ranks. Eleanor learns the name M.W. and what it costs to say it."
                .into(),
        ),
        ..Chapter::new(project.id, "What the Stone Remembers".into(), 1)
    };
    db::insert_chapter(&tx, &chapter)?;
    let scene = Scene {
        scene_status: SceneStatus::Revised,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(chapter.id, "Low Tide".into(), Some("At low water the foundation course is exposed, and the stones are older than 1798.".into()), 0)
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(&tx, &Beat { prose: Some("<p>The tide went out further than it had all year, and the light stood on something that was not its own foundation.</p><p>Eleanor crouched in the wrack and put her palm flat against the lowest course. Dressed stone, mortared, and weathered into softness—nothing like the hard Portland above it. Whoever built the lighthouse in 1798 had not started from nothing. They had started from whatever this was, and then they had built ninety-one feet of grey stone on top of it.</p>".into()), ..Beat::new(scene.id, "The foundation stones predate the tower by two centuries".into(), 0) })?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "Thomas admits he has seen Silas on the shore at night, counting".into(),
                1,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &thomas.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &harbour.id)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(
            chapter.id,
            "The Damaged Register".into(),
            Some(
                "A page has been cut from the parish register. The verger remembers who cut it."
                    .into(),
            ),
            1,
        )
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "The register is missing the page covering the spring Eleanor arrived".into(),
                0,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "The cut is clean and recent — a razor, not sixteen years of handling".into(),
                1,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "The verger gives her the name Margaret Wren and asks her not to come back".into(),
                2,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &margaret.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &harbour.id)?;
    let scene = Scene {
        scene_status: SceneStatus::Final,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Page,
        prose: Some("<p>Nothing in the room had been allowed to settle. The bed was turned down, the water jug full, the window unlatched. Someone had dusted the sill that morning.</p><p>Eleanor set the letter on the dressing table. The grain of the wood held a pale rectangle exactly its size.</p><p>For sixteen years she had thought herself abandoned. Here was the unbearable evidence of someone waiting.</p>".into()),
        ..Scene::new(chapter.id, "Margaret's Room".into(), Some("Above the chandlery, Eleanor finds a room that has been kept ready for sixteen years.".into()), 2)
    };
    db::insert_scene(&tx, &scene)?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &margaret.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &harbour.id)?;
    let chapter = Chapter {
        synopsis: Some(
            "Outline only. Silas's logbooks record a second, unexplained set of counts.".into(),
        ),
        ..Chapter::new(project.id, "The Keeper's Ledger".into(), 2)
    };
    db::insert_chapter(&tx, &chapter)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Fixed,
        editor_mode: EditorMode::Beat,
        ..Scene::new(chapter.id, "The Ledger".into(), Some("Sixteen years of keeper's logs, and in every one a column of numbers that has nothing to do with the light.".into()), 0)
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "Every log has a second column Silas has never explained".into(),
                0,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "The numbers are step counts, and they change".into(),
                1,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "Eleanor realises the tower is being measured, not tended".into(),
                2,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &silas.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &lighthouse.id)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Notes,
        planning_status: PlanningStatus::Flexible,
        editor_mode: EditorMode::Page,
        prose: Some("<p>Why does the ledger begin sixteen years ago? Check the spring tide against the date in the register. Decide whether Thomas recognises the handwriting before Eleanor does.</p>".into()),
        ..Scene::new(chapter.id, "Questions for the Keeper".into(), Some("Planning notes: reconcile the changing stair counts with the missing parish page.".into()), 1)
    };
    db::insert_scene(&tx, &scene)?;
    let scene = Scene {
        scene_status: SceneStatus::Draft,
        scene_type: SceneType::Normal,
        planning_status: PlanningStatus::Flexible,
        editor_mode: EditorMode::Beat,
        ..Scene::new(
            chapter.id,
            "Below the Light".into(),
            Some("Eleanor and Thomas follow the draught to a door beneath the foundations.".into()),
            2,
        )
    };
    db::insert_scene(&tx, &scene)?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "At low tide, the key turns in a lock buried beneath the tower".into(),
                0,
            )
        },
    )?;
    db::insert_beat(
        &tx,
        &Beat {
            prose: None,
            ..Beat::new(
                scene.id,
                "A lamp is burning on the other side of the door".into(),
                1,
            )
        },
    )?;
    db::add_scene_character_ref(&tx, &scene.id, &eleanor.id)?;
    db::add_scene_character_ref(&tx, &scene.id, &thomas.id)?;
    db::add_scene_location_ref(&tx, &scene.id, &lighthouse.id)?;
    db::add_scene_reference_item_ref(&tx, &scene.id, &seventh_step_key.id)?;
    tx.commit()?;
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Connection, Project) {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = seed_sample_project(&conn).unwrap();
        (conn, project)
    }

    #[test]
    fn narrative_has_varied_density_and_preserves_the_novelwriter_opening() {
        let (conn, project) = fixture();
        assert_eq!(project.name, "The Letter");
        assert_eq!(project.author_pen_name.as_deref(), Some("E. M. Hale"));
        assert_eq!(project.genre.as_deref(), Some("Gothic Mystery"));
        assert_eq!(project.word_target, Some(90000));
        let chapters = db::get_chapters(&conn, &project.id).unwrap();
        assert_eq!(
            chapters
                .iter()
                .map(|c| c.title.as_str())
                .collect::<Vec<_>>(),
            [
                "The Letter",
                "What the Stone Remembers",
                "The Keeper's Ledger"
            ]
        );
        let scenes = db::get_all_project_scenes(&conn, &project.id).unwrap();
        assert!(scenes.len() >= 8);
        let counts: Vec<_> = scenes
            .iter()
            .map(|s| db::get_beats(&conn, &s.id).unwrap().len())
            .collect();
        assert_eq!(counts.iter().min(), Some(&0));
        assert!(counts.iter().max().unwrap() >= &5);
        assert!(scenes.iter().any(|s| s.scene_type == SceneType::Notes));
        assert!(scenes.iter().any(|s| s.scene_status == SceneStatus::Final));
        assert!(scenes
            .iter()
            .any(|s| s.planning_status == PlanningStatus::Flexible));
        assert!(scenes
            .iter()
            .any(|s| s.editor_mode == EditorMode::Page && s.prose.is_some()));
        let first = &db::get_scenes(&conn, &chapters[0].id).unwrap()[0];
        assert_eq!(first.title, "On the Cliff");
        assert_eq!(first.scene_type, SceneType::Normal);
        assert_eq!(first.editor_mode, EditorMode::Beat);
        let beats = db::get_beats(&conn, &first.id).unwrap();
        assert_eq!(beats.len(), 3);
        assert!(beats[0].prose.as_ref().unwrap().contains("<blockquote>"));
        assert!(beats[0]
            .prose
            .as_ref()
            .unwrap()
            .contains("lighthouse keeper's daughter"));
        let prose = beats
            .iter()
            .filter_map(|b| b.prose.as_deref())
            .collect::<String>();
        let links = db::get_all_scene_character_refs(&conn, &project.id).unwrap();
        for name in ["Margaret Wren", "Silas Blackwood"] {
            let character = db::get_characters(&conn, &project.id)
                .unwrap()
                .into_iter()
                .find(|c| c.name == name)
                .unwrap();
            assert!(prose.contains(name));
            assert!(!links
                .iter()
                .any(|link| link.scene_id == first.id && link.character_id == character.id));
        }
    }

    #[test]
    fn references_have_typed_fields_and_a_working_hierarchical_filter() {
        let (conn, project) = fixture();
        let characters = db::get_characters(&conn, &project.id).unwrap();
        let locations = db::get_locations(&conn, &project.id).unwrap();
        let items = db::get_all_reference_items(&conn, &project.id).unwrap();
        assert!(characters.len() >= 4 && locations.len() >= 3 && items.len() >= 2);
        assert!(characters.iter().all(|c| c.attributes.is_empty()));
        assert!(locations.iter().all(|c| c.attributes.is_empty()));
        assert!(items.iter().all(|c| c.attributes.is_empty()));
        let definitions = db::get_field_definitions(&conn, &project.id, "character").unwrap();
        for kind in ["text", "number", "select", "multi_select"] {
            assert!(definitions.iter().any(|d| d.field_type == kind));
        }
        for character in &characters {
            let values = db::get_field_values(&conn, &character.id).unwrap();
            assert_eq!(values.len(), 4);
            for def in &definitions {
                let value = values
                    .iter()
                    .find(|v| v.field_definition_id == def.id)
                    .unwrap()
                    .value
                    .as_deref()
                    .unwrap();
                match def.field_type.as_str() {
                    "number" => {
                        value.parse::<u32>().unwrap();
                    }
                    "select" => {
                        let options: Vec<String> =
                            serde_json::from_str(def.options.as_ref().unwrap()).unwrap();
                        assert!(options.contains(&value.to_string()));
                    }
                    "multi_select" => {
                        let options: Vec<String> =
                            serde_json::from_str(def.options.as_ref().unwrap()).unwrap();
                        let selected: Vec<String> = serde_json::from_str(value).unwrap();
                        assert!(!selected.is_empty());
                        assert!(selected.iter().all(|v| options.contains(v)));
                    }
                    _ => assert!(!value.is_empty()),
                }
            }
        }
        let tags = db::get_tags(&conn, &project.id).unwrap();
        let roots: Vec<_> = tags.iter().filter(|t| t.parent_id.is_none()).collect();
        assert_eq!(roots.len(), 2);
        for root in roots {
            assert!(tags.iter().filter(|t| t.parent_id == Some(root.id)).count() >= 2);
        }
        assert!(tags.iter().any(|t| t.color.is_some()));
        assert!(
            db::get_all_entity_tags_for_project(&conn, &project.id)
                .unwrap()
                .len()
                >= 3
        );
        let filters = db::get_saved_filters(&conn, &project.id).unwrap();
        assert_eq!(filters.len(), 1);
        let matches = db::filter_entities(
            &conn,
            &project.id,
            &filters[0].entity_type,
            &filters[0].filter_json,
        )
        .unwrap();
        assert_eq!(
            matches,
            vec![
                characters
                    .iter()
                    .find(|c| c.name == "Eleanor Blackwood")
                    .unwrap()
                    .id
            ]
        );
        let second = seed_sample_project(&conn).unwrap();
        assert_ne!(project.id, second.id);
        assert_eq!(db::get_saved_filters(&conn, &second.id).unwrap().len(), 1);
    }
}
