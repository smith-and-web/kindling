use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::models::{Project, Chapter, Scene, Beat, Character, Location, SourceType};

#[derive(Debug, Error)]
pub enum PlottrError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Invalid Plottr file structure")]
    InvalidStructure,
}

// ============================================================================
// Plottr File Structure (from .pltr JSON files)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct PlottrFile {
    #[serde(default)]
    pub chapters: Vec<PlottrChapter>,
    #[serde(default)]
    pub lines: Vec<PlottrLine>,
    #[serde(default)]
    pub cards: Vec<PlottrCard>,
    #[serde(default)]
    pub characters: Vec<PlottrCharacter>,
    #[serde(default)]
    pub places: Vec<PlottrPlace>,
    #[serde(default)]
    pub tags: Option<Vec<PlottrTag>>,
}

#[derive(Debug, Deserialize)]
pub struct PlottrChapter {
    pub id: serde_json::Value, // Can be string or number
    pub title: String,
    #[serde(default)]
    pub position: i32,
    #[serde(rename = "autoOutlineSort")]
    pub auto_outline_sort: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PlottrLine {
    pub id: serde_json::Value,
    pub title: String,
    pub color: Option<String>,
    #[serde(default)]
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct PlottrCard {
    pub id: serde_json::Value,
    #[serde(rename = "lineId")]
    pub line_id: serde_json::Value,
    #[serde(rename = "chapterId")]
    pub chapter_id: serde_json::Value,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub characters: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub places: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub position: i32,
}

#[derive(Debug, Deserialize)]
pub struct PlottrCharacter {
    pub id: serde_json::Value,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    #[serde(flatten)]
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PlottrPlace {
    pub id: serde_json::Value,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    #[serde(flatten)]
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

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
// Parser Implementation
// ============================================================================

fn value_to_string(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        _ => val.to_string(),
    }
}

pub fn parse_plottr_file<P: AsRef<Path>>(path: P) -> Result<ParsedPlottr, PlottrError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;
    let plottr: PlottrFile = serde_json::from_str(&content)?;

    // Extract project name from filename
    let project_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let project = Project::new(
        project_name,
        SourceType::Plottr,
        Some(path.to_string_lossy().to_string()),
    );

    // Parse chapters
    let mut chapters: Vec<Chapter> = plottr.chapters
        .iter()
        .enumerate()
        .map(|(idx, ch)| {
            Chapter::new(
                project.id,
                ch.title.clone(),
                ch.position.max(idx as i32),
            )
        })
        .collect();

    chapters.sort_by_key(|c| c.position);

    // Build chapter ID map (Plottr ID -> our Chapter)
    let chapter_map: HashMap<String, &Chapter> = plottr.chapters
        .iter()
        .zip(chapters.iter())
        .map(|(plottr_ch, ch)| (value_to_string(&plottr_ch.id), ch))
        .collect();

    // Parse characters
    let characters: Vec<Character> = plottr.characters
        .iter()
        .map(|pc| {
            let mut attrs = HashMap::new();
            if let Some(notes) = &pc.notes {
                attrs.insert("notes".to_string(), notes.clone());
            }
            // Add custom attributes (excluding known fields)
            for (key, value) in &pc.custom_attributes {
                if !["id", "name", "description", "notes"].contains(&key.as_str()) {
                    if let Some(s) = value.as_str() {
                        attrs.insert(key.clone(), s.to_string());
                    }
                }
            }

            Character::new(
                project.id,
                pc.name.clone(),
                pc.description.clone(),
                Some(value_to_string(&pc.id)),
            ).with_attributes(attrs)
        })
        .collect();

    // Build character ID map
    let character_map: HashMap<String, &Character> = plottr.characters
        .iter()
        .zip(characters.iter())
        .map(|(pc, ch)| (value_to_string(&pc.id), ch))
        .collect();

    // Parse locations (places)
    let locations: Vec<Location> = plottr.places
        .iter()
        .map(|pp| {
            let mut attrs = HashMap::new();
            if let Some(notes) = &pp.notes {
                attrs.insert("notes".to_string(), notes.clone());
            }
            for (key, value) in &pp.custom_attributes {
                if !["id", "name", "description", "notes"].contains(&key.as_str()) {
                    if let Some(s) = value.as_str() {
                        attrs.insert(key.clone(), s.to_string());
                    }
                }
            }

            Location::new(
                project.id,
                pp.name.clone(),
                pp.description.clone(),
                Some(value_to_string(&pp.id)),
            ).with_attributes(attrs)
        })
        .collect();

    // Build location ID map
    let location_map: HashMap<String, &Location> = plottr.places
        .iter()
        .zip(locations.iter())
        .map(|(pp, loc)| (value_to_string(&pp.id), loc))
        .collect();

    // Parse cards as scenes (grouping by chapter)
    let mut scenes: Vec<Scene> = Vec::new();
    let mut beats: Vec<Beat> = Vec::new();
    let mut scene_character_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();
    let mut scene_location_refs: Vec<(uuid::Uuid, uuid::Uuid)> = Vec::new();

    // Group cards by chapter
    let mut cards_by_chapter: HashMap<String, Vec<&PlottrCard>> = HashMap::new();
    for card in &plottr.cards {
        let chapter_id = value_to_string(&card.chapter_id);
        cards_by_chapter
            .entry(chapter_id)
            .or_default()
            .push(card);
    }

    // Create scenes from cards
    for (chapter_id_str, cards) in cards_by_chapter {
        if let Some(chapter) = chapter_map.get(&chapter_id_str) {
            let mut sorted_cards = cards;
            sorted_cards.sort_by_key(|c| c.position);

            for (idx, card) in sorted_cards.iter().enumerate() {
                let scene = Scene::new(
                    chapter.id,
                    card.title.clone(),
                    card.description.clone(),
                    idx as i32,
                );

                // Create a beat from the card description if present
                if let Some(desc) = &card.description {
                    if !desc.trim().is_empty() {
                        let beat = Beat::new(scene.id, desc.clone(), 0);
                        beats.push(beat);
                    }
                }

                // Track character references
                if let Some(char_ids) = &card.characters {
                    for char_id in char_ids {
                        let id_str = value_to_string(char_id);
                        if let Some(character) = character_map.get(&id_str) {
                            scene_character_refs.push((scene.id, character.id));
                        }
                    }
                }

                // Track location references
                if let Some(place_ids) = &card.places {
                    for place_id in place_ids {
                        let id_str = value_to_string(place_id);
                        if let Some(location) = location_map.get(&id_str) {
                            scene_location_refs.push((scene.id, location.id));
                        }
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

    #[test]
    fn test_value_to_string() {
        assert_eq!(value_to_string(&serde_json::json!("test")), "test");
        assert_eq!(value_to_string(&serde_json::json!(123)), "123");
        assert_eq!(value_to_string(&serde_json::json!(12.5)), "12.5");
    }
}
