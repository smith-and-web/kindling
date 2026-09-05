//! Local novelWriter project interchange. Handles anchor record identity; beats
//! use positions within their scene. Notes and links are intentionally import-only
//! in sync, matching the other source types.
mod format;
mod markup;
mod writer;
use crate::models::*;
pub use format::*;
pub use markup::{html_to_nw, nw_to_html};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use uuid::Uuid;
pub use writer::*;

#[derive(Debug, thiserror::Error)]
pub enum NovelWriterError {
    #[error("Cannot read novelWriter project: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid novelWriter XML: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Invalid novelWriter project: {0}")]
    Invalid(String),
    #[error("Unsupported novelWriter fileVersion '{0}' (supported: 1.4, 1.5, 1.6)")]
    Version(String),
}
#[derive(Debug)]
pub struct ParsedNovelWriter {
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub beats: Vec<Beat>,
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
    pub reference_items: Vec<ReferenceItem>,
    pub scene_character_refs: Vec<(Uuid, Uuid)>,
    pub scene_location_refs: Vec<(Uuid, Uuid)>,
    pub scene_reference_item_refs: Vec<(Uuid, Uuid)>,
    pub scenes_with_beat_comments: HashSet<String>,
}
pub fn novelwriter_beat_source_id(handle: &str, position: i32) -> String {
    format!("novelwriter:beat:{handle}:{position}")
}

