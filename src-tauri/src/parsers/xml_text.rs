//! XML 1.0 character hygiene shared by every DOCX and EPUB writer.
//!
//! XML 1.0 only permits TAB, LF, CR, U+0020–U+D7FF, U+E000–U+FFFD and
//! U+10000 upward. Anything else makes the whole part malformed: Word reports
//! "unreadable content" and EPUB readers refuse the chapter. Prose can carry
//! such characters in from a paste or an import, and escaping `&<>"'` does not
//! remove them, so every XML writer passes its text (or its finished parts)
//! through here. Lone surrogates cannot occur in a Rust `str`.
use std::borrow::Cow;

/// U+000B (vertical tab) is what Word puts on the clipboard for a manual line
/// break (Shift+Enter) when text is copied as plain text; U+000C (form feed) is a
/// plain-text page break. Both separate text, so silently deleting them would
/// run the words either side together. Prose writers that can render a line
/// break split on these first and emit a real soft break; everywhere else
/// (titles, metadata, attribute values) [`xml_safe_text`] turns them into a
/// space.
pub(crate) const LINE_BREAK_CONTROLS: [char; 2] = ['\u{000B}', '\u{000C}'];

fn is_xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{0020}'..='\u{FFFD}' | '\u{10000}'..)
}

/// Make `text` safe to place in an XML 1.0 document. Line-separating controls
/// become a space (see [`LINE_BREAK_CONTROLS`]); every other disallowed
/// character, including the U+FFFE/U+FFFF non-characters, has no visible
/// meaning and is dropped. Borrows when nothing needs changing.
pub(crate) fn xml_safe_text(text: &str) -> Cow<'_, str> {
    if text.chars().all(is_xml_char) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(
        text.chars()
            .filter_map(|c| {
                if LINE_BREAK_CONTROLS.contains(&c) {
                    Some(' ')
                } else if is_xml_char(c) {
                    Some(c)
                } else {
                    None
                }
            })
            .collect(),
    )
}

/// Sanitise one serialised XML part in place. Serialised markup never contains
/// a disallowed character legitimately, so cleaning the whole part is
/// equivalent to cleaning each text node and cannot miss one.
fn xml_safe_part(part: &mut Vec<u8>) {
    let clean = match std::str::from_utf8(part).map(xml_safe_text) {
        Ok(Cow::Owned(clean)) => clean,
        _ => return,
    };
    *part = clean.into_bytes();
}

/// Build and pack a DOCX with every XML part made XML 1.0 safe. docx-rs escapes
/// markup characters but passes control characters straight into `<w:t>`, and
/// text reaches it through dozens of `add_text` call sites, so the finished
/// parts are cleaned here rather than trusting each call site. Every DOCX
/// writer must pack through this.
pub(crate) fn pack_docx<W>(docx: docx_rs::Docx, writer: W) -> Result<(), String>
where
    W: std::io::Write + std::io::Seek,
{
    let mut xml = docx.build();
    let props = &mut xml.doc_props;
    for part in [
        &mut xml.content_type,
        &mut xml.rels,
        &mut props.app,
        &mut props.core,
        &mut props.custom,
        &mut xml.styles,
        &mut xml.document,
        &mut xml.comments,
        &mut xml.document_rels,
        &mut xml.settings,
        &mut xml.font_table,
        &mut xml.numberings,
        &mut xml.comments_extended,
        &mut xml.taskpanes_rels,
        &mut xml.footnotes,
    ] {
        xml_safe_part(part);
    }
    for part in xml
        .headers
        .iter_mut()
        .chain(xml.header_rels.iter_mut())
        .chain(xml.footers.iter_mut())
        .chain(xml.footer_rels.iter_mut())
        .chain(xml.web_extensions.iter_mut())
        .chain(xml.custom_items.iter_mut())
        .chain(xml.custom_item_rels.iter_mut())
        .chain(xml.custom_item_props.iter_mut())
        .chain(xml.taskpanes.iter_mut())
    {
        xml_safe_part(part);
    }
    xml.pack(writer).map_err(|e| e.to_string())
}

/// Test oracle: `xml` parses and every character is one XML 1.0 allows.
/// quick-xml does not check character ranges, which is why a quick-xml pass
/// alone (like the workspace EPUB's) let these documents through.
#[cfg(test)]
pub(crate) fn assert_well_formed(name: &str, xml: &str) {
    if let Some(c) = xml.chars().find(|&c| !is_xml_char(c)) {
        panic!("{name} holds U+{:04X}, which XML 1.0 forbids", c as u32);
    }
    let mut reader = quick_xml::Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => panic!("{name} is not well-formed XML: {e}"),
            _ => (),
        }
    }
}

/// Assert every XML member of a DOCX/EPUB archive is well-formed.
#[cfg(test)]
pub(crate) fn assert_archive_well_formed(bytes: &[u8]) {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_string();
        if [".xml", ".rels", ".xhtml", ".opf", ".ncx"]
            .iter()
            .any(|ext| name.ends_with(ext))
        {
            let mut xml = String::new();
            std::io::Read::read_to_string(&mut file, &mut xml).unwrap();
            assert_well_formed(&name, &xml);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_valid_text_borrowed() {
        let text = "Tab\there,\nnew line\r\nand é 𝄞 \u{FFFD}";
        assert!(matches!(xml_safe_text(text), Cow::Borrowed(t) if t == text));
    }

    #[test]
    fn maps_line_break_controls_to_space_and_drops_the_rest() {
        assert_eq!(
            xml_safe_text(
                "one\u{000B}two\u{000C}three\u{0001}\u{0000}four\u{001F}\u{FFFE}\u{FFFF}"
            ),
            "one two threefour"
        );
    }

    #[test]
    fn pack_docx_cleans_every_text_part() {
        use docx_rs::{Docx, Footer, Header, Paragraph, Run};
        let dirty = "A\u{000B}B\u{0001}C";
        let docx = Docx::new()
            .header(
                Header::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(dirty))),
            )
            .footer(
                Footer::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(dirty))),
            )
            .add_paragraph(Paragraph::new().add_run(Run::new().add_text(dirty)));
        let mut buffer = Vec::new();
        pack_docx(docx, std::io::Cursor::new(&mut buffer)).unwrap();
        assert_archive_well_formed(&buffer);
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(buffer)).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("word/document.xml").unwrap(), &mut xml)
            .unwrap();
        assert!(xml.contains("A BC"), "{xml}");
    }
}
