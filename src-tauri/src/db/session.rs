use rusqlite::{params, Connection, OptionalExtension, Result};
use uuid::Uuid;

use crate::models::SessionState;

use super::queries::parse_uuid;

pub fn get_session_state(conn: &Connection, project_id: &Uuid) -> Result<Option<SessionState>> {
    // Deleted, archived, or moved-to-another-project scenes cannot be resumed.
    // A deleted beat only clears the editor position; the scene still opens.
    conn.query_row(
        "SELECT ss.project_id, s.id, c.id, b.id,
                CASE WHEN ss.current_beat_id IS NULL OR b.id IS NOT NULL
                     THEN ss.cursor_position END,
                ss.scroll_position,
                CASE WHEN ss.current_beat_id IS NULL OR b.id IS NOT NULL
                     THEN ss.editor_scroll_position END,
                ss.last_opened_at
         FROM session_state ss
         JOIN scenes s ON s.id = ss.current_scene_id AND s.archived = 0
         JOIN chapters c ON c.id = s.chapter_id AND c.project_id = ss.project_id AND c.archived = 0
         LEFT JOIN beats b ON b.id = ss.current_beat_id AND b.scene_id = s.id
         WHERE ss.project_id = ?1",
        [project_id.to_string()],
        |row| {
            Ok(SessionState {
                project_id: parse_uuid(&row.get::<_, String>(0)?)?,
                current_scene_id: Some(parse_uuid(&row.get::<_, String>(1)?)?),
                current_chapter_id: Some(parse_uuid(&row.get::<_, String>(2)?)?),
                current_beat_id: row
                    .get::<_, Option<String>>(3)?
                    .map(|id| parse_uuid(&id))
                    .transpose()?,
                cursor_position: row.get(4)?,
                scroll_position: row.get(5)?,
                editor_scroll_position: row.get(6)?,
                last_opened_at: row.get(7)?,
            })
        },
    )
    .optional()
}

