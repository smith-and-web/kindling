//! Local editorial workspaces. Prose and review decisions commit together, with
//! optimistic checks against both the workspace and every current prose source.
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::EditorMode;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewDocument {
    pub id: String,
    pub label: String,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewDraft {
    pub name: String,
    pub created_at: String,
    pub mode: EditorMode,
    pub documents: Vec<ReviewDocument>,
    /// Kept by the app before it changed prose ("Before accepting…", "Before restoring…").
    /// Only these are pruned; a draft the writer named is never removed. Drafts saved
    /// before this flag existed read as named, so upgrading cannot prune them.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub automatic: bool,
}

/// Each automatic draft is a full copy of the scene. Keep the newest few per scene, so
/// accepting suggestions and restoring drafts cannot grow history without bound.
pub const AUTOMATIC_DRAFTS_KEPT: usize = 20;

fn prune_automatic_drafts(drafts: &mut Vec<ReviewDraft>) {
    let mut excess = drafts
        .iter()
        .filter(|d| d.automatic)
        .count()
        .saturating_sub(AUTOMATIC_DRAFTS_KEPT);
    drafts.retain(|d| {
        let prune = d.automatic && excess > 0;
        excess -= usize::from(prune);
        !prune
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewMessage {
    pub author: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub document_id: String,
    pub anchor_html: String,
    pub from: usize,
    pub to: usize,
    pub quote: String,
    pub replacement: Option<String>,
    pub state: String,
    pub messages: Vec<ReviewMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewData {
    pub status: String,
    pub drafts: Vec<ReviewDraft>,
    pub annotations: Vec<Annotation>,
}

impl Default for ReviewData {
    fn default() -> Self {
        Self {
            status: "first_draft".into(),
            drafts: vec![],
            annotations: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneReview {
    pub scene_id: String,
    pub version: i64,
    pub mode: EditorMode,
    pub documents: Vec<ReviewDocument>,
    pub data: ReviewData,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// The revision status of a scene that has no saved review yet.
fn unreviewed_status(scene_status: &str) -> &'static str {
    match scene_status {
        "revised" => "revised",
        "final" => "final",
        _ => "first_draft",
    }
}

pub fn load(conn: &Connection, scene_id: &Uuid) -> Result<SceneReview> {
    let scene = super::get_scene_by_id(conn, scene_id)
        .map_err(err)?
        .ok_or("Scene no longer exists")?;
    // Keep both storage modes, including inactive page prose, so a restore can
    // recover the precise draft without concatenating or discarding beat prose.
    let mut documents = vec![ReviewDocument {
        id: scene_id.to_string(),
        label: "Scene page".into(),
        html: scene.prose.unwrap_or_default(),
    }];
    for (index, beat) in super::get_beats(conn, scene_id)
        .map_err(err)?
        .into_iter()
        .enumerate()
    {
        documents.push(ReviewDocument {
            id: beat.id.to_string(),
            label: format!("Beat {}", index + 1),
            html: beat.prose.unwrap_or_default(),
        });
    }
    let saved: Option<(i64, String)> = conn
        .query_row(
            "SELECT version, data FROM scene_reviews WHERE scene_id = ?1",
            [scene_id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    let (version, data) = match saved {
        Some((v, json)) => (v, serde_json::from_str(&json).map_err(err)?),
        None => (
            0,
            ReviewData {
                status: unreviewed_status(scene.scene_status.as_str()).into(),
                ..ReviewData::default()
            },
        ),
    };
    Ok(SceneReview {
        scene_id: scene_id.to_string(),
        version,
        mode: scene.editor_mode,
        documents,
        data,
    })
}

pub fn save(
    conn: &Connection,
    expected: &SceneReview,
    data: &ReviewData,
    next: Option<&ReviewDraft>,
) -> Result<SceneReview> {
    let tx = conn.unchecked_transaction().map_err(err)?;
    let result = save_in_transaction(&tx, expected, data, next)?;
    tx.commit().map_err(err)?;
    Ok(result)
}

/// Caller owns the encompassing transaction (e.g. multi-scene acceptance).
pub fn save_in_transaction(
    tx: &Connection,
    expected: &SceneReview,
    data: &ReviewData,
    next: Option<&ReviewDraft>,
) -> Result<SceneReview> {
    let id = Uuid::parse_str(&expected.scene_id).map_err(err)?;
    if super::is_scene_locked(tx, &id).map_err(err)? {
        return Err("Cannot edit a locked scene".into());
    }
    let current = load(tx, &id)?;
    if current.version != expected.version
        || current.mode != expected.mode
        || current.documents != expected.documents
    {
        return Err(
            "Scene or review changed. Close and reopen Revisions before trying again.".into(),
        );
    }
    if !["first_draft", "editor_review", "revised", "final"].contains(&data.status.as_str()) {
        return Err("Invalid revision status".into());
    }
    // Saved drafts are append-only. A malformed/stale client cannot erase history.
    if !data.drafts.starts_with(&current.data.drafts) {
        return Err("Saved drafts cannot be changed or removed".into());
    }
    if data.drafts.iter().any(|d| d.name.trim().is_empty()) {
        return Err("A draft needs a name".into());
    }
    if let Some(next) = next {
        if next.documents.len() != current.documents.len()
            || next
                .documents
                .iter()
                .zip(&current.documents)
                .any(|(a, b)| a.id != b.id)
        {
            return Err("Scene structure changed. This draft can be compared but cannot be restored to different beats.".into());
        }
        // Any prose replacement must first preserve the exact current draft.
        if !data.drafts[current.data.drafts.len()..]
            .iter()
            .any(|d| d.mode == current.mode && d.documents == current.documents)
        {
            return Err("Preserve the current draft before changing prose".into());
        }
        for doc in &next.documents {
            let doc_id = Uuid::parse_str(&doc.id).map_err(err)?;
            if doc.id == expected.scene_id {
                super::update_scene_prose(tx, &doc_id, &doc.html).map_err(err)?;
            } else {
                super::update_beat_prose(tx, &doc_id, &doc.html).map_err(err)?;
            }
        }
        tx.execute(
            "UPDATE scenes SET editor_mode = ?1 WHERE id = ?2",
            params![next.mode.as_str(), id.to_string()],
        )
        .map_err(err)?;
    }
    // Pruned after the checks above, which need the drafts exactly as the client sent them.
    // The one appended to preserve the current prose is the newest, so it always survives.
    let mut data = data.clone();
    prune_automatic_drafts(&mut data.drafts);
    let json = serde_json::to_string(&data).map_err(err)?;
    tx.execute(
        "UPDATE scenes SET scene_status=?1 WHERE id=?2",
        params![
            match data.status.as_str() {
                "final" => "final",
                "revised" => "revised",
                _ => "draft",
            },
            id.to_string()
        ],
    )
    .map_err(err)?;
    tx.execute(
        "INSERT INTO scene_reviews(scene_id, version, data) VALUES (?1, ?2, ?3)
        ON CONFLICT(scene_id) DO UPDATE SET version = excluded.version, data = excluded.data",
        params![id.to_string(), current.version + 1, json],
    )
    .map_err(err)?;
    let project_id = super::get_scene_project_id(tx, &id)
        .map_err(err)?
        .ok_or("Project no longer exists")?;
    super::update_project_modified(tx, &project_id).map_err(err)?;
    let result = load(tx, &id)?;
    Ok(result)
}

#[derive(Serialize)]
pub struct RevisionOverview {
    pub scene_id: String,
    pub title: String,
    pub chapter: String,
    pub status: String,
    pub drafts: usize,
}

pub fn overview(conn: &Connection, project_id: &Uuid) -> Result<Vec<RevisionOverview>> {
    let mut stmt = conn.prepare("SELECT s.id, s.title, c.title, r.data, s.scene_status FROM scenes s JOIN chapters c ON c.id = s.chapter_id
        LEFT JOIN scene_reviews r ON r.scene_id = s.id WHERE c.project_id = ?1 AND NOT s.archived AND NOT c.archived
        ORDER BY c.position, s.position").map_err(err)?;
    let rows = stmt
        .query_map([project_id.to_string()], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, String>(4)?,
            ))
        })
        .map_err(err)?
        .map(|row| {
            let (scene_id, title, chapter, json, status) = row.map_err(err)?;
            let data: ReviewData = json
                .map(|j| serde_json::from_str(&j))
                .transpose()
                .map_err(err)?
                .unwrap_or_else(|| ReviewData {
                    status: unreviewed_status(&status).into(),
                    ..ReviewData::default()
                });
            Ok(RevisionOverview {
                scene_id,
                title,
                chapter,
                status: data.status,
                drafts: data.drafts.len(),
            })
        })
        .collect();
    rows
}

/// Snapshot payloads preserve editorial metadata; old snapshots default to none.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewBackup {
    pub scene_id: Uuid,
    pub data: ReviewData,
}

pub fn backup(conn: &Connection, project_id: &Uuid) -> Result<Vec<ReviewBackup>> {
    let mut stmt = conn.prepare("SELECT r.scene_id, r.data FROM scene_reviews r JOIN scenes s ON s.id = r.scene_id JOIN chapters c ON c.id = s.chapter_id WHERE c.project_id = ?1").map_err(err)?;
    let rows = stmt
        .query_map([project_id.to_string()], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })
        .map_err(err)?
        .map(|row| {
            let (id, json) = row.map_err(err)?;
            Ok(ReviewBackup {
                scene_id: Uuid::parse_str(&id).map_err(err)?,
                data: serde_json::from_str(&json).map_err(err)?,
            })
        })
        .collect();
    rows
}

/// Replacing a project with a snapshot deletes its reviews along with its scenes. For a
/// scene the snapshot also contains, restore the snapshot's review as it was, but keep the
/// drafts and comments it never contained: an older snapshot, or one from before 1.3 with
/// no reviews at all, must not erase the draft history recorded since.
pub fn merge_backup(
    snapshot: Option<ReviewBackup>,
    current: ReviewBackup,
    scene_status: &str,
) -> ReviewBackup {
    let mut merged = snapshot.unwrap_or_else(|| ReviewBackup {
        scene_id: current.scene_id,
        data: ReviewData {
            status: unreviewed_status(scene_status).into(),
            ..ReviewData::default()
        },
    });
    // History is append-only, so it normally extends the snapshot's. If pruning dropped
    // drafts the snapshot still has, keep those first and then the ones recorded since.
    if current.data.drafts.starts_with(&merged.data.drafts) {
        merged.data.drafts = current.data.drafts;
    } else {
        let later: Vec<_> = current
            .data
            .drafts
            .into_iter()
            .filter(|d| !merged.data.drafts.contains(d))
            .collect();
        merged.data.drafts.extend(later);
    }
    prune_automatic_drafts(&mut merged.data.drafts);
    for annotation in current.data.annotations {
        if !merged
            .data
            .annotations
            .iter()
            .any(|a| a.id == annotation.id)
        {
            merged.data.annotations.push(annotation);
        }
    }
    merged
}

pub fn restore_backup(
    conn: &Connection,
    backup: &ReviewBackup,
    ids: &std::collections::HashMap<Uuid, Uuid>,
) -> Result<()> {
    let remap = |id: &str| -> String {
        Uuid::parse_str(id)
            .ok()
            .and_then(|id| ids.get(&id))
            .map(ToString::to_string)
            .unwrap_or_else(|| id.to_string())
    };
    let mut data = backup.data.clone();
    for draft in &mut data.drafts {
        for doc in &mut draft.documents {
            doc.id = remap(&doc.id);
        }
    }
    for annotation in &mut data.annotations {
        annotation.document_id = remap(&annotation.document_id);
    }
    conn.execute(
        "INSERT INTO scene_reviews(scene_id, version, data) VALUES (?1, 1, ?2)",
        params![
            remap(&backup.scene_id.to_string()),
            serde_json::to_string(&data).map_err(err)?
        ],
    )
    .map_err(err)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        models::{Beat, Chapter, Project, Scene, SourceType},
    };
    fn fixture() -> (Connection, Project, Chapter, Scene, Beat) {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let p = Project::new("Novel".into(), SourceType::Blank, None);
        db::insert_project(&conn, &p).unwrap();
        let c = Chapter::new(p.id, "Chapter".into(), 0);
        db::insert_chapter(&conn, &c).unwrap();
        let s = Scene::new(c.id, "Scene".into(), None, 0);
        db::insert_scene(&conn, &s).unwrap();
        let b = Beat::new(s.id, "Beat".into(), 0);
        db::insert_beat(&conn, &b).unwrap();
        db::update_beat_prose(&conn, &b.id, "<p>Original</p>").unwrap();
        (conn, p, c, s, b)
    }
    fn draft(r: &SceneReview) -> ReviewDraft {
        ReviewDraft {
            name: "First draft".into(),
            created_at: "2026-09-07".into(),
            mode: r.mode,
            documents: r.documents.clone(),
            automatic: false,
        }
    }
    #[test]
    fn persists_history_status_and_atomic_prose_restore() {
        let (conn, p, _, s, _) = fixture();
        let r = load(&conn, &s.id).unwrap();
        let mut data = r.data.clone();
        data.drafts.push(draft(&r));
        data.status = "editor_review".into();
        let r = save(&conn, &r, &data, None).unwrap();
        assert_eq!(r.version, 1);
        let mut next = draft(&r);
        next.documents[1].html = "<p>Revised</p>".into();
        assert!(save(&conn, &r, &data, Some(&next))
            .unwrap_err()
            .contains("Preserve"));
        data.drafts.push(draft(&r));
        let r = save(&conn, &r, &data, Some(&next)).unwrap();
        assert_eq!(r.documents[1].html, "<p>Revised</p>");
        data.drafts.push(draft(&r));
        let restored = save(&conn, &r, &data, Some(&data.drafts[0])).unwrap();
        assert_eq!(restored.documents[1].html, "<p>Original</p>");
        assert_eq!(restored.data.drafts[2].documents[1].html, "<p>Revised</p>");
        let rows = overview(&conn, &p.id).unwrap();
        assert_eq!(rows[0].drafts, 3);
        assert_eq!(rows[0].status, "editor_review");
        // Restores/editorial changes don't manufacture daily writing credit.
        assert_eq!(
            db::writing::stats(&conn, &p.id, chrono::Local::now().date_naive())
                .unwrap()
                .session_words,
            0
        );
    }
    #[test]
    fn prunes_only_the_oldest_automatic_drafts_and_never_a_named_one() {
        let (conn, _, _, s, _) = fixture();
        let mut r = load(&conn, &s.id).unwrap();
        let mut data = r.data.clone();
        data.drafts.push(draft(&r));
        r = save(&conn, &r, &data, None).unwrap();
        let rounds = AUTOMATIC_DRAFTS_KEPT + 5;
        for i in 0..rounds {
            let mut data = r.data.clone();
            if i == 2 {
                data.drafts.push(ReviewDraft {
                    name: "Named midway".into(),
                    ..draft(&r)
                });
            }
            data.drafts.push(ReviewDraft {
                name: format!("Before accepting {i}"),
                automatic: true,
                ..draft(&r)
            });
            let mut next = draft(&r);
            next.documents[1].html = format!("<p>Revision {i}</p>");
            r = save(&conn, &r, &data, Some(&next)).unwrap();
        }
        let drafts = load(&conn, &s.id).unwrap().data.drafts;
        let names: Vec<_> = drafts.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(drafts.len(), 2 + AUTOMATIC_DRAFTS_KEPT, "{names:?}");
        assert_eq!(names[0], "First draft");
        assert!(names.contains(&"Named midway"));
        assert!(!names.contains(&"Before accepting 4"));
        assert!(names.contains(&"Before accepting 5"));
        // The newest automatic draft preserves the prose from just before the last change.
        let newest = drafts.last().unwrap();
        assert_eq!(newest.name, format!("Before accepting {}", rounds - 1));
        assert_eq!(
            newest.documents[1].html,
            format!("<p>Revision {}</p>", rounds - 2)
        );
        // The next save still appends to the pruned history.
        let mut data = r.data.clone();
        data.drafts.push(draft(&r));
        assert_eq!(
            save(&conn, &r, &data, None).unwrap().data.drafts.len(),
            3 + AUTOMATIC_DRAFTS_KEPT
        );
        // Drafts saved before the flag existed are named, and a named draft stores no flag.
        let legacy: ReviewDraft = serde_json::from_str(
            r#"{"name":"Before accepting changes","created_at":"t","mode":"page","documents":[]}"#,
        )
        .unwrap();
        assert!(!legacy.automatic);
        assert!(!serde_json::to_string(&drafts[0])
            .unwrap()
            .contains("automatic"));
    }
    #[test]
    fn rejects_concurrent_edits_versions_and_locked_chapters() {
        let (conn, _, c, s, b) = fixture();
        let r = load(&conn, &s.id).unwrap();
        save(&conn, &r, &r.data, None).unwrap();
        assert!(save(&conn, &r, &r.data, None)
            .unwrap_err()
            .contains("changed"));
        let r = load(&conn, &s.id).unwrap();
        db::update_beat_prose(&conn, &b.id, "Newer prose").unwrap();
        assert!(save(&conn, &r, &r.data, None)
            .unwrap_err()
            .contains("changed"));
        let r = load(&conn, &s.id).unwrap();
        db::lock_chapter(&conn, &c.id).unwrap();
        assert!(save(&conn, &r, &r.data, None)
            .unwrap_err()
            .contains("locked"));
        assert_eq!(load(&conn, &s.id).unwrap().version, 1);
    }
    #[test]
    fn rejects_history_loss_invalid_status_and_changed_structure() {
        let (conn, _, _, s, _) = fixture();
        let r = load(&conn, &s.id).unwrap();
        let mut data = r.data.clone();
        data.status = "nonsense".into();
        assert!(save(&conn, &r, &data, None).is_err());
        data = r.data.clone();
        data.drafts.push(draft(&r));
        let r = save(&conn, &r, &data, None).unwrap();
        assert!(save(&conn, &r, &ReviewData::default(), None)
            .unwrap_err()
            .contains("cannot be changed"));
        let mut next = draft(&r);
        next.documents.pop();
        data.drafts.push(draft(&r));
        assert!(save(&conn, &r, &data, Some(&next))
            .unwrap_err()
            .contains("structure changed"));
        assert_eq!(load(&conn, &s.id).unwrap().version, 1);
        data.drafts[1].name = " ".into();
        assert!(save(&conn, &r, &data, None).unwrap_err().contains("name"));
    }
    #[test]
    fn backup_remaps_draft_sources_and_annotations_and_cascades_on_delete() {
        let (conn, p, c, s, b) = fixture();
        let r = load(&conn, &s.id).unwrap();
        let mut data = r.data.clone();
        data.drafts.push(draft(&r));
        data.annotations.push(Annotation {
            id: "thread".into(),
            document_id: b.id.to_string(),
            anchor_html: r.documents[1].html.clone(),
            from: 1,
            to: 9,
            quote: "Original".into(),
            replacement: None,
            state: "open".into(),
            messages: vec![ReviewMessage {
                author: "Editor".into(),
                text: "Comment".into(),
                created_at: "today".into(),
            }],
        });
        save(&conn, &r, &data, None).unwrap();
        let copy = Scene::new(c.id, "Copy".into(), None, 1);
        db::insert_scene(&conn, &copy).unwrap();
        let copy_beat = Beat::new(copy.id, "Beat".into(), 0);
        db::insert_beat(&conn, &copy_beat).unwrap();
        let backups = backup(&conn, &p.id).unwrap();
        restore_backup(
            &conn,
            &backups[0],
            &[(s.id, copy.id), (b.id, copy_beat.id)].into(),
        )
        .unwrap();
        let copied = load(&conn, &copy.id).unwrap();
        assert_eq!(copied.data.drafts[0].documents[0].id, copy.id.to_string());
        assert_eq!(
            copied.data.annotations[0].document_id,
            copy_beat.id.to_string()
        );
        conn.execute("DELETE FROM scenes WHERE id = ?1", [s.id.to_string()])
            .unwrap();
        assert_eq!(backup(&conn, &p.id).unwrap().len(), 1);
        assert!(load(&conn, &s.id).is_err());
    }
}
