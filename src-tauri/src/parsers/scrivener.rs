//! Scrivener 3 (.scriv) Parser and Generator
//!
//! Handles reading and writing Scrivener 3 `.scriv` bundles:
//! - Parse `.scrivx` XML to extract the document tree (BinderItems)
//! - Generate new `.scrivx` XML for export
//! - Convert HTML (from TipTap) to RTF for Scrivener content files
//!
//! A `.scriv` bundle is a directory containing:
//! - `<project>.scrivx` — XML index file with the document tree
//! - `Files/Data/<UUID>/content.rtf` — RTF content per document

use quick_xml::escape::unescape;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum ScrivenerError {
    #[error("Failed to read file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse XML: {0}")]
    XmlError(#[from] quick_xml::Error),
    #[error("Invalid .scrivx structure: {0}")]
    InvalidStructure(String),
}

// =============================================================================
// .scrivx Document Tree
// =============================================================================

/// A node in the Scrivener binder tree
#[derive(Debug, Clone)]
pub struct BinderItem {
    pub uuid: String,
    pub item_type: String,
    pub title: String,
    pub created: String,
    pub modified: String,
    pub include_in_compile: bool,
    pub children: Vec<BinderItem>,
    pub kindling_project_type: Option<String>,
}

/// Parsed contents of a .scrivx file
#[derive(Debug, Clone)]
pub struct ScrivxDocument {
    pub project_id: String,
    pub binder: Vec<BinderItem>,
}

/// Parse a .scrivx XML string into a document tree
pub fn parse_scrivx(xml: &str) -> Result<ScrivxDocument, ScrivenerError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut project_id = String::new();
    let mut binder_items: Vec<BinderItem> = Vec::new();
    let mut in_binder = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "ScrivenerProject" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Identifier" {
                            project_id = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                } else if name == "Binder" {
                    in_binder = true;
                } else if name == "BinderItem" && in_binder {
                    let item = parse_binder_item(&mut reader, e)?;
                    binder_items.push(item);
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"Binder" {
                    in_binder = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScrivenerError::XmlError(e)),
            _ => {}
        }
    }

    Ok(ScrivxDocument {
        project_id,
        binder: binder_items,
    })
}

fn parse_binder_item(
    reader: &mut Reader<&[u8]>,
    start: &BytesStart,
) -> Result<BinderItem, ScrivenerError> {
    let mut uuid = String::new();
    let mut item_type = String::new();
    let mut created = String::new();
    let mut modified = String::new();

    for attr in start.attributes().flatten() {
        match attr.key.as_ref() {
            b"UUID" => uuid = String::from_utf8_lossy(&attr.value).to_string(),
            b"Type" => item_type = String::from_utf8_lossy(&attr.value).to_string(),
            b"Created" => created = String::from_utf8_lossy(&attr.value).to_string(),
            b"Modified" => modified = String::from_utf8_lossy(&attr.value).to_string(),
            _ => {}
        }
    }

    let mut title = String::new();
    let mut include_in_compile = true;
    let mut children: Vec<BinderItem> = Vec::new();
    let mut kindling_project_type: Option<String> = None;
    let mut current_element = String::new();
    let mut compile_buf = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "BinderItem" {
                    let child = parse_binder_item(reader, e)?;
                    children.push(child);
                } else {
                    current_element = name;
                }
            }
            Ok(Event::Text(ref e)) => {
                let raw = String::from_utf8_lossy(e);
                let text = unescape(&raw)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|_| raw.to_string());
                match current_element.as_str() {
                    "Title" => title.push_str(&text),
                    "IncludeInCompile" => compile_buf.push_str(&text),
                    "KindlingProjectType" => {
                        kindling_project_type
                            .get_or_insert_with(String::new)
                            .push_str(&text);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "BinderItem" {
                    break;
                }
                if name == "IncludeInCompile" {
                    include_in_compile = compile_buf.trim() == "Yes";
                    compile_buf.clear();
                }
                if name == current_element {
                    current_element.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ScrivenerError::XmlError(e)),
            _ => {}
        }
    }

    Ok(BinderItem {
        uuid,
        item_type,
        title,
        created,
        modified,
        include_in_compile,
        children,
        kindling_project_type,
    })
}

/// Collect all text-type documents from a binder tree (flattened)
pub fn collect_text_documents(items: &[BinderItem]) -> Vec<&BinderItem> {
    let mut result = Vec::new();
    for item in items {
        if item.item_type == "Text" {
            result.push(item);
        }
        result.extend(collect_text_documents(&item.children));
    }
    result
}

// =============================================================================
// .scrivx Generation
// =============================================================================

/// A scene to include in the generated .scrivx
pub struct ExportScene {
    pub uuid: String,
    pub title: String,
    pub created: String,
    pub modified: String,
}

/// A chapter (folder) to include in the generated .scrivx
pub struct ExportChapter {
    pub uuid: String,
    pub title: String,
    pub is_part: bool,
    pub created: String,
    pub modified: String,
    pub scenes: Vec<ExportScene>,
    pub children: Vec<ExportChapter>,
}

/// Generate a complete .scrivx XML for a new Scrivener project
pub fn generate_scrivx(
    project_title: &str,
    chapters: &[ExportChapter],
    project_type: &str,
) -> Result<String, ScrivenerError> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let project_uuid = Uuid::new_v4().to_string().to_uppercase();
    let mut scriv_proj = BytesStart::new("ScrivenerProject");
    scriv_proj.push_attribute(("Identifier", project_uuid.as_str()));
    scriv_proj.push_attribute(("Version", "2.0"));
    writer.write_event(Event::Start(scriv_proj))?;

    // Binder
    writer.write_event(Event::Start(BytesStart::new("Binder")))?;

    // Draft folder
    let draft_uuid = Uuid::new_v4().to_string().to_uppercase();
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S %z")
        .to_string();
    let mut draft = BytesStart::new("BinderItem");
    draft.push_attribute(("UUID", draft_uuid.as_str()));
    draft.push_attribute(("Type", "DraftFolder"));
    draft.push_attribute(("Created", now.as_str()));
    draft.push_attribute(("Modified", now.as_str()));
    writer.write_event(Event::Start(draft))?;

    write_title(&mut writer, project_title)?;
    write_metadata_with_project_type(&mut writer, true, project_type)?;

    // Children of Draft
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    for chapter in chapters {
        if chapter.is_part && !chapter.children.is_empty() {
            write_part_folder_item(&mut writer, chapter)?;
        } else if chapter.is_part {
            write_text_item(
                &mut writer,
                &chapter.uuid,
                &chapter.title,
                &chapter.created,
                &chapter.modified,
                true,
            )?;
        } else {
            write_folder_item(
                &mut writer,
                &chapter.uuid,
                &chapter.title,
                &chapter.created,
                &chapter.modified,
                &chapter.scenes,
            )?;
        }
    }
    writer.write_event(Event::End(BytesEnd::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;

    // Research folder
    let research_uuid = Uuid::new_v4().to_string().to_uppercase();
    let mut research = BytesStart::new("BinderItem");
    research.push_attribute(("UUID", research_uuid.as_str()));
    research.push_attribute(("Type", "ResearchFolder"));
    research.push_attribute(("Created", now.as_str()));
    research.push_attribute(("Modified", now.as_str()));
    writer.write_event(Event::Start(research))?;
    write_title(&mut writer, "Research")?;
    write_metadata(&mut writer, false)?;
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;

    // Trash folder
    let trash_uuid = Uuid::new_v4().to_string().to_uppercase();
    let mut trash = BytesStart::new("BinderItem");
    trash.push_attribute(("UUID", trash_uuid.as_str()));
    trash.push_attribute(("Type", "TrashFolder"));
    trash.push_attribute(("Created", now.as_str()));
    trash.push_attribute(("Modified", now.as_str()));
    writer.write_event(Event::Start(trash))?;
    write_title(&mut writer, "Trash")?;
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;

    writer.write_event(Event::End(BytesEnd::new("Binder")))?;
    writer.write_event(Event::End(BytesEnd::new("ScrivenerProject")))?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| ScrivenerError::InvalidStructure(e.to_string()))
}

