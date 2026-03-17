use crate::models::{EntityTag, SavedFilter, Tag};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

// ============================================================================
// Tags CRUD
// ============================================================================

pub fn get_tags(conn: &Connection, project_id: &Uuid) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, color, parent_id, position, created_at
         FROM tags WHERE project_id = ?1 ORDER BY position",
    )?;

    let tags = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                name: row.get(2)?,
                color: row.get(3)?,
                parent_id: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                position: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tags)
}

pub fn create_tag(conn: &Connection, tag: &Tag) -> Result<()> {
    conn.execute(
        "INSERT INTO tags (id, project_id, name, color, parent_id, position, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            tag.id.to_string(),
            tag.project_id.to_string(),
            tag.name,
            tag.color,
            tag.parent_id.map(|id| id.to_string()),
            tag.position,
            tag.created_at,
        ],
    )?;
    Ok(())
}

pub fn update_tag(
    conn: &Connection,
    id: &Uuid,
    name: Option<&str>,
    color: Option<Option<&str>>,
    parent_id: Option<Option<&Uuid>>,
    position: Option<i32>,
) -> Result<()> {
    if let Some(name) = name {
        conn.execute(
            "UPDATE tags SET name = ?2 WHERE id = ?1",
            params![id.to_string(), name],
        )?;
    }
    if let Some(color) = color {
        conn.execute(
            "UPDATE tags SET color = ?2 WHERE id = ?1",
            params![id.to_string(), color],
        )?;
    }
    if let Some(parent_id) = parent_id {
        conn.execute(
            "UPDATE tags SET parent_id = ?2 WHERE id = ?1",
            params![id.to_string(), parent_id.map(|id| id.to_string())],
        )?;
    }
    if let Some(position) = position {
        conn.execute(
            "UPDATE tags SET position = ?2 WHERE id = ?1",
            params![id.to_string(), position],
        )?;
    }
    Ok(())
}

