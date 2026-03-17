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
    let mut current_element = String::new();

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
                let text = String::from_utf8_lossy(e).to_string();
                match current_element.as_str() {
                    "Title" => title = text,
                    "IncludeInCompile" => include_in_compile = text == "Yes",
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "BinderItem" {
                    break;
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
}

/// Generate a complete .scrivx XML for a new Scrivener project
pub fn generate_scrivx(
    project_title: &str,
    chapters: &[ExportChapter],
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
    write_metadata(&mut writer, true)?;

    // Children of Draft
    writer.write_event(Event::Start(BytesStart::new("Children")))?;
    for chapter in chapters {
        if chapter.is_part {
            write_text_item(
                &mut writer,
                &chapter.uuid,
                &chapter.title,
                &chapter.created,
                &chapter.modified,
                true,
            )?;
        } else if chapter.scenes.is_empty() {
            write_folder_item(
                &mut writer,
                &chapter.uuid,
                &chapter.title,
                &chapter.created,
                &chapter.modified,
                &[],
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
            rtf.push_str("\\pard\\pardirnatural\\partightenfactor0\n\\f0\\fs24 \\cf2 ");
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
            rtf.push_str("\\pard\\li720\\pardirnatural\\partightenfactor0\n\\f0\\i\\fs24 \\cf2 ");
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
            children: vec![
                BinderItem {
                    uuid: "SC-1".to_string(),
                    item_type: "Text".to_string(),
                    title: "Scene 1".to_string(),
                    created: String::new(),
                    modified: String::new(),
                    include_in_compile: true,
                    children: vec![],
                },
                BinderItem {
                    uuid: "CH-1".to_string(),
                    item_type: "Folder".to_string(),
                    title: "Chapter".to_string(),
                    created: String::new(),
                    modified: String::new(),
                    include_in_compile: true,
                    children: vec![BinderItem {
                        uuid: "SC-2".to_string(),
                        item_type: "Text".to_string(),
                        title: "Scene 2".to_string(),
                        created: String::new(),
                        modified: String::new(),
                        include_in_compile: true,
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
        }];

        let xml = generate_scrivx("Test Project", &chapters).unwrap();
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
}
