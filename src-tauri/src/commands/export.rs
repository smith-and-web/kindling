//! Export Command Handlers
//!
//! Commands for exporting projects to various formats (Markdown, DOCX).

use crate::commands::AppState;
use crate::db;
use crate::models::{Beat, Scene};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

/// Export scope - what to export
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportScope {
    /// Export entire project
    Project,
    /// Export single chapter by ID
    Chapter(String),
    /// Export single scene by ID
    Scene(String),
}

/// Export options for markdown export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownExportOptions {
    /// What to export (project, chapter, or scene)
    pub scope: ExportScope,
    /// Include beat markers (### Beat: content) in output
    pub include_beat_markers: bool,
    /// Output directory path
    pub output_path: String,
}

/// Result of export operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    /// Path where export was saved
    pub output_path: String,
    /// Number of files created
    pub files_created: usize,
    /// Total chapters exported
    pub chapters_exported: usize,
    /// Total scenes exported
    pub scenes_exported: usize,
}

/// Sanitize a filename by removing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Strip HTML tags from content (for prose that may contain HTML from TipTap)
fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag_name = String::new();
    let mut reading_tag_name = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                reading_tag_name = true;
                tag_name.clear();
            }
            '>' => {
                in_tag = false;
                reading_tag_name = false;
                // Add paragraph breaks after block-level closing tags
                let tag_lower = tag_name.to_lowercase();
                if (tag_lower == "/p" || tag_lower == "br" || tag_lower == "br/")
                    && !result.ends_with('\n')
                    && !result.is_empty()
                {
                    result.push_str("\n\n");
                }
                tag_name.clear();
            }
            ' ' | '/' if reading_tag_name && !tag_name.is_empty() => {
                reading_tag_name = false;
            }
            _ if in_tag && reading_tag_name => {
                tag_name.push(c);
            }
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    // Clean up multiple newlines and trim
    let mut cleaned = String::new();
    let mut prev_was_newline = false;
    for c in result.chars() {
        if c == '\n' {
            if !prev_was_newline {
                cleaned.push('\n');
                prev_was_newline = true;
            }
        } else {
            cleaned.push(c);
            prev_was_newline = false;
        }
    }

    // Join paragraphs with double newlines
    cleaned
        .split('\n')
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generate markdown content for a scene
fn generate_scene_markdown(scene: &Scene, beats: &[Beat], include_beat_markers: bool) -> String {
    let mut content = String::new();

    // Scene title as H1
    content.push_str(&format!("# {}\n\n", scene.title));

    // Synopsis as blockquote if present
    if let Some(ref synopsis) = scene.synopsis {
        if !synopsis.trim().is_empty() {
            content.push_str("> ");
            content.push_str(&synopsis.replace('\n', "\n> "));
            content.push_str("\n\n");
        }
    }

    // Beats
    for beat in beats {
        if include_beat_markers {
            content.push_str(&format!("## {}\n\n", beat.content));
        }

        // Beat prose
        if let Some(ref prose) = beat.prose {
            let clean_prose = strip_html(prose);
            if !clean_prose.is_empty() {
                content.push_str(&clean_prose);
                content.push_str("\n\n");
            }
        }
    }

    content
}

