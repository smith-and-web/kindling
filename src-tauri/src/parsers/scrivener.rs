use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::models::{Beat, Chapter, Character, Location, Project, Scene, SourceType};

#[derive(Debug, Error)]
pub enum ScrivenerError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] quick_xml::Error),
    #[error("Invalid Scrivener project structure")]
    InvalidStructure,
    #[error("project.scrivx not found")]
    ProjectFileNotFound,
}

// ============================================================================
// Scrivener Binder Item Types
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum BinderItemType {
    DraftFolder,
    Folder,
    Text,
    CharacterSheet,
    LocationSheet,
    ResearchFolder,
    TrashFolder,
    Other(String),
}

impl From<&str> for BinderItemType {
    fn from(s: &str) -> Self {
        match s {
            "DraftFolder" => BinderItemType::DraftFolder,
            "Folder" => BinderItemType::Folder,
            "Text" => BinderItemType::Text,
            "CharacterSheet" => BinderItemType::CharacterSheet,
            "LocationSheet" => BinderItemType::LocationSheet,
            "ResearchFolder" => BinderItemType::ResearchFolder,
            "TrashFolder" => BinderItemType::TrashFolder,
            other => BinderItemType::Other(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinderItem {
    pub uuid: String,
    pub item_type: BinderItemType,
    pub title: String,
    pub children: Vec<BinderItem>,
}

// ============================================================================
// Parsed Output
// ============================================================================

pub struct ParsedScrivener {
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub beats: Vec<Beat>,
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
}

// ============================================================================
// Parser Implementation
// ============================================================================

fn find_scrivx_file(scriv_path: &Path) -> Option<PathBuf> {
    for entry in WalkDir::new(scriv_path).max_depth(1).into_iter().flatten() {
        if entry.path().extension().is_some_and(|ext| ext == "scrivx") {
            return Some(entry.path().to_path_buf());
        }
    }
    None
}

fn read_synopsis(scriv_path: &Path, uuid: &str) -> Option<String> {
    let synopsis_path = scriv_path
        .join("Files")
        .join("Data")
        .join(uuid)
        .join("synopsis.txt");

    fs::read_to_string(synopsis_path).ok()
}

/// Split synopsis text into sentences for beat creation
/// Returns a Vec of sentences, each becoming a separate beat
fn split_into_sentences(text: &str) -> Vec<String> {
    // Split on sentence-ending punctuation followed by whitespace
    // This is a simple heuristic - handles ". ", "! ", "? "
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if (ch == '.' || ch == '!' || ch == '?') && !current.trim().is_empty() {
            // Check if next char would be whitespace or end of string
            let trimmed = current.trim().to_string();
            if !trimmed.is_empty() {
                sentences.push(trimmed);
            }
            current.clear();
        }
    }

    // Don't forget any trailing text without ending punctuation
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }

    sentences
}

fn parse_binder_xml(xml_content: &str) -> Result<Vec<BinderItem>, ScrivenerError> {
    let mut reader = Reader::from_str(xml_content);

    let mut items = Vec::new();
    let mut stack: Vec<BinderItem> = Vec::new();
    let mut in_title = false;
    let mut title_buffer = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"BinderItem" => {
                        // Parse attributes for the new item
                        let mut uuid = String::new();
                        let mut item_type = BinderItemType::Other("Unknown".to_string());

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"UUID" => {
                                    uuid = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"Type" => {
                                    item_type = BinderItemType::from(
                                        String::from_utf8_lossy(&attr.value).as_ref(),
                                    );
                                }
                                _ => {}
                            }
                        }

                        // Push a new item onto the stack
                        stack.push(BinderItem {
                            uuid,
                            item_type,
                            title: String::new(),
                            children: Vec::new(),
                        });
                    }
                    b"Title" if !stack.is_empty() => {
                        in_title = true;
                        title_buffer.clear();
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) if in_title => {
                let text = String::from_utf8_lossy(&e);
                title_buffer.push_str(&text);
            }
            // Handle entity references (e.g., &amp; &lt; &gt;)
            Ok(Event::GeneralRef(e)) if in_title => {
                let entity = String::from_utf8_lossy(&e);
                match entity.as_ref() {
                    "amp" => title_buffer.push('&'),
                    "lt" => title_buffer.push('<'),
                    "gt" => title_buffer.push('>'),
                    "quot" => title_buffer.push('"'),
                    "apos" => title_buffer.push('\''),
                    _ => {
                        // Unknown entity, preserve as-is
                        title_buffer.push('&');
                        title_buffer.push_str(&entity);
                        title_buffer.push(';');
                    }
                }
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Title" => {
                        if let Some(current) = stack.last_mut() {
                            current.title = title_buffer.trim().to_string();
                        }
                        in_title = false;
                    }
                    b"BinderItem" => {
                        // Pop the completed item from the stack
                        if let Some(completed) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                // Add to parent's children
                                parent.children.push(completed);
                            } else {
                                // No parent, add to top-level items
                                items.push(completed);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScrivenerError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    // Handle any remaining items on the stack (shouldn't happen with well-formed XML)
    while let Some(item) = stack.pop() {
        items.push(item);
    }

    Ok(items)
}

fn find_draft_folder(items: &[BinderItem]) -> Option<&BinderItem> {
    for item in items {
        if item.item_type == BinderItemType::DraftFolder {
            return Some(item);
        }
        if let Some(found) = find_draft_folder(&item.children) {
            return Some(found);
        }
    }
    None
}

fn collect_items_by_type(items: &[BinderItem], item_type: BinderItemType) -> Vec<&BinderItem> {
    let mut result = Vec::new();
    for item in items {
        if item.item_type == item_type {
            result.push(item);
        }
        result.extend(collect_items_by_type(&item.children, item_type.clone()));
    }
    result
}

pub fn parse_scrivener_project<P: AsRef<Path>>(
    scriv_path: P,
) -> Result<ParsedScrivener, ScrivenerError> {
    let scriv_path = scriv_path.as_ref();

    // Find the .scrivx file
    let scrivx_path = find_scrivx_file(scriv_path).ok_or(ScrivenerError::ProjectFileNotFound)?;

    let xml_content = fs::read_to_string(&scrivx_path)?;
    let binder_items = parse_binder_xml(&xml_content)?;

    // Extract project name from folder name
    let project_name = scriv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string()
        .replace(".scriv", "");

    let project = Project::new(
        project_name,
        SourceType::Scrivener,
        Some(scriv_path.to_string_lossy().to_string()),
    );

    let mut chapters: Vec<Chapter> = Vec::new();
    let mut scenes: Vec<Scene> = Vec::new();
    let mut beats: Vec<Beat> = Vec::new();
    let mut characters: Vec<Character> = Vec::new();
    let mut locations: Vec<Location> = Vec::new();

    // Find the Draft/Manuscript folder
    if let Some(draft) = find_draft_folder(&binder_items) {
        // Each child of Draft that's a Folder becomes a Chapter
        // Each child that's Text becomes a Scene
        for (ch_idx, child) in draft.children.iter().enumerate() {
            match child.item_type {
                BinderItemType::Folder => {
                    let chapter = Chapter::new(project.id, child.title.clone(), ch_idx as i32);

                    // Children of the folder become scenes
                    for (sc_idx, scene_item) in child.children.iter().enumerate() {
                        if scene_item.item_type == BinderItemType::Text {
                            // Read synopsis for scene display
                            let synopsis = read_synopsis(scriv_path, &scene_item.uuid);

                            // First sentence is the synopsis (brief summary)
                            // All sentences become beats (story moments within the scene)
                            let sentences = synopsis
                                .as_ref()
                                .map(|s| split_into_sentences(s))
                                .unwrap_or_default();

                            let scene = Scene::new(
                                chapter.id,
                                scene_item.title.clone(),
                                sentences.first().cloned(),
                                sc_idx as i32,
                            );

                            // Create a beat for each sentence
                            for (beat_idx, sentence) in sentences.iter().enumerate() {
                                let beat = Beat::new(scene.id, sentence.clone(), beat_idx as i32);
                                beats.push(beat);
                            }

                            scenes.push(scene);
                        }
                    }

                    chapters.push(chapter);
                }
                BinderItemType::Text => {
                    // Top-level text document - create a virtual chapter for it
                    let chapter = Chapter::new(project.id, child.title.clone(), ch_idx as i32);

                    let synopsis = read_synopsis(scriv_path, &child.uuid);
                    let sentences = synopsis
                        .as_ref()
                        .map(|s| split_into_sentences(s))
                        .unwrap_or_default();

                    let scene = Scene::new(
                        chapter.id,
                        child.title.clone(),
                        sentences.first().cloned(),
                        0,
                    );

                    // Create a beat for each sentence
                    for (beat_idx, sentence) in sentences.iter().enumerate() {
                        let beat = Beat::new(scene.id, sentence.clone(), beat_idx as i32);
                        beats.push(beat);
                    }

                    scenes.push(scene);
                    chapters.push(chapter);
                }
                _ => {}
            }
        }
    }

    // Collect character sheets
    let character_sheets = collect_items_by_type(&binder_items, BinderItemType::CharacterSheet);
    for char_item in &character_sheets {
        let description = read_synopsis(scriv_path, &char_item.uuid);
        let character = Character::new(
            project.id,
            char_item.title.clone(),
            description,
            Some(char_item.uuid.clone()),
        );
        characters.push(character);
    }

    // Collect location sheets
    let location_sheets = collect_items_by_type(&binder_items, BinderItemType::LocationSheet);
    for loc_item in &location_sheets {
        let description = read_synopsis(scriv_path, &loc_item.uuid);
        let location = Location::new(
            project.id,
            loc_item.title.clone(),
            description,
            Some(loc_item.uuid.clone()),
        );
        locations.push(location);
    }

    Ok(ParsedScrivener {
        project,
        chapters,
        scenes,
        beats,
        characters,
        locations,
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

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    #[test]
    fn test_find_scrivx_file() {
        let path = fixture_path("hamlet.scriv");
        let scrivx = find_scrivx_file(&path);
        assert!(scrivx.is_some(), "Should find .scrivx file");
        assert!(
            scrivx.unwrap().to_string_lossy().contains("hamlet.scrivx"),
            "Should find hamlet.scrivx"
        );
    }

    #[test]
    fn test_find_scrivx_file_nonexistent() {
        let path = fixture_path("nonexistent.scriv");
        let scrivx = find_scrivx_file(&path);
        assert!(scrivx.is_none(), "Should return None for nonexistent path");
    }

    #[test]
    fn test_read_synopsis() {
        let path = fixture_path("hamlet.scriv");
        let synopsis = read_synopsis(&path, "scene-3-1-uuid");
        assert!(synopsis.is_some(), "Should find synopsis for scene-3-1");
        let text = synopsis.unwrap();
        assert!(
            text.contains("To be or not to be"),
            "Should contain famous soliloquy reference"
        );
    }

    #[test]
    fn test_read_synopsis_nonexistent() {
        let path = fixture_path("hamlet.scriv");
        let synopsis = read_synopsis(&path, "nonexistent-uuid");
        assert!(
            synopsis.is_none(),
            "Should return None for nonexistent UUID"
        );
    }

    #[test]
    fn test_read_synopsis_character() {
        let path = fixture_path("hamlet.scriv");
        let synopsis = read_synopsis(&path, "char-hamlet-uuid");
        assert!(synopsis.is_some(), "Should find synopsis for Hamlet");
        let text = synopsis.unwrap();
        assert!(
            text.contains("Prince of Denmark"),
            "Hamlet should be Prince of Denmark"
        );
    }

    // ========================================================================
    // XML Parsing Tests
    // ========================================================================

    #[test]
    fn test_parse_binder_xml_basic() {
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="test-uuid" Type="Text">
                    <Title>Test Document</Title>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse basic XML");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].uuid, "test-uuid");
        assert_eq!(items[0].title, "Test Document");
        assert_eq!(items[0].item_type, BinderItemType::Text);
    }

    #[test]
    fn test_parse_binder_xml_with_children() {
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="folder-uuid" Type="Folder">
                    <Title>Chapter 1</Title>
                    <Children>
                        <BinderItem UUID="scene-uuid" Type="Text">
                            <Title>Scene 1</Title>
                        </BinderItem>
                    </Children>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse XML with children");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[0].children[0].title, "Scene 1");
    }

    #[test]
    fn test_parse_binder_xml_draft_folder() {
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="draft-uuid" Type="DraftFolder">
                    <Title>Manuscript</Title>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse DraftFolder");
        assert_eq!(items[0].item_type, BinderItemType::DraftFolder);
    }

    #[test]
    fn test_binder_item_type_from_str() {
        assert_eq!(
            BinderItemType::from("DraftFolder"),
            BinderItemType::DraftFolder
        );
        assert_eq!(BinderItemType::from("Folder"), BinderItemType::Folder);
        assert_eq!(BinderItemType::from("Text"), BinderItemType::Text);
        assert_eq!(
            BinderItemType::from("CharacterSheet"),
            BinderItemType::CharacterSheet
        );
        assert_eq!(
            BinderItemType::from("LocationSheet"),
            BinderItemType::LocationSheet
        );
        assert_eq!(
            BinderItemType::from("ResearchFolder"),
            BinderItemType::ResearchFolder
        );
        assert_eq!(
            BinderItemType::from("TrashFolder"),
            BinderItemType::TrashFolder
        );
        assert_eq!(
            BinderItemType::from("CustomType"),
            BinderItemType::Other("CustomType".to_string())
        );
    }

    // ========================================================================
    // find_draft_folder and collect_items_by_type Tests
    // ========================================================================

    #[test]
    fn test_find_draft_folder() {
        let items = vec![
            BinderItem {
                uuid: "other-uuid".to_string(),
                item_type: BinderItemType::Folder,
                title: "Other".to_string(),
                children: vec![],
            },
            BinderItem {
                uuid: "draft-uuid".to_string(),
                item_type: BinderItemType::DraftFolder,
                title: "Manuscript".to_string(),
                children: vec![],
            },
        ];

        let draft = find_draft_folder(&items);
        assert!(draft.is_some());
        assert_eq!(draft.unwrap().uuid, "draft-uuid");
    }

    #[test]
    fn test_find_draft_folder_nested() {
        let items = vec![BinderItem {
            uuid: "outer-uuid".to_string(),
            item_type: BinderItemType::Folder,
            title: "Outer".to_string(),
            children: vec![BinderItem {
                uuid: "draft-uuid".to_string(),
                item_type: BinderItemType::DraftFolder,
                title: "Manuscript".to_string(),
                children: vec![],
            }],
        }];

        let draft = find_draft_folder(&items);
        assert!(draft.is_some());
        assert_eq!(draft.unwrap().uuid, "draft-uuid");
    }

    #[test]
    fn test_find_draft_folder_missing() {
        let items = vec![BinderItem {
            uuid: "folder-uuid".to_string(),
            item_type: BinderItemType::Folder,
            title: "Just a folder".to_string(),
            children: vec![],
        }];

        let draft = find_draft_folder(&items);
        assert!(draft.is_none());
    }

    #[test]
    fn test_collect_items_by_type() {
        let items = vec![
            BinderItem {
                uuid: "char1-uuid".to_string(),
                item_type: BinderItemType::CharacterSheet,
                title: "Character 1".to_string(),
                children: vec![],
            },
            BinderItem {
                uuid: "folder-uuid".to_string(),
                item_type: BinderItemType::Folder,
                title: "Folder".to_string(),
                children: vec![BinderItem {
                    uuid: "char2-uuid".to_string(),
                    item_type: BinderItemType::CharacterSheet,
                    title: "Character 2".to_string(),
                    children: vec![],
                }],
            },
        ];

        let characters = collect_items_by_type(&items, BinderItemType::CharacterSheet);
        assert_eq!(characters.len(), 2);
    }

    // ========================================================================
    // Integration Tests - Full Project Parsing
    // ========================================================================

    #[test]
    fn test_parse_hamlet_project() {
        let path = fixture_path("hamlet.scriv");
        let result = parse_scrivener_project(&path);

        assert!(
            result.is_ok(),
            "Failed to parse hamlet.scriv: {:?}",
            result.err()
        );
        let parsed = result.unwrap();

        // Project
        assert_eq!(parsed.project.name, "hamlet");
        assert_eq!(parsed.project.source_type, SourceType::Scrivener);
    }

    #[test]
    fn test_parse_hamlet_chapters() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Should have 5 acts (chapters)
        assert_eq!(parsed.chapters.len(), 5, "Hamlet should have 5 acts");

        // Verify act titles
        let titles: Vec<&str> = parsed.chapters.iter().map(|c| c.title.as_str()).collect();
        assert!(titles.contains(&"Act 1"), "Should have Act 1");
        assert!(titles.contains(&"Act 2"), "Should have Act 2");
        assert!(titles.contains(&"Act 3"), "Should have Act 3");
        assert!(titles.contains(&"Act 4"), "Should have Act 4");
        assert!(titles.contains(&"Act 5"), "Should have Act 5");
    }

    #[test]
    fn test_parse_hamlet_scenes() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Should have 20 scenes total (5+2+4+7+2)
        assert_eq!(parsed.scenes.len(), 20, "Hamlet should have 20 scenes");

        // Find the famous "To be or not to be" scene
        let to_be_scene = parsed
            .scenes
            .iter()
            .find(|s| s.title.contains("To Be or Not To Be"));
        assert!(
            to_be_scene.is_some(),
            "Should find 'To be or not to be' scene"
        );
    }

    #[test]
    fn test_parse_hamlet_beats_from_sentences() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Each sentence in synopsis becomes a beat
        // Multi-sentence synopses should create multiple beats
        assert!(
            !parsed.beats.is_empty(),
            "Should create beats from synopsis sentences"
        );

        // Should have at least as many beats as scenes (some scenes have multi-sentence synopses)
        assert!(
            parsed.beats.len() >= parsed.scenes.len(),
            "Should have at least as many beats as scenes (beats: {}, scenes: {})",
            parsed.beats.len(),
            parsed.scenes.len()
        );

        // Check that a beat contains expected content
        let soliloquy_beat = parsed
            .beats
            .iter()
            .find(|b| b.content.contains("To be or not to be"));
        assert!(
            soliloquy_beat.is_some(),
            "Should have beat with soliloquy reference"
        );
    }

    #[test]
    fn test_parse_hamlet_characters() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Should have 19 characters
        assert_eq!(
            parsed.characters.len(),
            19,
            "Hamlet should have 19 characters"
        );

        // Verify key characters exist
        let hamlet = parsed.characters.iter().find(|c| c.name == "Hamlet");
        assert!(hamlet.is_some(), "Should have Hamlet character");
        let hamlet = hamlet.unwrap();
        assert!(
            hamlet
                .description
                .as_ref()
                .unwrap()
                .contains("Prince of Denmark"),
            "Hamlet should be Prince of Denmark"
        );

        let claudius = parsed.characters.iter().find(|c| c.name == "Claudius");
        assert!(claudius.is_some(), "Should have Claudius character");

        let ophelia = parsed.characters.iter().find(|c| c.name == "Ophelia");
        assert!(ophelia.is_some(), "Should have Ophelia character");

        let ghost = parsed
            .characters
            .iter()
            .find(|c| c.name == "Ghost of King Hamlet");
        assert!(ghost.is_some(), "Should have Ghost character");
    }

