use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use walkdir::WalkDir;

use crate::models::{
    Beat, Chapter, Character, Location, Project, Scene, SceneStatus, SceneType, SourceType,
};

const LONGFORM_DEFAULT_CHAPTER_SOURCE_ID: &str = "longform:default";
const LONGFORM_BEATS_MARKER: &str = "<!-- kindling: beats -->";

#[derive(Debug, Error)]
pub enum LongformError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Invalid Longform structure: {0}")]
    InvalidStructure(String),
}

// ============================================================================
// Parsed Output
// ============================================================================

pub struct ParsedLongform {
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
// Longform Index Frontmatter
// ============================================================================

#[derive(Debug, Deserialize)]
struct LongformFrontmatter {
    longform: Option<LongformIndex>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LongformIndex {
    format: Option<String>,
    title: Option<String>,
    #[serde(rename = "draftNumber")]
    draft_number: Option<i32>,
    workflow: Option<String>,
    #[serde(rename = "sceneFolder")]
    scene_folder: Option<String>,
    scenes: Option<serde_yaml::Value>,
    #[serde(rename = "sceneTemplate")]
    scene_template: Option<String>,
    #[serde(rename = "ignoredFiles")]
    ignored_files: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct SceneFrontmatter {
    synopsis: Option<String>,
    status: Option<String>,
    pov: Option<String>,
    characters: Option<FrontmatterList>,
    setting: Option<FrontmatterList>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FrontmatterList {
    Single(String),
    List(Vec<String>),
}

impl FrontmatterList {
    fn into_vec(self) -> Vec<String> {
        match self {
            FrontmatterList::Single(value) => vec![value],
            FrontmatterList::List(values) => values,
        }
    }
}

// ============================================================================
// Scene Content Parsing
// ============================================================================

struct BeatContent {
    content: String,
    prose: Option<String>,
}

struct SceneContent {
    synopsis: Option<String>,
    prose: Option<String>,
    scene_type: SceneType,
    scene_status: SceneStatus,
    beats: Vec<BeatContent>,
    characters: Vec<String>,
    locations: Vec<String>,
}

struct SceneEntry {
    name: String,
    _depth: usize,
}

// ============================================================================
// Parser Implementation
// ============================================================================

pub fn parse_longform_index<P: AsRef<Path>>(path: P) -> Result<ParsedLongform, LongformError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    let (frontmatter_str, _) = split_frontmatter(&content)
        .ok_or_else(|| LongformError::InvalidStructure("Missing YAML frontmatter".to_string()))?;

    let frontmatter: LongformFrontmatter = serde_yaml::from_str(&frontmatter_str)?;
    let longform = frontmatter.longform.ok_or_else(|| {
        LongformError::InvalidStructure("Missing longform frontmatter entry".to_string())
    })?;

    let format = longform.format.as_deref().unwrap_or("").to_lowercase();
    if format != "scenes" {
        return Err(LongformError::InvalidStructure(
            "Only multi-scene Longform projects are supported".to_string(),
        ));
    }

    let scene_folder = longform.scene_folder.unwrap_or_else(|| "/".to_string());
    let scene_folder = normalize_scene_folder(&scene_folder);

    let scenes_value = longform.scenes.ok_or_else(|| {
        LongformError::InvalidStructure("Missing longform.scenes list".to_string())
    })?;

    let scene_entries = parse_scene_entries(&scenes_value)?;
    let ignored_patterns = longform.ignored_files.unwrap_or_default();

    let project_name = longform
        .title
        .clone()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        });

    let project = Project::new(
        project_name,
        SourceType::Longform,
        Some(path.to_string_lossy().to_string()),
    );

    let index_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let scene_dir = resolve_scene_dir(index_dir, &scene_folder);

