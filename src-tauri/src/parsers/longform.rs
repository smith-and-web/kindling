use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::models::{Beat, Chapter, Project, Scene, SceneStatus, SceneType, SourceType};

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

    let chapter = Chapter::new(project.id, "Chapter 1".to_string(), 0)
        .with_source_id(Some(LONGFORM_DEFAULT_CHAPTER_SOURCE_ID.to_string()));

    let index_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let scene_dir = resolve_scene_dir(index_dir, &scene_folder);

    let mut scenes = Vec::new();
    let mut beats = Vec::new();

    for (scene_position, entry) in scene_entries.into_iter().enumerate() {
        let scene_file_name = ensure_markdown_extension(&entry.name);
        let scene_path = scene_dir.join(&scene_file_name);
        let scene_source_id = build_scene_source_id(index_dir, &scene_path);

        let scene_content = parse_scene_file(&scene_path)?;

        let mut scene = Scene::new(
            chapter.id,
            entry.name,
            scene_content.synopsis,
            scene_position as i32,
        )
        .with_source_id(Some(scene_source_id));
        scene.prose = scene_content.prose;
        scene.scene_type = scene_content.scene_type;
        scene.scene_status = scene_content.scene_status;

        for (beat_position, beat) in scene_content.beats.into_iter().enumerate() {
            let mut new_beat = Beat::new(scene.id, beat.content, beat_position as i32);
            new_beat.prose = beat.prose;
            beats.push(new_beat);
        }

        scenes.push(scene);
    }

    Ok(ParsedLongform {
        project,
        chapters: vec![chapter],
        scenes,
        beats,
    })
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
    Ok(parse_scene_body(&content))
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
}