pub fn delete_tag(conn: &Connection, id: &Uuid) -> Result<()> {
    conn.execute("DELETE FROM tags WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

pub fn reorder_tags(conn: &Connection, ids: &[Uuid]) -> Result<()> {
    for (i, id) in ids.iter().enumerate() {
        conn.execute(
            "UPDATE tags SET position = ?2 WHERE id = ?1",
            params![id.to_string(), i as i32],
        )?;
    }
    Ok(())
}

// ============================================================================
// Entity Tags
// ============================================================================

pub fn tag_entity(
    conn: &Connection,
    tag_id: &Uuid,
    entity_type: &str,
    entity_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO entity_tags (tag_id, entity_type, entity_id)
         VALUES (?1, ?2, ?3)",
        params![tag_id.to_string(), entity_type, entity_id.to_string()],
    )?;
    Ok(())
}

pub fn untag_entity(
    conn: &Connection,
    tag_id: &Uuid,
    entity_type: &str,
    entity_id: &Uuid,
) -> Result<()> {
    conn.execute(
        "DELETE FROM entity_tags WHERE tag_id = ?1 AND entity_type = ?2 AND entity_id = ?3",
        params![tag_id.to_string(), entity_type, entity_id.to_string()],
    )?;
    Ok(())
}

pub fn get_entity_tags(conn: &Connection, entity_type: &str, entity_id: &Uuid) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT t.id, t.project_id, t.name, t.color, t.parent_id, t.position, t.created_at
         FROM tags t
         JOIN entity_tags et ON et.tag_id = t.id
         WHERE et.entity_type = ?1 AND et.entity_id = ?2
         ORDER BY t.position",
    )?;

    let tags = stmt
        .query_map(params![entity_type, entity_id.to_string()], |row| {
            Ok(Tag {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                name: row.get(2)?,
                color: row.get(3)?,
                parent_id: row
                    .get::<_, Option<String>>(4)?
                    .and_then(|s| Uuid::parse_str(&s).ok()),
                position: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tags)
}

pub fn bulk_tag(
    conn: &Connection,
    tag_id: &Uuid,
    entity_type: &str,
    entity_ids: &[Uuid],
) -> Result<()> {
    for entity_id in entity_ids {
        tag_entity(conn, tag_id, entity_type, entity_id)?;
    }
    Ok(())
}

pub fn bulk_untag(
    conn: &Connection,
    tag_id: &Uuid,
    entity_type: &str,
    entity_ids: &[Uuid],
) -> Result<()> {
    for entity_id in entity_ids {
        untag_entity(conn, tag_id, entity_type, entity_id)?;
    }
    Ok(())
}

pub fn get_all_entity_tags_for_project(
    conn: &Connection,
    project_id: &Uuid,
) -> Result<Vec<EntityTag>> {
    let mut stmt = conn.prepare(
        "SELECT et.tag_id, et.entity_type, et.entity_id
         FROM entity_tags et
         JOIN tags t ON t.id = et.tag_id
         WHERE t.project_id = ?1",
    )?;

    let tags = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(EntityTag {
                tag_id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                entity_type: row.get(1)?,
                entity_id: Uuid::parse_str(&row.get::<_, String>(2)?).unwrap_or_default(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tags)
}

// ============================================================================
// Filter Engine
// ============================================================================

/// Filter entities by tag criteria. Returns matching entity IDs.
/// filter_json format: { "tags": ["tag-id-1", "tag-id-2"], "operator": "AND"|"OR" }
pub fn filter_entities(
    conn: &Connection,
    project_id: &Uuid,
    entity_type: &str,
    filter_json: &str,
) -> Result<Vec<Uuid>> {
    let filter: serde_json::Value =
        serde_json::from_str(filter_json).unwrap_or(serde_json::Value::Null);

    let tag_ids: Vec<String> = filter
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    if tag_ids.is_empty() {
        return Ok(vec![]);
    }

    let operator = filter
        .get("operator")
        .and_then(|v| v.as_str())
        .unwrap_or("AND");

    // Build the tag IDs including children (hierarchy traversal)
    let mut all_tag_ids = tag_ids.clone();
    for tag_id_str in &tag_ids {
        collect_child_tags(conn, tag_id_str, &mut all_tag_ids)?;
    }

    let placeholders: Vec<String> = all_tag_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 3))
        .collect();

    let sql = if operator == "OR" {
        format!(
            "SELECT DISTINCT et.entity_id FROM entity_tags et
             JOIN tags t ON t.id = et.tag_id
             WHERE t.project_id = ?1 AND et.entity_type = ?2 AND et.tag_id IN ({})
             ORDER BY et.entity_id",
            placeholders.join(", ")
        )
    } else {
        // AND: entity must have ALL specified tags (any tag_id among all_tag_ids for each original tag)
        format!(
            "SELECT et.entity_id FROM entity_tags et
             JOIN tags t ON t.id = et.tag_id
             WHERE t.project_id = ?1 AND et.entity_type = ?2 AND et.tag_id IN ({})
             GROUP BY et.entity_id
             HAVING COUNT(DISTINCT et.tag_id) >= ?{}",
            placeholders.join(", "),
            all_tag_ids.len() + 3
        )
    };

    let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = vec![
        Box::new(project_id.to_string()),
        Box::new(entity_type.to_string()),
    ];
    for id in &all_tag_ids {
        params_vec.push(Box::new(id.clone()));
    }
    if operator != "OR" {
        params_vec.push(Box::new(tag_ids.len() as i64));
    }

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_vec.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let ids = stmt
        .query_map(param_refs.as_slice(), |row| {
            let id_str: String = row.get(0)?;
            Ok(Uuid::parse_str(&id_str).unwrap_or_default())
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(ids)
}

fn collect_child_tags(
    conn: &Connection,
    parent_id: &str,
    collected: &mut Vec<String>,
) -> Result<()> {
    let mut stmt = conn.prepare("SELECT id FROM tags WHERE parent_id = ?1")?;
    let children: Vec<String> = stmt
        .query_map(params![parent_id], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    for child_id in children {
        if !collected.contains(&child_id) {
            collected.push(child_id.clone());
            collect_child_tags(conn, &child_id, collected)?;
        }
    }

    Ok(())
}

// ============================================================================
// Saved Filters
// ============================================================================

pub fn get_saved_filters(conn: &Connection, project_id: &Uuid) -> Result<Vec<SavedFilter>> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, name, entity_type, filter_json, position
         FROM saved_filters WHERE project_id = ?1 ORDER BY position",
    )?;

    let filters = stmt
        .query_map(params![project_id.to_string()], |row| {
            Ok(SavedFilter {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                project_id: Uuid::parse_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                name: row.get(2)?,
                entity_type: row.get(3)?,
                filter_json: row.get(4)?,
                position: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(filters)
}

pub fn save_filter(conn: &Connection, filter: &SavedFilter) -> Result<()> {
    conn.execute(
        "INSERT INTO saved_filters (id, project_id, name, entity_type, filter_json, position)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            filter.id.to_string(),
            filter.project_id.to_string(),
            filter.name,
            filter.entity_type,
            filter.filter_json,
            filter.position,
        ],
    )?;
    Ok(())
}

pub fn delete_saved_filter(conn: &Connection, id: &Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM saved_filters WHERE id = ?1",
        params![id.to_string()],
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
    fn test_tag_crud() {
        let (conn, project_id) = setup();

        let tag = Tag::new(
            project_id,
            "Action".to_string(),
            Some("#ff0000".to_string()),
            None,
            0,
        );
        create_tag(&conn, &tag).unwrap();

        let tags = get_tags(&conn, &project_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Action");
        assert_eq!(tags[0].color, Some("#ff0000".to_string()));

        update_tag(&conn, &tag.id, Some("Drama"), None, None, None).unwrap();
        let tags = get_tags(&conn, &project_id).unwrap();
        assert_eq!(tags[0].name, "Drama");

        delete_tag(&conn, &tag.id).unwrap();
        let tags = get_tags(&conn, &project_id).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_entity_tagging() {
        let (conn, project_id) = setup();

        let tag = Tag::new(project_id, "Flashback".to_string(), None, None, 0);
        create_tag(&conn, &tag).unwrap();

        let entity_id = Uuid::new_v4();
        tag_entity(&conn, &tag.id, "scene", &entity_id).unwrap();

        let tags = get_entity_tags(&conn, "scene", &entity_id).unwrap();
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Flashback");

        untag_entity(&conn, &tag.id, "scene", &entity_id).unwrap();
        let tags = get_entity_tags(&conn, "scene", &entity_id).unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_hierarchy_and_filter() {
        let (conn, project_id) = setup();

        let parent = Tag::new(project_id, "Subplot".to_string(), None, None, 0);
        create_tag(&conn, &parent).unwrap();

        let child = Tag::new(project_id, "Romance".to_string(), None, Some(parent.id), 1);
        create_tag(&conn, &child).unwrap();

        let scene1 = Uuid::new_v4();
        let scene2 = Uuid::new_v4();

        tag_entity(&conn, &parent.id, "scene", &scene1).unwrap();
        tag_entity(&conn, &child.id, "scene", &scene2).unwrap();

        // Filter by parent tag with OR should include children
        let filter = format!(r#"{{"tags":["{}"],"operator":"OR"}}"#, parent.id);
        let results = filter_entities(&conn, &project_id, "scene", &filter).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_saved_filters() {
        let (conn, project_id) = setup();

        let filter = SavedFilter::new(
            project_id,
            "Flashback scenes".to_string(),
            "scene".to_string(),
            r#"{"tags":["tag-1"],"operator":"AND"}"#.to_string(),
            0,
        );
        save_filter(&conn, &filter).unwrap();

        let filters = get_saved_filters(&conn, &project_id).unwrap();
        assert_eq!(filters.len(), 1);
        assert_eq!(filters[0].name, "Flashback scenes");

        delete_saved_filter(&conn, &filter.id).unwrap();
        let filters = get_saved_filters(&conn, &project_id).unwrap();
        assert!(filters.is_empty());
    }

    #[test]
    fn test_bulk_tag() {
        let (conn, project_id) = setup();

        let tag = Tag::new(project_id, "Important".to_string(), None, None, 0);
        create_tag(&conn, &tag).unwrap();

        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
        bulk_tag(&conn, &tag.id, "scene", &ids).unwrap();

        for id in &ids {
            let tags = get_entity_tags(&conn, "scene", id).unwrap();
            assert_eq!(tags.len(), 1);
        }

        bulk_untag(&conn, &tag.id, "scene", &ids).unwrap();
        for id in &ids {
            let tags = get_entity_tags(&conn, "scene", id).unwrap();
            assert!(tags.is_empty());
        }
    }
}
