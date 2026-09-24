//! novelWriter adds user-reviewed prose to sync without changing the behavior of
//! outline-only sources. Preview and apply share the same change construction.
use super::sync::{
    earlier_source_siblings, open_sync_slot, ReimportSummary, SyncAddition, SyncChange,
    SyncPreview, SyncSiblings,
};
use crate::{db, models::*, parsers::novelwriter::*};
use rusqlite::Connection;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use uuid::Uuid;

/// Reads the source for sync. A document with several headings that kindling
/// 1.2 imported as one chapter or scene is read that same way, so upgrading
/// never offers its later headings as new scenes over text the project already
/// has. Such a document has its first heading's items here, none of its later
/// headings', and no record: 1.3 records every Novel document it reads, so one
/// whose later scenes the writer deleted, or that gained a heading after a 1.3
/// import, still reads per heading.
pub(super) fn load(conn: &Connection, project: &Project) -> Result<ParsedNovelWriter, String> {
    let path = project
        .source_path
        .as_deref()
        .ok_or("Project has no novelWriter source folder")?;
    let parsed = parse_novelwriter_project(Path::new(path)).map_err(|e| e.to_string())?;
    if parsed.split_documents.is_empty() {
        return Ok(parsed);
    }
    let chapters =
        db::get_all_chapters_including_archived(conn, &project.id).map_err(|e| e.to_string())?;
    let scenes =
        db::get_all_scenes_including_archived(conn, &project.id).map_err(|e| e.to_string())?;
    let known: HashSet<_> = chapters
        .iter()
        .filter_map(|c| c.source_id.as_deref())
        .chain(scenes.iter().filter_map(|s| s.source_id.as_deref()))
        .collect();
    let recorded = split_documents(conn, project)?;
    let unsplit: HashSet<_> = parsed
        .split_documents
        .iter()
        .filter(|d| {
            !recorded.contains(&d.handle)
                && d.first.iter().any(|id| known.contains(id.as_str()))
                && !d.later.iter().any(|id| known.contains(id.as_str()))
        })
        .map(|d| d.handle.clone())
        .collect();
    if unsplit.is_empty() {
        return Ok(parsed);
    }
    parse_novelwriter_project_with(Path::new(path), &unsplit).map_err(|e| e.to_string())
}
/// Novel documents this project has read one chapter or scene per heading.
fn split_documents(conn: &Connection, project: &Project) -> Result<HashSet<String>, String> {
    let mut stmt = conn
        .prepare("SELECT handle FROM novelwriter_split_documents WHERE project_id = ?1")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([project.id.to_string()], |r| r.get(0))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

/// Records the documents this read per heading, at import and after each
/// sync, so they keep reading that way whatever the writer later adds or
/// deletes. Documents read whole (a 1.2 import) are left unrecorded.
pub(crate) fn record_split_documents(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<(), String> {
    for handle in &parsed.per_heading_documents {
        conn.execute(
            "INSERT OR IGNORE INTO novelwriter_split_documents (project_id, handle) VALUES (?1, ?2)",
            rusqlite::params![project.id.to_string(), handle],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// One field as kindling holds it and as novelWriter holds it now.
struct Field {
    kind: &'static str,
    id: Uuid,
    title: String,
    field: &'static str,
    local: String,
    incoming: String,
    /// kindling's HTML for prose fields, so an accepted change keeps the
    /// formatting of paragraphs novelWriter did not touch.
    html: String,
}
fn change(
    out: &mut Vec<Field>,
    kind: &'static str,
    id: Uuid,
    title: &str,
    field: &'static str,
    before: &str,
    after: &str,
) {
    out.push(Field {
        kind,
        id,
        title: title.into(),
        field,
        local: before.into(),
        incoming: after.into(),
        html: String::new(),
    });
}
/// Records a prose field together with kindling's HTML for it.
fn prose_change(
    out: &mut Vec<Field>,
    (kind, id, title): (&'static str, Uuid, &str),
    (local, html): (&str, String),
    incoming: &str,
) {
    change(out, kind, id, title, "prose", local, incoming);
    out.last_mut().unwrap().html = html;
}
fn addition(out: &mut SyncPreview, kind: &str, source: &str, title: &str, parent: Option<&str>) {
    out.additions.push(SyncAddition {
        id: format!("{kind}-{source}"),
        item_type: kind.into(),
        title: title.into(),
        parent_title: parent.map(str::to_string),
    });
}
fn prose(value: Option<&str>) -> String {
    html_to_nw(value.unwrap_or_default())
}
#[cfg(test)]
fn scene_prose(scene: &Scene, beats: &[Beat]) -> String {
    scene_prose_html(scene, beats).0
}
/// A scene's prose as compared (novelWriter text) and as stored (HTML).
fn scene_prose_html(scene: &Scene, beats: &[Beat]) -> (String, String) {
    let mut ordered: Vec<_> = beats.iter().filter(|b| b.scene_id == scene.id).collect();
    if scene.editor_mode == EditorMode::Page || ordered.is_empty() {
        let html = scene.prose.clone().unwrap_or_default();
        return (prose(Some(html.as_str())), html);
    }
    ordered.sort_by_key(|b| b.position);
    let nw = ordered
        .iter()
        .map(|b| prose(b.prose.as_deref()))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    let html = ordered.iter().filter_map(|b| b.prose.as_deref()).collect();
    (nw, html)
}

/// Pairs novelWriter's beats with kindling's. Titles pair first, whatever their
/// order: the k-th incoming beat with a title pairs with kindling's k-th beat
/// with that title, so a reordered scene pairs every beat and changes nothing.
/// An incoming beat left over then pairs, in order, with a leftover kindling
/// beat that follows the same paired neighbour (a rename). Anything still
/// unpaired on the incoming side is an addition; kindling beats novelWriter no
/// longer has are left alone, so a deletion never moves text onto a neighbour.
fn align<'a>(incoming: &[&'a Beat], local: &[&'a Beat]) -> Vec<(&'a Beat, Option<&'a Beat>)> {
    let mut partner: Vec<Option<usize>> = vec![None; incoming.len()];
    let mut taken = vec![false; local.len()];
    let mut by_title: HashMap<&str, std::collections::VecDeque<usize>> = HashMap::new();
    for (j, beat) in local.iter().enumerate() {
        by_title.entry(&beat.content).or_default().push_back(j);
    }
    for (i, beat) in incoming.iter().enumerate() {
        if let Some(j) = by_title
            .get_mut(beat.content.as_str())
            .and_then(|q| q.pop_front())
        {
            partner[i] = Some(j);
            taken[j] = true;
        }
    }
    // Renames: group leftovers by the kindling beat they follow.
    let mut gaps: HashMap<Option<usize>, std::collections::VecDeque<usize>> = HashMap::new();
    let mut after = None;
    for (j, t) in taken.iter().enumerate() {
        if *t {
            after = Some(j);
        } else {
            gaps.entry(after).or_default().push_back(j);
        }
    }
    let mut after = None;
    for slot in partner.iter_mut() {
        match *slot {
            Some(j) => after = Some(j),
            None => *slot = gaps.get_mut(&after).and_then(|q| q.pop_front()),
        }
    }
    incoming
        .iter()
        .zip(partner)
        .map(|(b, j)| (*b, j.map(|j| local[j])))
        .collect()
}

/// Beats whose identity must change so identities follow novelWriter's current
/// order: each paired kindling beat takes its partner's identity, and a
/// kindling beat novelWriter no longer has gives up an identity that is now
/// another beat's. Additions are placed after the beat holding the identity
/// before them, so this keeps them next to the right beat and unique.
type Rekey = Vec<(Uuid, String)>;

/// Walks the source and the project the same way for preview and baselines.
fn compare(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<(Vec<Field>, SyncPreview, Rekey), String> {
    let chapters = db::get_chapters(conn, &project.id).map_err(|e| e.to_string())?;
    let scenes = db::get_all_project_scenes(conn, &project.id).map_err(|e| e.to_string())?;
    let beats = db::get_all_project_beats(conn, &project.id).map_err(|e| e.to_string())?;
    let chapters_by_source: HashMap<_, _> = chapters
        .iter()
        .filter_map(|c| c.source_id.as_deref().map(|id| (id, c)))
        .collect();
    let scenes_by_source: HashMap<_, _> = scenes
        .iter()
        .filter_map(|s| s.source_id.as_deref().map(|id| (id, s)))
        .collect();
    let locked_chapters: HashSet<_> = chapters.iter().filter(|c| c.locked).map(|c| c.id).collect();
    let mut out = SyncPreview {
        additions: vec![],
        changes: vec![],
    };
    let mut fields = vec![];
    let mut rekey = vec![];
    for chapter in &parsed.chapters {
        let source = chapter.source_id.as_deref().unwrap();
        let local_chapter = chapters_by_source.get(source).copied();
        if local_chapter.is_some_and(|c| c.locked) {
            continue;
        }
        if let Some(local) = local_chapter {
            change(
                &mut fields,
                "chapter",
                local.id,
                &local.title,
                "title",
                &local.title,
                &chapter.title,
            );
        } else {
            addition(&mut out, "chapter", source, &chapter.title, None);
        }
        for scene in parsed.scenes.iter().filter(|s| s.chapter_id == chapter.id) {
            let source = scene.source_id.as_deref().unwrap();
            let local = scenes_by_source.get(source).copied();
            if local.is_some_and(|s| s.locked || locked_chapters.contains(&s.chapter_id)) {
                continue;
            }
            if let Some(local) = local {
                change(
                    &mut fields,
                    "scene",
                    local.id,
                    &local.title,
                    "title",
                    &local.title,
                    &scene.title,
                );
                change(
                    &mut fields,
                    "scene",
                    local.id,
                    &local.title,
                    "synopsis",
                    local.synopsis.as_deref().unwrap_or_default(),
                    scene.synopsis.as_deref().unwrap_or_default(),
                );
                if local.editor_mode == EditorMode::Page
                    || !parsed.scenes_with_beat_comments.contains(source)
                {
                    let (nw, html) = scene_prose_html(local, &beats);
                    prose_change(
                        &mut fields,
                        ("scene", local.id, &local.title),
                        (&nw, html),
                        &prose(scene.prose.as_deref()),
                    );
                    // Unmarked text cannot identify or replace local planning beats.
                    if !parsed.scenes_with_beat_comments.contains(source) {
                        continue;
                    }
                }
            } else {
                addition(
                    &mut out,
                    "scene",
                    source,
                    &scene.title,
                    Some(&chapter.title),
                );
            }
            let incoming: Vec<_> = parsed
                .beats
                .iter()
                .filter(|b| b.scene_id == scene.id)
                .collect();
            // Only beats that came from novelWriter take part; local splits stay local.
            let mut synced: Vec<_> = beats
                .iter()
                .filter(|b| local.is_some_and(|s| s.id == b.scene_id) && b.source_id.is_some())
                .collect();
            synced.sort_by_key(|b| b.position);
            let pairs = align(&incoming, &synced);
            let current: HashSet<_> = incoming
                .iter()
                .filter_map(|b| b.source_id.as_deref())
                .collect();
            for (beat, existing) in &pairs {
                if let Some(existing) = existing.filter(|e| e.source_id != beat.source_id) {
                    rekey.push((existing.id, beat.source_id.clone().unwrap()));
                }
            }
            for orphan in &synced {
                let paired = pairs
                    .iter()
                    .any(|(_, e)| e.is_some_and(|e| e.id == orphan.id));
                if !paired
                    && orphan
                        .source_id
                        .as_deref()
                        .is_some_and(|s| current.contains(s))
                {
                    rekey.push((orphan.id, format!("novelwriter:beat:kept:{}", orphan.id)));
                }
            }
            for (beat, existing) in pairs {
                let source = beat.source_id.as_deref().unwrap();
                if let Some(existing) = existing {
                    change(
                        &mut fields,
                        "beat",
                        existing.id,
                        &existing.content,
                        "content",
                        &existing.content,
                        &beat.content,
                    );
                    if local.is_some_and(|s| s.editor_mode == EditorMode::Beat) {
                        let html = existing.prose.clone().unwrap_or_default();
                        prose_change(
                            &mut fields,
                            ("beat", existing.id, &existing.content),
                            (&prose(Some(html.as_str())), html),
                            &prose(beat.prose.as_deref()),
                        );
                    }
                } else {
                    addition(&mut out, "beat", source, &beat.content, Some(&scene.title));
                }
            }
        }
    }
    Ok((fields, out, rekey))
}

type Baselines = HashMap<(String, String), String>;

/// The novelWriter text of each field at the last import or sync, by item and field.
fn baselines(conn: &Connection, project: &Project) -> Result<Baselines, String> {
    let mut stmt = conn
        .prepare(
            "SELECT item_id, field, value FROM novelwriter_sync_baselines WHERE project_id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([project.id.to_string()], |r| {
            Ok(((r.get(0)?, r.get(1)?), r.get(2)?))
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<_, _>>().map_err(|e| e.to_string())
}

/// Settle a field at novelWriter's current text, writing only when that changes it.
fn settle(
    conn: &Connection,
    project: &Project,
    baselines: &mut Baselines,
    f: &Field,
) -> Result<(), String> {
    let key = (f.id.to_string(), f.field.to_string());
    if baselines.get(&key) == Some(&f.incoming) {
        return Ok(());
    }
    conn.prepare_cached(
        "INSERT INTO novelwriter_sync_baselines (project_id, item_id, field, value)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(item_id, field) DO UPDATE SET value = excluded.value",
    )
    .and_then(|mut stmt| {
        stmt.execute(rusqlite::params![
            project.id.to_string(),
            key.0,
            f.field,
            f.incoming
        ])
    })
    .map_err(|e| e.to_string())?;
    baselines.insert(key, f.incoming.clone());
    Ok(())
}

/// Settle every field kindling already matches (used at import and for new items).
pub(crate) fn record_baselines(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<(), String> {
    let (fields, _, _) = compare(conn, project, parsed)?;
    let mut known = baselines(conn, project)?;
    for f in fields.iter().filter(|f| f.local == f.incoming) {
        settle(conn, project, &mut known, f)?;
    }
    Ok(())
}

/// Only fields novelWriter changed since the baseline are offered, paired with
/// whether they conflict. When kindling changed too, or there is no baseline
/// (imported before baselines existed), the writer must pick individually.
fn offered<'a>(fields: &'a [Field], baselines: &Baselines) -> Vec<(&'a Field, bool)> {
    fields
        .iter()
        .filter(|f| f.local != f.incoming)
        .filter_map(|f| {
            let base = baselines.get(&(f.id.to_string(), f.field.to_string()));
            // novelWriter unchanged since the baseline: the difference is a kindling edit.
            (base != Some(&f.incoming)).then_some((f, base != Some(&f.local)))
        })
        .collect()
}
/// Binds a change to the exact texts the writer reviewed, so apply can refuse
/// anything that changed on either side after the preview.
fn change_id(f: &Field) -> String {
    let reviewed = stable_handle(&format!("{}\u{1f}{}", f.local, f.incoming));
    format!("{}-{}-{}-{reviewed}", f.kind, f.field, f.id)
}

pub(super) fn preview(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<SyncPreview, String> {
    let (fields, mut out, _) = compare(conn, project, parsed)?;
    let baselines = baselines(conn, project)?;
    for (f, conflict) in offered(&fields, &baselines) {
        out.changes.push(SyncChange {
            id: change_id(f),
            item_type: f.kind.into(),
            field: f.field.into(),
            item_title: f.title.clone(),
            conflict,
            current_value: f.local.clone(),
            new_value: f.incoming.clone(),
            db_id: f.id.to_string(),
        });
    }
    Ok(out)
}

/// Splits HTML into its top-level blocks (`<p>`, `<ul>`, `<blockquote>`, ...).
fn blocks(html: &str) -> Vec<&str> {
    // Block bounds as start/end pairs, sliced once at the end.
    let mut out = vec![];
    let (mut depth, mut start, mut i) = (0usize, 0, 0);
    let text = |from: usize, to: usize, out: &mut Vec<usize>| {
        if !html[from..to].trim().is_empty() {
            out.extend([from, to]);
        }
    };
    while let Some(open) = html[i..].find('<').map(|o| i + o) {
        let Some(end) = html[open..].find('>').map(|e| open + e + 1) else {
            break;
        };
        let tag = &html[open + 1..end - 1];
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_ascii_lowercase();
        let void = tag.ends_with('/')
            || tag.starts_with('!')
            || matches!(name.as_str(), "br" | "hr" | "img" | "wbr");
        if tag.starts_with('/') {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                text(start, end, &mut out);
                start = end;
            }
        } else if depth == 0 {
            text(start, open, &mut out);
            start = open;
            if void {
                text(open, end, &mut out);
                start = end;
            } else {
                depth = 1;
            }
        } else if !void {
            depth += 1;
        }
        i = end;
    }
    text(start, html.len(), &mut out);
    out.chunks(2).map(|b| &html[b[0]..b[1]]).collect()
}

/// Builds the accepted prose from novelWriter's text while keeping kindling's
/// HTML for every block whose text novelWriter left unchanged. novelWriter
/// cannot express underline, alignment or lists, so rebuilding the whole field
/// from its text would silently strip them from untouched paragraphs. Blocks
/// with no text (an `<hr>` left between former beats, an empty paragraph) are
/// structure novelWriter never saw, so they stay where they are.
fn merge_prose(local_html: &str, incoming: &str) -> String {
    let incoming: Vec<&str> = incoming
        .split("\n\n")
        .filter(|p| !p.trim().is_empty())
        .collect();
    let local: Vec<(&str, Vec<String>)> = blocks(local_html)
        .into_iter()
        .map(|b| {
            let nw = html_to_nw(b);
            let paragraphs = nw.split("\n\n").filter(|p| !p.is_empty());
            (b, paragraphs.map(str::to_string).collect::<Vec<_>>())
        })
        .collect();
    let text: Vec<usize> = (0..local.len())
        .filter(|&i| !local[i].1.is_empty())
        .collect();
    // best[t][j]: most incoming paragraphs covered by keeping whole text blocks
    // t.. in order against incoming paragraphs j..
    let (n, m) = (text.len(), incoming.len());
    let size = |t: usize| local[text[t]].1.len();
    let fits = |t: usize, j: usize| {
        let p = &local[text[t]].1;
        j + p.len() <= m && p.iter().zip(&incoming[j..]).all(|(a, b)| a == b)
    };
    let mut best = vec![vec![0usize; m + 1]; n + 1];
    for t in (0..n).rev() {
        for j in (0..=m).rev() {
            let mut score = best[t + 1][j];
            if j < m {
                score = score.max(best[t][j + 1]);
            }
            if fits(t, j) {
                score = score.max(size(t) + best[t + 1][j + size(t)]);
            }
            best[t][j] = score;
        }
    }
    // Kept blocks, as (block index, first incoming paragraph it covers).
    let mut kept = vec![];
    let (mut t, mut j) = (0, 0);
    while t < n && j < m {
        if fits(t, j) && best[t][j] == size(t) + best[t + 1][j + size(t)] {
            kept.push((text[t], j));
            j += size(t);
            t += 1;
        } else if best[t][j] == best[t + 1][j] {
            t += 1;
        } else {
            j += 1;
        }
    }
    kept.push((local.len(), m));
    // Between kept blocks, each dropped text block gives its place to the next
    // incoming paragraph; paragraphs left over follow the segment.
    let mut out = String::new();
    let (mut block, mut next) = (0, 0);
    for (keep, from) in kept {
        for (html, paragraphs) in &local[block..keep] {
            if paragraphs.is_empty() {
                out.push_str(html);
            } else if next < from {
                out.push_str(&nw_to_html(incoming[next]));
                next += 1;
            }
        }
        for paragraph in &incoming[next..from] {
            out.push_str(&nw_to_html(paragraph));
        }
        if keep < local.len() {
            out.push_str(local[keep].0);
            next = from + local[keep].1.len();
        }
        block = keep + 1;
    }
    out
}

/// Refuses ids that no longer match what is offered now (either side changed
/// since the preview), and says whether the accepted ones replace any prose.
fn validate(
    fields: &[Field],
    known: &Baselines,
    accepted: &[String],
    kept: &[String],
) -> Result<bool, String> {
    let now: HashMap<_, _> = offered(fields, known)
        .into_iter()
        .map(|(f, conflict)| (change_id(f), (conflict, f.field)))
        .collect();
    if accepted.iter().any(|id| !now.contains_key(id))
        || kept.iter().any(|id| now.get(id).map(|n| n.0) != Some(true))
    {
        return Err("The novelWriter project or these items changed after you reviewed them. Nothing was changed; open Sync again to review the current text.".into());
    }
    Ok(accepted.iter().any(|id| now[id].1 == "prose"))
}

/// Applies the accepted changes and additions. Each conflict in `kept` (shown to
/// the writer and left unticked) is settled as "keep kindling": its baseline
/// moves to novelWriter's current text so it is not offered again until
/// novelWriter changes it. Unaccepted non-conflicting changes keep their
/// baseline and are offered again. Reimport passes no `kept` ids because it
/// shows the writer nothing.
///
/// Change ids carry the reviewed texts, so if either side changed after the
/// preview (a novelWriter autosave, say) nothing is written and the writer is
/// asked to review again.
pub(super) fn apply(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
    accepted: &[String],
    additions: &[String],
    kept: &[String],
    before_prose: &mut dyn FnMut(&Connection) -> Result<(), String>,
) -> Result<ReimportSummary, String> {
    // Compare and check once, before writing anything: a stale apply is
    // refused here, and `before_prose` (the pre-sync snapshot) runs only when
    // current changes will replace prose. The caller holds the connection, so
    // nothing changes between this check and the writes below.
    let (fields, preview, rekey) = compare(conn, project, parsed)?;
    let mut known = baselines(conn, project)?;
    if validate(&fields, &known, accepted, kept)? {
        before_prose(conn)?;
    }
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let kept: HashSet<_> = kept.iter().collect();
    let accepted: HashSet<_> = accepted.iter().collect();
    let additions: HashSet<_> = additions.iter().collect();
    let available: HashSet<_> = preview.additions.iter().map(|a| &a.id).collect();
    let mut summary = ReimportSummary {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
        prose_updated: 0,
    };
    let scenes: HashMap<_, _> = db::get_all_project_scenes(&tx, &project.id)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|s| (s.id, s))
        .collect();
    for (change, conflict) in offered(&fields, &known) {
        if !accepted.contains(&change_id(change)) {
            if change.field == "prose" {
                summary.prose_preserved += 1;
            }
            if conflict && kept.contains(&change_id(change)) {
                settle(&tx, project, &mut known, change)?;
            }
            continue;
        }
        let id = change.id;
        if change.field == "prose" {
            let html = merge_prose(&change.html, &change.incoming);
            if change.kind == "beat" {
                db::update_beat_prose(&tx, &id, &html).map_err(|e| e.to_string())?;
            } else {
                let scene = scenes.get(&id).ok_or("Scene no longer exists")?;
                if scene.editor_mode == EditorMode::Page {
                    db::update_scene_prose(&tx, &id, &html).map_err(|e| e.to_string())?;
                } else {
                    // A single accepted scene replacement has no beat boundaries.
                    // Keep planning beats and mode; place text in the first beat
                    // and clear the remaining prose as part of that same choice.
                    let beats = db::get_beats(&tx, &id).map_err(|e| e.to_string())?;
                    if beats.is_empty() {
                        let mut beat = Beat::new(id, "Scene Content".into(), 0);
                        beat.prose = Some(html);
                        beat.source_id = scene
                            .source_id
                            .as_deref()
                            .map(|s| novelwriter_beat_source_id(s, 0));
                        db::insert_beat(&tx, &beat).map_err(|e| e.to_string())?;
                    } else {
                        for (i, beat) in beats.iter().enumerate() {
                            db::update_beat_prose(&tx, &beat.id, if i == 0 { &html } else { "" })
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
            summary.prose_updated += 1;
        } else {
            // The table/column pair comes exclusively from our own comparison.
            let (table, column) = match (change.kind, change.field) {
                ("chapter", "title") => {
                    summary.chapters_updated += 1;
                    ("chapters", "title")
                }
                ("scene", "title") => {
                    summary.scenes_updated += 1;
                    ("scenes", "title")
                }
                ("scene", "synopsis") => {
                    summary.scenes_updated += 1;
                    ("scenes", "synopsis")
                }
                ("beat", "content") => {
                    summary.beats_updated += 1;
                    ("beats", "content")
                }
                _ => return Err("Unknown novelWriter sync field".into()),
            };
            tx.execute(
                &format!("UPDATE {table} SET {column} = ?1 WHERE id = ?2"),
                rusqlite::params![change.incoming, id.to_string()],
            )
            .map_err(|e| e.to_string())?;
        }
        settle(&tx, project, &mut known, change)?;
    }
    // Both sides already agree (e.g. a pre-baseline project): nothing to write
    // unless the baseline is missing or stale.
    for f in fields.iter().filter(|f| f.local == f.incoming) {
        settle(&tx, project, &mut known, f)?;
    }
    // Before any addition is placed by the identity that precedes it.
    for (id, source) in &rekey {
        db::update_beat_source_id(&tx, id, source).map_err(|e| e.to_string())?;
    }
    let mut chapter_ids = HashMap::new();
    for c in &parsed.chapters {
        let source = c.source_id.as_deref().unwrap();
        if let Some(existing) =
            db::find_chapter_by_source_id(&tx, &project.id, source).map_err(|e| e.to_string())?
        {
            chapter_ids.insert(c.id, existing.id);
        } else {
            let key = format!("chapter-{source}");
            if additions.contains(&key) && available.contains(&key) {
                let earlier = earlier_source_siblings(
                    parsed.chapters.iter().map(|c| (c.position, &c.source_id)),
                    c.position,
                );
                let mut c = c.clone();
                c.project_id = project.id;
                c.position = open_sync_slot(&tx, SyncSiblings::Chapters, &project.id, &earlier)?;
                db::insert_chapter(&tx, &c).map_err(|e| e.to_string())?;
                chapter_ids.insert(c.id, c.id);
                summary.chapters_added += 1;
            }
        }
    }
    let mut scene_ids = HashMap::new();
    for s in &parsed.scenes {
        let Some(chapter) = chapter_ids.get(&s.chapter_id) else {
            continue;
        };
        let source = s.source_id.as_deref().unwrap();
        if let Some(existing) =
            db::find_scene_by_source_id(&tx, chapter, source).map_err(|e| e.to_string())?
        {
            scene_ids.insert(s.id, existing.id);
        } else {
            let key = format!("scene-{source}");
            if additions.contains(&key) && available.contains(&key) {
                let earlier = earlier_source_siblings(
                    parsed
                        .scenes
                        .iter()
                        .filter(|o| o.chapter_id == s.chapter_id)
                        .map(|o| (o.position, &o.source_id)),
                    s.position,
                );
                let mut s = s.clone();
                s.chapter_id = *chapter;
                s.position = open_sync_slot(&tx, SyncSiblings::Scenes, chapter, &earlier)?;
                db::insert_scene(&tx, &s).map_err(|e| e.to_string())?;
                scene_ids.insert(s.id, s.id);
                summary.scenes_added += 1;
            }
        }
    }
    for b in &parsed.beats {
        let Some(scene) = scene_ids.get(&b.scene_id) else {
            continue;
        };
        let key = format!("beat-{}", b.source_id.as_deref().unwrap());
        if additions.contains(&key) && available.contains(&key) {
            let earlier = earlier_source_siblings(
                parsed
                    .beats
                    .iter()
                    .filter(|o| o.scene_id == b.scene_id)
                    .map(|o| (o.position, &o.source_id)),
                b.position,
            );
            let mut b = b.clone();
            b.scene_id = *scene;
            b.position = open_sync_slot(&tx, SyncSiblings::Beats, scene, &earlier)?;
            db::insert_beat(&tx, &b).map_err(|e| e.to_string())?;
            summary.beats_added += 1;
        }
    }
    // New items only exist after insertion, so they need one more comparison.
    if summary.chapters_added + summary.scenes_added + summary.beats_added > 0 {
        record_baselines(&tx, project, parsed)?;
    }
    record_split_documents(&tx, project, parsed)?;
    db::update_project_modified(&tx, &project.id).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::novelwriter::tests::fixture;
    fn imported() -> (Connection, Project, tempfile::TempDir) {
        let (conn, project) = fixture();
        let temp = tempfile::tempdir().unwrap();
        export_novelwriter_project(&conn, &project.id, temp.path(), &Default::default()).unwrap();
        let parsed = parse_novelwriter_project(temp.path()).unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        super::super::import::insert_novelwriter(&conn, &parsed).unwrap();
        (conn, parsed.project, temp)
    }
    #[test]
    fn no_beats_use_scene_prose_even_when_other_scenes_have_beats() {
        let mut scene = Scene::new(Uuid::new_v4(), "Scene".into(), None, 0);
        scene.prose = Some("<p>Fallback manuscript</p>".into());
        let unrelated = Beat::new(Uuid::new_v4(), "Other scene".into(), 0);
        assert_eq!(scene_prose(&scene, &[unrelated]), "Fallback manuscript");
        let mut beat = Beat::new(scene.id, "Outline".into(), 0);
        beat.prose = Some("<p>Beat manuscript</p>".into());
        assert_eq!(scene_prose(&scene, &[beat.clone()]), "Beat manuscript");
        scene.editor_mode = EditorMode::Page;
        assert_eq!(scene_prose(&scene, &[beat]), "Fallback manuscript");
    }

    #[test]
    fn selective_prose_sync_and_locks() {
        let (conn, project, _temp) = imported();
        let mut parsed = load(&conn, &project).unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        let local = db::get_all_project_beats(&conn, &project.id).unwrap();
        // TipTap representation changes alone do not produce false prose changes.
        db::update_beat_prose(
            &conn,
            &local[0].id,
            &local[0]
                .prose
                .clone()
                .unwrap()
                .replace("strong", "b")
                .replace("em>", "i>"),
        )
        .unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        parsed.beats[0].prose = Some("<p>The visitor called.</p>".into());
        parsed.beats[1].prose = Some("<p>Unaccepted.</p>".into());
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 2);
        assert!(changes.iter().all(|c| c.field == "prose"));
        let summary = apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!((summary.prose_updated, summary.prose_preserved), (1, 1));
        let updated = db::get_all_project_beats(&conn, &project.id).unwrap();
        assert_eq!(
            updated[0].prose.as_deref(),
            Some("<p>The visitor called.</p>")
        );
        assert_eq!(updated[1].prose, local[1].prose);
        conn.execute("UPDATE chapters SET locked = 1", []).unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        assert!(apply(
            &conn,
            &project,
            &parsed,
            &[changes[1].id.clone()],
            &[],
            &[],
            &mut |_| Ok(())
        )
        .is_err());
        conn.execute("UPDATE chapters SET locked = 0", []).unwrap();
        conn.execute("UPDATE scenes SET locked = 1", []).unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
    }
    /// Rewrites the scene's novelWriter document the way an edit in novelWriter would.
    fn edit_source(
        temp: &tempfile::TempDir,
        conn: &Connection,
        project: &Project,
        from: &str,
        to: &str,
    ) {
        let handle = db::get_all_project_scenes(conn, &project.id).unwrap()[0]
            .source_id
            .clone()
            .unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains(from));
        std::fs::write(file, text.replace(from, to)).unwrap();
    }

    #[test]
    fn local_edits_are_never_incoming_and_both_side_edits_are_conflicts() {
        let (conn, project, temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        // Revise a beat in kindling, then fix a typo in a different beat in novelWriter.
        db::update_beat_prose(&conn, &beats[0].id, "<p>Revised in kindling.</p>").unwrap();
        edit_source(&temp, &conn, &project, "listened", "listened closely");
        let parsed = load(&conn, &project).unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1, "{changes:?}");
        assert_eq!(changes[0].db_id, beats[1].id.to_string());
        assert!(!changes[0].conflict);
        // "All" + Apply: the novelWriter fix lands and the kindling revision survives.
        let all: Vec<_> = changes.iter().map(|c| c.id.clone()).collect();
        apply(&conn, &project, &parsed, &all, &[], &[], &mut |_| Ok(())).unwrap();
        let after = db::get_all_project_beats(&conn, &project.id).unwrap();
        assert_eq!(
            after[0].prose.as_deref(),
            Some("<p>Revised in kindling.</p>")
        );
        assert!(after[1]
            .prose
            .as_deref()
            .unwrap()
            .contains("listened closely"));
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());

        // A declined incoming change is offered again rather than forgotten.
        edit_source(&temp, &conn, &project, "listened closely", "listened hard");
        let parsed = load(&conn, &project).unwrap();
        apply(&conn, &project, &parsed, &[], &[], &[], &mut |_| Ok(())).unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!((changes.len(), changes[0].conflict), (1, false));

        // Both sides changed since the last sync: a conflict showing both texts.
        db::update_beat_prose(&conn, &beats[1].id, "<p>Also revised here.</p>").unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1);
        assert!(changes[0].conflict);
        assert_eq!(changes[0].current_value, "Also revised here.");
        assert!(changes[0].new_value.contains("listened hard"));
        // Explicitly accepting a conflict applies it and settles the baseline.
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());

        // Projects imported before baselines existed cannot tell who changed what.
        conn.execute("DELETE FROM novelwriter_sync_baselines", [])
            .unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].db_id, beats[0].id.to_string());
        assert!(changes[0].conflict);
    }

    #[test]
    fn declined_conflicts_keep_kindling_until_novelwriter_changes_again() {
        let (conn, project, temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        db::update_beat_prose(&conn, &beats[1].id, "<p>Kept in kindling.</p>").unwrap();
        edit_source(&temp, &conn, &project, "listened", "listened closely");
        let parsed = load(&conn, &project).unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!((changes.len(), changes[0].conflict), (1, true));
        // Reimport shows the writer nothing, so it must not settle the conflict.
        apply(&conn, &project, &parsed, &[], &[], &[], &mut |_| Ok(())).unwrap();
        assert_eq!(preview(&conn, &project, &parsed).unwrap().changes.len(), 1);
        // Applying with the conflict unticked keeps kindling's text and settles it.
        let kept_ids = [changes[0].id.clone()];
        let summary = apply(&conn, &project, &parsed, &[], &[], &kept_ids, &mut |_| {
            Ok(())
        })
        .unwrap();
        assert_eq!((summary.prose_updated, summary.prose_preserved), (0, 1));
        let kept = || db::get_beat(&conn, &beats[1].id).unwrap().unwrap().prose;
        assert_eq!(kept().as_deref(), Some("<p>Kept in kindling.</p>"));
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        // A later novelWriter edit to the same field is offered again.
        edit_source(&temp, &conn, &project, "listened closely", "listened hard");
        let parsed = load(&conn, &project).unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!((changes.len(), changes[0].conflict), (1, true));
        assert!(changes[0].new_value.contains("listened hard"));

        // Pre-baseline projects: every local edit starts as a conflict, and
        // keeping kindling's side is enough to get out of it.
        conn.execute("DELETE FROM novelwriter_sync_baselines", [])
            .unwrap();
        db::update_beat_prose(&conn, &beats[0].id, "<p>Also local.</p>").unwrap();
        let shown = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(shown.len(), 2);
        let kept_ids: Vec<_> = shown.iter().map(|c| c.id.clone()).collect();
        apply(&conn, &project, &parsed, &[], &[], &kept_ids, &mut |_| {
            Ok(())
        })
        .unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        assert_eq!(kept().as_deref(), Some("<p>Kept in kindling.</p>"));
    }

    #[test]
    fn apply_writes_only_baselines_that_change() {
        let (conn, project, temp) = imported();
        conn.execute_batch(
            "CREATE TEMP TABLE writes (n INTEGER);
             CREATE TEMP TRIGGER bi AFTER INSERT ON main.novelwriter_sync_baselines
                 BEGIN INSERT INTO writes VALUES (1); END;
             CREATE TEMP TRIGGER bu AFTER UPDATE ON main.novelwriter_sync_baselines
                 BEGIN INSERT INTO writes VALUES (1); END;",
        )
        .unwrap();
        let writes = || -> i64 {
            conn.query_row("SELECT COUNT(*) FROM writes", [], |r| r.get(0))
                .unwrap()
        };
        let parsed = load(&conn, &project).unwrap();
        apply(&conn, &project, &parsed, &[], &[], &[], &mut |_| Ok(())).unwrap();
        assert_eq!(writes(), 0, "an unchanged project rewrites no baselines");
        edit_source(&temp, &conn, &project, "listened", "listened closely");
        let parsed = load(&conn, &project).unwrap();
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(writes(), 1, "only the accepted field's baseline moves");
    }

    #[test]
    fn page_and_unmarked_scene_prose_use_correct_home() {
        let (conn, project, temp) = imported();
        let mut parsed = load(&conn, &project).unwrap();
        let local = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .remove(0);
        conn.execute(
            "UPDATE scenes SET editor_mode = 'page', prose = '<p>Local page</p>' WHERE id = ?1",
            [local.id.to_string()],
        )
        .unwrap();
        parsed.scenes[0].prose = Some("<p>Incoming page</p>".into());
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].item_type, "scene");
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        let scene = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .remove(0);
        assert_eq!(scene.editor_mode, EditorMode::Page);
        assert_eq!(scene.prose.as_deref(), Some("<p>Incoming page</p>"));
        conn.execute("UPDATE scenes SET editor_mode = 'beat'", [])
            .unwrap();
        let handle = local.source_id.unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        std::fs::write(file, "### Arrival\n%Synopsis: A stranger arrives.\n%Synopsis: At midnight.\n\nWhole replacement.\n").unwrap();
        let parsed = load(&conn, &project).unwrap();
        let diff = preview(&conn, &project, &parsed).unwrap();
        assert_eq!(diff.changes.len(), 1);
        assert!(diff.additions.is_empty());
        assert_eq!(diff.changes[0].item_type, "scene");
        apply(
            &conn,
            &project,
            &parsed,
            &[diff.changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        let beats = db::get_beats(&conn, &scene.id).unwrap();
        assert_eq!(beats.len(), 2);
        assert_eq!(beats[0].prose.as_deref(), Some("<p>Whole replacement.</p>"));
        assert_eq!(beats[1].prose.as_deref(), Some(""));
        assert_eq!(
            db::get_all_project_scenes(&conn, &project.id).unwrap()[0].editor_mode,
            EditorMode::Beat
        );
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
    }
    #[test]
    fn scene_added_at_the_head_of_a_chapter_takes_its_own_position() {
        let (conn, project, _temp) = imported();
        let mut parsed = load(&conn, &project).unwrap();
        let chapter = parsed.scenes[0].chapter_id;
        for s in parsed.scenes.iter_mut().filter(|s| s.chapter_id == chapter) {
            s.position += 1;
        }
        let mut added = Scene::new(chapter, "Cold open".into(), None, 0);
        added.source_id = Some("0000000000abc".into());
        parsed.scenes.insert(0, added.clone());
        let db_chapter = db::get_all_project_scenes(&conn, &project.id).unwrap()[0].chapter_id;
        let before: Vec<_> = db::get_scenes(&conn, &db_chapter)
            .unwrap()
            .into_iter()
            .map(|s| s.title)
            .collect();

        let additions: Vec<_> = preview(&conn, &project, &parsed)
            .unwrap()
            .additions
            .into_iter()
            .map(|a| a.id)
            .collect();
        assert_eq!(additions, ["scene-0000000000abc"]);
        let summary = apply(&conn, &project, &parsed, &[], &additions, &[], &mut |_| {
            Ok(())
        })
        .unwrap();
        assert_eq!(summary.scenes_added, 1);

        let after = db::get_scenes(&conn, &db_chapter).unwrap();
        let titles: Vec<_> = after.iter().map(|s| s.title.clone()).collect();
        let expected: Vec<_> = std::iter::once("Cold open".to_string())
            .chain(before)
            .collect();
        assert_eq!(titles, expected);
        let positions: Vec<_> = after.iter().map(|s| s.position).collect();
        assert_eq!(positions, (0..after.len() as i32).collect::<Vec<_>>());
    }

    #[test]
    fn split_beats_remain_local_and_preview_never_backfills() {
        let (conn, project, temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        let scene = beats[0].scene_id;
        db::shift_beat_positions(&conn, &scene, 1).unwrap();
        let mut split = Beat::new(scene, "Split off locally".into(), 1);
        split.prose = Some("<p>Keep this local prose.</p>".into());
        db::insert_beat(&conn, &split).unwrap();
        let mut parsed = load(&conn, &project).unwrap();
        parsed.beats[1].prose = Some("<p>Incoming second beat.</p>".into());
        for _ in 0..2 {
            let changes = preview(&conn, &project, &parsed).unwrap().changes;
            assert_eq!(changes.len(), 1);
            assert_eq!(changes[0].db_id, beats[1].id.to_string());
            assert!(db::get_beat(&conn, &split.id)
                .unwrap()
                .unwrap()
                .source_id
                .is_none());
        }
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(
            db::get_beat(&conn, &split.id).unwrap().unwrap().prose,
            split.prose
        );
        assert_eq!(
            db::get_beat(&conn, &beats[1].id)
                .unwrap()
                .unwrap()
                .prose
                .as_deref(),
            Some("<p>Incoming second beat.</p>")
        );
        // Beats are matched by title, so a split that shares an identity is
        // simply left alone rather than making the scene unsyncable.
        db::update_beat_source_id(&conn, &split.id, beats[1].source_id.as_deref().unwrap())
            .unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        // The reviewed change was already applied, so replaying it is refused.
        assert!(apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(())
        )
        .is_err());
        std::fs::remove_file(temp.path().join("nwProject.nwx")).unwrap();
        assert!(load(&conn, &project).unwrap_err().contains("nwProject.nwx"));
    }

    #[test]
    fn beats_are_matched_by_title_so_a_deleted_beat_never_shifts_text() {
        let (conn, project, temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        let handle = db::get_all_project_scenes(&conn, &project.id).unwrap()[0]
            .source_id
            .clone()
            .unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        let original = std::fs::read_to_string(&file).unwrap();
        let preview_with = |text: String| {
            std::fs::write(&file, text).unwrap();
            preview(&conn, &project, &load(&conn, &project).unwrap()).unwrap()
        };
        // Deleting the first beat in novelWriter must not move "Answer" onto it.
        let (knock, answer) = (
            original.find("% Beat: The knock").unwrap(),
            original.find("% Beat: Answer").unwrap(),
        );
        let deleted = preview_with(format!("{}{}", &original[..knock], &original[answer..]));
        assert!(deleted.changes.is_empty(), "{:?}", deleted.changes);
        assert!(deleted.additions.is_empty());
        // A renamed beat still pairs with its kindling beat.
        let renamed = preview_with(original.replace("% Beat: Answer", "% Beat: Reply"));
        assert_eq!(renamed.changes.len(), 1, "{:?}", renamed.changes);
        assert_eq!(renamed.changes[0].db_id, beats[1].id.to_string());
        assert_eq!(renamed.changes[0].new_value, "Reply");
        // An inserted beat is an addition and leaves its neighbours alone.
        let inserted = preview_with(original.replace(
            "% Beat: Answer",
            "% Beat: Middle\nNew middle text.\n\n% Beat: Answer",
        ));
        assert!(inserted.changes.is_empty(), "{:?}", inserted.changes);
        assert_eq!(inserted.additions.len(), 1);
        assert_eq!(inserted.additions[0].title, "Middle");
    }

    /// The scene document with its two beats swapped.
    fn swapped(original: &str) -> String {
        let (knock, answer) = (
            original.find("% Beat: The knock").unwrap(),
            original.find("% Beat: Answer").unwrap(),
        );
        format!(
            "{}{}\n\n{}\n",
            &original[..knock],
            original[answer..].trim_end(),
            original[knock..answer].trim_end()
        )
    }

    #[test]
    fn added_beats_follow_their_paired_neighbour_with_unique_identities() {
        let (conn, project, temp) = imported();
        let scene = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .remove(0);
        let file = temp
            .path()
            .join("content")
            .join(format!("{}.md", scene.source_id.as_deref().unwrap()));
        let original = std::fs::read_to_string(&file).unwrap();
        // novelWriter deletes "The knock" and adds "New" after "Answer".
        let (knock, answer) = (
            original.find("% Beat: The knock").unwrap(),
            original.find("% Beat: Answer").unwrap(),
        );
        std::fs::write(
            &file,
            format!(
                "{}{}\n\n% Beat: New\nNew closing text.\n",
                &original[..knock],
                original[answer..].trim_end()
            ),
        )
        .unwrap();
        let parsed = load(&conn, &project).unwrap();
        let diff = preview(&conn, &project, &parsed).unwrap();
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
        let additions: Vec<_> = diff.additions.iter().map(|a| a.id.clone()).collect();
        assert_eq!(additions.len(), 1);
        apply(&conn, &project, &parsed, &[], &additions, &[], &mut |_| {
            Ok(())
        })
        .unwrap();
        let beats = db::get_beats(&conn, &scene.id).unwrap();
        let titles: Vec<_> = beats.iter().map(|b| b.content.as_str()).collect();
        assert_eq!(titles, ["The knock", "Answer", "New"]);
        let ids: HashSet<_> = beats.iter().map(|b| b.source_id.clone()).collect();
        assert_eq!(ids.len(), beats.len(), "{beats:?}");
        // Nothing is offered again, and the paired beats keep syncing.
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .additions
            .is_empty());
        std::fs::write(
            &file,
            std::fs::read_to_string(&file)
                .unwrap()
                .replace("New closing text.", "New closing text, revised."),
        )
        .unwrap();
        let changes = preview(&conn, &project, &load(&conn, &project).unwrap())
            .unwrap()
            .changes;
        assert_eq!(changes.len(), 1, "{changes:?}");
        assert_eq!(changes[0].db_id, beats[2].id.to_string());
    }

    #[test]
    fn reordered_beats_pair_every_beat_and_add_nothing() {
        let (conn, project, temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        let handle = db::get_all_project_scenes(&conn, &project.id).unwrap()[0]
            .source_id
            .clone()
            .unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        let original = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, swapped(&original)).unwrap();
        let parsed = load(&conn, &project).unwrap();
        let diff = preview(&conn, &project, &parsed).unwrap();
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
        assert!(diff.additions.is_empty(), "{:?}", diff.additions);
        // Reimport accepts every addition, so it must not duplicate a beat either.
        apply(&conn, &project, &parsed, &[], &[], &[], &mut |_| Ok(())).unwrap();
        assert_eq!(
            db::get_all_project_beats(&conn, &project.id).unwrap().len(),
            2
        );
        // An edit made while reordering lands on the beat it belongs to.
        std::fs::write(
            &file,
            swapped(&original).replace("listened", "listened closely"),
        )
        .unwrap();
        let changes = preview(&conn, &project, &load(&conn, &project).unwrap())
            .unwrap()
            .changes;
        assert_eq!(changes.len(), 1, "{changes:?}");
        assert_eq!(changes[0].db_id, beats[1].id.to_string());
    }

    #[test]
    fn accepted_prose_keeps_formatting_novelwriter_cannot_express() {
        let (conn, project, _temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        let formatted = r#"<p style="text-align: center"><u>Centred</u> opening.</p><ul><li><p>First item</p></li><li><p>Second item</p></li></ul><p>Typo hre.</p>"#;
        db::update_beat_prose(&conn, &beats[0].id, formatted).unwrap();
        let mut parsed = load(&conn, &project).unwrap();
        let fixed = html_to_nw(formatted).replace("hre.", "here.");
        parsed.beats[0].prose = Some(nw_to_html(&fixed));
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1);
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(
            db::get_beat(&conn, &beats[0].id)
                .unwrap()
                .unwrap()
                .prose
                .as_deref(),
            Some(formatted.replace("hre.", "here.").as_str())
        );
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        // Changed, removed and added paragraphs around kept blocks.
        assert_eq!(
            merge_prose(
                "<p><u>A</u></p><p>B</p><ul><li><p>C</p></li></ul>",
                "A\n\nB2\n\nC\n\nD"
            ),
            "<p><u>A</u></p><p>B2</p><ul><li><p>C</p></li></ul><p>D</p>"
        );
        assert_eq!(merge_prose("<p><u>A</u></p><p>B</p>", "B"), "<p>B</p>");
    }

    #[test]
    fn accepted_page_prose_keeps_the_separators_between_former_beats() {
        let (conn, project, _temp) = imported();
        let scene = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .remove(0);
        // A Page View scene built from Beat View keeps an <hr> at each old beat.
        let page = "<p>One.</p><hr><p>Two with a tpyo.</p><hr><p>Three.</p>";
        conn.execute(
            "UPDATE scenes SET editor_mode = 'page', prose = ?1 WHERE id = ?2",
            [page, &scene.id.to_string()],
        )
        .unwrap();
        let mut parsed = load(&conn, &project).unwrap();
        parsed.scenes[0].prose = Some(nw_to_html("One.\n\nTwo with a typo.\n\nThree."));
        let changes = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(changes.len(), 1, "{changes:?}");
        apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        let stored = db::get_all_project_scenes(&conn, &project.id).unwrap()[0]
            .prose
            .clone();
        assert_eq!(
            stored.as_deref(),
            Some("<p>One.</p><hr><p>Two with a typo.</p><hr><p>Three.</p>")
        );
        assert!(preview(&conn, &project, &parsed)
            .unwrap()
            .changes
            .is_empty());
        // Empty paragraphs are structure too; a trailing one survives a removal.
        assert_eq!(
            merge_prose("<p>A</p><p></p><p>B</p><hr>", "A"),
            "<p>A</p><p></p><hr>"
        );
    }

    #[test]
    fn apply_writes_nothing_that_changed_after_the_preview() {
        let (conn, project, _temp) = imported();
        let beats = db::get_all_project_beats(&conn, &project.id).unwrap();
        let prose = |beat: &Beat| db::get_beat(&conn, &beat.id).unwrap().unwrap().prose;
        let mut parsed = load(&conn, &project).unwrap();
        parsed.beats[1].prose = Some("<p>Reviewed text.</p>".into());
        let reviewed = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(reviewed.len(), 1);
        // novelWriter autosaves between the preview and Apply.
        parsed.beats[1].prose = Some("<p>Unreviewed text.</p>".into());
        let err = apply(
            &conn,
            &project,
            &parsed,
            &[reviewed[0].id.clone()],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .err()
        .unwrap();
        assert!(err.contains("changed after you reviewed"), "{err}");
        assert_eq!(prose(&beats[1]), beats[1].prose);

        // Only conflicts the writer saw, with the texts they saw, are settled.
        db::update_beat_prose(&conn, &beats[1].id, "<p>Local.</p>").unwrap();
        let shown = preview(&conn, &project, &parsed).unwrap().changes;
        assert!(shown[0].conflict);
        parsed.beats[1].prose = Some("<p>Newer still.</p>".into());
        assert!(apply(
            &conn,
            &project,
            &parsed,
            &[],
            &[],
            &[shown[0].id.clone()],
            &mut |_| Ok(())
        )
        .is_err());
        db::update_beat_prose(&conn, &beats[0].id, "<p>Local zero.</p>").unwrap();
        parsed.beats[0].prose = Some("<p>Incoming zero.</p>".into());
        let shown = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(shown.len(), 2);
        let seen = shown
            .iter()
            .find(|c| c.db_id == beats[1].id.to_string())
            .unwrap();
        apply(
            &conn,
            &project,
            &parsed,
            &[],
            &[],
            std::slice::from_ref(&seen.id),
            &mut |_| Ok(()),
        )
        .unwrap();
        let left = preview(&conn, &project, &parsed).unwrap().changes;
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].db_id, beats[0].id.to_string());
        assert_eq!(prose(&beats[1]).as_deref(), Some("<p>Local.</p>"));
    }

    #[test]
    fn documents_imported_whole_before_1_3_keep_their_one_scene_shape() {
        let (original, source) = fixture();
        let temp = tempfile::tempdir().unwrap();
        export_novelwriter_project(&original, &source.id, temp.path(), &Default::default())
            .unwrap();
        let handle = parse_novelwriter_project(temp.path()).unwrap().scenes[0]
            .source_id
            .clone()
            .unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, format!("{text}\n### Second half\n\nMore text.\n")).unwrap();
        // kindling 1.2 read the whole document as one scene.
        let legacy =
            parse_novelwriter_project_with(temp.path(), &HashSet::from([handle.clone()])).unwrap();
        assert_eq!(legacy.scenes.len(), 1);
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        super::super::import::insert_novelwriter(&conn, &legacy).unwrap();
        let project = legacy.project;
        let text_of = |conn: &Connection| {
            db::get_all_project_beats(conn, &project.id)
                .unwrap()
                .into_iter()
                .filter_map(|b| b.prose)
                .collect::<String>()
        };
        let before = text_of(&conn);
        assert_eq!(before.matches("More text.").count(), 1);

        // After upgrading, sync offers nothing: no new scene, no prose removal.
        let parsed = load(&conn, &project).unwrap();
        let diff = preview(&conn, &project, &parsed).unwrap();
        assert!(diff.additions.is_empty(), "{:?}", diff.additions);
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
        // Reimport accepts every change and addition; nothing is duplicated.
        let all: Vec<_> = diff.additions.iter().map(|a| a.id.clone()).collect();
        apply(&conn, &project, &parsed, &[], &all, &[], &mut |_| Ok(())).unwrap();
        assert_eq!(
            db::get_all_project_scenes(&conn, &project.id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(text_of(&conn), before);
        // Later edits under the second heading still reach the one scene.
        std::fs::write(
            &file,
            std::fs::read_to_string(&file)
                .unwrap()
                .replace("More text.", "More text, revised."),
        )
        .unwrap();
        let diff = preview(&conn, &project, &load(&conn, &project).unwrap()).unwrap();
        assert!(diff.additions.is_empty(), "{:?}", diff.additions);
        assert_eq!(diff.changes.len(), 1, "{:?}", diff.changes);
        // A fresh import of the same folder gets one scene per heading.
        assert_eq!(
            parse_novelwriter_project(temp.path()).unwrap().scenes.len(),
            2
        );
    }

    #[test]
    fn a_1_3_import_stays_split_after_the_writer_deletes_a_later_scene() {
        let (original, source) = fixture();
        let temp = tempfile::tempdir().unwrap();
        export_novelwriter_project(&original, &source.id, temp.path(), &Default::default())
            .unwrap();
        let handle = parse_novelwriter_project(temp.path()).unwrap().scenes[0]
            .source_id
            .clone()
            .unwrap();
        let file = temp.path().join("content").join(format!("{handle}.md"));
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, format!("{text}\n### Second half\n\nMore text.\n")).unwrap();
        // Imported by 1.3: one scene per heading.
        let parsed = parse_novelwriter_project(temp.path()).unwrap();
        assert_eq!(parsed.scenes.len(), 2);
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        super::super::import::insert_novelwriter(&conn, &parsed).unwrap();
        let project = parsed.project;
        // The writer deletes the second scene in kindling.
        let second = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .into_iter()
            .find(|s| s.title == "Second half")
            .unwrap();
        conn.execute(
            "DELETE FROM beats WHERE scene_id = ?1",
            [second.id.to_string()],
        )
        .unwrap();
        conn.execute("DELETE FROM scenes WHERE id = ?1", [second.id.to_string()])
            .unwrap();

        // The first scene is not offered the whole document.
        let diff = preview(&conn, &project, &load(&conn, &project).unwrap()).unwrap();
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
        // Only the deleted scene (with its own beat) is offered back.
        let scenes: Vec<_> = diff
            .additions
            .iter()
            .filter(|a| a.item_type == "scene")
            .map(|a| a.title.as_str())
            .collect();
        assert_eq!(scenes, ["Second half"]);
        // Declining the re-offered scene still leaves the document split next time.
        apply(
            &conn,
            &project,
            &load(&conn, &project).unwrap(),
            &[],
            &[],
            &[],
            &mut |_| Ok(()),
        )
        .unwrap();
        let diff = preview(&conn, &project, &load(&conn, &project).unwrap()).unwrap();
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
    }

    #[test]
    fn a_heading_added_after_a_1_3_import_becomes_a_new_scene() {
        // Imported by 1.3 while the document still had one heading.
        let (conn, project, temp) = imported();
        let scene = db::get_all_project_scenes(&conn, &project.id)
            .unwrap()
            .remove(0);
        let file = temp
            .path()
            .join("content")
            .join(format!("{}.md", scene.source_id.as_deref().unwrap()));
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(&file, format!("{text}\n### Second half\n\nMore text.\n")).unwrap();
        // The new heading is a new scene, not more text for the first one.
        let diff = preview(&conn, &project, &load(&conn, &project).unwrap()).unwrap();
        assert!(diff.changes.is_empty(), "{:?}", diff.changes);
        let scenes: Vec<_> = diff
            .additions
            .iter()
            .filter(|a| a.item_type == "scene")
            .map(|a| a.title.as_str())
            .collect();
        assert_eq!(scenes, ["Second half"]);
    }
}