/// Export project to markdown files
///
/// Creates a folder structure: `ProjectName/ChapterName/SceneName.md`
#[tauri::command]
pub async fn export_to_markdown(
    project_id: String,
    options: MarkdownExportOptions,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get project info
    let project = db::queries::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project not found: {}", project_id))?;

    let output_base = PathBuf::from(&options.output_path);

    // Create project folder
    let project_folder = output_base.join(sanitize_filename(&project.name));
    fs::create_dir_all(&project_folder)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    let mut files_created = 0;
    let mut chapters_exported = 0;
    let mut scenes_exported = 0;

    match options.scope {
        ExportScope::Project => {
            // Get all chapters
            let chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;

            for chapter in &chapters {
                if chapter.archived {
                    continue;
                }

                let chapter_folder = project_folder.join(sanitize_filename(&chapter.title));
                fs::create_dir_all(&chapter_folder)
                    .map_err(|e| format!("Failed to create chapter directory: {}", e))?;

                // Get scenes for this chapter
                let scenes =
                    db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;

                for scene in &scenes {
                    if scene.archived {
                        continue;
                    }

                    let beats =
                        db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

                    let markdown =
                        generate_scene_markdown(scene, &beats, options.include_beat_markers);
                    let scene_file =
                        chapter_folder.join(format!("{}.md", sanitize_filename(&scene.title)));

                    fs::write(&scene_file, markdown)
                        .map_err(|e| format!("Failed to write scene file: {}", e))?;

                    files_created += 1;
                    scenes_exported += 1;
                }

                chapters_exported += 1;
            }
        }
        ExportScope::Chapter(chapter_id) => {
            let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;

            // Get chapter info
            let chapter = db::queries::get_chapter_by_id(&conn, &chapter_uuid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Chapter not found: {}", chapter_id))?;

            let chapter_folder = project_folder.join(sanitize_filename(&chapter.title));
            fs::create_dir_all(&chapter_folder)
                .map_err(|e| format!("Failed to create chapter directory: {}", e))?;

            // Get scenes for this chapter
            let scenes = db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;

            for scene in &scenes {
                if scene.archived {
                    continue;
                }

                let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

                let markdown = generate_scene_markdown(scene, &beats, options.include_beat_markers);
                let scene_file =
                    chapter_folder.join(format!("{}.md", sanitize_filename(&scene.title)));

                fs::write(&scene_file, markdown)
                    .map_err(|e| format!("Failed to write scene file: {}", e))?;

                files_created += 1;
                scenes_exported += 1;
            }

            chapters_exported = 1;
        }
        ExportScope::Scene(scene_id) => {
            let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;

            // Get scene info
            let scene = db::queries::get_scene_by_id(&conn, &scene_uuid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Scene not found: {}", scene_id))?;

            let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

            let markdown = generate_scene_markdown(&scene, &beats, options.include_beat_markers);
            let scene_file = project_folder.join(format!("{}.md", sanitize_filename(&scene.title)));

            fs::write(&scene_file, markdown)
                .map_err(|e| format!("Failed to write scene file: {}", e))?;

            files_created = 1;
            scenes_exported = 1;
        }
    }

    Ok(ExportResult {
        output_path: project_folder.to_string_lossy().to_string(),
        files_created,
        chapters_exported,
        scenes_exported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Normal Name"), "Normal Name");
        assert_eq!(sanitize_filename("Has/Slash"), "Has_Slash");
        assert_eq!(sanitize_filename("Has\\Backslash"), "Has_Backslash");
        assert_eq!(sanitize_filename("Has:Colon"), "Has_Colon");
        assert_eq!(sanitize_filename("Has*Star"), "Has_Star");
        assert_eq!(sanitize_filename("Has?Question"), "Has_Question");
        assert_eq!(sanitize_filename("Has\"Quote"), "Has_Quote");
        assert_eq!(sanitize_filename("Has<Less"), "Has_Less");
        assert_eq!(sanitize_filename("Has>Greater"), "Has_Greater");
        assert_eq!(sanitize_filename("Has|Pipe"), "Has_Pipe");
        assert_eq!(sanitize_filename("  Trimmed  "), "Trimmed");
        assert_eq!(
            sanitize_filename("Multiple///Slashes"),
            "Multiple___Slashes"
        );
    }

    #[test]
    fn test_strip_html_simple() {
        assert_eq!(strip_html("<p>Hello</p>"), "Hello");
        assert_eq!(strip_html("<p>Hello</p><p>World</p>"), "Hello\n\nWorld");
        assert_eq!(strip_html("<strong>Bold</strong>"), "Bold");
        assert_eq!(strip_html("<em>Italic</em>"), "Italic");
    }

    #[test]
    fn test_strip_html_nested() {
        assert_eq!(strip_html("<p><strong>Bold</strong> text</p>"), "Bold text");
        assert_eq!(
            strip_html("<p>First paragraph</p><p>Second paragraph</p>"),
            "First paragraph\n\nSecond paragraph"
        );
    }

    #[test]
    fn test_strip_html_plain_text() {
        assert_eq!(strip_html("Plain text"), "Plain text");
    }
}
