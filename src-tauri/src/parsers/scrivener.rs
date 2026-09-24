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

/// Windows-1252 code points for bytes 0x80–0x9F, the only range where cp1252
/// differs from Latin-1. `None` marks the five bytes cp1252 leaves undefined.
const CP1252_80_9F: [Option<char>; 32] = [
    Some('\u{20AC}'), // 0x80 euro sign
    None,             // 0x81
    Some('\u{201A}'), // 0x82 single low-9 quote
    Some('\u{0192}'), // 0x83 f with hook
    Some('\u{201E}'), // 0x84 double low-9 quote
    Some('\u{2026}'), // 0x85 ellipsis
    Some('\u{2020}'), // 0x86 dagger
    Some('\u{2021}'), // 0x87 double dagger
    Some('\u{02C6}'), // 0x88 circumflex
    Some('\u{2030}'), // 0x89 per mille
    Some('\u{0160}'), // 0x8A S caron
    Some('\u{2039}'), // 0x8B single left angle quote
    Some('\u{0152}'), // 0x8C OE ligature
    None,             // 0x8D
    Some('\u{017D}'), // 0x8E Z caron
    None,             // 0x8F
    None,             // 0x90
    Some('\u{2018}'), // 0x91 left single quote
    Some('\u{2019}'), // 0x92 right single quote / apostrophe
    Some('\u{201C}'), // 0x93 left double quote
    Some('\u{201D}'), // 0x94 right double quote
    Some('\u{2022}'), // 0x95 bullet
    Some('\u{2013}'), // 0x96 en dash
    Some('\u{2014}'), // 0x97 em dash
    Some('\u{02DC}'), // 0x98 small tilde
    Some('\u{2122}'), // 0x99 trade mark
    Some('\u{0161}'), // 0x9A s caron
    Some('\u{203A}'), // 0x9B single right angle quote
    Some('\u{0153}'), // 0x9C oe ligature
    None,             // 0x9D
    Some('\u{017E}'), // 0x9E z caron
    Some('\u{0178}'), // 0x9F Y diaeresis
];

/// The RTF default ANSI code page, and what Scrivener writes (`\ansicpg1252`).
const CODEPAGE_CP1252: u32 = 1252;

/// `\ansicpg` value for ISO-8859-1, whose 0x80–0x9F bytes are C1 controls.
const CODEPAGE_LATIN1: u32 = 28591;

/// Decode one byte of 8-bit RTF text (a `\'hh` escape or a raw byte).
///
/// Only Windows-1252 and Latin-1 are decoded exactly. Other single-byte code
/// pages fall back to cp1252, which is right for ASCII and wrong for their
/// upper half. Control bytes carry no text and are dropped, except tab.
fn decode_ansi_byte(byte: u8, codepage: u32) -> Option<char> {
    match byte {
        b'\t' => Some('\t'),
        0x00..=0x1F | 0x7F => None,
        0x80..=0x9F if codepage == CODEPAGE_LATIN1 => None,
        0x80..=0x9F => CP1252_80_9F[usize::from(byte - 0x80)],
        _ => Some(char::from(byte)),
    }
}

/// Decode an RTF file's bytes to text.
///
/// RTF is nominally 7-bit, but some writers emit raw 8-bit bytes in the
/// document code page. Those files are not valid UTF-8, so decode them as
/// cp1252 instead of discarding the whole document.
fn decode_rtf_bytes(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(err) => err
            .into_bytes()
            .into_iter()
            .filter_map(|b| match b {
                b'\r' | b'\n' => Some(char::from(b)),
                _ => decode_ansi_byte(b, CODEPAGE_CP1252),
            })
            .collect(),
    }
}

/// Destination groups whose content is never document prose. Their text
/// (font names, image hex, header text, list numbering templates) must not
/// leak into scenes. Groups introduced by `\*` are skipped regardless of name.
const RTF_SKIPPED_DESTINATIONS: &[&str] = &[
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "pict",
    "shppict",
    "nonshppict",
    "object",
    "objdata",
    "NeXTGraphic",
    "fldinst",
    "header",
    "headerl",
    "headerr",
    "headerf",
    "footer",
    "footerl",
    "footerr",
    "footerf",
    "listtable",
    "listoverridetable",
    "revtbl",
    "rsidtbl",
    "filetbl",
    "xmlnstbl",
    "themedata",
    "colorschememapping",
    "latentstyles",
    "datastore",
    "generator",
    "pgdsctbl",
    "pn",
    "pntxta",
    "pntxtb",
];

