//! Tolerant HTML events shared by prose exporters. Imported prose is not always
//! XML: preserve stray delimiters, unknown entities and malformed trailing text.
use quick_xml::{events::Event, Reader};

pub(crate) enum HtmlEvent {
    Start(String),
    End(String),
    Empty(String),
    Text(String),
}

/// HTML named entities beyond XML's five that imported prose actually carries:
/// typographic punctuation, spaces, and the Latin-1 letters and symbols.
/// Anything else stays as literal text, as before.
fn html_entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "nbsp" => " ",
        "ensp" => "\u{2002}",
        "emsp" => "\u{2003}",
        "thinsp" => "\u{2009}",
        "zwnj" => "\u{200C}",
        "zwj" => "\u{200D}",
        "shy" => "\u{00AD}",
        "ndash" => "\u{2013}",
        "mdash" => "\u{2014}",
        "lsquo" => "\u{2018}",
        "rsquo" => "\u{2019}",
        "sbquo" => "\u{201A}",
        "ldquo" => "\u{201C}",
        "rdquo" => "\u{201D}",
        "bdquo" => "\u{201E}",
        "lsaquo" => "\u{2039}",
        "rsaquo" => "\u{203A}",
        "laquo" => "\u{00AB}",
        "raquo" => "\u{00BB}",
        "hellip" => "\u{2026}",
        "bull" => "\u{2022}",
        "middot" => "\u{00B7}",
        "dagger" => "\u{2020}",
        "Dagger" => "\u{2021}",
        "prime" => "\u{2032}",
        "Prime" => "\u{2033}",
        "euro" => "\u{20AC}",
        "trade" => "\u{2122}",
        "copy" => "\u{00A9}",
        "reg" => "\u{00AE}",
        "sect" => "\u{00A7}",
        "para" => "\u{00B6}",
        "deg" => "\u{00B0}",
        "plusmn" => "\u{00B1}",
        "times" => "\u{00D7}",
        "divide" => "\u{00F7}",
        "frac14" => "\u{00BC}",
        "frac12" => "\u{00BD}",
        "frac34" => "\u{00BE}",
        "cent" => "\u{00A2}",
        "pound" => "\u{00A3}",
        "yen" => "\u{00A5}",
        "iexcl" => "\u{00A1}",
        "iquest" => "\u{00BF}",
        "ordf" => "\u{00AA}",
        "ordm" => "\u{00BA}",
        "Agrave" => "\u{00C0}",
        "Aacute" => "\u{00C1}",
        "Acirc" => "\u{00C2}",
        "Atilde" => "\u{00C3}",
        "Auml" => "\u{00C4}",
        "Aring" => "\u{00C5}",
        "AElig" => "\u{00C6}",
        "Ccedil" => "\u{00C7}",
        "Egrave" => "\u{00C8}",
        "Eacute" => "\u{00C9}",
        "Ecirc" => "\u{00CA}",
        "Euml" => "\u{00CB}",
        "Igrave" => "\u{00CC}",
        "Iacute" => "\u{00CD}",
        "Icirc" => "\u{00CE}",
        "Iuml" => "\u{00CF}",
        "ETH" => "\u{00D0}",
        "Ntilde" => "\u{00D1}",
        "Ograve" => "\u{00D2}",
        "Oacute" => "\u{00D3}",
        "Ocirc" => "\u{00D4}",
        "Otilde" => "\u{00D5}",
        "Ouml" => "\u{00D6}",
        "Oslash" => "\u{00D8}",
        "Ugrave" => "\u{00D9}",
        "Uacute" => "\u{00DA}",
        "Ucirc" => "\u{00DB}",
        "Uuml" => "\u{00DC}",
        "Yacute" => "\u{00DD}",
        "THORN" => "\u{00DE}",
        "szlig" => "\u{00DF}",
        "agrave" => "\u{00E0}",
        "aacute" => "\u{00E1}",
        "acirc" => "\u{00E2}",
        "atilde" => "\u{00E3}",
        "auml" => "\u{00E4}",
        "aring" => "\u{00E5}",
        "aelig" => "\u{00E6}",
        "ccedil" => "\u{00E7}",
        "egrave" => "\u{00E8}",
        "eacute" => "\u{00E9}",
        "ecirc" => "\u{00EA}",
        "euml" => "\u{00EB}",
        "igrave" => "\u{00EC}",
        "iacute" => "\u{00ED}",
        "icirc" => "\u{00EE}",
        "iuml" => "\u{00EF}",
        "eth" => "\u{00F0}",
        "ntilde" => "\u{00F1}",
        "ograve" => "\u{00F2}",
        "oacute" => "\u{00F3}",
        "ocirc" => "\u{00F4}",
        "otilde" => "\u{00F5}",
        "ouml" => "\u{00F6}",
        "oslash" => "\u{00F8}",
        "ugrave" => "\u{00F9}",
        "uacute" => "\u{00FA}",
        "ucirc" => "\u{00FB}",
        "uuml" => "\u{00FC}",
        "yacute" => "\u{00FD}",
        "thorn" => "\u{00FE}",
        "yuml" => "\u{00FF}",
        "OElig" => "\u{0152}",
        "oelig" => "\u{0153}",
        "Scaron" => "\u{0160}",
        "scaron" => "\u{0161}",
        "Yuml" => "\u{0178}",
        _ => return None,
    })
}

