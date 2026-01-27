use rusqlite::{params, Connection, OptionalExtension, Result};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{
    Beat, Chapter, Character, Location, Project, ReferenceItem, Scene, SceneCharacterRef,
    SceneLocationRef, SceneStatus, SceneType, SnapshotMetadata, SnapshotTrigger, SourceType,
};

// ============================================================================
// Project Queries
// ============================================================================

pub fn insert_project(conn: &Connection, project: &Project) -> Result<()> {
    let reference_types_json =
        serde_json::to_string(&project.reference_types).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO projects (id, name, source_type, source_path, created_at, modified_at, author_pen_name, genre, description, word_target, reference_types)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            project.id.to_string(),
            project.name,
            project.source_type.as_str(),
            project.source_path,
            project.created_at,
            project.modified_at,
            project.author_pen_name,
            project.genre,
            project.description,
            project.word_target,
            reference_types_json,
        ],
    )?;
    Ok(())
}

fn parse_reference_types(raw: Option<String>) -> Vec<String> {
    match raw {
        Some(value) => serde_json::from_str::<Vec<String>>(&value)
            .unwrap_or_else(|_| Project::default_reference_types()),
        None => Project::default_reference_types(),
    }
}

pub fn get_project(conn: &Connection, id: &Uuid) -> Result<Option<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_type, source_path, created_at, modified_at, author_pen_name, genre, description, word_target, reference_types
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
            author_pen_name: row.get(6)?,
            genre: row.get(7)?,
            description: row.get(8)?,
            word_target: row.get(9)?,
            reference_types: parse_reference_types(row.get(10)?),
        }))
    } else {
        Ok(None)
    }
}

pub fn get_recent_projects(conn: &Connection, limit: usize) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, source_type, source_path, created_at, modified_at, author_pen_name, genre, description, word_target, reference_types
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
                author_pen_name: row.get(6)?,
                genre: row.get(7)?,
                description: row.get(8)?,
                word_target: row.get(9)?,
                reference_types: parse_reference_types(row.get(10)?),
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

/// Delete a project and all its data (cascades via foreign keys)
pub fn delete_project(conn: &Connection, id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM projects WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Chapter Queries
// ============================================================================

pub fn insert_chapter(conn: &Connection, chapter: &Chapter) -> Result<()> {
    conn.execute(
        "INSERT INTO chapters (id, project_id, title, position, source_id, archived, locked, is_part)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            chapter.id.to_string(),
            chapter.project_id.to_string(),
            chapter.title,
            chapter.position,
            chapter.source_id,
            chapter.archived as i32,
            chapter.locked as i32,
            chapter.is_part as i32,
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

/// Shift all chapters at or after the given position up by 1 to make room for insertion
pub fn shift_chapters_after_position(
    conn: &Connection,
    project_id: &Uuid,
    position: i32,
) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET position = position + 1 WHERE project_id = ?1 AND position >= ?2",
        params![project_id.to_string(), position],
    )?;
    Ok(())
}

pub fn get_chapters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id, archived, locked, is_part
         FROM chapters WHERE project_id = ?1 AND archived = 0 ORDER BY position",
    )?;

    let chapters = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Chapter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                position: row.get(3)?,
                source_id: row.get(4)?,
                archived: row.get::<_, i32>(5)? != 0,
                locked: row.get::<_, i32>(6)? != 0,
                is_part: row.get::<_, i32>(7).unwrap_or(0) != 0,
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
        "INSERT INTO scenes (id, chapter_id, title, synopsis, prose, position, source_id, archived, locked, scene_type, scene_status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            scene.id.to_string(),
            scene.chapter_id.to_string(),
            scene.title,
            scene.synopsis,
            scene.prose,
            scene.position,
            scene.source_id,
            scene.archived as i32,
            scene.locked as i32,
            scene.scene_type.as_str(),
            scene.scene_status.as_str(),
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

