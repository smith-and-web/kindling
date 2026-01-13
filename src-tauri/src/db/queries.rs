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
        "INSERT INTO chapters (id, project_id, title, position)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            chapter.id.to_string(),
            chapter.project_id.to_string(),
            chapter.title,
            chapter.position,
        ],
    )?;
    Ok(())
}

pub fn get_chapters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position
         FROM chapters WHERE project_id = ?1 ORDER BY position",
    )?;

    let chapters = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Chapter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                position: row.get(3)?,
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
        "INSERT INTO scenes (id, chapter_id, title, synopsis, prose, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            scene.id.to_string(),
            scene.chapter_id.to_string(),
            scene.title,
            scene.synopsis,
            scene.prose,
            scene.position,
        ],
    )?;
    Ok(())
}

pub fn get_scenes(conn: &Connection, chapter_id: &Uuid) -> Result<Vec<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT id, chapter_id, title, synopsis, prose, position
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
        "INSERT INTO beats (id, scene_id, content, prose, position)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            beat.id.to_string(),
            beat.scene_id.to_string(),
            beat.content,
            beat.prose,
            beat.position,
        ],
    )?;
    Ok(())
}

pub fn get_beats(conn: &Connection, scene_id: &Uuid) -> Result<Vec<Beat>> {
    let mut stmt = conn.prepare(
        "SELECT id, scene_id, content, prose, position
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
