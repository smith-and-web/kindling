use super::*;
use crate::db;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelWriterExportOptions {
    #[serde(default = "yes")]
    pub include_beat_comments: bool,
    #[serde(default = "yes")]
    pub include_notes: bool,
    #[serde(default)]
    pub create_snapshot: bool,
}
fn yes() -> bool {
    true
}
impl Default for NovelWriterExportOptions {
    fn default() -> Self {
        Self {
            include_beat_comments: true,
            include_notes: true,
            create_snapshot: false,
        }
    }
}

struct Builder {
    doc: Nwx,
    bodies: BTreeMap<String, String>,
    used: HashSet<String>,
}
impl Builder {
    fn handle(&mut self, id: Uuid, source: Option<&str>) -> String {
        let mut handle = source
            .filter(|s| is_handle(s))
            .map(str::to_string)
            .unwrap_or_else(|| stable_handle(&id.to_string()));
        while !self.used.insert(handle.clone()) {
            handle = new_handle();
        }
        handle
    }
    fn root(&mut self, class: &str) -> String {
        if let Some(root) = self
            .doc
            .items
            .iter()
            .find(|i| i.item_type == "ROOT" && i.class == class)
        {
            return root.handle.clone();
        }
        let mut handle = stable_handle(&format!("kindling:novelwriter:root:{class}"));
        while !self.used.insert(handle.clone()) {
            handle = new_handle();
        }
        self.doc.items.push(NwItem {
            handle: handle.clone(),
            parent: "None".into(),
            root: handle.clone(),
            order: self
                .doc
                .items
                .iter()
                .filter(|i| i.item_type == "ROOT")
                .count(),
            item_type: "ROOT".into(),
            class: class.into(),
            layout: if class == "NOVEL" { "DOCUMENT" } else { "NOTE" }.into(),
            heading: "H0".into(),
            name: if class == "NOVEL" {
                "Manuscript"
            } else {
                class
            }
            .into(),
            status: "s000000".into(),
        });
        handle
    }
    fn push(&mut self, mut item: NwItem, body: String) {
        item.order = self
            .doc
            .items
            .iter()
            .filter(|i| i.parent == item.parent)
            .count();
        self.bodies
            .insert(item.handle.clone(), write_document(&item, &body));
        self.doc.items.push(item);
    }
}
fn line(text: &str) -> String {
    text.replace(['\r', '\n'], " ")
}
fn document(
    handle: String,
    parent: &str,
    root: &str,
    class: &str,
    name: &str,
    level: u8,
) -> NwItem {
    NwItem {
        handle,
        parent: parent.into(),
        root: root.into(),
        order: 0,
        item_type: "FILE".into(),
        class: class.into(),
        layout: if class == "NOVEL" { "DOCUMENT" } else { "NOTE" }.into(),
        heading: format!("H{level}"),
        name: name.into(),
        status: "s000000".into(),
    }
}

