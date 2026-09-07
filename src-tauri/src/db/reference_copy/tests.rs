use super::*;
use crate::{
    db,
    models::{ReferenceItem, SourceType},
};

fn setup() -> (Connection, Project, Project, CopyRequest) {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
    db::initialize_schema(&conn).unwrap();
    let source = Project::new("Book One".into(), SourceType::Blank, None);
    let mut destination = Project::new("Book Two".into(), SourceType::Blank, None);
    destination.reference_types.clear();
    destination.project_type = "screenplay".into();
    db::insert_project(&conn, &source).unwrap();
    db::insert_project(&conn, &destination).unwrap();
    let request = CopyRequest {
        source_project_id: source.id,
        destination_project_id: destination.id,
        ..Default::default()
    };
    (conn, source, destination, request)
}
fn add(conn: &Connection, project: Uuid, category: &str, name: &str) -> ReferenceKey {
    let id = Uuid::new_v4();
    let (table, attrs, attr_id) = storage(category);
    if table == "reference_items" {
        db::insert_reference_item(
            conn,
            &ReferenceItem {
                id,
                project_id: project,
                reference_type: category.into(),
                name: name.into(),
                description: Some("First line\nSecond line".into()),
                attributes: HashMap::new(),
                source_id: Some("external-id".into()),
            },
        )
        .unwrap();
    } else {
        conn.execute(&format!("INSERT INTO {table} (id, project_id, name, description, source_id) VALUES (?1, ?2, ?3, 'First line' || char(10) || 'Second line', 'external-id')"), params![id.to_string(),project.to_string(),name]).unwrap();
    }
    conn.execute(&format!("INSERT INTO {attrs} ({attr_id}, key, value) VALUES (?1, 'notes', 'A note' || char(10) || 'Another note'), (?1, 'nullable', NULL)"), [id.to_string()]).unwrap();
    ReferenceKey {
        id,
        reference_type: category.into(),
    }
}
fn field(conn: &Connection, project: Uuid, entity: &str, name: &str) -> FieldDefinition {
    let mut f = FieldDefinition::new(project, entity.into(), name.into(), "text".into(), 0);
    f.visible = false;
    f.required = true;
    f.default_value = Some("default".into());
    db::create_field_definition(conn, &f).unwrap();
    f
}
fn apply(conn: &Connection, request: &mut CopyRequest) -> CopyResult {
    let preview = preview(conn, request).unwrap();
    if request.selection.is_none() {
        request.selection = Some(preview.references.iter().map(|r| r.key.clone()).collect());
    }
    copy(conn, request, &preview.revision).unwrap()
}
fn dump(conn: &Connection, id: Uuid) -> serde_json::Value {
    serde_json::to_value(library(conn, id).unwrap()).unwrap()
}

// Startup may derive additional fields/values from legacy attributes. The
// original copied rows must still survive unchanged, including IDs and nulls.
fn assert_library_preserved(before: &serde_json::Value, after: &serde_json::Value) {
    for (key, value) in before.as_object().unwrap() {
        if key == "fields" || key == "values" {
            for original in value.as_array().unwrap() {
                assert!(
                    after[key].as_array().unwrap().contains(original),
                    "Changed {key} row: {original}"
                );
            }
        } else {
            assert_eq!(value, &after[key], "Changed {key}");
        }
    }
}