pub fn get_scene_project_id(conn: &Connection, scene_id: &Uuid) -> Result<Option<Uuid>> {
    let mut stmt = conn.prepare(
        "SELECT c.project_id FROM chapters c
         JOIN scenes s ON s.chapter_id = c.id
         WHERE s.id = ?1",
    )?;
    let mut rows = stmt.query(params![scene_id.to_string()])?;

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
        "SELECT id, chapter_id, title, synopsis, prose, position, source_id, archived, locked, scene_type, scene_status
         FROM scenes WHERE chapter_id = ?1 AND archived = 0 ORDER BY position",
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
                archived: row.get::<_, i32>(7)? != 0,
                locked: row.get::<_, i32>(8)? != 0,
                scene_type: SceneType::parse(&row.get::<_, String>(9)?),
                scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
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

pub fn get_max_beat_position(conn: &Connection, scene_id: &Uuid) -> Result<i32> {
    let mut stmt =
        conn.prepare("SELECT COALESCE(MAX(position), -1) FROM beats WHERE scene_id = ?1")?;
    let max: i32 = stmt.query_row(params![scene_id.to_string()], |row| row.get(0))?;
    Ok(max)
}

pub fn update_scene_synopsis(
    conn: &Connection,
    scene_id: &Uuid,
    synopsis: Option<&str>,
) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET synopsis = ?1 WHERE id = ?2",
        params![synopsis, scene_id.to_string()],
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

pub fn get_character_project_id(conn: &Connection, character_id: &Uuid) -> Result<Option<Uuid>> {
    let mut stmt = conn.prepare("SELECT project_id FROM characters WHERE id = ?1")?;
    let mut rows = stmt.query(params![character_id.to_string()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()))
    } else {
        Ok(None)
    }
}

