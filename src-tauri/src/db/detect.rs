use rusqlite::{params, Connection, Result};
use uuid::Uuid;

use super::queries::parse_uuid;

pub fn get_dismissed_suggestions(conn: &Connection, scene_id: &Uuid) -> Result<Vec<(Uuid, Uuid)>> {
    let mut stmt = conn.prepare(
        "SELECT scene_id, reference_id FROM dismissed_suggestions WHERE scene_id = ?1",
    )?;

    let pairs = stmt
        .query_map(params![scene_id.to_string()], |row| {
            Ok((
                parse_uuid(&row.get::<_, String>(0)?)?,
                parse_uuid(&row.get::<_, String>(1)?)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(pairs)
}

pub fn dismiss_suggestion(
    conn: &Connection,
    scene_id: &Uuid,
    reference_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO dismissed_suggestions (scene_id, reference_id) VALUES (?1, ?2)",
        params![scene_id.to_string(), reference_id.to_string()],
    )?;
    Ok(())
}

pub fn clear_dismissed_suggestions(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM dismissed_suggestions WHERE scene_id = ?1",
        params![scene_id.to_string()],
    )?;
    Ok(())
}
