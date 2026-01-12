use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::models::{Project, Chapter, Scene, Beat, SourceType};

#[derive(Debug, Error)]
pub enum MarkdownError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid markdown structure")]
    InvalidStructure,
}

// ============================================================================
// Parsed Output
// ============================================================================

pub struct ParsedMarkdown {
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub beats: Vec<Beat>,
}

// ============================================================================
// Parser Implementation
// ============================================================================

/// Parse a markdown outline file.
///
/// Expected format:
/// ```markdown
/// # Chapter Title
///
/// ## Scene Title
///
/// - Beat 1 content
/// - Beat 2 content
///
/// ## Another Scene
///
/// - More beat content
///
/// # Another Chapter
/// ...
/// ```
///
/// Alternatively supports:
/// - H1 as chapter
/// - H2 as scene
/// - List items or paragraphs under H2 as beats
pub fn parse_markdown_outline<P: AsRef<Path>>(path: P) -> Result<ParsedMarkdown, MarkdownError> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)?;

    let project_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    let project = Project::new(
        project_name,
        SourceType::Markdown,
        Some(path.to_string_lossy().to_string()),
    );

    let mut chapters: Vec<Chapter> = Vec::new();
    let mut scenes: Vec<Scene> = Vec::new();
    let mut beats: Vec<Beat> = Vec::new();

    let mut current_chapter: Option<Chapter> = None;
    let mut current_scene: Option<Scene> = None;
    let mut chapter_position = 0;
    let mut scene_position = 0;
    let mut beat_position = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            // Save previous scene and chapter if they exist
            if let Some(scene) = current_scene.take() {
                scenes.push(scene);
            }
            if let Some(chapter) = current_chapter.take() {
                chapters.push(chapter);
            }

            // New chapter
            let title = trimmed[2..].trim().to_string();
            current_chapter = Some(Chapter::new(project.id, title, chapter_position));
            chapter_position += 1;
            scene_position = 0;
            beat_position = 0;

        } else if trimmed.starts_with("## ") {
            // Save previous scene if it exists
            if let Some(scene) = current_scene.take() {
                scenes.push(scene);
            }

            // New scene under current chapter
            if let Some(ref chapter) = current_chapter {
                let title = trimmed[3..].trim().to_string();
                current_scene = Some(Scene::new(chapter.id, title, None, scene_position));
                scene_position += 1;
                beat_position = 0;
            }

        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // Beat (list item)
            if let Some(ref scene) = current_scene {
                let content = trimmed[2..].trim().to_string();
                if !content.is_empty() {
                    let beat = Beat::new(scene.id, content, beat_position);
                    beats.push(beat);
                    beat_position += 1;
                }
            }

        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            // Regular paragraph under a scene becomes a beat
            if let Some(ref scene) = current_scene {
                let beat = Beat::new(scene.id, trimmed.to_string(), beat_position);
                beats.push(beat);
                beat_position += 1;
            }
        }
    }

    // Don't forget the last scene and chapter
    if let Some(scene) = current_scene {
        scenes.push(scene);
    }
    if let Some(chapter) = current_chapter {
        chapters.push(chapter);
    }

    // If no chapters were found, create a default one
    if chapters.is_empty() {
        let default_chapter = Chapter::new(project.id, "Chapter 1".to_string(), 0);

        // If we have any orphan beats, create a scene for them
        if !beats.is_empty() {
            let default_scene = Scene::new(default_chapter.id, "Scene 1".to_string(), None, 0);

            // Re-assign all beats to this scene
            for beat in &mut beats {
                beat.scene_id = default_scene.id;
            }

            scenes.push(default_scene);
        }

        chapters.push(default_chapter);
    }

    Ok(ParsedMarkdown {
        project,
        chapters,
        scenes,
        beats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_outline() {
        let content = r#"# Chapter One

## The Beginning

- Hero wakes up
- Discovers the mystery

## The Middle

- Confronts the villain

# Chapter Two

## The End

- Resolution
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.chapters.len(), 2);
        assert_eq!(result.chapters[0].title, "Chapter One");
        assert_eq!(result.chapters[1].title, "Chapter Two");

        assert_eq!(result.scenes.len(), 3);
        assert_eq!(result.beats.len(), 4);
    }
}