fn decode(entity: &str) -> String {
    quick_xml::escape::unescape_with(entity, |name| {
        quick_xml::escape::resolve_predefined_entity(name).or_else(|| html_entity(name))
    })
    .map(|s| s.into_owned())
    .unwrap_or_else(|_| entity.into())
}

pub(crate) fn html_events(html: &str) -> Vec<HtmlEvent> {
    // Protect text delimiters before quick-xml sees them. Keep complete tags and
    // entity tokens intact so marks and text events retain their original order.
    let mut safe = String::with_capacity(html.len());
    let mut protected_end = 0;
    for (i, c) in html.char_indices() {
        if i < protected_end {
            continue;
        }
        for (open, close) in [("<![CDATA[", "]]>"), ("<!--", "-->")] {
            if html[i..].starts_with(open) {
                if let Some(end) = html[i + open.len()..].find(close) {
                    protected_end = i + open.len() + end + close.len();
                    safe.push_str(&html[i..protected_end]);
                    break;
                }
            }
        }
        if i < protected_end {
            continue;
        }
        let tail = &html[i + c.len_utf8()..];
        match c {
            '&' if !tail.find(';').is_some_and(|end| {
                end > 0
                    && tail[..end]
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '#')
            }) =>
            {
                safe.push_str("&amp;")
            }
            '<' if !((tail.starts_with("!--")
                || tail
                    .trim_start_matches('/')
                    .starts_with(|c: char| c.is_ascii_alphabetic()))
                && tail.find('>').is_some_and(|end| !tail[..end].contains('<'))) =>
            {
                safe.push_str("&lt;")
            }
            _ => safe.push(c),
        }
    }
    let mut reader = Reader::from_str(&safe);
    reader.config_mut().check_end_names = false;
    let mut events = Vec::new();
    loop {
        let offset = reader.buffer_position() as usize;
        match reader.read_event() {
            Ok(Event::Start(e)) => events.push(HtmlEvent::Start(
                String::from_utf8_lossy(e.name().as_ref()).to_ascii_lowercase(),
            )),
            Ok(Event::End(e)) => events.push(HtmlEvent::End(
                String::from_utf8_lossy(e.name().as_ref()).to_ascii_lowercase(),
            )),
            Ok(Event::Empty(e)) => events.push(HtmlEvent::Empty(
                String::from_utf8_lossy(e.name().as_ref()).to_ascii_lowercase(),
            )),
            Ok(Event::Text(e)) => {
                events.push(HtmlEvent::Text(String::from_utf8_lossy(&e).into_owned()))
            }
            Ok(Event::CData(e)) => {
                events.push(HtmlEvent::Text(String::from_utf8_lossy(&e).into_owned()))
            }
            Ok(Event::GeneralRef(e)) => events.push(HtmlEvent::Text(decode(&format!(
                "&{};",
                String::from_utf8_lossy(&e)
            )))),
            Ok(Event::Eof) => break,
            Err(_) => {
                // Never interpret an XML error as EOF. Preserve all remaining
                // text, even if malformed markup must become literal prose.
                events.push(HtmlEvent::Text(decode(&safe[offset..])));
                break;
            }
            _ => (),
        }
    }
    events
}