type RtfChars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

/// One RTF control sequence, read after its leading backslash.
enum RtfControl {
    /// Control word with its optional numeric parameter.
    Word(String, Option<i32>),
    /// `\'hh` hex-escaped byte (`None` if the hex digits are malformed).
    Hex(Option<u8>),
    /// Control symbol: backslash followed by one non-letter character.
    Symbol(char),
}

/// Read a control sequence; `chars` is positioned just after the backslash.
fn read_rtf_control(chars: &mut RtfChars<'_>) -> Option<RtfControl> {
    let first = *chars.peek()?;
    if first == '\'' {
        chars.next();
        let mut hex = String::new();
        for _ in 0..2 {
            match chars.peek() {
                Some(hc) if hc.is_ascii_hexdigit() => {
                    hex.push(*hc);
                    chars.next();
                }
                _ => break,
            }
        }
        return Some(RtfControl::Hex(u8::from_str_radix(&hex, 16).ok()));
    }
    if !first.is_ascii_alphabetic() {
        chars.next();
        return Some(RtfControl::Symbol(first));
    }

    let mut word = String::new();
    while let Some(&wc) = chars.peek() {
        if !wc.is_ascii_alphabetic() {
            break;
        }
        word.push(wc);
        chars.next();
    }
    let mut digits = String::new();
    if let Some(&pc) = chars.peek() {
        if pc == '-' || pc.is_ascii_digit() {
            digits.push(pc);
            chars.next();
            while let Some(&dc) = chars.peek() {
                if !dc.is_ascii_digit() {
                    break;
                }
                digits.push(dc);
                chars.next();
            }
        }
    }
    // A single space delimits the control word and is not document text.
    if chars.peek() == Some(&' ') {
        chars.next();
    }
    let param = digits
        .parse::<i64>()
        .ok()
        .and_then(|p| i32::try_from(p.clamp(i64::from(i32::MIN), i64::from(i32::MAX))).ok());
    Some(RtfControl::Word(word, param))
}

/// Skip the payload of `\binN`: N raw characters that may contain braces.
fn skip_rtf_binary(chars: &mut RtfChars<'_>, param: Option<i32>) {
    for _ in 0..param.unwrap_or(0).max(0) {
        if chars.next().is_none() {
            break;
        }
    }
}

/// Whether the group opened just before `chars` is a destination whose
/// content should be skipped (see `RTF_SKIPPED_DESTINATIONS`).
fn is_skipped_rtf_destination(chars: &RtfChars<'_>) -> bool {
    let mut peek = chars.clone();
    if peek.next() != Some('\\') {
        return false;
    }
    if peek.peek() == Some(&'*') {
        return true;
    }
    match read_rtf_control(&mut peek) {
        Some(RtfControl::Word(word, _)) => RTF_SKIPPED_DESTINATIONS.contains(&word.as_str()),
        _ => false,
    }
}

/// Character formatting, which RTF scopes to the enclosing `{...}` group.
#[derive(Clone, Copy)]
struct RtfCharFormat {
    bold: bool,
    italic: bool,
    underline: bool,
    /// `\ucN`: how many fallback characters follow each `\uN`.
    unicode_skip: usize,
}

impl Default for RtfCharFormat {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            unicode_skip: 1,
        }
    }
}

/// Accumulates decoded RTF text into TipTap-compatible HTML.
#[derive(Default)]
struct RtfHtmlWriter {
    html: String,
    text_buf: String,
    in_paragraph: bool,
    blockquote_active: bool,
    li_twips: i32,
    /// Formatting of the text currently in `text_buf`.
    fmt: RtfCharFormat,
}

impl RtfHtmlWriter {
    fn open_paragraph(&mut self) {
        if self.in_paragraph {
            return;
        }
        if self.blockquote_active && self.li_twips < BLOCKQUOTE_LI_THRESHOLD {
            self.html.push_str("</blockquote>");
            self.blockquote_active = false;
        }
        if !self.blockquote_active && self.li_twips >= BLOCKQUOTE_LI_THRESHOLD {
            self.html.push_str("<blockquote>");
            self.blockquote_active = true;
        }
        self.html.push_str("<p>");
        self.in_paragraph = true;
    }

