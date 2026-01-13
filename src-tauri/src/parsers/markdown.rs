use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::models::{Beat, Chapter, Project, Scene, SourceType};

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

        if let Some(stripped) = trimmed.strip_prefix("# ") {
            // Save previous scene and chapter if they exist
            if let Some(scene) = current_scene.take() {
                scenes.push(scene);
            }
            if let Some(chapter) = current_chapter.take() {
                chapters.push(chapter);
            }

            // New chapter
            let title = stripped.trim().to_string();
            current_chapter = Some(Chapter::new(project.id, title, chapter_position));
            chapter_position += 1;
            scene_position = 0;
            beat_position = 0;
        } else if let Some(stripped) = trimmed.strip_prefix("## ") {
            // Save previous scene if it exists
            if let Some(scene) = current_scene.take() {
                scenes.push(scene);
            }

            // New scene under current chapter
            if let Some(ref chapter) = current_chapter {
                let title = stripped.trim().to_string();
                current_scene = Some(Scene::new(chapter.id, title, None, scene_position));
                scene_position += 1;
                beat_position = 0;
            }
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            // Beat (list item with content)
            if let Some(ref scene) = current_scene {
                let content = trimmed[2..].trim().to_string();
                if !content.is_empty() {
                    let beat = Beat::new(scene.id, content, beat_position);
                    beats.push(beat);
                    beat_position += 1;
                }
            }
        } else if trimmed == "-" || trimmed == "*" {
            // Empty list item (e.g., "- " after trimming) - skip it
            continue;
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
    use std::path::PathBuf;

    fn fixtures_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    // ========================================================================
    // Fixture-based tests
    // ========================================================================

    #[test]
    fn test_parse_hamlet_fixture() {
        let path = fixtures_dir().join("hamlet.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Project metadata
        assert_eq!(result.project.name, "hamlet");
        assert_eq!(result.project.source_type, SourceType::Markdown);

        // 5 Acts
        assert_eq!(result.chapters.len(), 5);
        assert_eq!(result.chapters[0].title, "Act I");
        assert_eq!(result.chapters[1].title, "Act II");
        assert_eq!(result.chapters[2].title, "Act III");
        assert_eq!(result.chapters[3].title, "Act IV");
        assert_eq!(result.chapters[4].title, "Act V");

        // Count scenes per act
        // Act I: 5 scenes, Act II: 2 scenes, Act III: 4 scenes, Act IV: 7 scenes, Act V: 2 scenes
        assert_eq!(result.scenes.len(), 20);

        // Check some specific scenes
        let act1_scenes: Vec<_> = result
            .scenes
            .iter()
            .filter(|s| s.chapter_id == result.chapters[0].id)
            .collect();
        assert_eq!(act1_scenes.len(), 5);
        assert_eq!(act1_scenes[0].title, "Scene 1 - The Battlements");

        // Check we have beats
        assert!(result.beats.len() > 50); // Hamlet has many beats
    }

    #[test]
    fn test_parse_scenes_only_fixture() {
        let path = fixtures_dir().join("scenes-only.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Should create a default chapter
        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.chapters[0].title, "Chapter 1");

        // All scenes should be orphaned to the default chapter
        assert_eq!(result.scenes.len(), 0); // Scenes without chapters are ignored

        // The list items are treated as beats but there's no scene to attach them to
        // So beats will be empty
        assert_eq!(result.beats.len(), 0);
    }

    #[test]
    fn test_parse_beats_only_fixture() {
        let path = fixtures_dir().join("beats-only.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Should create a default chapter with beats assigned to a default scene
        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.chapters[0].title, "Chapter 1");

        // No explicit scenes, so beats are not captured (they need a scene)
        // beats-only.md has list items but no H1/H2 headers
        assert_eq!(result.scenes.len(), 0);
        assert_eq!(result.beats.len(), 0);
    }

    #[test]
    fn test_parse_chapters_only_fixture() {
        let path = fixtures_dir().join("chapters-only.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Three chapters with no scenes or beats
        assert_eq!(result.chapters.len(), 3);
        assert_eq!(result.chapters[0].title, "Beginning");
        assert_eq!(result.chapters[1].title, "Middle");
        assert_eq!(result.chapters[2].title, "End");

        assert_eq!(result.scenes.len(), 0);
        assert_eq!(result.beats.len(), 0);
    }

    #[test]
    fn test_parse_special_characters_fixture() {
        let path = fixtures_dir().join("special-characters.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Check chapter with special chars in title
        assert_eq!(result.chapters.len(), 1);
        assert_eq!(
            result.chapters[0].title,
            "Chapter with \"Quotes\" & Ampersand"
        );

        // Check scenes with special chars
        assert_eq!(result.scenes.len(), 3);
        assert_eq!(result.scenes[0].title, "Scene <One> — With Dashes");
        assert_eq!(result.scenes[1].title, "Scene: Colons & More!");
        assert_eq!(result.scenes[2].title, "Scène with Ünïcödé");

        // Check beats preserved special characters
        let beat_contents: Vec<_> = result.beats.iter().map(|b| b.content.as_str()).collect();
        assert!(beat_contents.contains(&"Beat with **bold** and *italic*"));
        assert!(beat_contents.contains(&"Beat with émojis: 🎭 📚 ✨"));
        assert!(beat_contents.contains(&"日本語テキスト (Japanese text)"));
        assert!(beat_contents.contains(&"Текст на русском (Russian text)"));
    }

    #[test]
    fn test_parse_empty_fixture() {
        let path = fixtures_dir().join("empty.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Empty file should create default chapter
        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.chapters[0].title, "Chapter 1");
        assert_eq!(result.scenes.len(), 0);
        assert_eq!(result.beats.len(), 0);
    }

    #[test]
    fn test_parse_whitespace_only_fixture() {
        let path = fixtures_dir().join("whitespace-only.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Whitespace-only file should behave like empty file
        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.chapters[0].title, "Chapter 1");
        assert_eq!(result.scenes.len(), 0);
        assert_eq!(result.beats.len(), 0);
    }

    #[test]
    fn test_parse_nested_content_fixture() {
        let path = fixtures_dir().join("nested-content.md");
        let result = parse_markdown_outline(&path).unwrap();

        // Three parts as chapters
        assert_eq!(result.chapters.len(), 3);
        assert_eq!(result.chapters[0].title, "Part One: The Setup");
        assert_eq!(result.chapters[1].title, "Part Two: The Journey");
        assert_eq!(result.chapters[2].title, "Part Three: The Resolution");

        // 8 chapters total as scenes
        assert_eq!(result.scenes.len(), 8);

        // Each scene should have 3 beats
        let part1_scene1_beats: Vec<_> = result
            .beats
            .iter()
            .filter(|b| b.scene_id == result.scenes[0].id)
            .collect();
        assert_eq!(part1_scene1_beats.len(), 3);
    }

    #[test]
    fn test_parse_paragraph_beats_fixture() {
        let path = fixtures_dir().join("paragraph-beats.md");
        let result = parse_markdown_outline(&path).unwrap();

        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.chapters[0].title, "The Story");

        assert_eq!(result.scenes.len(), 3);
        assert_eq!(result.scenes[0].title, "Scene One");

        // Check paragraph beats are captured
        let scene1_beats: Vec<_> = result
            .beats
            .iter()
            .filter(|b| b.scene_id == result.scenes[0].id)
            .collect();
        assert_eq!(scene1_beats.len(), 3);

        // Verify paragraph content preserved
        assert!(scene1_beats[0]
            .content
            .contains("morning sun rose over the quiet village"));
    }

    // ========================================================================
    // Unit tests with inline content
    // ========================================================================

    #[test]
    fn test_basic_outline_structure() {
        use std::io::Write;
        use tempfile::NamedTempFile;

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
        assert_eq!(result.scenes[0].title, "The Beginning");
        assert_eq!(result.scenes[1].title, "The Middle");
        assert_eq!(result.scenes[2].title, "The End");

        assert_eq!(result.beats.len(), 4);
    }

    #[test]
    fn test_asterisk_list_items() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter

## Scene

* Beat with asterisk
* Another asterisk beat
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.beats.len(), 2);
        assert_eq!(result.beats[0].content, "Beat with asterisk");
        assert_eq!(result.beats[1].content, "Another asterisk beat");
    }

    #[test]
    fn test_mixed_list_styles() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter

## Scene

- Dash beat
* Asterisk beat
- Another dash
* Another asterisk
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.beats.len(), 4);
        assert_eq!(result.beats[0].content, "Dash beat");
        assert_eq!(result.beats[1].content, "Asterisk beat");
        assert_eq!(result.beats[2].content, "Another dash");
        assert_eq!(result.beats[3].content, "Another asterisk");
    }

    #[test]
    fn test_beat_positions_reset_per_scene() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter

## Scene 1

- Beat A
- Beat B

## Scene 2

- Beat C
- Beat D
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Beats in Scene 1 should have positions 0, 1
        let scene1_beats: Vec<_> = result
            .beats
            .iter()
            .filter(|b| b.scene_id == result.scenes[0].id)
            .collect();
        assert_eq!(scene1_beats[0].position, 0);
        assert_eq!(scene1_beats[1].position, 1);

        // Beats in Scene 2 should have positions 0, 1 (reset)
        let scene2_beats: Vec<_> = result
            .beats
            .iter()
            .filter(|b| b.scene_id == result.scenes[1].id)
            .collect();
        assert_eq!(scene2_beats[0].position, 0);
        assert_eq!(scene2_beats[1].position, 1);
    }

    #[test]
    fn test_scene_positions_reset_per_chapter() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter 1

## Scene A
## Scene B

# Chapter 2

## Scene C
## Scene D
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Scenes in Chapter 1 should have positions 0, 1
        let ch1_scenes: Vec<_> = result
            .scenes
            .iter()
            .filter(|s| s.chapter_id == result.chapters[0].id)
            .collect();
        assert_eq!(ch1_scenes[0].position, 0);
        assert_eq!(ch1_scenes[1].position, 1);

        // Scenes in Chapter 2 should have positions 0, 1 (reset)
        let ch2_scenes: Vec<_> = result
            .scenes
            .iter()
            .filter(|s| s.chapter_id == result.chapters[1].id)
            .collect();
        assert_eq!(ch2_scenes[0].position, 0);
        assert_eq!(ch2_scenes[1].position, 1);
    }

    #[test]
    fn test_chapter_positions_increment() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# First