#[test]
fn full_library_preserves_content_remaps_dependencies_and_survives_restart() {
    let (conn, source, destination, mut request) = setup();
    let root = Tag::new(source.id, "Cast".into(), Some("orange".into()), None, 0);
    let child = Tag::new(source.id, "Heroes".into(), None, Some(root.id), 1);
    db::create_tag(&conn, &root).unwrap();
    db::create_tag(&conn, &child).unwrap();
    for (category, entity) in TYPES {
        let key = add(&conn, source.id, category, "Mara");
        let f = field(&conn, source.id, entity, "Role");
        db::set_field_value(&conn, &f.id, &key.id, Some("Hero\nNow captain")).unwrap();
        let null = field(&conn, source.id, entity, "Null");
        db::set_field_value(&conn, &null.id, &key.id, None).unwrap();
        let empty = field(&conn, source.id, entity, "Empty");
        db::set_field_value(&conn, &empty.id, &key.id, Some("")).unwrap();
        field(&conn, source.id, entity, "No value");
        db::tag_entity(&conn, &child.id, entity, &key.id).unwrap();
    }
    let before = dump(&conn, source.id);
    let result = apply(&conn, &mut request);
    assert_eq!(result.copied, 7);
    assert_eq!(result.project.reference_types.len(), 7);
    let dest = library(&conn, destination.id).unwrap();
    assert_eq!(dest.fields.len(), 28);
    assert_eq!(dest.values.len(), 21);
    assert_eq!(dest.tags.len(), 2);
    assert_eq!(dest.assignments.len(), 7);
    assert_eq!(dest.tags[1].parent_id, Some(dest.tags[0].id));
    for r in &dest.references {
        assert_eq!(r.source_id, None);
        assert!(!before["references"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["key"]["id"] == r.key.id.to_string()));
        assert_eq!(r.description.as_deref(), Some("First line\nSecond line"));
        assert_eq!(
            r.attributes["notes"].as_deref(),
            Some("A note\nAnother note")
        );
        assert_eq!(r.attributes["nullable"], None);
        let values: Vec<_> = dest
            .values
            .iter()
            .filter(|v| v.entity_id == r.key.id)
            .collect();
        assert!(values.iter().any(|v| v.value.is_none()));
        assert!(values.iter().any(|v| v.value.as_deref() == Some("")));
    }
    assert_eq!(before, dump(&conn, source.id));
    let saved = dump(&conn, destination.id);
    db::initialize_schema(&conn).unwrap();
    let reopened = dump(&conn, destination.id);
    assert_library_preserved(&saved, &reopened);
    assert_eq!(reopened["fields"].as_array().unwrap().len(), 38);
    assert_eq!(reopened["values"].as_array().unwrap().len(), 31);
    let saved = reopened;
    let repeated = apply(&conn, &mut request);
    assert_eq!(repeated.copied, 0);
    assert_eq!(repeated.skipped, 7);
    assert_eq!(saved, dump(&conn, destination.id));
}

#[test]
fn subset_conflicts_keep_both_and_noop_do_not_touch_existing_records() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", " Mara ");
    add(&conn, source.id, "locations", "Town");
    add(&conn, destination.id, "characters", "mara");
    add(&conn, destination.id, "characters", "Mara  (copy)");
    request.selection = Some(vec![key.clone()]);
    let before = dump(&conn, destination.id);
    assert_eq!(apply(&conn, &mut request).copied, 0);
    assert_eq!(before, dump(&conn, destination.id));
    request.keep_both = vec![key];
    let preview = preview(&conn, &request).unwrap();
    assert_eq!(preview.copied, 1);
    assert_eq!(preview.references[0].destination_name, " Mara  (copy 2)");
    assert_eq!(apply(&conn, &mut request).copied, 1);
    assert_eq!(library(&conn, destination.id).unwrap().references.len(), 3);
}

#[test]
fn incompatible_fields_and_tag_names_are_preserved_without_overwriting() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    let f = field(&conn, source.id, "character", "Age");
    let mut existing = field(&conn, destination.id, "character", "Age");
    existing.field_type = "number".into();
    conn.execute(
        "UPDATE field_definitions SET field_type = 'number' WHERE id = ?1",
        [existing.id.to_string()],
    )
    .unwrap();
    db::set_field_value(&conn, &f.id, &key.id, Some("unknown")).unwrap();
    let tag = Tag::new(source.id, "Cast".into(), Some("orange".into()), None, 0);
    let other = Tag::new(destination.id, "Cast".into(), Some("blue".into()), None, 0);
    db::create_tag(&conn, &tag).unwrap();
    db::create_tag(&conn, &other).unwrap();
    db::tag_entity(&conn, &tag.id, "character", &key.id).unwrap();
    let p = preview(&conn, &request).unwrap();
    assert_eq!(p.changes.len(), 2);
    assert!(p
        .changes
        .iter()
        .all(|c| c.destination_name.ends_with("(from Book One)")));
    apply(&conn, &mut request);
    let dest = library(&conn, destination.id).unwrap();
    assert_eq!(dest.fields[0].field_type, "number");
    assert_eq!(dest.tags[0].color.as_deref(), Some("blue"));
    assert_eq!(dest.values[0].value.as_deref(), Some("unknown"));
}

#[test]
fn stale_preview_includes_fields_tags_content_and_prevents_replay() {
    for change in [
        "UPDATE character_attributes SET value='changed' WHERE key='notes'",
        "UPDATE field_definitions SET visible=1",
        "UPDATE tags SET color='blue'",
        "UPDATE characters SET name='Changed'",
    ] {
        let (conn, source, destination, mut request) = setup();
        let key = add(&conn, source.id, "characters", "Mara");
        field(&conn, source.id, "character", "Role");
        db::create_tag(&conn, &Tag::new(source.id, "Cast".into(), None, None, 0)).unwrap();
        request.selection = Some(vec![key]);
        let p = preview(&conn, &request).unwrap();
        conn.execute(change, []).unwrap();
        let before = dump(&conn, destination.id);
        assert_eq!(
            copy(&conn, &request, &p.revision).unwrap_err().code,
            "stale_preview"
        );
        assert_eq!(before, dump(&conn, destination.id));
        let fresh = preview(&conn, &request).unwrap();
        copy(&conn, &request, &fresh.revision).unwrap();
        assert_eq!(
            copy(&conn, &request, &fresh.revision).unwrap_err().code,
            "stale_preview"
        );
    }
}

