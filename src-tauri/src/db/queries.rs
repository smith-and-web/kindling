use rusqlite::{params, Connection, Result};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{Beat, Chapter, Character, Location, Project, Scene, SourceType};

// ============================================================================
// Project Queries
// ============================================================================

pub fn insert_project(conn: &Connection, project: &Project) -> Result<()> {
    conn.execute(
        "INSERT INTO projects (id, name, source_type, source_path, created_at, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            project.id.to_string(),
            project.name,
            project.source_type.as_str(),
            project.source_path,
            project.created_at,
            project.modified_at,
        ],
    )?;
    Ok(())
}

pub fn get_project(conn: &Connection, id: &Uuid) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_type, source_path, created_at, modified_at
         FROM projects WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id.to_string()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Project {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            name: row.get(1)?,
            source_type: SourceType::parse(&row.get::<_, String>(2)?)
                .unwrap_or(SourceType::Markdown),
            source_path: row.get(3)?,
            created_at: row.get(4)?,
            modified_at: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn get_recent_projects(conn: &Connection, limit: usize) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_type, source_path, created_at, modified_at
         FROM projects ORDER BY modified_at DESC LIMIT ?1",
    )?;

    let projects = stmt
        .query_map(params![limit as i64], |row| {
            Ok(Project {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                name: row.get(1)?,
                source_type: SourceType::parse(&row.get::<_, String>(2)?)
                    .unwrap_or(SourceType::Markdown),
                source_path: row.get(3)?,
                created_at: row.get(4)?,
                modified_at: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(projects)
}

pub fn update_project_modified(conn: &Connection, id: &Uuid) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE projects SET modified_at = ?1 WHERE id = ?2",
        params![now, id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Chapter Queries
// ============================================================================

pub fn insert_chapter(conn: &Connection, chapter: &Chapter) -> Result<()> {
    conn.execute(
        "INSERT INTO chapters (id, project_id, title, position, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            chapter.id.to_string(),
            chapter.project_id.to_string(),
            chapter.title,
            chapter.position,
            chapter.source_id,
        ],
    )?;
    Ok(())
}

pub fn get_max_chapter_position(conn: &Connection, project_id: &Uuid) -> Result<i32> {
    let mut stmt =
        conn.prepare("SELECT COALESCE(MAX(position), -1) FROM chapters WHERE project_id = ?1")?;
    let max: i32 = stmt.query_row(params![project_id.to_string()], |row| row.get(0))?;
    Ok(max)
}

pub fn reorder_chapters(conn: &Connection, project_id: &Uuid, chapter_ids: &[Uuid]) -> Result<()> {
    conn.execute("BEGIN TRANSACTION", [])?;
    for (idx, id) in chapter_ids.iter().enumerate() {
        conn.execute(
            "UPDATE chapters SET position = ?1 WHERE id = ?2 AND project_id = ?3",
            params![idx as i32, id.to_string(), project_id.to_string()],
        )?;
    }
    conn.execute("COMMIT", [])?;
    Ok(())
}

pub fn get_chapters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id
         FROM chapters WHERE project_id = ?1 ORDER BY position",
    )?;

    let chapters = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Chapter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                position: row.get(3)?,
                source_id: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chapters)
}

// ============================================================================
// Scene Queries
// ============================================================================

pub fn insert_scene(conn: &Connection, scene: &Scene) -> Result<()> {
    conn.execute(
        "INSERT INTO scenes (id, chapter_id, title, synopsis, prose, position, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            scene.id.to_string(),
            scene.chapter_id.to_string(),
            scene.title,
            scene.synopsis,
            scene.prose,
            scene.position,
            scene.source_id,
        ],
    )?;
    Ok(())
}

pub fn get_max_scene_position(conn: &Connection, chapter_id: &Uuid) -> Result<i32> {
    let mut stmt =
        conn.prepare("SELECT COALESCE(MAX(position), -1) FROM scenes WHERE chapter_id = ?1")?;
    let max: i32 = stmt.query_row(params![chapter_id.to_string()], |row| row.get(0))?;
    Ok(max)
}

pub fn get_chapter_project_id(conn: &Connection, chapter_id: &Uuid) -> Result<Option<Uuid>> {
    let mut stmt = conn.prepare("SELECT project_id FROM chapters WHERE id = ?1")?;
    let mut rows = stmt.query(params![chapter_id.to_string()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()))
    } else {
        Ok(None)
    }
}