fn write_title(writer: &mut Writer<Cursor<Vec<u8>>>, title: &str) -> Result<(), ScrivenerError> {
    writer.write_event(Event::Start(BytesStart::new("Title")))?;
    writer.write_event(Event::Text(BytesText::new(title)))?;
    writer.write_event(Event::End(BytesEnd::new("Title")))?;
    Ok(())
}

fn write_metadata(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    include_in_compile: bool,
) -> Result<(), ScrivenerError> {
    writer.write_event(Event::Start(BytesStart::new("MetaData")))?;
    writer.write_event(Event::Start(BytesStart::new("IncludeInCompile")))?;
    writer.write_event(Event::Text(BytesText::new(if include_in_compile {
        "Yes"
    } else {
        "No"
    })))?;
    writer.write_event(Event::End(BytesEnd::new("IncludeInCompile")))?;
    writer.write_event(Event::End(BytesEnd::new("MetaData")))?;
    Ok(())
}

fn write_metadata_with_project_type(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    include_in_compile: bool,
    project_type: &str,
) -> Result<(), ScrivenerError> {
    writer.write_event(Event::Start(BytesStart::new("MetaData")))?;
    writer.write_event(Event::Start(BytesStart::new("IncludeInCompile")))?;
    writer.write_event(Event::Text(BytesText::new(if include_in_compile {
        "Yes"
    } else {
        "No"
    })))?;
    writer.write_event(Event::End(BytesEnd::new("IncludeInCompile")))?;
    writer.write_event(Event::Start(BytesStart::new("CustomMetaData")))?;
    writer.write_event(Event::Start(BytesStart::new("KindlingProjectType")))?;
    writer.write_event(Event::Text(BytesText::new(project_type)))?;
    writer.write_event(Event::End(BytesEnd::new("KindlingProjectType")))?;
    writer.write_event(Event::End(BytesEnd::new("CustomMetaData")))?;
    writer.write_event(Event::End(BytesEnd::new("MetaData")))?;
    Ok(())
}

fn write_text_item(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    uuid: &str,
    title: &str,
    created: &str,
    modified: &str,
    include_in_compile: bool,
) -> Result<(), ScrivenerError> {
    let mut elem = BytesStart::new("BinderItem");
    elem.push_attribute(("UUID", uuid));
    elem.push_attribute(("Type", "Text"));
    elem.push_attribute(("Created", created));
    elem.push_attribute(("Modified", modified));
    writer.write_event(Event::Start(elem))?;
    write_title(writer, title)?;
    write_metadata(writer, include_in_compile)?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;
    Ok(())
}

