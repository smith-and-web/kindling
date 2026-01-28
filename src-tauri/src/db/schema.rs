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
            modified_at TEXT NOT NULL,
            author_pen_name TEXT,
            genre TEXT,
            description TEXT,
            word_target INTEGER,
            reference_types TEXT
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
            source_id TEXT,
            scene_type TEXT NOT NULL DEFAULT 'normal',
            scene_status TEXT NOT NULL DEFAULT 'draft'
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

        CREATE TABLE IF NOT EXISTS reference_items (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            reference_type TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            source_id TEXT
        );

        CREATE TABLE IF NOT EXISTS reference_item_attributes (
            reference_item_id TEXT REFERENCES reference_items(id) ON DELETE CASCADE,
            key TEXT NOT NULL,
            value TEXT,
            PRIMARY KEY (reference_item_id, key)
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

        CREATE TABLE IF NOT EXISTS scene_reference_item_refs (
            scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
            reference_item_id TEXT REFERENCES reference_items(id) ON DELETE CASCADE,
            PRIMARY KEY (scene_id, reference_item_id)
        );

        CREATE TABLE IF NOT EXISTS scene_reference_state (
            scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
            reference_type TEXT NOT NULL,
            reference_id TEXT NOT NULL,
            position INTEGER NOT NULL,
            expanded INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (scene_id, reference_type, reference_id)
        );

        CREATE TABLE IF NOT EXISTS session_state (
            project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
            current_scene_id TEXT,
            cursor_position INTEGER,
            scroll_position REAL,
            last_opened_at TEXT
        );

        CREATE TABLE IF NOT EXISTS snapshots (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            trigger_type TEXT NOT NULL,
            created_at TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            uncompressed_size INTEGER,
            chapter_count INTEGER NOT NULL,
            scene_count INTEGER NOT NULL,
            beat_count INTEGER NOT NULL,
            word_count INTEGER,
            schema_version INTEGER NOT NULL DEFAULT 1
        );

        -- Create indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_chapters_project ON chapters(project_id);
        CREATE INDEX IF NOT EXISTS idx_scenes_chapter ON scenes(chapter_id);
        CREATE INDEX IF NOT EXISTS idx_beats_scene ON beats(scene_id);
        CREATE INDEX IF NOT EXISTS idx_characters_project ON characters(project_id);
        CREATE INDEX IF NOT EXISTS idx_locations_project ON locations(project_id);
        CREATE INDEX IF NOT EXISTS idx_reference_items_project ON reference_items(project_id);
        CREATE INDEX IF NOT EXISTS idx_reference_items_type ON reference_items(project_id, reference_type);
        CREATE INDEX IF NOT EXISTS idx_scene_reference_items_scene ON scene_reference_item_refs(scene_id);
        CREATE INDEX IF NOT EXISTS idx_scene_reference_items_item ON scene_reference_item_refs(reference_item_id);
        CREATE INDEX IF NOT EXISTS idx_scene_reference_state_scene ON scene_reference_state(scene_id);
        CREATE INDEX IF NOT EXISTS idx_scene_reference_state_type ON scene_reference_state(scene_id, reference_type);
        CREATE INDEX IF NOT EXISTS idx_snapshots_project ON snapshots(project_id);

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
    if !columns.contains(&"is_part".to_string()) {
        conn.execute(
            "ALTER TABLE chapters ADD COLUMN is_part INTEGER NOT NULL DEFAULT 0",
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
    if !columns.contains(&"scene_type".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN scene_type TEXT NOT NULL DEFAULT 'normal'",
            [],
        )?;
    }
    if !columns.contains(&"scene_status".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN scene_status TEXT NOT NULL DEFAULT 'draft'",
            [],
        )?;
    }

    // Migration: Add project-specific metadata columns
    let columns: Vec<String> = conn
        .prepare("PRAGMA table_info(projects)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !columns.contains(&"author_pen_name".to_string()) {
        conn.execute("ALTER TABLE projects ADD COLUMN author_pen_name TEXT", [])?;
    }
    if !columns.contains(&"genre".to_string()) {
        conn.execute("ALTER TABLE projects ADD COLUMN genre TEXT", [])?;
    }
    if !columns.contains(&"description".to_string()) {
        conn.execute("ALTER TABLE projects ADD COLUMN description TEXT", [])?;
    }
    if !columns.contains(&"word_target".to_string()) {
        conn.execute("ALTER TABLE projects ADD COLUMN word_target INTEGER", [])?;
    }
    if !columns.contains(&"reference_types".to_string()) {
        conn.execute("ALTER TABLE projects ADD COLUMN reference_types TEXT", [])?;
    }

    // Migration: Add scene reference tables if missing
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")?
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !tables.contains(&"scene_reference_item_refs".to_string()) {
        conn.execute(
            "CREATE TABLE scene_reference_item_refs (
                scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
                reference_item_id TEXT REFERENCES reference_items(id) ON DELETE CASCADE,
                PRIMARY KEY (scene_id, reference_item_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_reference_items_scene ON scene_reference_item_refs(scene_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_reference_items_item ON scene_reference_item_refs(reference_item_id)",
            [],
        )?;
    }

    if !tables.contains(&"scene_reference_state".to_string()) {
        conn.execute(
            "CREATE TABLE scene_reference_state (
                scene_id TEXT REFERENCES scenes(id) ON DELETE CASCADE,
                reference_type TEXT NOT NULL,
                reference_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                expanded INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (scene_id, reference_type, reference_id)
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_reference_state_scene ON scene_reference_state(scene_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_scene_reference_state_type ON scene_reference_state(scene_id, reference_type)",
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
        assert!(tables.contains(&"reference_items".to_string()));
        assert!(tables.contains(&"scene_reference_item_refs".to_string()));
        assert!(tables.contains(&"scene_reference_state".to_string()));
    }
}