    /// Flush accumulated text as a self-contained formatted run.
    fn flush(&mut self) {
        if self.text_buf.is_empty() {
            return;
        }
        self.open_paragraph();
        let fmt = self.fmt;
        if fmt.bold {
            self.html.push_str("<strong>");
        }
        if fmt.italic {
            self.html.push_str("<em>");
        }
        if fmt.underline {
            self.html.push_str("<u>");
        }
        self.html.push_str(&html_escape(&self.text_buf));
        self.text_buf.clear();
        if fmt.underline {
            self.html.push_str("</u>");
        }
        if fmt.italic {
            self.html.push_str("</em>");
        }
        if fmt.bold {
            self.html.push_str("</strong>");
        }
    }

    /// Change character formatting, closing the current run if it differs.
    fn set_format(&mut self, fmt: RtfCharFormat) {
        if fmt.bold != self.fmt.bold
            || fmt.italic != self.fmt.italic
            || fmt.underline != self.fmt.underline
        {
            self.flush();
        }
        self.fmt = fmt;
    }

    fn set_left_indent(&mut self, twips: i32) {
        self.flush();
        self.li_twips = twips.max(0);
    }

    fn end_paragraph(&mut self) {
        self.flush();
        if self.in_paragraph {
            self.html.push_str("</p>");
            self.in_paragraph = false;
        }
    }

    fn line_break(&mut self) {
        self.flush();
        self.open_paragraph();
        self.html.push_str("<br>");
    }

    fn push_char(&mut self, ch: char) {
        match ch {
            // Cocoa writes Shift-Return as U+2028 LINE SEPARATOR.
            '\u{2028}' => self.line_break(),
            '\u{2029}' => self.end_paragraph(),
            _ => self.text_buf.push(ch),
        }
    }

    fn finish(mut self) -> String {
        self.end_paragraph();
        if self.blockquote_active {
            self.html.push_str("</blockquote>");
        }
        self.html
    }
}

/// Decode a `\uN` parameter, pairing UTF-16 surrogates written as two words.
fn decode_rtf_unicode(param: i32, high_surrogate: &mut Option<u32>) -> Option<char> {
    // Code points above 32767 are written as negative 16-bit values.
    let code = u32::try_from(if param < 0 { param + 65536 } else { param }).ok()?;
    match (high_surrogate.take(), code) {
        (_, 0xD800..=0xDBFF) => {
            *high_surrogate = Some(code);
            None
        }
        (Some(high), 0xDC00..=0xDFFF) => {
            char::from_u32(0x10000 + ((high - 0xD800) << 10) + (code - 0xDC00))
        }
        _ => char::from_u32(code),
    }
}

