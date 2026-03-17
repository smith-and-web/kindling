use rusqlite::{params, Connection, Result};
use uuid::Uuid;

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
            source_id TEXT,
            synopsis TEXT,
            planning_status TEXT NOT NULL DEFAULT 'fixed'
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
            scene_status TEXT NOT NULL DEFAULT 'draft',
            planning_status TEXT NOT NULL DEFAULT 'fixed',
            editor_mode TEXT NOT NULL DEFAULT 'beat'
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

        CREATE TABLE IF NOT EXISTS discovery_notes (
            id TEXT PRIMARY KEY,
            scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
            content TEXT NOT NULL,
            tags TEXT,
            position INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL
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

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            color TEXT,
            parent_id TEXT REFERENCES tags(id) ON DELETE SET NULL,
            position INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(project_id, name)
        );

        CREATE TABLE IF NOT EXISTS entity_tags (
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            PRIMARY KEY (tag_id, entity_type, entity_id)
        );

        CREATE TABLE IF NOT EXISTS saved_filters (
            id TEXT PRIMARY KEY NOT NULL,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            filter_json TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS field_definitions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            entity_type TEXT NOT NULL,
            name TEXT NOT NULL,
            field_type TEXT NOT NULL DEFAULT 'text',
            options TEXT,
            default_value TEXT,
            position INTEGER NOT NULL,
            required INTEGER NOT NULL DEFAULT 0,
            visible INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS field_values (
            id TEXT PRIMARY KEY,
            field_definition_id TEXT NOT NULL REFERENCES field_definitions(id) ON DELETE CASCADE,
            entity_id TEXT NOT NULL,
            value TEXT,
            UNIQUE(field_definition_id, entity_id)
        );

        CREATE TABLE IF NOT EXISTS dismissed_suggestions (
            scene_id TEXT NOT NULL,
            reference_id TEXT NOT NULL,
            dismissed_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (scene_id, reference_id)
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
        CREATE INDEX IF NOT EXISTS idx_discovery_notes_scene ON discovery_notes(scene_id);
        CREATE INDEX IF NOT EXISTS idx_tags_project ON tags(project_id);
        CREATE INDEX IF NOT EXISTS idx_entity_tags_tag ON entity_tags(tag_id);
        CREATE INDEX IF NOT EXISTS idx_entity_tags_entity ON entity_tags(entity_type, entity_id);
        CREATE INDEX IF NOT EXISTS idx_saved_filters_project ON saved_filters(project_id);
        CREATE INDEX IF NOT EXISTS idx_field_definitions_project ON field_definitions(project_id, entity_type);
        CREATE INDEX IF NOT EXISTS idx_field_values_definition ON field_values(field_definition_id);
        CREATE INDEX IF NOT EXISTS idx_field_values_entity ON field_values(entity_id);
        CREATE INDEX IF NOT EXISTS idx_dismissed_suggestions_scene ON dismissed_suggestions(scene_id);

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

    if !tables.contains(&"discovery_notes".to_string()) {
        conn.execute(
            "CREATE TABLE discovery_notes (
                id TEXT PRIMARY KEY,
                scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
                content TEXT NOT NULL,
                tags TEXT,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        conn.execute(
            "CREATE INDEX idx_discovery_notes_scene ON discovery_notes(scene_id)",
            [],
        )?;
    }

    // Migration: Add planning_status and synopsis to chapters
    let chapter_columns: Vec<String> = conn
        .prepare("PRAGMA table_info(chapters)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !chapter_columns.contains(&"planning_status".to_string()) {
        conn.execute(
            "ALTER TABLE chapters ADD COLUMN planning_status TEXT NOT NULL DEFAULT 'fixed'",
            [],
        )?;
    }
    if !chapter_columns.contains(&"synopsis".to_string()) {
        conn.execute("ALTER TABLE chapters ADD COLUMN synopsis TEXT", [])?;
    }

    // Migration: Add planning_status to scenes
    let scene_columns: Vec<String> = conn
        .prepare("PRAGMA table_info(scenes)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !scene_columns.contains(&"planning_status".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN planning_status TEXT NOT NULL DEFAULT 'fixed'",
            [],
        )?;
    }

    // Migration: Add editor_mode to scenes
    let scene_cols: Vec<String> = conn
        .prepare("PRAGMA table_info(scenes)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();

    if !scene_cols.contains(&"editor_mode".to_string()) {
        conn.execute(
            "ALTER TABLE scenes ADD COLUMN editor_mode TEXT NOT NULL DEFAULT 'beat'",
            [],
        )?;
    }

    // Migration: Create field_definitions/field_values tables and migrate attributes
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table'")?
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !tables.contains(&"tags".to_string()) {
        conn.execute_batch(
            r#"
            CREATE TABLE tags (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                color TEXT,
                parent_id TEXT REFERENCES tags(id) ON DELETE SET NULL,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(project_id, name)
            );
            CREATE TABLE entity_tags (
                tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
                entity_type TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                PRIMARY KEY (tag_id, entity_type, entity_id)
            );
            CREATE TABLE saved_filters (
                id TEXT PRIMARY KEY NOT NULL,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                filter_json TEXT NOT NULL,
                position INTEGER NOT NULL DEFAULT 0
            );
            CREATE INDEX idx_tags_project ON tags(project_id);
            CREATE INDEX idx_entity_tags_tag ON entity_tags(tag_id);
            CREATE INDEX idx_entity_tags_entity ON entity_tags(entity_type, entity_id);
            CREATE INDEX idx_saved_filters_project ON saved_filters(project_id);
            "#,
        )?;
    }

    if !tables.contains(&"field_definitions".to_string()) {
        conn.execute_batch(
            r#"
            CREATE TABLE field_definitions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                field_type TEXT NOT NULL DEFAULT 'text',
                options TEXT,
                default_value TEXT,
                position INTEGER NOT NULL,
                required INTEGER NOT NULL DEFAULT 0,
                visible INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE TABLE field_values (
                id TEXT PRIMARY KEY,
                field_definition_id TEXT NOT NULL REFERENCES field_definitions(id) ON DELETE CASCADE,
                entity_id TEXT NOT NULL,
                value TEXT,
                UNIQUE(field_definition_id, entity_id)
            );
            CREATE INDEX idx_field_definitions_project ON field_definitions(project_id, entity_type);
            CREATE INDEX idx_field_values_definition ON field_values(field_definition_id);
            CREATE INDEX idx_field_values_entity ON field_values(entity_id);
            "#,
        )?;
    }

    if !tables.contains(&"dismissed_suggestions".to_string()) {
        conn.execute_batch(
            r#"
            CREATE TABLE dismissed_suggestions (
                scene_id TEXT NOT NULL,
                reference_id TEXT NOT NULL,
                dismissed_at TEXT NOT NULL DEFAULT (datetime('now')),
                PRIMARY KEY (scene_id, reference_id)
            );
            CREATE INDEX idx_dismissed_suggestions_scene ON dismissed_suggestions(scene_id);
            "#,
        )?;
    }

    // Auto-migrate existing *_attributes into field_definitions + field_values
    migrate_attributes_to_fields(conn)?;

    Ok(())
}

/// Migrate legacy *_attributes tables into the new field_definitions + field_values system.
/// Only runs once: skips if field_definitions already has rows for a given project+entity_type.
fn migrate_attributes_to_fields(conn: &Connection) -> Result<()> {
    struct AttrSource {
        entity_type: &'static str,
        table: &'static str,
        id_col: &'static str,
        project_join: &'static str,
    }

    let sources = [
        AttrSource {
            entity_type: "character",
            table: "character_attributes",
            id_col: "character_id",
            project_join: "JOIN characters c ON c.id = a.character_id",
        },
        AttrSource {
            entity_type: "location",
            table: "location_attributes",
            id_col: "location_id",
            project_join: "JOIN locations l ON l.id = a.location_id",
        },
        AttrSource {
            entity_type: "item",
            table: "reference_item_attributes",
            id_col: "reference_item_id",
            project_join: "JOIN reference_items ri ON ri.id = a.reference_item_id AND ri.reference_type = 'items'",
        },
        AttrSource {
            entity_type: "objective",
            table: "reference_item_attributes",
            id_col: "reference_item_id",
            project_join: "JOIN reference_items ri ON ri.id = a.reference_item_id AND ri.reference_type = 'objectives'",
        },
        AttrSource {
            entity_type: "organization",
            table: "reference_item_attributes",
            id_col: "reference_item_id",
            project_join: "JOIN reference_items ri ON ri.id = a.reference_item_id AND ri.reference_type = 'organizations'",
        },
    ];

    for source in &sources {
        // Check if this table actually exists
        let table_exists: bool = conn
            .prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1")?
            .query_row(params![source.table], |row| row.get::<_, i64>(0))
            .map(|c| c > 0)?;

        if !table_exists {
            continue;
        }

        // Check if any rows exist in the legacy table
        let legacy_count: i64 = conn
            .prepare(&format!("SELECT count(*) FROM {}", source.table))?
            .query_row([], |row| row.get(0))?;

        if legacy_count == 0 {
            continue;
        }

        // For reference_item_attributes, only migrate matching reference_type
        let project_col = if source.entity_type == "character" {
            "c.project_id"
        } else if source.entity_type == "location" {
            "l.project_id"
        } else {
            "ri.project_id"
        };

        // Get distinct (project_id, key) pairs
        let sql = format!(
            "SELECT DISTINCT {project_col}, a.key FROM {table} a {join}",
            project_col = project_col,
            table = source.table,
            join = source.project_join,
        );

        let mut stmt = conn.prepare(&sql)?;
        let pairs: Vec<(String, String)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (project_id, key) in &pairs {
            // Check if a field_definition already exists for this project+entity_type+name
            let exists: bool = conn
                .prepare(
                    "SELECT count(*) FROM field_definitions WHERE project_id = ?1 AND entity_type = ?2 AND name = ?3"
                )?
                .query_row(params![project_id, source.entity_type, key], |row| {
                    row.get::<_, i64>(0)
                })
                .map(|c| c > 0)?;

            if exists {
                continue;
            }

            // Get the next position for this project+entity_type
            let max_pos: i32 = conn
                .prepare(
                    "SELECT COALESCE(MAX(position), -1) FROM field_definitions WHERE project_id = ?1 AND entity_type = ?2"
                )?
                .query_row(params![project_id, source.entity_type], |row| row.get(0))?;

            let def_id = Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO field_definitions (id, project_id, entity_type, name, field_type, position)
                 VALUES (?1, ?2, ?3, ?4, 'text', ?5)",
                params![def_id, project_id, source.entity_type, key, max_pos + 1],
            )?;

            // Migrate values for this definition
            let entity_col = source.id_col;
            let values_sql = format!(
                "SELECT a.{entity_col}, a.value FROM {table} a {join} WHERE {project_col} = ?1 AND a.key = ?2",
                entity_col = entity_col,
                table = source.table,
                join = source.project_join,
                project_col = project_col,
            );

            let mut val_stmt = conn.prepare(&values_sql)?;
            let values: Vec<(String, Option<String>)> = val_stmt
                .query_map(params![project_id, key], |row| {
                    Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
                })?
                .filter_map(|r| r.ok())
                .collect();

            for (entity_id, value) in &values {
                let val_id = Uuid::new_v4().to_string();
                conn.execute(
                    "INSERT OR IGNORE INTO field_values (id, field_definition_id, entity_id, value)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![val_id, def_id, entity_id, value],
                )?;
            }
        }
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
        assert!(tables.contains(&"tags".to_string()));
        assert!(tables.contains(&"entity_tags".to_string()));
        assert!(tables.contains(&"saved_filters".to_string()));
        assert!(tables.contains(&"field_definitions".to_string()));
        assert!(tables.contains(&"field_values".to_string()));
        assert!(tables.contains(&"dismissed_suggestions".to_string()));
    }

    #[test]
    fn test_attribute_migration() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();

        let project_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO projects (id, name, source_type, created_at, modified_at) VALUES (?1, 'Test', 'Blank', datetime('now'), datetime('now'))",
            params![project_id],
        ).unwrap();

        let char_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO characters (id, project_id, name) VALUES (?1, ?2, 'Hero')",
            params![char_id, project_id],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO character_attributes (character_id, key, value) VALUES (?1, 'Age', '30')",
            params![char_id],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO character_attributes (character_id, key, value) VALUES (?1, 'Role', 'Protagonist')",
            params![char_id],
        ).unwrap();

        // Run migration
        migrate_attributes_to_fields(&conn).unwrap();

        // Verify field definitions were created
        let def_count: i64 = conn
            .prepare("SELECT count(*) FROM field_definitions WHERE project_id = ?1 AND entity_type = 'character'")
            .unwrap()
            .query_row(params![project_id], |row| row.get(0))
            .unwrap();
        assert_eq!(def_count, 2);

        // Verify field values were created
        let val_count: i64 = conn
            .prepare("SELECT count(*) FROM field_values WHERE entity_id = ?1")
            .unwrap()
            .query_row(params![char_id], |row| row.get(0))
            .unwrap();
        assert_eq!(val_count, 2);

        // Run migration again - should be idempotent
        migrate_attributes_to_fields(&conn).unwrap();
        let def_count_after: i64 = conn
            .prepare("SELECT count(*) FROM field_definitions WHERE project_id = ?1 AND entity_type = 'character'")
            .unwrap()
            .query_row(params![project_id], |row| row.get(0))
            .unwrap();
        assert_eq!(def_count_after, 2);
    }
}
