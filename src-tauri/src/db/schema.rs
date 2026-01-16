use rusqlite::{Connection, Result};

/// Initialize the database with the full schema and apply migrations
pub fn initialize_schema(conn: &Connection) -> Result<()> {
    // First create all tables
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            source_type TEXT NOT NULL,
            source_path TEXT,
            created_at TEXT NOT NULL,
            modified_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS chapters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            position INTEGER NOT NULL,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS scenes (
            id TEXT PRIMARY KEY,
            chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            synopsis TEXT,
            prose TEXT,
            position INTEGER NOT NULL,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS beats (
            id TEXT PRIMARY KEY,
            scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            prose TEXT,
            position INTEGER NOT NULL,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS characters (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS character_attributes (
            character_id TEXT REFERENCES characters(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT,
            PRIMARY KEY (character_id, key)
        );

        CREATE TABLE IF NOT EXISTS locations (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS location_attributes (
            location_id TEXT REFERENCES locations(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT,
            PRIMARY KEY (location_id, key)
        );

        CREATE TABLE IF NOT EXISTS scene_character_refs (
            scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
            character_id TEXT REFERENCES characters(id) ON DELETE CASCADE,
            PRIMARY KEY (scene_id, character_id)
        );

        CREATE TABLE IF NOT EXISTS scene_location_refs (
            scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
            location_id TEXT REFERENCES locations(id) ON DELETE CASCADE,
            PRIMARY KEY (scene_id, location_id)
        );

        CREATE TABLE IF NOT EXISTS session_state (
            project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            current_scene_id TEXT,
            cursor_position INTEGER,
            scroll_position REAL,
            last_opened_at TEXT
        );

        -- Create indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_chapters_project ON chapters(project_id);
        CREATE INDEX IF NOT EXISTS idx_scenes_chapter ON scenes(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_beats_scene ON beats(scene_id);
        CREATE INDEX IF NOT EXISTS idx_characters_project ON characters(project_id);
        CREATE INDEX IF NOT EXISTS idx_locations_project ON locations(project_id);

        -- Enable foreign key support
        PRAGMA foreign_keys = ON;
        "#,
    )?;

    // Apply migrations for existing databases
    apply_migrations(conn)
}

/// Apply schema migrations for existing databases
fn apply_migrations(conn: &Connection) -> Result<()> {
    // Migration: Add source_id column to chapters if missing
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(chapters)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"source_id".to_string()) {
        conn.execute("ALTER TABLE chapters ADD COLUMN source_id TEXT", [])?;
    }

    // Migration: Add source_id column to scenes if missing
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(scenes)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"source_id".to_string()) {
        conn.execute("ALTER TABLE scenes ADD COLUMN source_id TEXT", [])?;
    }

    // Migration: Add source_id column to beats if missing
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(beats)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"source_id".to_string()) {
        conn.execute("ALTER TABLE beats ADD COLUMN source_id TEXT", [])?;
    }

    // Migration: Add archived and locked columns to chapters
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(chapters)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"archived".to_string()) {
        conn.execute(
            "ALTER TABLE chapters ADD COLUMN archived INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !columns.contains(&"locked".to_string()) {
        conn.execute(
            "ALTER TABLE chapters ADD COLUMN locked INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    // Migration: Add archived and locked columns to scenes
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(scenes)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"archived".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN archived INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !columns.contains(&"locked".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN locked INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"projects".to_string()));
        assert!(tables.contains(&"chapters".to_string()));
        assert!(tables.contains(&"scenes".to_string()));
        assert!(tables.contains(&"beats".to_string()));
        assert!(tables.contains(&"characters".to_string()));
        assert!(tables.contains(&"locations".to_string()));
    }
}
