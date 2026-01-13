use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::models::{Beat, Chapter, Character, Location, Project, Scene, SourceType};

#[derive(Debug, Error)]
pub enum PlottrError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid Plottr file structure: {0}")]
    InvalidStructure(String),
}

// ============================================================================
// Plottr File Structure (from real .pltr JSON files)
// ============================================================================

/// Top-level structure of a .pltr file
#[derive(Debug, Deserialize)]
pub struct PlottrFile {
    /// File metadata (version, filename, etc.)
    #[serde(default)]
    pub file: Option<PlottrFileMetadata>,
    /// Series/project metadata
    #[serde(default)]
    pub series: Option<PlottrSeries>,
    /// Books container (dict with book IDs as keys + "allIds" array)
    #[serde(default)]
    pub books: Option<serde_json::Value>,
    /// Beats organized by book ID (dict of dicts with children/heap/index)
    #[serde(default)]
    pub beats: Option<serde_json::Value>,
    /// Scene cards
    #[serde(default)]
    pub cards: Vec<PlottrCard>,
    /// Plotlines/storylines
    #[serde(default)]
    pub lines: Vec<PlottrLine>,
    /// Characters
    #[serde(default)]
    pub characters: Vec<PlottrCharacter>,
    /// Places/locations
    #[serde(default)]
    pub places: Vec<PlottrPlace>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<PlottrTag>,
    /// Custom attribute definitions
    #[serde(rename = "customAttributes", default)]
    pub custom_attributes: Option<serde_json::Value>,
    /// Notes
    #[serde(default)]
    pub notes: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PlottrFileMetadata {
    #[serde(rename = "fileName")]
    pub file_name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlottrSeries {
    pub name: Option<String>,
    pub premise: Option<String>,
    pub genre: Option<String>,
    pub theme: Option<String>,
}

/// A beat in Plottr represents a chapter/act in the timeline
#[derive(Debug, Deserialize)]
pub struct PlottrBeat {
    pub id: serde_json::Value,
    #[serde(rename = "bookId")]
    pub book_id: Option<serde_json::Value>,
    #[serde(default)]
    pub position: i32,
    #[serde(default)]
    pub title: String,
}

/// A plotline/storyline
#[derive(Debug, Deserialize)]
pub struct PlottrLine {
    pub id: serde_json::Value,
    pub title: String,
    pub color: Option<String>,
    #[serde(default)]
    pub position: i32,
    #[serde(rename = "bookId")]
    pub book_id: Option<serde_json::Value>,
    #[serde(rename = "characterId")]
    pub character_id: Option<serde_json::Value>,
}

/// A scene card
#[derive(Debug, Deserialize)]
pub struct PlottrCard {
    pub id: serde_json::Value,
    #[serde(rename = "lineId")]
    pub line_id: serde_json::Value,
    /// Links card to a beat (chapter/act)
    #[serde(rename = "beatId")]
    pub beat_id: serde_json::Value,
    #[serde(rename = "bookId")]
    pub book_id: Option<serde_json::Value>,
    pub title: String,
    /// Rich text description (array of paragraph objects)
    pub description: Option<serde_json::Value>,
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
    #[serde(default)]
    pub characters: Vec<serde_json::Value>,
    #[serde(default)]
    pub places: Vec<serde_json::Value>,
    #[serde(default)]
    pub position: i32,
    #[serde(rename = "positionWithinLine", default)]
    pub position_within_line: i32,
    #[serde(rename = "positionInChapter", default)]
    pub position_in_chapter: i32,
}

/// A character
#[derive(Debug, Deserialize)]
pub struct PlottrCharacter {
    pub id: serde_json::Value,
    pub name: String,
    /// Plain text or rich text description
    pub description: Option<serde_json::Value>,
    /// Rich text notes (array of paragraph objects)
    pub notes: Option<serde_json::Value>,
    pub color: Option<String>,
    #[serde(default)]
    pub cards: Vec<serde_json::Value>,
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
    #[serde(rename = "categoryId")]
    pub category_id: Option<serde_json::Value>,
    #[serde(rename = "imageId")]
    pub image_id: Option<serde_json::Value>,
    /// Custom attributes are stored directly on the character object
    #[serde(flatten)]
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

/// A place/location
#[derive(Debug, Deserialize)]
pub struct PlottrPlace {
    pub id: serde_json::Value,
    pub name: String,
    /// Plain text or rich text description
    pub description: Option<serde_json::Value>,
    /// Rich text notes (array of paragraph objects)
    pub notes: Option<serde_json::Value>,
    pub color: Option<String>,
    #[serde(default)]
    pub cards: Vec<serde_json::Value>,
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
    #[serde(rename = "imageId")]
    pub image_id: Option<serde_json::Value>,
    #[serde(flatten)]
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

/// A tag for categorization
#[derive(Debug, Deserialize)]
pub struct PlottrTag {
    pub id: serde_json::Value,
    pub title: String,
    pub color: Option<String>,
}

// ============================================================================
// Parsed Output
// ============================================================================

pub struct ParsedPlottr {
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub beats: Vec<Beat>,
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
    pub scene_character_refs: Vec<(uuid::Uuid, uuid::Uuid)>,
    pub scene_location_refs: Vec<(uuid::Uuid, uuid::Uuid)>,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a serde_json::Value to a String ID
fn value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Null => "null".to_string(),
        _ => val.to_string(),
    }
}

/// Extract plain text from Plottr's rich text format
/// Rich text is stored as an array of paragraph objects:
/// [{"type": "paragraph", "children": [{"text": "content"}]}]
fn extract_text_from_rich_text(value: &serde_json::Value) -> Option<String> {
    match value {
        // Plain string
        serde_json::Value::String(s) => Some(s.clone()),
        // Rich text array
        serde_json::Value::Array(paragraphs) => {
            let mut para_texts = Vec::new();
            for para in paragraphs {
                if let Some(children) = para.get("children") {
                    if let Some(children_arr) = children.as_array() {
                        // Concatenate all text within a paragraph (inline text)
                        let para_text: String = children_arr
                            .iter()
                            .filter_map(|child| child.get("text").and_then(|t| t.as_str()))
                            .collect();
                        if !para_text.is_empty() {
                            para_texts.push(para_text);
                        }
                    }
                }
            }
            if para_texts.is_empty() {
                None
            } else {
                // Join paragraphs with newlines
                Some(para_texts.join("\n"))
            }
        }
        _ => None,
    }
}

/// Extract custom attribute value as a string
fn extract_attribute_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(if *b { "Yes" } else { "No" }.to_string()),
        serde_json::Value::Array(_) => extract_text_from_rich_text(value),
        _ => None,
    }
}

/// Parse beats from the nested structure
/// Beats structure: {"1": {"children": {...}, "heap": {...}, "index": {beat_id: beat_data}}, "series": {...}}
fn parse_beats_from_structure(beats_value: &serde_json::Value) -> Vec<PlottrBeat> {
    let mut beats = Vec::new();

    if let Some(beats_obj) = beats_value.as_object() {
        for (book_id, book_beats) in beats_obj {
            // Skip non-numeric book IDs like "series" for now
            if book_id == "series" {
                continue;
            }

            if let Some(index) = book_beats.get("index") {
                if let Some(index_obj) = index.as_object() {
                    for (_beat_id, beat_data) in index_obj {
                        if let Ok(beat) = serde_json::from_value::<PlottrBeat>(beat_data.clone()) {
                            beats.push(beat);
                        }
                    }
                }
            }
        }
    }

    beats.sort_by_key(|b| b.position);
    beats
}

// ============================================================================
// Parser Implementation
// ============================================================================

pub fn parse_plottr_file<P: AsRef<Path>>(path: P) -> Result<ParsedPlottr, PlottrError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    let plottr: PlottrFile = serde_json::from_str(&content)?;