pub fn reorder_scenes(conn: &Connection, chapter_id: &Uuid, scene_ids: &[Uuid]) -> Result<()> {
    conn.execute("BEGIN TRANSACTION", [])?;
    for (idx, id) in scene_ids.iter().enumerate() {
        conn.execute(
            "UPDATE scenes SET position = ?1 WHERE id = ?2 AND chapter_id = ?3",
            params![idx as i32, id.to_string(), chapter_id.to_string()],
        )?;
    }
    conn.execute("COMMIT", [])?;
    Ok(())
}

pub fn move_scene_to_chapter(
    conn: &Connection,
    scene_id: &Uuid,
    target_chapter_id: &Uuid,
    position: i32,
) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET chapter_id = ?1, position = ?2 WHERE id = ?3",
        params![
            target_chapter_id.to_string(),
            position,
            scene_id.to_string()
        ],
    )?;
    Ok(())
}

pub fn get_scenes(conn: &Connection, chapter_id: &Uuid) -> Result<Vec<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT id, chapter_id, title, synopsis, prose, position, source_id
         FROM scenes WHERE chapter_id = ?1 ORDER BY position",
    )?;

    let scenes = stmt
        .query_map(params![chapter_id.to_string()], |row| {
            Ok(Scene {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                chapter_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                synopsis: row.get(3)?,
                prose: row.get(4)?,
                position: row.get(5)?,
                source_id: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(scenes)
}

pub fn update_scene_prose(conn: &Connection, scene_id: &Uuid, prose: &str) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET prose = ?1 WHERE id = ?2",
        params![prose, scene_id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Beat Queries
// ============================================================================

pub fn insert_beat(conn: &Connection, beat: &Beat) -> Result<()> {
    conn.execute(
        "INSERT INTO beats (id, scene_id, content, prose, position, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            beat.id.to_string(),
            beat.scene_id.to_string(),
            beat.content,
            beat.prose,
            beat.position,
            beat.source_id,
        ],
    )?;
    Ok(())
}

pub fn get_beats(conn: &Connection, scene_id: &Uuid) -> Result<Vec<Beat>> {
    let mut stmt = conn.prepare(
        "SELECT id, scene_id, content, prose, position, source_id
         FROM beats WHERE scene_id = ?1 ORDER BY position",
    )?;

    let beats = stmt
        .query_map(params![scene_id.to_string()], |row| {
            Ok(Beat {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                scene_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                content: row.get(2)?,
                prose: row.get(3)?,
                position: row.get(4)?,
                source_id: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(beats)
}

pub fn update_beat_prose(conn: &Connection, beat_id: &Uuid, prose: &str) -> Result<()> {
    conn.execute(
        "UPDATE beats SET prose = ?1 WHERE id = ?2",
        params![prose, beat_id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Character Queries
// ============================================================================

pub fn insert_character(conn: &Connection, character: &Character) -> Result<()> {
    conn.execute(
        "INSERT INTO characters (id, project_id, name, description, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            character.id.to_string(),
            character.project_id.to_string(),
            character.name,
            character.description,
            character.source_id,
        ],
    )?;

    // Insert attributes
    for (key, value) in &character.attributes {
        conn.execute(
            "INSERT INTO character_attributes (character_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![character.id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn get_characters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Character>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, source_id
         FROM characters WHERE project_id = ?1 ORDER BY name",
    )?;

    let mut characters: Vec<Character> = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Character {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                name: row.get(2)?,
                description: row.get(3)?,
                attributes: HashMap::new(),
                source_id: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Load attributes for each character
    for character in &mut characters {
        let mut attr_stmt =
            conn.prepare("SELECT key, value FROM character_attributes WHERE character_id = ?1")?;

        let attrs: Vec<(String, String)> = attr_stmt
            .query_map(params![character.id.to_string()], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        character.attributes = attrs.into_iter().collect();
    }

    Ok(characters)
}

// ============================================================================
// Location Queries
// ============================================================================

pub fn insert_location(conn: &Connection, location: &Location) -> Result<()> {
    conn.execute(
        "INSERT INTO locations (id, project_id, name, description, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            location.id.to_string(),
            location.project_id.to_string(),
            location.name,
            location.description,
            location.source_id,
        ],
    )?;

    // Insert attributes
    for (key, value) in &location.attributes {
        conn.execute(
            "INSERT INTO location_attributes (location_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![location.id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn get_locations(conn: &Connection, project_id: &Uuid) -> Result<Vec<Location>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, source_id
         FROM locations WHERE project_id = ?1 ORDER BY name",
    )?;

    let mut locations: Vec<Location> = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Location {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                name: row.get(2)?,
                description: row.get(3)?,
                attributes: HashMap::new(),
                source_id: row.get(4)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    // Load attributes for each location
    for location in &mut locations {
        let mut attr_stmt =
            conn.prepare("SELECT key, value FROM location_attributes WHERE location_id = ?1")?;

        let attrs: Vec<(String, String)> = attr_stmt
            .query_map(params![location.id.to_string()], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        location.attributes = attrs.into_iter().collect();
    }

    Ok(locations)
}

// ============================================================================
// Scene References
// ============================================================================

pub fn add_scene_character_ref(
    conn: &Connection,
    scene_id: &Uuid,
    character_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO scene_character_refs (scene_id, character_id)
         VALUES (?1, ?2)",
        params![scene_id.to_string(), character_id.to_string()],
    )?;
    Ok(())
}

pub fn add_scene_location_ref(
    conn: &Connection,
    scene_id: &Uuid,
    location_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO scene_location_refs (scene_id, location_id)
         VALUES (?1, ?2)",
        params![scene_id.to_string(), location_id.to_string()],
    )?;
    Ok(())
}

pub fn get_scene_characters(conn: &Connection, scene_id: &Uuid) -> Result<Vec<Uuid>> {
    let mut stmt =
        conn.prepare("SELECT character_id FROM scene_character_refs WHERE scene_id = ?1")?;

    let ids = stmt
        .query_map(params![scene_id.to_string()], |row| {
            Ok(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ids)
}

pub fn get_scene_locations(conn: &Connection, scene_id: &Uuid) -> Result<Vec<Uuid>> {
    let mut stmt =
        conn.prepare("SELECT location_id FROM scene_location_refs WHERE scene_id = ?1")?;

    let ids = stmt
        .query_map(params![scene_id.to_string()], |row| {
            Ok(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap())
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ids)
}

// ============================================================================
// Delete Operations
// ============================================================================

/// Get counts of scenes and beats in a chapter (for confirmation dialog)
pub fn get_chapter_content_counts(conn: &Connection, chapter_id: &Uuid) -> Result<(i32, i32)> {
    let scene_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM scenes WHERE chapter_id = ?1",
        params![chapter_id.to_string()],
        |row| row.get(0),
    )?;

    let beat_count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM beats WHERE scene_id IN (SELECT id FROM scenes WHERE chapter_id = ?1)",
        params![chapter_id.to_string()],
        |row| row.get(0),
    )?;

    Ok((scene_count, beat_count))
}

/// Get beat count for a scene (for confirmation dialog)
pub fn get_scene_beat_count(conn: &Connection, scene_id: &Uuid) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM beats WHERE scene_id = ?1",
        params![scene_id.to_string()],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Delete a chapter and all its scenes, beats, and references
pub fn delete_chapter(conn: &Connection, chapter_id: &Uuid) -> Result<()> {
    conn.execute("BEGIN TRANSACTION", [])?;

    // Delete scene references for all scenes in this chapter
    conn.execute(
        "DELETE FROM scene_character_refs WHERE scene_id IN (SELECT id FROM scenes WHERE chapter_id = ?1)",
        params![chapter_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM scene_location_refs WHERE scene_id IN (SELECT id FROM scenes WHERE chapter_id = ?1)",
        params![chapter_id.to_string()],
    )?;

    // Delete beats for all scenes in this chapter
    conn.execute(
        "DELETE FROM beats WHERE scene_id IN (SELECT id FROM scenes WHERE chapter_id = ?1)",
        params![chapter_id.to_string()],
    )?;

    // Delete all scenes in this chapter
    conn.execute(
        "DELETE FROM scenes WHERE chapter_id = ?1",
        params![chapter_id.to_string()],
    )?;

    // Delete the chapter itself
    conn.execute(
        "DELETE FROM chapters WHERE id = ?1",
        params![chapter_id.to_string()],
    )?;

    conn.execute("COMMIT", [])?;
    Ok(())
}

/// Delete a scene and all its beats and references
pub fn delete_scene(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute("BEGIN TRANSACTION", [])?;

    // Delete scene references
    conn.execute(
        "DELETE FROM scene_character_refs WHERE scene_id = ?1",
        params![scene_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM scene_location_refs WHERE scene_id = ?1",
        params![scene_id.to_string()],
    )?;

    // Delete beats
    conn.execute(
        "DELETE FROM beats WHERE scene_id = ?1",
        params![scene_id.to_string()],
    )?;

    // Delete the scene
    conn.execute(
        "DELETE FROM scenes WHERE id = ?1",
        params![scene_id.to_string()],
    )?;

    conn.execute("COMMIT", [])?;
    Ok(())
}

// ============================================================================
// Re-import / Merge Queries
// ============================================================================

/// Find a chapter by source_id (for reimport matching)
pub fn find_chapter_by_source_id(
    conn: &Connection,
    project_id: &Uuid,
    source_id: &str,
) -> Result<Option<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id
         FROM chapters WHERE project_id = ?1 AND source_id = ?2",
    )?;

    let mut rows = stmt.query(params![project_id.to_string(), source_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Chapter {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            title: row.get(2)?,
            position: row.get(3)?,
            source_id: row.get(4)?,
        }))
    } else {
        Ok(None)
    }
}

/// Find a scene by source_id (for reimport matching)
pub fn find_scene_by_source_id(
    conn: &Connection,
    chapter_id: &Uuid,
    source_id: &str,
) -> Result<Option<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT id, chapter_id, title, synopsis, prose, position, source_id
         FROM scenes WHERE chapter_id = ?1 AND source_id = ?2",
    )?;

    let mut rows = stmt.query(params![chapter_id.to_string(), source_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Scene {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            chapter_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            title: row.get(2)?,
            synopsis: row.get(3)?,
            prose: row.get(4)?,
            position: row.get(5)?,
            source_id: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

/// Find a beat by source_id (for reimport matching)
pub fn find_beat_by_source_id(
    conn: &Connection,
    scene_id: &Uuid,
    source_id: &str,
) -> Result<Option<Beat>> {
    let mut stmt = conn.prepare(
        "SELECT id, scene_id, content, prose, position, source_id
         FROM beats WHERE scene_id = ?1 AND source_id = ?2",
    )?;

    let mut rows = stmt.query(params![scene_id.to_string(), source_id])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Beat {
            id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
            scene_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            content: row.get(2)?,
            prose: row.get(3)?,
            position: row.get(4)?,
            source_id: row.get(5)?,
        }))
    } else {
        Ok(None)
    }
}

/// Update a chapter's title and position (for reimport merge)
pub fn update_chapter(
    conn: &Connection,
    chapter_id: &Uuid,
    title: &str,
    position: i32,
) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET title = ?1, position = ?2 WHERE id = ?3",
        params![title, position, chapter_id.to_string()],
    )?;
    Ok(())
}

/// Update a scene's title, synopsis, and position (preserves prose)
pub fn update_scene(
    conn: &Connection,
    scene_id: &Uuid,
    title: &str,
    synopsis: Option<&str>,
    position: i32,
) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET title = ?1, synopsis = ?2, position = ?3 WHERE id = ?4",
        params![title, synopsis, position, scene_id.to_string()],
    )?;
    Ok(())
}

/// Update a beat's content and position (preserves prose)
pub fn update_beat(conn: &Connection, beat_id: &Uuid, content: &str, position: i32) -> Result<()> {
    conn.execute(
        "UPDATE beats SET content = ?1, position = ?2 WHERE id = ?3",
        params![content, position, beat_id.to_string()],
    )?;
    Ok(())
}

/// Get all chapters for a project (for reimport)
pub fn get_all_chapters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Chapter>> {
    get_chapters(conn, project_id)
}

/// Get all scenes for a project across all chapters (for reimport stats)
pub fn get_all_project_scenes(conn: &Connection, project_id: &Uuid) -> Result<Vec<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.chapter_id, s.title, s.synopsis, s.prose, s.position, s.source_id
         FROM scenes s
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1
         ORDER BY c.position, s.position",
    )?;

    let scenes = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Scene {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                chapter_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                synopsis: row.get(3)?,
                prose: row.get(4)?,
                position: row.get(5)?,
                source_id: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(scenes)
}

/// Get all beats for a project across all scenes (for reimport stats)
pub fn get_all_project_beats(conn: &Connection, project_id: &Uuid) -> Result<Vec<Beat>> {
    let mut stmt = conn.prepare(
        "SELECT b.id, b.scene_id, b.content, b.prose, b.position, b.source_id
         FROM beats b
         JOIN scenes s ON b.scene_id = s.id
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1
         ORDER BY c.position, s.position, b.position",
    )?;

    let beats = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Beat {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                scene_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                content: row.get(2)?,
                prose: row.get(3)?,
                position: row.get(4)?,
                source_id: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(beats)
}