fn write_folder_item(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    uuid: &str,
    title: &str,
    created: &str,
    modified: &str,
    scenes: &[ExportScene],
) -> Result<(), ScrivenerError> {
    let mut elem = BytesStart::new("BinderItem");
    elem.push_attribute(("UUID", uuid));
    elem.push_attribute(("Type", "Folder"));
    elem.push_attribute(("Created", created));
    elem.push_attribute(("Modified", modified));
    writer.write_event(Event::Start(elem))?;
    write_title(writer, title)?;
    write_metadata(writer, true)?;
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    for scene in scenes {
        write_text_item(
            writer,
            &scene.uuid,
            &scene.title,
            &scene.created,
            &scene.modified,
            true,
        )?;
    }
    writer.write_event(Event::End(BytesEnd::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;
    Ok(())
}

fn write_part_folder_item(
    writer: &mut Writer<Cursor<Vec<u8>>>,
    part: &ExportChapter,
) -> Result<(), ScrivenerError> {
    let mut elem = BytesStart::new("BinderItem");
    elem.push_attribute(("UUID", part.uuid.as_str()));
    elem.push_attribute(("Type", "Folder"));
    elem.push_attribute(("Created", part.created.as_str()));
    elem.push_attribute(("Modified", part.modified.as_str()));
    writer.write_event(Event::Start(elem))?;
    write_title(writer, &part.title)?;
    write_metadata(writer, true)?;
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    for child in &part.children {
        write_folder_item(
            writer,
            &child.uuid,
            &child.title,
            &child.created,
            &child.modified,
            &child.scenes,
        )?;
    }
    writer.write_event(Event::End(BytesEnd::new("Children")))?;
    writer.write_event(Event::End(BytesEnd::new("BinderItem")))?;
    Ok(())
}

/// One `BinderItem` subtree (folder + scenes, part branch, or empty part as Text) for appending to an existing `.scrivx`.
pub fn export_chapter_binder_fragment(chapter: &ExportChapter) -> Result<String, ScrivenerError> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    if chapter.is_part && !chapter.children.is_empty() {
        write_part_folder_item(&mut writer, chapter)?;
    } else if chapter.is_part {
        write_text_item(
            &mut writer,
            &chapter.uuid,
            &chapter.title,
            &chapter.created,
            &chapter.modified,
            true,
        )?;
    } else {
        write_folder_item(
            &mut writer,
            &chapter.uuid,
            &chapter.title,
            &chapter.created,
            &chapter.modified,
            &chapter.scenes,
        )?;
    }
    let bytes = writer.into_inner().into_inner();
    String::from_utf8(bytes).map_err(|e| ScrivenerError::InvalidStructure(e.to_string()))
}

// =============================================================================
// HTML → RTF Converter
// =============================================================================

/// RTF header compatible with Scrivener 3
const RTF_HEADER: &str = r"{\rtf1\ansi\ansicpg1252\cocoartf2761
{\fonttbl\f0\fmodern\fcharset0 Courier;\f1\fswiss\fcharset0 Helvetica;}
{\colortbl;\red255\green255\blue255;\red0\green0\blue0;}
\paperw12240\paperh15840\margl1440\margr1440\margt1440\margb1440
";

/// Convert TipTap HTML to Scrivener-compatible RTF
pub fn html_to_rtf(html: &str) -> String {
    let mut rtf = String::from(RTF_HEADER);
    let trimmed = html.trim();

    if trimmed.is_empty() {
        rtf.push('}');
        return rtf;
    }

    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut text_buf = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut chars = trimmed.chars().peekable();
    let mut first_paragraph = true;

    while let Some(c) = chars.next() {
        if c == '<' {
            if !text_buf.is_empty() {
                rtf.push_str(&escape_rtf(&text_buf));
                text_buf.clear();
            }
            in_tag = true;
            tag_buf.clear();
        } else if c == '>' && in_tag {
            in_tag = false;
            let tag = tag_buf.trim().to_lowercase();
            process_tag(&tag, &mut rtf, &mut tags, &mut first_paragraph);
            tag_buf.clear();
        } else if in_tag {
            tag_buf.push(c);
        } else {
            // Handle HTML entities
            if c == '&' {
                let mut entity = String::from("&");
                for ec in chars.by_ref() {
                    entity.push(ec);
                    if ec == ';' {
                        break;
                    }
                }
                text_buf.push_str(&decode_entity(&entity));
            } else {
                text_buf.push(c);
            }
        }
    }

    if !text_buf.is_empty() {
        rtf.push_str(&escape_rtf(&text_buf));
    }

    rtf.push('}');
    rtf
}

fn process_tag(tag: &str, rtf: &mut String, tags: &mut Vec<String>, first_paragraph: &mut bool) {
    let (tag_name, is_closing) = if let Some(stripped) = tag.strip_prefix('/') {
        (stripped.split_whitespace().next().unwrap_or(""), true)
    } else {
        (
            tag.split_whitespace()
                .next()
                .unwrap_or(tag.split('/').next().unwrap_or("")),
            false,
        )
    };

    let is_self_closing = tag.ends_with('/');

    match tag_name {
        "p" if !is_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            if tags.iter().any(|t| t == "blockquote") {
                rtf.push_str(
                    "\\pard\\li720\\pardirnatural\\partightenfactor0\n\\f0\\i\\fs24 \\cf2 ",
                );
            } else {
                rtf.push_str("\\pard\\pardirnatural\\partightenfactor0\n\\f0\\fs24 \\cf2 ");
            }
        }
        "p" if is_closing => {}
        "strong" | "b" if !is_closing => {
            rtf.push_str("\\b ");
            tags.push("b".to_string());
        }
        "strong" | "b" if is_closing => {
            rtf.push_str("\\b0 ");
            tags.retain(|t| t != "b");
        }
        "em" | "i" if !is_closing => {
            rtf.push_str("\\i ");
            tags.push("i".to_string());
        }
        "em" | "i" if is_closing => {
            rtf.push_str("\\i0 ");
            tags.retain(|t| t != "i");
        }
        "u" if !is_closing => {
            rtf.push_str("\\ul ");
            tags.push("u".to_string());
        }
        "u" if is_closing => {
            rtf.push_str("\\ulnone ");
            tags.retain(|t| t != "u");
        }
        "h1" if !is_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            rtf.push_str("\\pard\\pardirnatural\\partightenfactor0\n\\f1\\b\\fs36 \\cf2 ");
        }
        "h1" if is_closing => {
            rtf.push_str("\\b0\\f0\\fs24 ");
        }
        "h2" if !is_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            rtf.push_str("\\pard\\pardirnatural\\partightenfactor0\n\\f1\\b\\fs30 \\cf2 ");
        }
        "h2" if is_closing => {
            rtf.push_str("\\b0\\f0\\fs24 ");
        }
        "h3" if !is_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            rtf.push_str("\\pard\\pardirnatural\\partightenfactor0\n\\f1\\b\\fs26 \\cf2 ");
        }
        "h3" if is_closing => {
            rtf.push_str("\\b0\\f0\\fs24 ");
        }
        "blockquote" if !is_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            tags.push("blockquote".to_string());
        }
        "blockquote" if is_closing => {
            rtf.push_str("\\i0 ");
            tags.retain(|t| t != "blockquote");
        }
        "br" | "br/" => {
            rtf.push_str("\\line ");
        }
        "hr" if !is_closing || is_self_closing => {
            if !*first_paragraph {
                rtf.push_str("\\par\n");
            }
            *first_paragraph = false;
            rtf.push_str("\\pard\\qc\\pardirnatural\\partightenfactor0\n\\f0\\fs24 \\cf2 * * *");
        }
        _ => {}
    }
}

fn escape_rtf(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '{' => result.push_str("\\{"),
            '}' => result.push_str("\\}"),
            '\n' => result.push_str("\\line "),
            c if (c as u32) > 127 => {
                result.push_str(&format!("\\u{}?", c as i16 as u16));
            }
            _ => result.push(c),
        }
    }
    result
}

fn decode_entity(entity: &str) -> String {
    match entity {
        "&amp;" => "&".to_string(),
        "&lt;" => "<".to_string(),
        "&gt;" => ">".to_string(),
        "&quot;" => "\"".to_string(),
        "&#39;" | "&apos;" => "'".to_string(),
        "&nbsp;" => " ".to_string(),
        "&mdash;" | "&#8212;" => "\u{2014}".to_string(),
        "&ndash;" | "&#8211;" => "\u{2013}".to_string(),
        "&hellip;" | "&#8230;" => "\u{2026}".to_string(),
        "&lsquo;" | "&#8216;" => "\u{2018}".to_string(),
        "&rsquo;" | "&#8217;" => "\u{2019}".to_string(),
        "&ldquo;" | "&#8220;" => "\u{201C}".to_string(),
        "&rdquo;" | "&#8221;" => "\u{201D}".to_string(),
        _ => entity.to_string(),
    }
}

// =============================================================================
// RTF → HTML Converter (for Scrivener import)
// =============================================================================

/// Left indent in twips at or above this threshold is treated as `<blockquote>`
/// (matches `html_to_rtf` which uses `\li720` for blockquotes). Use 720 so normal
/// indented paragraphs (\li360, etc.) are not misclassified.
const BLOCKQUOTE_LI_THRESHOLD: i32 = 720;