pub fn parse_novelwriter_project(path: &Path) -> Result<ParsedNovelWriter, NovelWriterError> {
    let index = path.join("nwProject.nwx");
    let xml = std::fs::read_to_string(&index)
        .map_err(|e| NovelWriterError::Invalid(format!("Cannot read {}: {e}", index.display())))?;
    let doc = read_nwx(&xml)?;
    let mut project = Project::new(
        doc.name.clone(),
        SourceType::NovelWriter,
        Some(path.to_string_lossy().into_owned()),
    );
    project.author_pen_name = (!doc.author.is_empty()).then_some(doc.author.clone());
    let mut parsed = ParsedNovelWriter {
        project,
        chapters: vec![],
        scenes: vec![],
        beats: vec![],
        characters: vec![],
        locations: vec![],
        reference_items: vec![],
        scene_character_refs: vec![],
        scene_location_refs: vec![],
        scene_reference_item_refs: vec![],
        scenes_with_beat_comments: HashSet::new(),
    };
    let mut ordered = Vec::new();
    fn walk<'a>(parent: &str, doc: &'a Nwx, out: &mut Vec<&'a NwItem>) {
        let mut children: Vec<_> = doc.items.iter().filter(|i| i.parent == parent).collect();
        children.sort_by_key(|i| i.order);
        for item in children {
            out.push(item);
            walk(&item.handle, doc, out);
        }
    }
    walk("None", &doc, &mut ordered);
    let novel_root = ordered
        .iter()
        .find(|i| i.item_type == "ROOT" && i.class == "NOVEL")
        .map(|i| i.handle.as_str());
    let mut chapter_id = None;
    let mut part_handle: Option<String> = None;
    let mut tags: HashMap<String, (String, Uuid)> = HashMap::new();
    let mut links = Vec::new();
    for item in ordered {
        let root = doc.items.iter().find(|r| r.handle == item.root).unwrap();
        if item.item_type != "FILE"
            || ["ARCHIVE", "TRASH", "TEMPLATE"].contains(&root.class.as_str())
        {
            continue;
        }
        if root.class == "NOVEL" && Some(root.handle.as_str()) != novel_root {
            continue;
        }
        let modern = path.join("content").join(format!("{}.md", item.handle));
        let file = if modern.exists() {
            modern
        } else {
            path.join("content").join(format!("{}.nwd", item.handle))
        };
        let text = std::fs::read_to_string(&file).map_err(|e| {
            NovelWriterError::Invalid(format!("Cannot read {}: {e}", file.display()))
        })?;
        let body = read_document(&text)?.body;
        if root.class == "NOVEL" && item.layout != "NOTE" {
            let heading = body.lines().find_map(heading);
            let (level, title) = heading.unwrap_or_else(|| {
                (
                    item.heading
                        .strip_prefix('H')
                        .and_then(|n| n.parse().ok())
                        .unwrap_or(3),
                    item.name.clone(),
                )
            });
            if level <= 2 {
                let ch = Chapter::new(parsed.project.id, title, parsed.chapters.len() as i32)
                    .with_source_id(Some(item.handle.clone()))
                    .with_is_part(level == 1);
                chapter_id = (level == 2).then_some(ch.id);
                if level == 1 {
                    part_handle = Some(item.handle.clone());
                }
                parsed.chapters.push(ch);
                // Heading documents may carry prose (common in native projects).
                // Preserve it in a child scene instead of silently dropping it.
                let content = scene_content(&body);
                if content.1.iter().all(|(_, p)| p.trim().is_empty()) {
                    continue;
                }
                if level == 1 {
                    let ch = Chapter::new(
                        parsed.project.id,
                        item.name.clone(),
                        parsed.chapters.len() as i32,
                    )
                    .with_source_id(Some(stable_handle(&format!("{}:chapter", item.handle))));
                    chapter_id = Some(ch.id);
                    parsed.chapters.push(ch);
                }
                add_scene(
                    &mut parsed,
                    item,
                    chapter_id.unwrap(),
                    item.name.clone(),
                    &body,
                    &mut links,
                    true,
                );
            } else {
                let id = if let Some(id) = chapter_id {
                    id
                } else {
                    let ch = Chapter::new(
                        parsed.project.id,
                        "Manuscript".into(),
                        parsed.chapters.len() as i32,
                    )
                    .with_source_id(Some(stable_handle(&format!(
                        "{}:chapter",
                        part_handle.as_deref().unwrap_or(&item.root)
                    ))));
                    let id = ch.id;
                    parsed.chapters.push(ch);
                    chapter_id = Some(id);
                    id
                };
                add_scene(&mut parsed, item, id, title, &body, &mut links, false);
            }
        } else {
            let mut description = Vec::new();
            let mut attributes: HashMap<String, String> = HashMap::new();
            let mut last_attribute: Option<String> = None;
            let mut tag = None;
            for line in body.lines() {
                if let (Some(key), Some(continuation)) =
                    (&last_attribute, line.strip_prefix("    "))
                {
                    if let Some(value) = attributes.get_mut(key) {
                        value.push('\n');
                        value.push_str(continuation);
                    }
                    continue;
                }
                last_attribute = None;
                if let Some(t) = line.strip_prefix("@tag:") {
                    tag = Some(
                        t.split('|')
                            .next()
                            .unwrap_or_default()
                            .trim()
                            .to_lowercase(),
                    );
                } else if let Some(d) = line.strip_prefix("%Short:") {
                    description.push(d.trim().to_string());
                } else if let Some((key, value)) =
                    line.strip_prefix("**").and_then(|s| s.split_once(":** "))
                {
                    attributes.insert(key.into(), value.into());
                    last_attribute = Some(key.into());
                } else if !line.starts_with(['#', '@', '%']) && !line.trim().is_empty() {
                    description.push(line.into());
                }
            }
            let description =
                (!description.is_empty()).then(|| nw_to_html(&description.join("\n")));
            let name = body
                .lines()
                .find_map(heading)
                .map(|(_, n)| n)
                .unwrap_or_else(|| item.name.clone());
            let (kind, id) = match root.class.as_str() {
                "CHARACTER" => {
                    let c = Character::new(
                        parsed.project.id,
                        name,
                        description,
                        Some(item.handle.clone()),
                    )
                    .with_attributes(attributes);
                    let id = c.id;
                    parsed.characters.push(c);
                    ("char".into(), id)
                }
                "WORLD" => {
                    let l = Location::new(
                        parsed.project.id,
                        name,
                        description,
                        Some(item.handle.clone()),
                    )
                    .with_attributes(attributes);
                    let id = l.id;
                    parsed.locations.push(l);
                    ("location".into(), id)
                }
                class => {
                    let reference_type = reference_type(class).to_string();
                    if !parsed.project.reference_types.contains(&reference_type) {
                        parsed.project.reference_types.push(reference_type.clone());
                    }
                    let mut r = ReferenceItem::new(
                        parsed.project.id,
                        reference_type,
                        name,
                        description,
                        Some(item.handle.clone()),
                    );
                    r.attributes = attributes;
                    let id = r.id;
                    parsed.reference_items.push(r);
                    (reference_keyword(class).into(), id)
                }
            };
            if let Some(tag) = tag {
                tags.entry(tag).or_insert((kind, id));
            }
        }
    }
    for (scene, keyword, tag) in links {
        if let Some((kind, id)) = tags.get(&tag.to_lowercase()) {
            if kind != &keyword {
                continue;
            }
            match kind.as_str() {
                "char" => parsed.scene_character_refs.push((scene, *id)),
                "location" => parsed.scene_location_refs.push((scene, *id)),
                _ => parsed.scene_reference_item_refs.push((scene, *id)),
            }
        }
    }
    Ok(parsed)
}
fn heading(line: &str) -> Option<(usize, String)> {
    let n = line.bytes().take_while(|&c| c == b'#').count();
    if n == 0 || n > 3 {
        return None;
    }
    let rest = line[n..].trim_start_matches('!');
    rest.starts_with(' ').then(|| (n, rest.trim().into()))
}
fn scene_content(body: &str) -> (bool, Vec<(String, String)>, Option<String>) {
    let mut marked = false;
    let mut segments: Vec<(String, String)> = vec![];
    let mut title = "Scene Content".to_string();
    let mut prose = Vec::new();
    let mut synopsis = Vec::new();
    for line in body.lines() {
        if let Some(beat) = line.strip_prefix("% Beat:") {
            if marked || prose.iter().any(|s: &&str| !s.trim().is_empty()) {
                segments.push((title, prose.join("\n").trim().into()));
            }
            title = beat.trim().into();
            prose.clear();
            marked = true;
        } else if let Some(s) = line.strip_prefix("%Synopsis:") {
            synopsis.push(s.trim());
        } else if heading(line).is_some() || line.starts_with(['%', '@']) {
            continue;
        } else {
            prose.push(line.strip_prefix("#### ").unwrap_or(line));
        }
    }
    if marked || !prose.is_empty() || segments.is_empty() {
        segments.push((title, prose.join("\n").trim().into()));
    }
    (
        marked,
        segments,
        (!synopsis.is_empty()).then(|| synopsis.join("\n")),
    )
}
fn add_scene(
    parsed: &mut ParsedNovelWriter,
    item: &NwItem,
    chapter: Uuid,
    title: String,
    body: &str,
    links: &mut Vec<(Uuid, String, String)>,
    derived: bool,
) {
    let (marked, segments, synopsis) = scene_content(body);
    let handle = if derived {
        stable_handle(&format!("{}:prose", item.handle))
    } else {
        item.handle.clone()
    };
    let position = parsed
        .scenes
        .iter()
        .filter(|s| s.chapter_id == chapter)
        .count() as i32;
    let mut scene =
        Scene::new(chapter, title, synopsis, position).with_source_id(Some(handle.clone()));
    scene.scene_status = match item.status.as_str() {
        "s000001" => SceneStatus::Revised,
        "s000002" => SceneStatus::Final,
        _ => SceneStatus::Draft,
    };
    if marked {
        parsed.scenes_with_beat_comments.insert(handle.clone());
    } else {
        scene.editor_mode = EditorMode::Page;
    }
    scene.prose = Some(nw_to_html(
        &segments
            .iter()
            .map(|(_, p)| p.as_str())
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n"),
    ));
    for (n, (content, prose)) in segments.into_iter().enumerate() {
        let mut beat = Beat::new(scene.id, content, n as i32)
            .with_source_id(Some(novelwriter_beat_source_id(&handle, n as i32)));
        beat.prose = Some(nw_to_html(&prose));
        parsed.beats.push(beat);
    }
    for line in body.lines().filter(|l| l.starts_with('@')) {
        if let Some((key, values)) = line[1..].split_once(':') {
            if [
                "char", "location", "object", "entity", "plot", "time", "custom",
            ]
            .contains(&key)
            {
                for tag in values.split(',') {
                    links.push((
                        scene.id,
                        key.into(),
                        tag.split('|').next().unwrap_or("").trim().into(),
                    ));
                }
            }
        }
    }
    parsed.scenes.push(scene);
}
pub fn reference_class(kind: &str) -> &'static str {
    match kind.to_lowercase().as_str() {
        "plot" | "plots" | "objectives" => "PLOT",
        "timeline" | "timelines" => "TIMELINE",
        "object" | "objects" | "items" => "OBJECT",
        "entity" | "entities" | "organizations" => "ENTITY",
        _ => "CUSTOM",
    }
}
fn reference_type(class: &str) -> &'static str {
    match class {
        "PLOT" => "objectives",
        "TIMELINE" => "timelines",
        "OBJECT" => "items",
        "ENTITY" => "organizations",
        _ => "custom",
    }
}
fn reference_keyword(class: &str) -> &'static str {
    match class {
        "CHARACTER" => "char",
        "WORLD" => "location",
        "OBJECT" => "object",
        "ENTITY" => "entity",
        "PLOT" => "plot",
        "TIMELINE" => "time",
        _ => "custom",
    }
}

#[cfg(test)]
pub(crate) mod tests;
