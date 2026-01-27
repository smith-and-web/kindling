//! yWriter 7 (.yw7) Parser
//!
//! Parses yWriter project files and converts them to Kindling's data model.
//! yWriter stores projects as single XML files with chapters, scenes, characters,
//! locations, and items.
//!
//! Key mappings:
//! - yWriter Chapter → Kindling Chapter
//! - yWriter Scene → Kindling Scene
//! - yWriter Goal/Conflict/Outcome → Kindling Beats (scene scaffolding)
//! - yWriter SceneContent → Kindling prose
//! - yWriter Character → Kindling Character
//! - yWriter Location → Kindling Location

use encoding_rs::{Encoding, UTF_16BE, UTF_16LE, UTF_8};
use quick_xml::escape::unescape;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::models::{Beat, Chapter, Character, Location, Project, Scene, SourceType};

#[derive(Debug, Error)]
pub enum YWriterError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] quick_xml::Error),
    #[error("Invalid yWriter file structure: {0}")]
    InvalidStructure(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
}

// ============================================================================
// yWriter Data Structures
// ============================================================================

/// Raw yWriter project data extracted from XML
#[derive(Debug, Default)]
struct YWriterProject {
    title: Option<String>,
    author: Option<String>,
    description: Option<String>,
    word_target: Option<i32>,
}

#[derive(Debug, Default)]
struct YWriterProjectNote {
    id: i32,
    title: String,
    description: Option<String>,
    sort_order: i32,
}

/// Raw yWriter chapter data
#[derive(Debug, Default)]
struct YWriterChapter {
    id: i32,
    sort_order: i32,
    title: String,
    description: Option<String>,
    chapter_type: i32, // 0=Normal, 1=Notes, 2=ToDo
    scene_ids: Vec<i32>,
    /// True if this chapter has <SectionStart> element, indicating it's a Part header
    section_start: bool,
}

/// Raw yWriter scene data
#[derive(Debug, Default)]
struct YWriterScene {
    id: i32,
    title: String,
    description: Option<String>,
    goal: Option<String>,
    conflict: Option<String>,
    outcome: Option<String>,
    scene_content: Option<String>,
    status: i32,
    reaction_scene: bool,
    character_ids: Vec<i32>,
    location_ids: Vec<i32>,
    date: Option<String>,
    time: Option<String>,
    day: Option<String>,
}

/// Raw yWriter character data
#[derive(Debug, Default)]
struct YWriterCharacter {
    id: i32,
    title: String,
    full_name: Option<String>,
    description: Option<String>,
    bio: Option<String>,
    goals: Option<String>,
    notes: Option<String>,
}

/// Raw yWriter location data
#[derive(Debug, Default)]
struct YWriterLocation {
    id: i32,
    title: String,
    description: Option<String>,
    aka: Option<String>,
}

/// Raw yWriter item data
#[derive(Debug, Default)]
struct YWriterItem {
    id: i32,
    title: String,
    description: Option<String>,
}

// ============================================================================
// Parsed Output
// ============================================================================

/// Result of parsing a yWriter file
#[derive(Debug)]
pub struct ParsedYWriter {
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
// Encoding Detection
// ============================================================================

/// Detect the encoding of a byte slice by checking for BOM
fn detect_encoding(bytes: &[u8]) -> &'static Encoding {
    // Check for UTF-16 BOM
    if bytes.len() >= 2 {
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            return UTF_16LE;
        }
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            return UTF_16BE;
        }
    }
    // Check for UTF-8 BOM
    if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        return UTF_8;
    }
    // Default to UTF-8
    UTF_8
}

/// Decode bytes to string, handling various encodings
fn decode_content(bytes: &[u8]) -> Result<String, YWriterError> {
    let encoding = detect_encoding(bytes);

    let (decoded, _, had_errors) = encoding.decode(bytes);
    if had_errors {
        return Err(YWriterError::EncodingError(
            "Failed to decode file content".to_string(),
        ));
    }

    Ok(decoded.into_owned())
}

// ============================================================================
// XML Parsing Helpers
// ============================================================================

/// Parse semicolon-delimited ID string into a vector of integers
fn parse_id_list(s: &str) -> Vec<i32> {
    s.split(';')
        .filter_map(|part| part.trim().parse::<i32>().ok())
        .collect()
}