    build_longform_structure(
        project,
        scene_entries,
        &ignored_patterns,
        index_dir,
        &scene_dir,
    )
}

pub fn parse_longform_path<P: AsRef<Path>>(path: P) -> Result<ParsedLongform, LongformError> {
    let path = path.as_ref();
    if path.is_dir() {
        let indexes = find_longform_indexes(path)?;
        if indexes.is_empty() {
            return Err(LongformError::InvalidStructure(
                "No Longform index files found in vault".to_string(),
            ));
        }
        if indexes.len() > 1 {
            let list = indexes
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join("\n");
            return Err(LongformError::InvalidStructure(format!(
                "Multiple Longform index files found. Please pick one:\n{list}"
            )));
        }
        return parse_longform_index(&indexes[0]);
    }

    parse_longform_index(path)
}

fn find_longform_indexes(vault_dir: &Path) -> Result<Vec<PathBuf>, LongformError> {
    let mut indexes = Vec::new();

    for entry in WalkDir::new(vault_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = fs::read_to_string(entry.path())?;
        let (frontmatter_str, _) = match split_frontmatter(&content) {
            Some(parts) => parts,
            None => continue,
        };

        let frontmatter: LongformFrontmatter = match serde_yaml::from_str(&frontmatter_str) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if let Some(longform) = frontmatter.longform {
            let format = longform.format.unwrap_or_default().to_lowercase();
            if format == "scenes" || format == "single" {
                indexes.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(indexes)
}

fn parse_scene_entries(value: &serde_yaml::Value) -> Result<Vec<SceneEntry>, LongformError> {
    let mut entries = Vec::new();
    let scenes = value.as_sequence().ok_or_else(|| {
        LongformError::InvalidStructure("longform.scenes must be a list".to_string())
    })?;

    collect_scene_entries(scenes, 0, &mut entries)?;

    Ok(entries)
}

fn collect_scene_entries(
    scenes: &[serde_yaml::Value],
    depth: usize,
    entries: &mut Vec<SceneEntry>,
) -> Result<(), LongformError> {
    for entry in scenes {
        match entry {
            serde_yaml::Value::Sequence(children) => {
                collect_scene_entries(children, depth + 1, entries)?;
            }
            serde_yaml::Value::String(name) => entries.push(SceneEntry {
                name: name.clone(),
                _depth: depth,
            }),
            serde_yaml::Value::Number(num) => entries.push(SceneEntry {
                name: num.to_string(),
                _depth: depth,
            }),
            serde_yaml::Value::Bool(value) => entries.push(SceneEntry {
                name: value.to_string(),
                _depth: depth,
            }),
            _ => {
                return Err(LongformError::InvalidStructure(
                    "Scene names must be strings".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn parse_scene_file(path: &Path) -> Result<SceneContent, LongformError> {
    let content = fs::read_to_string(path)?;
    let (frontmatter, body) = match split_frontmatter(&content) {
        Some((frontmatter_str, body)) => {
            let parsed = serde_yaml::from_str::<SceneFrontmatter>(&frontmatter_str).ok();
            (parsed, body)
        }
        None => (None, content),
    };

    let mut scene_content = parse_scene_body(&body);
    if let Some(frontmatter) = frontmatter {
        if scene_content.synopsis.is_none() {
            if let Some(synopsis) = normalize_block(frontmatter.synopsis.as_deref().unwrap_or("")) {
                scene_content.synopsis = Some(synopsis);
            }
        }
        if let Some(status) = frontmatter.status {
            scene_content.scene_status = parse_obsidian_status(&status);
        }
        let mut characters: Vec<String> = Vec::new();
        let mut locations: Vec<String> = Vec::new();
        if let Some(list) = frontmatter.characters {
            characters.extend(list.into_vec());
        }
        if let Some(pov) = frontmatter.pov {
            characters.push(pov);
        }
        if let Some(list) = frontmatter.setting {
            locations.extend(list.into_vec());
        }
        scene_content.characters = normalize_reference_list(characters);
        scene_content.locations = normalize_reference_list(locations);
    }

    Ok(scene_content)
}

fn parse_scene_body(content: &str) -> SceneContent {
    let mut scene_type = SceneType::Normal;
    let mut scene_status = SceneStatus::Draft;
    let mut synopsis = None;
    let mut body_lines = Vec::new();
    let mut beat_lines = Vec::new();
    let mut in_beats = false;
    let mut metadata_parsed = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if !in_beats && trimmed.eq_ignore_ascii_case(LONGFORM_BEATS_MARKER) {
            in_beats = true;
            continue;
        }

        if !in_beats && !metadata_parsed {
            if let Some(meta) = parse_kindling_comment(trimmed) {
                if let Some(value) = meta.get("scene_type") {
                    scene_type = SceneType::parse(value);
                }
                if let Some(value) = meta.get("scene_status") {
                    scene_status = SceneStatus::parse(value);
                }
                if let Some(value) = meta.get("synopsis") {
                    let synopsis_value = value.trim();
                    if !synopsis_value.is_empty() {
                        synopsis = Some(synopsis_value.to_string());
                    }
                }
                metadata_parsed = true;
                continue;
            }
        }

        if in_beats {
            beat_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    let prose = normalize_block(&body_lines.join("\n"));
    let beats = parse_beats_block(&beat_lines.join("\n"));

    SceneContent {
        synopsis,
        prose,
        scene_type,
        scene_status,
        beats,
        characters: Vec::new(),
        locations: Vec::new(),
    }
}

fn parse_beats_block(content: &str) -> Vec<BeatContent> {
    let mut beats = Vec::new();
    let mut current_content: Option<String> = None;
    let mut prose_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_end();
        let trimmed_start = trimmed.trim_start();
        let bullet = trimmed_start
            .strip_prefix("- ")
            .or_else(|| trimmed_start.strip_prefix("* "));

        if let Some(bullet_content) = bullet {
            if let Some(existing) = current_content.take() {
                let prose = normalize_block(&prose_lines.join("\n"));
                beats.push(BeatContent {
                    content: existing,
                    prose,
                });
                prose_lines.clear();
            }
            let beat_title = bullet_content.trim().to_string();
            if !beat_title.is_empty() {
                current_content = Some(beat_title);
            }
        } else if current_content.is_some() {
            prose_lines.push(trimmed.trim_start().to_string());
        }
    }

    if let Some(existing) = current_content.take() {
        let prose = normalize_block(&prose_lines.join("\n"));
        beats.push(BeatContent {
            content: existing,
            prose,
        });
    }

    beats
}

fn normalize_block(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_scene_name(name: &str) -> String {
    name.trim().to_string()
}

fn chapter_source_id_from_title(title: &str) -> String {
    format!("longform:chapter:{}", normalize_scene_name(title))
}

fn should_ignore_scene(name: &str, patterns: &[String]) -> bool {
    let trimmed = normalize_scene_name(name);
    if trimmed.is_empty() {
        return true;
    }
    let file_name = format!("{trimmed}.md");
    patterns
        .iter()
        .any(|pattern| wildcard_match(pattern, &trimmed) || wildcard_match(pattern, &file_name))
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    let mut p = 0;
    let mut t = 0;
    let mut star_index: Option<usize> = None;
    let mut match_index = 0;

    while t < text_chars.len() {
        if p < pattern_chars.len() && (pattern_chars[p] == '?' || pattern_chars[p] == text_chars[t])
        {
            p += 1;
            t += 1;
        } else if p < pattern_chars.len() && pattern_chars[p] == '*' {
            star_index = Some(p);
            match_index = t;
            p += 1;
        } else if let Some(star_pos) = star_index {
            p = star_pos + 1;
            match_index += 1;
            t = match_index;
        } else {
            return false;
        }
    }

    while p < pattern_chars.len() && pattern_chars[p] == '*' {
        p += 1;
    }

    p == pattern_chars.len()
}

fn parse_obsidian_status(raw: &str) -> SceneStatus {
    match raw.trim().to_lowercase().as_str() {
        "revised" => SceneStatus::Revised,
        "final" => SceneStatus::Final,
        "idea" | "outline" | "draft" => SceneStatus::Draft,
        _ => SceneStatus::Draft,
    }
}

fn normalize_reference_list(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();

    for value in values {
        if let Some(name) = normalize_reference_name(&value) {
            let key = name.to_lowercase();
            if seen.insert(key) {
                normalized.push(name);
            }
        }
    }

    normalized
}

fn normalize_reference_name(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let name = parse_wikilink_target(trimmed).unwrap_or_else(|| trimmed.to_string());
    let cleaned = name.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn parse_wikilink_target(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if !trimmed.starts_with("[[") || !trimmed.ends_with("]]") {
        return None;
    }

    let inner = &trimmed[2..trimmed.len() - 2];
    let target = inner.split('|').next()?.trim();
    if target.is_empty() {
        None
    } else {
        Some(target.to_string())
    }
}

fn build_longform_structure(
    project: Project,
    scene_entries: Vec<SceneEntry>,
    ignored_patterns: &[String],
    index_dir: &Path,
    scene_dir: &Path,
) -> Result<ParsedLongform, LongformError> {
    let mut chapters = Vec::new();
    let mut scenes = Vec::new();
    let mut beats = Vec::new();
    let mut characters = Vec::new();
    let mut locations = Vec::new();
    let mut scene_character_refs = Vec::new();
    let mut scene_location_refs = Vec::new();
    let mut character_index: HashMap<String, uuid::Uuid> = HashMap::new();
    let mut location_index: HashMap<String, uuid::Uuid> = HashMap::new();

    let has_hierarchy = scene_entries.iter().any(|entry| entry._depth > 0);

    if !has_hierarchy {
        let chapter = Chapter::new(project.id, "Chapter 1".to_string(), 0)
            .with_source_id(Some(LONGFORM_DEFAULT_CHAPTER_SOURCE_ID.to_string()));
        let mut scene_position = 0;

        for entry in scene_entries {
            if should_ignore_scene(&entry.name, ignored_patterns) {
                continue;
            }
            add_scene_from_entry(
                &chapter,
                entry,
                scene_position,
                index_dir,
                scene_dir,
                &mut scenes,
                &mut beats,
                &mut characters,
                &mut locations,
                &mut scene_character_refs,
                &mut scene_location_refs,
                &mut character_index,
                &mut location_index,
            )?;
            scene_position += 1;
        }

        chapters.push(chapter);
    } else {
        let mut current_chapter: Option<Chapter> = None;
        let mut chapter_position = 0;
        let mut scene_position = 0;
        let mut chapter_has_scenes = false;

        for entry in scene_entries {
            if entry._depth == 0 {
                if let Some(chapter) = current_chapter.take() {
                    if chapter_has_scenes {
                        chapters.push(chapter);
                    }
                }
                chapter_has_scenes = false;
                scene_position = 0;

                current_chapter = Some(
                    Chapter::new(project.id, entry.name.clone(), chapter_position)
                        .with_source_id(Some(chapter_source_id_from_title(&entry.name))),
                );
                chapter_position += 1;

                if !should_ignore_scene(&entry.name, ignored_patterns) {
                    if let Some(ref chapter) = current_chapter {
                        add_scene_from_entry(
                            chapter,
                            entry,
                            scene_position,
                            index_dir,
                            scene_dir,
                            &mut scenes,
                            &mut beats,
                            &mut characters,
                            &mut locations,
                            &mut scene_character_refs,
                            &mut scene_location_refs,
                            &mut character_index,
                            &mut location_index,
                        )?;
                        scene_position += 1;
                        chapter_has_scenes = true;
                    }
                }
            } else {
                if current_chapter.is_none() {
                    current_chapter = Some(
                        Chapter::new(project.id, "Chapter 1".to_string(), chapter_position)
                            .with_source_id(Some(LONGFORM_DEFAULT_CHAPTER_SOURCE_ID.to_string())),
                    );
                    chapter_position += 1;
                    scene_position = 0;
                }
                if !should_ignore_scene(&entry.name, ignored_patterns) {
                    if let Some(ref chapter) = current_chapter {
                        add_scene_from_entry(
                            chapter,
                            entry,
                            scene_position,
                            index_dir,
                            scene_dir,
                            &mut scenes,
                            &mut beats,
                            &mut characters,
                            &mut locations,
                            &mut scene_character_refs,
                            &mut scene_location_refs,
                            &mut character_index,
                            &mut location_index,
                        )?;
                        scene_position += 1;
                        chapter_has_scenes = true;
                    }
                }
            }
        }

        if let Some(chapter) = current_chapter.take() {
            if chapter_has_scenes {
                chapters.push(chapter);
            }
        }
    }

    Ok(ParsedLongform {
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

fn add_scene_from_entry(
    chapter: &Chapter,
    entry: SceneEntry,
    scene_position: i32,
    index_dir: &Path,
    scene_dir: &Path,
    scenes: &mut Vec<Scene>,
    beats: &mut Vec<Beat>,
    characters: &mut Vec<Character>,
    locations: &mut Vec<Location>,
    scene_character_refs: &mut Vec<(uuid::Uuid, uuid::Uuid)>,
    scene_location_refs: &mut Vec<(uuid::Uuid, uuid::Uuid)>,
    character_index: &mut HashMap<String, uuid::Uuid>,
    location_index: &mut HashMap<String, uuid::Uuid>,
) -> Result<(), LongformError> {
    let scene_name = normalize_scene_name(&entry.name);
    let scene_file_name = ensure_markdown_extension(&scene_name);
    let scene_path = scene_dir.join(&scene_file_name);
    let scene_source_id = build_scene_source_id(index_dir, &scene_path);

    let scene_content = parse_scene_file(&scene_path)?;

    let mut scene = Scene::new(
        chapter.id,
        scene_name,
        scene_content.synopsis,
        scene_position,
    )
    .with_source_id(Some(scene_source_id));
    scene.prose = scene_content.prose;
    scene.scene_type = scene_content.scene_type;
    scene.scene_status = scene_content.scene_status;

    register_scene_characters(
        chapter.project_id,
        scene.id,
        &scene_content.characters,
        characters,
        scene_character_refs,
        character_index,
    );
    register_scene_locations(
        chapter.project_id,
        scene.id,
        &scene_content.locations,
        locations,
        scene_location_refs,
        location_index,
    );

    for (beat_position, beat) in scene_content.beats.into_iter().enumerate() {
        let mut new_beat = Beat::new(scene.id, beat.content, beat_position as i32);
        new_beat.prose = beat.prose;
        beats.push(new_beat);
    }

    scenes.push(scene);
    Ok(())
}

fn register_scene_characters(
    project_id: uuid::Uuid,
    scene_id: uuid::Uuid,
    names: &[String],
    characters: &mut Vec<Character>,
    scene_character_refs: &mut Vec<(uuid::Uuid, uuid::Uuid)>,
    character_index: &mut HashMap<String, uuid::Uuid>,
) {
    for name in names {
        let key = name.to_lowercase();
        let character_id = match character_index.get(&key) {
            Some(existing) => *existing,
            None => {
                let character = Character::new(project_id, name.clone(), None, None);
                let id = character.id;
                characters.push(character);
                character_index.insert(key, id);
                id
            }
        };
        scene_character_refs.push((scene_id, character_id));
    }
}

fn register_scene_locations(
    project_id: uuid::Uuid,
    scene_id: uuid::Uuid,
    names: &[String],
    locations: &mut Vec<Location>,
    scene_location_refs: &mut Vec<(uuid::Uuid, uuid::Uuid)>,
    location_index: &mut HashMap<String, uuid::Uuid>,
) {
    for name in names {
        let key = name.to_lowercase();
        let location_id = match location_index.get(&key) {
            Some(existing) => *existing,
            None => {
                let location = Location::new(project_id, name.clone(), None, None);
                let id = location.id;
                locations.push(location);
                location_index.insert(key, id);
                id
            }
        };
        scene_location_refs.push((scene_id, location_id));
    }
}

fn parse_kindling_comment(line: &str) -> Option<HashMap<String, String>> {
    if !line.starts_with("<!--") || !line.ends_with("-->") {
        return None;
    }

    let inner = line
        .trim_start_matches("<!--")
        .trim_end_matches("-->")
        .trim();

    let payload = inner.strip_prefix("kindling:")?.trim();
    if payload.eq_ignore_ascii_case("beats") {
        return None;
    }

    Some(parse_key_values(payload))
}

fn parse_key_values(input: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let mut chars = input.chars().peekable();

    while let Some(next) = chars.peek() {
        if next.is_whitespace() {
            chars.next();
            continue;
        }

        let mut key = String::new();
        while let Some(&ch) = chars.peek() {
            if ch == '=' || ch.is_whitespace() {
                break;
            }
            key.push(ch);
            chars.next();
        }

        while let Some(&ch) = chars.peek() {
            if ch == '=' {
                chars.next();
                break;
            }
            if !ch.is_whitespace() {
                break;
            }
            chars.next();
        }

        if key.is_empty() {
            break;
        }

        let value = if let Some(&'"') = chars.peek() {
            chars.next();
            parse_quoted_value(&mut chars)
        } else {
            let mut raw = String::new();
            while let Some(&ch) = chars.peek() {
                if ch.is_whitespace() {
                    break;
                }
                raw.push(ch);
                chars.next();
            }
            raw
        };

        if !value.is_empty() {
            values.insert(key, value);
        }
    }

    values
}

fn parse_quoted_value<I>(chars: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();
    let mut escaped = false;

    for ch in chars.by_ref() {
        if escaped {
            match ch {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                '\\' => value.push('\\'),
                '"' => value.push('"'),
                other => value.push(other),
            }
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            break;
        }

        value.push(ch);
    }

    value
}

fn split_frontmatter(content: &str) -> Option<(String, String)> {
    let mut lines = content.lines();
    let first = lines.next()?;
    if first.trim() != "---" {
        return None;
    }

    let mut frontmatter_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }

        if in_frontmatter {
            frontmatter_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return None;
    }

    Some((frontmatter_lines.join("\n"), body_lines.join("\n")))
}

fn normalize_scene_folder(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "/" {
        "/".to_string()
    } else {
        trimmed.trim_start_matches('/').to_string()
    }
}

fn resolve_scene_dir(index_dir: &Path, scene_folder: &str) -> PathBuf {
    if scene_folder == "/" || scene_folder == "." || scene_folder == "./" {
        index_dir.to_path_buf()
    } else {
        index_dir.join(scene_folder)
    }
}

fn ensure_markdown_extension(name: &str) -> String {
    if name.to_lowercase().ends_with(".md") {
        name.to_string()
    } else {
        format!("{name}.md")
    }
}

fn build_scene_source_id(index_dir: &Path, scene_path: &Path) -> String {
    let relative = scene_path
        .strip_prefix(index_dir)
        .unwrap_or(scene_path)
        .to_string_lossy()
        .to_string();
    relative.replace('\\', "/")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_parse_longform_index_and_scenes() {
        let dir = tempdir().unwrap();
        let scenes_dir = dir.path().join("Scenes");
        fs::create_dir_all(&scenes_dir).unwrap();

        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: Scenes
  scenes:
    - first scene
    - - second scene
---"#;
        fs::write(&index_path, index_content).unwrap();

        let scene_one = r#"<!-- kindling: scene_type=notes scene_status=final synopsis="First scene" -->
This is some prose.

<!-- kindling: beats -->
- Beat one
  Beat prose line
- Beat two
"#;
        let scene_two = "Just prose.";

        fs::write(scenes_dir.join("first scene.md"), scene_one).unwrap();
        fs::write(scenes_dir.join("second scene.md"), scene_two).unwrap();

        let parsed = parse_longform_index(&index_path).unwrap();

        assert_eq!(parsed.project.name, "Test Project");
        assert_eq!(parsed.project.source_type, SourceType::Longform);
        assert_eq!(parsed.chapters.len(), 1);
        assert_eq!(parsed.scenes.len(), 2);
        assert_eq!(parsed.beats.len(), 2);

        let first_scene = &parsed.scenes[0];
        assert_eq!(first_scene.title, "first scene");
        assert_eq!(first_scene.scene_type, SceneType::Notes);
        assert_eq!(first_scene.scene_status, SceneStatus::Final);
        assert_eq!(first_scene.synopsis.as_deref(), Some("First scene"));
        assert!(first_scene.prose.as_deref().unwrap().contains("some prose"));

        let second_scene = &parsed.scenes[1];
        assert_eq!(second_scene.title, "second scene");
        assert_eq!(second_scene.scene_type, SceneType::Normal);
        assert_eq!(second_scene.scene_status, SceneStatus::Draft);
        assert_eq!(second_scene.prose.as_deref(), Some("Just prose."));
    }

    #[test]
    fn test_parse_scene_list_with_numbers() {
        let yaml = serde_yaml::from_str::<serde_yaml::Value>(
            r#"
- "1"
- 2
        "#,
        )
        .unwrap();

        let entries = parse_scene_entries(&yaml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "1");
        assert_eq!(entries[1].name, "2");
    }

    #[test]
    fn test_parse_scene_frontmatter_status_and_synopsis() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: /
  scenes:
    - frontmatter scene
---"#;
        fs::write(&index_path, index_content).unwrap();

        let scene_content = r#"---
status: revised
synopsis: Frontmatter synopsis
---

Scene prose."#;
        fs::write(dir.path().join("frontmatter scene.md"), scene_content).unwrap();

        let parsed = parse_longform_index(&index_path).unwrap();
        assert_eq!(parsed.scenes.len(), 1);
        assert_eq!(
            parsed.scenes[0].synopsis.as_deref(),
            Some("Frontmatter synopsis")
        );
        assert_eq!(parsed.scenes[0].scene_status, SceneStatus::Revised);
    }

    #[test]
    fn test_parse_scene_frontmatter_references() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: /
  scenes:
    - reference scene
---"#;
        fs::write(&index_path, index_content).unwrap();

        let scene_content = r#"---
characters:
  - "[[Sarah]]"
  - "John"
setting: "[[Downtown Cafe|Cafe]]"
pov: "Sarah"
---

Scene prose."#;
        fs::write(dir.path().join("reference scene.md"), scene_content).unwrap();

        let parsed = parse_longform_index(&index_path).unwrap();
        assert_eq!(parsed.characters.len(), 2);
        assert!(parsed.characters.iter().any(|c| c.name == "Sarah"));
        assert!(parsed.characters.iter().any(|c| c.name == "John"));
        assert_eq!(parsed.locations.len(), 1);
        assert_eq!(parsed.locations[0].name, "Downtown Cafe");
        assert_eq!(parsed.scene_character_refs.len(), 2);
        assert_eq!(parsed.scene_location_refs.len(), 1);
    }

    #[test]
    fn test_ignored_files_patterns() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: /
  scenes:
    - keep scene
    - skip-scratch
  ignoredFiles:
    - "*-scratch"
---"#;
        fs::write(&index_path, index_content).unwrap();

        fs::write(dir.path().join("keep scene.md"), "Content").unwrap();
        fs::write(dir.path().join("skip-scratch.md"), "Content").unwrap();

        let parsed = parse_longform_index(&index_path).unwrap();
        assert_eq!(parsed.scenes.len(), 1);
        assert_eq!(parsed.scenes[0].title, "keep scene");
    }

    #[test]
    fn test_nested_scene_list_creates_chapters() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: /
  scenes:
    - first scene
    - - second scene
      - third scene
    - fourth scene
---"#;
        fs::write(&index_path, index_content).unwrap();

        fs::write(dir.path().join("first scene.md"), "First").unwrap();
        fs::write(dir.path().join("second scene.md"), "Second").unwrap();
        fs::write(dir.path().join("third scene.md"), "Third").unwrap();
        fs::write(dir.path().join("fourth scene.md"), "Fourth").unwrap();

        let parsed = parse_longform_index(&index_path).unwrap();
        assert_eq!(parsed.chapters.len(), 2);
        assert_eq!(parsed.scenes.len(), 4);
        assert_eq!(parsed.chapters[0].title, "first scene");
        assert_eq!(parsed.chapters[1].title, "fourth scene");
    }

    #[test]
    fn test_parse_longform_path_directory() {
        let dir = tempdir().unwrap();
        let index_path = dir.path().join("Project.md");
        let index_content = r#"---
longform:
  format: scenes
  title: Test Project
  workflow: Default Workflow
  sceneFolder: /
  scenes:
    - single scene
---"#;
        fs::write(&index_path, index_content).unwrap();
        fs::write(dir.path().join("single scene.md"), "Scene content").unwrap();

        let parsed = parse_longform_path(dir.path()).unwrap();
        assert_eq!(parsed.project.name, "Test Project");
        assert_eq!(parsed.scenes.len(), 1);
    }
}
