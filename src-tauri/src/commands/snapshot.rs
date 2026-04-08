//! Snapshot Commands
//!
//! Handles creating, listing, restoring, and deleting project snapshots.

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};
use uuid::Uuid;

use crate::db;
use crate::models::{
    Beat, Chapter, Character, DiscoveryNote, Location, Project, ReferenceItem, RestoreMode, Scene,
    SceneReferenceState, SnapshotData, SnapshotMetadata, SnapshotTrigger,
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
    format!("{}_{}.json.gz", timestamp, trigger_str)
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

    Ok(SnapshotData::new(
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
    ))
}

/// Serialize and compress snapshot data to a file
fn serialize_and_compress(data: &SnapshotData, file_path: &PathBuf) -> Result<(i64, i64), String> {
    let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
    let uncompressed_size = json.len() as i64;

    let file = File::create(file_path).map_err(|e| e.to_string())?;
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

    // Collect project data
    let data = collect_project_data(&conn, &project_uuid)?;

    // Generate file path
    let snapshots_dir = get_snapshots_dir(&app_handle, &project_uuid)?;
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

    // Store metadata in database
    db::insert_snapshot_metadata(&conn, &metadata).map_err(|e| e.to_string())?;

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

    // Get snapshot metadata to find file path
    let metadata = db::get_snapshot_by_id(&conn, &snapshot_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Snapshot not found".to_string())?;

    // Delete the file
    let file_path = PathBuf::from(&metadata.file_path);
    if file_path.exists() {
        fs::remove_file(&file_path).map_err(|e| e.to_string())?;
    }

    // Delete metadata from database
    db::delete_snapshot_metadata(&conn, &snapshot_uuid).map_err(|e| e.to_string())?;

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

    // Delete all existing project content
    db::delete_all_project_content(&tx, &project_id).map_err(|e| e.to_string())?;

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

    tx.commit().map_err(|e| e.to_string())?;

    Ok(new_project)
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
    fn test_generate_snapshot_filename_includes_trigger() {
        let filename = generate_snapshot_filename(&SnapshotTrigger::Manual);
        assert!(filename.ends_with("_manual.json.gz"));
        let parts: Vec<&str> = filename.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[2], "manual.json.gz");
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
