//! Tolerant HTML events shared by prose exporters. Imported prose is not always
//! XML: preserve stray delimiters, unknown entities and malformed trailing text.
use quick_xml::{events::Event, Reader};

pub(crate) enum HtmlEvent {
    Start(String),
    End(String),
    Empty(String),
    Text(String),
}

fn decode(entity: &str) -> String {
    if entity == "&nbsp;" {
        " ".into()
    } else {
        quick_xml::escape::unescape(entity)
            .map(|s| s.into_owned())
            .unwrap_or_else(|_| entity.into())
    }
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
    let mut paragraphs = Vec::new();
    let mut current = HtmlParagraph::default();
    let mut marks = [0u32; 4];
    let mut quotes = 0u32;
    fn flush(current: &mut HtmlParagraph, paragraphs: &mut Vec<HtmlParagraph>) {
        if !current.runs.is_empty() {
            paragraphs.push(HtmlParagraph {
                runs: std::mem::take(&mut current.runs),
                kind: current.kind,
            });
        }
    }
    for event in html_events(html) {
        let (tag, ending, empty) = match event {
            HtmlEvent::Text(text) => {
                let text = transform(&text);
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
