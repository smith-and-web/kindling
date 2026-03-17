use std::collections::{HashMap, HashSet};

use rusqlite::{params, Connection, Result};
use uuid::Uuid;

use crate::db;
use crate::models::ReferenceSuggestion;

/// Entry in the name index: (reference_id, reference_type, display_name, confidence)
type NameEntry = (Uuid, String, String, f32);

/// Build a lookup from normalised name -> (reference_id, type, display_name, confidence).
///
/// Characters get their full name at 1.0 and, when the first token is unique across
/// all references, that first name at 0.7.  Locations and reference_items get their
/// full name at 1.0.
pub fn build_name_index(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<HashMap<String, NameEntry>> {
    let characters = db::get_characters(conn, project_id)?;
    let locations = db::get_locations(conn, project_id)?;
    let reference_items = db::get_all_reference_items(conn, project_id)?;

    let mut index: HashMap<String, NameEntry> = HashMap::new();

    // Collect all first-name tokens to detect uniqueness
    let mut first_name_counts: HashMap<String, u32> = HashMap::new();
    for ch in &characters {
        if let Some(first) = ch.name.split_whitespace().next() {
            let key = first.to_lowercase();
            *first_name_counts.entry(key).or_insert(0) += 1;
        }
    }
    for loc in &locations {
        if let Some(first) = loc.name.split_whitespace().next() {
            let key = first.to_lowercase();
            *first_name_counts.entry(key).or_insert(0) += 1;
        }
    }
    for ri in &reference_items {
        if let Some(first) = ri.name.split_whitespace().next() {
            let key = first.to_lowercase();
            *first_name_counts.entry(key).or_insert(0) += 1;
        }
    }

    for ch in &characters {
        let full_key = ch.name.trim().to_lowercase();
        if !full_key.is_empty() {
            index.insert(
                full_key.clone(),
                (ch.id, "character".to_string(), ch.name.clone(), 1.0),
            );
        }

        let parts: Vec<&str> = ch.name.split_whitespace().collect();
        if parts.len() > 1 {
            let first_key = parts[0].to_lowercase();
            if first_name_counts.get(&first_key).copied().unwrap_or(0) == 1
                && !index.contains_key(&first_key)
            {
                index.insert(
                    first_key,
                    (ch.id, "character".to_string(), ch.name.clone(), 0.7),
                );
            }
        }
    }

    for loc in &locations {
        let key = loc.name.trim().to_lowercase();
        if !key.is_empty() {
            index
                .entry(key)
                .or_insert((loc.id, "location".to_string(), loc.name.clone(), 1.0));
        }
    }

    for ri in &reference_items {
        let key = ri.name.trim().to_lowercase();
        if !key.is_empty() {
            index
                .entry(key)
                .or_insert((ri.id, ri.reference_type.clone(), ri.name.clone(), 1.0));
        }
    }

    Ok(index)
}

/// Strip HTML tags from a string, returning plain text.
pub fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }

    out
}

/// Detect references mentioned in a single scene's prose (and beat prose).
///
/// Returns suggestions for references that are not already linked and not dismissed.
pub fn detect_references(
    conn: &Connection,
    project_id: &Uuid,
    scene_id: &Uuid,
) -> Result<Vec<ReferenceSuggestion>> {
    let name_index = build_name_index(conn, project_id)?;
    detect_references_with_index(conn, &name_index, scene_id)
}