#[test]
fn failure_after_inserts_rolls_back_dependencies_references_and_timestamp() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    let f = field(&conn, source.id, "character", "Role");
    db::set_field_value(&conn, &f.id, &key.id, Some("Hero")).unwrap();
    let tag = Tag::new(source.id, "Cast".into(), None, None, 0);
    db::create_tag(&conn, &tag).unwrap();
    db::tag_entity(&conn, &tag.id, "character", &key.id).unwrap();
    request.selection = Some(vec![key]);
    let p = preview(&conn, &request).unwrap();
    let before = dump(&conn, destination.id);
    conn.execute_batch("CREATE TRIGGER fail_copy BEFORE INSERT ON entity_tags BEGIN SELECT RAISE(ABORT, 'injected failure'); END;").unwrap();
    assert!(copy(&conn, &request, &p.revision).is_err());
    assert_eq!(before, dump(&conn, destination.id));
}

#[test]
fn rejects_invalid_selections_and_dependencies() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    request.selection = Some(vec![key.clone(), key.clone()]);
    assert!(preview(&conn, &request).is_err());
    request.selection = Some(vec![ReferenceKey {
        id: Uuid::new_v4(),
        ..key.clone()
    }]);
    assert!(preview(&conn, &request).is_err());
    request.selection = Some(vec![key.clone()]);
    request.keep_both = vec![ReferenceKey {
        id: Uuid::new_v4(),
        ..key.clone()
    }];
    assert!(preview(&conn, &request).is_err());
    request.keep_both.clear();
    let foreign = field(&conn, destination.id, "character", "Foreign");
    db::set_field_value(&conn, &foreign.id, &key.id, Some("bad")).unwrap();
    assert!(preview(&conn, &request)
        .unwrap_err()
        .message
        .contains("field from another"));
    conn.execute("DELETE FROM field_values", []).unwrap();
    let tag = Tag::new(source.id, "Cycle".into(), None, None, 0);
    db::create_tag(&conn, &tag).unwrap();
    db::tag_entity(&conn, &tag.id, "character", &key.id).unwrap();
    conn.execute("UPDATE tags SET parent_id=id", []).unwrap();
    assert!(preview(&conn, &request)
        .unwrap_err()
        .message
        .contains("cycle"));
    request.selection = Some(vec![]);
    assert_eq!(preview(&conn, &request).unwrap().copied, 0);
    request.destination_project_id = source.id;
    assert!(preview(&conn, &request).is_err());
}

#[test]
fn compatible_definitions_are_reused_but_distinct_source_fields_never_collapse() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    let first = field(&conn, source.id, "character", "Role");
    let second = field(&conn, source.id, "character", "Role");
    let existing = field(&conn, destination.id, "character", "Role");
    db::set_field_value(&conn, &first.id, &key.id, Some("Captain")).unwrap();
    db::set_field_value(&conn, &second.id, &key.id, Some("Sister")).unwrap();
    apply(&conn, &mut request);
    let library = library(&conn, destination.id).unwrap();
    assert_eq!(library.fields.len(), 2);
    assert!(library.fields.iter().any(|f| f.id == existing.id));
    assert_eq!(library.values.len(), 2);
    assert_ne!(
        library.values[0].field_definition_id,
        library.values[1].field_definition_id
    );
    assert!(library
        .values
        .iter()
        .any(|v| v.value.as_deref() == Some("Captain")));
    assert!(library
        .values
        .iter()
        .any(|v| v.value.as_deref() == Some("Sister")));
}

#[test]
fn tag_names_conflicting_under_different_parents_are_renamed_project_wide() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    let parent = Tag::new(source.id, "Cast".into(), None, None, 0);
    let child = Tag::new(source.id, "Heroes".into(), None, Some(parent.id), 1);
    let existing = Tag::new(destination.id, "Heroes".into(), None, None, 0);
    for t in [&parent, &child, &existing] {
        db::create_tag(&conn, t).unwrap();
    }
    db::tag_entity(&conn, &child.id, "character", &key.id).unwrap();
    apply(&conn, &mut request);
    let lib = library(&conn, destination.id).unwrap();
    assert_eq!(lib.tags.len(), 3);
    let copied_child = lib
        .tags
        .iter()
        .find(|t| t.name == "Heroes (from Book One)")
        .unwrap();
    let copied_parent = lib.tags.iter().find(|t| t.name == "Cast").unwrap();
    assert_eq!(copied_child.parent_id, Some(copied_parent.id));
    assert_eq!(lib.assignments[0].tag_id, copied_child.id);
}

