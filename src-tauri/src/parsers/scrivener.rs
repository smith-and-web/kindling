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

fn parse_binder_xml(xml_content: &str) -> Result<Vec<BinderItem>, ScrivenerError> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut stack: Vec<BinderItem> = Vec::new();
    let mut current_uuid = String::new();
    let mut current_type = BinderItemType::Other("Unknown".to_string());
    let mut current_title = String::new();
    let mut in_title = false;
    let mut in_binder_item = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                match e.name().as_ref() {
                    b"BinderItem" => {
                        in_binder_item = true;
                        current_uuid = String::new();
                        current_type = BinderItemType::Other("Unknown".to_string());
                        current_title = String::new();

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"UUID" => {
                                    current_uuid = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"Type" => {
                                    current_type = BinderItemType::from(
                                        String::from_utf8_lossy(&attr.value).as_ref(),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    b"Title" if in_binder_item => {
                        in_title = true;
                    }
                    b"Children" if in_binder_item => {
                        // Push current item to stack, we're going into children
                        let item = BinderItem {
                            uuid: current_uuid.clone(),
                            item_type: current_type.clone(),
                            title: current_title.clone(),
                            children: Vec::new(),
                        };
                        stack.push(item);
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) if in_title => {
                current_title = String::from_utf8_lossy(&e).to_string();
            }
            Ok(Event::End(e)) => {
                match e.name().as_ref() {
                    b"Title" => {
                        in_title = false;
                    }
                    b"BinderItem" => {
                        let item = BinderItem {
                            uuid: current_uuid.clone(),
                            item_type: current_type.clone(),
                            title: current_title.clone(),
                            children: Vec::new(),
                        };

                        if let Some(mut parent) = stack.pop() {
                            parent.children.push(item);
                            stack.push(parent);
                        } else {
                            items.push(item);
                        }
                        in_binder_item = false;
                    }
                    b"Children" => {
                        // Pop from stack and finalize
                        if let Some(completed) = stack.pop() {
                            if let Some(mut parent) = stack.pop() {
                                parent.children.push(completed);
                                stack.push(parent);
                            } else {
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

    // Handle any remaining items on the stack
    while let Some(item) = stack.pop() {
        if let Some(mut parent) = stack.pop() {
            parent.children.push(item);
            stack.push(parent);
        } else {
            items.push(item);
        }
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
                            // Read synopsis as beat content
                            let synopsis = read_synopsis(scriv_path, &scene_item.uuid);

                            let scene = Scene::new(
                                chapter.id,
                                scene_item.title.clone(),
                                synopsis.clone(),
                                sc_idx as i32,
                            );

                            // Create beat from synopsis
                            if let Some(syn) = synopsis {
                                if !syn.trim().is_empty() {
                                    let beat = Beat::new(scene.id, syn, 0);
                                    beats.push(beat);
                                }
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
                    let scene = Scene::new(chapter.id, child.title.clone(), synopsis.clone(), 0);

                    if let Some(syn) = synopsis {
                        if !syn.trim().is_empty() {
                            let beat = Beat::new(scene.id, syn, 0);
                            beats.push(beat);
                        }
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
