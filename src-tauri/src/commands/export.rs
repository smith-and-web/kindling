//! Export Command Handlers
//!
//! Commands for exporting projects to various formats (Markdown, DOCX).

use crate::commands::{load_app_settings, AppState};
use crate::db;
use crate::models::{AppSettings, Beat, Chapter, Project, Scene, SnapshotTrigger};
use docx_rs::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, State};
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

/// Chapter heading style for DOCX export
///
/// Standard Manuscript Format supports various chapter heading styles.
/// All styles produce centered, ALL CAPS headings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChapterHeadingStyle {
    /// "CHAPTER ONE" - word number only (default SMF style)
    #[default]
    NumberOnly,
    /// "CHAPTER ONE: THE BEGINNING" - word number with chapter title
    NumberAndTitle,
    /// "THE BEGINNING" - title only, no chapter number
    TitleOnly,
    /// "CHAPTER 1" - Arabic numeral instead of word
    NumberArabic,
    /// "CHAPTER 1: THE BEGINNING" - Arabic numeral with title
    NumberArabicAndTitle,
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
    /// Delete existing export folder if it exists
    #[serde(default)]
    pub delete_existing: bool,
    /// Custom name for the export folder (defaults to project name if not provided)
    #[serde(default)]
    pub export_name: Option<String>,
    /// Create a snapshot before exporting
    #[serde(default)]
    pub create_snapshot: bool,
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

/// Export options for DOCX export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxExportOptions {
    /// What to export (project, chapter, or scene)
    pub scope: ExportScope,
    /// Include beat markers as Heading 3 in output
    pub include_beat_markers: bool,
    /// Include scene synopsis as italicized paragraph
    pub include_synopsis: bool,
    /// Output file path (full path including filename)
    pub output_path: String,
    /// Create a snapshot before exporting
    #[serde(default)]
    pub create_snapshot: bool,
    /// Add page breaks between chapters
    #[serde(default = "default_page_breaks")]
    pub page_breaks_between_chapters: bool,
    /// Include a Standard Manuscript Format title page
    #[serde(default = "default_title_page")]
    pub include_title_page: bool,
    /// Chapter heading style (how chapter headings are formatted)
    #[serde(default)]
    pub chapter_heading_style: ChapterHeadingStyle,
}

fn default_page_breaks() -> bool {
    true
}

fn default_title_page() -> bool {
    true
}

/// Extract surname from a full name
///
/// Assumes the last word in the name is the surname.
/// Examples:
/// - "John Smith" -> "Smith"
/// - "Mary Jane Watson" -> "Watson"
/// - "Prince" -> "Prince" (single name)
fn extract_surname(full_name: &str) -> String {
    full_name
        .split_whitespace()
        .last()
        .unwrap_or(full_name)
        .to_string()
}

/// Abbreviate a title for the running header
///
/// If the title is longer than max_words, truncate to max_words.
/// The title is converted to uppercase as per SMF.
fn abbreviate_title(title: &str, max_words: usize) -> String {
    let words: Vec<&str> = title.split_whitespace().collect();
    if words.len() <= max_words {
        title.to_uppercase()
    } else {
        words[..max_words].join(" ").to_uppercase()
    }
}

/// Convert a chapter number to its word form (uppercase)
///
/// Standard Manuscript Format typically uses word numbers for chapters.
/// Supports chapters 1-100, falls back to Arabic numerals for higher numbers.
fn number_to_word(n: usize) -> String {
    const ONES: [&str; 20] = [
        "",
        "ONE",
        "TWO",
        "THREE",
        "FOUR",
        "FIVE",
        "SIX",
        "SEVEN",
        "EIGHT",
        "NINE",
        "TEN",
        "ELEVEN",
        "TWELVE",
        "THIRTEEN",
        "FOURTEEN",
        "FIFTEEN",
        "SIXTEEN",
        "SEVENTEEN",
        "EIGHTEEN",
        "NINETEEN",
    ];
    const TENS: [&str; 10] = [
        "", "", "TWENTY", "THIRTY", "FORTY", "FIFTY", "SIXTY", "SEVENTY", "EIGHTY", "NINETY",
    ];

    match n {
        0 => "ZERO".to_string(),
        1..=19 => ONES[n].to_string(),
        20..=99 => {
            let tens_digit = n / 10;
            let ones_digit = n % 10;
            if ones_digit == 0 {
                TENS[tens_digit].to_string()
            } else {
                format!("{}-{}", TENS[tens_digit], ONES[ones_digit])
            }
        }
        100 => "ONE HUNDRED".to_string(),
        _ => n.to_string(), // Fall back to Arabic for large numbers
    }
}