/// Create-only export. Build everything before touching disk; source IDs are
/// persisted only after all files are written successfully. Foreign sources keep
/// their sync identities and receive deterministic UUID-derived export handles.
pub fn export_novelwriter_project(
    conn: &Connection,
    project_id: &Uuid,
    destination: &Path,
    options: &NovelWriterExportOptions,
) -> Result<usize, String> {
    fn metadata_files(destination: &Path) -> Result<Vec<std::path::PathBuf>, String> {
        if !destination.exists() {
            return Ok(vec![]);
        }
        if !destination.is_dir() {
            return Err("Choose an empty destination folder for novelWriter export".into());
        }
        std::fs::read_dir(destination)
            .map_err(|e| e.to_string())?
            .map(|entry| {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.file_type().map_err(|e| e.to_string())?.is_file()
                    && matches!(
                        entry.file_name().to_str(),
                        Some(".DS_Store" | "Thumbs.db" | "desktop.ini")
                    )
                {
                    Ok(entry.path())
                } else {
                    Err("Choose an empty destination folder for novelWriter export".into())
                }
            })
            .collect()
    }
    metadata_files(destination)?;
    let project = db::get_project(conn, project_id)
        .map_err(|e| e.to_string())?
        .ok_or("Project not found")?;
    if project.project_type == "screenplay" {
        return Err("novelWriter export is available for novels only".into());
    }
    let chapters = db::get_chapters(conn, project_id).map_err(|e| e.to_string())?;
    let scenes = db::get_all_project_scenes(conn, project_id).map_err(|e| e.to_string())?;
    let beats = db::get_all_project_beats(conn, project_id).map_err(|e| e.to_string())?;
    let mut b = Builder {
        doc: Nwx {
            id: project.id.to_string(),
            name: project.name.clone(),
            author: project.author_pen_name.clone().unwrap_or_default(),
            items: vec![],
        },
        bodies: BTreeMap::new(),
        used: HashSet::new(),
    };
    let novel = b.root("NOVEL");
    let mut ids: Vec<(&str, Uuid, String)> = vec![];
    let mut tags: HashMap<Uuid, (String, String)> = HashMap::new();
    if options.include_notes {
        let mut notes = Vec::new();
        for c in db::get_characters(conn, project_id).map_err(|e| e.to_string())? {
            notes.push((
                "characters",
                c.id,
                "CHARACTER",
                c.name,
                c.description,
                c.attributes,
                c.source_id,
            ));
        }
        for l in db::get_locations(conn, project_id).map_err(|e| e.to_string())? {
            notes.push((
                "locations",
                l.id,
                "WORLD",
                l.name,
                l.description,
                l.attributes,
                l.source_id,
            ));
        }
        for r in db::get_all_reference_items(conn, project_id).map_err(|e| e.to_string())? {
            notes.push((
                "reference_items",
                r.id,
                reference_class(&r.reference_type),
                r.name,
                r.description,
                r.attributes,
                r.source_id,
            ));
        }
        // Stable class/name/handle order makes cross-kind tag collisions deterministic.
        notes.sort_by_key(|(_, id, class, name, _, _, sid)| {
            (
                class.to_string(),
                name.to_lowercase(),
                sid.clone()
                    .unwrap_or_else(|| stable_handle(&id.to_string())),
            )
        });
        let mut used_tags = HashSet::new();
        for (table, id, class, name, description, attributes, source) in notes {
            let root = b.root(class);
            let handle = b.handle(id, source.as_deref());
            let slug = name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() {
                        c.to_ascii_lowercase()
                    } else {
                        '-'
                    }
                })
                .collect::<String>()
                .split('-')
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("-");
            let slug = if slug.is_empty() { "note".into() } else { slug };
            let mut tag = slug.clone();
            let mut suffix = 2;
            while !used_tags.insert(tag.to_lowercase()) {
                tag = format!("{slug}-{suffix}");
                suffix += 1;
            }
            let mut body = format!("# {}\n\n@tag: {tag}\n", line(&name));
            if let Some(description) = description {
                for part in html_to_nw(&description).lines() {
                    body.push_str(&format!("%Short: {part}\n"));
                }
            }
            if !attributes.is_empty() {
                body.push_str("\n## Details\n\n");
                for (key, value) in attributes.into_iter().collect::<BTreeMap<_, _>>() {
                    body.push_str(&format!(
                        "**{}:** {}\n",
                        line(&key),
                        value.replace('\n', "\n    ")
                    ));
                }
            }
            tags.insert(id, (reference_keyword(class).into(), tag));
            ids.push((table, id, handle.clone()));
            b.push(document(handle, &root, &root, class, &name, 1), body);
        }
    }
    let mut refs: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for r in db::get_all_scene_character_refs(conn, project_id).map_err(|e| e.to_string())? {
        refs.entry(r.scene_id).or_default().push(r.character_id);
    }
    for r in db::get_all_scene_location_refs(conn, project_id).map_err(|e| e.to_string())? {
        refs.entry(r.scene_id).or_default().push(r.location_id);
    }
    for r in db::get_all_scene_reference_item_refs(conn, project_id).map_err(|e| e.to_string())? {
        refs.entry(r.scene_id)
            .or_default()
            .push(r.reference_item_id);
    }
    let mut part = novel.clone();
    for ch in chapters.iter().filter(|c| !c.archived) {
        let handle = b.handle(ch.id, ch.source_id.as_deref());
        let parent = if ch.is_part { &novel } else { &part };
        let level = if ch.is_part { 1 } else { 2 };
        b.push(
            document(handle.clone(), parent, &novel, "NOVEL", &ch.title, level),
            format!("{} {}\n", "#".repeat(level as usize), line(&ch.title)),
        );
        ids.push(("chapters", ch.id, handle.clone()));
        if ch.is_part {
            part = handle.clone();
        }
        let mut chapter_scenes: Vec<_> = scenes
            .iter()
            .filter(|s| s.chapter_id == ch.id && !s.archived)
            .collect();
        chapter_scenes.sort_by_key(|s| s.position);
        for scene in chapter_scenes {
            let sh = b.handle(scene.id, scene.source_id.as_deref());
            let mut body = format!("### {}\n", line(&scene.title));
            let mut references: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for id in refs.get(&scene.id).into_iter().flatten() {
                if let Some((key, tag)) = tags.get(id) {
                    references.entry(key.clone()).or_default().push(tag.clone());
                }
            }
            for (key, mut values) in references {
                values.sort();
                values.dedup();
                body.push_str(&format!("@{key}: {}\n", values.join(", ")));
            }
            if let Some(synopsis) = &scene.synopsis {
                for s in synopsis.lines() {
                    body.push_str(&format!("%Synopsis: {s}\n"));
                }
            }
            let mut scene_beats: Vec<_> = beats
                .iter()
                .filter(|beat| beat.scene_id == scene.id)
                .collect();
            scene_beats.sort_by_key(|beat| beat.position);
            if scene.editor_mode == EditorMode::Page || scene_beats.is_empty() {
                // Page prose has no defensible beat boundaries.
                body.push_str(&format!(
                    "\n{}\n",
                    html_to_nw(scene.prose.as_deref().unwrap_or_default())
                ));
            } else {
                for beat in scene_beats {
                    body.push('\n');
                    if options.include_beat_comments {
                        body.push_str(&format!("% Beat: {}\n", line(&beat.content)));
                    }
                    body.push_str(&html_to_nw(beat.prose.as_deref().unwrap_or_default()));
                    body.push('\n');
                }
            }
            let mut item = document(sh.clone(), &handle, &novel, "NOVEL", &scene.title, 3);
            item.status = match scene.scene_status {
                SceneStatus::Draft => "s000000",
                SceneStatus::Revised => "s000001",
                SceneStatus::Final => "s000002",
            }
            .into();
            ids.push(("scenes", scene.id, sh));
            b.push(item, body);
        }
    }
    // Stage beside the destination so failed writes cannot leave a half-project.
    let parent = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stage = parent.join(format!(".kindling-nw-{}", Uuid::new_v4()));
    std::fs::create_dir(&stage).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        std::fs::create_dir(stage.join("content")).map_err(|e| e.to_string())?;
        for (handle, body) in &b.bodies {
            std::fs::write(stage.join("content").join(format!("{handle}.md")), body)
                .map_err(|e| e.to_string())?;
        }
        std::fs::write(stage.join("nwProject.nwx"), write_nwx(&b.doc))
            .map_err(|e| e.to_string())?;
        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        if matches!(
            project.source_type,
            SourceType::NovelWriter | SourceType::Blank
        ) {
            for (table, id, handle) in ids {
                tx.execute(
                    &format!("UPDATE {table} SET source_id = ?1 WHERE id = ?2"),
                    rusqlite::params![handle, id.to_string()],
                )
                .map_err(|e| e.to_string())?;
            }
        }
        // Recheck just before publishing; retain OS metadata in the new folder.
        for file in metadata_files(destination)? {
            std::fs::copy(&file, stage.join(file.file_name().unwrap()))
                .map_err(|e| e.to_string())?;
        }
        if destination.exists() {
            for file in metadata_files(destination)? {
                std::fs::remove_file(file).map_err(|e| e.to_string())?;
            }
            std::fs::remove_dir(destination).map_err(|e| e.to_string())?;
        }
        std::fs::rename(&stage, destination).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    })();
    if stage.exists() {
        let _ = std::fs::remove_dir_all(&stage);
    }
    result?;
    Ok(b.bodies.len())
}