/// Convert Scrivener RTF content to TipTap-compatible HTML.
///
/// Each text run is self-contained with its own formatting tags, so changes
/// to bold/italic/underline mid-paragraph produce correct HTML.
pub fn rtf_to_html(rtf: &str) -> String {
    let mut html = String::new();
    let mut in_paragraph = false;
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut li_twips: i32 = 0;
    let mut blockquote_active = false;
    let mut skip_depth: u32 = 0;
    let mut chars = rtf.chars().peekable();
    let mut text_buf = String::new();

    while let Some(c) = chars.next() {
        if c == '{' {
            if skip_depth > 0 {
                skip_depth += 1;
                continue;
            }
            // Peek ahead to detect groups we should skip entirely
            let mut peek_buf = String::new();
            let mut peek_chars = chars.clone();
            for _ in 0..14 {
                if let Some(pc) = peek_chars.next() {
                    peek_buf.push(pc);
                }
            }
            if peek_buf.starts_with("\\fonttbl")
                || peek_buf.starts_with("\\colortbl")
                || peek_buf.starts_with("\\stylesheet")
                || peek_buf.starts_with("\\info")
                || peek_buf.starts_with("\\*")
            {
                skip_depth = 1;
            }
            continue;
        }

        if c == '}' {
            skip_depth = skip_depth.saturating_sub(1);
            continue;
        }

        if skip_depth > 0 {
            continue;
        }

        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    '\\' => {
                        chars.next();
                        text_buf.push('\\');
                    }
                    '{' => {
                        chars.next();
                        text_buf.push('{');
                    }
                    '}' => {
                        chars.next();
                        text_buf.push('}');
                    }
                    '\'' => {
                        chars.next();
                        let mut hex = String::new();
                        for _ in 0..2 {
                            if let Some(&hc) = chars.peek() {
                                if hc.is_ascii_hexdigit() {
                                    hex.push(hc);
                                    chars.next();
                                }
                            }
                        }
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            text_buf.push(byte as char);
                        }
                    }
                    _ => {
                        let mut word = String::new();
                        while let Some(&wc) = chars.peek() {
                            if wc.is_ascii_alphabetic() {
                                word.push(wc);
                                chars.next();
                            } else {
                                break;
                            }
                        }

                        let mut param = String::new();
                        if let Some(&pc) = chars.peek() {
                            if pc == '-' || pc.is_ascii_digit() {
                                param.push(pc);
                                chars.next();
                                while let Some(&dc) = chars.peek() {
                                    if dc.is_ascii_digit() {
                                        param.push(dc);
                                        chars.next();
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }

                        if let Some(&sc) = chars.peek() {
                            if sc == ' ' {
                                chars.next();
                            }
                        }

                        match word.as_str() {
                            "par" | "line" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                if in_paragraph {
                                    html.push_str("</p>");
                                    in_paragraph = false;
                                }
                            }
                            "b" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                bold = param != "0";
                            }
                            "i" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                italic = param != "0";
                            }
                            "ul" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                underline = true;
                            }
                            "ulnone" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                underline = false;
                            }
                            "li" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                li_twips = param.parse::<i32>().unwrap_or(0).max(0);
                            }
                            "u" => {
                                if let Ok(code) = param.parse::<i32>() {
                                    let code = if code < 0 {
                                        (code + 65536) as u32
                                    } else {
                                        code as u32
                                    };
                                    if let Some(ch) = char::from_u32(code) {
                                        text_buf.push(ch);
                                    }
                                }
                                if let Some(&rc) = chars.peek() {
                                    if rc == '?' {
                                        chars.next();
                                    }
                                }
                            }
                            "pard" | "plain" => {
                                rtf_flush_run(
                                    &mut html,
                                    &mut text_buf,
                                    &mut in_paragraph,
                                    &mut blockquote_active,
                                    li_twips,
                                    bold,
                                    italic,
                                    underline,
                                );
                                bold = false;
                                italic = false;
                                underline = false;
                                li_twips = 0;
                            }
                            _ => {}
                        }
                    }
                }
            }
            continue;
        }

        if c == '\r' || c == '\n' {
            continue;
        }

        text_buf.push(c);
    }

    rtf_flush_run(
        &mut html,
        &mut text_buf,
        &mut in_paragraph,
        &mut blockquote_active,
        li_twips,
        bold,
        italic,
        underline,
    );

    if in_paragraph {
        html.push_str("</p>");
    }
    if blockquote_active {
        html.push_str("</blockquote>");
    }

    html
}