/// Internal: run detection for a single scene using a pre-built index.
fn detect_references_with_index(
    conn: &Connection,
    name_index: &HashMap<String, NameEntry>,
    scene_id: &Uuid,
) -> Result<Vec<ReferenceSuggestion>> {
    // Gather prose text
    let mut prose_parts: Vec<String> = Vec::new();

    if let Some(scene) = db::get_scene_by_id(conn, scene_id)? {
        if let Some(p) = &scene.prose {
            prose_parts.push(p.clone());
        }
    }

    let beats = db::get_beats(conn, scene_id)?;
    for beat in &beats {
        if let Some(p) = &beat.prose {
            prose_parts.push(p.clone());
        }
    }

    let raw_html = prose_parts.join(" ");
    if raw_html.is_empty() {
        return Ok(Vec::new());
    }

    let plain = strip_html(&raw_html);

    // Tokenise on whitespace, keeping track of character offsets
    let tokens: Vec<(usize, &str)> = plain
        .split_whitespace()
        .map(|tok| {
            let offset = tok.as_ptr() as usize - plain.as_ptr() as usize;
            (offset, tok)
        })
        .collect();

    // Scan n-grams (1..=3 words) for matches
    let mut hits: HashMap<Uuid, (ReferenceSuggestion, f32)> = HashMap::new();
    let max_n = 3.min(tokens.len());

    for n in 1..=max_n {
        for window in tokens.windows(n) {
            let combined: String = window.iter().map(|(_, t)| *t).collect::<Vec<_>>().join(" ");

            let normalized = combined.to_lowercase();
            // Strip trailing punctuation for matching
            let trimmed = normalized.trim_end_matches(|c: char| c.is_ascii_punctuation());

            if let Some((ref_id, ref_type, display_name, confidence)) = name_index.get(trimmed) {
                let pos = window[0].0;

                let entry = hits.entry(*ref_id).or_insert_with(|| {
                    (
                        ReferenceSuggestion {
                            reference_id: ref_id.to_string(),
                            reference_type: ref_type.clone(),
                            reference_name: display_name.clone(),
                            match_text: combined.clone(),
                            positions: Vec::new(),
                            confidence: *confidence,
                        },
                        *confidence,
                    )
                });

                entry.0.positions.push(pos);
                if *confidence > entry.1 {
                    entry.0.confidence = *confidence;
                    entry.0.match_text = combined.clone();
                    entry.1 = *confidence;
                }
            }
        }
    }

    if hits.is_empty() {
        return Ok(Vec::new());
    }

    // Filter out already-linked references
    let linked_characters: HashSet<Uuid> = db::get_scene_characters(conn, scene_id)?
        .into_iter()
        .collect();
    let linked_locations: HashSet<Uuid> = db::get_scene_locations(conn, scene_id)?
        .into_iter()
        .collect();

    let linked_ref_items: HashSet<Uuid> = {
        let mut stmt = conn.prepare(
            "SELECT reference_item_id FROM scene_reference_item_refs WHERE scene_id = ?1",
        )?;
        let ids: Vec<Uuid> = stmt
            .query_map(params![scene_id.to_string()], |row| {
                crate::db::queries::parse_uuid(&row.get::<_, String>(0)?)
            })?
            .filter_map(|r| r.ok())
            .collect();
        ids.into_iter().collect()
    };

    let mut linked: HashSet<Uuid> = HashSet::new();
    linked.extend(linked_characters);
    linked.extend(linked_locations);
    linked.extend(linked_ref_items);

    // Filter out dismissed suggestions
    let dismissed: HashSet<Uuid> = db::get_dismissed_suggestions(conn, scene_id)?
        .into_iter()
        .map(|(_, ref_id)| ref_id)
        .collect();

    let suggestions: Vec<ReferenceSuggestion> = hits
        .into_values()
        .map(|(s, _)| s)
        .filter(|s| {
            let id = Uuid::parse_str(&s.reference_id).unwrap_or(Uuid::nil());
            !linked.contains(&id) && !dismissed.contains(&id)
        })
        .collect();

    Ok(suggestions)
}