    #[test]
    fn test_parse_hamlet_locations() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Should have 5 locations
        assert_eq!(parsed.locations.len(), 5, "Hamlet should have 5 locations");

        // Verify key locations exist
        let elsinore = parsed
            .locations
            .iter()
            .find(|l| l.name == "Elsinore Castle");
        assert!(elsinore.is_some(), "Should have Elsinore Castle");
        assert!(
            elsinore
                .unwrap()
                .description
                .as_ref()
                .unwrap()
                .contains("royal castle"),
            "Elsinore should be the royal castle"
        );

        let graveyard = parsed.locations.iter().find(|l| l.name == "Graveyard");
        assert!(graveyard.is_some(), "Should have Graveyard");

        let battlements = parsed
            .locations
            .iter()
            .find(|l| l.name == "The Battlements");
        assert!(battlements.is_some(), "Should have The Battlements");
    }

    #[test]
    fn test_parse_hamlet_scene_chapter_relationships() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        // Find Act 1
        let act1 = parsed.chapters.iter().find(|c| c.title == "Act 1");
        assert!(act1.is_some());
        let act1 = act1.unwrap();

        // Count scenes in Act 1 (should be 5)
        let act1_scenes: Vec<_> = parsed
            .scenes
            .iter()
            .filter(|s| s.chapter_id == act1.id)
            .collect();
        assert_eq!(act1_scenes.len(), 5, "Act 1 should have 5 scenes");
    }

    #[test]
    fn test_parse_hamlet_project_metadata() {
        let path = fixture_path("hamlet.scriv");
        let parsed = parse_scrivener_project(&path).expect("Failed to parse hamlet.scriv");

        assert_eq!(parsed.project.name, "hamlet");
        assert!(parsed.project.source_path.is_some());
        assert!(parsed
            .project
            .source_path
            .as_ref()
            .unwrap()
            .contains("hamlet.scriv"));
    }

    // ========================================================================
    // Edge Case Tests
    // ========================================================================

    #[test]
    fn test_parse_nonexistent_project() {
        let path = fixture_path("nonexistent.scriv");
        let result = parse_scrivener_project(&path);
        assert!(result.is_err(), "Should fail for nonexistent project");
    }

    #[test]
    fn test_parse_binder_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse empty binder");
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_binder_xml_truncated() {
        // quick_xml is lenient with truncated XML - it returns what it parsed
        // before hitting EOF. This tests that behavior.
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="test" Type="Text">
                    <Title>Unclosed"#;

        let result = parse_binder_xml(xml);
        // Truncated XML doesn't error - it just parses what it can
        assert!(result.is_ok(), "Truncated XML should parse without error");
        // The incomplete item should still be returned (from the stack cleanup)
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].uuid, "test");
    }

    #[test]
    fn test_parse_binder_xml_invalid_encoding() {
        // Test with invalid XML that quick_xml will reject
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="test" Type="Text">
                    <Title>Test</Mismatched>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let result = parse_binder_xml(xml);
        assert!(result.is_err(), "Mismatched tags should fail");
    }

    #[test]
    fn test_deeply_nested_children() {
        let xml = r#"<?xml version="1.0"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="l1" Type="Folder">
                    <Title>Level 1</Title>
                    <Children>
                        <BinderItem UUID="l2" Type="Folder">
                            <Title>Level 2</Title>
                            <Children>
                                <BinderItem UUID="l3" Type="Text">
                                    <Title>Level 3</Title>
                                </BinderItem>
                            </Children>
                        </BinderItem>
                    </Children>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse nested structure");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].children.len(), 1);
        assert_eq!(items[0].children[0].children.len(), 1);
        assert_eq!(items[0].children[0].children[0].title, "Level 3");
    }

    #[test]
    fn test_utf8_characters_in_title() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="utf8-test" Type="Text">
                    <Title>Caf&amp;eacute; &amp; R&amp;eacute;sum&amp;eacute;</Title>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse UTF-8 content");
        assert_eq!(items.len(), 1);
        // The XML parser will handle entity encoding
    }

    #[test]
    fn test_special_characters_in_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <ScrivenerProject>
            <Binder>
                <BinderItem UUID="special-chars" Type="Text">
                    <Title>Chapter &amp; Scene &lt;1&gt;</Title>
                </BinderItem>
            </Binder>
        </ScrivenerProject>"#;

        let items = parse_binder_xml(xml).expect("Should parse special characters");
        assert_eq!(items[0].title, "Chapter & Scene <1>");
    }
}
