use super::NovelWriterError;
use quick_xml::{events::Event, Reader};
use std::collections::{BTreeMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NwItem {
    pub handle: String,
    pub parent: String,
    pub root: String,
    pub order: usize,
    pub item_type: String,
    pub class: String,
    pub layout: String,
    pub heading: String,
    pub name: String,
    pub status: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nwx {
    pub id: String,
    pub name: String,
    pub author: String,
    pub items: Vec<NwItem>,
}

pub fn is_handle(s: &str) -> bool {
    s.len() == 13
        && s.bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub fn new_handle() -> String {
    Uuid::new_v4().simple().to_string()[..13].into()
}
pub fn stable_handle(key: &str) -> String {
    sha1(key.as_bytes())[..13].into()
}
fn xml(s: &str) -> String {
    quick_xml::escape::escape(s).into_owned()
}

pub fn write_nwx(doc: &Nwx) -> String {
    let mut out = format!("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<novelWriterXML appVersion=\"26.2\" hexVersion=\"0x26020000\" fileVersion=\"1.6\" fileRevision=\"0\" timeStamp=\"{}\">\n  <project id=\"{}\"><name>{}</name><author>{}</author></project>\n  <settings><doBackup>no</doBackup><language>en</language><spellChecking auto=\"no\">None</spellChecking><status>\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"), xml(&doc.id), xml(&doc.name), xml(&doc.author));
    for (i, name) in ["Draft", "Revised", "Final"].iter().enumerate() {
        out.push_str(&format!("    <entry key=\"s00000{i}\" count=\"0\" color=\"base\" shape=\"CIRCLE\">{name}</entry>\n"));
    }
    out.push_str("  </status><importance><entry key=\"i000000\" count=\"0\" color=\"base\" shape=\"SQUARE\">Default</entry></importance></settings>\n  <content>\n");
    for i in &doc.items {
        out.push_str(&format!("    <item handle=\"{}\" parent=\"{}\" root=\"{}\" order=\"{}\" type=\"{}\" class=\"{}\" layout=\"{}\">\n      <meta expanded=\"no\" heading=\"{}\" charCount=\"0\" wordCount=\"0\" paraCount=\"0\" cursorPos=\"0\" />\n      <name status=\"{}\" import=\"i000000\" active=\"yes\">{}</name>\n    </item>\n", xml(&i.handle), xml(&i.parent), xml(&i.root), i.order, xml(&i.item_type), xml(&i.class), xml(&i.layout), xml(&i.heading), xml(&i.status), xml(&i.name)));
    }
    out.push_str("  </content>\n</novelWriterXML>\n");
    out
}

pub fn read_nwx(text: &str) -> Result<Nwx, NovelWriterError> {
    let mut reader = Reader::from_str(text);
    let mut stack: Vec<String> = Vec::new();
    let mut doc = Nwx {
        id: String::new(),
        name: String::new(),
        author: String::new(),
        items: vec![],
    };
    let mut current: Option<NwItem> = None;
    let mut root_seen = false;
    loop {
        let event = reader.read_event()?;
        let empty = matches!(event, Event::Empty(_));
        match event {
            Event::Start(e) | Event::Empty(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let attrs: BTreeMap<String, String> = e
                    .attributes()
                    .map(|a| {
                        let a = a.map_err(|e| NovelWriterError::Invalid(e.to_string()))?;
                        Ok((
                            String::from_utf8_lossy(a.key.as_ref()).into_owned(),
                            a.decode_and_unescape_value(reader.decoder())?.into_owned(),
                        ))
                    })
                    .collect::<Result<_, NovelWriterError>>()?;
                let get = |key: &str| attrs.get(key).cloned().unwrap_or_default();
                if root_seen && stack.is_empty() {
                    return Err(NovelWriterError::Invalid(
                        "Multiple XML root elements".into(),
                    ));
                }
                if !root_seen {
                    if name != "novelWriterXML" {
                        return Err(NovelWriterError::Invalid(
                            "Root element must be novelWriterXML".into(),
                        ));
                    }
                    root_seen = true;
                    let version = get("fileVersion");
                    if !["1.4", "1.5", "1.6"].contains(&version.as_str()) {
                        return Err(NovelWriterError::Version(version));
                    }
                }
                match name.as_str() {
                    "project" => doc.id = get("id"),
                    "item" => {
                        current = Some(NwItem {
                            handle: get("handle"),
                            parent: get("parent"),
                            root: get("root"),
                            order: get("order").parse().unwrap_or(0),
                            item_type: get("type"),
                            class: get("class"),
                            layout: get("layout"),
                            heading: String::new(),
                            name: String::new(),
                            status: "s000000".into(),
                        })
                    }
                    "meta" => {
                        if let Some(i) = &mut current {
                            i.heading = get("heading");
                        }
                    }
                    "name" => {
                        if let Some(i) = &mut current {
                            i.status = get("status");
                        }
                    }
                    _ => (),
                }
                // Empty elements never produce an End event.
                if !empty {
                    stack.push(name);
                }
            }
            Event::Text(e) => {
                let value = e
                    .xml_content()
                    .map_err(|e| NovelWriterError::Invalid(e.to_string()))?;
                append_text(&mut doc, &mut current, &stack, &value);
            }
            Event::GeneralRef(e) => {
                let entity = format!("&{};", String::from_utf8_lossy(&e));
                let value = quick_xml::escape::unescape(&entity)
                    .map_err(|e| NovelWriterError::Invalid(e.to_string()))?;
                append_text(&mut doc, &mut current, &stack, &value);
            }
            Event::End(e) => {
                if e.name().as_ref() == b"item" {
                    if let Some(i) = current.take() {
                        doc.items.push(i);
                    }
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => (),
        }
    }
    if !root_seen || !stack.is_empty() {
        return Err(NovelWriterError::Invalid(
            "Incomplete novelWriterXML document".into(),
        ));
    }
    let mut handles = HashSet::new();
    for i in &doc.items {
        if !is_handle(&i.handle) || !handles.insert(i.handle.clone()) {
            return Err(NovelWriterError::Invalid(format!(
                "Invalid or duplicate handle: {}",
                i.handle
            )));
        }
    }
    for i in &doc.items {
        if !handles.contains(&i.root) || (i.item_type != "ROOT" && !handles.contains(&i.parent)) {
            return Err(NovelWriterError::Invalid(format!(
                "Missing parent/root for {}",
                i.handle
            )));
        }
        let mut ancestors = HashSet::new();
        let mut node = i;
        while node.item_type != "ROOT" {
            if !ancestors.insert(&node.handle) {
                return Err(NovelWriterError::Invalid("Cyclic project tree".into()));
            }
            node = doc
                .items
                .iter()
                .find(|p| p.handle == node.parent)
                .ok_or_else(|| NovelWriterError::Invalid("Missing parent".into()))?;
        }
        if node.handle != i.root {
            return Err(NovelWriterError::Invalid("Inconsistent root".into()));
        }
    }
    Ok(doc)
}
fn append_text(doc: &mut Nwx, current: &mut Option<NwItem>, stack: &[String], text: &str) {
    match stack.last().map(String::as_str) {
        Some("name") => {
            if let Some(i) = current {
                i.name.push_str(text);
            } else if stack.iter().any(|s| s == "project") {
                doc.name.push_str(text);
            }
        }
        Some("author") => doc.author.push_str(text),
        _ => (),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NwDocument {
    pub metadata: BTreeMap<String, String>,
    pub body: String,
}
pub fn write_document(item: &NwItem, body: &str) -> String {
    let body = format!("{}\n", body.trim_end_matches('\n'));
    let mut out = String::from("+++\n");
    for (key, value) in [
        ("name", item.name.clone()),
        ("parent", item.parent.clone()),
        ("handle", item.handle.clone()),
        ("class", item.class.clone()),
        ("layout", item.layout.clone()),
        ("textHash", sha1(body.as_bytes())),
        ("createdDate", "Unknown".into()),
        ("updatedDate", "Unknown".into()),
    ] {
        out.push_str(&format!(
            "{key} = {}\n",
            serde_json::to_string(&value).unwrap()
        ));
    }
    out.push_str("+++\n");
    out.push_str(&body);
    out
}
pub fn read_document(text: &str) -> Result<NwDocument, NovelWriterError> {
    let normalized = text.replace("\r\n", "\n");
    let mut metadata = BTreeMap::new();
    if let Some(rest) = normalized.strip_prefix("+++\n") {
        let (header, body) = rest
            .split_once("\n+++\n")
            .ok_or_else(|| NovelWriterError::Invalid("Unclosed document TOML header".into()))?;
        for line in header
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| NovelWriterError::Invalid("Invalid document header".into()))?;
            let value = value.trim();
            let decoded = if value.len() >= 2 && value.starts_with('\'') && value.ends_with('\'') {
                value[1..value.len() - 1].into()
            } else {
                serde_json::from_str(value)
                    .map_err(|e| NovelWriterError::Invalid(format!("Invalid header string: {e}")))?
            };
            metadata.insert(key.trim().into(), decoded);
        }
        Ok(NwDocument {
            metadata,
            body: body.into(),
        })
    } else {
        // Legacy 1.4/1.5 .nwd headers; body edits need not refresh textHash.
        let body = normalized
            .lines()
            .skip_while(|l| l.starts_with("%%~"))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(NwDocument { metadata, body })
    }
}

/// SHA-1 for novelWriter's document checksum (not used for security).
/// Kept local to avoid changing the dependency/lockfile contract.
pub fn sha1(data: &[u8]) -> String {
    use sha1::{Digest, Sha1};
    format!("{:x}", Sha1::digest(data))
}