    // Extract project name - prefer series name, fall back to filename
    let project_name = plottr
        .series
        .as_ref()
        .and_then(|s| s.name.clone())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    let project = Project::new(
        project_name,
        SourceType::Plottr,
        Some(path.to_string_lossy().to_string()),
    );

    // Parse beats (these become our chapters)
    let plottr_beats = plottr
        .beats
        .as_ref()
        .map(parse_beats_from_structure)
        .unwrap_or_default();

    // Create chapters from beats
    let mut chapters: Vec<Chapter> = plottr_beats
        .iter()
        .enumerate()
        .map(|(idx, beat)| {
            let title = if beat.title.is_empty() || beat.title == "auto" {
                format!("Chapter {}", idx + 1)
            } else {
                beat.title.clone()
            };
            Chapter::new(project.id, title, beat.position.max(idx as i32))
        })
        .collect();

    chapters.sort_by_key(|c| c.position);

    // Build beat ID -> Chapter map
    let beat_to_chapter: HashMap<String, &Chapter> = plottr_beats
        .iter()
        .zip(chapters.iter())
        .map(|(plottr_beat, ch)| (value_to_string(&plottr_beat.id), ch))
        .collect();

    // Parse characters
    let characters: Vec<Character> = plottr
        .characters
        .iter()
        .map(|pc| {
            let mut attrs = HashMap::new();

            // Extract notes as an attribute
            if let Some(notes) = &pc.notes {
                if let Some(text) = extract_text_from_rich_text(notes) {
                    attrs.insert("notes".to_string(), text);
                }
            }

            // Known fields to exclude from custom attributes
            let known_fields = [
                "id",
                "name",
                "description",
                "notes",
                "color",
                "cards",
                "noteIds",
                "templates",
                "tags",
                "categoryId",
                "imageId",
                "bookIds",
            ];

            // Add custom attributes
            for (key, value) in &pc.custom_attributes {
                if !known_fields.contains(&key.as_str()) {
                    if let Some(attr_value) = extract_attribute_value(value) {
                        attrs.insert(key.clone(), attr_value);
                    }
                }
            }

            // Extract description (can be plain text or rich text)
            let description = pc.description.as_ref().and_then(extract_attribute_value);

            Character::new(
                project.id,
                pc.name.clone(),
                description,
                Some(value_to_string(&pc.id)),
            )
            .with_attributes(attrs)
        })
        .collect();