/// Detect references across all scenes in a project.
pub fn detect_all_references(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<HashMap<Uuid, Vec<ReferenceSuggestion>>> {
    let name_index = build_name_index(conn, project_id)?;
    let scenes = db::get_all_scenes_including_archived(conn, project_id)?;

    let mut result: HashMap<Uuid, Vec<ReferenceSuggestion>> = HashMap::new();

    for scene in &scenes {
        let suggestions = detect_references_with_index(conn, &name_index, &scene.id)?;
        if !suggestions.is_empty() {
            result.insert(scene.id, suggestions);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::initialize_schema;
    use crate::models::{Beat, Chapter, Character, Location, ReferenceItem, Scene};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        initialize_schema(&conn).unwrap();
        conn
    }

    fn insert_test_project(conn: &Connection) -> Uuid {
        let id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO projects (id, name, source_type, created_at, modified_at) VALUES (?1, 'Test', 'Blank', datetime('now'), datetime('now'))",
            params![id.to_string()],
        ).unwrap();
        id
    }

    // ---- strip_html tests ----

    #[test]
    fn test_strip_html_plain_text() {
        assert_eq!(strip_html("hello world"), "hello world");
    }

    #[test]
    fn test_strip_html_basic_tags() {
        let result = strip_html("<p>Hello <strong>world</strong></p>");
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
        assert!(!result.contains('<'));
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn test_strip_html_nested_tags() {
        let html = "<div><p>Some <em>italic</em> text</p></div>";
        let result = strip_html(html);
        assert!(result.contains("Some"));
        assert!(result.contains("italic"));
        assert!(result.contains("text"));
        assert!(!result.contains('<'));
    }

    // ---- build_name_index tests ----

    #[test]
    fn test_build_name_index_characters() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let ch = Character::new(project_id, "Alice Wonderland".to_string(), None, None);
        db::insert_character(&conn, &ch).unwrap();

        let index = build_name_index(&conn, &project_id).unwrap();

        // Full name match
        assert!(index.contains_key("alice wonderland"));
        let entry = &index["alice wonderland"];
        assert_eq!(entry.0, ch.id);
        assert_eq!(entry.1, "character");
        assert!((entry.3 - 1.0).abs() < f32::EPSILON);

        // First-name match (unique, so should be present)
        assert!(index.contains_key("alice"));
        let first = &index["alice"];
        assert_eq!(first.0, ch.id);
        assert!((first.3 - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn test_build_name_index_no_first_name_when_ambiguous() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let ch1 = Character::new(project_id, "Alice Wonderland".to_string(), None, None);
        let ch2 = Character::new(project_id, "Alice Smith".to_string(), None, None);
        db::insert_character(&conn, &ch1).unwrap();
        db::insert_character(&conn, &ch2).unwrap();

        let index = build_name_index(&conn, &project_id).unwrap();

        // Full names present
        assert!(index.contains_key("alice wonderland"));
        assert!(index.contains_key("alice smith"));
        // First name NOT present because "alice" is ambiguous
        assert!(!index.contains_key("alice"));
    }

    #[test]
    fn test_build_name_index_locations_and_items() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let loc = Location::new(project_id, "Dark Forest".to_string(), None, None);
        db::insert_location(&conn, &loc).unwrap();

        let item = ReferenceItem::new(
            project_id,
            "items".to_string(),
            "Magic Sword".to_string(),
            None,
            None,
        );
        db::insert_reference_item(&conn, &item).unwrap();

        let index = build_name_index(&conn, &project_id).unwrap();

        assert!(index.contains_key("dark forest"));
        assert!(index.contains_key("magic sword"));
    }

    // ---- detect_references tests ----

    #[test]
    fn test_detect_references_basic() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let chapter = Chapter::new(project_id, "Ch 1".to_string(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();

        let mut scene = Scene::new(chapter.id, "Scene 1".to_string(), None, 0);
        scene.prose = Some("<p>Alice walked into the Dark Forest.</p>".to_string());
        db::insert_scene(&conn, &scene).unwrap();

        let ch = Character::new(project_id, "Alice".to_string(), None, None);
        db::insert_character(&conn, &ch).unwrap();

        let loc = Location::new(project_id, "Dark Forest".to_string(), None, None);
        db::insert_location(&conn, &loc).unwrap();

        let suggestions = detect_references(&conn, &project_id, &scene.id).unwrap();

        let ref_ids: HashSet<String> = suggestions.iter().map(|s| s.reference_id.clone()).collect();
        assert!(ref_ids.contains(&ch.id.to_string()));
        assert!(ref_ids.contains(&loc.id.to_string()));
    }

    #[test]
    fn test_detect_references_filters_linked() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let chapter = Chapter::new(project_id, "Ch 1".to_string(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();

        let mut scene = Scene::new(chapter.id, "Scene 1".to_string(), None, 0);
        scene.prose = Some("<p>Alice and Bob went to the park.</p>".to_string());
        db::insert_scene(&conn, &scene).unwrap();

        let alice = Character::new(project_id, "Alice".to_string(), None, None);
        let bob = Character::new(project_id, "Bob".to_string(), None, None);
        db::insert_character(&conn, &alice).unwrap();
        db::insert_character(&conn, &bob).unwrap();

        // Link Alice to the scene — she should be filtered out
        db::add_scene_character_ref(&conn, &scene.id, &alice.id).unwrap();

        let suggestions = detect_references(&conn, &project_id, &scene.id).unwrap();

        let ref_ids: HashSet<String> = suggestions.iter().map(|s| s.reference_id.clone()).collect();
        assert!(!ref_ids.contains(&alice.id.to_string()));
        assert!(ref_ids.contains(&bob.id.to_string()));
    }

    #[test]
    fn test_detect_references_filters_dismissed() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let chapter = Chapter::new(project_id, "Ch 1".to_string(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();

        let mut scene = Scene::new(chapter.id, "Scene 1".to_string(), None, 0);
        scene.prose = Some("<p>Alice talked to Bob.</p>".to_string());
        db::insert_scene(&conn, &scene).unwrap();

        let alice = Character::new(project_id, "Alice".to_string(), None, None);
        let bob = Character::new(project_id, "Bob".to_string(), None, None);
        db::insert_character(&conn, &alice).unwrap();
        db::insert_character(&conn, &bob).unwrap();

        // Dismiss Bob
        db::dismiss_suggestion(&conn, &scene.id, &bob.id).unwrap();

        let suggestions = detect_references(&conn, &project_id, &scene.id).unwrap();

        let ref_ids: HashSet<String> = suggestions.iter().map(|s| s.reference_id.clone()).collect();
        assert!(ref_ids.contains(&alice.id.to_string()));
        assert!(!ref_ids.contains(&bob.id.to_string()));
    }

    #[test]
    fn test_detect_references_from_beat_prose() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let chapter = Chapter::new(project_id, "Ch 1".to_string(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();

        let scene = Scene::new(chapter.id, "Scene 1".to_string(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();

        let mut beat = Beat::new(scene.id, "outline beat".to_string(), 0);
        beat.prose = Some("<p>Alice entered the room.</p>".to_string());
        db::insert_beat(&conn, &beat).unwrap();

        let ch = Character::new(project_id, "Alice".to_string(), None, None);
        db::insert_character(&conn, &ch).unwrap();

        let suggestions = detect_references(&conn, &project_id, &scene.id).unwrap();

        let ref_ids: HashSet<String> = suggestions.iter().map(|s| s.reference_id.clone()).collect();
        assert!(ref_ids.contains(&ch.id.to_string()));
    }

    #[test]
    fn test_detect_references_empty_prose() {
        let conn = setup_test_db();
        let project_id = insert_test_project(&conn);

        let chapter = Chapter::new(project_id, "Ch 1".to_string(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();

        let scene = Scene::new(chapter.id, "Scene 1".to_string(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();

        let ch = Character::new(project_id, "Alice".to_string(), None, None);
        db::insert_character(&conn, &ch).unwrap();

        let suggestions = detect_references(&conn, &project_id, &scene.id).unwrap();
        assert!(suggestions.is_empty());
    }
}