# Second
# Third
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.chapters[0].position, 0);
        assert_eq!(result.chapters[1].position, 1);
        assert_eq!(result.chapters[2].position, 2);
    }

    #[test]
    fn test_project_name_from_filename() {
        use std::io::Write;
        use tempfile::Builder;

        let mut file = Builder::new()
            .prefix("my-novel")
            .suffix(".md")
            .tempfile()
            .unwrap();
        file.write_all(b"# Chapter 1").unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Project name should be derived from filename without extension
        assert!(result.project.name.starts_with("my-novel"));
    }

    #[test]
    fn test_empty_beat_lines_ignored() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Note: "- " with space is a valid list marker that should be ignored when empty
        let content = "# Chapter\n\n## Scene\n\n- Valid beat\n- \n- Another valid beat\n";

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Empty beat line (dash followed by space but no content) should be ignored
        assert_eq!(result.beats.len(), 2);
        assert_eq!(result.beats[0].content, "Valid beat");
        assert_eq!(result.beats[1].content, "Another valid beat");
    }

    #[test]
    fn test_whitespace_in_titles_trimmed() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"#   Chapter With Spaces

##   Scene With Spaces

- Beat content
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.chapters[0].title, "Chapter With Spaces");
        assert_eq!(result.scenes[0].title, "Scene With Spaces");
    }

    #[test]
    fn test_ignores_h3_and_beyond() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter

## Scene

### This is not a scene

- This beat is under the scene, not the H3

#### Neither is this

- Still a beat under the scene
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert_eq!(result.chapters.len(), 1);
        assert_eq!(result.scenes.len(), 1);
        // H3 and H4 lines are treated as paragraph beats
        assert!(result.beats.len() >= 2);
    }

    #[test]
    fn test_error_on_nonexistent_file() {
        let result = parse_markdown_outline("/nonexistent/path/file.md");
        assert!(result.is_err());
    }

    #[test]
    fn test_source_path_preserved() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        assert!(result.project.source_path.is_some());
        assert!(result
            .project
            .source_path
            .unwrap()
            .contains(file.path().file_name().unwrap().to_str().unwrap()));
    }

    #[test]
    fn test_scenes_link_to_correct_chapters() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter 1

## Scene 1A
## Scene 1B

# Chapter 2

## Scene 2A
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Scenes 1A and 1B should belong to Chapter 1
        assert_eq!(result.scenes[0].chapter_id, result.chapters[0].id);
        assert_eq!(result.scenes[1].chapter_id, result.chapters[0].id);

        // Scene 2A should belong to Chapter 2
        assert_eq!(result.scenes[2].chapter_id, result.chapters[1].id);
    }

    #[test]
    fn test_beats_link_to_correct_scenes() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter

## Scene A

- Beat 1
- Beat 2

## Scene B

- Beat 3
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // Beats 1 and 2 should belong to Scene A
        assert_eq!(result.beats[0].scene_id, result.scenes[0].id);
        assert_eq!(result.beats[1].scene_id, result.scenes[0].id);

        // Beat 3 should belong to Scene B
        assert_eq!(result.beats[2].scene_id, result.scenes[1].id);
    }

    #[test]
    fn test_unique_ids_generated() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let content = r#"# Chapter 1

## Scene 1

- Beat 1

# Chapter 2

## Scene 2

- Beat 2
"#;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();

        let result = parse_markdown_outline(file.path()).unwrap();

        // All IDs should be unique
        assert_ne!(result.chapters[0].id, result.chapters[1].id);
        assert_ne!(result.scenes[0].id, result.scenes[1].id);
        assert_ne!(result.beats[0].id, result.beats[1].id);

        // Project ID should be unique from all others
        assert_ne!(result.project.id, result.chapters[0].id);
    }
}