/// Convert yWriter markup to HTML
/// yWriter uses: [i]italic[/i], [b]bold[/b]
pub fn convert_ywriter_markup(text: &str) -> String {
    text.replace("[i]", "<em>")
        .replace("[/i]", "</em>")
        .replace("[b]", "<strong>")
        .replace("[/b]", "</strong>")
        // Convert line breaks to HTML paragraphs
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .map(|p| format!("<p>{}</p>", p.trim().replace('\n', "<br>")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Get text content from current XML element
fn read_element_text(
    reader: &mut Reader<&[u8]>,
    buf: &mut Vec<u8>,
) -> Result<String, YWriterError> {
    let mut text = String::new();
    loop {
        match reader.read_event_into(buf)? {
            Event::Text(e) => {
                // First decode bytes to string, then unescape XML entities
                let decoded = String::from_utf8_lossy(&e);
                let unescaped = unescape(&decoded).map_err(|e| {
                    YWriterError::EncodingError(format!("Failed to unescape XML text: {:?}", e))
                })?;
                text.push_str(&unescaped);
            }
            Event::CData(e) => {
                text.push_str(&String::from_utf8_lossy(&e.into_inner()));
            }
            Event::GeneralRef(e) => {
                // Handle entity references like &amp; &lt; &gt; &quot; &apos;
                let ref_name = String::from_utf8_lossy(&e).to_string();
                let resolved = match ref_name.as_str() {
                    "amp" => "&",
                    "lt" => "<",
                    "gt" => ">",
                    "quot" => "\"",
                    "apos" => "'",
                    _ => {
                        // For numeric character references or unknown entities,
                        // try to pass through for now
                        text.push('&');
                        text.push_str(&ref_name);
                        text.push(';');
                        continue;
                    }
                };
                text.push_str(resolved);
            }
            Event::End(_) => break,
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    Ok(text)
}

// ============================================================================
// XML Parser
// ============================================================================

/// Parse a yWriter 7 project file
pub fn parse_ywriter_file<P: AsRef<Path>>(path: P) -> Result<ParsedYWriter, YWriterError> {
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    let content = decode_content(&bytes)?;

    parse_ywriter_content(&content, path)
}

/// Parse yWriter XML content
fn parse_ywriter_content(content: &str, path: &Path) -> Result<ParsedYWriter, YWriterError> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();

    // Collected raw data
    let mut project_data = YWriterProject::default();
    let mut chapters: Vec<YWriterChapter> = Vec::new();
    let mut scenes: HashMap<i32, YWriterScene> = HashMap::new();
    let mut characters: HashMap<i32, YWriterCharacter> = HashMap::new();
    let mut locations: HashMap<i32, YWriterLocation> = HashMap::new();
    let mut items: HashMap<i32, YWriterItem> = HashMap::new();
    let mut project_notes: Vec<YWriterProjectNote> = Vec::new();

    // Current parsing context
    let mut current_chapter: Option<YWriterChapter> = None;
    let mut current_scene: Option<YWriterScene> = None;
    let mut current_character: Option<YWriterCharacter> = None;
    let mut current_location: Option<YWriterLocation> = None;
    let mut current_item: Option<YWriterItem> = None;
    let mut current_project_note: Option<YWriterProjectNote> = None;
    let mut in_project = false;
    let mut in_scene_characters = false;
    let mut in_scene_locations = false;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "PROJECT" => {
                        in_project = true;
                    }
                    "CHAPTER" => {
                        current_chapter = Some(YWriterChapter::default());
                    }
                    "SCENE" => {
                        current_scene = Some(YWriterScene::default());
                    }
                    "CHARACTER" => {
                        current_character = Some(YWriterCharacter::default());
                    }
                    "LOCATION" => {
                        current_location = Some(YWriterLocation::default());
                    }
                    "ITEM" => {
                        current_item = Some(YWriterItem::default());
                    }
                    "PROJECTNOTE" => {
                        current_project_note = Some(YWriterProjectNote::default());
                    }
                    // Project fields
                    "Title"
                        if in_project
                            && current_chapter.is_none()
                            && current_project_note.is_none() =>
                    {
                        project_data.title = Some(read_element_text(&mut reader, &mut buf)?);
                    }
                    "Author" | "AuthorName" if in_project => {
                        project_data.author = Some(read_element_text(&mut reader, &mut buf)?);
                    }
                    "Desc"
                        if in_project
                            && current_chapter.is_none()
                            && current_scene.is_none()
                            && current_project_note.is_none() =>
                    {
                        project_data.description = Some(read_element_text(&mut reader, &mut buf)?);
                    }
                    "WordTarget" if in_project => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        project_data.word_target = text.parse().ok();
                    }
                    // Project note fields
                    "ID" if current_project_note.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut note) = current_project_note {
                            note.id = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_project_note.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut note) = current_project_note {
                            note.title = text;
                        }
                    }
                    "Desc" if current_project_note.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut note) = current_project_note {
                            note.description = Some(text);
                        }
                    }
                    "SortOrder" if current_project_note.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut note) = current_project_note {
                            note.sort_order = text.parse().unwrap_or(0);
                        }
                    }
                    // Chapter fields
                    "ID" if current_chapter.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.id = text.parse().unwrap_or(0);
                        }
                    }
                    "SortOrder" if current_chapter.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.sort_order = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_chapter.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.title = text;
                        }
                    }
                    "Desc" if current_chapter.is_some() && current_scene.is_none() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.description = Some(text);
                        }
                    }
                    "Type" if current_chapter.is_some() && current_scene.is_none() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.chapter_type = text.parse().unwrap_or(0);
                        }
                    }
                    "Scenes" if current_chapter.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.scene_ids = parse_id_list(&text);
                        }
                    }
                    // SectionStart marks a chapter as a Part header (section heading)
                    "SectionStart" if current_chapter.is_some() => {
                        // The presence of this element (regardless of content) marks a Part
                        let _ = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_chapter {
                            ch.section_start = true;
                        }
                    }
                    // Scene fields
                    "ID" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.id = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.title = text;
                        }
                    }
                    "Desc" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.description = Some(text);
                        }
                    }
                    "Goal" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.goal = Some(text);
                        }
                    }
                    "Conflict" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.conflict = Some(text);
                        }
                    }
                    "Outcome" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.outcome = Some(text);
                        }
                    }
                    "SceneContent" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.scene_content = Some(text);
                        }
                    }
                    "Status" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.status = text.parse().unwrap_or(0);
                        }
                    }
                    "ReactionScene" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            sc.reaction_scene = text == "1";
                        }
                    }
                    "Characters" if current_scene.is_some() => {
                        // Enter Characters block for this scene
                        // This block may contain <CharID> children OR semicolon-separated text
                        in_scene_characters = true;
                    }
                    "CharID" if current_scene.is_some() && in_scene_characters => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            if let Ok(id) = text.trim().parse::<i32>() {
                                sc.character_ids.push(id);
                            }
                        }
                    }
                    "Locations" if current_scene.is_some() => {
                        // Enter Locations block for this scene
                        // This block may contain <LocID> children OR semicolon-separated text
                        in_scene_locations = true;
                    }
                    "LocID" if current_scene.is_some() && in_scene_locations => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            if let Ok(id) = text.trim().parse::<i32>() {
                                sc.location_ids.push(id);
                            }
                        }
                    }
                    "Items" if current_scene.is_some() => {
                        // Skip Items block
                    }
                    "Date" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            if text != "-" && !text.is_empty() {
                                sc.date = Some(text);
                            }
                        }
                    }
                    "Time" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            if text != "-" && !text.is_empty() {
                                sc.time = Some(text);
                            }
                        }
                    }
                    "Day" if current_scene.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut sc) = current_scene {
                            if text != "-" && !text.is_empty() {
                                sc.day = Some(text);
                            }
                        }
                    }
                    // Character fields
                    "ID" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.id = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.title = text;
                        }
                    }
                    "FullName" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.full_name = Some(text);
                        }
                    }
                    "Desc" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.description = Some(text);
                        }
                    }
                    "Bio" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.bio = Some(text);
                        }
                    }
                    "Goals" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.goals = Some(text);
                        }
                    }
                    "Notes" if current_character.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut ch) = current_character {
                            ch.notes = Some(text);
                        }
                    }
                    // Location fields
                    "ID" if current_location.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut loc) = current_location {
                            loc.id = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_location.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut loc) = current_location {
                            loc.title = text;
                        }
                    }
                    "Desc" if current_location.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut loc) = current_location {
                            loc.description = Some(text);
                        }
                    }
                    "Aka" if current_location.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut loc) = current_location {
                            loc.aka = Some(text);
                        }
                    }
                    // Item fields
                    "ID" if current_item.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut item) = current_item {
                            item.id = text.parse().unwrap_or(0);
                        }
                    }
                    "Title" if current_item.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut item) = current_item {
                            item.title = text;
                        }
                    }
                    "Desc" if current_item.is_some() => {
                        let text = read_element_text(&mut reader, &mut buf)?;
                        if let Some(ref mut item) = current_item {
                            item.description = Some(text);
                        }
                    }
                    _ => {}
                }
            }
            Event::End(e) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match tag_name.as_str() {
                    "PROJECT" => {
                        in_project = false;
                    }
                    "CHAPTER" => {
                        if let Some(ch) = current_chapter.take() {
                            chapters.push(ch);
                        }
                    }
                    "SCENE" => {
                        if let Some(sc) = current_scene.take() {
                            scenes.insert(sc.id, sc);
                        }
                    }
                    "CHARACTER" => {
                        if let Some(ch) = current_character.take() {
                            characters.insert(ch.id, ch);
                        }
                    }
                    "LOCATION" => {
                        if let Some(loc) = current_location.take() {
                            locations.insert(loc.id, loc);
                        }
                    }
                    "ITEM" => {
                        if let Some(item) = current_item.take() {
                            items.insert(item.id, item);
                        }
                    }
                    "PROJECTNOTE" => {
                        if let Some(note) = current_project_note.take() {
                            project_notes.push(note);
                        }
                    }
                    "Characters" if current_scene.is_some() => {
                        in_scene_characters = false;
                    }
                    "Locations" if current_scene.is_some() => {
                        in_scene_locations = false;
                    }
                    _ => {}
                }
            }
            Event::Text(e) => {
                // Handle text content in Characters/Locations blocks (semicolon-separated format)
                if in_scene_characters && current_scene.is_some() {
                    let text = String::from_utf8_lossy(&e).to_string();
                    let ids = parse_id_list(&text);
                    if !ids.is_empty() {
                        if let Some(ref mut sc) = current_scene {
                            sc.character_ids.extend(ids);
                        }
                    }
                } else if in_scene_locations && current_scene.is_some() {
                    let text = String::from_utf8_lossy(&e).to_string();
                    let ids = parse_id_list(&text);
                    if !ids.is_empty() {
                        if let Some(ref mut sc) = current_scene {
                            sc.location_ids.extend(ids);
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    // Convert to Kindling data model
    convert_to_kindling(
        project_data,
        project_notes,
        chapters,
        scenes,
        characters,
        locations,
        items,
        path,
    )
}

// ============================================================================
// Conversion to Kindling Model
// ============================================================================

#[allow(clippy::too_many_arguments)]
fn convert_to_kindling(
    project_data: YWriterProject,
    mut project_notes: Vec<YWriterProjectNote>,
    yw_chapters: Vec<YWriterChapter>,
    yw_scenes: HashMap<i32, YWriterScene>,
    yw_characters: HashMap<i32, YWriterCharacter>,
    yw_locations: HashMap<i32, YWriterLocation>,
    _yw_items: HashMap<i32, YWriterItem>,
    path: &Path,
) -> Result<ParsedYWriter, YWriterError> {
    // Create project
    let project_name = project_data.title.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .to_string()
    });

    let mut project = Project::new(
        project_name,
        SourceType::YWriter,
        Some(path.to_string_lossy().to_string()),
    );

    // Set project author if available
    if let Some(author) = project_data.author {
        project.author_pen_name = Some(author);
    }

    // Set project description and notes if available
    let mut description_parts = Vec::new();
    if let Some(description) = project_data.description {
        if !description.trim().is_empty() {
            description_parts.push(description);
        }
    }

    if !project_notes.is_empty() {
        project_notes.sort_by_key(|note| note.sort_order);
        let mut notes_section = Vec::new();
        for note in project_notes {
            let mut note_block = String::new();
            if !note.title.trim().is_empty() {
                note_block.push_str(note.title.trim());
            }
            if let Some(desc) = note.description {
                if !desc.trim().is_empty() {
                    if !note_block.is_empty() {
                        note_block.push('\n');
                    }
                    note_block.push_str(desc.trim());
                }
            }
            if !note_block.is_empty() {
                notes_section.push(note_block);
            }
        }
        if !notes_section.is_empty() {
            description_parts.push("Project Notes:".to_string());
            description_parts.extend(notes_section);
        }
    }

    if !description_parts.is_empty() {
        project.description = Some(description_parts.join("\n\n"));
    }

    if project_data.word_target.is_some() {
        project.word_target = project_data.word_target;
    }

    // Build ID mappings for references
    let mut yw_char_id_to_uuid: HashMap<i32, uuid::Uuid> = HashMap::new();
    let mut yw_loc_id_to_uuid: HashMap<i32, uuid::Uuid> = HashMap::new();
    let mut yw_scene_id_to_uuid: HashMap<i32, uuid::Uuid> = HashMap::new();

    // Convert characters
    let mut kindling_characters: Vec<Character> = Vec::new();
    for (yw_id, yw_char) in &yw_characters {
        // Build a combined description from yWriter's description, bio, goals, and notes
        // Output as HTML for proper rendering in the References panel
        let mut description_parts = Vec::new();
        if let Some(ref desc) = yw_char.description {
            // Convert yWriter markup in the description
            description_parts.push(convert_ywriter_markup(desc));
        }
        if let Some(ref bio) = yw_char.bio {
            description_parts.push(format!(
                "<p><strong>Bio:</strong> {}</p>",
                bio.replace('\n', "<br>")
            ));
        }
        if let Some(ref goals) = yw_char.goals {
            description_parts.push(format!(
                "<p><strong>Goals:</strong> {}</p>",
                goals.replace('\n', "<br>")
            ));
        }
        if let Some(ref notes) = yw_char.notes {
            description_parts.push(convert_ywriter_markup(notes));
        }

        let description = if description_parts.is_empty() {
            None
        } else {
            Some(description_parts.join("\n"))
        };

        let character = Character::new(
            project.id,
            yw_char
                .full_name
                .clone()
                .unwrap_or_else(|| yw_char.title.clone()),
            description,
            Some(yw_id.to_string()),
        );
        yw_char_id_to_uuid.insert(*yw_id, character.id);
        kindling_characters.push(character);
    }

    // Convert locations
    let mut kindling_locations: Vec<Location> = Vec::new();
    for (yw_id, yw_loc) in &yw_locations {
        // Build location description as HTML for proper rendering
        let mut description_parts = Vec::new();
        if let Some(ref desc) = yw_loc.description {
            description_parts.push(convert_ywriter_markup(desc));
        }
        if let Some(ref aka) = yw_loc.aka {
            description_parts.push(format!("<p><em>Also known as:</em> {}</p>", aka));
        }

        let description = if description_parts.is_empty() {
            None
        } else {
            Some(description_parts.join("\n"))
        };

        let location = Location::new(
            project.id,
            yw_loc.title.clone(),
            description,
            Some(yw_id.to_string()),
        );
        yw_loc_id_to_uuid.insert(*yw_id, location.id);
        kindling_locations.push(location);
    }

    // Sort chapters by sort_order
    let mut sorted_chapters = yw_chapters;
    sorted_chapters.sort_by_key(|c| c.sort_order);

    // Filter to only normal chapters (type 0)
    let normal_chapters: Vec<_> = sorted_chapters
        .into_iter()
        .filter(|c| c.chapter_type == 0)
        .collect();

    // Convert chapters, scenes, and beats
    let mut kindling_chapters: Vec<Chapter> = Vec::new();
    let mut kindling_scenes: Vec<Scene> = Vec::new();
    let mut kindling_beats: Vec<Beat> = Vec::new();
    let mut scene_character_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();
    let mut scene_location_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();

    for (chapter_pos, yw_chapter) in normal_chapters.iter().enumerate() {
        let chapter = Chapter::new(project.id, yw_chapter.title.clone(), chapter_pos as i32)
            .with_source_id(Some(yw_chapter.id.to_string()))
            .with_is_part(yw_chapter.section_start);

        // Process scenes in this chapter
        for (scene_pos, yw_scene_id) in yw_chapter.scene_ids.iter().enumerate() {
            if let Some(yw_scene) = yw_scenes.get(yw_scene_id) {
                let scene = Scene::new(
                    chapter.id,
                    yw_scene.title.clone(),
                    yw_scene.description.clone(),
                    scene_pos as i32,
                )
                .with_source_id(Some(yw_scene_id.to_string()));

                yw_scene_id_to_uuid.insert(*yw_scene_id, scene.id);

                // Create beats from Goal, Conflict, Outcome
                let mut beat_pos = 0;

                // Use reaction scene labels if applicable
                let (goal_label, conflict_label, outcome_label) = if yw_scene.reaction_scene {
                    ("Response", "Dilemma", "Decision")
                } else {
                    ("Goal", "Conflict", "Outcome")
                };

                if let Some(ref goal) = yw_scene.goal {
                    if !goal.trim().is_empty() {
                        let beat =
                            Beat::new(scene.id, format!("{}: {}", goal_label, goal), beat_pos)
                                .with_source_id(Some(format!("{}-goal", yw_scene_id)));
                        kindling_beats.push(beat);
                        beat_pos += 1;
                    }
                }

                if let Some(ref conflict) = yw_scene.conflict {
                    if !conflict.trim().is_empty() {
                        let beat = Beat::new(
                            scene.id,
                            format!("{}: {}", conflict_label, conflict),
                            beat_pos,
                        )
                        .with_source_id(Some(format!("{}-conflict", yw_scene_id)));
                        kindling_beats.push(beat);
                        beat_pos += 1;
                    }
                }

                if let Some(ref outcome) = yw_scene.outcome {
                    if !outcome.trim().is_empty() {
                        let beat = Beat::new(
                            scene.id,
                            format!("{}: {}", outcome_label, outcome),
                            beat_pos,
                        )
                        .with_source_id(Some(format!("{}-outcome", yw_scene_id)));
                        kindling_beats.push(beat);
                        beat_pos += 1;
                    }
                }

                // If scene has prose content, add it to the first beat or create a "Prose" beat
                if let Some(ref content) = yw_scene.scene_content {
                    if !content.trim().is_empty() {
                        let html_content = convert_ywriter_markup(content);

                        if beat_pos > 0 {
                            // Add prose to the first beat
                            if let Some(first_beat) = kindling_beats
                                .iter_mut()
                                .find(|b| b.scene_id == scene.id && b.position == 0)
                            {
                                first_beat.prose = Some(html_content);
                            }
                        } else {
                            // No GCO beats, create a prose-only beat
                            let mut beat = Beat::new(scene.id, "Scene Content".to_string(), 0)
                                .with_source_id(Some(format!("{}-prose", yw_scene_id)));
                            beat.prose = Some(html_content);
                            kindling_beats.push(beat);
                        }
                    }
                }

                // Track character references
                for char_id in &yw_scene.character_ids {
                    if let Some(&uuid) = yw_char_id_to_uuid.get(char_id) {
                        scene_character_refs.push((scene.id, uuid));
                    }
                }

                // Track location references
                for loc_id in &yw_scene.location_ids {
                    if let Some(&uuid) = yw_loc_id_to_uuid.get(loc_id) {
                        scene_location_refs.push((scene.id, uuid));
                    }
                }

                kindling_scenes.push(scene);
            }
        }

        kindling_chapters.push(chapter);
    }

    Ok(ParsedYWriter {
        project,
        chapters: kindling_chapters,
        scenes: kindling_scenes,
        beats: kindling_beats,
        characters: kindling_characters,
        locations: kindling_locations,
        scene_character_refs,
        scene_location_refs,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_id_list() {
        assert_eq!(parse_id_list("1;2;3"), vec![1, 2, 3]);
        assert_eq!(parse_id_list("1"), vec![1]);
        assert_eq!(parse_id_list(""), Vec::<i32>::new());
        assert_eq!(parse_id_list("1;2;invalid;3"), vec![1, 2, 3]);
        assert_eq!(parse_id_list("  1 ; 2 ; 3  "), vec![1, 2, 3]);
    }

    #[test]
    fn test_convert_ywriter_markup() {
        // Test italic
        assert!(convert_ywriter_markup("[i]italic[/i]").contains("<em>italic</em>"));

        // Test bold
        assert!(convert_ywriter_markup("[b]bold[/b]").contains("<strong>bold</strong>"));

        // Test paragraphs
        let result = convert_ywriter_markup("Para one.\n\nPara two.");
        assert!(result.contains("<p>Para one.</p>"));
        assert!(result.contains("<p>Para two.</p>"));

        // Test line breaks within paragraph
        let result = convert_ywriter_markup("Line one.\nLine two.");
        assert!(result.contains("<br>"));
    }

    #[test]
    fn test_detect_encoding_utf8() {
        let bytes = b"Hello world";
        assert_eq!(detect_encoding(bytes), UTF_8);
    }

    #[test]
    fn test_detect_encoding_utf8_bom() {
        let bytes = &[0xEF, 0xBB, 0xBF, b'H', b'i'];
        assert_eq!(detect_encoding(bytes), UTF_8);
    }

    #[test]
    fn test_detect_encoding_utf16le() {
        let bytes = &[0xFF, 0xFE, 0x48, 0x00];
        assert_eq!(detect_encoding(bytes), UTF_16LE);
    }

    #[test]
    fn test_detect_encoding_utf16be() {
        let bytes = &[0xFE, 0xFF, 0x00, 0x48];
        assert_eq!(detect_encoding(bytes), UTF_16BE);
    }

    #[test]
    fn test_parse_hamlet_fixture() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let result = parse_ywriter_file(&path);
        assert!(
            result.is_ok(),
            "Failed to parse hamlet.yw7: {:?}",
            result.err()
        );

        let parsed = result.unwrap();

        // Check project
        assert_eq!(parsed.project.name, "Hamlet");
        assert_eq!(
            parsed.project.author_pen_name,
            Some("William Shakespeare".to_string())
        );
        assert_eq!(
            parsed.project.description.as_deref(),
            Some("The Tragedy of Hamlet, Prince of Denmark")
        );
        assert_eq!(parsed.project.word_target, Some(80000));

        // Check chapters
        assert_eq!(parsed.chapters.len(), 3);
        assert_eq!(parsed.chapters[0].title, "Act I");
        assert_eq!(parsed.chapters[1].title, "Act II");
        assert_eq!(parsed.chapters[2].title, "Act III");

        // Check scenes
        assert_eq!(parsed.scenes.len(), 7);

        // Check characters
        assert_eq!(parsed.characters.len(), 10);
        let hamlet = parsed
            .characters
            .iter()
            .find(|c| c.name == "Hamlet, Prince of Denmark");
        assert!(hamlet.is_some());

        // Check locations
        assert_eq!(parsed.locations.len(), 3);
        let battlements = parsed
            .locations
            .iter()
            .find(|l| l.name == "The Battlements");
        assert!(battlements.is_some());
    }

    #[test]
    fn test_parse_hamlet_beats() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Find beats for "The Ghost Appears" scene
        let ghost_scene = parsed
            .scenes
            .iter()
            .find(|s| s.title == "The Ghost Appears")
            .unwrap();
        let ghost_beats: Vec<_> = parsed
            .beats
            .iter()
            .filter(|b| b.scene_id == ghost_scene.id)
            .collect();

        // Should have Goal, Conflict, Outcome beats
        assert_eq!(ghost_beats.len(), 3);
        assert!(ghost_beats.iter().any(|b| b.content.starts_with("Goal:")));
        assert!(ghost_beats
            .iter()
            .any(|b| b.content.starts_with("Conflict:")));
        assert!(ghost_beats
            .iter()
            .any(|b| b.content.starts_with("Outcome:")));

        // First beat should have prose
        let first_beat = ghost_beats.iter().find(|b| b.position == 0).unwrap();
        assert!(first_beat.prose.is_some());
        assert!(first_beat.prose.as_ref().unwrap().contains("BERNARDO"));
    }

    #[test]
    fn test_parse_hamlet_reaction_scene() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Find "To Be or Not To Be" which is marked as a reaction scene
        let soliloquy_scene = parsed
            .scenes
            .iter()
            .find(|s| s.title == "To Be or Not To Be")
            .unwrap();
        let soliloquy_beats: Vec<_> = parsed
            .beats
            .iter()
            .filter(|b| b.scene_id == soliloquy_scene.id)
            .collect();

        // Reaction scenes should use Response/Dilemma/Decision labels
        assert!(soliloquy_beats
            .iter()
            .any(|b| b.content.starts_with("Response:")));
        assert!(soliloquy_beats
            .iter()
            .any(|b| b.content.starts_with("Dilemma:")));
        assert!(soliloquy_beats
            .iter()
            .any(|b| b.content.starts_with("Decision:")));
    }

    #[test]
    fn test_parse_hamlet_scene_references() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Check that scenes have character and location references
        assert!(!parsed.scene_character_refs.is_empty());
        assert!(!parsed.scene_location_refs.is_empty());

        // The Mousetrap scene should have many characters
        let mousetrap = parsed
            .scenes
            .iter()
            .find(|s| s.title == "The Mousetrap")
            .unwrap();
        let mousetrap_chars: Vec<_> = parsed
            .scene_character_refs
            .iter()
            .filter(|(scene_id, _)| *scene_id == mousetrap.id)
            .collect();
        assert!(mousetrap_chars.len() >= 5);
    }

    // ========================================================================
    // Error Handling Tests
    // ========================================================================

    #[test]
    fn test_parse_nonexistent_file() {
        let result = parse_ywriter_file("/nonexistent/path/to/file.yw7");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, YWriterError::IoError(_)));
    }

    #[test]
    fn test_parse_malformed_xml() {
        // quick-xml is quite resilient, so we need very malformed XML
        // This has an invalid UTF-8 sequence which should cause parsing to fail
        let malformed_bytes: &[u8] = &[
            0x3C, 0x3F, 0x78, 0x6D, 0x6C, 0x20, // <?xml
            0xFF, 0xFE, // Invalid UTF-8 sequence in middle of XML declaration
        ];

        // Test with invalid bytes by reading from a temp file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("malformed_test.yw7");
        std::fs::write(&temp_file, malformed_bytes).unwrap();

        let result = parse_ywriter_file(&temp_file);
        // Clean up
        let _ = std::fs::remove_file(&temp_file);

        // Either parsing fails (XmlError) or encoding fails (EncodingError)
        // Either is acceptable for malformed input
        assert!(result.is_err(), "Malformed XML should fail to parse");
    }

    #[test]
    fn test_parse_empty_xml() {
        let empty_xml = r#"<?xml version="1.0"?>
<YWRITER7>
</YWRITER7>"#;

        let result = parse_ywriter_content(empty_xml, Path::new("empty.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Should use filename as project name when no title present
        assert_eq!(parsed.project.name, "empty");
        assert!(parsed.chapters.is_empty());
        assert!(parsed.scenes.is_empty());
        assert!(parsed.beats.is_empty());
        assert!(parsed.characters.is_empty());
        assert!(parsed.locations.is_empty());
    }

    // ========================================================================
    // Minimal/Edge Case Tests
    // ========================================================================

    #[test]
    fn test_parse_minimal_project() {
        let minimal_xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Minimal Project</Title>
  </PROJECT>
</YWRITER7>"#;

        let result = parse_ywriter_content(minimal_xml, Path::new("minimal.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(parsed.project.name, "Minimal Project");
        assert!(parsed.chapters.is_empty());
        assert!(parsed.scenes.is_empty());
    }

    #[test]
    fn test_parse_chapter_with_no_scenes() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Empty Chapter Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Empty Chapter</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
  </CHAPTERS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(parsed.chapters.len(), 1);
        assert_eq!(parsed.chapters[0].title, "Empty Chapter");
        assert!(parsed.scenes.is_empty());
    }

    #[test]
    fn test_parse_scene_without_gco() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>No GCO Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Chapter One</Title>
      <Type>0</Type>
      <Scenes>1</Scenes>
    </CHAPTER>
  </CHAPTERS>
  <SCENES>
    <SCENE>
      <ID>1</ID>
      <Title>Scene Without GCO</Title>
      <Desc>A scene with only prose content</Desc>
      <SceneContent>This is the prose content of the scene.</SceneContent>
      <Status>2</Status>
      <ReactionScene>0</ReactionScene>
    </SCENE>
  </SCENES>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(parsed.scenes.len(), 1);
        assert_eq!(parsed.scenes[0].title, "Scene Without GCO");

        // Should create a single beat with "Scene Content" label and prose
        assert_eq!(parsed.beats.len(), 1);
        assert_eq!(parsed.beats[0].content, "Scene Content");
        assert!(parsed.beats[0].prose.is_some());
        assert!(parsed.beats[0]
            .prose
            .as_ref()
            .unwrap()
            .contains("prose content"));
    }

    #[test]
    fn test_parse_scene_with_empty_gco() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Empty GCO Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Chapter One</Title>
      <Type>0</Type>
      <Scenes>1</Scenes>
    </CHAPTER>
  </CHAPTERS>
  <SCENES>
    <SCENE>
      <ID>1</ID>
      <Title>Scene With Empty GCO</Title>
      <Goal>   </Goal>
      <Conflict></Conflict>
      <Outcome></Outcome>
      <SceneContent>Only prose here.</SceneContent>
      <Status>2</Status>
      <ReactionScene>0</ReactionScene>
    </SCENE>
  </SCENES>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Empty/whitespace-only GCO fields should not create beats
        // Should only have a prose-only beat
        assert_eq!(parsed.beats.len(), 1);
        assert_eq!(parsed.beats[0].content, "Scene Content");
    }

    #[test]
    fn test_filter_notes_and_todo_chapters() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Chapter Types Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Normal Chapter</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
    <CHAPTER>
      <ID>2</ID>
      <SortOrder>2</SortOrder>
      <Title>Notes Chapter</Title>
      <Type>1</Type>
      <Scenes></Scenes>
    </CHAPTER>
    <CHAPTER>
      <ID>3</ID>
      <SortOrder>3</SortOrder>
      <Title>ToDo Chapter</Title>
      <Type>2</Type>
      <Scenes></Scenes>
    </CHAPTER>
    <CHAPTER>
      <ID>4</ID>
      <SortOrder>4</SortOrder>
      <Title>Another Normal</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
  </CHAPTERS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Only Type=0 (Normal) chapters should be imported
        assert_eq!(parsed.chapters.len(), 2);
        assert_eq!(parsed.chapters[0].title, "Normal Chapter");
        assert_eq!(parsed.chapters[1].title, "Another Normal");
    }

    #[test]
    fn test_chapter_sort_order() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Sort Order Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>3</ID>
      <SortOrder>3</SortOrder>
      <Title>Third</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>First</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
    <CHAPTER>
      <ID>2</ID>
      <SortOrder>2</SortOrder>
      <Title>Second</Title>
      <Type>0</Type>
      <Scenes></Scenes>
    </CHAPTER>
  </CHAPTERS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Chapters should be sorted by sort_order, not XML order
        assert_eq!(parsed.chapters.len(), 3);
        assert_eq!(parsed.chapters[0].title, "First");
        assert_eq!(parsed.chapters[1].title, "Second");
        assert_eq!(parsed.chapters[2].title, "Third");

        // Positions should be assigned based on sorted order
        assert_eq!(parsed.chapters[0].position, 0);
        assert_eq!(parsed.chapters[1].position, 1);
        assert_eq!(parsed.chapters[2].position, 2);
    }

    // ========================================================================
    // Character & Location Edge Cases
    // ========================================================================

    #[test]
    fn test_character_uses_full_name_when_available() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Character Name Test</Title>
  </PROJECT>
  <CHARACTERS>
    <CHARACTER>
      <ID>1</ID>
      <Title>John</Title>
      <FullName>John Smith III</FullName>
      <Desc>A test character</Desc>
    </CHARACTER>
    <CHARACTER>
      <ID>2</ID>
      <Title>Jane</Title>
      <Desc>No full name provided</Desc>
    </CHARACTER>
  </CHARACTERS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(parsed.characters.len(), 2);

        // First character should use FullName
        let john = parsed
            .characters
            .iter()
            .find(|c| c.name.contains("John"))
            .unwrap();
        assert_eq!(john.name, "John Smith III");

        // Second character should fall back to Title
        let jane = parsed.characters.iter().find(|c| c.name == "Jane").unwrap();
        assert_eq!(jane.name, "Jane");
    }

    #[test]
    fn test_character_description_combines_fields() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Character Fields Test</Title>
  </PROJECT>
  <CHARACTERS>
    <CHARACTER>
      <ID>1</ID>
      <Title>Hero</Title>
      <Desc>The main character</Desc>
      <Bio>Born in a small village</Bio>
      <Goals>Save the world</Goals>
      <Notes>Important: likes cats</Notes>
    </CHARACTER>
  </CHARACTERS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let hero = &parsed.characters[0];
        let desc = hero.description.as_ref().unwrap();

        // All fields should be combined into description as HTML
        assert!(desc.contains("The main character"));
        assert!(desc.contains("<strong>Bio:</strong> Born in a small village"));
        assert!(desc.contains("<strong>Goals:</strong> Save the world"));
        assert!(desc.contains("Important: likes cats"));
    }

    #[test]
    fn test_location_aka_appended_to_description() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Location AKA Test</Title>
  </PROJECT>
  <LOCATIONS>
    <LOCATION>
      <ID>1</ID>
      <Title>The Castle</Title>
      <Desc>A grand fortress</Desc>
      <Aka>Fort Knox</Aka>
    </LOCATION>
    <LOCATION>
      <ID>2</ID>
      <Title>The Forest</Title>
      <Aka>Sherwood</Aka>
    </LOCATION>
  </LOCATIONS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        let castle = parsed
            .locations
            .iter()
            .find(|l| l.name == "The Castle")
            .unwrap();
        let castle_desc = castle.description.as_ref().unwrap();
        assert!(castle_desc.contains("A grand fortress"));
        assert!(castle_desc.contains("<em>Also known as:</em> Fort Knox"));

        // Location with only Aka (no Desc) should still work
        let forest = parsed
            .locations
            .iter()
            .find(|l| l.name == "The Forest")
            .unwrap();
        let forest_desc = forest.description.as_ref().unwrap();
        assert!(forest_desc.contains("<em>Also known as:</em> Sherwood"));
    }

    #[test]
    fn test_scene_character_and_location_refs() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>References Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Chapter One</Title>
      <Type>0</Type>
      <Scenes>1</Scenes>
    </CHAPTER>
  </CHAPTERS>
  <SCENES>
    <SCENE>
      <ID>1</ID>
      <Title>Test Scene</Title>
      <Status>2</Status>
      <ReactionScene>0</ReactionScene>
      <Characters>1;2</Characters>
      <Locations>1</Locations>
    </SCENE>
  </SCENES>
  <CHARACTERS>
    <CHARACTER>
      <ID>1</ID>
      <Title>Alice</Title>
    </CHARACTER>
    <CHARACTER>
      <ID>2</ID>
      <Title>Bob</Title>
    </CHARACTER>
    <CHARACTER>
      <ID>3</ID>
      <Title>Charlie</Title>
    </CHARACTER>
  </CHARACTERS>
  <LOCATIONS>
    <LOCATION>
      <ID>1</ID>
      <Title>The Park</Title>
    </LOCATION>
  </LOCATIONS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Scene should reference Alice and Bob (not Charlie)
        assert_eq!(parsed.scene_character_refs.len(), 2);

        // Scene should reference The Park
        assert_eq!(parsed.scene_location_refs.len(), 1);

        // All characters should still be imported
        assert_eq!(parsed.characters.len(), 3);
    }

    // ========================================================================
    // XML Entity & Encoding Tests
    // ========================================================================

    #[test]
    fn test_xml_entity_unescaping() {
        // Note: trim_text(true) causes spaces around entities to be trimmed
        // So "Test &amp; Entities" becomes "Test&Entities" (space before & and after & are trimmed)
        // Use entities without surrounding spaces to test proper unescaping
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Test&amp;Entities</Title>
    <Desc>&lt;escaped&gt;&quot;quotes&quot;&apos;apostrophes&apos;</Desc>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>1</ID>
      <SortOrder>1</SortOrder>
      <Title>Chapter&amp;Stuff</Title>
      <Type>0</Type>
      <Scenes>1</Scenes>
    </CHAPTER>
  </CHAPTERS>
  <SCENES>
    <SCENE>
      <ID>1</ID>
      <Title>Scene&lt;tags&gt;</Title>
      <Goal>Goal&quot;quotes&quot;</Goal>
      <Status>2</Status>
      <ReactionScene>0</ReactionScene>
    </SCENE>
  </SCENES>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Project title should have unescaped ampersand
        assert_eq!(parsed.project.name, "Test&Entities");

        // Chapter title should have unescaped ampersand
        assert_eq!(parsed.chapters[0].title, "Chapter&Stuff");

        // Scene title should have unescaped angle brackets
        assert_eq!(parsed.scenes[0].title, "Scene<tags>");

        // Beat content should have unescaped quotes
        assert!(parsed.beats[0].content.contains("\"quotes\""));
    }

    #[test]
    fn test_decode_utf8_content() {
        // UTF-8 bytes with special characters
        let utf8_bytes = "Ça va très bien! 日本語 🎉".as_bytes();
        let result = decode_content(utf8_bytes);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Ça va très bien! 日本語 🎉");
    }

    #[test]
    fn test_decode_utf8_with_bom() {
        // UTF-8 BOM followed by content
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice("Hello UTF-8 BOM".as_bytes());

        let result = decode_content(&bytes);
        assert!(result.is_ok());
        // BOM should be handled (may or may not be stripped depending on implementation)
        assert!(result.unwrap().contains("Hello UTF-8 BOM"));
    }

    // ========================================================================
    // Source ID Tracking Tests
    // ========================================================================

    #[test]
    fn test_source_ids_are_preserved() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Source ID Test</Title>
  </PROJECT>
  <CHAPTERS>
    <CHAPTER>
      <ID>42</ID>
      <SortOrder>1</SortOrder>
      <Title>Chapter 42</Title>
      <Type>0</Type>
      <Scenes>99</Scenes>
    </CHAPTER>
  </CHAPTERS>
  <SCENES>
    <SCENE>
      <ID>99</ID>
      <Title>Scene 99</Title>
      <Goal>Test goal</Goal>
      <Status>2</Status>
      <ReactionScene>0</ReactionScene>
    </SCENE>
  </SCENES>
  <CHARACTERS>
    <CHARACTER>
      <ID>7</ID>
      <Title>Character 7</Title>
    </CHARACTER>
  </CHARACTERS>
  <LOCATIONS>
    <LOCATION>
      <ID>13</ID>
      <Title>Location 13</Title>
    </LOCATION>
  </LOCATIONS>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        // Chapter source_id should be "42"
        assert_eq!(parsed.chapters[0].source_id, Some("42".to_string()));

        // Scene source_id should be "99"
        assert_eq!(parsed.scenes[0].source_id, Some("99".to_string()));

        // Beat source_id should include scene ID
        assert!(parsed.beats[0]
            .source_id
            .as_ref()
            .unwrap()
            .starts_with("99-"));

        // Character source_id should be "7"
        assert_eq!(parsed.characters[0].source_id, Some("7".to_string()));

        // Location source_id should be "13"
        assert_eq!(parsed.locations[0].source_id, Some("13".to_string()));
    }

    // ========================================================================
    // Markup Conversion Tests
    // ========================================================================

    #[test]
    fn test_convert_ywriter_markup_nested() {
        // Test nested bold and italic
        let result = convert_ywriter_markup("[b][i]bold italic[/i][/b]");
        assert!(result.contains("<strong><em>bold italic</em></strong>"));
    }

    #[test]
    fn test_convert_ywriter_markup_empty_input() {
        let result = convert_ywriter_markup("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_ywriter_markup_whitespace_only() {
        let result = convert_ywriter_markup("   \n\n   ");
        assert_eq!(result, "");
    }

    #[test]
    fn test_convert_ywriter_markup_multiple_paragraphs() {
        let input = "First para.\n\nSecond para.\n\nThird para.";
        let result = convert_ywriter_markup(input);

        assert!(result.contains("<p>First para.</p>"));
        assert!(result.contains("<p>Second para.</p>"));
        assert!(result.contains("<p>Third para.</p>"));

        // Count paragraphs
        let p_count = result.matches("<p>").count();
        assert_eq!(p_count, 3);
    }

    // ========================================================================
    // Project Metadata Tests
    // ========================================================================

    #[test]
    fn test_project_author_is_set() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Authored Project</Title>
    <Author>Jane Doe</Author>
  </PROJECT>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(parsed.project.author_pen_name, Some("Jane Doe".to_string()));
    }

    #[test]
    fn test_project_source_type_is_ywriter() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Test</Title>
  </PROJECT>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("test.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert!(matches!(parsed.project.source_type, SourceType::YWriter));
    }

    #[test]
    fn test_project_source_path_is_set() {
        let xml = r#"<?xml version="1.0"?>
<YWRITER7>
  <PROJECT>
    <Title>Test</Title>
  </PROJECT>
</YWRITER7>"#;

        let result = parse_ywriter_content(xml, Path::new("/path/to/my/project.yw7"));
        assert!(result.is_ok());
        let parsed = result.unwrap();

        assert_eq!(
            parsed.project.source_path,
            Some("/path/to/my/project.yw7".to_string())
        );
    }

    // ========================================================================
    // Hamlet Fixture Additional Coverage
    // ========================================================================

    #[test]
    fn test_hamlet_scene_with_gco_but_no_prose() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // "Polonius Instructs Reynaldo" has Goal/Conflict/Outcome but no SceneContent
        let reynaldo_scene = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Polonius Instructs Reynaldo")
            .unwrap();

        let reynaldo_beats: Vec<_> = parsed
            .beats
            .iter()
            .filter(|b| b.scene_id == reynaldo_scene.id)
            .collect();

        // This scene has GCO but no prose - should have 3 beats
        assert_eq!(reynaldo_beats.len(), 3);

        // None of the beats should have prose
        for beat in &reynaldo_beats {
            assert!(
                beat.prose.is_none(),
                "Beat should not have prose when scene has no SceneContent"
            );
        }
    }

    #[test]
    fn test_hamlet_location_description_with_aka() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // The Battlements has an Aka of "Castle Walls"
        let battlements = parsed
            .locations
            .iter()
            .find(|l| l.name == "The Battlements")
            .unwrap();

        let desc = battlements.description.as_ref().unwrap();
        assert!(desc.contains("<em>Also known as:</em> Castle Walls"));
    }

    #[test]
    fn test_hamlet_character_with_bio_and_goals() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Hamlet has Bio, Goals, and Notes
        let hamlet = parsed
            .characters
            .iter()
            .find(|c| c.name == "Hamlet, Prince of Denmark")
            .unwrap();

        let desc = hamlet.description.as_ref().unwrap();
        assert!(desc.contains("<strong>Bio:</strong>"));
        assert!(desc.contains("<strong>Goals:</strong>"));
        assert!(desc.contains("Avenge his father"));
    }

    // ========================================================================
    // Hal Spacejock Fixture Tests (Official Simon Haynes Sample Project)
    // ========================================================================

    #[test]
    fn test_hal_spacejock_parses_successfully() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let result = parse_ywriter_file(&path);
        assert!(
            result.is_ok(),
            "Failed to parse Hal Spacejock: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_hal_spacejock_project_metadata() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        assert_eq!(
            parsed.project.name,
            "Hal Spacejock book one - Sample yWriter Project"
        );
        assert_eq!(
            parsed.project.author_pen_name,
            Some("Simon Haynes".to_string())
        );
        assert!(matches!(parsed.project.source_type, SourceType::YWriter));
    }

    #[test]
    fn test_hal_spacejock_chapter_count() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Has 32 chapters (Type=0) plus 1 info chapter (Type=1) which is filtered out
        // Also has Chapter 33 which is Unused=-1 and Chapter 14 which has no scenes
        assert!(
            parsed.chapters.len() >= 30,
            "Expected at least 30 chapters, got {}",
            parsed.chapters.len()
        );
    }

    #[test]
    fn test_hal_spacejock_scene_count() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // The file has many scenes but some are unused/filtered
        // Verify we have a reasonable number (at least 30 used scenes)
        assert!(
            parsed.scenes.len() >= 30,
            "Expected at least 30 scenes, got {}",
            parsed.scenes.len()
        );
    }

    #[test]
    fn test_hal_spacejock_character_count() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Has 12 characters: Hal, Jerling, Clunk, Vurdi, Farrell, Terry, Carina, Mike, Gordon, Navcom, Brutus, Portmaster
        assert_eq!(parsed.characters.len(), 12);
    }

    #[test]
    fn test_hal_spacejock_location_count() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Has 4 locations: Black Gull, Lamira Spaceport, Planet Forg, Jerling Enterprises
        assert_eq!(parsed.locations.len(), 4);
    }

    #[test]
    fn test_hal_spacejock_character_references() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 "Meet Hal, threatened by Vurdi" has characters: Hal, Navcom, Vurdi, Brutus
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        let scene1_char_refs: Vec<_> = parsed
            .scene_character_refs
            .iter()
            .filter(|(scene_id, _)| *scene_id == scene1.id)
            .collect();

        assert_eq!(
            scene1_char_refs.len(),
            4,
            "Scene 1 should reference 4 characters"
        );
    }

    #[test]
    fn test_hal_spacejock_location_references() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 "Meet Hal, threatened by Vurdi" has locations: Black Gull, Lamira Spaceport
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        let scene1_loc_refs: Vec<_> = parsed
            .scene_location_refs
            .iter()
            .filter(|(scene_id, _)| *scene_id == scene1.id)
            .collect();

        assert_eq!(
            scene1_loc_refs.len(),
            2,
            "Scene 1 should reference 2 locations"
        );
    }

    #[test]
    fn test_hal_spacejock_scene_with_prose() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 has scene content starting with "Hal Spacejock was sitting..."
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        // Find the beat for this scene that has prose
        let prose_beat = parsed
            .beats
            .iter()
            .find(|b| b.scene_id == scene1.id && b.prose.is_some())
            .expect("Scene 1 should have a beat with prose");

        let prose = prose_beat.prose.as_ref().unwrap();
        assert!(prose.contains("Hal Spacejock was sitting"));
        assert!(prose.contains("Black Gull's flight console"));
    }

    #[test]
    fn test_hal_spacejock_project_notes_imported() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        let description = parsed.project.description.as_ref().unwrap();
        assert!(description.contains("Project Notes:"));
        assert!(description.contains("About the sample project"));
        assert!(description.contains("This sample project contains the outline"));
    }

    #[test]
    fn test_hal_spacejock_scene_with_detailed_synopsis() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 has a detailed synopsis in the Desc field
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        let synopsis = scene1.synopsis.as_ref().unwrap();
        assert!(synopsis.contains("Hal Spacejock is aboard his ship"));
        assert!(synopsis.contains("Vurdi the debt collector"));
    }

    #[test]
    fn test_hal_spacejock_unused_scene_filtered() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene ID 58 "New Scene" has Unused=-1 and should be filtered out
        let unused_scene = parsed.scenes.iter().find(|s| {
            s.title == "New Scene"
                && s.synopsis
                    .as_ref()
                    .is_some_and(|syn| syn.contains("This bird is not dead"))
        });

        assert!(
            unused_scene.is_none(),
            "Unused scene should be filtered out"
        );
    }

    #[test]
    fn test_hal_spacejock_info_chapter_filtered() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Chapter "Info" has Type=1 (non-normal) and should be filtered out
        let info_chapter = parsed.chapters.iter().find(|c| c.title == "Info");

        assert!(
            info_chapter.is_none(),
            "Info chapter (Type=1) should be filtered out"
        );
    }

    #[test]
    fn test_hal_spacejock_character_with_full_details() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // "Hal" character has extensive bio
        let hal = parsed
            .characters
            .iter()
            .find(|c| c.name == "Hal Spacejock")
            .expect("Should find Hal character");

        let desc = hal.description.as_ref().unwrap();
        assert!(desc.contains("cargo game"));
        assert!(desc.contains("strong sense of justice"));
    }

    #[test]
    fn test_hal_spacejock_character_with_goals() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // "Jerling" character has Goals defined
        let jerling = parsed
            .characters
            .iter()
            .find(|c| c.name == "Walter Jerling")
            .expect("Should find Jerling character");

        let desc = jerling.description.as_ref().unwrap();
        assert!(desc.contains("<strong>Goals:</strong>"));
        assert!(desc.contains("urgent shipment"));
    }

    #[test]
    fn test_hal_spacejock_location_with_description() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // "Black Gull" has a description
        let black_gull = parsed
            .locations
            .iter()
            .find(|l| l.name == "Black Gull")
            .expect("Should find Black Gull location");

        let desc = black_gull.description.as_ref().unwrap();
        assert!(desc.contains("decrepit old freighter"));
        assert!(desc.contains("dodgy loan"));
    }

    #[test]
    fn test_hal_spacejock_beats_have_correct_positions() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 should have beats with incrementing positions
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        let scene1_beats: Vec<_> = parsed
            .beats
            .iter()
            .filter(|b| b.scene_id == scene1.id)
            .collect();

        // Verify positions are sequential
        let mut positions: Vec<_> = scene1_beats.iter().map(|b| b.position).collect();
        positions.sort();

        for (i, pos) in positions.iter().enumerate() {
            assert_eq!(*pos, i as i32, "Beat positions should be sequential");
        }
    }

    #[test]
    fn test_hal_spacejock_inline_comment_in_prose() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 1 has inline comments like /* this is the author's first note */
        let scene1 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Meet Hal, threatened by Vurdi")
            .expect("Should find scene 1");

        let prose_beat = parsed
            .beats
            .iter()
            .find(|b| b.scene_id == scene1.id && b.prose.is_some())
            .expect("Scene 1 should have prose");

        let prose = prose_beat.prose.as_ref().unwrap();
        // The inline comment should be present in the content
        assert!(prose.contains("/* this is the author's first note */"));
    }

    #[test]
    fn test_hal_spacejock_ywriter_markup() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Scene 5 "Hal going to port control" has [i]Black Gull[/i] markup
        let scene5 = parsed
            .scenes
            .iter()
            .find(|s| s.title == "Hal going to port control")
            .expect("Should find scene 5");

        let prose_beat = parsed
            .beats
            .iter()
            .find(|b| b.scene_id == scene5.id && b.prose.is_some());

        if let Some(beat) = prose_beat {
            let prose = beat.prose.as_ref().unwrap();
            // Check that italic markup was converted to HTML
            assert!(
                prose.contains("<em>Black Gull</em>"),
                "yWriter [i] markup should be converted to HTML <em>"
            );
        }
    }

    // ========================================================================
    // Parts (SectionStart) Tests
    // ========================================================================

    #[test]
    fn test_parts_example_parses_successfully() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts_example.yw7");

        let result = parse_ywriter_file(&path);
        assert!(
            result.is_ok(),
            "Failed to parse parts_example.yw7: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_parts_example_project_metadata() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts_example.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();
        assert_eq!(parsed.project.name, "A Novel in Parts");
    }

    #[test]
    fn test_parts_example_has_part_chapters() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts_example.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Should have 5 chapters total: Part 1, Ch 1, Ch 2, Part 2, Ch 3
        assert_eq!(parsed.chapters.len(), 5);

        // Part 1 and Part 2 should be marked as is_part
        let part_chapters: Vec<_> = parsed.chapters.iter().filter(|ch| ch.is_part).collect();
        assert_eq!(part_chapters.len(), 2, "Should have 2 Part chapters");

        // Verify the Part chapters are named correctly
        let part_names: Vec<_> = part_chapters.iter().map(|ch| ch.title.as_str()).collect();
        assert!(part_names.contains(&"Part 1"), "Should have Part 1");
        assert!(part_names.contains(&"Part 2"), "Should have Part 2");
    }

    #[test]
    fn test_parts_example_regular_chapters_not_parts() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts_example.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Regular chapters should NOT be marked as is_part
        let regular_chapters: Vec<_> = parsed.chapters.iter().filter(|ch| !ch.is_part).collect();
        assert_eq!(regular_chapters.len(), 3, "Should have 3 regular chapters");

        let chapter_names: Vec<_> = regular_chapters
            .iter()
            .map(|ch| ch.title.as_str())
            .collect();
        assert!(chapter_names.contains(&"Chapter 1"));
        assert!(chapter_names.contains(&"Chapter 2"));
        assert!(chapter_names.contains(&"Chapter 3"));
    }

    #[test]
    fn test_parts_example_chapter_ordering() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts_example.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        // Verify the ordering is correct: Part 1, Ch 1, Ch 2, Part 2, Ch 3
        let titles: Vec<_> = parsed.chapters.iter().map(|ch| ch.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["Part 1", "Chapter 1", "Chapter 2", "Part 2", "Chapter 3"]
        );
    }

    #[test]
    fn test_hal_spacejock_has_no_parts() {
        // Verify that hal_spacejock.yw7 (which has no SectionStart elements) has no Parts
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hal_spacejock.yw7");

        let parsed = parse_ywriter_file(&path).unwrap();

        let part_chapters: Vec<_> = parsed.chapters.iter().filter(|ch| ch.is_part).collect();
        assert_eq!(
            part_chapters.len(),
            0,
            "hal_spacejock should have no Part chapters"
        );
    }
}