pub fn save_session_state(conn: &Connection, session: &SessionState) -> Result<()> {
    // INSERT ... SELECT also ignores late saves after a scene/project was deleted.
    conn.execute(
        "INSERT INTO session_state
            (project_id, current_scene_id, current_beat_id, cursor_position,
             scroll_position, editor_scroll_position, last_opened_at)
         SELECT ?1, s.id, b.id, ?4, ?5, ?6, ?7
         FROM scenes s
         JOIN chapters c ON c.id = s.chapter_id AND c.project_id = ?1
         LEFT JOIN beats b ON b.id = ?3 AND b.scene_id = s.id
         WHERE s.id = ?2
         ON CONFLICT(project_id) DO UPDATE SET
            current_scene_id = excluded.current_scene_id,
            current_beat_id = excluded.current_beat_id,
            cursor_position = excluded.cursor_position,
            scroll_position = excluded.scroll_position,
            editor_scroll_position = excluded.editor_scroll_position,
            last_opened_at = excluded.last_opened_at",
        params![
            session.project_id.to_string(),
            session.current_scene_id.map(|id| id.to_string()),
            session.current_beat_id.map(|id| id.to_string()),
            session.cursor_position,
            session
                .scroll_position
                .filter(|n| n.is_finite())
                .map(|n| n.max(0.0)),
            session
                .editor_scroll_position
                .filter(|n| n.is_finite())
                .map(|n| n.max(0.0)),
            chrono::Utc::now().to_rfc3339(),
        ],
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

    fn seed(conn: &Connection) -> SessionState {
        let project = Project::new("Story".into(), SourceType::Blank, None);
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        let scene = Scene::new(chapter.id, "Scene".into(), None, 0);
        let beat = Beat::new(scene.id, "Beat".into(), 0);
        db::insert_project(conn, &project).unwrap();
        db::insert_chapter(conn, &chapter).unwrap();
        db::insert_scene(conn, &scene).unwrap();
        db::insert_beat(conn, &beat).unwrap();
        SessionState {
            project_id: project.id,
            current_chapter_id: Some(chapter.id),
            current_scene_id: Some(scene.id),
            current_beat_id: Some(beat.id),
            cursor_position: Some(17),
            scroll_position: Some(432.5),
            editor_scroll_position: Some(91.25),
            last_opened_at: None,
        }
    }

    #[test]
    fn sessions_survive_restart_and_are_isolated_per_project() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        db::initialize_schema(&conn).unwrap();
        let mut first = seed(&conn);
        let second = seed(&conn);
        assert!(get_session_state(&conn, &first.project_id)
            .unwrap()
            .is_none());
        save_session_state(&conn, &first).unwrap();
        save_session_state(&conn, &second).unwrap();
        first.cursor_position = Some(105);
        first.scroll_position = Some(999.75);
        save_session_state(&conn, &first).unwrap();
        drop(conn);

        let conn = Connection::open(&path).unwrap();
        db::initialize_schema(&conn).unwrap();
        for original in [first, second] {
            let loaded = get_session_state(&conn, &original.project_id)
                .unwrap()
                .unwrap();
            assert!(loaded.last_opened_at.is_some());
            assert_eq!(
                loaded,
                SessionState {
                    last_opened_at: loaded.last_opened_at.clone(),
                    ..original
                }
            );
        }
    }

    #[test]
    fn migration_preserves_legacy_sessions_and_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let session = seed(&conn);
        conn.execute_batch(
            "DROP TABLE session_state;
            CREATE TABLE session_state (
                project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
                current_scene_id TEXT, cursor_position INTEGER,
                scroll_position REAL, last_opened_at TEXT);",
        )
        .unwrap();
        conn.execute(
            "INSERT INTO session_state VALUES (?1, ?2, 9, 12.5, 'yesterday')",
            params![
                session.project_id.to_string(),
                session.current_scene_id.unwrap().to_string()
            ],
        )
        .unwrap();
        db::initialize_schema(&conn).unwrap();
        db::initialize_schema(&conn).unwrap();
        let loaded = get_session_state(&conn, &session.project_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.current_beat_id, None);
        assert_eq!(loaded.editor_scroll_position, None);
        assert_eq!(loaded.cursor_position, Some(9));
        assert_eq!(loaded.scroll_position, Some(12.5));
    }

    #[test]
    fn deleted_beats_clear_cursor_but_keep_the_scene_and_archived_scenes_do_not_resume() {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let session = seed(&conn);
        save_session_state(&conn, &session).unwrap();
        conn.execute(
            "DELETE FROM beats WHERE id = ?1",
            [session.current_beat_id.unwrap().to_string()],
        )
        .unwrap();
        let loaded = get_session_state(&conn, &session.project_id)
            .unwrap()
            .unwrap();
        assert_eq!(loaded.current_scene_id, session.current_scene_id);
        assert_eq!(loaded.current_beat_id, None);
        assert_eq!(loaded.cursor_position, None);
        assert_eq!(loaded.editor_scroll_position, None);
        conn.execute(
            "UPDATE scenes SET archived = 1 WHERE id = ?1",
            [session.current_scene_id.unwrap().to_string()],
        )
        .unwrap();
        assert!(get_session_state(&conn, &session.project_id)
            .unwrap()
            .is_none());
        conn.execute(
            "DELETE FROM scenes WHERE id = ?1",
            [session.current_scene_id.unwrap().to_string()],
        )
        .unwrap();
        save_session_state(&conn, &session).unwrap();
        assert!(get_session_state(&conn, &session.project_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn resolves_moved_scenes_and_ignores_cross_project_positions_and_late_deleted_project_saves() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON").unwrap();
        db::initialize_schema(&conn).unwrap();
        let session = seed(&conn);
        let other = seed(&conn);
        let chapter = Chapter::new(session.project_id, "New chapter".into(), 1);
        db::insert_chapter(&conn, &chapter).unwrap();
        save_session_state(&conn, &session).unwrap();
        conn.execute(
            "UPDATE scenes SET chapter_id = ?1 WHERE id = ?2",
            params![
                chapter.id.to_string(),
                session.current_scene_id.unwrap().to_string()
            ],
        )
        .unwrap();
        assert_eq!(
            get_session_state(&conn, &session.project_id)
                .unwrap()
                .unwrap()
                .current_chapter_id,
            Some(chapter.id)
        );
        let wrong_scene = SessionState {
            current_scene_id: other.current_scene_id,
            ..session.clone()
        };
        save_session_state(&conn, &wrong_scene).unwrap();
        assert_eq!(
            get_session_state(&conn, &session.project_id)
                .unwrap()
                .unwrap()
                .current_scene_id,
            session.current_scene_id
        );
        conn.execute(
            "UPDATE chapters SET archived = 1 WHERE id = ?1",
            [chapter.id.to_string()],
        )
        .unwrap();
        assert!(get_session_state(&conn, &session.project_id)
            .unwrap()
            .is_none());
        db::delete_project(&conn, &session.project_id).unwrap();
        save_session_state(&conn, &session).unwrap();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM session_state", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}