/// Convert Scrivener RTF content to TipTap-compatible HTML.
///
/// Each text run is self-contained with its own formatting tags, so changes
/// to bold/italic/underline mid-paragraph produce correct HTML.
pub fn rtf_to_html(rtf: &str) -> String {
    let mut out = RtfHtmlWriter::default();
    let mut fmt = RtfCharFormat::default();
    let mut group_stack: Vec<RtfCharFormat> = Vec::new();
    let mut skip_depth: u32 = 0;
    let mut codepage = CODEPAGE_CP1252;
    // ANSI fallback characters still to discard after a `\uN` (per `\ucN`).
    let mut unicode_fallback: usize = 0;
    let mut high_surrogate: Option<u32> = None;
    let mut chars = rtf.chars().peekable();

    while let Some(c) = chars.next() {
        if skip_depth > 0 {
            match c {
                '{' => skip_depth += 1,
                '}' => skip_depth -= 1,
                // Consume escapes whole so `\{`, `\}` and `\bin` data cannot
                // unbalance the brace count.
                '\\' => {
                    if let Some(RtfControl::Word(word, param)) = read_rtf_control(&mut chars) {
                        if word == "bin" {
                            skip_rtf_binary(&mut chars, param);
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        match c {
            '{' => {
                unicode_fallback = 0;
                if is_skipped_rtf_destination(&chars) {
                    skip_depth = 1;
                } else {
                    group_stack.push(fmt);
                }
                continue;
            }
            '}' => {
                unicode_fallback = 0;
                if let Some(outer) = group_stack.pop() {
                    fmt = outer;
                    out.set_format(fmt);
                }
                continue;
            }
            // Literal line breaks in RTF source are layout, not text.
            '\r' | '\n' => continue,
            '\\' => {}
            _ => {
                if unicode_fallback > 0 {
                    unicode_fallback -= 1;
                } else {
                    out.push_char(c);
                }
                continue;
            }
        }

        let Some(control) = read_rtf_control(&mut chars) else {
            continue;
        };
        if unicode_fallback > 0 {
            // Part of the ANSI fallback for the preceding `\uN`; each control
            // sequence counts as one character.
            unicode_fallback -= 1;
            if let RtfControl::Word(word, param) = &control {
                if word == "bin" {
                    skip_rtf_binary(&mut chars, *param);
                }
            }
            continue;
        }

        match control {
            RtfControl::Hex(byte) => {
                if let Some(ch) = byte.and_then(|b| decode_ansi_byte(b, codepage)) {
                    out.push_char(ch);
                }
            }
            RtfControl::Symbol(sym) => match sym {
                '\\' | '{' | '}' => out.push_char(sym),
                // Cocoa (macOS, so Mac Scrivener) writes each paragraph break
                // as a backslash followed by a newline.
                '\n' | '\r' => out.end_paragraph(),
                '~' => out.push_char('\u{00A0}'),
                '_' => out.push_char('\u{2011}'),
                // `\-` optional hyphen, stray `\*`, `\:` and friends.
                _ => {}
            },
            RtfControl::Word(word, param) => {
                let on = param != Some(0);
                match word.as_str() {
                    "par" | "sect" | "page" => out.end_paragraph(),
                    "line" => out.line_break(),
                    "tab" => out.push_char('\t'),
                    "emdash" => out.push_char('\u{2014}'),
                    "endash" => out.push_char('\u{2013}'),
                    "lquote" => out.push_char('\u{2018}'),
                    "rquote" => out.push_char('\u{2019}'),
                    "ldblquote" => out.push_char('\u{201C}'),
                    "rdblquote" => out.push_char('\u{201D}'),
                    "bullet" => out.push_char('\u{2022}'),
                    "emspace" => out.push_char('\u{2003}'),
                    "enspace" => out.push_char('\u{2002}'),
                    "b" => {
                        fmt.bold = on;
                        out.set_format(fmt);
                    }
                    "i" => {
                        fmt.italic = on;
                        out.set_format(fmt);
                    }
                    "ul" | "uld" | "uldb" | "ulw" | "ulth" => {
                        fmt.underline = on;
                        out.set_format(fmt);
                    }
                    "ulnone" => {
                        fmt.underline = false;
                        out.set_format(fmt);
                    }
                    "li" => out.set_left_indent(param.unwrap_or(0)),
                    "uc" => {
                        fmt.unicode_skip = usize::try_from(param.unwrap_or(1)).unwrap_or(0);
                        out.set_format(fmt);
                    }
                    "u" => {
                        if let Some(ch) =
                            param.and_then(|p| decode_rtf_unicode(p, &mut high_surrogate))
                        {
                            out.push_char(ch);
                        }
                        unicode_fallback = fmt.unicode_skip;
                    }
                    "pard" | "plain" => {
                        // Unlike the RTF spec, `\pard` also resets character
                        // formatting here. Long-standing behaviour, pinned by
                        // test_rtf_to_html_pard_resets_formatting.
                        fmt.bold = false;
                        fmt.italic = false;
                        fmt.underline = false;
                        out.set_format(fmt);
                        out.set_left_indent(0);
                    }
                    "ansicpg" => {
                        if let Some(cp) = param.and_then(|p| u32::try_from(p).ok()) {
                            codepage = cp;
                        }
                    }
                    "bin" => skip_rtf_binary(&mut chars, param),
                    _ => {}
                }
            }
        }
    }

    out.finish()
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
) -> Result<(), ScrivenerError> {
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
                    )?;
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
                            let prose = read_rtf_content(data_dir, &scene_item.uuid)?;
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

                let prose = read_rtf_content(data_dir, &child.uuid)?;
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
    Ok(())
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
    if data_dir.exists()
        && !data_dir
            .canonicalize()?
            .starts_with(scriv_path.canonicalize()?)
    {
        return Err(ScrivenerError::InvalidStructure(
            "Files/Data escapes the selected bundle".into(),
        ));
    }

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
        )?;
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
fn read_rtf_content(
    data_dir: &std::path::Path,
    uuid: &str,
) -> Result<Option<String>, ScrivenerError> {
    // Scrivener identifiers are directory names, never user-supplied paths.
    // Reject both platforms' separators even when importing on macOS/Linux.
    // Older binder formats have no UUID/content path in Files/Data.
    if uuid.is_empty() {
        return Ok(None);
    }
    if !uuid
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(ScrivenerError::InvalidStructure(
            "Invalid binder content identifier".into(),
        ));
    }
    let rtf_path = data_dir.join(uuid).join("content.rtf");
    let canonical = match rtf_path.canonicalize() {
        Ok(path) => path,
        // Keep the existing empty-document fallback for absent/unreadable files.
        // No file is read when its canonical location cannot be established.
        Err(_) => return Ok(None),
    };
    if !canonical.starts_with(data_dir.canonicalize()?) {
        return Err(ScrivenerError::InvalidStructure(
            "Binder content escapes Files/Data".into(),
        ));
    }
    let Ok(bytes) = std::fs::read(canonical) else {
        return Ok(None);
    };
    let html = rtf_to_html(&decode_rtf_bytes(bytes));
    Ok((!html.is_empty()).then_some(html))
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

    /// Any C1 control character (U+0080–U+009F) in imported prose is invisible
    /// garbage; cp1252 text must never decode to one.
    fn assert_no_c1_controls(html: &str) {
        assert!(
            !html.chars().any(|c| ('\u{80}'..='\u{9F}').contains(&c)),
            "C1 control characters in: {html:?}"
        );
    }

    #[test]
    fn test_rtf_to_html_cocoa_scrivener_document() {
        // Shape of a Scrivener 3 (macOS) content.rtf written by Cocoa.
        let rtf = "{\\rtf1\\ansi\\ansicpg1252\\cocoartf2761\n\
\\cocoatextscaling0\\cocoaplatform0{\\fonttbl\\f0\\fnil\\fcharset0 Palatino-Roman;}\n\
{\\colortbl;\\red255\\green255\\blue255;}\n\
{\\*\\expandedcolortbl;;}\n\
\\pard\\tx360\\sl264\\slmult1\\pardirnatural\\partightenfactor0\n\
\n\
\\f0\\fs26 \\cf0 It\\'92s the first paragraph \\'97 with a dash.\\\n\
\\tab Second paragraph opens with a tab.\\\n\
\\\n\
Third, after a blank line.}";
        let html = rtf_to_html(rtf);
        assert_eq!(
            html,
            "<p>It\u{2019}s the first paragraph \u{2014} with a dash.</p>\
<p>\tSecond paragraph opens with a tab.</p>\
<p>Third, after a blank line.</p>"
        );
    }

    #[test]
    fn test_rtf_to_html_backslash_newline_is_paragraph_break() {
        let rtf = "{\\rtf1\\ansi First.\\\nSecond.\\\r\nThird.}";
        let html = rtf_to_html(rtf);
        assert_eq!(html, "<p>First.</p><p>Second.</p><p>Third.</p>");
    }

    #[test]
    fn test_rtf_to_html_cp1252_quotes_and_dashes() {
        let rtf =
            r"{\rtf1\ansi\ansicpg1252 \'93Don\'92t,\'94 she said \'96 twice \'97 then\'85 \'80}";
        let html = rtf_to_html(rtf);
        assert_eq!(
            html,
            "<p>\u{201C}Don\u{2019}t,\u{201D} she said \u{2013} twice \u{2014} then\u{2026} \u{20AC}</p>"
        );
        assert_no_c1_controls(&html);
    }

    #[test]
    fn test_rtf_to_html_cp1252_is_default_codepage() {
        // No \ansicpg: RTF's default ANSI code page is cp1252.
        let html = rtf_to_html(r"{\rtf1\ansi It\'92s caf\'e9.}");
        assert_eq!(html, "<p>It\u{2019}s caf\u{e9}.</p>");
    }

    #[test]
    fn test_rtf_to_html_respects_latin1_codepage() {
        // Latin-1 has no printable characters at 0x80–0x9F; drop, don't mangle.
        let html = rtf_to_html(r"{\rtf1\ansi\ansicpg28591 caf\'e9\'92}");
        assert_eq!(html, "<p>caf\u{e9}</p>");
    }

    #[test]
    fn test_rtf_to_html_undefined_cp1252_byte_dropped() {
        let html = rtf_to_html(r"{\rtf1\ansi a\'81b\'9dc}");
        assert_eq!(html, "<p>abc</p>");
    }

    #[test]
    fn test_rtf_to_html_tab_kept() {
        let html = rtf_to_html(r"{\rtf1\ansi Name:\tab Value}");
        assert_eq!(html, "<p>Name:\tValue</p>");
    }

    #[test]
    fn test_rtf_to_html_line_is_soft_break() {
        // Matches html_to_rtf, which writes <br> as \line.
        let html = rtf_to_html(r"{\rtf1\ansi Line one\line Line two\par Next}");
        assert_eq!(html, "<p>Line one<br>Line two</p><p>Next</p>");
    }

    #[test]
    fn test_rtf_to_html_cocoa_line_separator_is_soft_break() {
        let html = rtf_to_html(r"{\rtf1\ansi\uc0 Line one\u8232 Line two}");
        assert_eq!(html, "<p>Line one<br>Line two</p>");
    }

    #[test]
    fn test_rtf_to_html_skips_pict_hex() {
        // Windows RTF (Word, Scrivener for Windows) embeds images as hex.
        let rtf = r"{\rtf1\ansi Before.{\pict\wmetafile8\picw529\pich529 010009000003a2 ffd8ffe000104a46}\par
{\*\shppict{\pict\pngblip 89504e470d0a1a0a}}{\nonshppict{\pict\wmetafile8 0100090000}}After.}";
        let html = rtf_to_html(rtf);
        assert_eq!(html, "<p>Before.</p><p>After.</p>");
    }

    #[test]
    fn test_rtf_to_html_skips_cocoa_attachment_and_list_templates() {
        let rtf = r"{\rtf1\ansi {{\NeXTGraphic Pasted Graphic.png \width100 \height100}}Text{\pn\pnlvlblt{\pntxtb \'b7}} more}";
        let html = rtf_to_html(rtf);
        assert_eq!(html, "<p>Text more</p>");
    }

    #[test]
    fn test_rtf_to_html_escaped_brace_in_skipped_group() {
        // An escaped brace inside a skipped group must not unbalance it.
        let html = rtf_to_html(r"{\rtf1\ansi{\info{\title A \{ B}}Body text.}");
        assert_eq!(html, "<p>Body text.</p>");
    }

    #[test]
    fn test_rtf_to_html_unicode_hex_fallback_not_duplicated() {
        // Word writes \uN followed by an \'hh ANSI fallback (default \uc1).
        let html = rtf_to_html(r"{\rtf1\ansi \u8220\'93Hi\u8221\'94}");
        assert_eq!(html, "<p>\u{201C}Hi\u{201D}</p>");
    }

    #[test]
    fn test_rtf_to_html_uc_skip_count() {
        // \uc0 (what Cocoa writes): nothing follows, so "?" is real text.
        let html = rtf_to_html(r"{\rtf1\ansi\uc0 Why\u8253 ?}");
        assert_eq!(html, "<p>Why\u{203D}?</p>");
        // \uc2: two fallback characters to discard.
        let html = rtf_to_html(r"{\rtf1\ansi\uc2 a\u8212 --b}");
        assert_eq!(html, "<p>a\u{2014}b</p>");
    }

    #[test]
    fn test_rtf_to_html_unicode_surrogate_pair() {
        let html = rtf_to_html(r"{\rtf1\ansi Cat \u-10179?\u-8704?}");
        assert_eq!(html, "<p>Cat \u{1F600}</p>");
    }

    #[test]
    fn test_rtf_to_html_group_scopes_formatting() {
        let html = rtf_to_html(r"{\rtf1\ansi {\b bold} plain {\i it} \ul u\ul0  done}");
        assert_eq!(
            html,
            "<p><strong>bold</strong> plain <em>it</em> <u>u</u> done</p>"
        );
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
    fn test_parse_scrivener_bundle_raw_8bit_rtf_keeps_prose() {
        // Some writers emit raw cp1252 bytes rather than \'hh escapes, so the
        // file is not UTF-8. Its prose must still import.
        let dir = tempfile::tempdir().unwrap();
        let scriv = dir.path().join("Raw.scriv");
        std::fs::create_dir_all(&scriv).unwrap();
        let scrivx = r#"<?xml version="1.0" encoding="UTF-8"?>
<ScrivenerProject Identifier="RAW-1" Version="2.0">
  <Binder>
    <BinderItem UUID="DRAFT" Type="DraftFolder" Created="2024-01-01" Modified="2024-01-01">
      <Title>Draft</Title>
      <Children>
        <BinderItem UUID="TXT1" Type="Text" Created="2024-01-01" Modified="2024-01-01">
          <Title>Scene</Title>
          <MetaData><IncludeInCompile>Yes</IncludeInCompile></MetaData>
        </BinderItem>
      </Children>
    </BinderItem>
  </Binder>
</ScrivenerProject>"#;
        std::fs::write(scriv.join("Raw.scrivx"), scrivx).unwrap();
        let data = scriv.join("Files").join("Data");
        std::fs::create_dir_all(data.join("TXT1")).unwrap();
        let mut rtf = br"{\rtf1\ansi\ansicpg1252 It".to_vec();
        rtf.push(0x92);
        rtf.extend_from_slice(b"s caf");
        rtf.push(0xE9);
        rtf.extend_from_slice(b".}");
        std::fs::write(data.join("TXT1").join("content.rtf"), rtf).unwrap();

        let parsed = parse_scrivener_bundle(&scriv).unwrap();
        assert_eq!(parsed.beats.len(), 1);
        assert_eq!(
            parsed.beats[0].prose.as_deref(),
            Some("<p>It\u{2019}s caf\u{e9}.</p>")
        );
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

#[cfg(test)]
mod content_path_tests {
    use super::*;
    #[test]
    fn content_identifiers_cannot_escape_the_bundle() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("Data");
        std::fs::create_dir_all(data.join("SC-1")).unwrap();
        std::fs::write(data.join("SC-1/content.rtf"), "{\\rtf1 Safe prose}").unwrap();
        std::fs::write(dir.path().join("content.rtf"), "{\\rtf1 Outside prose}").unwrap();
        for id in [
            "..",
            "../",
            "../Data/SC-1",
            "/tmp",
            "C:\\outside",
            "..\\outside",
        ] {
            assert!(read_rtf_content(&data, id).is_err(), "{id}");
        }
        assert!(read_rtf_content(&data, dir.path().to_str().unwrap()).is_err());
        assert!(read_rtf_content(&data, "SC-1")
            .unwrap()
            .unwrap()
            .contains("Safe prose"));
        assert!(read_rtf_content(&data, "MISSING").unwrap().is_none());
    }
    #[test]
    fn unreadable_or_legacy_documents_keep_the_empty_document_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("Data");
        std::fs::create_dir_all(data.join("LEGACY")).unwrap();
        // A content.rtf that cannot be read (here, a directory) imports empty.
        std::fs::create_dir_all(data.join("LEGACY/content.rtf")).unwrap();
        assert!(read_rtf_content(&data, "LEGACY").unwrap().is_none());
        // Non-UTF-8 bytes are decoded as cp1252 rather than discarding the file.
        std::fs::create_dir_all(data.join("ANSI")).unwrap();
        std::fs::write(data.join("ANSI/content.rtf"), b"{\\rtf1 \xff}").unwrap();
        assert_eq!(
            read_rtf_content(&data, "ANSI").unwrap().as_deref(),
            Some("<p>\u{ff}</p>")
        );
        // An absent UUID must not resolve to Data/content.rtf.
        std::fs::write(data.join("content.rtf"), "{\\rtf1 Not this document}").unwrap();
        assert!(read_rtf_content(&data, "").unwrap().is_none());
    }

    #[test]
    #[cfg(unix)]
    fn content_symlinks_cannot_escape_the_data_directory() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("Data");
        std::fs::create_dir_all(data.join("SC-1")).unwrap();
        let outside = dir.path().join("outside.rtf");
        std::fs::write(&outside, "{\\rtf1 Outside prose}").unwrap();
        std::os::unix::fs::symlink(&outside, data.join("SC-1/content.rtf")).unwrap();
        assert!(read_rtf_content(&data, "SC-1").is_err());
    }
}
