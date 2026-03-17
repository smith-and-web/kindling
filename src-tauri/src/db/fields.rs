use crate::models::{FieldDefinition, FieldValue};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

// ============================================================================
// Field Definitions
// ============================================================================

pub fn get_field_definitions(
    conn: &Connection,
    project_id: &Uuid,
    entity_type: &str,
) -> Result<Vec<FieldDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, entity_type, name, field_type, options, default_value,
                position, required, visible, created_at
         FROM field_definitions
         WHERE project_id = ?1 AND entity_type = ?2
         ORDER BY position",
    )?;

    let defs = stmt
        .query_map(params![project_id.to_string(), entity_type], |row| {
            Ok(FieldDefinition {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                entity_type: row.get(2)?,
                name: row.get(3)?,
                field_type: row.get(4)?,
                options: row.get(5)?,
                default_value: row.get(6)?,
                position: row.get(7)?,
                required: row.get::<_, bool>(8)?,
                visible: row.get::<_, bool>(9)?,
                created_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(defs)
}

pub fn get_all_field_definitions(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<FieldDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, entity_type, name, field_type, options, default_value,
                position, required, visible, created_at
         FROM field_definitions
         WHERE project_id = ?1
         ORDER BY entity_type, position",
    )?;

    let defs = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(FieldDefinition {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                entity_type: row.get(2)?,
                name: row.get(3)?,
                field_type: row.get(4)?,
                options: row.get(5)?,
                default_value: row.get(6)?,
                position: row.get(7)?,
                required: row.get::<_, bool>(8)?,
                visible: row.get::<_, bool>(9)?,
                created_at: row.get(10)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(defs)
}

pub fn create_field_definition(conn: &Connection, def: &FieldDefinition) -> Result<()> {
    conn.execute(
        "INSERT INTO field_definitions (id, project_id, entity_type, name, field_type, options,
         default_value, position, required, visible, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            def.id.to_string(),
            def.project_id.to_string(),
            def.entity_type,
            def.name,
            def.field_type,
            def.options,
            def.default_value,
            def.position,
            def.required,
            def.visible,
            def.created_at,
        ],
    )?;
    Ok(())
}

pub fn update_field_definition(
    conn: &Connection,
    id: &Uuid,
    name: &str,
    field_type: &str,
    options: Option<&str>,
    default_value: Option<&str>,
    required: bool,
    visible: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE field_definitions SET name = ?2, field_type = ?3, options = ?4,
         default_value = ?5, required = ?6, visible = ?7
         WHERE id = ?1",
        params![
            id.to_string(),
            name,
            field_type,
            options,
            default_value,
            required,
            visible,
        ],
    )?;
    Ok(())
}

pub fn delete_field_definition(conn: &Connection, id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM field_definitions WHERE id = ?1",
        params![id.to_string()],
    )?;
    Ok(())
}

pub fn reorder_field_definitions(conn: &Connection, ids: &[Uuid]) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE field_definitions SET position = ?2 WHERE id = ?1",
            params![id.to_string(), i as i32],
        )?;
    }
    Ok(())
}

// ============================================================================
// Field Values
// ============================================================================

