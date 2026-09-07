//! Saved-prose accounting. Only explicit prose saves earn writing credit; imports,
//! mode switches and structural edits never manufacture words written.
use std::collections::HashMap;

use chrono::NaiveDate;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::Serialize;
use uuid::Uuid;

use crate::parsers::html::{html_events, HtmlEvent};

pub fn count_words(html: &str) -> i64 {
    let mut text = String::new();
    for event in html_events(html) {
        match event {
            HtmlEvent::Text(value) => text.push_str(&value),
            HtmlEvent::Start(tag) | HtmlEvent::End(tag) | HtmlEvent::Empty(tag)
                if matches!(
                    tag.as_str(),
                    "p" | "div" | "br" | "hr" | "li" | "h1" | "h2" | "h3" | "blockquote"
                ) =>
            {
                text.push(' ');
            }
            _ => (),
        }
    }
    text.split_whitespace().count() as i64
}

pub fn scene_words(conn: &Connection, scene_id: &Uuid) -> Result<i64> {
    let scene =
        super::get_scene_by_id(conn, scene_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    let beats = super::get_beats(conn, scene_id)?;
    Ok(
        if scene.editor_mode == crate::models::EditorMode::Page || beats.is_empty() {
            count_words(scene.prose.as_deref().unwrap_or(""))
        } else {
            beats
                .iter()
                .map(|b| count_words(b.prose.as_deref().unwrap_or("")))
                .sum()
        },
    )
}

pub fn daily_goal(conn: &Connection, project_id: &str) -> Result<i64> {
    Ok(conn
        .query_row(
            "SELECT daily_goal FROM writing_goals WHERE project_id = ?1",
            [project_id],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or(500))
}

pub fn set_goal(conn: &Connection, project_id: &str, goal: i64, today: NaiveDate) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    set_goal_in_transaction(&tx, project_id, goal, today)?;
    tx.commit()
}

/// Caller holds the transaction for metadata and goal updates.
pub fn set_goal_in_transaction(
    tx: &Connection,
    project_id: &str,
    goal: i64,
    today: NaiveDate,
) -> Result<()> {
    tx.execute(
        "INSERT INTO writing_goals(project_id, daily_goal) VALUES (?1, ?2)
        ON CONFLICT(project_id) DO UPDATE SET daily_goal = excluded.daily_goal",
        params![project_id, goal],
    )?;
    // Past days retain their target; a change applies to today's progress and future days.
    tx.execute(
        "UPDATE writing_sessions SET goal = ?1 WHERE project_id = ?2 AND date = ?3",
        params![goal, project_id, today.to_string()],
    )?;
    Ok(())
}

#[derive(Clone, Copy)]
pub enum ProseTarget {
    Beat(Uuid),
    Scene(Uuid),
}

fn saved_document(conn: &Connection, target: ProseTarget) -> Result<(Uuid, i64)> {
    match target {
        ProseTarget::Beat(id) => {
            let beat = super::get_beat(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            Ok((
                beat.scene_id,
                count_words(beat.prose.as_deref().unwrap_or("")),
            ))
        }
        ProseTarget::Scene(id) => {
            let scene =
                super::get_scene_by_id(conn, &id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
            Ok((id, count_words(scene.prose.as_deref().unwrap_or(""))))
        }
    }
}

/// The prose and accounting updates share one transaction, so failed/retried
/// saves cannot lose writing credit or count the same edit twice.
pub fn save_prose(
    conn: &Connection,
    target: ProseTarget,
    today: NaiveDate,
    save: impl FnOnce(&Connection) -> Result<()>,
) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    save_prose_in_transaction(&tx, target, today, save)?;
    tx.commit()
}

/// Caller must hold a transaction spanning the prose and accounting writes.
pub fn save_prose_in_transaction(
    tx: &Connection,
    target: ProseTarget,
    today: NaiveDate,
    save: impl FnOnce(&Connection) -> Result<()>,
) -> Result<()> {
    // Measure the document being saved, including recovered drafts from an older editor mode.
    let (scene_id, before) = saved_document(tx, target)?;
    save(tx)?;
    let delta = saved_document(tx, target)?.1 - before;
    let project_id =
        super::get_scene_project_id(tx, &scene_id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    if delta != 0 {
        let goal = daily_goal(tx, &project_id.to_string())?;
        tx.execute(
            "INSERT INTO writing_sessions(project_id, date, words, goal) VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(project_id, date) DO UPDATE SET words = words + excluded.words",
            params![project_id.to_string(), today.to_string(), delta, goal],
        )?;
        tx.execute(
            "INSERT INTO writing_runtime(project_id, words) VALUES (?1, ?2)
            ON CONFLICT(project_id) DO UPDATE SET words = words + excluded.words",
            params![project_id.to_string(), delta],
        )?;
    }
    super::update_project_modified(tx, &project_id)?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct WritingStats {
    pub project_id: String,
    pub project_words: i64,
    pub chapter_words: HashMap<String, i64>,
    pub scene_words: HashMap<String, i64>,
    pub daily_goal: i64,
    pub today_words: i64,
    pub session_words: i64,
    pub streak: usize,
}

pub fn stats(conn: &Connection, project_id: &Uuid, today: NaiveDate) -> Result<WritingStats> {
    if super::get_project(conn, project_id)?.is_none() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    let id = project_id.to_string();
    let mut stats = WritingStats {
        project_id: id.clone(),
        project_words: 0,
        chapter_words: HashMap::new(),
        scene_words: HashMap::new(),
        daily_goal: daily_goal(conn, &id)?,
        today_words: conn
            .query_row(
                "SELECT words FROM writing_sessions WHERE project_id = ?1 AND date = ?2",
                params![id, today.to_string()],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0),
        session_words: conn
            .query_row(
                "SELECT words FROM writing_runtime WHERE project_id = ?1",
                [&id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0),
        streak: 0,
    };
    // One bulk read avoids per-scene queries while holding the application's DB mutex.
    let mut stmt = conn.prepare(
        "SELECT c.id, s.id, s.editor_mode, s.prose, b.id, b.prose
        FROM chapters c
        LEFT JOIN scenes s ON s.chapter_id = c.id AND s.archived = 0
        LEFT JOIN beats b ON b.scene_id = s.id
        WHERE c.project_id = ?1 AND c.archived = 0",
    )?;
    let mut rows = stmt.query([&id])?;
    while let Some(row) = rows.next()? {
        let chapter_id: String = row.get(0)?;
        let chapter_words = stats.chapter_words.entry(chapter_id).or_default();
        let Some(scene_id) = row.get::<_, Option<String>>(1)? else {
            continue;
        };
        let mode: String = row.get(2)?;
        let beat_id: Option<String> = row.get(4)?;
        let count = if mode == "page" || beat_id.is_none() {
            // A Page View scene may join several stale beat copies; count the page once.
            if stats.scene_words.contains_key(&scene_id) {
                0
            } else {
                count_words(row.get::<_, Option<String>>(3)?.as_deref().unwrap_or(""))
            }
        } else {
            count_words(row.get::<_, Option<String>>(5)?.as_deref().unwrap_or(""))
        };
        *stats.scene_words.entry(scene_id).or_default() += count;
        *chapter_words += count;
        stats.project_words += count;
    }
    let mut stmt = conn.prepare("SELECT date FROM writing_sessions WHERE project_id = ?1 AND goal > 0 AND words >= goal AND date <= ?2 ORDER BY date DESC")?;
    let days = stmt
        .query_map(params![id, today.to_string()], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>>>()?;
    let mut expected = today;
    for (index, date) in days.iter().enumerate() {
        // An unfinished today does not break yesterday's streak until midnight.
        if index == 0 && date != &today.to_string() {
            expected = today.pred_opt().unwrap_or(today);
        }
        if date != &expected.to_string() {
            break;
        }
        stats.streak += 1;
        expected = expected.pred_opt().unwrap_or(expected);
    }
    Ok(stats)
}

pub fn reset_session(conn: &Connection, project_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM writing_runtime WHERE project_id = ?1",
        [project_id],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db,
        models::{Beat, Chapter, Project, Scene, SourceType},
    };

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, day).unwrap()
    }
    fn seed(conn: &Connection) -> (Project, Chapter, Scene, Beat) {
        db::initialize_schema(conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let project = Project::new("Novel".into(), SourceType::Blank, None);
        db::insert_project(conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        db::insert_chapter(conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "Scene".into(), None, 0);
        db::insert_scene(conn, &scene).unwrap();
        let beat = Beat::new(scene.id, "Outline doesn't count".into(), 0);
        db::insert_beat(conn, &beat).unwrap();
        (project, chapter, scene, beat)
    }
    fn save(conn: &Connection, _scene: &Scene, beat: &Beat, day: u32, prose: &str) {
        save_prose(conn, ProseTarget::Beat(beat.id), date(day), |tx| {
            db::update_beat_prose(tx, &beat.id, prose)
        })
        .unwrap();
    }

    #[test]
    fn words_preserve_inline_marks_and_decode_entities() {
        assert_eq!(
            count_words("<p>hel<strong>lo</strong>&nbsp;world</p><p>next<br/>line &#32; end</p>"),
            5
        );
        assert_eq!(count_words("<p>&nbsp; &#160;</p>"), 0);
        assert_eq!(
            count_words("<p>one</p><!-- ignored --><p>two &amp; three</p>"),
            4
        );
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn net_saves_are_idempotent_and_ignore_imports() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, c, s, b) = seed(&conn);
        db::update_beat_prose(&conn, &b.id, "imported words").unwrap();
        save(&conn, &s, &b, 7, "imported words plus two");
        save(&conn, &s, &b, 7, "imported words plus two");
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!(
            (stat.project_words, stat.today_words, stat.session_words),
            (4, 2, 2)
        );
        assert_eq!(stat.chapter_words[&c.id.to_string()], 4);
        assert_eq!(stat.scene_words[&s.id.to_string()], 4);
        save(&conn, &s, &b, 7, "imported");
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!((stat.today_words, stat.session_words), (-1, -1));
    }

    #[test]
    fn mode_switches_do_not_duplicate_counts_or_credit() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, _, s, b) = seed(&conn);
        save(&conn, &s, &b, 7, "one two");
        db::switch_scene_editor_mode(&conn, &s.id, "page").unwrap();
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().project_words, 2);
        save_prose(&conn, ProseTarget::Scene(s.id), date(7), |tx| {
            db::save_scene_page_prose(tx, &s.id, "one two three")
        })
        .unwrap();
        db::switch_scene_editor_mode(&conn, &s.id, "beat").unwrap();
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!((stat.project_words, stat.today_words), (3, 3));
    }

    #[test]
    fn failed_accounting_rolls_back_prose_and_retry_counts_once() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, _, s, b) = seed(&conn);
        conn.execute_batch("CREATE TRIGGER fail_writing BEFORE INSERT ON writing_sessions BEGIN SELECT RAISE(ABORT, 'disk failure'); END;").unwrap();
        assert!(save_prose(&conn, ProseTarget::Beat(b.id), date(7), |tx| {
            db::update_beat_prose(tx, &b.id, "new words")
        })
        .is_err());
        assert_eq!(scene_words(&conn, &s.id).unwrap(), 0);
        conn.execute_batch("DROP TRIGGER fail_writing").unwrap();
        save(&conn, &s, &b, 7, "new words");
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().today_words, 2);
    }

    #[test]
    fn streak_rolls_over_and_retains_historical_goals() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, _, s, b) = seed(&conn);
        set_goal(&conn, &p.id.to_string(), 2, date(5)).unwrap();
        save(&conn, &s, &b, 5, "one two");
        save(&conn, &s, &b, 6, "one two three four");
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().streak, 2);
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().today_words, 0);
        set_goal(&conn, &p.id.to_string(), 1, date(7)).unwrap();
        save(&conn, &s, &b, 7, "one two three four five");
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().streak, 3);
        assert_eq!(stats(&conn, &p.id, date(9)).unwrap().streak, 0);
        set_goal(&conn, &p.id.to_string(), 3, date(7)).unwrap();
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().streak, 2);
        set_goal(&conn, &p.id.to_string(), 0, date(7)).unwrap();
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().streak, 2);
        assert!(set_goal(&conn, &p.id.to_string(), -1, date(7)).is_err());
    }

    #[test]
    fn reset_and_reopen_preserve_daily_history_and_goal() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let conn = Connection::open(file.path()).unwrap();
        let (p, _, s, b) = seed(&conn);
        set_goal(&conn, &p.id.to_string(), 2, date(7)).unwrap();
        save(&conn, &s, &b, 7, "one two");
        reset_session(&conn, &p.id.to_string()).unwrap();
        save(&conn, &s, &b, 7, "one two three");
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().session_words, 1);
        drop(conn);
        let conn = Connection::open(file.path()).unwrap();
        db::initialize_schema(&conn).unwrap();
        db::initialize_schema(&conn).unwrap();
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!(
            (
                stat.today_words,
                stat.session_words,
                stat.daily_goal,
                stat.streak
            ),
            (3, 0, 2, 1)
        );
    }

    #[test]
    fn archived_content_is_excluded_without_erasing_history() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, c, s, b) = seed(&conn);
        save(&conn, &s, &b, 7, "one two");
        conn.execute(
            "UPDATE scenes SET archived = 1 WHERE id = ?1",
            [s.id.to_string()],
        )
        .unwrap();
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!((stat.project_words, stat.today_words), (0, 2));
        conn.execute(
            "UPDATE scenes SET archived = 0 WHERE id = ?1",
            [s.id.to_string()],
        )
        .unwrap();
        conn.execute(
            "UPDATE chapters SET archived = 1 WHERE id = ?1",
            [c.id.to_string()],
        )
        .unwrap();
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().project_words, 0);
        conn.execute("DELETE FROM projects WHERE id = ?1", [p.id.to_string()])
            .unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM writing_sessions", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn empty_beat_list_uses_scene_prose_and_projects_are_isolated() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, _, s, b) = seed(&conn);
        conn.execute("DELETE FROM beats WHERE id = ?1", [b.id.to_string()])
            .unwrap();
        save_prose(&conn, ProseTarget::Scene(s.id), date(7), |tx| {
            db::update_scene_prose(tx, &s.id, "scene words")
        })
        .unwrap();
        let (other, _, _, _) = seed(&conn);
        assert_eq!(stats(&conn, &p.id, date(7)).unwrap().project_words, 2);
        assert_eq!(stats(&conn, &other.id, date(7)).unwrap().today_words, 0);
        assert!(stats(&conn, &Uuid::new_v4(), date(7)).is_err());
    }
    #[test]
    fn recovered_drafts_credit_the_saved_document_after_a_mode_change() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, _, s, b) = seed(&conn);
        save(&conn, &s, &b, 7, "one two");
        db::switch_scene_editor_mode(&conn, &s.id, "page").unwrap();
        save(&conn, &s, &b, 7, "one two three four");
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!(
            (stat.project_words, stat.today_words, stat.session_words),
            (2, 4, 4)
        );
    }

    #[test]
    fn bulk_counts_include_empty_chapters_and_count_page_copies_only_once() {
        let conn = Connection::open_in_memory().unwrap();
        let (p, c, s, b) = seed(&conn);
        save(&conn, &s, &b, 7, "one two");
        let mut second = Beat::new(s.id, "second".into(), 1);
        second.prose = Some("three four".into());
        db::insert_beat(&conn, &second).unwrap();
        let empty = Chapter::new(p.id, "Empty".into(), 1);
        db::insert_chapter(&conn, &empty).unwrap();
        assert_eq!(
            stats(&conn, &p.id, date(7)).unwrap().chapter_words[&c.id.to_string()],
            4
        );
        db::switch_scene_editor_mode(&conn, &s.id, "page").unwrap();
        let stat = stats(&conn, &p.id, date(7)).unwrap();
        assert_eq!(stat.project_words, 4);
        assert_eq!(stat.chapter_words[&empty.id.to_string()], 0);
    }
}
