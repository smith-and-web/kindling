//! novelWriter adds user-reviewed prose to sync without changing the behavior of
//! outline-only sources. Preview and apply share the same change construction.
use super::sync::{ReimportSummary, SyncAddition, SyncChange, SyncPreview};
use crate::{db, models::*, parsers::novelwriter::*};
use rusqlite::Connection;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use uuid::Uuid;

pub(super) fn load(_conn: &Connection, project: &Project) -> Result<ParsedNovelWriter, String> {
    let path = project
        .source_path
        .as_deref()
        .ok_or("Project has no novelWriter source folder")?;
    let parsed = parse_novelwriter_project(Path::new(path)).map_err(|e| e.to_string())?;
    Ok(parsed)
}
/// One field as kindling holds it and as novelWriter holds it now.
struct Field {
    kind: &'static str,
    id: Uuid,
    title: String,
    field: &'static str,
    local: String,
    incoming: String,
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
    });
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
fn scene_prose(scene: &Scene, beats: &[Beat]) -> String {
    let mut ordered: Vec<_> = beats.iter().filter(|b| b.scene_id == scene.id).collect();
    if scene.editor_mode == EditorMode::Page || ordered.is_empty() {
        return prose(scene.prose.as_deref());
    }
    ordered.sort_by_key(|b| b.position);
    ordered
        .iter()
        .map(|b| prose(b.prose.as_deref()))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Walks the source and the project the same way for preview and baselines.
fn compare(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<(Vec<Field>, SyncPreview), String> {
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
    let mut beats_by_source = HashMap::new();
    for beat in &beats {
        if let Some(source) = beat.source_id.as_deref() {
            if beats_by_source.insert(source, beat).is_some() {
                return Err("Duplicate novelWriter beat identities. Import the source into a new project to review it safely; your current prose has been kept.".into());
            }
        }
    }
    let mut out = SyncPreview {
        additions: vec![],
        changes: vec![],
    };
    let mut fields = vec![];
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
                    change(
                        &mut fields,
                        "scene",
                        local.id,
                        &local.title,
                        "prose",
                        &scene_prose(local, &beats),
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
            for beat in parsed.beats.iter().filter(|b| b.scene_id == scene.id) {
                let source = beat.source_id.as_deref().unwrap();
                if let Some(existing) = beats_by_source
                    .get(source)
                    .copied()
                    .filter(|b| local.is_some_and(|s| s.id == b.scene_id))
                {
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
                        change(
                            &mut fields,
                            "beat",
                            existing.id,
                            &existing.content,
                            "prose",
                            &prose(existing.prose.as_deref()),
                            &prose(beat.prose.as_deref()),
                        );
                    }
                } else {
                    addition(&mut out, "beat", source, &beat.content, Some(&scene.title));
                }
            }
        }
    }
    Ok((fields, out))
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
    let (fields, _) = compare(conn, project, parsed)?;
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
fn change_id(f: &Field) -> String {
    format!("{}-{}-{}", f.kind, f.field, f.id)
}

pub(super) fn preview(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
) -> Result<SyncPreview, String> {
    let (fields, mut out) = compare(conn, project, parsed)?;
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

/// Applies the accepted changes and additions. With `settle_conflicts`, every
/// conflict left unaccepted is settled as "keep kindling": its baseline moves to
/// novelWriter's current text so it is not offered again until novelWriter
/// changes it. Unaccepted non-conflicting changes keep their baseline and are
/// offered again. Reimport passes `false` because it shows the writer nothing.
pub(super) fn apply(
    conn: &Connection,
    project: &Project,
    parsed: &ParsedNovelWriter,
    accepted: &[String],
    additions: &[String],
    settle_conflicts: bool,
) -> Result<ReimportSummary, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let (fields, preview) = compare(&tx, project, parsed)?;
    let mut known = baselines(&tx, project)?;
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
            if conflict && settle_conflicts {
                settle(&tx, project, &mut known, change)?;
            }
            continue;
        }
        let id = change.id;
        if change.field == "prose" {
            let html = nw_to_html(&change.incoming);
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
                let mut c = c.clone();
                c.project_id = project.id;
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
                let mut s = s.clone();
                s.chapter_id = *chapter;
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
            let mut b = b.clone();
            b.scene_id = *scene;
            db::insert_beat(&tx, &b).map_err(|e| e.to_string())?;
            summary.beats_added += 1;
        }
    }
    // New items only exist after insertion, so they need one more comparison.
    if summary.chapters_added + summary.scenes_added + summary.beats_added > 0 {
        record_baselines(&tx, project, parsed)?;
    }
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
            true,
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
        assert_eq!(
            apply(
                &conn,
                &project,
                &parsed,
                &[changes[1].id.clone()],
                &[],
                true
            )
            .unwrap()
            .prose_updated,
            0
        );
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
        apply(&conn, &project, &parsed, &all, &[], true).unwrap();
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
        apply(&conn, &project, &parsed, &[], &[], true).unwrap();
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
            true,
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
        apply(&conn, &project, &parsed, &[], &[], false).unwrap();
        assert_eq!(preview(&conn, &project, &parsed).unwrap().changes.len(), 1);
        // Applying with the conflict unticked keeps kindling's text and settles it.
        let summary = apply(&conn, &project, &parsed, &[], &[], true).unwrap();
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
        assert_eq!(preview(&conn, &project, &parsed).unwrap().changes.len(), 2);
        apply(&conn, &project, &parsed, &[], &[], true).unwrap();
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
        apply(&conn, &project, &parsed, &[], &[], true).unwrap();
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
            true,
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
            true,
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
            true,
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
            true,
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
        db::update_beat_source_id(&conn, &split.id, beats[1].source_id.as_deref().unwrap())
            .unwrap();
        assert!(preview(&conn, &project, &parsed)
            .unwrap_err()
            .contains("Duplicate"));
        assert!(apply(
            &conn,
            &project,
            &parsed,
            &[changes[0].id.clone()],
            &[],
            true
        )
        .is_err());
        std::fs::remove_file(temp.path().join("nwProject.nwx")).unwrap();
        assert!(load(&conn, &project).unwrap_err().contains("nwProject.nwx"));
    }
}