pub fn get_field_values(
    conn: &Connection,
    entity_id: &Uuid,
) -> Result<Vec<FieldValue>> {
    let mut stmt = conn.prepare(
        "SELECT fv.id, fv.field_definition_id, fv.entity_id, fv.value
         FROM field_values fv
         WHERE fv.entity_id = ?1",
    )?;

    let values = stmt
        .query_map(params![entity_id.to_string()], |row| {
            Ok(FieldValue {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                field_definition_id: Uuid::parse_str(&row.get::<_, String>(1)?)
                    .unwrap_or_default(),
                entity_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                value: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(values)
}

pub fn get_field_values_bulk(
    conn: &Connection,
    entity_ids: &[Uuid],
) -> Result<Vec<FieldValue>> {
    if entity_ids.is_empty() {
        return Ok(vec![]);
    }

    let placeholders: Vec<String> = entity_ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT id, field_definition_id, entity_id, value
         FROM field_values
         WHERE entity_id IN ({})",
        placeholders.join(", ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let str_ids: Vec<String> = entity_ids.iter().map(|id| id.to_string()).collect();
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        str_ids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

    let values = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(FieldValue {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                field_definition_id: Uuid::parse_str(&row.get::<_, String>(1)?)
                    .unwrap_or_default(),
                entity_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                value: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(values)
}

pub fn set_field_value(
    conn: &Connection,
    field_definition_id: &Uuid,
    entity_id: &Uuid,
    value: Option<&str>,
) -> Result<()> {
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO field_values (id, field_definition_id, entity_id, value)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(field_definition_id, entity_id)
         DO UPDATE SET value = excluded.value",
        params![
            id,
            field_definition_id.to_string(),
            entity_id.to_string(),
            value,
        ],
    )?;
    Ok(())
}

pub fn clear_field_value(
    conn: &Connection,
    field_definition_id: &Uuid,
    entity_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "DELETE FROM field_values WHERE field_definition_id = ?1 AND entity_id = ?2",
        params![field_definition_id.to_string(), entity_id.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_schema;

    fn setup() -> (Connection, Uuid) {
        let conn = Connection::open_in_memory().unwrap();
        initialize_schema(&conn).unwrap();
        let project_id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO projects (id, name, source_type, created_at, modified_at) VALUES (?1, 'Test', 'Blank', datetime('now'), datetime('now'))",
            params![project_id.to_string()],
        ).unwrap();
        (conn, project_id)
    }

    #[test]
    fn test_field_definitions_crud() {
        let (conn, project_id) = setup();

        let def = FieldDefinition::new(
            project_id,
            "character".to_string(),
            "Age".to_string(),
            "text".to_string(),
            0,
        );
        create_field_definition(&conn, &def).unwrap();

        let defs = get_field_definitions(&conn, &project_id, "character").unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "Age");
        assert_eq!(defs[0].field_type, "text");

        update_field_definition(
            &conn, &def.id, "Age", "number", None, None, false, true,
        ).unwrap();

        let defs = get_field_definitions(&conn, &project_id, "character").unwrap();
        assert_eq!(defs[0].field_type, "number");

        delete_field_definition(&conn, &def.id).unwrap();
        let defs = get_field_definitions(&conn, &project_id, "character").unwrap();
        assert!(defs.is_empty());
    }

    #[test]
    fn test_field_values_crud() {
        let (conn, project_id) = setup();

        let def = FieldDefinition::new(
            project_id,
            "character".to_string(),
            "Age".to_string(),
            "text".to_string(),
            0,
        );
        create_field_definition(&conn, &def).unwrap();

        let entity_id = Uuid::new_v4();
        set_field_value(&conn, &def.id, &entity_id, Some("30")).unwrap();

        let values = get_field_values(&conn, &entity_id).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, Some("30".to_string()));

        // Upsert
        set_field_value(&conn, &def.id, &entity_id, Some("31")).unwrap();
        let values = get_field_values(&conn, &entity_id).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].value, Some("31".to_string()));

        clear_field_value(&conn, &def.id, &entity_id).unwrap();
        let values = get_field_values(&conn, &entity_id).unwrap();
        assert!(values.is_empty());
    }

    #[test]
    fn test_reorder_field_definitions() {
        let (conn, project_id) = setup();

        let def1 = FieldDefinition::new(project_id, "character".to_string(), "Age".to_string(), "text".to_string(), 0);
        let def2 = FieldDefinition::new(project_id, "character".to_string(), "Role".to_string(), "text".to_string(), 1);
        create_field_definition(&conn, &def1).unwrap();
        create_field_definition(&conn, &def2).unwrap();

        // Reverse order
        reorder_field_definitions(&conn, &[def2.id, def1.id]).unwrap();

        let defs = get_field_definitions(&conn, &project_id, "character").unwrap();
        assert_eq!(defs[0].name, "Role");
        assert_eq!(defs[1].name, "Age");
    }
}