#[test]
fn unsupported_categories_and_corrupt_rows_fail_loudly() {
    let (conn, source, _, request) = setup();
    add(&conn, source.id, "future-type", "Unknown");
    assert!(preview(&conn, &request)
        .unwrap_err()
        .message
        .contains("Unsupported"));
    conn.execute("DELETE FROM reference_items", []).unwrap();
    conn.execute(
        "INSERT INTO characters (id, project_id, name) VALUES ('not-a-uuid', ?1, 'Broken')",
        [source.id.to_string()],
    )
    .unwrap();
    assert!(preview(&conn, &request).is_err());
}

#[test]
fn file_database_copy_survives_close_and_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("copy.db");
    let source = Project::new("One".into(), SourceType::Blank, None);
    let destination = Project::new("Two".into(), SourceType::Blank, None);
    let saved;
    {
        let conn = Connection::open(&path).unwrap();
        db::initialize_schema(&conn).unwrap();
        db::insert_project(&conn, &source).unwrap();
        db::insert_project(&conn, &destination).unwrap();
        add(&conn, source.id, "characters", "Mara");
        field(&conn, source.id, "character", "Typed field");
        let mut request = CopyRequest {
            source_project_id: source.id,
            destination_project_id: destination.id,
            ..Default::default()
        };
        apply(&conn, &mut request);
        saved = dump(&conn, destination.id);
    }
    let conn = Connection::open(&path).unwrap();
    db::initialize_schema(&conn).unwrap();
    let reopened = dump(&conn, destination.id);
    assert_library_preserved(&saved, &reopened);
    assert_eq!(reopened["fields"].as_array().unwrap().len(), 3);
    assert_eq!(reopened["values"].as_array().unwrap().len(), 2);
}

#[test]
fn large_selection_is_batched_and_preview_is_read_only() {
    let (conn, source, destination, mut request) = setup();
    for i in 0..500 {
        add(&conn, source.id, "characters", &format!("Character {i:04}"));
    }
    let before = dump(&conn, destination.id);
    let p = preview(&conn, &request).unwrap();
    assert_eq!(p.copied, 500);
    assert_eq!(before, dump(&conn, destination.id));
    let result = apply(&conn, &mut request);
    assert_eq!(result.copied, 500);
    assert_eq!(
        result
            .created_reference_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len(),
        500
    );
}

#[test]
fn review_accepts_singular_and_plural_tag_assignments_without_normalizing() {
    let (conn, source, destination, mut request) = setup();
    let tag = Tag::new(source.id, "Shared".into(), None, None, 0);
    db::create_tag(&conn, &tag).unwrap();
    for (category, entity) in TYPES {
        let key = add(&conn, source.id, category, category);
        for stored_type in BTreeSet::from([*category, *entity]) {
            db::tag_entity(&conn, &tag.id, stored_type, &key.id).unwrap();
        }
    }
    let before = dump(&conn, source.id);
    let result = apply(&conn, &mut request);
    assert_eq!(result.copied, TYPES.len());
    let dest = library(&conn, destination.id).unwrap();
    assert_eq!(dest.assignments.len(), TYPES.len() * 2 - 1);
    for r in &dest.references {
        let types: BTreeSet<_> = dest
            .assignments
            .iter()
            .filter(|a| a.entity_id == r.key.id)
            .map(|a| a.entity_type.as_str())
            .collect();
        assert_eq!(
            types,
            BTreeSet::from([
                r.key.reference_type.as_str(),
                entity_type(&r.key.reference_type).unwrap()
            ])
        );
    }
    assert_eq!(before, dump(&conn, source.id));
}