    // Build character ID map
    let character_map: HashMap<String, &Character> = plottr
        .characters
        .iter()
        .zip(characters.iter())
        .map(|(pc, ch)| (value_to_string(&pc.id), ch))
        .collect();

    // Parse locations (places)
    let locations: Vec<Location> = plottr
        .places
        .iter()
        .map(|pp| {
            let mut attrs = HashMap::new();

            // Extract notes as an attribute
            if let Some(notes) = &pp.notes {
                if let Some(text) = extract_text_from_rich_text(notes) {
                    attrs.insert("notes".to_string(), text);
                }
            }

            let known_fields = [
                "id",
                "name",
                "description",
                "notes",
                "color",
                "cards",
                "noteIds",
                "templates",
                "tags",
                "imageId",
                "bookIds",
            ];

            for (key, value) in &pp.custom_attributes {
                if !known_fields.contains(&key.as_str()) {
                    if let Some(attr_value) = extract_attribute_value(value) {
                        attrs.insert(key.clone(), attr_value);
                    }
                }
            }

            let description = pp.description.as_ref().and_then(extract_attribute_value);

            Location::new(
                project.id,
                pp.name.clone(),
                description,
                Some(value_to_string(&pp.id)),
            )
            .with_attributes(attrs)
        })
        .collect();

    // Build location ID map
    let location_map: HashMap<String, &Location> = plottr
        .places
        .iter()
        .zip(locations.iter())
        .map(|(pp, loc)| (value_to_string(&pp.id), loc))
        .collect();

    // Parse cards as scenes (grouping by beat)
    let mut scenes: Vec<Scene> = Vec::new();
    let mut beats: Vec<Beat> = Vec::new();
    let mut scene_character_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();
    let mut scene_location_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();

    // Group cards by beat ID
    let mut cards_by_beat: HashMap<String, Vec<&PlottrCard>> = HashMap::new();
    for card in &plottr.cards {
        let beat_id = value_to_string(&card.beat_id);
        cards_by_beat.entry(beat_id).or_default().push(card);
    }

    // Create scenes from cards
    for (beat_id_str, cards) in cards_by_beat {
        if let Some(chapter) = beat_to_chapter.get(&beat_id_str) {
            let mut sorted_cards = cards;
            sorted_cards.sort_by_key(|c| (c.position_within_line, c.position));

            for (idx, card) in sorted_cards.iter().enumerate() {
                // Extract description text
                let synopsis = card
                    .description
                    .as_ref()
                    .and_then(extract_text_from_rich_text);

                let scene =
                    Scene::new(chapter.id, card.title.clone(), synopsis.clone(), idx as i32);

                // Create a beat from the card description if present
                if let Some(desc) = &synopsis {
                    if !desc.trim().is_empty() {
                        let beat = Beat::new(scene.id, desc.clone(), 0);
                        beats.push(beat);
                    }
                }

                // Track character references
                for char_id in &card.characters {
                    let id_str = value_to_string(char_id);
                    if let Some(character) = character_map.get(&id_str) {
                        scene_character_refs.push((scene.id, character.id));
                    }
                }

                // Track location references
                for place_id in &card.places {
                    let id_str = value_to_string(place_id);
                    if let Some(location) = location_map.get(&id_str) {
                        scene_location_refs.push((scene.id, location.id));
                    }
                }

                scenes.push(scene);
            }
        }
    }

