//! Snapshot Commands
//!
//! Handles creating, listing, restoring, and deleting project snapshots.

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::db;
use crate::models::{
    Beat, Chapter, Character, DiscoveryNote, FieldDefinition, FieldValue, Location, Project,
    ReferenceItem, RestoreMode, SavedFilter, Scene, SceneReferenceState, SnapshotData,
    SnapshotMetadata, SnapshotOrganization, SnapshotTrigger, Tag,
};

use super::AppState;

/// Get the snapshots directory for a project
fn get_snapshots_dir(app_handle: &AppHandle, project_id: &Uuid) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let snapshots_dir = app_data_dir.join("snapshots").join(project_id.to_string());
    fs::create_dir_all(&snapshots_dir).map_err(|e| e.to_string())?;

    Ok(snapshots_dir)
}

/// Generate a snapshot filename based on trigger type
fn generate_snapshot_filename(trigger: &SnapshotTrigger) -> String {
    let timestamp = chrono::Utc::now().format("%Y-%m-%d_%H%M%S");
    let trigger_str = trigger.as_str();
    format!("{}_{}_{}.json.gz", timestamp, Uuid::new_v4(), trigger_str)
}

/// Collect all project data for snapshotting
fn collect_project_data(
    conn: &rusqlite::Connection,
    project_id: &Uuid,
) -> Result<SnapshotData, String> {
    let project = db::get_project(conn, project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let chapters =
        db::get_all_chapters_including_archived(conn, project_id).map_err(|e| e.to_string())?;

    let scenes =
        db::get_all_scenes_including_archived(conn, project_id).map_err(|e| e.to_string())?;

    let beats = db::get_all_beats_for_project(conn, project_id).map_err(|e| e.to_string())?;

    let characters = db::get_characters(conn, project_id).map_err(|e| e.to_string())?;

    let locations = db::get_locations(conn, project_id).map_err(|e| e.to_string())?;

    let reference_items =
        db::get_all_reference_items(conn, project_id).map_err(|e| e.to_string())?;

    let scene_character_refs =
        db::get_all_scene_character_refs(conn, project_id).map_err(|e| e.to_string())?;

    let scene_location_refs =
        db::get_all_scene_location_refs(conn, project_id).map_err(|e| e.to_string())?;

    let scene_reference_item_refs =
        db::get_all_scene_reference_item_refs(conn, project_id).map_err(|e| e.to_string())?;

    let scene_reference_states =
        db::get_all_scene_reference_states(conn, project_id).map_err(|e| e.to_string())?;

    let discovery_notes =
        db::get_all_discovery_notes_for_project(conn, project_id).map_err(|e| e.to_string())?;

    let mut data = SnapshotData::new(
        project,
        chapters,
        scenes,
        beats,
        characters,
        locations,
        reference_items,
        scene_character_refs,
        scene_location_refs,
        scene_reference_item_refs,
        scene_reference_states,
        discovery_notes,
    );
    data.scene_reviews = db::revisions::backup(conn, project_id)?;
    data.organization = Some(SnapshotOrganization {
        tags: db::get_tags(conn, project_id).map_err(|e| e.to_string())?,
        entity_tags: db::get_all_entity_tags_for_project(conn, project_id)
            .map_err(|e| e.to_string())?,
        saved_filters: db::get_saved_filters(conn, project_id).map_err(|e| e.to_string())?,
        field_definitions: db::get_all_field_definitions(conn, project_id)
            .map_err(|e| e.to_string())?,
        field_values: db::get_all_field_values_for_project(conn, project_id)
            .map_err(|e| e.to_string())?,
    });
    Ok(data)
}

/// Serialize and compress snapshot data to a file
fn serialize_and_compress(data: &SnapshotData, file_path: &PathBuf) -> Result<(i64, i64), String> {
    let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let uncompressed_size = json.len() as i64;

    let file = File::create_new(file_path).map_err(|e| e.to_string())?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder
        .write_all(json.as_bytes())
        .map_err(|e| e.to_string())?;
    encoder.finish().map_err(|e| e.to_string())?;

    let file_size = fs::metadata(file_path).map_err(|e| e.to_string())?.len() as i64;

    Ok((file_size, uncompressed_size))
}

/// Decompress and deserialize snapshot data from a file
fn decompress_and_deserialize(file_path: &PathBuf) -> Result<SnapshotData, String> {
    let file = File::open(file_path).map_err(|e| e.to_string())?;
    let mut decoder = GzDecoder::new(file);
    let mut json = String::new();
    decoder
        .read_to_string(&mut json)
        .map_err(|e| e.to_string())?;

    let data: SnapshotData = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(data)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSnapshotOptions {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: SnapshotTrigger,
}

#[tauri::command]
pub async fn create_snapshot(
    project_id: String,
    options: CreateSnapshotOptions,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<SnapshotMetadata, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    write_snapshot(&conn, &app_handle, &project_uuid, options)
}

/// Writes a snapshot with a connection the caller already holds, so work that
/// is under the lock (a sync apply) can take one without releasing it.
pub(super) fn write_snapshot(
    conn: &rusqlite::Connection,
    app_handle: &AppHandle,
    project_uuid: &Uuid,
    options: CreateSnapshotOptions,
) -> Result<SnapshotMetadata, String> {
    let project_uuid = *project_uuid;
    // Collect project data
    let data = collect_project_data(conn, &project_uuid)?;

    // Generate file path
    let snapshots_dir = get_snapshots_dir(app_handle, &project_uuid)?;
    let filename = generate_snapshot_filename(&options.trigger_type);
    let file_path = snapshots_dir.join(&filename);

    // Serialize and compress
    let (file_size, uncompressed_size) = serialize_and_compress(&data, &file_path)?;

    // Create metadata
    let metadata = SnapshotMetadata::new(
        project_uuid,
        options.name,
        options.description,
        options.trigger_type,
        file_path.to_string_lossy().to_string(),
        file_size,
        Some(uncompressed_size),
        data.chapters.len() as i32,
        data.scenes.len() as i32,
        data.beats.len() as i32,
        Some(data.word_count()),
    );

    // Store metadata in database. Without its record the file is invisible and never
    // pruned, so remove it if the insert fails.
    if let Err(e) = db::insert_snapshot_metadata(conn, &metadata) {
        let _ = std::fs::remove_file(&file_path);
        return Err(e.to_string());
    }

    Ok(metadata)
}

#[tauri::command]
pub async fn list_snapshots(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SnapshotMetadata>, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let snapshots =
        db::get_snapshots_for_project(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(snapshots)
}

#[tauri::command]
pub async fn delete_snapshot(
    snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let snapshot_uuid = Uuid::parse_str(&snapshot_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    remove_snapshot(&conn, &snapshot_uuid)
}

/// Deletes a snapshot's file and record with a connection the caller holds.
pub(super) fn remove_snapshot(
    conn: &rusqlite::Connection,
    snapshot_uuid: &Uuid,
) -> Result<(), String> {
    // Get snapshot metadata to find file path
    let metadata = db::get_snapshot_by_id(conn, snapshot_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Snapshot not found".to_string())?;

    // Delete the file
    let file_path = PathBuf::from(&metadata.file_path);
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    // Delete metadata from database
    db::delete_snapshot_metadata(conn, snapshot_uuid).map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreSnapshotOptions {
    pub mode: RestoreMode,
    pub new_project_name: Option<String>,
}

#[tauri::command]
pub async fn restore_snapshot(
    snapshot_id: String,
    options: RestoreSnapshotOptions,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let snapshot_uuid = Uuid::parse_str(&snapshot_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get snapshot metadata
    let metadata = db::get_snapshot_by_id(&conn, &snapshot_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Snapshot not found".to_string())?;

    // Load snapshot data
    let file_path = PathBuf::from(&metadata.file_path);
    let data = decompress_and_deserialize(&file_path)?;

    match options.mode {
        RestoreMode::ReplaceCurrent => restore_replace_current(&conn, data),
        RestoreMode::CreateNew => restore_create_new(&conn, data, options.new_project_name),
    }
}

/// Restore by replacing current project data
fn restore_replace_current(
    conn: &rusqlite::Connection,
    data: SnapshotData,
) -> Result<Project, String> {
    let project_id = data.project.id;

    // Begin transaction
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Draft history is deleted with the scenes below. Keep it to merge back in.
    let current_reviews = db::revisions::backup(&tx, &project_id)?;

    // Delete all existing project content
    db::delete_all_project_content(&tx, &project_id).map_err(|e| e.to_string())?;

    // The snapshot's settings come back with its content. Name and source stay as they are,
    // as they do for a copy.
    let mut current = db::get_project(&tx, &project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;
    current.author_pen_name = data.project.author_pen_name.clone();
    current.genre = data.project.genre.clone();
    current.description = data.project.description.clone();
    current.word_target = data.project.word_target;
    current.reference_types = data.project.reference_types.clone();
    current.project_type = data.project.project_type.clone();
    current.target_page_count = data.project.target_page_count;
    db::update_project(&tx, &current).map_err(|e| e.to_string())?;

    // Insert chapters
    for chapter in &data.chapters {
        db::insert_chapter(&tx, chapter).map_err(|e| e.to_string())?;
    }

    // Insert scenes
    for scene in &data.scenes {
        db::insert_scene(&tx, scene).map_err(|e| e.to_string())?;
    }

    // Insert beats
    for beat in &data.beats {
        db::insert_beat(&tx, beat).map_err(|e| e.to_string())?;
    }

    // A scene the snapshot restores keeps the draft history recorded since the snapshot.
    // A scene it doesn't contain is removed with its history, like the rest of its content.
    let statuses: HashMap<Uuid, &str> = data
        .scenes
        .iter()
        .map(|scene| (scene.id, scene.scene_status.as_str()))
        .collect();
    let mut snapshot_reviews: HashMap<Uuid, db::revisions::ReviewBackup> = data
        .scene_reviews
        .iter()
        .map(|review| (review.scene_id, review.clone()))
        .collect();
    let mut reviews = Vec::new();
    for current in current_reviews {
        if let Some(status) = statuses.get(&current.scene_id) {
            let snapshot = snapshot_reviews.remove(&current.scene_id);
            reviews.push(db::revisions::merge_backup(snapshot, current, status));
        }
    }
    reviews.extend(snapshot_reviews.into_values());
    for review in &reviews {
        db::revisions::restore_backup(&tx, review, &HashMap::new())?;
    }

    // Insert characters
    for character in &data.characters {
        db::insert_character(&tx, character).map_err(|e| e.to_string())?;
    }

    // Insert locations
    for location in &data.locations {
        db::insert_location(&tx, location).map_err(|e| e.to_string())?;
    }

    // Insert reference items
    for item in &data.reference_items {
        db::insert_reference_item(&tx, item).map_err(|e| e.to_string())?;
    }

    // Insert scene-character references
    for r in &data.scene_character_refs {
        db::add_scene_character_ref(&tx, &r.scene_id, &r.character_id)
            .map_err(|e| e.to_string())?;
    }

    // Insert scene-location references
    for r in &data.scene_location_refs {
        db::add_scene_location_ref(&tx, &r.scene_id, &r.location_id).map_err(|e| e.to_string())?;
    }

    // Insert scene-reference-item references
    for r in &data.scene_reference_item_refs {
        db::add_scene_reference_item_ref(&tx, &r.scene_id, &r.reference_item_id)
            .map_err(|e| e.to_string())?;
    }

    // Insert scene reference state entries
    for state in &data.scene_reference_states {
        db::insert_scene_reference_state(&tx, state).map_err(|e| e.to_string())?;
    }

    // Insert discovery notes
    for note in &data.discovery_notes {
        db::insert_discovery_note(&tx, note).map_err(|e| e.to_string())?;
    }

    if let Some(organization) = &data.organization {
        delete_organization(&tx, &project_id)?;
        restore_organization(&tx, organization, project_id, None)?;
    }

    db::update_project_modified(&tx, &project_id).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    db::get_project(conn, &project_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to retrieve restored project".to_string())
}

/// Restore by creating a new project
fn restore_create_new(
    conn: &rusqlite::Connection,
    data: SnapshotData,
    new_name: Option<String>,
) -> Result<Project, String> {
    // Build ID mappings
    let mut id_map: HashMap<Uuid, Uuid> = HashMap::new();

    // Generate new project ID
    let old_project_id = data.project.id;
    let new_project_id = Uuid::new_v4();
    id_map.insert(old_project_id, new_project_id);

    // Generate new IDs for all entities
    for chapter in &data.chapters {
        id_map.insert(chapter.id, Uuid::new_v4());
    }
    for scene in &data.scenes {
        id_map.insert(scene.id, Uuid::new_v4());
    }
    for beat in &data.beats {
        id_map.insert(beat.id, Uuid::new_v4());
    }
    for character in &data.characters {
        id_map.insert(character.id, Uuid::new_v4());
    }
    for location in &data.locations {
        id_map.insert(location.id, Uuid::new_v4());
    }
    for item in &data.reference_items {
        id_map.insert(item.id, Uuid::new_v4());
    }
    for note in &data.discovery_notes {
        id_map.insert(note.id, Uuid::new_v4());
    }

    // Begin transaction
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Create new project
    let now = chrono::Utc::now().to_rfc3339();
    let new_project = Project {
        id: new_project_id,
        name: new_name.unwrap_or_else(|| format!("{} (Copy)", data.project.name)),
        source_type: data.project.source_type,
        source_path: data.project.source_path,
        created_at: now.clone(),
        modified_at: now,
        // Copy project-specific metadata from snapshot
        author_pen_name: data.project.author_pen_name,
        genre: data.project.genre,
        description: data.project.description,
        word_target: data.project.word_target,
        reference_types: data.project.reference_types,
        project_type: data.project.project_type,
        target_page_count: data.project.target_page_count,
    };

    db::insert_project(&tx, &new_project).map_err(|e| e.to_string())?;

    // Insert chapters with remapped IDs
    let map_id = |old: &Uuid| -> Result<Uuid, String> {
        id_map
            .get(old)
            .copied()
            .ok_or_else(|| format!("Missing ID mapping for {old}"))
    };

    for chapter in &data.chapters {
        let new_chapter = Chapter {
            id: map_id(&chapter.id)?,
            project_id: new_project_id,
            title: chapter.title.clone(),
            position: chapter.position,
            source_id: chapter.source_id.clone(),
            archived: chapter.archived,
            locked: chapter.locked,
            is_part: chapter.is_part,
            synopsis: chapter.synopsis.clone(),
            planning_status: chapter.planning_status,
        };
        db::insert_chapter(&tx, &new_chapter).map_err(|e| e.to_string())?;
    }

    // Insert scenes with remapped IDs
    for scene in &data.scenes {
        let new_scene = Scene {
            id: map_id(&scene.id)?,
            chapter_id: map_id(&scene.chapter_id)?,
            title: scene.title.clone(),
            synopsis: scene.synopsis.clone(),
            prose: scene.prose.clone(),
            position: scene.position,
            source_id: scene.source_id.clone(),
            archived: scene.archived,
            locked: scene.locked,
            scene_type: scene.scene_type,
            scene_status: scene.scene_status,
            planning_status: scene.planning_status,
            editor_mode: scene.editor_mode,
        };
        db::insert_scene(&tx, &new_scene).map_err(|e| e.to_string())?;
    }

    // Insert beats with remapped IDs
    for beat in &data.beats {
        let new_beat = Beat {
            id: map_id(&beat.id)?,
            scene_id: map_id(&beat.scene_id)?,
            content: beat.content.clone(),
            prose: beat.prose.clone(),
            position: beat.position,
            source_id: beat.source_id.clone(),
        };
        db::insert_beat(&tx, &new_beat).map_err(|e| e.to_string())?;
    }

    for review in &data.scene_reviews {
        db::revisions::restore_backup(&tx, review, &id_map)?;
    }

    // Insert characters with remapped IDs
    for character in &data.characters {
        let new_character = Character {
            id: map_id(&character.id)?,
            project_id: new_project_id,
            name: character.name.clone(),
            description: character.description.clone(),
            attributes: character.attributes.clone(),
            source_id: character.source_id.clone(),
        };
        db::insert_character(&tx, &new_character).map_err(|e| e.to_string())?;
    }

    // Insert locations with remapped IDs
    for location in &data.locations {
        let new_location = Location {
            id: map_id(&location.id)?,
            project_id: new_project_id,
            name: location.name.clone(),
            description: location.description.clone(),
            attributes: location.attributes.clone(),
            source_id: location.source_id.clone(),
        };
        db::insert_location(&tx, &new_location).map_err(|e| e.to_string())?;
    }

    // Insert reference items with remapped IDs
    for item in &data.reference_items {
        let new_item = ReferenceItem {
            id: map_id(&item.id)?,
            project_id: new_project_id,
            reference_type: item.reference_type.clone(),
            name: item.name.clone(),
            description: item.description.clone(),
            attributes: item.attributes.clone(),
            source_id: item.source_id.clone(),
        };
        db::insert_reference_item(&tx, &new_item).map_err(|e| e.to_string())?;
    }

    // Insert scene-character references with remapped IDs
    for r in &data.scene_character_refs {
        let new_scene_id = map_id(&r.scene_id)?;
        let new_character_id = map_id(&r.character_id)?;
        db::add_scene_character_ref(&tx, &new_scene_id, &new_character_id)
            .map_err(|e| e.to_string())?;
    }

    // Insert scene-location references with remapped IDs
    for r in &data.scene_location_refs {
        let new_scene_id = map_id(&r.scene_id)?;
        let new_location_id = map_id(&r.location_id)?;
        db::add_scene_location_ref(&tx, &new_scene_id, &new_location_id)
            .map_err(|e| e.to_string())?;
    }

    // Insert scene-reference-item references with remapped IDs
    for r in &data.scene_reference_item_refs {
        let new_scene_id = map_id(&r.scene_id)?;
        let new_reference_item_id = map_id(&r.reference_item_id)?;
        db::add_scene_reference_item_ref(&tx, &new_scene_id, &new_reference_item_id)
            .map_err(|e| e.to_string())?;
    }

    // Insert scene reference state entries with remapped IDs
    for state in &data.scene_reference_states {
        let new_scene_id = map_id(&state.scene_id)?;
        let new_reference_id = match id_map.get(&state.reference_id) {
            Some(id) => id,
            None => continue,
        };
        let new_state = SceneReferenceState {
            scene_id: new_scene_id,
            reference_type: state.reference_type.clone(),
            reference_id: *new_reference_id,
            position: state.position,
            expanded: state.expanded,
        };
        db::insert_scene_reference_state(&tx, &new_state).map_err(|e| e.to_string())?;
    }

    // Insert discovery notes with remapped IDs
    for note in &data.discovery_notes {
        let new_scene_id = map_id(&note.scene_id)?;
        let new_note = DiscoveryNote {
            id: map_id(&note.id)?,
            scene_id: new_scene_id,
            content: note.content.clone(),
            tags: note.tags.clone(),
            position: note.position,
            created_at: note.created_at.clone(),
        };
        db::insert_discovery_note(&tx, &new_note).map_err(|e| e.to_string())?;
    }

    if let Some(organization) = &data.organization {
        restore_organization(&tx, organization, new_project_id, Some(&id_map))?;
    }

    tx.commit().map_err(|e| e.to_string())?;

    Ok(new_project)
}

/// Remove a project's tags, custom fields and saved filters, with their assignments.
fn delete_organization(conn: &rusqlite::Connection, project_id: &Uuid) -> Result<(), String> {
    let project = project_id.to_string();
    for sql in [
        "DELETE FROM field_values WHERE field_definition_id IN
            (SELECT id FROM field_definitions WHERE project_id = ?1)",
        "DELETE FROM field_definitions WHERE project_id = ?1",
        "DELETE FROM entity_tags WHERE tag_id IN (SELECT id FROM tags WHERE project_id = ?1)",
        "DELETE FROM tags WHERE project_id = ?1",
        "DELETE FROM saved_filters WHERE project_id = ?1",
    ] {
        conn.execute(sql, [&project]).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Write a snapshot's tags, custom fields and saved filters into `project_id`.
///
/// `entity_ids` is the old-to-new ID map when restoring as a copy: the organisation then gets
/// new IDs of its own and its assignments follow the entities they were on. `None` restores
/// in place, where every ID is still the one the snapshot recorded.
fn restore_organization(
    conn: &rusqlite::Connection,
    organization: &SnapshotOrganization,
    project_id: Uuid,
    entity_ids: Option<&HashMap<Uuid, Uuid>>,
) -> Result<(), String> {
    let copying = entity_ids.is_some();
    let fresh = |old: Uuid| if copying { Uuid::new_v4() } else { old };
    let tag_map: HashMap<Uuid, Uuid> = organization
        .tags
        .iter()
        .map(|t| (t.id, fresh(t.id)))
        .collect();
    let definition_map: HashMap<Uuid, Uuid> = organization
        .field_definitions
        .iter()
        .map(|d| (d.id, fresh(d.id)))
        .collect();
    let entity = |old: &Uuid| match entity_ids {
        Some(map) => map.get(old).copied(),
        None => Some(*old),
    };

    // A tag's parent has to exist first. A parent the snapshot doesn't hold, or a cycle,
    // leaves the tag at the top level.
    let mut pending: Vec<Tag> = organization
        .tags
        .iter()
        .map(|tag| Tag {
            id: tag_map[&tag.id],
            project_id,
            parent_id: tag.parent_id.and_then(|p| tag_map.get(&p).copied()),
            ..tag.clone()
        })
        .collect();
    let mut inserted: HashSet<Uuid> = HashSet::new();
    while !pending.is_empty() {
        let (ready, waiting): (Vec<Tag>, Vec<Tag>) = pending
            .into_iter()
            .partition(|tag| tag.parent_id.is_none_or(|p| inserted.contains(&p)));
        if ready.is_empty() {
            for mut tag in waiting {
                tag.parent_id = None;
                db::create_tag(conn, &tag).map_err(|e| e.to_string())?;
            }
            break;
        }
        for tag in ready {
            db::create_tag(conn, &tag).map_err(|e| e.to_string())?;
            inserted.insert(tag.id);
        }
        pending = waiting;
    }

    for assignment in &organization.entity_tags {
        let (Some(tag_id), Some(entity_id)) = (
            tag_map.get(&assignment.tag_id),
            entity(&assignment.entity_id),
        ) else {
            continue;
        };
        db::tag_entity(conn, tag_id, &assignment.entity_type, &entity_id)
            .map_err(|e| e.to_string())?;
    }

    for filter in &organization.saved_filters {
        db::save_filter(
            conn,
            &SavedFilter {
                id: fresh(filter.id),
                project_id,
                filter_json: remap_filter_tags(&filter.filter_json, &tag_map),
                ..filter.clone()
            },
        )
        .map_err(|e| e.to_string())?;
    }

    for definition in &organization.field_definitions {
        db::create_field_definition(
            conn,
            &FieldDefinition {
                id: definition_map[&definition.id],
                project_id,
                ..definition.clone()
            },
        )
        .map_err(|e| e.to_string())?;
    }

    for value in &organization.field_values {
        let (Some(definition_id), Some(entity_id)) = (
            definition_map.get(&value.field_definition_id),
            entity(&value.entity_id),
        ) else {
            continue;
        };
        db::insert_field_value(
            conn,
            &FieldValue {
                id: fresh(value.id),
                field_definition_id: *definition_id,
                entity_id,
                value: value.value.clone(),
            },
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// A saved filter names its tags by ID, so a copy's filters must name the copy's tags.
fn remap_filter_tags(filter_json: &str, tag_map: &HashMap<Uuid, Uuid>) -> String {
    let Ok(mut filter) = serde_json::from_str::<serde_json::Value>(filter_json) else {
        return filter_json.to_string();
    };
    if let Some(tags) = filter.get_mut("tags").and_then(|t| t.as_array_mut()) {
        for tag in tags {
            let mapped = tag
                .as_str()
                .and_then(|id| Uuid::parse_str(id).ok())
                .and_then(|id| tag_map.get(&id));
            if let Some(new_id) = mapped {
                *tag = serde_json::Value::String(new_id.to_string());
            }
        }
    }
    filter.to_string()
}

/// Preview snapshot - returns light metadata without full deserialization
#[derive(Debug, Serialize)]
pub struct SnapshotPreview {
    pub metadata: SnapshotMetadata,
    pub project_name: String,
}

#[tauri::command]
pub async fn preview_snapshot(
    snapshot_id: String,
    state: State<'_, AppState>,
) -> Result<SnapshotPreview, String> {
    let snapshot_uuid = Uuid::parse_str(&snapshot_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get snapshot metadata
    let metadata = db::get_snapshot_by_id(&conn, &snapshot_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Snapshot not found".to_string())?;

    // Load snapshot data to get project name
    let file_path = PathBuf::from(&metadata.file_path);
    let data = decompress_and_deserialize(&file_path)?;

    Ok(SnapshotPreview {
        metadata,
        project_name: data.project.name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SourceType;
    use tempfile::tempdir;

    #[test]
    fn tags_fields_filters_and_settings_survive_both_restore_paths() {
        use crate::models::{FieldDefinition, SavedFilter, Tag};
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let mut project = Project::new("Organised".into(), SourceType::Blank, None);
        project.word_target = Some(80_000);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "Scene".into(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();
        let character = Character::new(project.id, "Ana".into(), None, None);
        db::insert_character(&conn, &character).unwrap();

        // A child written before its parent in position order exercises parent-first insertion.
        let parent = Tag::new(project.id, "Subplot".into(), None, None, 1);
        let child = Tag::new(project.id, "Romance".into(), None, Some(parent.id), 0);
        db::create_tag(&conn, &parent).unwrap();
        db::create_tag(&conn, &child).unwrap();
        db::tag_entity(&conn, &child.id, "scene", &scene.id).unwrap();
        let definition = FieldDefinition::new(
            project.id,
            "character".into(),
            "Age".into(),
            "text".into(),
            0,
        );
        db::create_field_definition(&conn, &definition).unwrap();
        db::set_field_value(&conn, &definition.id, &character.id, Some("30")).unwrap();
        let filter = SavedFilter::new(
            project.id,
            "Romance scenes".into(),
            "scene".into(),
            format!(r#"{{"tags":["{}"],"operator":"AND"}}"#, child.id),
            0,
        );
        db::save_filter(&conn, &filter).unwrap();

        let serialized =
            serde_json::to_string(&collect_project_data(&conn, &project.id).unwrap()).unwrap();
        let snapshot: SnapshotData = serde_json::from_str(&serialized).unwrap();

        // Restoring as a copy keeps all of it, on the copy's own IDs.
        let copy = restore_create_new(&conn, snapshot.clone(), Some("Copy".into())).unwrap();
        let tags = db::get_tags(&conn, &copy.id).unwrap();
        assert_eq!(tags.len(), 2);
        let copied_child = tags.iter().find(|t| t.name == "Romance").unwrap();
        let copied_parent = tags.iter().find(|t| t.name == "Subplot").unwrap();
        assert_eq!(copied_child.parent_id, Some(copied_parent.id));
        assert_ne!(copied_child.id, child.id);
        let copied_chapters = db::get_chapters(&conn, &copy.id).unwrap();
        let copied_scene = &db::get_scenes(&conn, &copied_chapters[0].id).unwrap()[0];
        let scene_tags = db::get_entity_tags(&conn, "scene", &copied_scene.id).unwrap();
        assert_eq!(scene_tags.len(), 1);
        assert_eq!(scene_tags[0].id, copied_child.id);
        let definitions = db::get_all_field_definitions(&conn, &copy.id).unwrap();
        assert_eq!(definitions.len(), 1);
        let copied_character = &db::get_characters(&conn, &copy.id).unwrap()[0];
        let values = db::get_field_values(&conn, &copied_character.id).unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].field_definition_id, definitions[0].id);
        assert_eq!(values[0].value.as_deref(), Some("30"));
        let filters = db::get_saved_filters(&conn, &copy.id).unwrap();
        assert_eq!(filters.len(), 1);
        assert!(filters[0]
            .filter_json
            .contains(&copied_child.id.to_string()));
        assert!(!filters[0].filter_json.contains(&child.id.to_string()));
        assert_eq!(copy.word_target, Some(80_000));

        // Restoring in place rolls organisation and settings back to the snapshot.
        db::delete_tag(&conn, &child.id).unwrap();
        db::create_tag(
            &conn,
            &Tag::new(project.id, "Added later".into(), None, None, 5),
        )
        .unwrap();
        db::delete_field_definition(&conn, &definition.id).unwrap();
        db::delete_saved_filter(&conn, &filter.id).unwrap();
        let mut changed = db::get_project(&conn, &project.id).unwrap().unwrap();
        changed.word_target = Some(1);
        db::update_project(&conn, &changed).unwrap();

        let restored = restore_replace_current(&conn, snapshot).unwrap();
        assert_eq!(restored.word_target, Some(80_000));
        let mut names: Vec<_> = db::get_tags(&conn, &project.id)
            .unwrap()
            .into_iter()
            .map(|t| t.name)
            .collect();
        names.sort();
        assert_eq!(names, ["Romance", "Subplot"]);
        assert_eq!(
            db::get_entity_tags(&conn, "scene", &scene.id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            db::get_all_field_definitions(&conn, &project.id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            db::get_field_values(&conn, &character.id).unwrap()[0]
                .value
                .as_deref(),
            Some("30")
        );
        assert_eq!(db::get_saved_filters(&conn, &project.id).unwrap().len(), 1);

        // A snapshot from before organisation was captured leaves current organisation alone.
        let mut old =
            serde_json::to_value(collect_project_data(&conn, &project.id).unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("organization");
        let old: SnapshotData = serde_json::from_value(old).unwrap();
        assert!(old.organization.is_none());
        restore_replace_current(&conn, old).unwrap();
        assert_eq!(db::get_tags(&conn, &project.id).unwrap().len(), 2);
        assert_eq!(
            db::get_all_field_definitions(&conn, &project.id)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn editorial_history_survives_both_snapshot_restore_paths() {
        use crate::db::revisions::{self, Annotation, ReviewDraft};
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let project = Project::new("Editorial snapshot".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "Scene".into(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();
        let mut beat = Beat::new(scene.id, "Beat".into(), 0);
        beat.prose = Some("<p>Original prose</p>".into());
        db::insert_beat(&conn, &beat).unwrap();
        let review = revisions::load(&conn, &scene.id).unwrap();
        let mut data = review.data.clone();
        data.drafts.push(ReviewDraft {
            name: "First draft".into(),
            created_at: "today".into(),
            mode: review.mode,
            documents: review.documents.clone(),
            automatic: false,
        });
        data.annotations.push(Annotation {
            id: "thread".into(),
            document_id: beat.id.to_string(),
            anchor_html: beat.prose.clone().unwrap(),
            from: 1,
            to: 9,
            quote: "Original".into(),
            replacement: Some("Revised".into()),
            state: "open".into(),
            messages: vec![],
        });
        revisions::save(&conn, &review, &data, None).unwrap();
        let snapshot = collect_project_data(&conn, &project.id).unwrap();
        let serialized = serde_json::to_string(&snapshot).unwrap();
        let snapshot: SnapshotData = serde_json::from_str(&serialized).unwrap();
        db::update_beat_prose(&conn, &beat.id, "Later prose").unwrap();
        restore_replace_current(&conn, snapshot.clone()).unwrap();
        let restored = revisions::load(&conn, &scene.id).unwrap();
        assert_eq!(restored.documents[1].html, "<p>Original prose</p>");
        assert_eq!(restored.data.drafts.len(), 1);
        assert_eq!(
            restored.data.annotations[0].document_id,
            beat.id.to_string()
        );
        let copy = restore_create_new(&conn, snapshot, Some("Copy".into())).unwrap();
        let chapters = db::get_chapters(&conn, &copy.id).unwrap();
        let scenes = db::get_scenes(&conn, &chapters[0].id).unwrap();
        let copied = revisions::load(&conn, &scenes[0].id).unwrap();
        assert_ne!(copied.scene_id, restored.scene_id);
        assert_eq!(copied.data.drafts[0].documents[0].id, copied.scene_id);
        assert_eq!(
            copied.data.annotations[0].document_id,
            copied.documents[1].id
        );
        assert_ne!(copied.documents[1].id, beat.id.to_string());
        // Earlier snapshots have no editorial field and still deserialize.
        let mut old =
            serde_json::to_value(collect_project_data(&conn, &project.id).unwrap()).unwrap();
        old.as_object_mut().unwrap().remove("scene_reviews");
        assert!(serde_json::from_value::<SnapshotData>(old)
            .unwrap()
            .scene_reviews
            .is_empty());
    }

    #[test]
    fn replacing_from_a_snapshot_keeps_draft_history_it_never_contained() {
        use crate::db::revisions::{self, Annotation, ReviewDraft};
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        let project = Project::new("History".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let chapter = Chapter::new(project.id, "Chapter".into(), 0);
        db::insert_chapter(&conn, &chapter).unwrap();
        let scene = Scene::new(chapter.id, "Scene".into(), None, 0);
        db::insert_scene(&conn, &scene).unwrap();
        let mut beat = Beat::new(scene.id, "Beat".into(), 0);
        beat.prose = Some("<p>Original prose</p>".into());
        db::insert_beat(&conn, &beat).unwrap();
        let named = |review: &revisions::SceneReview, name: &str| ReviewDraft {
            name: name.into(),
            created_at: "today".into(),
            mode: review.mode,
            documents: review.documents.clone(),
            automatic: false,
        };
        let note = |id: &str| Annotation {
            id: id.into(),
            document_id: beat.id.to_string(),
            anchor_html: "<p>Original prose</p>".into(),
            from: 1,
            to: 9,
            quote: "Original".into(),
            replacement: None,
            state: "open".into(),
            messages: vec![],
        };
        let review = revisions::load(&conn, &scene.id).unwrap();
        let mut data = review.data.clone();
        data.drafts.push(named(&review, "First draft"));
        data.annotations.push(note("thread"));
        revisions::save(&conn, &review, &data, None).unwrap();
        let snapshot = collect_project_data(&conn, &project.id).unwrap();

        // Since the snapshot: another draft, the thread resolved, a new comment, and a new
        // scene with history of its own.
        let review = revisions::load(&conn, &scene.id).unwrap();
        let mut data = review.data.clone();
        data.drafts.push(named(&review, "Second draft"));
        data.annotations[0].state = "resolved".into();
        data.annotations.push(note("later"));
        data.status = "editor_review".into();
        revisions::save(&conn, &review, &data, None).unwrap();
        let added = Scene::new(chapter.id, "Added".into(), None, 1);
        db::insert_scene(&conn, &added).unwrap();
        let review = revisions::load(&conn, &added.id).unwrap();
        let mut data = review.data.clone();
        data.drafts.push(named(&review, "Added draft"));
        revisions::save(&conn, &review, &data, None).unwrap();

        let names = |review: &revisions::SceneReview| {
            review
                .data
                .drafts
                .iter()
                .map(|d| d.name.clone())
                .collect::<Vec<_>>()
        };
        restore_replace_current(&conn, snapshot.clone()).unwrap();
        let restored = revisions::load(&conn, &scene.id).unwrap();
        assert_eq!(names(&restored), ["First draft", "Second draft"]);
        // What the snapshot held is restored as it was; what it never held is kept.
        let states: Vec<_> = restored
            .data
            .annotations
            .iter()
            .map(|a| (a.id.as_str(), a.state.as_str()))
            .collect();
        assert_eq!(states, [("thread", "open"), ("later", "open")]);
        assert_eq!(restored.data.status, "first_draft");
        // A scene the snapshot doesn't contain goes, with its history, like its prose.
        assert!(revisions::load(&conn, &added.id).is_err());

        // A snapshot from before 1.3 carries no reviews at all.
        let mut old = serde_json::to_value(&snapshot).unwrap();
        old.as_object_mut().unwrap().remove("scene_reviews");
        let old: SnapshotData = serde_json::from_value(old).unwrap();
        restore_replace_current(&conn, old).unwrap();
        let restored = revisions::load(&conn, &scene.id).unwrap();
        assert_eq!(names(&restored), ["First draft", "Second draft"]);
        assert_eq!(restored.data.annotations.len(), 2);
        assert_eq!(restored.data.status, "first_draft");
    }

    #[test]
    fn test_generate_snapshot_filename_includes_trigger() {
        let filename = generate_snapshot_filename(&SnapshotTrigger::Manual);
        assert!(filename.ends_with("_manual.json.gz"));
        let parts: Vec<&str> = filename.split('_').collect();
        assert_eq!(parts.len(), 4);
        assert!(Uuid::parse_str(parts[2]).is_ok());
        assert_eq!(parts[3], "manual.json.gz");
    }

    #[test]
    fn rapid_snapshots_keep_independent_content_and_survive_deletion() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        let project = Project::new("First content".into(), SourceType::Blank, None);
        db::insert_project(&conn, &project).unwrap();
        let mut data = collect_project_data(&conn, &project.id).unwrap();
        let dir = tempdir().unwrap();
        for trigger in [SnapshotTrigger::Manual, SnapshotTrigger::Export] {
            // No sleeps: both names may be generated during the same clock tick.
            let first = dir.path().join(generate_snapshot_filename(&trigger));
            let second = dir.path().join(generate_snapshot_filename(&trigger));
            assert_ne!(first, second);
            data.project.name = "First content".into();
            serialize_and_compress(&data, &first).unwrap();
            data.project.name = "Second content".into();
            serialize_and_compress(&data, &second).unwrap();
            assert!(
                serialize_and_compress(&data, &first).is_err(),
                "Existing files must never be truncated"
            );
            assert_eq!(
                decompress_and_deserialize(&first).unwrap().project.name,
                "First content"
            );
            assert_eq!(
                decompress_and_deserialize(&second).unwrap().project.name,
                "Second content"
            );
            fs::remove_file(first).unwrap();
            assert_eq!(
                decompress_and_deserialize(&second).unwrap().project.name,
                "Second content"
            );
        }
    }

    #[test]
    fn test_serialize_and_decompress_roundtrip() {
        let project = Project::new("Snapshot Test".to_string(), SourceType::Markdown, None);
        let data = SnapshotData::new(
            project.clone(),
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
            vec![],
        );

        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("snapshot.json.gz");

        let (file_size, uncompressed_size) = serialize_and_compress(&data, &file_path).unwrap();
        assert!(file_size > 0);
        assert!(uncompressed_size > 0);
        assert!(file_path.exists());

        let restored = decompress_and_deserialize(&file_path).unwrap();
        assert_eq!(restored.project.id, data.project.id);
        assert_eq!(restored.project.name, data.project.name);
    }
}