/// Format a chapter heading based on the selected style
///
/// Returns the formatted chapter heading string in ALL CAPS.
fn format_chapter_heading(
    chapter_number: usize,
    chapter_title: &str,
    style: &ChapterHeadingStyle,
) -> String {
    match style {
        ChapterHeadingStyle::NumberOnly => {
            format!("CHAPTER {}", number_to_word(chapter_number))
        }
        ChapterHeadingStyle::NumberAndTitle => {
            format!(
                "CHAPTER {}: {}",
                number_to_word(chapter_number),
                chapter_title.to_uppercase()
            )
        }
        ChapterHeadingStyle::TitleOnly => chapter_title.to_uppercase(),
        ChapterHeadingStyle::NumberArabic => {
            format!("CHAPTER {}", chapter_number)
        }
        ChapterHeadingStyle::NumberArabicAndTitle => {
            format!(
                "CHAPTER {}: {}",
                chapter_number,
                chapter_title.to_uppercase()
            )
        }
    }
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

/// Count words in text (simple whitespace split)
fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Calculate total word count from all beats in the project
fn calculate_project_word_count(
    conn: &rusqlite::Connection,
    project_uuid: &Uuid,
) -> Result<usize, String> {
    let chapters = db::queries::get_chapters(conn, project_uuid).map_err(|e| e.to_string())?;

    let mut total_words = 0;

    for chapter in chapters.iter().filter(|c| !c.archived) {
        let scenes = db::queries::get_scenes(conn, &chapter.id).map_err(|e| e.to_string())?;

        for scene in scenes.iter().filter(|s| !s.archived) {
            let beats = db::queries::get_beats(conn, &scene.id).map_err(|e| e.to_string())?;

            for beat in &beats {
                if let Some(ref prose) = beat.prose {
                    let clean_prose = strip_html(prose);
                    total_words += count_words(&clean_prose);
                }
            }
        }
    }

    Ok(total_words)
}

/// Round word count to nearest thousand for manuscript format
fn round_word_count(count: usize) -> String {
    if count < 1000 {
        format!("{} words", count)
    } else {
        // Round to nearest thousand
        let thousands = ((count + 500) / 1000) * 1000;
        format!("approx. {} words", thousands)
    }
}

/// Generate a Standard Manuscript Format title page
///
/// Layout (top to bottom):
/// - Contact info (top left): Name, Address, Phone, Email
/// - Word count (top right): "approx. XX,XXX words"
/// - Title (centered, middle): PROJECT TITLE
/// - Byline (centered, below title): "by" + Author Name or Pen Name
/// - Genre (centered, below byline, optional)
fn add_title_page(
    docx: Docx,
    project: &Project,
    app_settings: &AppSettings,
    word_count: usize,
) -> Docx {
    let mut docx = docx;

    // Get author name: use pen name if set, otherwise fall back to app settings author name
    let author_name = project
        .author_pen_name
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .or(app_settings.author_name.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Contact name (always use legal name from app settings, not pen name)
    let contact_name = app_settings
        .author_name
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Build contact info lines
    let mut contact_lines: Vec<String> = Vec::new();
    if !contact_name.is_empty() {
        contact_lines.push(contact_name);
    }
    if let Some(ref addr1) = app_settings.contact_address_line1 {
        if !addr1.trim().is_empty() {
            contact_lines.push(addr1.clone());
        }
    }
    if let Some(ref addr2) = app_settings.contact_address_line2 {
        if !addr2.trim().is_empty() {
            contact_lines.push(addr2.clone());
        }
    }
    if let Some(ref phone) = app_settings.contact_phone {
        if !phone.trim().is_empty() {
            contact_lines.push(phone.clone());
        }
    }
    if let Some(ref email) = app_settings.contact_email {
        if !email.trim().is_empty() {
            contact_lines.push(email.clone());
        }
    }

    // Word count string
    let word_count_str = round_word_count(word_count);

    // Add contact info at top left (each line as separate paragraph)
    // SMF title page uses same Courier New font as body
    for line in &contact_lines {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(line)
                        .size(24) // 12pt
                        .fonts(RunFonts::new().ascii("Courier New")),
                )
                .align(AlignmentType::Left)
                .line_spacing(LineSpacing::new().line(240)), // Single spacing for contact info
        );
    }

    // Add word count aligned right (on same conceptual "row" but we'll add it after contact)
    // In SMF, word count typically goes top right. We'll add blank lines then a right-aligned paragraph.
    // Since we can't easily do two-column layout, we'll put word count on its own line after contact
    if !contact_lines.is_empty() {
        docx = docx.add_paragraph(Paragraph::new()); // Blank line
    }
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&word_count_str)
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .align(AlignmentType::Right),
    );

    // Add vertical space to push title toward center (approximately 1/3 down the page)
    for _ in 0..12 {
        docx = docx.add_paragraph(Paragraph::new());
    }

    // Title (centered, uppercase)
    // SMF: Title is typically uppercase, same size as body text
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(project.name.to_uppercase())
                    .size(24) // 12pt - same as body for SMF
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .align(AlignmentType::Center),
    );

    // Blank line
    docx = docx.add_paragraph(Paragraph::new());

    // "by" line
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text("by")
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .align(AlignmentType::Center),
    );

    // Blank line
    docx = docx.add_paragraph(Paragraph::new());

    // Author name (pen name or real name)
    if !author_name.is_empty() {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(&author_name)
                        .size(24)
                        .fonts(RunFonts::new().ascii("Courier New")),
                )
                .align(AlignmentType::Center),
        );
    }

    // Genre (optional, below author name)
    if let Some(ref genre) = project.genre {
        if !genre.trim().is_empty() {
            docx = docx.add_paragraph(Paragraph::new()); // Blank line
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text(genre)
                            .size(24)
                            .italic()
                            .fonts(RunFonts::new().ascii("Courier New")),
                    )
                    .align(AlignmentType::Center),
            );
        }
    }

    // Page break after title page
    docx = docx.add_paragraph(Paragraph::new().page_break_before(true));

    docx
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
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;

    // Create snapshot if requested (before taking the connection lock)
    if options.create_snapshot {
        let snapshot_name = options
            .export_name
            .as_ref()
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| "Pre-export snapshot".to_string());

        let snapshot_options = super::CreateSnapshotOptions {
            name: snapshot_name,
            description: Some("Automatic snapshot created before export".to_string()),
            trigger_type: SnapshotTrigger::Export,
        };

        super::create_snapshot(
            project_id.clone(),
            snapshot_options,
            app_handle,
            state.clone(),
        )
        .await?;
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get project info
    let project = db::queries::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project not found: {}", project_id))?;

    let output_base = PathBuf::from(&options.output_path);

    // Use custom export name if provided, otherwise use project name
    let folder_name = options
        .export_name
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .map(|s| sanitize_filename(s))
        .unwrap_or_else(|| sanitize_filename(&project.name));

    // Create project folder
    let project_folder = output_base.join(folder_name);

    let mut files_created = 0;
    let mut chapters_exported = 0;
    let mut scenes_exported = 0;

    match options.scope {
        ExportScope::Project => {
            // Delete existing project folder if requested (only for project-level export)
            if options.delete_existing && project_folder.exists() {
                fs::remove_dir_all(&project_folder)
                    .map_err(|e| format!("Failed to delete existing folder: {}", e))?;
            }

            fs::create_dir_all(&project_folder)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;
            // Get all chapters
            let chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;

            let mut chapter_num = 0;
            for chapter in &chapters {
                if chapter.archived {
                    continue;
                }
                chapter_num += 1;

                let chapter_folder_name =
                    format!("{:02} - {}", chapter_num, sanitize_filename(&chapter.title));
                let chapter_folder = project_folder.join(&chapter_folder_name);
                fs::create_dir_all(&chapter_folder)
                    .map_err(|e| format!("Failed to create chapter directory: {}", e))?;

                // Get scenes for this chapter
                let scenes =
                    db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;

                let mut scene_num = 0;
                for scene in &scenes {
                    if scene.archived {
                        continue;
                    }
                    scene_num += 1;

                    let beats =
                        db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

                    let markdown =
                        generate_scene_markdown(scene, &beats, options.include_beat_markers);

                    let scene_file = chapter_folder.join(format!(
                        "{:02} - {}.md",
                        scene_num,
                        sanitize_filename(&scene.title)
                    ));

                    fs::write(&scene_file, markdown)
                        .map_err(|e| format!("Failed to write scene file: {}", e))?;

                    files_created += 1;
                    scenes_exported += 1;
                }

                chapters_exported += 1;
            }
        }
        ExportScope::Chapter(chapter_id) => {
            // Create project folder (don't delete it for chapter-level export)
            fs::create_dir_all(&project_folder)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;

            let chapter_uuid = Uuid::parse_str(&chapter_id).map_err(|e| e.to_string())?;

            // Get all chapters to find this chapter's position
            let all_chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;

            // Find the chapter and its position (1-based, excluding archived)
            let mut chapter_num = 0;
            let mut target_chapter = None;
            for ch in &all_chapters {
                if !ch.archived {
                    chapter_num += 1;
                    if ch.id == chapter_uuid {
                        target_chapter = Some(ch);
                        break;
                    }
                }
            }

            let chapter =
                target_chapter.ok_or_else(|| format!("Chapter not found: {}", chapter_id))?;

            let chapter_folder_name =
                format!("{:02} - {}", chapter_num, sanitize_filename(&chapter.title));
            let chapter_folder = project_folder.join(&chapter_folder_name);

            // Delete existing chapter folder if requested
            if options.delete_existing && chapter_folder.exists() {
                fs::remove_dir_all(&chapter_folder)
                    .map_err(|e| format!("Failed to delete existing chapter folder: {}", e))?;
            }

            fs::create_dir_all(&chapter_folder)
                .map_err(|e| format!("Failed to create chapter directory: {}", e))?;

            // Get scenes for this chapter
            let scenes = db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;

            let mut scene_num = 0;
            for scene in &scenes {
                if scene.archived {
                    continue;
                }
                scene_num += 1;

                let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

                let markdown = generate_scene_markdown(scene, &beats, options.include_beat_markers);

                let scene_file = chapter_folder.join(format!(
                    "{:02} - {}.md",
                    scene_num,
                    sanitize_filename(&scene.title)
                ));

                fs::write(&scene_file, markdown)
                    .map_err(|e| format!("Failed to write scene file: {}", e))?;

                files_created += 1;
                scenes_exported += 1;
            }

            chapters_exported = 1;
        }
        ExportScope::Scene(scene_id) => {
            // Create project folder (don't delete it for scene-level export)
            fs::create_dir_all(&project_folder)
                .map_err(|e| format!("Failed to create output directory: {}", e))?;

            let scene_uuid = Uuid::parse_str(&scene_id).map_err(|e| e.to_string())?;

            // Get scene info
            let scene = db::queries::get_scene_by_id(&conn, &scene_uuid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Scene not found: {}", scene_id))?;

            // Get chapter info to determine chapter position
            let chapter = db::queries::get_chapter_by_id(&conn, &scene.chapter_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Scene's chapter not found".to_string())?;

            // Get all chapters to find chapter position
            let all_chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;

            let mut chapter_num = 0;
            for ch in &all_chapters {
                if !ch.archived {
                    chapter_num += 1;
                    if ch.id == chapter.id {
                        break;
                    }
                }
            }

            // Get all scenes in this chapter to find scene position
            let all_scenes =
                db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;

            let mut scene_num = 0;
            for sc in &all_scenes {
                if !sc.archived {
                    scene_num += 1;
                    if sc.id == scene.id {
                        break;
                    }
                }
            }

            // Create chapter folder with prefix (don't delete it for scene-level export)
            let chapter_folder_name =
                format!("{:02} - {}", chapter_num, sanitize_filename(&chapter.title));
            let chapter_folder = project_folder.join(&chapter_folder_name);
            fs::create_dir_all(&chapter_folder)
                .map_err(|e| format!("Failed to create chapter directory: {}", e))?;

            let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

            let markdown = generate_scene_markdown(&scene, &beats, options.include_beat_markers);
            let scene_file = chapter_folder.join(format!(
                "{:02} - {}.md",
                scene_num,
                sanitize_filename(&scene.title)
            ));

            // Delete existing scene file if requested
            if options.delete_existing && scene_file.exists() {
                fs::remove_file(&scene_file)
                    .map_err(|e| format!("Failed to delete existing scene file: {}", e))?;
            }

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

/// Create the running header for the document
///
/// Standard Manuscript Format running header:
/// - Right-aligned: "Surname / TITLE / PageNumber"
/// - Courier New 12pt font
/// - Only appears on pages after the title page
fn create_running_header(author_surname: &str, title: &str) -> Header {
    // Format: Surname / TITLE / [page number]
    // Use abbreviated title (max 3 words) in uppercase
    let abbreviated_title = abbreviate_title(title, 3);
    let header_text = format!("{} / {} / ", author_surname, abbreviated_title);

    Header::new().add_paragraph(
        Paragraph::new()
            // Add the static text part
            .add_run(
                Run::new()
                    .add_text(&header_text)
                    .size(24) // 12pt
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            // Add the page number field
            // Field structure: BEGIN -> instruction -> SEPARATE -> result -> END
            .add_run(
                Run::new()
                    .add_field_char(FieldCharType::Begin, false)
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .add_run(
                Run::new()
                    .add_instr_text(InstrText::PAGE(InstrPAGE {}))
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .add_run(
                Run::new()
                    .add_field_char(FieldCharType::Separate, false)
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .add_run(
                Run::new()
                    .add_text("1") // Placeholder that Word will replace
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .add_run(
                Run::new()
                    .add_field_char(FieldCharType::End, false)
                    .size(24)
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .align(AlignmentType::Right),
    )
}

/// Create an empty header for the first page (title page)
fn create_empty_first_header() -> Header {
    Header::new()
}

/// Create heading styles and page setup for the DOCX document
///
/// Standard Manuscript Format:
/// - 1-inch margins on all sides
/// - Courier New 12pt font
/// - Double-spaced body text
/// - Running header with Surname / TITLE / PageNumber (not on title page)
fn create_docx_styles(author_name: Option<&str>, project_title: &str) -> Docx {
    // 1440 twips = 1 inch (there are 1440 twips per inch)
    let page_margin = PageMargin::new()
        .top(1440)
        .bottom(1440)
        .left(1440)
        .right(1440)
        .header(720) // 0.5 inch header margin
        .footer(720); // 0.5 inch footer margin

    // Extract surname for running header
    let surname = author_name.map(extract_surname).unwrap_or_default();

    // Create the running header (for all pages except first)
    let running_header = create_running_header(&surname, project_title);

    // Create empty header for title page
    let empty_header = create_empty_first_header();

    // With title_pg() enabled:
    // - first_header() sets the header for the first page only
    // - header() sets the header for all other pages (default header)
    // Note: Order matters - header() must be called before first_header()
    Docx::new()
        // Set page margins (1 inch on all sides)
        .page_margin(page_margin)
        // Enable different first page header
        .title_pg()
        // Running header for all pages after the first (must be set before first_header)
        .header(running_header)
        // Empty header for title page (first page)
        .first_header(empty_header)
        // Heading 1 style (for chapters) - large, bold, Courier New
        .add_style(
            Style::new("Heading1", StyleType::Paragraph)
                .name("Heading 1")
                .size(56) // 28pt (size is in half-points)
                .bold()
                .fonts(RunFonts::new().ascii("Courier New")),
        )
        // Heading 2 style (for scenes) - medium, bold, Courier New
        .add_style(
            Style::new("Heading2", StyleType::Paragraph)
                .name("Heading 2")
                .size(40) // 20pt
                .bold()
                .fonts(RunFonts::new().ascii("Courier New")),
        )
        // Heading 3 style (for beats) - smaller, bold italic, Courier New
        .add_style(
            Style::new("Heading3", StyleType::Paragraph)
                .name("Heading 3")
                .size(26) // 13pt
                .bold()
                .italic()
                .fonts(RunFonts::new().ascii("Courier New")),
        )
        // Synopsis style (italicized, Courier New)
        .add_style(
            Style::new("Synopsis", StyleType::Paragraph)
                .name("Synopsis")
                .size(22) // 11pt
                .italic()
                .fonts(RunFonts::new().ascii("Courier New")),
        )
        // Normal/body text style - Courier New, 12pt (set at run level for double-spacing)
        .add_style(
            Style::new("BodyText", StyleType::Paragraph)
                .name("Body Text")
                .size(24) // 12pt
                .fonts(RunFonts::new().ascii("Courier New")),
        )
}

/// Add a chapter to the document
///
/// SMF chapter formatting:
/// - Hard page break before each chapter (new page)
/// - Chapter heading ~1/3 down the page (~12-15 blank lines)
/// - Centered, ALL CAPS chapter heading
/// - 4-6 blank lines between heading and first paragraph
fn add_chapter_to_docx(
    docx: Docx,
    chapter: &Chapter,
    chapter_number: usize,
    scenes: &[Scene],
    beats_by_scene: &std::collections::HashMap<Uuid, Vec<Beat>>,
    options: &DocxExportOptions,
    is_first_chapter: bool,
) -> Docx {
    let mut docx = docx;

    // Add page break before chapter (except first chapter after title page)
    if !is_first_chapter && options.page_breaks_between_chapters {
        docx = docx.add_paragraph(Paragraph::new().page_break_before(true));
    }

    // SMF: Chapter heading should be about 1/3 down the page
    // Add approximately 12-14 blank lines to position heading at ~1/3 page
    // Using double-spaced blank paragraphs (480 twips line spacing)
    for _ in 0..12 {
        docx = docx.add_paragraph(
            Paragraph::new().line_spacing(LineSpacing::new().line(480)), // Double-spaced blank line
        );
    }

    // Format chapter heading based on selected style
    let chapter_heading = format_chapter_heading(
        chapter_number,
        &chapter.title,
        &options.chapter_heading_style,
    );

    // Chapter heading: centered, ALL CAPS, 12pt Courier New
    docx = docx.add_paragraph(
        Paragraph::new()
            .add_run(
                Run::new()
                    .add_text(&chapter_heading)
                    .size(24) // 12pt for SMF
                    .fonts(RunFonts::new().ascii("Courier New")),
            )
            .style("Heading1")
            .align(AlignmentType::Center)
            .line_spacing(LineSpacing::new().line(480)), // Double-spaced
    );

    // SMF: 4-6 blank lines between chapter heading and first paragraph
    // Using 4 double-spaced blank lines
    for _ in 0..4 {
        docx = docx.add_paragraph(
            Paragraph::new().line_spacing(LineSpacing::new().line(480)), // Double-spaced blank line
        );
    }

    // Add scenes with separators between them
    let active_scenes: Vec<&Scene> = scenes.iter().filter(|s| !s.archived).collect();
    for (i, scene) in active_scenes.iter().enumerate() {
        // Add scene separator (centered #) between scenes, not before first
        // Per Standard Manuscript Format, scene breaks use a single # character
        if i > 0 {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .add_run(
                        Run::new()
                            .add_text("#")
                            .size(24)
                            .fonts(RunFonts::new().ascii("Courier New")),
                    )
                    .align(AlignmentType::Center)
                    .line_spacing(LineSpacing::new().before(480).after(480).line(480)), // Double-spaced with extra space
            );
        }

        docx = add_scene_to_docx(
            docx,
            scene,
            beats_by_scene
                .get(&scene.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]),
            options,
        );
    }

    docx
}

/// Add a scene to the document
fn add_scene_to_docx(
    docx: Docx,
    scene: &Scene,
    beats: &[Beat],
    options: &DocxExportOptions,
) -> Docx {
    let mut docx = docx;

    // Scene title as Heading 2 - only include if beat markers are enabled
    // SMF: Scene titles are typically not included in manuscript submissions
    // They're organizational tools for the author, not content for the reader
    if options.include_beat_markers {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(&scene.title)
                        .size(24) // 12pt for SMF
                        .bold()
                        .fonts(RunFonts::new().ascii("Courier New")),
                )
                .style("Heading2")
                .line_spacing(LineSpacing::new().before(480).after(240).line(480)), // Double-spaced
        );
    }

    // Synopsis if requested and present - italicized, indented
    if options.include_synopsis {
        if let Some(ref synopsis) = scene.synopsis {
            if !synopsis.trim().is_empty() {
                docx = docx.add_paragraph(
                    Paragraph::new()
                        .add_run(
                            Run::new()
                                .add_text(synopsis)
                                .size(24) // 12pt
                                .italic()
                                .fonts(RunFonts::new().ascii("Courier New")),
                        )
                        .style("Synopsis")
                        .indent(Some(720), None, None, None) // 720 twips = 0.5 inch left indent
                        .line_spacing(LineSpacing::new().after(240).line(480)), // Double-spaced
                );
            }
        }
    }

    // Add beats
    for beat in beats {
        docx = add_beat_to_docx(docx, beat, options);
    }

    docx
}

/// Add a beat to the document
fn add_beat_to_docx(docx: Docx, beat: &Beat, options: &DocxExportOptions) -> Docx {
    let mut docx = docx;

    // Beat marker as Heading 3 if requested - with spacing
    if options.include_beat_markers {
        docx = docx.add_paragraph(
            Paragraph::new()
                .add_run(
                    Run::new()
                        .add_text(&beat.content)
                        .size(24) // 12pt
                        .bold()
                        .italic()
                        .fonts(RunFonts::new().ascii("Courier New")),
                )
                .style("Heading3")
                .line_spacing(LineSpacing::new().before(480).after(240).line(480)), // Double-spaced
        );
    }

    // Beat prose - Standard Manuscript Format: Courier New 12pt, double-spaced, 0.5" first-line indent
    if let Some(ref prose) = beat.prose {
        let clean_prose = strip_html(prose);
        if !clean_prose.is_empty() {
            // Split by double newlines to create separate paragraphs
            let paragraphs: Vec<&str> = clean_prose.split("\n\n").collect();
            for (i, para_text) in paragraphs.iter().enumerate() {
                let trimmed = para_text.trim();
                if !trimmed.is_empty() {
                    // SMF: First paragraph after heading has no indent, subsequent paragraphs have 0.5" indent
                    // 720 twips = 0.5 inch (standard manuscript indent)
                    // Line spacing 480 = double-spaced for 12pt font (240 twips = 12pt single)
                    let para = if i == 0 {
                        Paragraph::new()
                            .add_run(
                                Run::new()
                                    .add_text(trimmed)
                                    .size(24) // 12pt
                                    .fonts(RunFonts::new().ascii("Courier New")),
                            )
                            .style("BodyText")
                            .line_spacing(LineSpacing::new().line(480)) // Double-spaced
                    } else {
                        Paragraph::new()
                            .add_run(
                                Run::new()
                                    .add_text(trimmed)
                                    .size(24) // 12pt
                                    .fonts(RunFonts::new().ascii("Courier New")),
                            )
                            .style("BodyText")
                            .indent(None, None, Some(720), None) // 720 twips = 0.5 inch first-line indent
                            .line_spacing(LineSpacing::new().line(480)) // Double-spaced
                    };
                    docx = docx.add_paragraph(para);
                }
            }
        }
    }

    docx
}

/// Export project to DOCX file
///
/// Creates a single .docx file with chapters as H1, scenes as H2, beats as H3
#[tauri::command]
pub async fn export_to_docx(
    project_id: String,
    options: DocxExportOptions,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;

    // Create snapshot if requested (before taking the connection lock)
    if options.create_snapshot {
        let snapshot_options = super::CreateSnapshotOptions {
            name: "Pre-export snapshot".to_string(),
            description: Some("Automatic snapshot created before DOCX export".to_string()),
            trigger_type: SnapshotTrigger::Export,
        };

        super::create_snapshot(
            project_id.clone(),
            snapshot_options,
            app_handle.clone(),
            state.clone(),
        )
        .await?;
    }

    // Load app settings for title page (before taking db lock)
    let app_settings = load_app_settings(&app_handle)?;

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get project info
    let project = db::queries::get_project(&conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project not found: {}", project_id))?;

    let mut chapters_exported = 0;
    let mut scenes_exported = 0;

    // Determine author name for running header (pen name or app settings author name)
    let author_name_for_header = project
        .author_pen_name
        .as_ref()
        .filter(|s| !s.trim().is_empty())
        .or(app_settings.author_name.as_ref())
        .map(|s| s.as_str());

    // Initialize document with styles and running header
    let mut docx = create_docx_styles(author_name_for_header, &project.name);

    // Add title page if requested
    if options.include_title_page {
        // Calculate word count for title page
        let word_count = calculate_project_word_count(&conn, &project_uuid)?;
        docx = add_title_page(docx, &project, &app_settings, word_count);
    }

    match &options.scope {
        ExportScope::Project => {
            // Get all chapters
            let chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;

            // Pre-fetch all scenes and beats for efficiency
            let mut beats_by_scene: std::collections::HashMap<Uuid, Vec<Beat>> =
                std::collections::HashMap::new();

            let mut is_first_chapter = true;
            let mut chapter_number = 0;
            for chapter in chapters.iter().filter(|c| !c.archived) {
                chapter_number += 1;

                let scenes =
                    db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;
                let active_scenes: Vec<Scene> =
                    scenes.into_iter().filter(|s| !s.archived).collect();

                // Fetch beats for each scene
                for scene in &active_scenes {
                    let beats =
                        db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;
                    beats_by_scene.insert(scene.id, beats);
                }

                scenes_exported += active_scenes.len();

                docx = add_chapter_to_docx(
                    docx,
                    chapter,
                    chapter_number,
                    &active_scenes,
                    &beats_by_scene,
                    &options,
                    is_first_chapter,
                );

                chapters_exported += 1;
                is_first_chapter = false;
            }
        }
        ExportScope::Chapter(chapter_id) => {
            let chapter_uuid = Uuid::parse_str(chapter_id).map_err(|e| e.to_string())?;
            let chapter = db::queries::get_chapter_by_id(&conn, &chapter_uuid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Chapter not found: {}", chapter_id))?;

            // Get all chapters to determine this chapter's position number
            let all_chapters =
                db::queries::get_chapters(&conn, &project_uuid).map_err(|e| e.to_string())?;
            let chapter_number = all_chapters
                .iter()
                .filter(|c| !c.archived)
                .position(|c| c.id == chapter_uuid)
                .map(|pos| pos + 1) // Convert 0-indexed to 1-indexed
                .unwrap_or(1);

            let scenes = db::queries::get_scenes(&conn, &chapter.id).map_err(|e| e.to_string())?;
            let active_scenes: Vec<Scene> = scenes.into_iter().filter(|s| !s.archived).collect();

            let mut beats_by_scene: std::collections::HashMap<Uuid, Vec<Beat>> =
                std::collections::HashMap::new();

            for scene in &active_scenes {
                let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;
                beats_by_scene.insert(scene.id, beats);
            }

            scenes_exported = active_scenes.len();

            docx = add_chapter_to_docx(
                docx,
                &chapter,
                chapter_number,
                &active_scenes,
                &beats_by_scene,
                &options,
                true,
            );

            chapters_exported = 1;
        }
        ExportScope::Scene(scene_id) => {
            let scene_uuid = Uuid::parse_str(scene_id).map_err(|e| e.to_string())?;
            let scene = db::queries::get_scene_by_id(&conn, &scene_uuid)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("Scene not found: {}", scene_id))?;

            let beats = db::queries::get_beats(&conn, &scene.id).map_err(|e| e.to_string())?;

            docx = add_scene_to_docx(docx, &scene, &beats, &options);

            scenes_exported = 1;
        }
    }

    // Build and write the document
    let output_path = PathBuf::from(&options.output_path);

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    let file = fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    docx.build()
        .pack(file)
        .map_err(|e| format!("Failed to write DOCX file: {}", e))?;

    Ok(ExportResult {
        output_path: output_path.to_string_lossy().to_string(),
        files_created: 1,
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

    #[test]
    fn test_create_docx_styles() {
        // Test that the styles are created without panicking
        let docx = create_docx_styles(Some("John Smith"), "My Novel Title");
        // Build should succeed
        let built = docx.build();
        // Pack to a buffer should succeed
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        // Should produce a non-empty zip file
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_create_docx_styles_no_author() {
        // Test with no author name
        let docx = create_docx_styles(None, "Untitled");
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_extract_surname() {
        assert_eq!(extract_surname("John Smith"), "Smith");
        assert_eq!(extract_surname("Mary Jane Watson"), "Watson");
        assert_eq!(extract_surname("Prince"), "Prince");
        assert_eq!(extract_surname("John   Smith"), "Smith"); // Multiple spaces
        assert_eq!(extract_surname(""), "");
    }

    #[test]
    fn test_abbreviate_title() {
        // Short titles stay the same (but uppercase)
        assert_eq!(abbreviate_title("My Novel", 3), "MY NOVEL");
        assert_eq!(abbreviate_title("Title", 3), "TITLE");

        // Long titles get truncated
        assert_eq!(
            abbreviate_title("The Very Long Title of My Book", 3),
            "THE VERY LONG"
        );
        assert_eq!(abbreviate_title("A Tale of Two Cities", 3), "A TALE OF");

        // Exactly max_words
        assert_eq!(abbreviate_title("One Two Three", 3), "ONE TWO THREE");
    }

    #[test]
    fn test_add_beat_to_docx_with_prose() {
        use crate::models::Beat;
        use uuid::Uuid;

        let beat = Beat {
            id: Uuid::new_v4(),
            scene_id: Uuid::new_v4(),
            content: "Test Beat".to_string(),
            position: 0,
            prose: Some("<p>Test prose content</p>".to_string()),
            source_id: None,
        };

        let options = DocxExportOptions {
            scope: ExportScope::Project,
            include_beat_markers: true,
            include_synopsis: true,
            output_path: "/tmp/test.docx".to_string(),
            create_snapshot: false,
            page_breaks_between_chapters: true,
            include_title_page: true,
            chapter_heading_style: ChapterHeadingStyle::NumberOnly,
        };

        let docx = Docx::new();
        let docx = add_beat_to_docx(docx, &beat, &options);

        // Build should succeed
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_add_beat_to_docx_without_markers() {
        use crate::models::Beat;
        use uuid::Uuid;

        let beat = Beat {
            id: Uuid::new_v4(),
            scene_id: Uuid::new_v4(),
            content: "Test Beat".to_string(),
            position: 0,
            prose: Some("Plain text prose".to_string()),
            source_id: None,
        };

        let options = DocxExportOptions {
            scope: ExportScope::Project,
            include_beat_markers: false,
            include_synopsis: false,
            output_path: "/tmp/test.docx".to_string(),
            create_snapshot: false,
            page_breaks_between_chapters: false,
            include_title_page: false,
            chapter_heading_style: ChapterHeadingStyle::TitleOnly,
        };

        let docx = Docx::new();
        let docx = add_beat_to_docx(docx, &beat, &options);

        // Build should succeed
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_add_scene_to_docx() {
        use crate::models::{Beat, Scene};
        use uuid::Uuid;

        let scene = Scene {
            id: Uuid::new_v4(),
            chapter_id: Uuid::new_v4(),
            title: "Test Scene".to_string(),
            position: 0,
            synopsis: Some("This is a synopsis".to_string()),
            prose: None,
            locked: false,
            archived: false,
            source_id: None,
        };

        let beats = vec![Beat {
            id: Uuid::new_v4(),
            scene_id: scene.id,
            content: "Beat 1".to_string(),
            position: 0,
            prose: Some("Beat prose".to_string()),
            source_id: None,
        }];

        let options = DocxExportOptions {
            scope: ExportScope::Project,
            include_beat_markers: true,
            include_synopsis: true,
            output_path: "/tmp/test.docx".to_string(),
            create_snapshot: false,
            page_breaks_between_chapters: true,
            include_title_page: true,
            chapter_heading_style: ChapterHeadingStyle::NumberAndTitle,
        };

        let docx = Docx::new();
        let docx = add_scene_to_docx(docx, &scene, &beats, &options);

        // Build should succeed
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("  multiple   spaces  "), 2);
        assert_eq!(count_words("one"), 1);
        assert_eq!(count_words(""), 0);
        assert_eq!(
            count_words("This is a longer sentence with several words."),
            8
        );
    }

    #[test]
    fn test_round_word_count() {
        assert_eq!(round_word_count(500), "500 words");
        assert_eq!(round_word_count(999), "999 words");
        assert_eq!(round_word_count(1000), "approx. 1000 words");
        assert_eq!(round_word_count(1499), "approx. 1000 words");
        assert_eq!(round_word_count(1500), "approx. 2000 words");
        assert_eq!(round_word_count(75000), "approx. 75000 words");
        assert_eq!(round_word_count(75499), "approx. 75000 words");
        assert_eq!(round_word_count(75500), "approx. 76000 words");
    }

    #[test]
    fn test_add_title_page() {
        use crate::models::{Project, SourceType};

        let project = Project {
            id: uuid::Uuid::new_v4(),
            name: "My Novel".to_string(),
            source_type: SourceType::Markdown,
            source_path: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            author_pen_name: Some("Pen Name".to_string()),
            genre: Some("Literary Fiction".to_string()),
        };

        let app_settings = AppSettings {
            author_name: Some("Real Name".to_string()),
            contact_address_line1: Some("123 Main St".to_string()),
            contact_address_line2: Some("City, Country 12345".to_string()),
            contact_phone: Some("+1 555 1234".to_string()),
            contact_email: Some("author@email.com".to_string()),
        };

        let docx = Docx::new();
        let docx = add_title_page(docx, &project, &app_settings, 75000);

        // Build should succeed
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_add_title_page_minimal() {
        use crate::models::{Project, SourceType};

        // Test with minimal settings (no contact info)
        let project = Project {
            id: uuid::Uuid::new_v4(),
            name: "Untitled".to_string(),
            source_type: SourceType::Plottr,
            source_path: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            modified_at: chrono::Utc::now().to_rfc3339(),
            author_pen_name: None,
            genre: None,
        };

        let app_settings = AppSettings::default();

        let docx = Docx::new();
        let docx = add_title_page(docx, &project, &app_settings, 0);

        // Build should succeed even with no settings
        let built = docx.build();
        let mut buffer = Vec::new();
        built.pack(&mut std::io::Cursor::new(&mut buffer)).unwrap();
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_number_to_word() {
        // Basic numbers
        assert_eq!(number_to_word(0), "ZERO");
        assert_eq!(number_to_word(1), "ONE");
        assert_eq!(number_to_word(5), "FIVE");
        assert_eq!(number_to_word(10), "TEN");
        assert_eq!(number_to_word(11), "ELEVEN");
        assert_eq!(number_to_word(19), "NINETEEN");

        // Tens
        assert_eq!(number_to_word(20), "TWENTY");
        assert_eq!(number_to_word(30), "THIRTY");
        assert_eq!(number_to_word(50), "FIFTY");

        // Compound numbers
        assert_eq!(number_to_word(21), "TWENTY-ONE");
        assert_eq!(number_to_word(42), "FORTY-TWO");
        assert_eq!(number_to_word(99), "NINETY-NINE");

        // Edge cases
        assert_eq!(number_to_word(100), "ONE HUNDRED");
        assert_eq!(number_to_word(101), "101"); // Falls back to Arabic
    }

    #[test]
    fn test_format_chapter_heading() {
        // NumberOnly style (default SMF)
        assert_eq!(
            format_chapter_heading(1, "The Beginning", &ChapterHeadingStyle::NumberOnly),
            "CHAPTER ONE"
        );
        assert_eq!(
            format_chapter_heading(15, "Middle", &ChapterHeadingStyle::NumberOnly),
            "CHAPTER FIFTEEN"
        );

        // NumberAndTitle style
        assert_eq!(
            format_chapter_heading(1, "The Beginning", &ChapterHeadingStyle::NumberAndTitle),
            "CHAPTER ONE: THE BEGINNING"
        );
        assert_eq!(
            format_chapter_heading(
                5,
                "The Journey Continues",
                &ChapterHeadingStyle::NumberAndTitle
            ),
            "CHAPTER FIVE: THE JOURNEY CONTINUES"
        );

        // TitleOnly style
        assert_eq!(
            format_chapter_heading(1, "The Beginning", &ChapterHeadingStyle::TitleOnly),
            "THE BEGINNING"
        );

        // NumberArabic style
        assert_eq!(
            format_chapter_heading(1, "The Beginning", &ChapterHeadingStyle::NumberArabic),
            "CHAPTER 1"
        );
        assert_eq!(
            format_chapter_heading(42, "Whatever", &ChapterHeadingStyle::NumberArabic),
            "CHAPTER 42"
        );

        // NumberArabicAndTitle style
        assert_eq!(
            format_chapter_heading(
                1,
                "The Beginning",
                &ChapterHeadingStyle::NumberArabicAndTitle
            ),
            "CHAPTER 1: THE BEGINNING"
        );
    }

    #[test]
    fn test_chapter_heading_style_default() {
        // Default should be NumberOnly
        let style = ChapterHeadingStyle::default();
        assert!(matches!(style, ChapterHeadingStyle::NumberOnly));
    }
}
