//! Portable, offline editorial rounds. Imported feedback never writes manuscript
//! prose: explicit decisions use optimistic, atomic writes and preserve drafts.
use std::{
    collections::HashSet,
    fs,
    io::{Read, Write},
    path::Path,
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

use super::AppState;
use crate::{db, models::EditorMode};

type Result<T> = std::result::Result<T, String>;
const MAX_PACKAGE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub scene_id: String,
    pub chapter_id: String,
    pub chapter: String,
    pub scene: String,
    pub mode: EditorMode,
    pub html: String,
    pub locked: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Round {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub name: String,
    pub brief: String,
    pub created_at: String,
    pub sources: Vec<Source>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub format: String,
    pub version: u32,
    pub kind: String,
    pub round: Round,
    pub session: Option<Session>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub reviewer_id: String,
    pub name: String,
    pub generation: u64,
    pub document: Value,
    pub changes: Vec<Change>,
    pub position: usize,
    #[serde(default)]
    pub reading_position: usize,
    #[serde(default)]
    pub writer_version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub revision: u64,
    pub kind: String,
    pub from: usize,
    pub to: usize,
    pub before: Value,
    pub after: Value,
    pub state: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub anchor_offset: Option<[usize; 2]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub writer_decision: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub author: String,
    pub text: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedbackEntry {
    pub key: String,
    pub reviewer: String,
    pub change: Change,
    pub decision: String,
    #[serde(default)]
    pub decided_by_writer: bool,
}

#[derive(Debug, Serialize)]
pub struct Feedback {
    pub round: Round,
    pub entries: Vec<FeedbackEntry>,
    pub sources: Vec<Source>,
    pub version: i64,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

pub fn initialize(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("CREATE TABLE IF NOT EXISTS editorial_rounds (
        id TEXT PRIMARY KEY, project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
        data TEXT NOT NULL, feedback TEXT NOT NULL DEFAULT '[]', version INTEGER NOT NULL DEFAULT 0);
        CREATE TABLE IF NOT EXISTS editorial_sessions (
        round_id TEXT PRIMARY KEY, round_data TEXT NOT NULL, data TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS editorial_returns (
        round_id TEXT NOT NULL REFERENCES editorial_rounds(id) ON DELETE CASCADE,
        reviewer_id TEXT NOT NULL, generation INTEGER NOT NULL, data TEXT NOT NULL,
        PRIMARY KEY(round_id, reviewer_id));")
}

pub fn sources(conn: &Connection, project_id: &str) -> Result<Vec<Source>> {
    let id = Uuid::parse_str(project_id).map_err(err)?;
    if db::get_project(conn, &id).map_err(err)?.is_none() {
        return Err("The original project is not on this computer. Open this feedback on the writer's computer with the original project.".into());
    }
    let mut result = vec![];
    for chapter in db::get_chapters(conn, &id).map_err(err)? {
        for scene in db::get_scenes(conn, &chapter.id).map_err(err)? {
            if scene.scene_type.as_str() != "normal" {
                continue;
            }
            let base = Source {
                id: scene.id.to_string(),
                scene_id: scene.id.to_string(),
                chapter_id: chapter.id.to_string(),
                chapter: chapter.title.clone(),
                scene: scene.title,
                mode: scene.editor_mode,
                html: scene.prose.unwrap_or_default(),
                locked: chapter.locked || scene.locked,
            };
            let beats = db::get_beats(conn, &scene.id).map_err(err)?;
            if scene.editor_mode == EditorMode::Page || beats.is_empty() {
                result.push(base);
            } else {
                for beat in beats {
                    result.push(Source {
                        id: beat.id.to_string(),
                        html: beat.prose.unwrap_or_default(),
                        ..base.clone()
                    });
                }
            }
        }
    }
    Ok(result)
}

fn validate(package: &Package) -> Result<()> {
    if package.format != "kindling-editorial" || package.version != 1 {
        return Err("This review package version is not supported. Update Kindling or ask the sender to export a compatible package.".into());
    }
    if !["review", "feedback"].contains(&package.kind.as_str()) {
        return Err("Unknown review package type".into());
    }
    Uuid::parse_str(&package.round.id).map_err(err)?;
    Uuid::parse_str(&package.round.project_id).map_err(err)?;
    if package.round.sources.is_empty() || package.round.name.trim().is_empty() {
        return Err("The review package has no manuscript or review round name.".into());
    }
    let mut seen = HashSet::new();
    for source in &package.round.sources {
        for id in [&source.id, &source.scene_id, &source.chapter_id] {
            Uuid::parse_str(id).map_err(err)?;
        }
        if !seen.insert(&source.id) {
            return Err("Duplicate manuscript source in package".into());
        }
    }
    if let Some(session) = &package.session {
        Uuid::parse_str(&session.reviewer_id).map_err(err)?;
        if session.generation > i64::MAX as u64 {
            return Err("Invalid feedback generation".into());
        }
        let mut ids = HashSet::new();
        for change in &session.changes {
            if !ids.insert(&change.id)
                || change.from > change.to
                || !["suggestion", "comment"].contains(&change.kind.as_str())
                || !["open", "resolved", "withdrawn"].contains(&change.state.as_str())
            {
                return Err("Invalid or duplicate editorial annotation".into());
            }
        }
    }
    if package.kind == "feedback" && package.session.is_none() {
        return Err("The feedback file contains no review.".into());
    }
    Ok(())
}

fn read_package(path: &Path) -> Result<Package> {
    let file = fs::File::open(path).map_err(err)?;
    let mut bytes = vec![];
    file.take(MAX_PACKAGE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    if bytes.len() as u64 > MAX_PACKAGE_BYTES {
        return Err("This review file exceeds the 64 MiB package limit.".into());
    }
    let package: Package = serde_json::from_slice(&bytes)
        .map_err(|_| "This is not a valid Kindling review file.".to_string())?;
    validate(&package)?;
    Ok(package)
}

fn write_package(path: &Path, package: &Package) -> Result<()> {
    validate(package)?;
    let bytes = serde_json::to_vec(package).map_err(err)?;
    if bytes.len() as u64 > MAX_PACKAGE_BYTES {
        return Err("This review exceeds the 64 MiB package limit. Export fewer chapters.".into());
    }
    // Write beside the destination, sync, then rename. Never truncate an existing
    // return file before a complete replacement exists.
    let temporary = path.with_extension(format!("{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(err)?;
        file.write_all(&bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        fs::rename(&temporary, path).map_err(err)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

fn round(conn: &Connection, id: &str) -> Result<Round> {
    let data: Option<String> = conn
        .query_row(
            "SELECT data FROM editorial_rounds WHERE id = ?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?;
    serde_json::from_str(&data.ok_or("The original review round is not on this computer. Return this file to the writer who exported it.")?).map_err(err)
}

fn feedback(conn: &Connection, id: &str) -> Result<Feedback> {
    let round = round(conn, id)?;
    let (json, version): (String, i64) = conn
        .query_row(
            "SELECT feedback, version FROM editorial_rounds WHERE id = ?1",
            [id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(err)?;
    let available = sources(conn, &round.project_id)?;
    Ok(Feedback {
        sources: available
            .into_iter()
            .filter(|s| round.sources.iter().any(|o| o.scene_id == s.scene_id))
            .collect(),
        round,
        entries: serde_json::from_str(&json).map_err(err)?,
        version,
    })
}

fn import_feedback(conn: &Connection, package: &Package) -> Result<Feedback> {
    validate(package)?;
    let session = package
        .session
        .as_ref()
        .ok_or("No feedback in this package")?;
    let tx = conn.unchecked_transaction().map_err(err)?;
    if round(&tx, &package.round.id)? != package.round {
        return Err(
            "This feedback does not match the manuscript originally sent for this review round."
                .into(),
        );
    }
    let previous: Option<(i64, String)> = tx.query_row("SELECT generation, data FROM editorial_returns WHERE round_id = ?1 AND reviewer_id = ?2", params![package.round.id, session.reviewer_id], |r| Ok((r.get(0)?, r.get(1)?))).optional().map_err(err)?;
    let json = serde_json::to_string(session).map_err(err)?;
    if let Some((generation, old)) = previous {
        if generation > session.generation as i64 {
            return feedback(&tx, &package.round.id);
        }
        if generation == session.generation as i64 {
            if old != json {
                return Err("Two different responses have the same review generation. Ask the editor to export their latest saved review again.".into());
            }
            return feedback(&tx, &package.round.id);
        }
    }
    let mut current = feedback(&tx, &package.round.id)?;
    let prefix = format!("{}/", session.reviewer_id);
    for entry in &mut current.entries {
        if entry.key.starts_with(&prefix) && entry.decision == "open" {
            let still_open = session.changes.iter().any(|c| {
                entry.key == format!("{}{}/{}", prefix, c.id, c.revision) && c.state != "withdrawn"
            });
            if !still_open {
                entry.decision = "withdrawn".into();
            }
        }
    }
    for change in &session.changes {
        let key = format!("{}{}/{}", prefix, change.id, change.revision);
        if let Some(existing) = current.entries.iter_mut().find(|e| e.key == key) {
            if !existing.decided_by_writer {
                existing.decision = change.state.clone();
            }
            existing.change.state = change.state.clone();
            for message in &change.messages {
                if !existing.change.messages.iter().any(|m| m.id == message.id) {
                    existing.change.messages.push(message.clone());
                }
            }
        } else {
            current.entries.push(FeedbackEntry {
                key,
                reviewer: session.name.clone(),
                change: change.clone(),
                decision: change.state.clone(),
                decided_by_writer: false,
            });
        }
    }
    tx.execute(
        "UPDATE editorial_rounds SET feedback = ?1, version = version + 1 WHERE id = ?2",
        params![
            serde_json::to_string(&current.entries).map_err(err)?,
            package.round.id
        ],
    )
    .map_err(err)?;
    tx.execute("INSERT INTO editorial_returns(round_id, reviewer_id, generation, data) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(round_id, reviewer_id) DO UPDATE SET generation=excluded.generation, data=excluded.data", params![package.round.id, session.reviewer_id, session.generation as i64, json]).map_err(err)?;
    tx.commit().map_err(err)?;
    feedback(conn, &package.round.id)
}

#[tauri::command]
pub async fn editorial_sources(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Source>> {
    sources(&*state.db.lock().map_err(err)?, &project_id)
}

#[tauri::command]
pub async fn export_editorial_review(
    project_id: String,
    name: String,
    brief: String,
    chapter_ids: Vec<String>,
    path: String,
    state: State<'_, AppState>,
) -> Result<Round> {
    let conn = state.db.lock().map_err(err)?;
    let all = sources(&conn, &project_id)?;
    let project = db::get_project(&conn, &Uuid::parse_str(&project_id).map_err(err)?)
        .map_err(err)?
        .ok_or("Project no longer exists")?;
    let round = Round {
        id: Uuid::new_v4().to_string(),
        project_id,
        title: project.name,
        name: name.trim().into(),
        brief,
        created_at: chrono::Utc::now().to_rfc3339(),
        sources: all
            .into_iter()
            .filter(|s| chapter_ids.is_empty() || chapter_ids.contains(&s.chapter_id))
            .collect(),
    };
    let package = Package {
        format: "kindling-editorial".into(),
        version: 1,
        kind: "review".into(),
        round: round.clone(),
        session: None,
    };
    validate(&package)?;
    // Register the immutable baseline before publishing the file. An interrupted
    // export can leave an unused round, never an unrecognisable returned review.
    conn.execute(
        "INSERT INTO editorial_rounds(id, project_id, data) VALUES (?1, ?2, ?3)",
        params![
            round.id,
            round.project_id,
            serde_json::to_string(&round).map_err(err)?
        ],
    )
    .map_err(err)?;
    write_package(Path::new(&path), &package)?;
    Ok(round)
}

#[derive(Serialize)]
pub struct OpenedPackage {
    #[serde(flatten)]
    pub package: Package,
    pub saved_generation: Option<u64>,
}

fn resume_package(conn: &Connection, mut package: Package) -> Result<OpenedPackage> {
    let mut saved_generation = None;
    if package.kind == "review" {
        let saved: Option<(String, String)> = conn
            .query_row(
                "SELECT round_data, data FROM editorial_sessions WHERE round_id = ?1",
                [&package.round.id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(err)?;
        if let Some((original, data)) = saved {
            if serde_json::from_str::<Round>(&original).map_err(err)? != package.round {
                return Err(
                    "This package reuses a review identity with a different manuscript.".into(),
                );
            }
            let mut local: Session = serde_json::from_str(&data).map_err(err)?;
            saved_generation = Some(local.generation);
            if let Some(incoming) = &package.session {
                if incoming.reviewer_id != local.reviewer_id {
                    return Err("This personalized response belongs to another reviewer. Open it on that reviewer's installation; your local review is preserved.".into());
                }
                let authoritative = if incoming.writer_version >= local.writer_version {
                    incoming.clone()
                } else {
                    local.clone()
                };
                let other = if incoming.generation > local.generation {
                    let previous = local;
                    local = incoming.clone();
                    previous
                } else {
                    incoming.clone()
                };
                // Merge discussions in either direction without replacing newer editor prose.
                for incoming_change in &other.changes {
                    if let Some(change) = local
                        .changes
                        .iter_mut()
                        .find(|c| c.id == incoming_change.id)
                    {
                        for message in &incoming_change.messages {
                            if !change.messages.iter().any(|m| m.id == message.id) {
                                change.messages.push(message.clone());
                            }
                        }
                    } else if !incoming_change.messages.is_empty() {
                        let mut discussion = incoming_change.clone();
                        if discussion.kind != "comment" {
                            discussion.id = format!("discussion-{}", discussion.id);
                        }
                        discussion.kind = "comment".into();
                        discussion.after = Value::Null;
                        discussion.anchor_offset = None;
                        discussion.state = "open".into();
                        discussion.writer_decision = None;
                        if let Some(existing) =
                            local.changes.iter_mut().find(|c| c.id == discussion.id)
                        {
                            for message in discussion.messages {
                                if !existing.messages.iter().any(|m| m.id == message.id) {
                                    existing.messages.push(message);
                                }
                            }
                        } else {
                            local.changes.push(discussion);
                        }
                    }
                }
                // Writer responses have their own monotonic version; opening an older
                // file cannot undo a later decision even at the same editor generation.
                for incoming_change in &authoritative.changes {
                    if let Some(change) = local.changes.iter_mut().find(|c| {
                        c.id == incoming_change.id && c.revision == incoming_change.revision
                    }) {
                        change.writer_decision = incoming_change.writer_decision.clone();
                    }
                }
                local.writer_version = authoritative.writer_version;
            }
            package.session = Some(local);
        }
    } else if round(conn, &package.round.id)? != package.round {
        return Err("The feedback manuscript does not match its original review round.".into());
    }
    Ok(OpenedPackage {
        package,
        saved_generation,
    })
}

#[tauri::command]
pub async fn open_editorial_package(
    path: String,
    state: State<'_, AppState>,
) -> Result<OpenedPackage> {
    let package = read_package(Path::new(&path))?;
    resume_package(&*state.db.lock().map_err(err)?, package)
}

#[tauri::command]
pub async fn export_editorial_recovery(round: Round, session: Session, path: String) -> Result<()> {
    write_package(
        Path::new(&path),
        &Package {
            format: "kindling-editorial".into(),
            version: 1,
            kind: "review".into(),
            round,
            session: Some(session),
        },
    )
}

fn writer_response(conn: &Connection, round_id: &str, reviewer_id: &str) -> Result<Package> {
    let data = feedback(conn, round_id)?;
    let json: String = conn
        .query_row(
            "SELECT data FROM editorial_returns WHERE round_id=?1 AND reviewer_id=?2",
            params![round_id, reviewer_id],
            |r| r.get(0),
        )
        .map_err(err)?;
    let mut session: Session = serde_json::from_str(&json).map_err(err)?;
    session.writer_version = data.version as u64;
    for change in &mut session.changes {
        if let Some(entry) = data
            .entries
            .iter()
            .find(|e| e.key == format!("{}/{}/{}", reviewer_id, change.id, change.revision))
        {
            change.messages = entry.change.messages.clone();
            if entry.decided_by_writer {
                change.writer_decision = Some(entry.decision.clone());
            }
        }
    }
    for entry in &data.entries {
        if !entry.key.starts_with(&format!("{reviewer_id}/")) || entry.change.messages.is_empty() {
            continue;
        }
        // A refined annotation keeps its conversation. Merge replies from earlier
        // versions onto it; only a missing annotation needs a standalone discussion.
        if let Some(existing) = session.changes.iter_mut().find(|c| c.id == entry.change.id) {
            for message in &entry.change.messages {
                if !existing.messages.iter().any(|m| m.id == message.id) {
                    existing.messages.push(message.clone());
                }
            }
            continue;
        }
        let mut discussion = entry.change.clone();
        if discussion.kind != "comment" {
            discussion.id = format!("discussion-{}", discussion.id);
        }
        discussion.kind = "comment".into();
        discussion.state = "open".into();
        discussion.writer_decision = None;
        discussion.after = Value::Null;
        discussion.anchor_offset = None;
        if let Some(existing) = session.changes.iter_mut().find(|c| c.id == discussion.id) {
            for message in discussion.messages {
                if !existing.messages.iter().any(|m| m.id == message.id) {
                    existing.messages.push(message);
                }
            }
        } else {
            session.changes.push(discussion);
        }
    }
    Ok(Package {
        format: "kindling-editorial".into(),
        version: 1,
        kind: "review".into(),
        round: data.round,
        session: Some(session),
    })
}

#[tauri::command]
pub async fn export_editorial_reply(
    round_id: String,
    reviewer_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<()> {
    let conn = state.db.lock().map_err(err)?;
    write_package(
        Path::new(&path),
        &writer_response(&conn, &round_id, &reviewer_id)?,
    )
}

#[tauri::command]
pub async fn save_editorial_session(
    round: Round,
    session: Session,
    expected_generation: Option<u64>,
    state: State<'_, AppState>,
) -> Result<()> {
    let package = Package {
        format: "kindling-editorial".into(),
        version: 1,
        kind: "review".into(),
        round,
        session: Some(session),
    };
    validate(&package)?;
    let session = package.session.as_ref().unwrap();
    let conn = state.db.lock().map_err(err)?;
    let previous: Option<(String, String)> = conn
        .query_row(
            "SELECT round_data, data FROM editorial_sessions WHERE round_id = ?1",
            [&package.round.id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(err)?;
    if let Some((original, data)) = &previous {
        let old: Session = serde_json::from_str(data).map_err(err)?;
        if serde_json::from_str::<Round>(original).map_err(err)? != package.round
            || Some(old.generation) != expected_generation
            || old.reviewer_id != session.reviewer_id
        {
            return Err("This review was updated in another window. Keep this window open and export a recovery copy before reopening.".into());
        }
    } else if expected_generation.is_some() {
        return Err("The saved review is no longer available.".into());
    }
    if session.generation <= expected_generation.unwrap_or(0) {
        return Err("Review save generation must advance.".into());
    }
    conn.execute("INSERT INTO editorial_sessions(round_id, round_data, data) VALUES (?1, ?2, ?3) ON CONFLICT(round_id) DO UPDATE SET data=excluded.data", params![package.round.id, serde_json::to_string(&package.round).map_err(err)?, serde_json::to_string(session).map_err(err)?]).map_err(err)?;
    Ok(())
}

#[tauri::command]
pub async fn export_editorial_feedback(round: Round, session: Session, path: String) -> Result<()> {
    if session.name.trim().is_empty() {
        return Err("Enter your name before returning feedback.".into());
    }
    write_package(
        Path::new(&path),
        &Package {
            format: "kindling-editorial".into(),
            version: 1,
            kind: "feedback".into(),
            round,
            session: Some(session),
        },
    )
}

#[tauri::command]
pub async fn import_editorial_feedback(
    package: Package,
    state: State<'_, AppState>,
) -> Result<Feedback> {
    import_feedback(&*state.db.lock().map_err(err)?, &package)
}

#[tauri::command]
pub async fn get_editorial_feedback(
    round_id: String,
    state: State<'_, AppState>,
) -> Result<Feedback> {
    feedback(&*state.db.lock().map_err(err)?, &round_id)
}

#[derive(Deserialize)]
pub struct Replacement {
    pub id: String,
    pub expected: String,
    pub html: String,
}

fn decide(
    conn: &Connection,
    round_id: &str,
    version: i64,
    keys: &[String],
    decision: &str,
    replacements: &[Replacement],
) -> Result<Feedback> {
    if !["accepted", "rejected", "resolved", "open"].contains(&decision) {
        return Err("Invalid feedback decision".into());
    }
    if decision != "accepted" && !replacements.is_empty() {
        return Err("Only acceptance can change prose".into());
    }
    let tx = conn.unchecked_transaction().map_err(err)?;
    let mut data = feedback(&tx, round_id)?;
    if data.version != version {
        return Err("Feedback changed. Refresh the review before deciding.".into());
    }
    for key in keys {
        let entry = data
            .entries
            .iter_mut()
            .find(|e| &e.key == key)
            .ok_or("Feedback no longer exists")?;
        if decision == "accepted" && (entry.decision != "open" || entry.change.kind != "suggestion")
        {
            return Err("Only pending suggestions can be accepted.".into());
        }
        entry.decision = decision.into();
        entry.decided_by_writer = true;
    }
    let mut scenes = HashSet::new();
    let mut ids = HashSet::new();
    for replacement in replacements {
        let source = data.sources.iter().find(|s| s.id == replacement.id).ok_or(
            "The manuscript structure or editing mode changed. Refresh and review this passage.",
        )?;
        if source.locked {
            return Err(format!("‘{}’ is locked in your project. Unlock its scene or chapter before accepting these suggestions.", source.scene));
        }
        if source.html != replacement.expected || !ids.insert(&replacement.id) {
            return Err("The manuscript changed. Refresh before accepting changes.".into());
        }
        scenes.insert(source.scene_id.clone());
    }
    for scene_id in scenes {
        let id = Uuid::parse_str(&scene_id).map_err(err)?;
        let review = db::revisions::load(&tx, &id)?;
        let mut history = review.data.clone();
        history.drafts.push(db::revisions::ReviewDraft {
            name: format!("Before accepting {}", data.round.name),
            created_at: chrono::Utc::now().to_rfc3339(),
            mode: review.mode,
            documents: review.documents.clone(),
        });
        history.status = "revised".into();
        let mut next = history.drafts.last().unwrap().clone();
        for doc in &mut next.documents {
            if let Some(change) = replacements.iter().find(|r| r.id == doc.id) {
                doc.html = change.html.clone();
            }
        }
        db::revisions::save_in_transaction(&tx, &review, &history, Some(&next))?;
    }
    tx.execute(
        "UPDATE editorial_rounds SET feedback=?1, version=version+1 WHERE id=?2",
        params![serde_json::to_string(&data.entries).map_err(err)?, round_id],
    )
    .map_err(err)?;
    tx.commit().map_err(err)?;
    feedback(conn, round_id)
}

#[tauri::command]
pub async fn decide_editorial_feedback(
    round_id: String,
    version: i64,
    keys: Vec<String>,
    decision: String,
    replacements: Vec<Replacement>,
    state: State<'_, AppState>,
) -> Result<Feedback> {
    decide(
        &*state.db.lock().map_err(err)?,
        &round_id,
        version,
        &keys,
        &decision,
        &replacements,
    )
}

#[tauri::command]
pub async fn list_editorial_rounds(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Round>> {
    let conn = state.db.lock().map_err(err)?;
    let mut statement = conn
        .prepare("SELECT data FROM editorial_rounds WHERE project_id=?1 ORDER BY rowid DESC")
        .map_err(err)?;
    let rows = statement
        .query_map([project_id], |r| r.get::<_, String>(0))
        .map_err(err)?;
    rows.map(|r| serde_json::from_str(&r.map_err(err)?).map_err(err))
        .collect()
}

#[tauri::command]
pub async fn reply_editorial_feedback(
    round_id: String,
    version: i64,
    key: String,
    message: Message,
    state: State<'_, AppState>,
) -> Result<Feedback> {
    if message.author.trim().is_empty() || message.text.trim().is_empty() {
        return Err("Enter your name and reply.".into());
    }
    let conn = state.db.lock().map_err(err)?;
    let mut data = feedback(&conn, &round_id)?;
    if data.version != version {
        return Err("Feedback changed. Refresh before replying.".into());
    }
    let entry = data
        .entries
        .iter_mut()
        .find(|e| e.key == key)
        .ok_or("Feedback no longer exists")?;
    if !entry.change.messages.iter().any(|m| m.id == message.id) {
        entry.change.messages.push(message);
    }
    conn.execute(
        "UPDATE editorial_rounds SET feedback=?1, version=version+1 WHERE id=?2",
        params![serde_json::to_string(&data.entries).map_err(err)?, round_id],
    )
    .map_err(err)?;
    feedback(&conn, &round_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Beat, Chapter, Project, Scene, SourceType};

    fn fixture() -> (Connection, Package) {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = Project::new("The Letter".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "The Letter".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "On the Cliff".into(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();
        for position in 0..2 {
            let mut beat = Beat::new(scene.id, "Read the letter".into(), position);
            beat.prose = Some(format!("<p>Passage {position}</p>"));
            db::insert_beat(&conn, &beat).unwrap();
        }
        let round = Round {
            id: Uuid::new_v4().to_string(),
            project_id: project.id.to_string(),
            title: project.name,
            name: "Continuity pass".into(),
            brief: "Check Eleanor's motivation".into(),
            created_at: "2026-01-15T09:00:00Z".into(),
            sources: sources(&conn, &project.id.to_string()).unwrap(),
        };
        conn.execute(
            "INSERT INTO editorial_rounds(id,project_id,data) VALUES (?1,?2,?3)",
            params![
                round.id,
                round.project_id,
                serde_json::to_string(&round).unwrap()
            ],
        )
        .unwrap();
        let package = Package {
            format: "kindling-editorial".into(),
            version: 1,
            kind: "feedback".into(),
            round,
            session: Some(Session {
                reviewer_id: Uuid::new_v4().to_string(),
                name: "Rowan".into(),
                generation: 1,
                document: serde_json::json!({"type":"doc"}),
                position: 1,
                reading_position: 1,
                writer_version: 0,
                changes: vec![Change {
                    id: "first".into(),
                    revision: 1,
                    kind: "suggestion".into(),
                    from: 1,
                    to: 4,
                    before: Value::Null,
                    after: Value::Null,
                    state: "open".into(),
                    messages: vec![],
                    anchor_offset: None,
                    writer_decision: None,
                }],
            }),
        };
        (conn, package)
    }

    #[test]
    fn recovery_resumes_without_a_writer_project_and_uses_the_local_cas_generation() {
        let (conn, mut package) = fixture();
        package.kind = "review".into();
        let mut saved = package.session.as_ref().unwrap().clone();
        saved.generation = 4;
        conn.execute(
            "INSERT INTO editorial_sessions(round_id,round_data,data) VALUES (?1,?2,?3)",
            params![
                package.round.id,
                serde_json::to_string(&package.round).unwrap(),
                serde_json::to_string(&saved).unwrap()
            ],
        )
        .unwrap();
        package.session.as_mut().unwrap().generation = 6;
        package.session.as_mut().unwrap().document = serde_json::json!({"recovered":"work"});
        let opened = resume_package(&conn, package.clone()).unwrap();
        assert_eq!(opened.saved_generation, Some(4));
        assert_eq!(
            opened.package.session.unwrap().document,
            serde_json::json!({"recovered":"work"})
        );
        let fresh = Connection::open_in_memory().unwrap();
        db::initialize_schema(&fresh).unwrap();
        let opened = resume_package(&fresh, package).unwrap();
        assert_eq!(opened.saved_generation, None);
        assert_eq!(opened.package.session.unwrap().generation, 6);
    }

    #[test]
    fn writer_replies_and_decisions_return_without_replacing_newer_editor_work() {
        let (conn, package) = fixture();
        let data = import_feedback(&conn, &package).unwrap();
        let mut entries = data.entries;
        entries[0].decision = "accepted".into();
        entries[0].decided_by_writer = true;
        entries[0].change.messages.push(Message {
            id: "writer-note".into(),
            author: "Writer".into(),
            text: "Thank you; this clarifies Eleanor's choice.".into(),
            created_at: "today".into(),
        });
        conn.execute(
            "UPDATE editorial_rounds SET feedback=?1 WHERE id=?2",
            params![serde_json::to_string(&entries).unwrap(), package.round.id],
        )
        .unwrap();
        let mut local = package.session.as_ref().unwrap().clone();
        local.generation = 9;
        local.document = serde_json::json!({"newer":"editor work"});
        conn.execute(
            "INSERT INTO editorial_sessions(round_id,round_data,data) VALUES (?1,?2,?3)",
            params![
                package.round.id,
                serde_json::to_string(&package.round).unwrap(),
                serde_json::to_string(&local).unwrap()
            ],
        )
        .unwrap();
        let response = writer_response(&conn, &package.round.id, &local.reviewer_id).unwrap();
        let opened = resume_package(&conn, response).unwrap();
        let resumed = opened.package.session.unwrap();
        assert_eq!(resumed.document, local.document);
        assert_eq!(
            resumed.changes[0].writer_decision.as_deref(),
            Some("accepted")
        );
        assert_eq!(resumed.changes[0].messages[0].id, "writer-note");
        assert_eq!(opened.saved_generation, Some(9));
    }

    #[test]
    fn repeated_writer_responses_merge_refined_threads_and_keep_one_withdrawn_discussion() {
        let (conn, mut package) = fixture();
        package.session.as_mut().unwrap().changes[0]
            .messages
            .push(Message {
                id: "old-note".into(),
                author: "Rowan".into(),
                text: "Opening discussion".into(),
                created_at: "today".into(),
            });
        import_feedback(&conn, &package).unwrap();
        let session = package.session.as_mut().unwrap();
        session.generation = 2;
        session.changes[0].revision = 2;
        let reviewer = session.reviewer_id.clone();
        import_feedback(&conn, &package).unwrap();
        let mut response = writer_response(&conn, &package.round.id, &reviewer).unwrap();
        assert_eq!(response.session.as_ref().unwrap().changes.len(), 1);
        assert_eq!(response.session.as_ref().unwrap().changes[0].id, "first");
        response.kind = "feedback".into();
        response.session.as_mut().unwrap().generation = 3;
        response.session.as_mut().unwrap().changes.clear();
        import_feedback(&conn, &response).unwrap();
        let mut response = writer_response(&conn, &package.round.id, &reviewer).unwrap();
        assert_eq!(response.session.as_ref().unwrap().changes.len(), 1);
        assert_eq!(
            response.session.as_ref().unwrap().changes[0].id,
            "discussion-first"
        );
        response.kind = "feedback".into();
        response.session.as_mut().unwrap().generation = 4;
        import_feedback(&conn, &response).unwrap();
        let next = writer_response(&conn, &package.round.id, &reviewer).unwrap();
        validate(&next).unwrap();
        let changes = next.session.unwrap().changes;
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].id, "discussion-first");
        assert_eq!(changes[0].messages.len(), 1);
    }

    #[test]
    fn stale_writer_responses_cannot_reverse_newer_decisions_even_in_a_recovery() {
        let (conn, mut package) = fixture();
        package.kind = "review".into();
        let mut local = package.session.as_ref().unwrap().clone();
        local.generation = 9;
        local.writer_version = 2;
        local.changes[0].writer_decision = Some("open".into());
        conn.execute(
            "INSERT INTO editorial_sessions(round_id,round_data,data) VALUES (?1,?2,?3)",
            params![
                package.round.id,
                serde_json::to_string(&package.round).unwrap(),
                serde_json::to_string(&local).unwrap()
            ],
        )
        .unwrap();
        let incoming = package.session.as_mut().unwrap();
        incoming.writer_version = 1;
        incoming.changes[0].writer_decision = Some("resolved".into());
        incoming.changes[0].messages.push(Message {
            id: "late-note".into(),
            author: "Writer".into(),
            text: "Still preserve this discussion.".into(),
            created_at: "today".into(),
        });
        for generation in [1, 10] {
            package.session.as_mut().unwrap().generation = generation;
            let opened = resume_package(&conn, package.clone())
                .unwrap()
                .package
                .session
                .unwrap();
            assert_eq!(opened.writer_version, 2);
            assert_eq!(opened.changes[0].writer_decision.as_deref(), Some("open"));
            assert_eq!(opened.changes[0].messages.len(), 1);
        }
    }

    #[test]
    fn package_round_trip_is_portable_versioned_and_validated() {
        let (_conn, package) = fixture();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("review.kindling-feedback");
        write_package(&path, &package).unwrap();
        let imported = read_package(&path).unwrap();
        assert_eq!(imported.round, package.round);
        assert_eq!(imported.session.unwrap().changes.len(), 1);
        write_package(&path, &package).unwrap(); // replacing an exported response
        let mut invalid = package.clone();
        invalid.version = 99;
        assert!(write_package(&path, &invalid)
            .unwrap_err()
            .contains("version"));
        assert!(read_package(&path).is_ok());
        fs::write(&path, "not json").unwrap();
        assert!(read_package(&path).is_err());
        invalid = package.clone();
        invalid.round.sources.push(invalid.round.sources[0].clone());
        assert!(validate(&invalid).is_err());
        invalid = package.clone();
        invalid.session = None;
        assert!(validate(&invalid).is_err());
        invalid = package;
        invalid.session.as_mut().unwrap().changes[0].from = 5;
        assert!(validate(&invalid).is_err());
    }

    #[test]
    fn repeated_partial_returns_merge_without_overwriting_decisions_or_prose() {
        let (conn, mut package) = fixture();
        let first = import_feedback(&conn, &package).unwrap();
        let key = first.entries[0].key.clone();
        assert_eq!(first.sources, package.round.sources);
        let rejected = decide(
            &conn,
            &package.round.id,
            first.version,
            &[key],
            "rejected",
            &[],
        )
        .unwrap();
        assert_eq!(
            import_feedback(&conn, &package).unwrap().version,
            rejected.version
        );
        let session = package.session.as_mut().unwrap();
        session.generation += 1;
        let mut second = session.changes[0].clone();
        second.id = "second".into();
        session.changes.push(second);
        let imported = import_feedback(&conn, &package).unwrap();
        assert_eq!(imported.entries.len(), 2);
        assert_eq!(imported.entries[0].decision, "rejected");
        assert_eq!(imported.sources, package.round.sources);
        package.session.as_mut().unwrap().generation = 1;
        assert_eq!(import_feedback(&conn, &package).unwrap().entries.len(), 2);
    }

    #[test]
    fn changed_suggestions_keep_old_decisions_and_withdraw_superseded_pending_versions() {
        let (conn, mut package) = fixture();
        import_feedback(&conn, &package).unwrap();
        let session = package.session.as_mut().unwrap();
        session.generation += 1;
        session.changes[0].revision += 1;
        let current = import_feedback(&conn, &package).unwrap();
        assert_eq!(current.entries.len(), 2);
        assert_eq!(current.entries[0].decision, "withdrawn");
        assert_eq!(current.entries[1].decision, "open");
        package.session.as_mut().unwrap().generation += 1;
        package.session.as_mut().unwrap().changes.clear();
        assert!(import_feedback(&conn, &package)
            .unwrap()
            .entries
            .iter()
            .all(|e| e.decision == "withdrawn"));
    }

    #[test]
    fn mismatched_round_or_equal_generation_different_content_is_rejected() {
        let (conn, mut package) = fixture();
        import_feedback(&conn, &package).unwrap();
        package.session.as_mut().unwrap().name = "Changed without saving".into();
        assert!(import_feedback(&conn, &package)
            .unwrap_err()
            .contains("same review generation"));
        package.round.sources[0].html = "Different baseline".into();
        assert!(import_feedback(&conn, &package)
            .unwrap_err()
            .contains("does not match"));
        package.round.id = Uuid::new_v4().to_string();
        assert!(import_feedback(&conn, &package).is_err());
    }

    #[test]
    fn acceptance_is_atomic_preserves_drafts_and_does_not_award_writing_credit() {
        let (conn, package) = fixture();
        let current = import_feedback(&conn, &package).unwrap();
        let replacements: Vec<_> = current
            .sources
            .iter()
            .map(|s| Replacement {
                id: s.id.clone(),
                expected: s.html.clone(),
                html: "<p>Edited</p>".into(),
            })
            .collect();
        let result = decide(
            &conn,
            &package.round.id,
            current.version,
            &[current.entries[0].key.clone()],
            "accepted",
            &replacements,
        )
        .unwrap();
        assert!(result.sources.iter().all(|s| s.html == "<p>Edited</p>"));
        let scene = Uuid::parse_str(&result.sources[0].scene_id).unwrap();
        let saved = db::revisions::load(&conn, &scene).unwrap();
        assert_eq!(saved.data.drafts.len(), 1);
        assert!(saved.data.drafts[0]
            .documents
            .iter()
            .any(|d| d.html == "<p>Passage 0</p>"));
        assert_eq!(saved.data.status, "revised");
        let stats = db::writing::stats(
            &conn,
            &Uuid::parse_str(&package.round.project_id).unwrap(),
            chrono::Local::now().date_naive(),
        )
        .unwrap();
        assert_eq!(stats.today_words, 0);
        assert!(decide(
            &conn,
            &package.round.id,
            current.version,
            &[],
            "rejected",
            &[]
        )
        .is_err());
    }

    #[test]
    fn stale_locked_and_foreign_replacements_leave_prose_and_decisions_untouched() {
        let (conn, package) = fixture();
        let current = import_feedback(&conn, &package).unwrap();
        let keys = vec![current.entries[0].key.clone()];
        for id in [current.sources[1].id.clone(), Uuid::new_v4().to_string()] {
            let replacements = vec![
                Replacement {
                    id: current.sources[0].id.clone(),
                    expected: current.sources[0].html.clone(),
                    html: "changed".into(),
                },
                Replacement {
                    id,
                    expected: "stale".into(),
                    html: "changed".into(),
                },
            ];
            assert!(decide(
                &conn,
                &package.round.id,
                current.version,
                &keys,
                "accepted",
                &replacements
            )
            .is_err());
            assert_eq!(
                feedback(&conn, &package.round.id).unwrap().sources,
                current.sources
            );
        }
        db::lock_scene(
            &conn,
            &Uuid::parse_str(&current.sources[0].scene_id).unwrap(),
        )
        .unwrap();
        let replacement = Replacement {
            id: current.sources[0].id.clone(),
            expected: current.sources[0].html.clone(),
            html: "changed".into(),
        };
        assert!(decide(
            &conn,
            &package.round.id,
            current.version,
            &keys,
            "accepted",
            &[replacement]
        )
        .unwrap_err()
        .contains("locked"));
        assert_eq!(
            feedback(&conn, &package.round.id).unwrap().entries[0].decision,
            "open"
        );
        assert!(db::revisions::load(
            &conn,
            &Uuid::parse_str(&current.sources[0].scene_id).unwrap()
        )
        .unwrap()
        .data
        .drafts
        .is_empty());
    }

    #[test]
    fn independent_reviewers_and_subsequent_rounds_do_not_collide() {
        let (conn, mut package) = fixture();
        import_feedback(&conn, &package).unwrap();
        package.session.as_mut().unwrap().reviewer_id = Uuid::new_v4().to_string();
        assert_eq!(import_feedback(&conn, &package).unwrap().entries.len(), 2);
        package.round.id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO editorial_rounds(id,project_id,data) VALUES (?1,?2,?3)",
            params![
                package.round.id,
                package.round.project_id,
                serde_json::to_string(&package.round).unwrap()
            ],
        )
        .unwrap();
        assert_eq!(import_feedback(&conn, &package).unwrap().entries.len(), 1);
    }
}