#[test]
fn review_later_legacy_keys_still_migrate_when_typed_fields_exist() {
    let (conn, source, _, _) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    let existing = field(&conn, source.id, "character", "Role");
    db::set_field_value(&conn, &existing.id, &key.id, Some("Captain")).unwrap();
    conn.execute("INSERT INTO character_attributes (character_id, key, value) VALUES (?1, 'Added later', 'New information')", [key.id.to_string()]).unwrap();
    db::initialize_schema(&conn).unwrap();
    let migrated = library(&conn, source.id).unwrap();
    let new = migrated
        .fields
        .iter()
        .find(|f| f.name == "Added later")
        .expect("later legacy key should migrate");
    assert!(migrated
        .values
        .iter()
        .any(|v| v.field_definition_id == new.id && v.value.as_deref() == Some("New information")));
    assert!(migrated
        .values
        .iter()
        .any(|v| v.field_definition_id == existing.id && v.value.as_deref() == Some("Captain")));
    let saved = dump(&conn, source.id);
    db::initialize_schema(&conn).unwrap();
    assert_eq!(saved, dump(&conn, source.id));
}

#[test]
fn review_project_metadata_saves_do_not_invalidate_preview_or_get_overwritten() {
    let (conn, source, destination, mut request) = setup();
    let key = add(&conn, source.id, "characters", "Mara");
    request.selection = Some(vec![key]);
    let p = preview(&conn, &request).unwrap();
    // This is the timestamp update used by prose saves; metadata is unrelated to copying.
    db::update_project_modified(&conn, &destination.id).unwrap();
    conn.execute(
        "UPDATE projects SET description = 'New synopsis', modified_at = 'later'",
        [],
    )
    .unwrap();
    assert_eq!(p.revision, preview(&conn, &request).unwrap().revision);
    let result = copy(&conn, &request, &p.revision).unwrap();
    assert_eq!(result.copied, 1);
    assert_eq!(result.project.description.as_deref(), Some("New synopsis"));
}

#[test]
fn review_intra_source_duplicates_all_copy_into_empty_destination() {
    let (conn, source, destination, mut request) = setup();
    let first = add(&conn, source.id, "characters", "Mara");
    let second = add(&conn, source.id, "characters", "Mara");
    request.selection = Some(vec![first.clone(), second.clone()]);
    let p = preview(&conn, &request).unwrap();
    assert_eq!(p.copied, 2);
    assert_eq!(p.skipped, 0);
    assert!(p
        .references
        .iter()
        .all(|r| !r.conflict && r.action == "copy"));
    assert_eq!(
        p.references
            .iter()
            .map(|r| r.destination_name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["Mara", "Mara (copy)"])
    );
    let copied = copy(&conn, &request, &p.revision).unwrap();
    assert_eq!(copied.created_reference_ids.len(), 2);
    assert!(copied
        .created_reference_ids
        .iter()
        .all(|id| *id != first.id && *id != second.id));
    assert_eq!(library(&conn, destination.id).unwrap().references.len(), 2);
}

#[test]
fn review_library_and_category_changes_still_invalidate_preview() {
    for project_is_source in [true, false] {
        for change in ["categories", "value", "assignment"] {
            let (conn, source, destination, mut request) = setup();
            let key = add(&conn, source.id, "items", "Compass");
            let other = add(&conn, destination.id, "items", "Map");
            let (project, entity) = if project_is_source {
                (source.id, key.id)
            } else {
                (destination.id, other.id)
            };
            let f = field(&conn, project, "item", "History");
            db::set_field_value(&conn, &f.id, &entity, Some("Original")).unwrap();
            let tag = Tag::new(project, "Journey".into(), None, None, 0);
            db::create_tag(&conn, &tag).unwrap();
            db::tag_entity(&conn, &tag.id, "item", &entity).unwrap();
            request.selection = Some(vec![key]);
            let p = preview(&conn, &request).unwrap();
            match change {
                "categories" => {
                    conn.execute(
                        "UPDATE projects SET reference_types = '[\"items\"]' WHERE id = ?1",
                        [project.to_string()],
                    )
                    .unwrap();
                }
                "value" => {
                    db::set_field_value(&conn, &f.id, &entity, Some("Changed")).unwrap();
                }
                _ => {
                    conn.execute(
                        "UPDATE entity_tags SET entity_type = 'items' WHERE entity_id = ?1",
                        [entity.to_string()],
                    )
                    .unwrap();
                }
            }
            assert_eq!(
                copy(&conn, &request, &p.revision).unwrap_err().code,
                "stale_preview",
                "{change}, source={project_is_source}"
            );
        }
    }
}

#[test]
fn review_wrong_category_tag_assignment_remains_invalid() {
    let (conn, source, _, request) = setup();
    let key = add(&conn, source.id, "items", "Compass");
    let tag = Tag::new(source.id, "Journey".into(), None, None, 0);
    db::create_tag(&conn, &tag).unwrap();
    db::tag_entity(&conn, &tag.id, "locations", &key.id).unwrap();
    assert!(preview(&conn, &request)
        .unwrap_err()
        .message
        .contains("invalid tag assignment"));
}