pub fn update_character(
    conn: &Connection,
    character_id: &Uuid,
    name: &str,
    description: Option<&str>,
    attributes: &HashMap<String, String>,
) -> Result<()> {
    conn.execute(
        "UPDATE characters SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, character_id.to_string()],
    )?;

    conn.execute(
        "DELETE FROM character_attributes WHERE character_id = ?1",
        params![character_id.to_string()],
    )?;

    for (key, value) in attributes {
        conn.execute(
            "INSERT INTO character_attributes (character_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![character_id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn delete_character(conn: &Connection, character_id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM characters WHERE id = ?1",
        params![character_id.to_string()],
    )?;
    Ok(())
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

pub fn get_location_project_id(conn: &Connection, location_id: &Uuid) -> Result<Option<Uuid>> {
    let mut stmt = conn.prepare("SELECT project_id FROM locations WHERE id = ?1")?;
    let mut rows = stmt.query(params![location_id.to_string()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()))
    } else {
        Ok(None)
    }
}

pub fn update_location(
    conn: &Connection,
    location_id: &Uuid,
    name: &str,
    description: Option<&str>,
    attributes: &HashMap<String, String>,
) -> Result<()> {
    conn.execute(
        "UPDATE locations SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, location_id.to_string()],
    )?;

    conn.execute(
        "DELETE FROM location_attributes WHERE location_id = ?1",
        params![location_id.to_string()],
    )?;

    for (key, value) in attributes {
        conn.execute(
            "INSERT INTO location_attributes (location_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![location_id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn delete_location(conn: &Connection, location_id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM locations WHERE id = ?1",
        params![location_id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Reference Item Queries
// ============================================================================

pub fn insert_reference_item(conn: &Connection, item: &ReferenceItem) -> Result<()> {
    conn.execute(
        "INSERT INTO reference_items (id, project_id, reference_type, name, description, source_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            item.id.to_string(),
            item.project_id.to_string(),
            item.reference_type,
            item.name,
            item.description,
            item.source_id,
        ],
    )?;

    for (key, value) in &item.attributes {
        conn.execute(
            "INSERT INTO reference_item_attributes (reference_item_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![item.id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn get_reference_items(
    conn: &Connection,
    project_id: &Uuid,
    reference_type: &str,
) -> Result<Vec<ReferenceItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, reference_type, name, description, source_id
         FROM reference_items WHERE project_id = ?1 AND reference_type = ?2 ORDER BY name",
    )?;

    let mut items: Vec<ReferenceItem> = stmt
        .query_map(params![project_id.to_string(), reference_type], |row| {
            Ok(ReferenceItem {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                reference_type: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                attributes: HashMap::new(),
                source_id: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    for item in &mut items {
        let mut attr_stmt = conn.prepare(
            "SELECT key, value FROM reference_item_attributes WHERE reference_item_id = ?1",
        )?;

        let attrs: Vec<(String, String)> = attr_stmt
            .query_map(params![item.id.to_string()], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        item.attributes = attrs.into_iter().collect();
    }

    Ok(items)
}

pub fn get_all_reference_items(conn: &Connection, project_id: &Uuid) -> Result<Vec<ReferenceItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, reference_type, name, description, source_id
         FROM reference_items WHERE project_id = ?1 ORDER BY reference_type, name",
    )?;

    let mut items: Vec<ReferenceItem> = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(ReferenceItem {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                reference_type: row.get(2)?,
                name: row.get(3)?,
                description: row.get(4)?,
                attributes: HashMap::new(),
                source_id: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    for item in &mut items {
        let mut attr_stmt = conn.prepare(
            "SELECT key, value FROM reference_item_attributes WHERE reference_item_id = ?1",
        )?;

        let attrs: Vec<(String, String)> = attr_stmt
            .query_map(params![item.id.to_string()], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        item.attributes = attrs.into_iter().collect();
    }

    Ok(items)
}

pub fn get_reference_item_project_id(
    conn: &Connection,
    reference_item_id: &Uuid,
) -> Result<Option<Uuid>> {
    let mut stmt = conn.prepare("SELECT project_id FROM reference_items WHERE id = ?1")?;
    let mut rows = stmt.query(params![reference_item_id.to_string()])?;

    if let Some(row) = rows.next()? {
        Ok(Some(Uuid::parse_str(&row.get::<_, String>(0)?).unwrap()))
    } else {
        Ok(None)
    }
}

pub fn update_reference_item(
    conn: &Connection,
    reference_item_id: &Uuid,
    name: &str,
    description: Option<&str>,
    attributes: &HashMap<String, String>,
) -> Result<()> {
    conn.execute(
        "UPDATE reference_items SET name = ?1, description = ?2 WHERE id = ?3",
        params![name, description, reference_item_id.to_string()],
    )?;

    conn.execute(
        "DELETE FROM reference_item_attributes WHERE reference_item_id = ?1",
        params![reference_item_id.to_string()],
    )?;

    for (key, value) in attributes {
        conn.execute(
            "INSERT INTO reference_item_attributes (reference_item_id, key, value)
             VALUES (?1, ?2, ?3)",
            params![reference_item_id.to_string(), key, value],
        )?;
    }

    Ok(())
}

pub fn delete_reference_item(conn: &Connection, reference_item_id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM reference_items WHERE id = ?1",
        params![reference_item_id.to_string()],
    )?;
    Ok(())
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
        "SELECT id, project_id, title, position, source_id, archived, locked, is_part
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
            archived: row.get::<_, i32>(5)? != 0,
            locked: row.get::<_, i32>(6)? != 0,
            is_part: row.get::<_, i32>(7).unwrap_or(0) != 0,
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
        "SELECT id, chapter_id, title, synopsis, prose, position, source_id, archived, locked, scene_type, scene_status
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
            archived: row.get::<_, i32>(7)? != 0,
            locked: row.get::<_, i32>(8)? != 0,
            scene_type: SceneType::parse(&row.get::<_, String>(9)?),
            scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
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

/// Update a scene's title, synopsis, position, and metadata (preserves prose)
pub fn update_scene(
    conn: &Connection,
    scene_id: &Uuid,
    title: &str,
    synopsis: Option<&str>,
    position: i32,
    scene_type: &SceneType,
    scene_status: &SceneStatus,
) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET title = ?1, synopsis = ?2, position = ?3, scene_type = ?4, scene_status = ?5 WHERE id = ?6",
        params![
            title,
            synopsis,
            position,
            scene_type.as_str(),
            scene_status.as_str(),
            scene_id.to_string()
        ],
    )?;
    Ok(())
}

pub fn update_scene_metadata(
    conn: &Connection,
    scene_id: &Uuid,
    scene_type: &SceneType,
    scene_status: &SceneStatus,
) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET scene_type = ?1, scene_status = ?2 WHERE id = ?3",
        params![
            scene_type.as_str(),
            scene_status.as_str(),
            scene_id.to_string()
        ],
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
        "SELECT s.id, s.chapter_id, s.title, s.synopsis, s.prose, s.position, s.source_id, s.archived, s.locked, s.scene_type, s.scene_status
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
                archived: row.get::<_, i32>(7)? != 0,
                locked: row.get::<_, i32>(8)? != 0,
                scene_type: SceneType::parse(&row.get::<_, String>(9)?),
                scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
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

// ============================================================================
// Archive Operations
// ============================================================================

pub fn archive_chapter(conn: &Connection, chapter_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET archived = 1 WHERE id = ?1",
        params![chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn restore_chapter(conn: &Connection, chapter_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET archived = 0 WHERE id = ?1",
        params![chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn archive_scene(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET archived = 1 WHERE id = ?1",
        params![scene_id.to_string()],
    )?;
    Ok(())
}

pub fn restore_scene(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET archived = 0 WHERE id = ?1",
        params![scene_id.to_string()],
    )?;
    Ok(())
}

pub fn get_archived_chapters(conn: &Connection, project_id: &Uuid) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id, archived, locked, is_part
         FROM chapters WHERE project_id = ?1 AND archived = 1 ORDER BY position",
    )?;

    let chapters = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Chapter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                position: row.get(3)?,
                source_id: row.get(4)?,
                archived: row.get::<_, i32>(5)? != 0,
                locked: row.get::<_, i32>(6)? != 0,
                is_part: row.get::<_, i32>(7).unwrap_or(0) != 0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chapters)
}

pub fn get_archived_scenes(conn: &Connection, project_id: &Uuid) -> Result<Vec<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.chapter_id, s.title, s.synopsis, s.prose, s.position, s.source_id, s.archived, s.locked, s.scene_type, s.scene_status
         FROM scenes s
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1 AND s.archived = 1
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
                archived: row.get::<_, i32>(7)? != 0,
                locked: row.get::<_, i32>(8)? != 0,
                scene_type: SceneType::parse(&row.get::<_, String>(9)?),
                scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(scenes)
}

// ============================================================================
// Lock Operations
// ============================================================================

pub fn lock_chapter(conn: &Connection, chapter_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET locked = 1 WHERE id = ?1",
        params![chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn unlock_chapter(conn: &Connection, chapter_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET locked = 0 WHERE id = ?1",
        params![chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn set_chapter_is_part(conn: &Connection, chapter_id: &Uuid, is_part: bool) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET is_part = ?1 WHERE id = ?2",
        params![is_part as i32, chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn lock_scene(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET locked = 1 WHERE id = ?1",
        params![scene_id.to_string()],
    )?;
    Ok(())
}

pub fn unlock_scene(conn: &Connection, scene_id: &Uuid) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET locked = 0 WHERE id = ?1",
        params![scene_id.to_string()],
    )?;
    Ok(())
}

pub fn is_scene_locked(conn: &Connection, scene_id: &Uuid) -> Result<bool> {
    let mut stmt = conn.prepare(
        "SELECT s.locked, c.locked FROM scenes s
         JOIN chapters c ON s.chapter_id = c.id
         WHERE s.id = ?1",
    )?;
    let (scene_locked, chapter_locked): (i32, i32) = stmt
        .query_row(params![scene_id.to_string()], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
    Ok(scene_locked != 0 || chapter_locked != 0)
}

pub fn is_chapter_locked(conn: &Connection, chapter_id: &Uuid) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT locked FROM chapters WHERE id = ?1")?;
    let locked: i32 = stmt.query_row(params![chapter_id.to_string()], |row| row.get(0))?;
    Ok(locked != 0)
}

// ============================================================================
// Rename Operations
// ============================================================================

pub fn rename_chapter(conn: &Connection, chapter_id: &Uuid, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE chapters SET title = ?1 WHERE id = ?2",
        params![title, chapter_id.to_string()],
    )?;
    Ok(())
}

pub fn rename_scene(conn: &Connection, scene_id: &Uuid, title: &str) -> Result<()> {
    conn.execute(
        "UPDATE scenes SET title = ?1 WHERE id = ?2",
        params![title, scene_id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// Get by ID Operations
// ============================================================================

pub fn get_chapter_by_id(conn: &Connection, chapter_id: &Uuid) -> Result<Option<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id, archived, locked, is_part
         FROM chapters WHERE id = ?1",
    )?;

    let chapter = stmt
        .query_row(params![chapter_id.to_string()], |row| {
            Ok(Chapter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                position: row.get(3)?,
                source_id: row.get(4)?,
                archived: row.get::<_, i32>(5)? != 0,
                locked: row.get::<_, i32>(6)? != 0,
                is_part: row.get::<_, i32>(7).unwrap_or(0) != 0,
            })
        })
        .optional()?;

    Ok(chapter)
}

pub fn get_scene_by_id(conn: &Connection, scene_id: &Uuid) -> Result<Option<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT id, chapter_id, title, synopsis, prose, position, source_id, archived, locked, scene_type, scene_status
         FROM scenes WHERE id = ?1",
    )?;

    let scene = stmt
        .query_row(params![scene_id.to_string()], |row| {
            Ok(Scene {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                chapter_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                title: row.get(2)?,
                synopsis: row.get(3)?,
                prose: row.get(4)?,
                position: row.get(5)?,
                source_id: row.get(6)?,
                archived: row.get::<_, i32>(7)? != 0,
                locked: row.get::<_, i32>(8)? != 0,
                scene_type: SceneType::parse(&row.get::<_, String>(9)?),
                scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
            })
        })
        .optional()?;

    Ok(scene)
}

// ============================================================================
// Snapshot Queries
// ============================================================================

pub fn insert_snapshot_metadata(conn: &Connection, snapshot: &SnapshotMetadata) -> Result<()> {
    conn.execute(
        "INSERT INTO snapshots (id, project_id, name, description, trigger_type, created_at, file_path, file_size, uncompressed_size, chapter_count, scene_count, beat_count, word_count, schema_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            snapshot.id.to_string(),
            snapshot.project_id.to_string(),
            snapshot.name,
            snapshot.description,
            snapshot.trigger_type.as_str(),
            snapshot.created_at,
            snapshot.file_path,
            snapshot.file_size,
            snapshot.uncompressed_size,
            snapshot.chapter_count,
            snapshot.scene_count,
            snapshot.beat_count,
            snapshot.word_count,
            snapshot.schema_version,
        ],
    )?;
    Ok(())
}

pub fn get_snapshots_for_project(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<SnapshotMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, trigger_type, created_at, file_path, file_size, uncompressed_size, chapter_count, scene_count, beat_count, word_count, schema_version
         FROM snapshots WHERE project_id = ?1 ORDER BY created_at DESC",
    )?;

    let snapshots = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(SnapshotMetadata {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                name: row.get(2)?,
                description: row.get(3)?,
                trigger_type: SnapshotTrigger::parse(&row.get::<_, String>(4)?)
                    .unwrap_or(SnapshotTrigger::Manual),
                created_at: row.get(5)?,
                file_path: row.get(6)?,
                file_size: row.get(7)?,
                uncompressed_size: row.get(8)?,
                chapter_count: row.get(9)?,
                scene_count: row.get(10)?,
                beat_count: row.get(11)?,
                word_count: row.get(12)?,
                schema_version: row.get(13)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(snapshots)
}

pub fn get_snapshot_by_id(
    conn: &Connection,
    snapshot_id: &Uuid,
) -> Result<Option<SnapshotMetadata>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, description, trigger_type, created_at, file_path, file_size, uncompressed_size, chapter_count, scene_count, beat_count, word_count, schema_version
         FROM snapshots WHERE id = ?1",
    )?;

    let snapshot = stmt
        .query_row(params![snapshot_id.to_string()], |row| {
            Ok(SnapshotMetadata {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
                name: row.get(2)?,
                description: row.get(3)?,
                trigger_type: SnapshotTrigger::parse(&row.get::<_, String>(4)?)
                    .unwrap_or(SnapshotTrigger::Manual),
                created_at: row.get(5)?,
                file_path: row.get(6)?,
                file_size: row.get(7)?,
                uncompressed_size: row.get(8)?,
                chapter_count: row.get(9)?,
                scene_count: row.get(10)?,
                beat_count: row.get(11)?,
                word_count: row.get(12)?,
                schema_version: row.get(13)?,
            })
        })
        .optional()?;

    Ok(snapshot)
}

pub fn delete_snapshot_metadata(conn: &Connection, snapshot_id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM snapshots WHERE id = ?1",
        params![snapshot_id.to_string()],
    )?;
    Ok(())
}

/// Get all scene-character references for a project (for snapshots)
pub fn get_all_scene_character_refs(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<SceneCharacterRef>> {
    let mut stmt = conn.prepare(
        "SELECT scr.scene_id, scr.character_id
         FROM scene_character_refs scr
         JOIN scenes s ON scr.scene_id = s.id
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1",
    )?;

    let refs = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(SceneCharacterRef {
                scene_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                character_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(refs)
}

/// Get all scene-location references for a project (for snapshots)
pub fn get_all_scene_location_refs(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<SceneLocationRef>> {
    let mut stmt = conn.prepare(
        "SELECT slr.scene_id, slr.location_id
         FROM scene_location_refs slr
         JOIN scenes s ON slr.scene_id = s.id
         JOIN chapters c ON s.chapter_id = c.id
         WHERE c.project_id = ?1",
    )?;

    let refs = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(SceneLocationRef {
                scene_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap(),
                location_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(refs)
}

/// Get all chapters for a project including archived (for snapshots)
pub fn get_all_chapters_including_archived(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<Chapter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, title, position, source_id, archived, locked, is_part
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
                archived: row.get::<_, i32>(5)? != 0,
                locked: row.get::<_, i32>(6)? != 0,
                is_part: row.get::<_, i32>(7).unwrap_or(0) != 0,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(chapters)
}

/// Get all scenes for a project including archived (for snapshots)
pub fn get_all_scenes_including_archived(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<Scene>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.chapter_id, s.title, s.synopsis, s.prose, s.position, s.source_id, s.archived, s.locked, s.scene_type, s.scene_status
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
                archived: row.get::<_, i32>(7)? != 0,
                locked: row.get::<_, i32>(8)? != 0,
                scene_type: SceneType::parse(&row.get::<_, String>(9)?),
                scene_status: SceneStatus::parse(&row.get::<_, String>(10)?),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(scenes)
}

/// Get all beats for a project (for snapshots)
pub fn get_all_beats_for_project(conn: &Connection, project_id: &Uuid) -> Result<Vec<Beat>> {
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

/// Delete all project content (for restore)
pub fn delete_all_project_content(conn: &Connection, project_id: &Uuid) -> Result<()> {
    // Delete scene references first
    conn.execute(
        "DELETE FROM scene_character_refs WHERE scene_id IN (
            SELECT s.id FROM scenes s
            JOIN chapters c ON s.chapter_id = c.id
            WHERE c.project_id = ?1
        )",
        params![project_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM scene_location_refs WHERE scene_id IN (
            SELECT s.id FROM scenes s
            JOIN chapters c ON s.chapter_id = c.id
            WHERE c.project_id = ?1
        )",
        params![project_id.to_string()],
    )?;

    // Delete beats
    conn.execute(
        "DELETE FROM beats WHERE scene_id IN (
            SELECT s.id FROM scenes s
            JOIN chapters c ON s.chapter_id = c.id
            WHERE c.project_id = ?1
        )",
        params![project_id.to_string()],
    )?;

    // Delete scenes
    conn.execute(
        "DELETE FROM scenes WHERE chapter_id IN (
            SELECT id FROM chapters WHERE project_id = ?1
        )",
        params![project_id.to_string()],
    )?;

    // Delete chapters
    conn.execute(
        "DELETE FROM chapters WHERE project_id = ?1",
        params![project_id.to_string()],
    )?;

    // Delete characters and their attributes
    conn.execute(
        "DELETE FROM character_attributes WHERE character_id IN (
            SELECT id FROM characters WHERE project_id = ?1
        )",
        params![project_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM characters WHERE project_id = ?1",
        params![project_id.to_string()],
    )?;

    // Delete locations and their attributes
    conn.execute(
        "DELETE FROM location_attributes WHERE location_id IN (
            SELECT id FROM locations WHERE project_id = ?1
        )",
        params![project_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM locations WHERE project_id = ?1",
        params![project_id.to_string()],
    )?;

    // Delete reference items and their attributes
    conn.execute(
        "DELETE FROM reference_item_attributes WHERE reference_item_id IN (
            SELECT id FROM reference_items WHERE project_id = ?1
        )",
        params![project_id.to_string()],
    )?;
    conn.execute(
        "DELETE FROM reference_items WHERE project_id = ?1",
        params![project_id.to_string()],
    )?;

    Ok(())
}

/// Update project metadata
pub fn update_project(conn: &Connection, project: &Project) -> Result<()> {
    let reference_types_json =
        serde_json::to_string(&project.reference_types).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "UPDATE projects SET name = ?1, source_type = ?2, source_path = ?3, modified_at = ?4, author_pen_name = ?5, genre = ?6, description = ?7, word_target = ?8, reference_types = ?9 WHERE id = ?10",
        params![
            project.name,
            project.source_type.as_str(),
            project.source_path,
            project.modified_at,
            project.author_pen_name,
            project.genre,
            project.description,
            project.word_target,
            reference_types_json,
            project.id.to_string(),
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_schema;
    use crate::models::SourceType;
    use std::collections::HashMap;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        conn
    }

    fn create_test_project(conn: &Connection) -> Project {
        let project = Project::new(
            "Test Project".to_string(),
            SourceType::Markdown,
            Some("/test/path".to_string()),
        );
        insert_project(conn, &project).unwrap();
        project
    }

    fn create_test_chapter(conn: &Connection, project_id: Uuid) -> Chapter {
        let chapter = Chapter {
            id: Uuid::new_v4(),
            project_id,
            title: "Test Chapter".to_string(),
            position: 0,
            source_id: None,
            archived: false,
            locked: false,
            is_part: false,
        };
        insert_chapter(conn, &chapter).unwrap();
        chapter
    }

    fn create_test_scene(conn: &Connection, chapter_id: Uuid) -> Scene {
        let scene = Scene {
            id: Uuid::new_v4(),
            chapter_id,
            title: "Test Scene".to_string(),
            synopsis: Some("A test synopsis".to_string()),
            prose: None,
            position: 0,
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
        };
        insert_scene(conn, &scene).unwrap();
        scene
    }

    // ========================================================================
    // Project Tests
    // ========================================================================

    #[test]
    fn test_insert_and_get_project() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);

        let retrieved = get_project(&conn, &project.id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Test Project");
        assert_eq!(retrieved.source_type, SourceType::Markdown);
    }

    #[test]
    fn test_get_nonexistent_project() {
        let conn = setup_test_db();
        let result = get_project(&conn, &Uuid::new_v4()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_recent_projects() {
        let conn = setup_test_db();
        create_test_project(&conn);
        create_test_project(&conn);

        let projects = get_recent_projects(&conn, 10).unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_update_project_modified() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let original_time = project.modified_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        update_project_modified(&conn, &project.id).unwrap();

        let updated = get_project(&conn, &project.id).unwrap().unwrap();
        assert!(updated.modified_at > original_time);
    }

    // ========================================================================
    // Chapter Tests
    // ========================================================================

    #[test]
    fn test_insert_and_get_chapters() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        let chapters = get_chapters(&conn, &project.id).unwrap();
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, chapter.title);
    }

    #[test]
    fn test_get_max_chapter_position() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);

        // Empty project should return -1
        let max = get_max_chapter_position(&conn, &project.id).unwrap();
        assert_eq!(max, -1);

        // After adding a chapter at position 0
        create_test_chapter(&conn, project.id);
        let max = get_max_chapter_position(&conn, &project.id).unwrap();
        assert_eq!(max, 0);
    }

    #[test]
    fn test_reorder_chapters() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);

        let ch1 = Chapter {
            id: Uuid::new_v4(),
            project_id: project.id,
            title: "Chapter 1".to_string(),
            position: 0,
            source_id: None,
            archived: false,
            locked: false,
            is_part: false,
        };
        let ch2 = Chapter {
            id: Uuid::new_v4(),
            project_id: project.id,
            title: "Chapter 2".to_string(),
            position: 1,
            source_id: None,
            archived: false,
            locked: false,
            is_part: false,
        };
        insert_chapter(&conn, &ch1).unwrap();
        insert_chapter(&conn, &ch2).unwrap();

        // Reverse order
        reorder_chapters(&conn, &project.id, &[ch2.id, ch1.id]).unwrap();

        let chapters = get_chapters(&conn, &project.id).unwrap();
        assert_eq!(chapters[0].id, ch2.id);
        assert_eq!(chapters[0].position, 0);
        assert_eq!(chapters[1].id, ch1.id);
        assert_eq!(chapters[1].position, 1);
    }

    #[test]
    fn test_rename_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        rename_chapter(&conn, &chapter.id, "New Title").unwrap();

        let updated = get_chapter_by_id(&conn, &chapter.id).unwrap().unwrap();
        assert_eq!(updated.title, "New Title");
    }

    #[test]
    fn test_delete_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        delete_chapter(&conn, &chapter.id).unwrap();

        let chapters = get_chapters(&conn, &project.id).unwrap();
        assert!(chapters.is_empty());
    }

    // ========================================================================
    // Scene Tests
    // ========================================================================

    #[test]
    fn test_insert_and_get_scenes() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        let scenes = get_scenes(&conn, &chapter.id).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].title, scene.title);
    }

    #[test]
    fn test_update_scene_prose() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        update_scene_prose(&conn, &scene.id, "New prose content").unwrap();

        let updated = get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert_eq!(updated.prose, Some("New prose content".to_string()));
    }

    #[test]
    fn test_update_scene_synopsis() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        update_scene_synopsis(&conn, &scene.id, Some("Updated synopsis")).unwrap();

        let updated = get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert_eq!(updated.synopsis, Some("Updated synopsis".to_string()));
    }

    #[test]
    fn test_move_scene_to_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter1 = create_test_chapter(&conn, project.id);
        let chapter2 = Chapter {
            id: Uuid::new_v4(),
            project_id: project.id,
            title: "Chapter 2".to_string(),
            position: 1,
            source_id: None,
            archived: false,
            locked: false,
            is_part: false,
        };
        insert_chapter(&conn, &chapter2).unwrap();

        let scene = create_test_scene(&conn, chapter1.id);
        move_scene_to_chapter(&conn, &scene.id, &chapter2.id, 0).unwrap();

        let updated = get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert_eq!(updated.chapter_id, chapter2.id);
    }

    #[test]
    fn test_delete_scene() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        delete_scene(&conn, &scene.id).unwrap();

        let scenes = get_scenes(&conn, &chapter.id).unwrap();
        assert!(scenes.is_empty());
    }

    // ========================================================================
    // Beat Tests
    // ========================================================================

    #[test]
    fn test_insert_and_get_beats() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        let beat = Beat {
            id: Uuid::new_v4(),
            scene_id: scene.id,
            content: "Test beat content".to_string(),
            prose: None,
            position: 0,
            source_id: None,
        };
        insert_beat(&conn, &beat).unwrap();

        let beats = get_beats(&conn, &scene.id).unwrap();
        assert_eq!(beats.len(), 1);
        assert_eq!(beats[0].content, "Test beat content");
    }

    #[test]
    fn test_update_beat_prose() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        let beat = Beat {
            id: Uuid::new_v4(),
            scene_id: scene.id,
            content: "Test beat".to_string(),
            prose: None,
            position: 0,
            source_id: None,
        };
        insert_beat(&conn, &beat).unwrap();

        update_beat_prose(&conn, &beat.id, "Beat prose").unwrap();

        let beats = get_beats(&conn, &scene.id).unwrap();
        assert_eq!(beats[0].prose, Some("Beat prose".to_string()));
    }

    // ========================================================================
    // Lock Tests
    // ========================================================================

    #[test]
    fn test_lock_unlock_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        assert!(!is_chapter_locked(&conn, &chapter.id).unwrap());

        lock_chapter(&conn, &chapter.id).unwrap();
        assert!(is_chapter_locked(&conn, &chapter.id).unwrap());

        unlock_chapter(&conn, &chapter.id).unwrap();
        assert!(!is_chapter_locked(&conn, &chapter.id).unwrap());
    }

    #[test]
    fn test_lock_unlock_scene() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        assert!(!is_scene_locked(&conn, &scene.id).unwrap());

        lock_scene(&conn, &scene.id).unwrap();
        assert!(is_scene_locked(&conn, &scene.id).unwrap());

        unlock_scene(&conn, &scene.id).unwrap();
        assert!(!is_scene_locked(&conn, &scene.id).unwrap());
    }

    #[test]
    fn test_scene_locked_by_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        // Scene is unlocked
        assert!(!is_scene_locked(&conn, &scene.id).unwrap());

        // Lock the chapter - scene should now be locked too
        lock_chapter(&conn, &chapter.id).unwrap();
        assert!(is_scene_locked(&conn, &scene.id).unwrap());
    }

    // ========================================================================
    // Character and Location Tests
    // ========================================================================

    #[test]
    fn test_insert_and_get_characters() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);

        let character = Character {
            id: Uuid::new_v4(),
            project_id: project.id,
            name: "Hero".to_string(),
            description: Some("The main character".to_string()),
            attributes: HashMap::from([("role".to_string(), "protagonist".to_string())]),
            source_id: None,
        };
        insert_character(&conn, &character).unwrap();

        let characters = get_characters(&conn, &project.id).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].name, "Hero");
    }

    #[test]
    fn test_insert_and_get_locations() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);

        let location = Location {
            id: Uuid::new_v4(),
            project_id: project.id,
            name: "Castle".to_string(),
            description: Some("A grand castle".to_string()),
            attributes: HashMap::from([("type".to_string(), "building".to_string())]),
            source_id: None,
        };
        insert_location(&conn, &location).unwrap();

        let locations = get_locations(&conn, &project.id).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].name, "Castle");
    }

    // ========================================================================
    // Archive Tests
    // ========================================================================

    #[test]
    fn test_archive_restore_chapter() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        archive_chapter(&conn, &chapter.id).unwrap();
        let archived = get_chapter_by_id(&conn, &chapter.id).unwrap().unwrap();
        assert!(archived.archived);

        restore_chapter(&conn, &chapter.id).unwrap();
        let restored = get_chapter_by_id(&conn, &chapter.id).unwrap().unwrap();
        assert!(!restored.archived);
    }

    #[test]
    fn test_archive_restore_scene() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        archive_scene(&conn, &scene.id).unwrap();
        let archived = get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert!(archived.archived);

        restore_scene(&conn, &scene.id).unwrap();
        let restored = get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert!(!restored.archived);
    }

    #[test]
    fn test_get_archived_chapters() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);

        // Initially no archived chapters
        let archived = get_archived_chapters(&conn, &project.id).unwrap();
        assert!(archived.is_empty());

        // Archive the chapter
        archive_chapter(&conn, &chapter.id).unwrap();
        let archived = get_archived_chapters(&conn, &project.id).unwrap();
        assert_eq!(archived.len(), 1);
    }

    #[test]
    fn test_get_archived_scenes() {
        let conn = setup_test_db();
        let project = create_test_project(&conn);
        let chapter = create_test_chapter(&conn, project.id);
        let scene = create_test_scene(&conn, chapter.id);

        // Initially no archived scenes
        let archived = get_archived_scenes(&conn, &project.id).unwrap();
        assert!(archived.is_empty());

        // Archive the scene
        archive_scene(&conn, &scene.id).unwrap();
        let archived = get_archived_scenes(&conn, &project.id).unwrap();
        assert_eq!(archived.len(), 1);
    }
}
