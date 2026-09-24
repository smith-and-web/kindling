//! Format writers for the export workspace. Rendering is staged before publishing new output.
use super::*;
use crate::models::AppSettings;
use crate::parsers::html::{html_paragraphs, ParagraphKind};
use crate::parsers::xml_text::{pack_docx, xml_safe_text, LINE_BREAK_CONTROLS};
use docx_rs::*;
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};
use tauri::{AppHandle, State};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

type Result<T> = std::result::Result<T, String>;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDocument {
    title: String,
    author: String,
    subtitle: String,
    language: String,
    description: String,
    word_count: String,
    title_page: bool,
    contents: bool,
    cover_path: String,
    layout: Layout,
    chapters: Vec<WorkspaceChapter>,
    front_matter: Option<String>,
    text: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceChapter {
    title: String,
    navigation_title: String,
    part: Option<String>,
    xhtml: String,
    scenes: Vec<WorkspaceScene>,
}
#[derive(Deserialize)]
struct WorkspaceScene {
    title: String,
    synopsis: String,
    separator: Option<String>,
    blocks: Vec<WorkspaceBlock>,
}
#[derive(Deserialize)]
struct WorkspaceBlock {
    heading: String,
    html: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    font: String,
    font_size: f64,
    line_spacing: f64,
    paragraph_spacing: f64,
    indent: f64,
    alignment: String,
    first_paragraph_flush: bool,
    paper: String,
    margin: f64,
    chapter_breaks: bool,
    header: String,
}
fn xml(text: &str) -> String {
    xml_safe_text(text)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
fn validate(doc: &WorkspaceDocument) -> Result<()> {
    if doc.chapters.is_empty() {
        return Err("Select manuscript content before exporting.".into());
    }
    let l = &doc.layout;
    for (value, low, high) in [
        (l.font_size, 8., 24.),
        (l.line_spacing, 1., 3.),
        (l.paragraph_spacing, 0., 36.),
        (l.indent, 0., 1.),
        (l.margin, 0.25, 2.),
    ] {
        if !value.is_finite() || value < low || value > high {
            return Err("An export layout value is out of range.".into());
        }
    }
    if !["Times New Roman", "Courier New", "Georgia", "Arial"].contains(&l.font.as_str())
        || !["left", "justify"].contains(&l.alignment.as_str())
        || !["letter", "a4"].contains(&l.paper.as_str())
        || !["none", "title", "author_title"].contains(&l.header.as_str())
    {
        return Err("Unsupported export layout option.".into());
    }
    Ok(())
}
fn new_file(path: &Path, extension: &str) -> Result<tempfile::NamedTempFile> {
    if !path.is_absolute() || path.extension().and_then(|x| x.to_str()) != Some(extension) {
        return Err(format!(
            "Choose an absolute filename ending in .{extension}."
        ));
    }
    if path.symlink_metadata().is_ok() {
        return Err(
            "This destination already exists. Choose a new name; existing files are preserved."
                .into(),
        );
    }
    // Not NamedTempFile::new_in: its 0600 file would publish as owner-only.
    super::staging::staged_file_in(path.parent().ok_or("Choose a destination folder.")?)
        .map_err(|e| e.to_string())
}
fn publish(file: tempfile::NamedTempFile, target: &Path) -> Result<()> {
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist_noclobber(target)
        .map_err(|e| format!("Could not publish export; existing files are preserved. {e}"))?;
    Ok(())
}
fn styled(r: Run, l: &Layout) -> Run {
    r.fonts(RunFonts::new().ascii(&l.font).hi_ansi(&l.font))
        .size((l.font_size * 2.).round() as usize)
}
fn run(text: &str, l: &Layout) -> Run {
    styled(Run::new().add_text(text), l)
}
/// Scene titles, beat headings and similar secondary headings: body size, bold,
/// as the classic Standard Manuscript Format exporter sets them.
fn heading(text: &str, l: &Layout, style: &str) -> Paragraph {
    Paragraph::new()
        .style(style)
        .align(AlignmentType::Center)
        .add_run(run(text, l).bold())
        .line_spacing(LineSpacing::new().before(240).after(240))
        .keep_next(true)
}
/// One body line, in twips, at the layout's font size and line spacing.
fn body_line(l: &Layout) -> u32 {
    (l.font_size * 20. * l.line_spacing).round() as u32
}
/// Standard Manuscript Format chapter and part headings: centred, the same size
/// and weight as the body text, and spaced with the body's line spacing.
fn manuscript_heading(text: &str, l: &Layout, before: u32, after: u32) -> Paragraph {
    Paragraph::new()
        .style("Heading1")
        .align(AlignmentType::Center)
        .add_run(run(text, l))
        .line_spacing(
            LineSpacing::new()
                .line((l.line_spacing * 240.).round() as i32)
                .before(before)
                .after(after),
        )
        .keep_next(true)
}
fn page_break() -> Paragraph {
    Paragraph::new().add_run(Run::new().add_break(BreakType::Page))
}
/// The last word of the author name, as the classic exporter's running header uses.
fn surname(author: &str) -> &str {
    author.split_whitespace().last().unwrap_or_default()
}
/// The running header's short title: the first three words, uppercased.
fn short_title(title: &str) -> String {
    title
        .split_whitespace()
        .take(3)
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}
/// "Surname / SHORT TITLE / page", right-aligned, with a live PAGE field.
fn running_header(d: &WorkspaceDocument) -> Header {
    let l = &d.layout;
    let author = surname(&d.author);
    let text = if l.header == "author_title" && !author.is_empty() {
        format!("{author} / {} / ", short_title(&d.title))
    } else {
        format!("{} / ", short_title(&d.title))
    };
    Header::new().add_paragraph(
        Paragraph::new()
            .align(AlignmentType::Right)
            .add_run(run(&text, l))
            .add_run(styled(
                Run::new().add_field_char(FieldCharType::Begin, false),
                l,
            ))
            .add_run(styled(
                Run::new().add_instr_text(InstrText::PAGE(InstrPAGE {})),
                l,
            ))
            .add_run(styled(
                Run::new().add_field_char(FieldCharType::Separate, false),
                l,
            ))
            .add_run(run("1", l))
            .add_run(styled(
                Run::new().add_field_char(FieldCharType::End, false),
                l,
            )),
    )
}
/// The title page's top-left contact block, from Settings → Author & Contact.
fn contact_block(settings: &AppSettings) -> Vec<String> {
    [
        &settings.author_name,
        &settings.contact_address_line1,
        &settings.contact_address_line2,
        &settings.contact_phone,
        &settings.contact_email,
    ]
    .into_iter()
    .flatten()
    .map(|line| line.trim().to_string())
    .filter(|line| !line.is_empty())
    .collect()
}
fn prose(mut doc: Docx, html: &str, l: &Layout, first: &mut bool) -> Docx {
    for p in html_paragraphs(html, str::to_string) {
        let normal = matches!(p.kind, ParagraphKind::Normal);
        let mut para = Paragraph::new()
            .style("Normal")
            .align(if l.alignment == "justify" {
                AlignmentType::Both
            } else {
                AlignmentType::Left
            })
            .line_spacing(
                LineSpacing::new()
                    .line((l.line_spacing * 240.).round() as i32)
                    .after((l.paragraph_spacing * 20.).round() as u32),
            )
            .widow_control(true);
        if normal {
            let indent = if *first && l.first_paragraph_flush {
                0
            } else {
                (l.indent * 1440.).round() as i32
            };
            para = para.indent(None, Some(SpecialIndentType::FirstLine(indent)), None, None);
            *first = false;
        } else if matches!(p.kind, ParagraphKind::Blockquote) {
            para = para.indent(Some(720), None, Some(720), None);
        } else {
            *first = true;
            para = para.style("Heading2").keep_next(true);
        }
        for source in p.runs {
            // `<br>` arrives as \n; a pasted manual line break as U+000B/U+000C.
            for (i, line) in source
                .text
                .split(|c: char| c == '\n' || LINE_BREAK_CONTROLS.contains(&c))
                .enumerate()
            {
                if i > 0 {
                    para = para.add_run(Run::new().add_break(BreakType::TextWrapping));
                }
                if line.is_empty() {
                    continue;
                }
                let mut r = run(line, l);
                if source.marks[0] {
                    r = r.bold();
                }
                if source.marks[1] {
                    r = r.italic();
                }
                if source.marks[2] {
                    r = r.strike();
                }
                if source.marks[3] {
                    r = r.underline("single");
                }
                para = para.add_run(r);
            }
        }
        doc = doc.add_paragraph(para);
    }
    doc
}
/// Word output is a Standard Manuscript Format submission manuscript. The rules
/// follow the classic exporter (`export.rs`): contact block and word count at the
/// top of the title page, "Surname / SHORT TITLE / page" running header, chapter
/// headings at body size and weight about a third of the way down a new page, and
/// the typography (font, size, spacing, indent, margins) chosen in the profile.
fn docx_document(d: &WorkspaceDocument, contact: &[String]) -> Docx {
    let l = &d.layout;
    let margin = (l.margin * 1440.).round() as i32;
    let (width, height) = if l.paper == "a4" {
        (11906, 16838)
    } else {
        (12240, 15840)
    };
    let body_size = (l.font_size * 2.).round() as usize;
    let fonts = RunFonts::new().ascii(&l.font).hi_ansi(&l.font);
    // docx-rs always writes a `Normal` style, so the body typography goes in the
    // document defaults rather than in a second `Normal` definition.
    let mut doc = Docx::new()
        .page_size(width, height)
        .page_margin(
            PageMargin::new()
                .top(margin)
                .bottom(margin)
                .left(margin)
                .right(margin)
                .header(margin / 2)
                .footer(margin / 2),
        )
        .default_fonts(fonts.clone())
        .default_size(body_size);
    for (id, name, bold) in [
        ("Heading1", "Heading 1", false),
        ("Heading2", "Heading 2", true),
    ] {
        let mut style = Style::new(id, StyleType::Paragraph)
            .name(name)
            .fonts(fonts.clone())
            .size(body_size);
        if bold {
            style = style.bold();
        }
        doc = doc.add_style(style);
    }
    if l.header != "none" {
        doc = doc.header(running_header(d));
        if d.title_page {
            doc = doc.title_pg().first_header(Header::new());
        }
    }
    let line = body_line(l);
    // About a third of the way down the text block, as SMF places chapter openings.
    let drop = (height as i32 - 2 * margin).max(0) as u32 / 3;
    if d.title_page {
        for text in contact {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Left)
                    .add_run(run(text, l))
                    .line_spacing(LineSpacing::new().line(240)),
            );
        }
        if !d.word_count.is_empty() {
            doc = doc.add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Right)
                    .add_run(run(&d.word_count, l))
                    .line_spacing(LineSpacing::new().line(240).before(if contact.is_empty() {
                        0
                    } else {
                        240
                    })),
            );
        }
        doc = doc.add_paragraph(
            Paragraph::new()
                .align(AlignmentType::Center)
                .add_run(run(&d.title, l))
                .line_spacing(LineSpacing::new().before(drop).after(240))
                .keep_next(true),
        );
        for text in [
            d.subtitle.clone(),
            if d.author.is_empty() {
                String::new()
            } else {
                format!("by {}", d.author)
            },
        ] {
            if !text.is_empty() {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Center)
                        .add_run(run(&text, l))
                        .line_spacing(LineSpacing::new().after(240)),
                );
            }
        }
        doc = doc.add_paragraph(page_break());
    }
    if d.contents {
        doc = doc.add_paragraph(heading("Contents", l, "Heading2"));
        for (i, c) in d.chapters.iter().enumerate() {
            doc = doc.add_paragraph(
                Paragraph::new().add_hyperlink(
                    Hyperlink::new(format!("chapter_{i}"), HyperlinkType::Anchor)
                        .add_run(run(&c.navigation_title, l)),
                ),
            );
        }
        doc = doc.add_paragraph(page_break());
    }
    for (i, c) in d.chapters.iter().enumerate() {
        let new_page = i == 0 || l.chapter_breaks;
        if i > 0 && l.chapter_breaks {
            doc = doc.add_paragraph(page_break());
        }
        let mut before = if new_page { drop } else { 2 * line };
        if let Some(part) = &c.part {
            // Like the classic exporter, a Part title opens its own page.
            doc = doc.add_paragraph(manuscript_heading(part, l, before, 2 * line));
            if l.chapter_breaks {
                doc = doc.add_paragraph(page_break());
            } else {
                before = 0;
            }
        }
        let mut title = if c.title.is_empty() {
            Paragraph::new()
        } else {
            manuscript_heading(&c.title, l, before, 4 * line)
        };
        title = title
            .add_bookmark_start(i, format!("chapter_{i}"))
            .add_bookmark_end(i);
        doc = doc.add_paragraph(title);
        for s in &c.scenes {
            if let Some(marker) = &s.separator {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .align(AlignmentType::Center)
                        .add_run(run(marker, l))
                        .line_spacing(LineSpacing::new().before(240).after(240)),
                );
            }
            if !s.title.is_empty() {
                doc = doc.add_paragraph(heading(&s.title, l, "Heading2"));
            }
            if !s.synopsis.is_empty() {
                doc = doc.add_paragraph(
                    Paragraph::new()
                        .add_run(run(&s.synopsis, l).italic())
                        .line_spacing(LineSpacing::new().after(240)),
                );
            }
            let mut first = true;
            for b in &s.blocks {
                if !b.heading.is_empty() {
                    doc = doc.add_paragraph(heading(&b.heading, l, "Heading2"));
                    first = true;
                }
                doc = prose(doc, &b.html, l, &mut first);
            }
        }
    }
    doc
}
fn epub(file: &mut fs::File, d: &WorkspaceDocument) -> Result<()> {
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    zip.start_file("mimetype", stored)
        .map_err(|e| e.to_string())?;
    zip.write_all(b"application/epub+zip")
        .map_err(|e| e.to_string())?;
    let mut entries:Vec<(String,String)>=vec![("META-INF/container.xml".into(),r#"<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#.into())];
    let mut manifest = String::from(
        r#"<item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>"#,
    );
    let mut spine = String::new();
    if !d.cover_path.is_empty() {
        let mut cover =
            fs::File::open(&d.cover_path).map_err(|e| format!("Could not read cover: {e}"))?;
        let mut bytes = Vec::new();
        std::io::Read::by_ref(&mut cover)
            .take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err("Cover exceeds 16 MB.".into());
        }
        let (ext, mime) = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            ("png", "image/png")
        } else if bytes.starts_with(b"\xff\xd8\xff") {
            ("jpg", "image/jpeg")
        } else {
            return Err("Choose a PNG or JPEG cover image.".into());
        };
        zip.start_file(format!("OEBPS/cover.{ext}"), deflated)
            .map_err(|e| e.to_string())?;
        zip.write_all(&bytes).map_err(|e| e.to_string())?;
        manifest += &format!(
            r#"<item id="cover-image" href="cover.{ext}" media-type="{mime}" properties="cover-image"/><item id="cover" href="cover.xhtml" media-type="application/xhtml+xml"/>"#
        );
        spine += r#"<itemref idref="cover"/>"#;
        entries.push(("OEBPS/cover.xhtml".into(),format!(r#"<html xmlns="http://www.w3.org/1999/xhtml"><head><title>Cover</title></head><body><img src="cover.{ext}" alt="Cover" style="max-width:100%"/></body></html>"#)));
    }
    if let Some(front) = &d.front_matter {
        manifest += r#"<item id="title" href="title.xhtml" media-type="application/xhtml+xml"/>"#;
        spine += r#"<itemref idref="title"/>"#;
        entries.push(("OEBPS/title.xhtml".into(), front.clone()));
    }
    if d.contents {
        spine += r#"<itemref idref="nav"/>"#;
    }
    let mut nav = String::new();
    for (i, c) in d.chapters.iter().enumerate() {
        manifest += &format!(
            r#"<item id="chapter-{i}" href="chapter-{i}.xhtml" media-type="application/xhtml+xml"/>"#
        );
        spine += &format!(r#"<itemref idref="chapter-{i}"/>"#);
        nav += &format!(
            r#"<li><a href="chapter-{i}.xhtml">{}</a></li>"#,
            xml(&c.navigation_title)
        );
        entries.push((format!("OEBPS/chapter-{i}.xhtml"), c.xhtml.clone()));
    }
    entries.push(("OEBPS/nav.xhtml".into(),format!(r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="{}"><head><title>Contents</title></head><body><nav epub:type="toc" id="toc"><h1>Contents</h1><ol>{nav}</ol></nav></body></html>"#,xml(&d.language))));
    entries.push(("OEBPS/content.opf".into(),format!(r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="book-id">urn:uuid:{}</dc:identifier><dc:title>{}</dc:title><dc:creator>{}</dc:creator><dc:language>{}</dc:language><dc:description>{}</dc:description><meta property="dcterms:modified">{}</meta></metadata><manifest>{manifest}</manifest><spine>{spine}</spine></package>"#,uuid::Uuid::new_v4(),xml(&d.title),xml(&d.author),xml(&d.language),xml(&d.description),chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"))));
    for (path, body) in entries {
        // Chapter and title pages are rendered by the frontend; clean the whole
        // document, since no XML 1.0 markup can legitimately hold these characters.
        let body = xml_safe_text(&body);
        let mut reader = quick_xml::Reader::from_str(&body);
        loop {
            match reader.read_event() {
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(format!("Invalid EPUB XML in {path}: {e}")),
                _ => (),
            }
        }
        zip.start_file(path, deflated).map_err(|e| e.to_string())?;
        zip.write_all(body.as_bytes()).map_err(|e| e.to_string())?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub fn export_workspace_document(
    path: String,
    format: String,
    document: WorkspaceDocument,
    app_handle: AppHandle,
) -> Result<()> {
    // Only a Word title page carries the contact block.
    let contact = if format == "docx" && document.title_page {
        contact_block(&load_app_settings(&app_handle).map_err(|e| {
            format!("Could not read the contact details in Settings → Author & Contact. {e}")
        })?)
    } else {
        Vec::new()
    };
    write_workspace_document(path, format, document, &contact)
}
fn write_workspace_document(
    path: String,
    format: String,
    document: WorkspaceDocument,
    contact: &[String],
) -> Result<()> {
    validate(&document)?;
    let extension = match format.as_str() {
        "docx" => "docx",
        "epub" => "epub",
        "markdown" => "md",
        "txt" => "txt",
        _ => return Err("Unsupported manuscript format.".into()),
    };
    let target = Path::new(&path);
    let mut staged = new_file(target, extension)?;
    match format.as_str() {
        "docx" => pack_docx(docx_document(&document, contact), staged.as_file_mut())?,
        "epub" => epub(staged.as_file_mut(), &document)?,
        _ => staged
            .write_all(document.text.as_bytes())
            .map_err(|e| e.to_string())?,
    }
    publish(staged, target)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeOptions {
    include_notes: bool,
    include_beat_comments: bool,
    treatment_level: TreatmentLevel,
    treatment_format: TreatmentFormat,
}
fn copy_new_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir(target).map_err(|e| {
        format!("Choose a new destination folder. Existing output is preserved. {e}")
    })?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let to = target.join(entry.file_name());
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            copy_new_tree(&entry.path(), &to)?;
        } else {
            let mut out = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(to)
                .map_err(|e| e.to_string())?;
            let mut from = fs::File::open(entry.path()).map_err(|e| e.to_string())?;
            std::io::copy(&mut from, &mut out).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
/// Exchange formats retain their dedicated structure and metadata writers. Only new destinations are published.
#[tauri::command]
pub async fn export_workspace_exchange(
    project_id: String,
    path: String,
    format: String,
    options: ExchangeOptions,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ExportResult> {
    let target = Path::new(&path);
    if !target.is_absolute() || target.file_name().is_none() || target.symlink_metadata().is_ok() {
        return Err(
            "Choose a new absolute export destination. Existing files and folders are preserved."
                .into(),
        );
    }
    if !["longform", "scrivener", "novelwriter", "treatment"].contains(&format.as_str()) {
        return Err("Unsupported exchange format.".into());
    }
    if format == "scrivener" && target.extension().and_then(|ext| ext.to_str()) != Some("scriv") {
        return Err("Choose a new project name ending in .scriv.".into());
    }
    let temp = tempfile::tempdir_in(target.parent().ok_or("Choose a destination folder.")?)
        .map_err(|e| e.to_string())?;
    let stage = temp.path().join(if format == "scrivener" {
        target.file_name().ok_or("Choose a project name.")?
    } else {
        std::ffi::OsStr::new("Export")
    });
    let stage_string = stage.to_string_lossy().into_owned();
    let mut result = match format.as_str() {
        "longform" => {
            export_to_longform(
                project_id,
                LongformExportOptions {
                    scope: ExportScope::Project,
                    output_path: temp.path().to_string_lossy().into_owned(),
                    export_name: Some("Export".into()),
                    delete_existing: false,
                    create_snapshot: false,
                },
                app_handle,
                state,
            )
            .await?
        }
        "scrivener" => {
            export_to_scrivener(
                project_id,
                ScrivenerExportOptions {
                    mode: ScrivenerExportMode::CreateNew,
                    output_path: stage_string,
                    backup: true,
                    include_unmatched: true,
                    create_snapshot: false,
                },
                app_handle,
                state,
            )
            .await?
        }
        "novelwriter" => {
            export_to_novelwriter(
                project_id,
                stage_string,
                crate::parsers::novelwriter::NovelWriterExportOptions {
                    include_notes: options.include_notes,
                    include_beat_comments: options.include_beat_comments,
                    create_snapshot: false,
                },
                app_handle,
                state,
            )
            .await?
        }
        _ => {
            let ext = match options.treatment_format {
                TreatmentFormat::Docx => "docx",
                TreatmentFormat::Txt => "txt",
            };
            let source = temp.path().join(format!("treatment.{ext}"));
            let result = generate_treatment(
                project_id,
                TreatmentOptions {
                    detail_level: options.treatment_level,
                    format: options.treatment_format,
                    output_path: source.to_string_lossy().into_owned(),
                    create_snapshot: false,
                },
                app_handle,
                state,
            )
            .await?;
            let mut file = new_file(target, ext)?;
            let mut input = fs::File::open(source).map_err(|e| e.to_string())?;
            std::io::copy(&mut input, &mut file).map_err(|e| e.to_string())?;
            publish(file, target)?;
            return Ok(ExportResult {
                output_path: path,
                ..result
            });
        }
    };
    copy_new_tree(&stage,target).map_err(|e|format!("Export could not finish publishing to {}. Any new partial folder can be removed before retrying. {e}",target.display()))?;
    result.output_path = path;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn document() -> WorkspaceDocument {
        serde_json::from_value(serde_json::json!({
            "title":"The & Book", "author":"A Writer", "subtitle":"A subtitle", "language":"en", "description":"An & adventure",
            "wordCount":"12 words", "titlePage":true, "contents":true, "coverPath":"", "frontMatter":"<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Book</title></head><body>Title</body></html>", "text":"A manuscript\n",
            "layout":{"font":"Georgia","fontSize":14,"lineSpacing":1.5,"paragraphSpacing":8,"indent":0.5,"alignment":"justify","firstParagraphFlush":false,"paper":"a4","margin":1.25,"chapterBreaks":true,"header":"author_title"},
            "chapters":[{"title":"Chapter IV: Arrival","navigationTitle":"Chapter IV: Arrival","part":"Part One","xhtml":"<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Arrival</title></head><body><p>Prose &amp; more.</p></body></html>",
            "scenes":[{"title":"The shore","synopsis":"Arrival at dawn","separator":null,"blocks":[{"heading":"First sight","html":"<p>She <em>waited</em> &amp; <strong>listened</strong>.<br/>Then <s>ran</s> walked. 🌊</p>"}]}]}]
        })).unwrap()
    }
    fn export(path: String, format: String, document: WorkspaceDocument) -> Result<()> {
        write_workspace_document(path, format, document, &[])
    }
    fn member(path: &Path, name: &str) -> String {
        let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
        let mut text = String::new();
        zip.by_name(name)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }
    #[test]
    fn docx_honors_layout_formatting_and_contents_links() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.docx");
        export(path.to_string_lossy().into(), "docx".into(), document()).unwrap();
        let xml = member(&path, "word/document.xml");
        for value in [
            "Georgia",
            "w:sz w:val=\"28\"",
            "w:line=\"360\"",
            "w:firstLine=\"720\"",
            "w:top=\"1800\"",
            "w:w=\"11906\"",
            "w:anchor=\"chapter_0\"",
            "w:strike",
            "🌊",
        ] {
            assert!(xml.contains(value), "Missing {value}");
        }
        assert!(member(&path, "word/header1.xml").contains("Writer / THE &amp; BOOK / "));
    }
    #[test]
    fn flush_indentation_resets_after_each_heading_but_not_each_beat() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("flush.docx");
        let mut d = document();
        d.layout.first_paragraph_flush = true;
        let scene = &mut d.chapters[0].scenes[0];
        scene.blocks = vec![
            WorkspaceBlock {
                heading: "First beat".into(),
                html: "<p>First prose</p><p>Second prose</p>".into(),
            },
            WorkspaceBlock {
                heading: "".into(),
                html: "<p>Continuation</p>".into(),
            },
            WorkspaceBlock {
                heading: "Next beat".into(),
                html: "<p>After beat heading</p><h2>Prose heading</h2><p>After prose heading</p>"
                    .into(),
            },
        ];
        export(path.to_string_lossy().into(), "docx".into(), d).unwrap();
        let xml = member(&path, "word/document.xml");
        for (text, indent) in [
            ("First prose", 0),
            ("Second prose", 720),
            ("Continuation", 720),
            ("After beat heading", 0),
            ("After prose heading", 0),
        ] {
            let paragraph = xml.split("</w:p>").find(|p| p.contains(text)).unwrap();
            assert!(
                paragraph.contains(&format!("w:firstLine=\"{indent}\"")),
                "Wrong indent for {text}: {paragraph}"
            );
        }
    }
    #[test]
    fn epub_has_required_package_navigation_metadata_and_uncompressed_mimetype() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.epub");
        export(path.to_string_lossy().into(), "epub".into(), document()).unwrap();
        let mut zip = zip::ZipArchive::new(fs::File::open(&path).unwrap()).unwrap();
        let first = zip.by_index(0).unwrap();
        assert_eq!(first.name(), "mimetype");
        assert_eq!(first.compression(), CompressionMethod::Stored);
        drop(first);
        let opf = member(&path, "OEBPS/content.opf");
        assert!(opf.contains("The &amp; Book"));
        assert!(opf.contains("An &amp; adventure"));
        assert!(opf.contains("properties=\"nav\""));
        assert!(member(&path, "OEBPS/nav.xhtml").contains("chapter-0.xhtml"));
        assert!(member(&path, "OEBPS/chapter-0.xhtml").contains("Prose &amp; more."));
    }
    /// Prose pasted from Word carries U+000B for Shift+Enter; imports can carry
    /// other C0 controls. Neither may reach the XML.
    fn document_with_control_characters() -> WorkspaceDocument {
        let mut d = document();
        d.title = "The\u{0001} Book".into();
        d.author = "A\u{000B}Writer".into();
        d.description = "An\u{0001} adventure\u{FFFF}".into();
        d.front_matter = Some("<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Book\u{0001}</title></head><body>Title\u{000B}</body></html>".into());
        let c = &mut d.chapters[0];
        c.title = "Arrival\u{0001}".into();
        c.navigation_title = "Arrival\u{000C}".into();
        c.xhtml = "<html xmlns=\"http://www.w3.org/1999/xhtml\"><head><title>Arrival\u{0001}</title></head><body><p>Line\u{000B}two\u{0001}.</p></body></html>".into();
        c.scenes[0].synopsis = "Dawn\u{0001}".into();
        c.scenes[0].blocks[0].html = "<p>Line\u{000B}two\u{0001}.</p>".into();
        d
    }
    #[test]
    fn docx_with_control_characters_is_well_formed_and_keeps_line_breaks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.docx");
        export(
            path.to_string_lossy().into(),
            "docx".into(),
            document_with_control_characters(),
        )
        .unwrap();
        crate::parsers::xml_text::assert_archive_well_formed(&fs::read(&path).unwrap());
        let xml = member(&path, "word/document.xml");
        let paragraph = xml.split("</w:p>").find(|p| p.contains("Line")).unwrap();
        assert!(
            paragraph.contains("<w:br w:type=\"textWrapping\" />") && paragraph.contains("two."),
            "U+000B should become a soft line break: {paragraph}"
        );
        assert!(member(&path, "word/header1.xml").contains("Writer / THE BOOK / "));
    }
    /// Published exports must not be owner-only (tempfile's 0600): they get
    /// the same mode as any file created beside them.
    #[cfg(unix)]
    #[test]
    fn published_exports_get_default_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode() & 0o7777;
        let dir = tempfile::tempdir().unwrap();
        let reference = dir.path().join("reference.txt");
        fs::File::create(&reference).unwrap();
        for (name, format) in [
            ("book.docx", "docx"),
            ("book.epub", "epub"),
            ("book.md", "markdown"),
        ] {
            let path = dir.path().join(name);
            write_workspace_document(
                path.to_string_lossy().into(),
                format.into(),
                document(),
                &[],
            )
            .unwrap();
            assert_eq!(
                mode(&path),
                mode(&reference),
                "{name} is {:o}, a new file here is {:o}",
                mode(&path),
                mode(&reference)
            );
        }
    }
    #[test]
    fn epub_with_control_characters_is_well_formed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.epub");
        export(
            path.to_string_lossy().into(),
            "epub".into(),
            document_with_control_characters(),
        )
        .unwrap();
        crate::parsers::xml_text::assert_archive_well_formed(&fs::read(&path).unwrap());
        let opf = member(&path, "OEBPS/content.opf");
        assert!(opf.contains("<dc:title>The Book</dc:title>"), "{opf}");
        assert!(opf.contains("<dc:creator>A Writer</dc:creator>"), "{opf}");
        assert!(member(&path, "OEBPS/chapter-0.xhtml").contains("Line two."));
    }
    #[test]
    fn invalid_cover_and_existing_output_leave_previous_files_intact() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.epub");
        let mut d = document();
        d.cover_path = dir.path().join("missing.png").to_string_lossy().into();
        assert!(export(path.to_string_lossy().into(), "epub".into(), d).is_err());
        assert!(!path.exists());
        fs::write(&path, "original").unwrap();
        assert!(export(path.to_string_lossy().into(), "epub".into(), document()).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "original");
    }
    #[test]
    fn plain_exports_and_folder_publication_never_replace_existing_destinations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.md");
        export(path.to_string_lossy().into(), "markdown".into(), document()).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), "A manuscript\n");
        let source = dir.path().join("source");
        let target = dir.path().join("target");
        fs::create_dir(&source).unwrap();
        fs::write(source.join("file.md"), "first").unwrap();
        copy_new_tree(&source, &target).unwrap();
        assert!(copy_new_tree(&source, &target).is_err());
        assert_eq!(fs::read_to_string(target.join("file.md")).unwrap(), "first");
    }

    /// The "Agent submission" starter profile's Word payload.
    fn submission() -> WorkspaceDocument {
        let mut d = document();
        d.title = "The Long Way Home".into();
        d.author = "Jane Q. Writer".into();
        d.subtitle = String::new();
        d.contents = false;
        d.word_count = "80,000 words (approx.)".into();
        d.layout = serde_json::from_value(serde_json::json!({
            "font":"Times New Roman","fontSize":12,"lineSpacing":2,"paragraphSpacing":0,"indent":0.5,
            "alignment":"left","firstParagraphFlush":true,"paper":"letter","margin":1,
            "chapterBreaks":true,"header":"author_title"
        }))
        .unwrap();
        let c = &mut d.chapters[0];
        c.title = "Chapter 1: Arrival".into();
        c.part = None;
        c.scenes[0].title = String::new();
        c.scenes[0].synopsis = String::new();
        c.scenes[0].blocks = vec![WorkspaceBlock {
            heading: String::new(),
            html: "<p>First prose</p><p>Second prose</p>".into(),
        }];
        d
    }
    fn export_submission(d: WorkspaceDocument, contact: &[String]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("book.docx");
        write_workspace_document(path.to_string_lossy().into(), "docx".into(), d, contact).unwrap();
        crate::parsers::xml_text::assert_archive_well_formed(&fs::read(&path).unwrap());
        dir
    }
    fn paragraph<'a>(xml: &'a str, text: &str) -> &'a str {
        xml.split("</w:p>")
            .find(|p| p.contains(text))
            .unwrap_or_else(|| panic!("No paragraph containing {text}"))
    }
    fn bold(xml: &str) -> bool {
        xml.contains("<w:b />") || xml.contains("<w:b/>") || xml.contains("<w:b ")
    }
    fn style<'a>(styles: &'a str, id: &str) -> &'a str {
        styles
            .split("</w:style>")
            .find(|s| s.contains(&format!("w:styleId=\"{id}\"")))
            .unwrap_or_else(|| panic!("No style {id}"))
    }
    #[test]
    fn submission_header_has_surname_short_title_and_page_field() {
        let dir = export_submission(submission(), &[]);
        let path = dir.path().join("book.docx");
        let header = member(&path, "word/header1.xml");
        assert!(
            header.contains("Writer / THE LONG WAY / "),
            "Running header should be Surname / SHORT TITLE / page: {header}"
        );
        assert!(!header.contains("Jane"), "{header}");
        assert!(header.contains("w:fldCharType=\"begin\""), "{header}");
        assert!(header.contains("PAGE"), "No PAGE field: {header}");
        assert!(header.contains("w:fldCharType=\"end\""), "{header}");
        assert!(header.contains("Times New Roman"), "{header}");
        // The title page keeps its own empty first-page header.
        assert!(member(&path, "word/document.xml").contains("w:titlePg"));
    }
    #[test]
    fn title_only_header_keeps_the_page_field_without_an_author() {
        let mut d = submission();
        d.layout.header = "title".into();
        let dir = export_submission(d, &[]);
        let header = member(&dir.path().join("book.docx"), "word/header1.xml");
        assert!(header.contains(">THE LONG WAY / <"), "{header}");
        assert!(!header.contains("Writer"), "{header}");
        assert!(header.contains("PAGE"), "{header}");
    }
    #[test]
    fn docx_defines_the_normal_style_exactly_once() {
        let dir = export_submission(submission(), &[]);
        let styles = member(&dir.path().join("book.docx"), "word/styles.xml");
        assert_eq!(
            styles.matches("w:styleId=\"Normal\"").count(),
            1,
            "{styles}"
        );
        for id in ["Heading1", "Heading2"] {
            assert_eq!(styles.matches(&format!("w:styleId=\"{id}\"")).count(), 1);
        }
        // Body typography comes from the document defaults instead.
        let defaults = styles.split("</w:docDefaults>").next().unwrap();
        assert!(defaults.contains("Times New Roman"), "{defaults}");
        assert!(defaults.contains("w:sz w:val=\"24\""), "{defaults}");
    }
    #[test]
    fn submission_chapter_heading_is_body_size_not_bold_and_prose_is_flush_then_indented() {
        let dir = export_submission(submission(), &[]);
        let path = dir.path().join("book.docx");
        let xml = member(&path, "word/document.xml");
        let heading = paragraph(&xml, "Chapter 1: Arrival");
        assert!(heading.contains("w:val=\"Heading1\""), "{heading}");
        assert!(heading.contains("w:jc w:val=\"center\""), "{heading}");
        assert!(heading.contains("w:sz w:val=\"24\""), "{heading}");
        assert!(!bold(heading), "Heading run is bold: {heading}");
        // About a third of the way down a letter page's text block (9in / 3).
        assert!(heading.contains("w:before=\"4320\""), "{heading}");
        let styles = member(&path, "word/styles.xml");
        let h1 = style(&styles, "Heading1");
        assert!(!bold(h1), "Heading 1 style is bold: {h1}");
        // Control: the matcher does see bold where it is set.
        assert!(bold(style(&styles, "Heading2")));
        assert!(h1.contains("w:sz w:val=\"24\""), "{h1}");
        let first = paragraph(&xml, "First prose");
        assert!(
            first.contains("w:firstLine=\"0\"") || !first.contains("w:firstLine"),
            "First paragraph after a heading is indented: {first}"
        );
        assert!(
            first.contains("w:line=\"480\""),
            "Not double spaced: {first}"
        );
        assert!(first.contains("Times New Roman") && first.contains("w:sz w:val=\"24\""));
        assert!(paragraph(&xml, "Second prose").contains("w:firstLine=\"720\""));
    }
    #[test]
    fn submission_title_page_opens_with_contact_block_and_word_count() {
        let contact = ["Jane Writer", "1 Main Street", "jane@example.com"].map(String::from);
        let dir = export_submission(submission(), &contact);
        let xml = member(&dir.path().join("book.docx"), "word/document.xml");
        let body = xml.split("<w:body>").nth(1).unwrap();
        let order: Vec<usize> = [
            "Jane Writer",
            "1 Main Street",
            "jane@example.com",
            "80,000 words (approx.)",
            "The Long Way Home",
            "by Jane Q. Writer",
            "Chapter 1: Arrival",
        ]
        .iter()
        .map(|text| body.find(text).unwrap_or_else(|| panic!("Missing {text}")))
        .collect();
        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        let name = paragraph(body, "Jane Writer");
        assert!(name.contains("w:jc w:val=\"left\""), "{name}");
        assert!(
            name.contains("w:line=\"240\""),
            "Contact block is single spaced"
        );
        assert!(paragraph(body, "80,000 words").contains("w:jc w:val=\"right\""));
        let title = paragraph(body, "The Long Way Home");
        assert!(!bold(title) && !title.contains("Heading"), "{title}");
    }
    #[test]
    fn contact_block_uses_the_non_blank_author_settings_in_order() {
        let settings = AppSettings {
            author_name: Some(" Jane Writer ".into()),
            contact_address_line1: Some("  ".into()),
            contact_address_line2: Some("Springfield".into()),
            contact_phone: None,
            contact_email: Some("jane@example.com".into()),
        };
        assert_eq!(
            contact_block(&settings),
            ["Jane Writer", "Springfield", "jane@example.com"]
        );
        assert!(contact_block(&AppSettings::default()).is_empty());
    }
}