#[derive(Clone, Copy, Default)]
pub(crate) enum ParagraphKind {
    #[default]
    Normal,
    Heading(u8),
    Blockquote,
}

pub(crate) struct HtmlRun {
    pub text: String,
    // bold, italic, strikethrough, underline
    pub marks: [bool; 4],
}

#[derive(Default)]
pub(crate) struct HtmlParagraph {
    pub runs: Vec<HtmlRun>,
    pub kind: ParagraphKind,
}

/// One TipTap walker for every prose exporter. Text policy is supplied by the
/// caller: manuscript typography may transform punctuation; interchange must not.
pub(crate) fn html_paragraphs(html: &str, transform: fn(&str) -> String) -> Vec<HtmlParagraph> {
    html_paragraphs_in_context(html, |_, text, _| transform(text))
}

/// [`html_paragraphs`] for typography that depends on neighbouring text. Inline
/// markup (`"I <em>hate</em>"`) and entities (`&quot;`) split a paragraph's
/// text into several chunks, so each chunk is transformed knowing the last
/// character already output before it in the paragraph (`None` at the start,
/// `'\n'` after a line break) and the raw text of the chunk after it (empty at
/// the end). That keeps a closing quote after an italic word closing.
pub(crate) fn html_paragraphs_in_context(
    html: &str,
    transform: impl Fn(Option<char>, &str, &str) -> String,
) -> Vec<HtmlParagraph> {
    let mut paragraphs = Vec::new();
    let mut current = HtmlParagraph::default();
    let mut marks = [0u32; 4];
    let mut quotes = 0u32;
    let flush = |current: &mut HtmlParagraph, paragraphs: &mut Vec<HtmlParagraph>| {
        let mut runs = std::mem::take(&mut current.runs);
        let mut prev = None;
        for i in 0..runs.len() {
            if runs[i].text == "\n" {
                prev = Some('\n');
                continue;
            }
            let next = runs[i + 1..]
                .iter()
                .map(|run| run.text.clone())
                .find(|text| !text.is_empty())
                .unwrap_or_default();
            let text = transform(prev, &runs[i].text, &next);
            prev = text.chars().last().or(prev);
            runs[i].text = text;
        }
        runs.retain(|run| !run.text.is_empty());
        if !runs.is_empty() {
            paragraphs.push(HtmlParagraph {
                runs,
                kind: current.kind,
            });
        }
    };
    for event in html_events(html) {
        let (tag, ending, empty) = match event {
            HtmlEvent::Text(text) => {
                if !text.is_empty() {
                    current.runs.push(HtmlRun {
                        text,
                        marks: marks.map(|n| n > 0),
                    });
                }
                continue;
            }
            HtmlEvent::Start(tag) => (tag, false, false),
            HtmlEvent::End(tag) => (tag, true, false),
            HtmlEvent::Empty(tag) => (tag, false, true),
        };
        let mark = match tag.as_str() {
            "strong" | "b" => Some(0),
            "em" | "i" => Some(1),
            "s" | "del" | "strike" => Some(2),
            "u" => Some(3),
            _ => None,
        };
        if let Some(index) = mark {
            if !empty {
                marks[index] = if ending {
                    marks[index].saturating_sub(1)
                } else {
                    marks[index] + 1
                };
            }
            continue;
        }
        match tag.as_str() {
            "br" if !ending => current.runs.push(HtmlRun {
                text: "\n".into(),
                marks: marks.map(|n| n > 0),
            }),
            "p" | "div" | "li" | "blockquote" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                flush(&mut current, &mut paragraphs);
                if tag == "blockquote" && !empty {
                    quotes = if ending {
                        quotes.saturating_sub(1)
                    } else {
                        quotes + 1
                    };
                }
                current.kind = if !ending && !empty && tag.starts_with('h') {
                    ParagraphKind::Heading(tag.as_bytes()[1] - b'0')
                } else if quotes > 0 {
                    ParagraphKind::Blockquote
                } else {
                    ParagraphKind::Normal
                };
            }
            _ => (),
        }
    }
    flush(&mut current, &mut paragraphs);
    paragraphs
}