    Ok(ParsedPlottr {
        project,
        chapters,
        scenes,
        beats,
        characters,
        locations,
        scene_character_refs,
        scene_location_refs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("fixtures");
        path.push(name);
        path
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(value_to_string(&serde_json::json!("test")), "test");
        assert_eq!(value_to_string(&serde_json::json!(123)), "123");
        assert_eq!(value_to_string(&serde_json::json!(12.5)), "12.5");
        assert_eq!(value_to_string(&serde_json::Value::Null), "null");
    }

    #[test]
    fn test_extract_text_from_rich_text_plain_string() {
        let value = serde_json::json!("Simple text");
        assert_eq!(
            extract_text_from_rich_text(&value),
            Some("Simple text".to_string())
        );
    }

    #[test]
    fn test_extract_text_from_rich_text_array() {
        let value = serde_json::json!([
            {"type": "paragraph", "children": [{"text": "First paragraph"}]},
            {"type": "paragraph", "children": [{"text": "Second paragraph"}]}
        ]);
        assert_eq!(
            extract_text_from_rich_text(&value),
            Some("First paragraph\nSecond paragraph".to_string())
        );
    }

    #[test]
    fn test_extract_text_from_rich_text_with_formatting() {
        let value = serde_json::json!([
            {"type": "paragraph", "children": [
                {"text": "Normal "},
                {"text": "bold", "bold": true},
                {"text": " text"}
            ]}
        ]);
        // Inline text within a paragraph should be concatenated without newlines
        assert_eq!(
            extract_text_from_rich_text(&value),
            Some("Normal bold text".to_string())
        );
    }

    #[test]
    fn test_parse_hamlet_file() {
        let path = fixture_path("hamlet.pltr");
        let result = parse_plottr_file(&path);

        assert!(
            result.is_ok(),
            "Failed to parse hamlet.pltr: {:?}",
            result.err()
        );
        let parsed = result.unwrap();

        // Project
        assert_eq!(parsed.project.name, "Hamlet");

        // Characters (19 in Hamlet)
        assert_eq!(parsed.characters.len(), 19);

        // Verify Hamlet character
        let hamlet = parsed.characters.iter().find(|c| c.name == "Hamlet");
        assert!(hamlet.is_some(), "Should find Hamlet character");
        let hamlet = hamlet.unwrap();
        assert_eq!(hamlet.description, Some("Prince of Denmark".to_string()));

        // Check custom attributes
        assert_eq!(
            hamlet.attributes.get("Role"),
            Some(&"Protagonist".to_string())
        );
        assert_eq!(hamlet.attributes.get("Gender"), Some(&"Male".to_string()));

        // Locations (5 in Hamlet)
        assert_eq!(parsed.locations.len(), 5);
        let elsinore = parsed
            .locations
            .iter()
            .find(|l| l.name == "Elsinore Castle");
        assert!(elsinore.is_some(), "Should find Elsinore Castle location");

        // Chapters (5 acts)
        assert_eq!(parsed.chapters.len(), 5);

        // Verify acts
        let act_titles: Vec<&str> = parsed.chapters.iter().map(|c| c.title.as_str()).collect();
        assert!(act_titles.contains(&"Act 1"), "Should have Act 1");
        assert!(act_titles.contains(&"Act 5"), "Should have Act 5");

        // Scenes (25 cards)
        assert_eq!(parsed.scenes.len(), 25);

        // Character references in scenes
        assert!(
            !parsed.scene_character_refs.is_empty(),
            "Should have character refs"
        );

        // Location references in scenes
        assert!(
            !parsed.scene_location_refs.is_empty(),
            "Should have location refs"
        );
    }

    #[test]
    fn test_parse_hamlet_character_attributes() {
        let path = fixture_path("hamlet.pltr");
        let parsed = parse_plottr_file(&path).expect("Failed to parse hamlet.pltr");

        // Check that custom attributes are properly parsed
        let claudius = parsed.characters.iter().find(|c| c.name == "Claudius");
        assert!(claudius.is_some());
        let claudius = claudius.unwrap();

        // Should have custom attributes like Role, Gender, etc.
        assert!(
            claudius.attributes.contains_key("Role"),
            "Claudius should have Role attribute"
        );
        assert!(
            claudius.attributes.contains_key("Gender"),
            "Claudius should have Gender attribute"
        );
    }

    #[test]
    fn test_parse_hamlet_scene_relationships() {
        let path = fixture_path("hamlet.pltr");
        let parsed = parse_plottr_file(&path).expect("Failed to parse hamlet.pltr");

        // The duel scene should reference multiple characters and Elsinore Castle
        let duel_scene = parsed.scenes.iter().find(|s| s.title.contains("duel"));
        assert!(duel_scene.is_some(), "Should find duel scene");
        let duel_scene = duel_scene.unwrap();

        // Count character refs for this scene
        let char_refs: Vec<_> = parsed
            .scene_character_refs
            .iter()
            .filter(|(scene_id, _)| *scene_id == duel_scene.id)
            .collect();

        assert!(
            char_refs.len() >= 2,
            "Duel scene should have multiple character refs"
        );

        // Count location refs for this scene
        let loc_refs: Vec<_> = parsed
            .scene_location_refs
            .iter()
            .filter(|(scene_id, _)| *scene_id == duel_scene.id)
            .collect();

        assert!(!loc_refs.is_empty(), "Duel scene should have location refs");
    }

    #[test]
    fn test_parse_hamlet_project_metadata() {
        let path = fixture_path("hamlet.pltr");
        let parsed = parse_plottr_file(&path).expect("Failed to parse hamlet.pltr");

        assert_eq!(parsed.project.name, "Hamlet");
        assert!(parsed.project.source_path.is_some());
        assert!(parsed
            .project
            .source_path
            .as_ref()
            .unwrap()
            .contains("hamlet.pltr"));
    }

    #[test]
    fn test_parse_plottr_file_structure() {
        // Test that we can at least deserialize the file structure correctly
        let path = fixture_path("hamlet.pltr");
        let content = std::fs::read_to_string(&path).expect("Failed to read hamlet.pltr");
        let plottr: PlottrFile = serde_json::from_str(&content).expect("Failed to parse JSON");

        // Should have tags
        assert_eq!(plottr.tags.len(), 13, "Hamlet should have 13 tags");

        // First tag should be a stage marker
        assert!(
            plottr.tags[0].title.contains("STAGE"),
            "First tag should be a stage marker"
        );

        // Should have lines (storylines)
        assert_eq!(plottr.lines.len(), 3, "Hamlet should have 3 plotlines");

        // Should have series metadata
        assert!(plottr.series.is_some());
        assert_eq!(
            plottr.series.as_ref().unwrap().name,
            Some("Hamlet".to_string())
        );

        // Should have cards
        assert_eq!(plottr.cards.len(), 25, "Hamlet should have 25 scene cards");
    }

    #[test]
    fn test_parse_plottr_lines() {
        let path = fixture_path("hamlet.pltr");
        let content = std::fs::read_to_string(&path).expect("Failed to read hamlet.pltr");
        let plottr: PlottrFile = serde_json::from_str(&content).expect("Failed to parse JSON");

        // Check plotline titles
        let line_titles: Vec<&str> = plottr.lines.iter().map(|l| l.title.as_str()).collect();
        assert!(
            line_titles.contains(&"Scenes"),
            "Should have Scenes plotline"
        );
        assert!(
            line_titles.contains(&"Main Plot"),
            "Should have Main Plot plotline"
        );
    }

    #[test]
    fn test_beats_are_properly_parsed_as_chapters() {
        let path = fixture_path("hamlet.pltr");
        let parsed = parse_plottr_file(&path).expect("Failed to parse hamlet.pltr");

        // Hamlet has 5 acts which should map to 5 chapters
        assert_eq!(parsed.chapters.len(), 5);

        // They should be in order
        for (i, chapter) in parsed.chapters.iter().enumerate() {
            assert_eq!(
                chapter.position, i as i32,
                "Chapter {} should have position {}",
                chapter.title, i
            );
        }
    }
}