/// Flush accumulated text as a self-contained formatted run.
#[allow(clippy::too_many_arguments)]
fn rtf_flush_run(
    html: &mut String,
    text_buf: &mut String,
    in_paragraph: &mut bool,
    blockquote_active: &mut bool,
    li_twips: i32,
    bold: bool,
    italic: bool,
    underline: bool,
) {
    if text_buf.is_empty() {
        return;
    }

    if !*in_paragraph {
        if *blockquote_active && li_twips < BLOCKQUOTE_LI_THRESHOLD {
            html.push_str("</blockquote>");
            *blockquote_active = false;
        }
        if !*blockquote_active && li_twips >= BLOCKQUOTE_LI_THRESHOLD {
            html.push_str("<blockquote>");
            *blockquote_active = true;
        }
        html.push_str("<p>");
        *in_paragraph = true;
    }

    if bold {
        html.push_str("<strong>");
    }
    if italic {
        html.push_str("<em>");
    }
    if underline {
        html.push_str("<u>");
    }

    html.push_str(&html_escape(text_buf));
    text_buf.clear();

    if underline {
        html.push_str("</u>");
    }
    if italic {
        html.push_str("</em>");
    }
    if bold {
        html.push_str("</strong>");
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// =============================================================================
// Scrivener Bundle Parser (for import)
// =============================================================================

/// Result of parsing a .scriv bundle for import
pub struct ParsedScrivener {
    pub project: crate::models::Project,
    pub chapters: Vec<crate::models::Chapter>,
    pub scenes: Vec<crate::models::Scene>,
    pub beats: Vec<crate::models::Beat>,
}

/// Recursively process binder children, detecting Parts from nested Folder structure.
/// A Folder with at least one Folder child is treated as a Part (`is_part: true`).
fn process_binder_children(
    children: &[BinderItem],
    project_id: uuid::Uuid,
    data_dir: &std::path::Path,
    chapters: &mut Vec<crate::models::Chapter>,
    scenes: &mut Vec<crate::models::Scene>,
    beats: &mut Vec<crate::models::Beat>,
    position: &mut i32,
) {
    use crate::models::{Beat, Chapter, Scene};

    for child in children {
        match child.item_type.as_str() {
            "Folder" => {
                let has_folder_children = child.children.iter().any(|c| c.item_type == "Folder");

                if has_folder_children {
                    chapters.push(Chapter {
                        id: uuid::Uuid::new_v4(),
                        project_id,
                        title: child.title.clone(),
                        synopsis: None,
                        position: *position,
                        is_part: true,
                        archived: false,
                        locked: false,
                        source_id: Some(child.uuid.clone()),
                        planning_status: Default::default(),
                    });
                    *position += 1;

                    process_binder_children(
                        &child.children,
                        project_id,
                        data_dir,
                        chapters,
                        scenes,
                        beats,
                        position,
                    );
                } else {
                    let chapter = Chapter {
                        id: uuid::Uuid::new_v4(),
                        project_id,
                        title: child.title.clone(),
                        synopsis: None,
                        position: *position,
                        is_part: false,
                        archived: false,
                        locked: false,
                        source_id: Some(child.uuid.clone()),
                        planning_status: Default::default(),
                    };

                    let mut scene_pos: i32 = 0;
                    for scene_item in &child.children {
                        if scene_item.item_type == "Text" {
                            let prose = read_rtf_content(data_dir, &scene_item.uuid);
                            let scene_id = uuid::Uuid::new_v4();

                            if let Some(ref prose_html) = prose {
                                let mut beat = Beat::new(scene_id, "Scene Content".to_string(), 0)
                                    .with_source_id(Some(format!("{}-prose", scene_item.uuid)));
                                beat.prose = Some(prose_html.clone());
                                beats.push(beat);
                            }

                            scenes.push(Scene {
                                id: scene_id,
                                chapter_id: chapter.id,
                                title: scene_item.title.clone(),
                                synopsis: None,
                                prose: None,
                                position: scene_pos,
                                source_id: Some(scene_item.uuid.clone()),
                                archived: false,
                                locked: false,
                                scene_type: Default::default(),
                                scene_status: Default::default(),
                                planning_status: Default::default(),
                                editor_mode: Default::default(),
                            });
                            scene_pos += 1;
                        }
                    }

                    chapters.push(chapter);
                    *position += 1;
                }
            }
            "Text" => {
                let chapter = Chapter {
                    id: uuid::Uuid::new_v4(),
                    project_id,
                    title: child.title.clone(),
                    synopsis: None,
                    position: *position,
                    is_part: false,
                    archived: false,
                    locked: false,
                    source_id: Some(child.uuid.clone()),
                    planning_status: Default::default(),
                };

                let prose = read_rtf_content(data_dir, &child.uuid);
                let scene_id = uuid::Uuid::new_v4();

                if let Some(ref prose_html) = prose {
                    let mut beat = Beat::new(scene_id, "Scene Content".to_string(), 0)
                        .with_source_id(Some(format!("{}-prose", child.uuid)));
                    beat.prose = Some(prose_html.clone());
                    beats.push(beat);
                }

                scenes.push(Scene {
                    id: scene_id,
                    chapter_id: chapter.id,
                    title: child.title.clone(),
                    synopsis: None,
                    prose: None,
                    position: 0,
                    source_id: Some(child.uuid.clone()),
                    archived: false,
                    locked: false,
                    scene_type: Default::default(),
                    scene_status: Default::default(),
                    planning_status: Default::default(),
                    editor_mode: Default::default(),
                });

                chapters.push(chapter);
                *position += 1;
            }
            _ => {}
        }
    }
}

/// Parse a .scriv bundle directory into Kindling data structures
pub fn parse_scrivener_bundle(
    scriv_path: &std::path::Path,
) -> Result<ParsedScrivener, ScrivenerError> {
    use crate::models::{Chapter, Project, Scene, SourceType};

    // Find and read the .scrivx file
    let scrivx_path = find_scrivx_in_bundle(scriv_path)?;
    let xml = std::fs::read_to_string(&scrivx_path).map_err(ScrivenerError::IoError)?;
    let doc = parse_scrivx(&xml)?;

    // Extract project name from the .scriv directory name
    let project_name = scriv_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Scrivener Project")
        .to_string();

    let data_dir = scriv_path.join("Files").join("Data");

    let mut project = Project::new(
        project_name,
        SourceType::Scrivener,
        Some(scriv_path.to_string_lossy().to_string()),
    );

    let mut chapters: Vec<Chapter> = Vec::new();
    let mut scenes: Vec<Scene> = Vec::new();
    let mut beats: Vec<crate::models::Beat> = Vec::new();

    // Find the Draft folder in the binder
    let draft = doc
        .binder
        .iter()
        .find(|item| item.item_type == "DraftFolder");

    if let Some(draft_folder) = draft {
        // Check for Kindling project type metadata on the DraftFolder
        if let Some(ref pt) = draft_folder.kindling_project_type {
            project.project_type = pt.clone();
        }

        let mut position: i32 = 0;
        process_binder_children(
            &draft_folder.children,
            project.id,
            &data_dir,
            &mut chapters,
            &mut scenes,
            &mut beats,
            &mut position,
        );
    }

    // If no explicit project type was set, use content-based detection:
    // any scene title matching INT./EXT. slugline pattern → screenplay
    if project.project_type == "novel" {
        let has_sluglines = scenes.iter().any(|s| {
            let upper = s.title.trim().to_uppercase();
            upper.starts_with("INT.") || upper.starts_with("EXT.")
        });
        if has_sluglines {
            project.project_type = "screenplay".to_string();
        }
    }

    Ok(ParsedScrivener {
        project,
        chapters,
        scenes,
        beats,
    })
}

/// Find the .scrivx file inside a .scriv bundle
fn find_scrivx_in_bundle(
    scriv_path: &std::path::Path,
) -> Result<std::path::PathBuf, ScrivenerError> {
    let entries = std::fs::read_dir(scriv_path).map_err(ScrivenerError::IoError)?;

    for entry in entries.flatten() {
        if let Some(ext) = entry.path().extension() {
            if ext == "scrivx" {
                return Ok(entry.path());
            }
        }
    }

    Err(ScrivenerError::InvalidStructure(
        "No .scrivx file found in the .scriv bundle".to_string(),
    ))
}

/// Read and convert RTF content for a Scrivener document
fn read_rtf_content(data_dir: &std::path::Path, uuid: &str) -> Option<String> {
    let rtf_path = data_dir.join(uuid).join("content.rtf");
    if rtf_path.exists() {
        if let Ok(rtf) = std::fs::read_to_string(&rtf_path) {
            let html = rtf_to_html(&rtf);
            if html.is_empty() {
                None
            } else {
                Some(html)
            }
        } else {
            None
        }
    } else {
        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_rtf_simple_paragraph() {
        let rtf = html_to_rtf("<p>Hello world.</p>");
        assert!(rtf.starts_with(r"{\rtf1"));
        assert!(rtf.contains("Hello world."));
        assert!(rtf.ends_with('}'));
    }

    #[test]
    fn test_html_to_rtf_bold() {
        let rtf = html_to_rtf("<p><strong>Bold</strong> text</p>");
        assert!(rtf.contains("\\b Bold"));
        assert!(rtf.contains("\\b0  text"));
    }

    #[test]
    fn test_html_to_rtf_italic() {
        let rtf = html_to_rtf("<p><em>Italic</em> text</p>");
        assert!(rtf.contains("\\i Italic"));
        assert!(rtf.contains("\\i0  text"));
    }

    #[test]
    fn test_html_to_rtf_underline() {
        let rtf = html_to_rtf("<p><u>Underlined</u></p>");
        assert!(rtf.contains("\\ul Underlined"));
        assert!(rtf.contains("\\ulnone"));
    }

    #[test]
    fn test_html_to_rtf_multiple_paragraphs() {
        let rtf = html_to_rtf("<p>First.</p><p>Second.</p>");
        assert!(rtf.contains("First."));
        assert!(rtf.contains("\\par"));
        assert!(rtf.contains("Second."));
    }

    #[test]
    fn test_html_to_rtf_heading() {
        let rtf = html_to_rtf("<h1>Title</h1><p>Body</p>");
        assert!(rtf.contains("\\fs36"));
        assert!(rtf.contains("Title"));
    }

    #[test]
    fn test_html_to_rtf_special_chars() {
        let rtf = html_to_rtf("<p>Curly {braces} and \\backslash</p>");
        assert!(rtf.contains("Curly \\{braces\\}"));
        assert!(rtf.contains("\\\\backslash"));
    }

    #[test]
    fn test_html_to_rtf_entities() {
        let rtf = html_to_rtf("<p>&amp; &lt; &gt;</p>");
        assert!(rtf.contains("& < >"));
    }

    #[test]
    fn test_html_to_rtf_empty() {
        let rtf = html_to_rtf("");
        assert!(rtf.starts_with(r"{\rtf1"));
        assert!(rtf.ends_with('}'));
    }

    #[test]
    fn test_html_to_rtf_blockquote() {
        let rtf = html_to_rtf("<blockquote><p>Quoted text</p></blockquote>");
        assert!(rtf.contains("\\li720"));
        assert!(rtf.contains("Quoted text"));
    }

    #[test]
    fn test_html_to_rtf_line_break() {
        let rtf = html_to_rtf("<p>Line one<br>Line two</p>");
        assert!(rtf.contains("Line one\\line Line two"));
    }

    #[test]
    fn test_parse_scrivx_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ScrivenerProject Identifier="ABC-123" Version="2.0">
  <Binder>
    <BinderItem UUID="DRAFT-1" Type="DraftFolder" Created="2024-01-01" Modified="2024-01-01">
      <Title>Draft</Title>
      <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
      <Children>
        <BinderItem UUID="CH-1" Type="Folder" Created="2024-01-01" Modified="2024-01-01">
          <Title>Chapter 1</Title>
          <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
          <Children>
            <BinderItem UUID="SC-1" Type="Text" Created="2024-01-01" Modified="2024-01-01">
              <Title>Scene 1</Title>
              <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
            </BinderItem>
          </Children>
        </BinderItem>
      </Children>
    </BinderItem>
  </Binder>
</ScrivenerProject>"#;

        let doc = parse_scrivx(xml).unwrap();
        assert_eq!(doc.project_id, "ABC-123");
        assert_eq!(doc.binder.len(), 1);
        assert_eq!(doc.binder[0].title, "Draft");
        assert_eq!(doc.binder[0].item_type, "DraftFolder");
        assert_eq!(doc.binder[0].children.len(), 1);
        assert_eq!(doc.binder[0].children[0].title, "Chapter 1");
        assert_eq!(doc.binder[0].children[0].children.len(), 1);
        assert_eq!(doc.binder[0].children[0].children[0].title, "Scene 1");
        assert_eq!(doc.binder[0].children[0].children[0].uuid, "SC-1");
    }

    #[test]
    fn test_collect_text_documents() {
        let items = vec![BinderItem {
            uuid: "DRAFT".to_string(),
            item_type: "DraftFolder".to_string(),
            title: "Draft".to_string(),
            created: String::new(),
            modified: String::new(),
            include_in_compile: true,
            kindling_project_type: None,
            children: vec![
                BinderItem {
                    uuid: "SC-1".to_string(),
                    item_type: "Text".to_string(),
                    title: "Scene 1".to_string(),
                    created: String::new(),
                    modified: String::new(),
                    include_in_compile: true,
                    kindling_project_type: None,
                    children: vec![],
                },
                BinderItem {
                    uuid: "CH-1".to_string(),
                    item_type: "Folder".to_string(),
                    title: "Chapter".to_string(),
                    created: String::new(),
                    modified: String::new(),
                    include_in_compile: true,
                    kindling_project_type: None,
                    children: vec![BinderItem {
                        uuid: "SC-2".to_string(),
                        item_type: "Text".to_string(),
                        title: "Scene 2".to_string(),
                        created: String::new(),
                        modified: String::new(),
                        include_in_compile: true,
                        kindling_project_type: None,
                        children: vec![],
                    }],
                },
            ],
        }];

        let docs = collect_text_documents(&items);
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].title, "Scene 1");
        assert_eq!(docs[1].title, "Scene 2");
    }

    #[test]
    fn test_generate_scrivx_roundtrip() {
        let chapters = vec![ExportChapter {
            uuid: "CH-UUID-1".to_string(),
            title: "Chapter One".to_string(),
            is_part: false,
            created: "2024-01-01 00:00:00 +0000".to_string(),
            modified: "2024-01-01 00:00:00 +0000".to_string(),
            scenes: vec![ExportScene {
                uuid: "SC-UUID-1".to_string(),
                title: "Opening Scene".to_string(),
                created: "2024-01-01 00:00:00 +0000".to_string(),
                modified: "2024-01-01 00:00:00 +0000".to_string(),
            }],
            children: Vec::new(),
        }];

        let xml = generate_scrivx("Test Project", &chapters, "novel").unwrap();
        let doc = parse_scrivx(&xml).unwrap();

        assert_eq!(doc.binder.len(), 3); // Draft, Research, Trash
        let draft = &doc.binder[0];
        assert_eq!(draft.item_type, "DraftFolder");
        assert_eq!(draft.children.len(), 1);
        assert_eq!(draft.children[0].title, "Chapter One");
        assert_eq!(draft.children[0].item_type, "Folder");
        assert_eq!(draft.children[0].children.len(), 1);
        assert_eq!(draft.children[0].children[0].title, "Opening Scene");
    }

    #[test]
    fn test_generate_scrivx_preserves_project_type() {
        let chapters = vec![ExportChapter {
            uuid: "CH-UUID-1".to_string(),
            title: "Chapter One".to_string(),
            is_part: false,
            created: "2024-01-01 00:00:00 +0000".to_string(),
            modified: "2024-01-01 00:00:00 +0000".to_string(),
            scenes: Vec::new(),
            children: Vec::new(),
        }];

        let xml = generate_scrivx("Test", &chapters, "screenplay").unwrap();
        let doc = parse_scrivx(&xml).unwrap();
        let draft = &doc.binder[0];
        assert_eq!(draft.kindling_project_type.as_deref(), Some("screenplay"));
    }

    #[test]
    fn test_generate_scrivx_part_hierarchy() {
        let chapters = vec![ExportChapter {
            uuid: "PART-UUID".to_string(),
            title: "Act I".to_string(),
            is_part: true,
            created: "2024-01-01 00:00:00 +0000".to_string(),
            modified: "2024-01-01 00:00:00 +0000".to_string(),
            scenes: Vec::new(),
            children: vec![ExportChapter {
                uuid: "CH-UUID-1".to_string(),
                title: "Sequence 1".to_string(),
                is_part: false,
                created: "2024-01-01 00:00:00 +0000".to_string(),
                modified: "2024-01-01 00:00:00 +0000".to_string(),
                scenes: vec![ExportScene {
                    uuid: "SC-UUID-1".to_string(),
                    title: "Scene 1".to_string(),
                    created: "2024-01-01 00:00:00 +0000".to_string(),
                    modified: "2024-01-01 00:00:00 +0000".to_string(),
                }],
                children: Vec::new(),
            }],
        }];

        let xml = generate_scrivx("Test", &chapters, "screenplay").unwrap();
        let doc = parse_scrivx(&xml).unwrap();
        let draft = &doc.binder[0];

        // Act I should be a Folder (not Text)
        assert_eq!(draft.children.len(), 1);
        assert_eq!(draft.children[0].title, "Act I");
        assert_eq!(draft.children[0].item_type, "Folder");

        // Sequence 1 should be a nested Folder under Act I
        assert_eq!(draft.children[0].children.len(), 1);
        assert_eq!(draft.children[0].children[0].title, "Sequence 1");
        assert_eq!(draft.children[0].children[0].item_type, "Folder");

        // Scene 1 should be a Text under Sequence 1
        assert_eq!(draft.children[0].children[0].children.len(), 1);
        assert_eq!(draft.children[0].children[0].children[0].title, "Scene 1");
        assert_eq!(draft.children[0].children[0].children[0].item_type, "Text");
    }

    #[test]
    fn test_escape_rtf_special_chars() {
        assert_eq!(escape_rtf("\\"), "\\\\");
        assert_eq!(escape_rtf("{"), "\\{");
        assert_eq!(escape_rtf("}"), "\\}");
        assert_eq!(escape_rtf("abc"), "abc");
    }

    #[test]
    fn test_decode_entities() {
        assert_eq!(decode_entity("&amp;"), "&");
        assert_eq!(decode_entity("&lt;"), "<");
        assert_eq!(decode_entity("&gt;"), ">");
        assert_eq!(decode_entity("&mdash;"), "\u{2014}");
    }

    // =========================================================================
    // RTF → HTML tests
    // =========================================================================

    #[test]
    fn test_rtf_to_html_plain_text() {
        let rtf = r"{\rtf1\ansi\deff0{\fonttbl{\f0 Times New Roman;}}Hello world.}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<p>Hello world.</p>"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_bold() {
        let rtf = r"{\rtf1\ansi \b Bold\b0  normal}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<strong>Bold</strong>"), "got: {html}");
        assert!(html.contains("normal"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_italic() {
        let rtf = r"{\rtf1\ansi \i Italic\i0  text}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<em>Italic</em>"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_underline() {
        let rtf = r"{\rtf1\ansi \ul Underlined\ulnone  text}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<u>Underlined</u>"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_combined_formatting() {
        let rtf = r"{\rtf1\ansi \b\i Bold italic\i0\b0  plain}";
        let html = rtf_to_html(rtf);
        assert!(
            html.contains("<strong><em>Bold italic</em></strong>"),
            "got: {html}"
        );
    }

    #[test]
    fn test_rtf_to_html_multiple_paragraphs() {
        let rtf = r"{\rtf1\ansi First paragraph.\par Second paragraph.}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<p>First paragraph.</p>"), "got: {html}");
        assert!(html.contains("<p>Second paragraph.</p>"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_unicode() {
        let rtf = r"{\rtf1\ansi Smart \u8220?quote\u8221?}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("\u{201C}"), "got: {html}");
        assert!(html.contains("\u{201D}"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_hex_char() {
        let rtf = r"{\rtf1\ansi caf\'e9}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("caf\u{e9}"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_escaped_braces() {
        let rtf = r"{\rtf1\ansi Open \{ and close \}}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("{"), "got: {html}");
        assert!(html.contains("}"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_empty() {
        let rtf = r"{\rtf1\ansi}";
        let html = rtf_to_html(rtf);
        assert_eq!(html, "");
    }

    #[test]
    fn test_rtf_to_html_pard_resets_formatting() {
        let rtf = r"{\rtf1\ansi \b Bold\pard  plain}";
        let html = rtf_to_html(rtf);
        assert!(html.contains("<strong>Bold</strong>"), "got: {html}");
        assert!(html.contains("plain"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_skips_fonttbl() {
        let rtf = r"{\rtf1\ansi{\fonttbl{\f0\froman Times New Roman;}{\f1\fswiss Arial;}}Hello.}";
        let html = rtf_to_html(rtf);
        assert!(!html.contains("Times"), "got: {html}");
        assert!(html.contains("Hello."), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_skips_colortbl() {
        let rtf = r"{\rtf1\ansi{\colortbl;\red0\green0\blue0;\red255\green0\blue0;}Some text.}";
        let html = rtf_to_html(rtf);
        assert!(!html.contains("red"), "got: {html}");
        assert!(html.contains("Some text."), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_li720_blockquote() {
        // Mirrors Kindling's html_to_rtf blockquote output: \pard\li720 ...
        let rtf = r"{\rtf1\ansi\pard\li720\i Quoted line\i0\par}";
        let html = rtf_to_html(rtf);
        assert!(
            html.contains("<blockquote>") && html.contains("</blockquote>"),
            "expected blockquote wrapper, got: {html}"
        );
        assert!(html.contains("Quoted line"), "got: {html}");
    }

    #[test]
    fn test_rtf_to_html_li360_not_blockquote() {
        let rtf = r"{\rtf1\ansi\pard\li360 Indented\par}";
        let html = rtf_to_html(rtf);
        assert!(
            !html.contains("<blockquote>"),
            "li360 should stay plain paragraph, got: {html}"
        );
        assert!(html.contains("Indented"), "got: {html}");
    }

    #[test]
    fn test_html_rtf_html_blockquote_roundtrip() {
        let original =
            "<p>Before</p><blockquote><p>My dearest Eleanor,</p></blockquote><p>After</p>";
        let rtf = html_to_rtf(original);
        let html = rtf_to_html(&rtf);
        assert!(
            html.contains("<blockquote>"),
            "blockquote lost in round-trip, got: {html}"
        );
        assert!(
            html.contains("My dearest Eleanor,"),
            "quoted text lost, got: {html}"
        );
        assert!(html.contains("Before"), "got: {html}");
        assert!(html.contains("After"), "got: {html}");
    }

    // =========================================================================
    // Bundle parser tests (filesystem)
    // =========================================================================

    #[test]
    fn test_parse_scrivener_bundle_basic() {
        let dir = tempfile::tempdir().unwrap();
        let scriv = dir.path().join("Test.scriv");
        std::fs::create_dir_all(&scriv).unwrap();

        let scrivx = r#"<?xml version="1.0" encoding="UTF-8"?>
<ScrivenerProject Identifier="TEST-1" Version="2.0">
  <Binder>
    <BinderItem UUID="DRAFT" Type="DraftFolder" Created="2024-01-01" Modified="2024-01-01">
      <Title>Draft</Title>
      <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
      <Children>
        <BinderItem UUID="CH1" Type="Folder" Created="2024-01-01" Modified="2024-01-01">
          <Title>Chapter One</Title>
          <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
          <Children>
            <BinderItem UUID="SC1" Type="Text" Created="2024-01-01" Modified="2024-01-01">
              <Title>Opening</Title>
              <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
            </BinderItem>
            <BinderItem UUID="SC2" Type="Text" Created="2024-01-01" Modified="2024-01-01">
              <Title>Middle</Title>
              <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
            </BinderItem>
          </Children>
        </BinderItem>
      </Children>
    </BinderItem>
  </Binder>
</ScrivenerProject>"#;

        std::fs::write(scriv.join("Test.scrivx"), scrivx).unwrap();

        let data = scriv.join("Files").join("Data");
        std::fs::create_dir_all(data.join("SC1")).unwrap();
        std::fs::create_dir_all(data.join("SC2")).unwrap();
        std::fs::write(
            data.join("SC1").join("content.rtf"),
            r"{\rtf1\ansi It was a dark and stormy night.}",
        )
        .unwrap();
        std::fs::write(
            data.join("SC2").join("content.rtf"),
            r"{\rtf1\ansi The wind howled.}",
        )
        .unwrap();

        let parsed = parse_scrivener_bundle(&scriv).unwrap();
        assert_eq!(parsed.project.name, "Test");
        assert_eq!(parsed.chapters.len(), 1);
        assert_eq!(parsed.chapters[0].title, "Chapter One");
        assert_eq!(parsed.scenes.len(), 2);
        assert_eq!(parsed.scenes[0].title, "Opening");
        assert!(
            parsed.scenes[0].prose.is_none(),
            "prose should be on beats, not scenes"
        );
        assert_eq!(parsed.beats.len(), 2);
        assert!(
            parsed.beats[0]
                .prose
                .as_ref()
                .unwrap()
                .contains("dark and stormy"),
            "beat prose: {:?}",
            parsed.beats[0].prose
        );
        assert_eq!(parsed.beats[0].scene_id, parsed.scenes[0].id);
        assert_eq!(parsed.beats[1].scene_id, parsed.scenes[1].id);
        assert_eq!(parsed.scenes[1].title, "Middle");
        assert_eq!(parsed.scenes[0].source_id.as_deref(), Some("SC1"));
        assert_eq!(parsed.scenes[1].source_id.as_deref(), Some("SC2"));
    }

    #[test]
    fn test_parse_scrivener_bundle_top_level_text() {
        let dir = tempfile::tempdir().unwrap();
        let scriv = dir.path().join("TopLevel.scriv");
        std::fs::create_dir_all(&scriv).unwrap();

        let scrivx = r#"<?xml version="1.0" encoding="UTF-8"?>
<ScrivenerProject Identifier="TOP-1" Version="2.0">
  <Binder>
    <BinderItem UUID="DRAFT" Type="DraftFolder" Created="2024-01-01" Modified="2024-01-01">
      <Title>Draft</Title>
      <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
      <Children>
        <BinderItem UUID="TXT1" Type="Text" Created="2024-01-01" Modified="2024-01-01">
          <Title>Standalone Scene</Title>
          <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
        </BinderItem>
      </Children>
    </BinderItem>
  </Binder>
</ScrivenerProject>"#;

        std::fs::write(scriv.join("TopLevel.scrivx"), scrivx).unwrap();

        let data = scriv.join("Files").join("Data");
        std::fs::create_dir_all(data.join("TXT1")).unwrap();
        std::fs::write(
            data.join("TXT1").join("content.rtf"),
            r"{\rtf1\ansi A lonely scene.}",
        )
        .unwrap();

        let parsed = parse_scrivener_bundle(&scriv).unwrap();
        assert_eq!(parsed.chapters.len(), 1);
        assert_eq!(parsed.chapters[0].title, "Standalone Scene");
        assert_eq!(parsed.scenes.len(), 1);
        assert_eq!(parsed.scenes[0].title, "Standalone Scene");
        assert!(parsed.scenes[0].prose.is_none());
        assert_eq!(parsed.beats.len(), 1);
        assert!(parsed.beats[0]
            .prose
            .as_ref()
            .unwrap()
            .contains("lonely scene"));
        assert_eq!(parsed.beats[0].scene_id, parsed.scenes[0].id);
    }

    #[test]
    fn test_parse_scrivener_bundle_no_scrivx() {
        let dir = tempfile::tempdir().unwrap();
        let scriv = dir.path().join("Empty.scriv");
        std::fs::create_dir_all(&scriv).unwrap();

        let result = parse_scrivener_bundle(&scriv);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_scrivener_bundle_no_rtf() {
        let dir = tempfile::tempdir().unwrap();
        let scriv = dir.path().join("NoContent.scriv");
        std::fs::create_dir_all(&scriv).unwrap();

        let scrivx = r#"<?xml version="1.0" encoding="UTF-8"?>
<ScrivenerProject Identifier="NC-1" Version="2.0">
  <Binder>
    <BinderItem UUID="DRAFT" Type="DraftFolder" Created="2024-01-01" Modified="2024-01-01">
      <Title>Draft</Title>
      <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
      <Children>
        <BinderItem UUID="SC-EMPTY" Type="Text" Created="2024-01-01" Modified="2024-01-01">
          <Title>Empty Scene</Title>
          <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
        </BinderItem>
      </Children>
    </BinderItem>
  </Binder>
</ScrivenerProject>"#;

        std::fs::write(scriv.join("NoContent.scrivx"), scrivx).unwrap();

        let parsed = parse_scrivener_bundle(&scriv).unwrap();
        assert_eq!(parsed.scenes.len(), 1);
        assert!(parsed.scenes[0].prose.is_none());
    }

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("a & b < c > d"), "a &amp; b &lt; c &gt; d");
    }
}
