//! Editorial examples in the public sample, with anchors in ProseMirror UTF-16
//! positions. Each quote below belongs to the first paragraph of its source.
use crate::{
    db,
    db::revisions::{self, Annotation, ReviewDraft, ReviewMessage},
    models::EditorMode,
};
use rusqlite::{params, Connection};
use uuid::Uuid;

const DATE: &str = "2026-01-15T09:00:00Z";
fn note(author: &str, text: &str) -> ReviewMessage {
    ReviewMessage {
        author: author.into(),
        text: text.into(),
        created_at: DATE.into(),
    }
}
fn annotation(
    doc: &revisions::ReviewDocument,
    from: usize,
    quote: &str,
    replacement: Option<&str>,
    state: &str,
    reason: &str,
) -> Annotation {
    Annotation {
        id: Uuid::new_v4().to_string(),
        document_id: doc.id.clone(),
        anchor_html: doc.html.clone(),
        from,
        to: from + quote.encode_utf16().count(),
        quote: quote.into(),
        replacement: replacement.map(str::to_string),
        state: state.into(),
        messages: vec![note("Rowan Ellis · Editor", reason)],
    }
}

pub fn seed(conn: &Connection, project: &Uuid) -> rusqlite::Result<()> {
    for scene in db::get_all_project_scenes(conn, project)? {
        let mut review =
            revisions::load(conn, &scene.id).map_err(rusqlite::Error::InvalidParameterName)?;
        let doc = review
            .documents
            .iter()
            .find(|d| {
                !d.html.is_empty()
                    && (review.mode == EditorMode::Page || d.id != scene.id.to_string())
            })
            .cloned();
        let Some(doc) = doc else {
            continue;
        };
        let draft = |name: &str| ReviewDraft {
            name: name.into(),
            created_at: DATE.into(),
            mode: review.mode,
            documents: review.documents.clone(),
        };
        match scene.title.as_str() {
            "On the Cliff" => {
                review.data.status = "editor_review".into();
                review.data.drafts.push(draft("First complete cliff scene"));
                let mut earlier = draft("Opening before the letter became a warning");
                earlier
                    .documents
                    .iter_mut()
                    .find(|d| d.id == doc.id)
                    .unwrap()
                    .html = doc.html.replace(
                    "Trust no one—especially not the man who calls himself your father.",
                    "Ask Silas what happened the night the lamp went dark.",
                );
                review.data.drafts.insert(0, earlier);
                let mut thread = annotation(&doc, 1, "The lighthouse keeper's daughter", None, "open", "Could we name Eleanor sooner? Her relationship to the keeper matters, but I want to meet her as a person before the mystery takes over.");
                thread.messages.push(note("E. M. Hale · Writer", "I held her name back to foreground Silas. Does the second paragraph arrive too late?"));
                thread.messages.push(note("Rowan Ellis · Editor", "A little. Try her name in the first sentence and let the letter supply the relationship."));
                review.data.annotations.push(thread);
                review.data.annotations.push(annotation(
                    &doc,
                    1,
                    "The",
                    Some("A"),
                    "open",
                    "Try a less declarative opening; compare it aloud with the original.",
                ));
                review.data.annotations.push(annotation(
                    &doc,
                    5,
                    "lighthouse",
                    Some("light"),
                    "open",
                    "A shorter compound may make the first line easier to read.",
                ));
                review.data.annotations.push(annotation(
                    &doc,
                    16,
                    "keeper's",
                    Some(""),
                    "open",
                    "Test removing the possessive to see what the sentence loses.",
                ));
                review.data.annotations.push(annotation(
                    &doc,
                    1,
                    "",
                    Some("At dusk, "),
                    "open",
                    "Ground the opening in time before the letter changes her evening.",
                ));
            }
            "The Seventh Step" => {
                review.data.status = "editor_review".into();
                review
                    .data
                    .drafts
                    .push(draft("Before splitting the discovery into beats"));
                let mut obsolete = draft("Single-beat discovery");
                obsolete
                    .documents
                    .retain(|d| d.id == scene.id.to_string() || d.id == doc.id);
                review.data.drafts.insert(0, obsolete);
                review.data.annotations.push(annotation(
                    &doc,
                    1,
                    "It came up",
                    Some("The stone lifted"),
                    "open",
                    "Name the stone so the action is immediately clear.",
                ));
                review.data.annotations.push(annotation(&doc, 1, "It came up without protest.", Some("The step lifted too easily."), "open", "An alternative to the shorter change: connect the ease of lifting to Eleanor's suspicion."));
                let mut stale = annotation(&doc, 1, "The stone came up", Some("The stone lifted"), "open", "This note predates the opening rewrite. Select the intended new wording to re-anchor it.");
                stale.anchor_html = doc.html.replacen("It came up", "The stone came up", 1);
                review.data.annotations.push(stale);
            }
            "Supper with Silas" => {
                review.data.status = "revised".into();
                let mut old = draft("Before accepting changes");
                old.documents
                    .iter_mut()
                    .find(|d| d.id == doc.id)
                    .unwrap()
                    .html = doc.html.replace("three places", "two places");
                review.data.drafts.push(old.clone());
                review.data.drafts.push(draft("The empty chair pass"));
                let old_doc = old.documents.iter().find(|d| d.id == doc.id).unwrap();
                review.data.annotations.push(annotation(old_doc, 12, "two places", Some("three places"), "accepted", "An unused place setting gives the scene a physical question before Eleanor speaks."));
                review.data.annotations.push(annotation(
                    &doc,
                    1,
                    "Silas",
                    Some("Her father"),
                    "rejected",
                    "Would the family relationship increase the tension here?",
                ));
                let mut resolved = annotation(
                    &doc,
                    1,
                    "Silas",
                    None,
                    "resolved",
                    "Keep Silas's name. Eleanor is beginning to think of him as a stranger.",
                );
                resolved.messages.push(note(
                    "E. M. Hale · Writer",
                    "Agreed. I kept his name and let the empty chair carry the tension.",
                ));
                review.data.annotations.push(resolved);
            }
            "Low Tide" => {
                review.data.status = "first_draft".into();
            }
            "Margaret's Room" => {
                review.data.status = "final".into();
                review.data.drafts.push(draft("Final continuity pass"));
                review.data.annotations.push(annotation(&doc, 1, "Nothing in the room", None, "resolved", "The untouched room pays off the abandoned-child belief beautifully. The sixteen-year interval now agrees with the letter."));
                let mut earlier = draft("Before restoring the quieter final opening");
                let old_doc = earlier
                    .documents
                    .iter_mut()
                    .find(|d| d.id == doc.id)
                    .unwrap();
                old_doc.html =
                    old_doc
                        .html
                        .replacen("Nothing in the room", "Nothing in Margaret's room", 1);
                review.data.annotations.push(annotation(old_doc, 12, "Margaret's", Some("the"), "accepted", "The scene title already names Margaret; the simpler opening keeps attention on the evidence of waiting."));
                review.data.drafts.insert(0, earlier);
            }
            "The Damaged Register" => {
                review.data.status = "editor_review".into();
                let mut reopened = annotation(&doc, 1, "The", None, "open", "Reopened: verify what the townspeople know about Margaret before Eleanor asks this question.");
                reopened.messages.push(note(
                    "E. M. Hale · Writer",
                    "Resolved after the first continuity pass.",
                ));
                reopened.messages.push(note("Rowan Ellis · Editor", "Reopening after reading the keeper's ledger: their silence may imply more knowledge than intended."));
                // This scene's first word is used verbatim, regardless of future sample prose changes.
                let first = doc
                    .html
                    .strip_prefix("<p>")
                    .unwrap_or("")
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                reopened.quote = first.into();
                reopened.to = 1 + first.encode_utf16().count();
                review.data.annotations.push(reopened);
                let inactive = review.documents.iter_mut().find(|d| d.id != doc.id);
                if let Some(inactive) = inactive {
                    inactive.html = "<p>The town kept its silence.</p>".into();
                    review.data.annotations.push(annotation(inactive, 1, "The town", None, "open", "A note on the earlier beat-mode prose. Return to that mode to review this source."));
                    let id = Uuid::parse_str(&inactive.id).unwrap();
                    if id == scene.id {
                        db::update_scene_prose(conn, &id, &inactive.html)?;
                    } else {
                        db::update_beat_prose(conn, &id, &inactive.html)?;
                    }
                }
            }
            _ => continue,
        }
        conn.execute("INSERT INTO scene_reviews(scene_id,version,data) VALUES (?1,1,?2) ON CONFLICT(scene_id) DO UPDATE SET version=version+1,data=excluded.data", params![scene.id.to_string(), serde_json::to_string(&review.data).unwrap()])?;
        let status = match review.data.status.as_str() {
            "final" => "final",
            "revised" => "revised",
            _ => "draft",
        };
        conn.execute(
            "UPDATE scenes SET scene_status=?1 WHERE id=?2",
            params![status, scene.id.to_string()],
        )?;
        if scene.title == "Margaret's Room" {
            db::lock_scene(conn, &scene.id)?;
        }
    }
    Ok(())
}
