//! Sync Commands
//!
//! Handles sync/reimport functionality for keeping projects in sync with source files.

use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use tauri::{AppHandle, State};
use uuid::Uuid;

use crate::db;
use crate::models::{Beat, Chapter, EditorMode, PlanningStatus, Scene, SnapshotTrigger};
use crate::parsers::{
    parse_longform_index, parse_markdown_outline, parse_plottr_file, parse_ywriter_file,
};

use super::AppState;

// ============================================================================
// Types
// ============================================================================

#[derive(serde::Serialize)]
pub struct ReimportSummary {
    pub chapters_added: i32,
    pub chapters_updated: i32,
    pub scenes_added: i32,
    pub scenes_updated: i32,
    pub beats_added: i32,
    pub beats_updated: i32,
    pub prose_preserved: i32,
    pub prose_updated: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncAddition {
    pub id: String,
    pub item_type: String, // "chapter", "scene", "beat"
    pub title: String,
    pub parent_title: Option<String>, // Chapter name for scenes, Scene name for beats
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncChange {
    pub id: String,
    pub item_type: String, // "chapter", "scene", "beat"
    pub field: String,     // "title", "synopsis", "content"
    pub item_title: String,
    pub current_value: String,
    pub new_value: String,
    pub db_id: String, // The database ID to update if accepted
    /// Both sides changed since the last sync (or there is no baseline), so
    /// accepting may discard a kindling edit. Never bulk-selected.
    pub conflict: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncPreview {
    pub additions: Vec<SyncAddition>,
    pub changes: Vec<SyncChange>,
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn reimport_project(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<ReimportSummary, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    reimport_project_with_connection(&conn, project_uuid)
}

fn reimport_project_with_connection(
    conn: &Connection,
    project_uuid: Uuid,
) -> Result<ReimportSummary, String> {
    // Get the existing project to find source path and type
    let project = db::get_project(conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for reimport".to_string())?;

    // Re-parse the source file based on source type
    let mut parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::YWriter => {
            let yw_parsed = parse_ywriter_file(source_path).map_err(|e| e.to_string())?;
            // Convert ParsedYWriter to the same structure as ParsedPlottr
            crate::parsers::ParsedPlottr {
                project: yw_parsed.project,
                chapters: yw_parsed.chapters,
                scenes: yw_parsed.scenes,
                beats: yw_parsed.beats,
                characters: yw_parsed.characters,
                locations: yw_parsed.locations,
                scene_character_refs: yw_parsed.scene_character_refs,
                scene_location_refs: yw_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Scrivener => {
            return Err(
                "Scrivener import has been deprecated. This project cannot be reimported."
                    .to_string(),
            );
        }
        crate::models::SourceType::Longform => {
            let lf_parsed = parse_longform_index(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: lf_parsed.project,
                chapters: lf_parsed.chapters,
                scenes: lf_parsed.scenes,
                beats: lf_parsed.beats,
                characters: lf_parsed.characters,
                locations: lf_parsed.locations,
                scene_character_refs: lf_parsed.scene_character_refs,
                scene_location_refs: lf_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Markdown => {
            let md_parsed = parse_markdown_outline(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: md_parsed.project,
                chapters: md_parsed.chapters,
                scenes: md_parsed.scenes,
                beats: md_parsed.beats,
                characters: Vec::new(),
                locations: Vec::new(),
                scene_character_refs: Vec::new(),
                scene_location_refs: Vec::new(),
            }
        }
        crate::models::SourceType::NovelWriter => {
            let parsed = super::novelwriter_sync::load(conn, &project)?;
            let preview = super::novelwriter_sync::preview(conn, &project, &parsed)?;
            return super::novelwriter_sync::apply(
                conn,
                &project,
                &parsed,
                &preview
                    .changes
                    .into_iter()
                    .filter(|c| c.field != "prose" && !c.conflict)
                    .map(|c| c.id)
                    .collect::<Vec<_>>(),
                &preview
                    .additions
                    .into_iter()
                    .map(|a| a.id)
                    .collect::<Vec<_>>(),
                &[],
                &mut |_| Ok(()),
            );
        }
        crate::models::SourceType::Blank => {
            return Err("Blank projects have no source to reimport".to_string());
        }
    };
    reconcile_markdown(conn, &project, &mut parsed)?;

    let mut summary = ReimportSummary {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
        prose_updated: 0,
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Process chapters
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            // Try to find existing chapter by source_id
            if let Some(existing) = db::find_chapter_by_source_id(&tx, &project_uuid, source_id)
                .map_err(|e| e.to_string())?
            {
                if existing.locked {
                    continue;
                }
                // Update existing chapter
                db::update_chapter(conn, &existing.id, &new_chapter.title, new_chapter.position)
                    .map_err(|e| e.to_string())?;
                summary.chapters_updated += 1;
            } else {
                // Insert new chapter with project's actual UUID
                let chapter_to_insert = Chapter {
                    id: new_chapter.id,
                    project_id: project_uuid,
                    title: new_chapter.title.clone(),
                    position: new_chapter.position,
                    source_id: new_chapter.source_id.clone(),
                    archived: false,
                    locked: false,
                    is_part: new_chapter.is_part,
                    synopsis: None,
                    planning_status: PlanningStatus::Fixed,
                };
                db::insert_chapter(&tx, &chapter_to_insert).map_err(|e| e.to_string())?;
                summary.chapters_added += 1;
            }
        }
    }

    // Build a map from parsed chapter source_id to our DB chapter
    let db_chapters = db::get_chapters(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let chapter_source_to_db: HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to parsed chapter source_id
    let parsed_chapter_id_to_source: HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Find the DB chapter this scene belongs to
            let parsed_chapter_source_id =
                parsed_chapter_id_to_source
                    .get(&new_scene.chapter_id)
                    .ok_or_else(|| "Scene references unknown chapter".to_string())?;
            let db_chapter = chapter_source_to_db
                .get(parsed_chapter_source_id)
                .ok_or_else(|| "Could not find DB chapter for scene".to_string())?;

            if db_chapter.locked {
                continue;
            }
            // Try to find existing scene by source_id
            if let Some(existing) = db::find_scene_by_source_id(&tx, &db_chapter.id, source_id)
                .map_err(|e| e.to_string())?
            {
                if db::is_scene_locked(&tx, &existing.id).map_err(|e| e.to_string())? {
                    continue;
                }
                // Update existing scene (preserving prose!)
                db::update_scene(
                    conn,
                    &existing.id,
                    &new_scene.title,
                    new_scene.synopsis.as_deref(),
                    new_scene.position,
                    &new_scene.scene_type,
                    &new_scene.scene_status,
                )
                .map_err(|e| e.to_string())?;
                summary.scenes_updated += 1;
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Insert new scene with DB chapter's UUID
                let scene_to_insert = Scene {
                    id: new_scene.id,
                    chapter_id: db_chapter.id,
                    title: new_scene.title.clone(),
                    synopsis: new_scene.synopsis.clone(),
                    prose: None,
                    position: new_scene.position,
                    source_id: new_scene.source_id.clone(),
                    archived: false,
                    locked: false,
                    scene_type: new_scene.scene_type,
                    scene_status: new_scene.scene_status,
                    planning_status: PlanningStatus::Fixed,
                    editor_mode: EditorMode::Beat,
                };
                db::insert_scene(&tx, &scene_to_insert).map_err(|e| e.to_string())?;
                summary.scenes_added += 1;
            }
        }
    }

    // Build scene source_id to DB scene map
    let db_scenes = db::get_all_project_scenes(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let scene_source_to_db: HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed scene ID to parsed scene source_id
    let parsed_scene_id_to_source: HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    let parsed_scene_chapters: HashMap<Uuid, Uuid> =
        parsed.scenes.iter().map(|s| (s.id, s.chapter_id)).collect();
    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // A new scene may have been skipped because its chapter was locked.
            // Skip its descendants before requiring a database scene to exist.
            if parsed_scene_chapters
                .get(&new_beat.scene_id)
                .and_then(|id| parsed_chapter_id_to_source.get(id))
                .and_then(|sid| chapter_source_to_db.get(sid))
                .is_some_and(|c| c.locked)
            {
                continue;
            }
            // Find the DB scene this beat belongs to
            let parsed_scene_source_id = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .ok_or_else(|| "Beat references unknown scene".to_string())?;
            let db_scene = scene_source_to_db
                .get(parsed_scene_source_id)
                .ok_or_else(|| "Could not find DB scene for beat".to_string())?;

            if db::is_scene_locked(&tx, &db_scene.id).map_err(|e| e.to_string())? {
                continue;
            }
            // Try to find existing beat by source_id
            if let Some(existing) = db::find_beat_by_source_id(&tx, &db_scene.id, source_id)
                .map_err(|e| e.to_string())?
            {
                // Update existing beat (preserving prose!)
                db::update_beat(&tx, &existing.id, &new_beat.content, new_beat.position)
                    .map_err(|e| e.to_string())?;
                summary.beats_updated += 1;
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Insert new beat with DB scene's UUID
                let beat_to_insert = Beat {
                    id: new_beat.id,
                    scene_id: db_scene.id,
                    content: new_beat.content.clone(),
                    prose: None,
                    position: new_beat.position,
                    source_id: new_beat.source_id.clone(),
                };
                db::insert_beat(&tx, &beat_to_insert).map_err(|e| e.to_string())?;
                summary.beats_added += 1;
            }
        }
    }

    db::update_project_modified(&tx, &project_uuid).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(summary)
}

#[tauri::command]
pub async fn get_sync_preview(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<SyncPreview, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    get_sync_preview_with_connection(&conn, project_uuid)
}

pub(super) fn get_sync_preview_with_connection(
    conn: &Connection,
    project_uuid: Uuid,
) -> Result<SyncPreview, String> {
    // Get the existing project to find source path and type
    let project = db::get_project(conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for sync".to_string())?;

    // Re-parse the source file based on source type
    let mut parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::YWriter => {
            let yw_parsed = parse_ywriter_file(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: yw_parsed.project,
                chapters: yw_parsed.chapters,
                scenes: yw_parsed.scenes,
                beats: yw_parsed.beats,
                characters: yw_parsed.characters,
                locations: yw_parsed.locations,
                scene_character_refs: yw_parsed.scene_character_refs,
                scene_location_refs: yw_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Scrivener => {
            return Err(
                "Scrivener import has been deprecated. This project cannot be synced.".to_string(),
            );
        }
        crate::models::SourceType::Longform => {
            let lf_parsed = parse_longform_index(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: lf_parsed.project,
                chapters: lf_parsed.chapters,
                scenes: lf_parsed.scenes,
                beats: lf_parsed.beats,
                characters: lf_parsed.characters,
                locations: lf_parsed.locations,
                scene_character_refs: lf_parsed.scene_character_refs,
                scene_location_refs: lf_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Markdown => {
            let md_parsed = parse_markdown_outline(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: md_parsed.project,
                chapters: md_parsed.chapters,
                scenes: md_parsed.scenes,
                beats: md_parsed.beats,
                characters: Vec::new(),
                locations: Vec::new(),
                scene_character_refs: Vec::new(),
                scene_location_refs: Vec::new(),
            }
        }
        crate::models::SourceType::NovelWriter => {
            let parsed = super::novelwriter_sync::load(conn, &project)?;
            return super::novelwriter_sync::preview(conn, &project, &parsed);
        }
        crate::models::SourceType::Blank => {
            return Err("Blank projects have no source to reimport".to_string());
        }
    };
    reconcile_markdown(conn, &project, &mut parsed)?;

    let mut preview = SyncPreview {
        additions: Vec::new(),
        changes: Vec::new(),
    };

    // Get existing DB data
    let db_chapters = db::get_chapters(conn, &project_uuid).map_err(|e| e.to_string())?;
    let chapter_source_to_db: HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to chapter for lookups
    let parsed_chapter_map: HashMap<Uuid, &Chapter> =
        parsed.chapters.iter().map(|c| (c.id, c)).collect();

    // Process chapters
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            if let Some(existing) = chapter_source_to_db.get(source_id) {
                // Skip locked chapters
                if existing.locked {
                    continue;
                }
                // Check for changes
                if existing.title != new_chapter.title {
                    preview.changes.push(SyncChange {
                        id: format!("chapter-title-{}", existing.id),
                        item_type: "chapter".to_string(),
                        field: "title".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing.title.clone(),
                        new_value: new_chapter.title.clone(),
                        db_id: existing.id.to_string(),
                        conflict: false,
                    });
                }
            } else {
                // New chapter
                preview.additions.push(SyncAddition {
                    id: format!("chapter-{}", source_id),
                    item_type: "chapter".to_string(),
                    title: new_chapter.title.clone(),
                    parent_title: None,
                });
            }
        }
    }

    // Get all scenes for the project
    let db_scenes = db::get_all_project_scenes(conn, &project_uuid).map_err(|e| e.to_string())?;
    let scene_source_to_db: HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed chapter ID to source_id
    let parsed_chapter_id_to_source: HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Build map from parsed scene ID to scene for lookups
    let parsed_scene_map: HashMap<Uuid, &Scene> = parsed.scenes.iter().map(|s| (s.id, s)).collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Get parent chapter name for context
            let parent_chapter_name = parsed_chapter_id_to_source
                .get(&new_scene.chapter_id)
                .and_then(|ch_source_id| chapter_source_to_db.get(ch_source_id))
                .map(|ch| ch.title.clone())
                .or_else(|| {
                    parsed_chapter_map
                        .get(&new_scene.chapter_id)
                        .map(|ch| ch.title.clone())
                });

            if parsed_chapter_id_to_source
                .get(&new_scene.chapter_id)
                .and_then(|sid| chapter_source_to_db.get(sid))
                .is_some_and(|ch| ch.locked)
            {
                continue;
            }
            if let Some(existing) = scene_source_to_db.get(source_id) {
                // Skip locked scenes (or scenes in locked chapters)
                if db::is_scene_locked(conn, &existing.id).map_err(|e| e.to_string())? {
                    continue;
                }
                // Check if parent chapter is locked
                if let Some(ch_source_id) = parsed_chapter_id_to_source.get(&new_scene.chapter_id) {
                    if let Some(ch) = chapter_source_to_db.get(ch_source_id) {
                        if ch.locked {
                            continue;
                        }
                    }
                }
                // Check for title changes
                if existing.title != new_scene.title {
                    preview.changes.push(SyncChange {
                        id: format!("scene-title-{}", existing.id),
                        item_type: "scene".to_string(),
                        field: "title".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing.title.clone(),
                        new_value: new_scene.title.clone(),
                        db_id: existing.id.to_string(),
                        conflict: false,
                    });
                }
                // Check for synopsis changes
                let existing_synopsis = existing.synopsis.clone().unwrap_or_default();
                let new_synopsis = new_scene.synopsis.clone().unwrap_or_default();
                if existing_synopsis != new_synopsis {
                    preview.changes.push(SyncChange {
                        id: format!("scene-synopsis-{}", existing.id),
                        item_type: "scene".to_string(),
                        field: "synopsis".to_string(),
                        item_title: existing.title.clone(),
                        current_value: existing_synopsis,
                        new_value: new_synopsis,
                        db_id: existing.id.to_string(),
                        conflict: false,
                    });
                }
            } else {
                // New scene
                preview.additions.push(SyncAddition {
                    id: format!("scene-{}", source_id),
                    item_type: "scene".to_string(),
                    title: new_scene.title.clone(),
                    parent_title: parent_chapter_name,
                });
            }
        }
    }

    // Get all beats for the project
    let db_beats = db::get_all_project_beats(conn, &project_uuid).map_err(|e| e.to_string())?;
    let beat_source_to_db: HashMap<String, &Beat> = db_beats
        .iter()
        .filter_map(|b| b.source_id.as_ref().map(|sid| (sid.clone(), b)))
        .collect();

    // Build map from parsed scene ID to source_id
    let parsed_scene_id_to_source: HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // Get parent scene name for context
            let parent_scene_name = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .and_then(|sc_source_id| scene_source_to_db.get(sc_source_id))
                .map(|sc| sc.title.clone())
                .or_else(|| {
                    parsed_scene_map
                        .get(&new_beat.scene_id)
                        .map(|sc| sc.title.clone())
                });

            if parsed_scene_map
                .get(&new_beat.scene_id)
                .and_then(|s| parsed_chapter_id_to_source.get(&s.chapter_id))
                .and_then(|sid| chapter_source_to_db.get(sid))
                .is_some_and(|ch| ch.locked)
            {
                continue;
            }
            // Check if parent scene is locked
            if let Some(sc_source_id) = parsed_scene_id_to_source.get(&new_beat.scene_id) {
                if let Some(sc) = scene_source_to_db.get(sc_source_id) {
                    if db::is_scene_locked(conn, &sc.id).map_err(|e| e.to_string())? {
                        continue;
                    }
                }
            }

            if let Some(existing) = beat_source_to_db.get(source_id) {
                if db::is_scene_locked(conn, &existing.scene_id).map_err(|e| e.to_string())? {
                    continue;
                }
                // Check for content changes
                if existing.content != new_beat.content {
                    preview.changes.push(SyncChange {
                        id: format!("beat-content-{}", existing.id),
                        item_type: "beat".to_string(),
                        field: "content".to_string(),
                        item_title: truncate_string(&existing.content, 50),
                        current_value: existing.content.clone(),
                        new_value: new_beat.content.clone(),
                        db_id: existing.id.to_string(),
                        conflict: false,
                    });
                }
            } else {
                // New beat
                preview.additions.push(SyncAddition {
                    id: format!("beat-{}", source_id),
                    item_type: "beat".to_string(),
                    title: truncate_string(&new_beat.content, 50),
                    parent_title: parent_scene_name,
                });
            }
        }
    }

    Ok(preview)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    match s.char_indices().nth(max_len) {
        Some((end, _)) => format!("{}...", &s[..end]),
        None => s.to_string(),
    }
}

/// Markdown has no durable source IDs. Match labels within parents, retain
/// database identities, and reject ambiguous moves/renames before writing.
fn reconcile_markdown(
    conn: &Connection,
    project: &crate::models::Project,
    parsed: &mut crate::parsers::ParsedPlottr,
) -> Result<(), String> {
    if !matches!(project.source_type, crate::models::SourceType::Markdown) {
        return Ok(());
    }
    let mut chapters =
        db::get_all_chapters_including_archived(conn, &project.id).map_err(|e| e.to_string())?;
    let mut scenes = db::get_all_project_scenes(conn, &project.id).map_err(|e| e.to_string())?;
    let mut beats = db::get_all_project_beats(conn, &project.id).map_err(|e| e.to_string())?;
    let unlinked: HashSet<Uuid> = chapters
        .iter()
        .filter(|c| c.source_id.is_none())
        .map(|c| c.id)
        .chain(
            scenes
                .iter()
                .filter(|s| s.source_id.is_none())
                .map(|s| s.id),
        )
        .chain(beats.iter().filter(|b| b.source_id.is_none()).map(|b| b.id))
        .collect();
    let mut counts = HashMap::new();
    for sid in chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref())
        .chain(scenes.iter().filter_map(|s| s.source_id.as_ref()))
        .chain(beats.iter().filter_map(|b| b.source_id.as_ref()))
    {
        *counts.entry(sid.clone()).or_insert(0) += 1;
    }
    // A completely untagged legacy import first needs an unambiguous mapping.
    // Once matched source rows are tagged, unmatched local rows remain independent.
    let candidate_unlinked = if counts.is_empty() {
        HashSet::new()
    } else {
        unlinked.clone()
    };
    let archived_chapters: HashSet<_> = chapters
        .iter()
        .filter(|c| c.archived)
        .map(|c| c.id)
        .collect();
    let archived_scenes: HashSet<_> = scenes
        .iter()
        .filter(|s| s.archived || archived_chapters.contains(&s.chapter_id))
        .map(|s| s.id)
        .collect();
    // Repair missing and duplicate legacy IDs using database UUIDs. Unmatched
    // local nodes remain unlinked; simply opening sync must not adopt them.
    let mut chapter_backfills = Vec::new();
    let mut scene_backfills = Vec::new();
    let mut beat_backfills = Vec::new();
    for chapter in &mut chapters {
        if chapter.source_id.as_ref().is_none_or(|sid| counts[sid] > 1) {
            let sid = format!("markdown:uuid:chapter:{}", chapter.id);
            chapter.source_id = Some(sid.clone());
            chapter_backfills.push((chapter.id, sid));
        }
    }
    for scene in &mut scenes {
        if scene.source_id.as_ref().is_none_or(|sid| counts[sid] > 1) {
            let sid = format!("markdown:uuid:scene:{}", scene.id);
            scene.source_id = Some(sid.clone());
            scene_backfills.push((scene.id, sid));
        }
    }
    for beat in &mut beats {
        if beat.source_id.as_ref().is_none_or(|sid| counts[sid] > 1) {
            let sid = format!("markdown:uuid:beat:{}", beat.id);
            beat.source_id = Some(sid.clone());
            beat_backfills.push((beat.id, sid));
        }
    }
    let mut reserved: HashSet<String> = chapters
        .iter()
        .filter_map(|c| c.source_id.clone())
        .chain(scenes.iter().filter_map(|s| s.source_id.clone()))
        .chain(beats.iter().filter_map(|b| b.source_id.clone()))
        .collect();
    let labels: Vec<_> = parsed.chapters.iter().map(|c| c.title.as_str()).collect();
    let chapter_ids = match_markdown_labels(
        "chapter",
        "",
        labels.clone(),
        markdown_candidates(
            &labels,
            chapters
                .iter()
                .map(|c| (c.id, c.title.as_str(), c.source_id.as_deref(), c.archived))
                .collect(),
            &candidate_unlinked,
        ),
        &mut reserved,
    )?;
    for (chapter, sid) in parsed.chapters.iter_mut().zip(chapter_ids) {
        chapter.source_id = Some(sid.clone());
        let existing = chapters.iter().find(|c| c.source_id.as_ref() == Some(&sid));
        if existing.is_some_and(|c| c.archived) {
            continue;
        }
        let children: Vec<_> = parsed
            .scenes
            .iter_mut()
            .filter(|s| s.chapter_id == chapter.id)
            .collect();
        let labels: Vec<_> = children.iter().map(|s| s.title.as_str()).collect();
        let scene_ids = match_markdown_labels(
            "scene",
            &sid,
            labels.clone(),
            markdown_candidates(
                &labels,
                scenes
                    .iter()
                    .filter(|s| existing.is_some_and(|c| s.chapter_id == c.id))
                    .map(|s| {
                        (
                            s.id,
                            s.title.as_str(),
                            s.source_id.as_deref(),
                            archived_scenes.contains(&s.id),
                        )
                    })
                    .collect(),
                &candidate_unlinked,
            ),
            &mut reserved,
        )?;
        for (scene, sid) in children.into_iter().zip(scene_ids) {
            scene.source_id = Some(sid.clone());
            let existing = scenes.iter().find(|s| s.source_id.as_ref() == Some(&sid));
            if existing.is_some_and(|s| archived_scenes.contains(&s.id)) {
                continue;
            }
            let children: Vec<_> = parsed
                .beats
                .iter_mut()
                .filter(|b| b.scene_id == scene.id)
                .collect();
            let labels: Vec<_> = children.iter().map(|b| b.content.as_str()).collect();
            let beat_ids = match_markdown_labels(
                "beat",
                &sid,
                labels.clone(),
                markdown_candidates(
                    &labels,
                    beats
                        .iter()
                        .filter(|b| existing.is_some_and(|s| b.scene_id == s.id))
                        .map(|b| {
                            (
                                b.id,
                                b.content.as_str(),
                                b.source_id.as_deref(),
                                archived_scenes.contains(&b.scene_id),
                            )
                        })
                        .collect(),
                    &candidate_unlinked,
                ),
                &mut reserved,
            )?;
            for (beat, sid) in children.into_iter().zip(beat_ids) {
                beat.source_id = Some(sid);
            }
        }
    }
    let matched: HashSet<_> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref())
        .chain(parsed.scenes.iter().filter_map(|s| s.source_id.as_ref()))
        .chain(parsed.beats.iter().filter_map(|b| b.source_id.as_ref()))
        .cloned()
        .collect();
    // Archived source nodes retain their identities but are not previewed,
    // modified, or resurrected by sync/reimport, including their descendants.
    parsed.chapters.retain(|c| {
        !chapters
            .iter()
            .any(|old| old.source_id == c.source_id && old.archived)
    });
    let visible_chapters: HashSet<_> = parsed.chapters.iter().map(|c| c.id).collect();
    parsed.scenes.retain(|s| {
        visible_chapters.contains(&s.chapter_id)
            && !scenes
                .iter()
                .any(|old| old.source_id == s.source_id && archived_scenes.contains(&old.id))
    });
    let visible_scenes: HashSet<_> = parsed.scenes.iter().map(|s| s.id).collect();
    parsed
        .beats
        .retain(|b| visible_scenes.contains(&b.scene_id));
    // A label disappearing under one parent and appearing under another may
    // be a move. Require the writer to align that move in Kindling first.
    for old in scenes
        .iter()
        .filter(|s| !unlinked.contains(&s.id) && !archived_scenes.contains(&s.id))
    {
        if !parsed.scenes.iter().any(|s| s.source_id == old.source_id)
            && parsed.scenes.iter().any(|s| {
                s.title == old.title
                    && !scenes.iter().any(|existing| {
                        existing.source_id == s.source_id && !unlinked.contains(&existing.id)
                    })
            })
        {
            return Err("Ambiguous Markdown scene move. Move the existing scene in kindling to match the source before syncing. No changes were applied.".into());
        }
    }
    for old in beats
        .iter()
        .filter(|b| !unlinked.contains(&b.id) && !archived_scenes.contains(&b.scene_id))
    {
        if !parsed.beats.iter().any(|b| b.source_id == old.source_id)
            && parsed.beats.iter().any(|b| {
                b.content == old.content
                    && !beats.iter().any(|existing| {
                        existing.source_id == b.source_id && !unlinked.contains(&existing.id)
                    })
            })
        {
            return Err("Ambiguous Markdown beat move. Align the beat's scene in kindling and the source before syncing. No changes were applied.".into());
        }
    }
    // Metadata repair is atomic and only follows successful matching. Cancelling
    // preview never applies outline/prose changes; repaired identities persist.
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    for (id, sid) in chapter_backfills
        .into_iter()
        .filter(|(id, sid)| !unlinked.contains(id) || matched.contains(sid))
    {
        db::update_chapter_source_id(&tx, &id, &sid).map_err(|e| e.to_string())?;
    }
    for (id, sid) in scene_backfills
        .into_iter()
        .filter(|(id, sid)| !unlinked.contains(id) || matched.contains(sid))
    {
        db::update_scene_source_id(&tx, &id, &sid).map_err(|e| e.to_string())?;
    }
    for (id, sid) in beat_backfills
        .into_iter()
        .filter(|(id, sid)| !unlinked.contains(id) || matched.contains(sid))
    {
        db::update_beat_source_id(&tx, &id, &sid).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Local-only rows are candidates only when the source explicitly gains their
/// unique label. Archived rows absent from the source do not block additions.
fn markdown_candidates<'a>(
    labels: &[&str],
    nodes: Vec<(Uuid, &'a str, Option<&'a str>, bool)>,
    unlinked: &HashSet<Uuid>,
) -> Vec<(&'a str, Option<&'a str>)> {
    // Linked live identities take precedence, then linked archived identities.
    // Only when there is no linked identity may a unique local row be adopted.
    let priority = |id: &Uuid, archived: bool| (unlinked.contains(id), archived);
    nodes
        .iter()
        .filter(|(id, label, _, archived)| {
            let best = nodes
                .iter()
                .filter(|(_, name, _, _)| name == label)
                .map(|(other, _, _, hidden)| priority(other, *hidden))
                .min()
                .unwrap();
            if priority(id, *archived) != best {
                return false;
            }
            if best != (false, false) && !labels.contains(label) {
                return false;
            }
            if *archived {
                // All-archived matches are ignored downstream. Pick one stable
                // representative instead of making irrelevant ambiguity block sync.
                return nodes
                    .iter()
                    .filter(|(other, name, _, hidden)| {
                        name == label && priority(other, *hidden) == best
                    })
                    .map(|(other, _, _, _)| other)
                    .min()
                    == Some(id);
            }
            true
        })
        .map(|(_, label, sid, _)| (*label, *sid))
        .collect()
}

fn match_markdown_labels(
    kind: &str,
    parent: &str,
    incoming: Vec<&str>,
    existing: Vec<(&str, Option<&str>)>,
    used: &mut HashSet<String>,
) -> Result<Vec<String>, String> {
    let ambiguous = || {
        format!(
        "Ambiguous Markdown {kind} matching. Give sibling chapters/scenes unique titles and beats unique text, and align renamed labels in kindling and the source before syncing. No changes were applied."
    )
    };
    let mut seen = HashSet::new();
    if incoming.iter().any(|label| !seen.insert(*label)) {
        return Err(ambiguous());
    }
    seen.clear();
    if existing.iter().any(|(label, _)| !seen.insert(*label)) {
        return Err(ambiguous());
    }
    let added = incoming
        .iter()
        .any(|label| !existing.iter().any(|(old, _)| old == label));
    let removed = existing.iter().any(|(old, _)| !incoming.contains(old));
    if added && removed {
        return Err(ambiguous());
    }
    used.extend(
        existing
            .iter()
            .filter_map(|(_, sid)| sid.map(str::to_owned)),
    );
    incoming
        .into_iter()
        .map(|label| {
            if let Some((_, sid)) = existing.iter().find(|(old, _)| *old == label) {
                sid.map(str::to_owned).ok_or_else(ambiguous)
            } else {
                // Length-prefix the parent so arbitrary user text cannot collide.
                let base = format!("markdown:label:{kind}:{}:{parent}:{label}", parent.len());
                let mut sid = base.clone();
                let mut suffix = 0;
                while !used.insert(sid.clone()) {
                    suffix += 1;
                    sid = format!("{base}:{suffix}");
                }
                Ok(sid)
            }
        })
        .collect()
}

#[tauri::command]
pub async fn apply_sync(
    project_id: String,
    accepted_change_ids: Vec<String>,
    accepted_addition_ids: Vec<String>,
    // Conflicts the writer saw and left unticked, to settle as "keep kindling".
    kept_conflict_ids: Option<Vec<String>>,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<ReimportSummary, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    // Accepted prose replaces the writer's text, so apply takes a restorable
    // copy first, once it has checked the changes are current: a stale or
    // refused apply writes no snapshot.
    let mut snapshot = None;
    let result = apply_reviewed_sync(
        &conn,
        project_uuid,
        accepted_change_ids,
        accepted_addition_ids,
        &kept_conflict_ids.unwrap_or_default(),
        &mut |conn| {
            snapshot = Some(super::snapshot::write_snapshot(
                conn,
                &app_handle,
                &project_uuid,
                super::CreateSnapshotOptions {
                    name: "Before sync".to_string(),
                    description: Some(
                        "Automatic snapshot created before sync replaced prose".into(),
                    ),
                    trigger_type: SnapshotTrigger::Auto,
                },
            )?);
            Ok(())
        },
    );
    if let (Err(_), Some(snapshot)) = (&result, snapshot) {
        // The writes failed and rolled back, so the copy is only clutter.
        if let Err(e) = super::snapshot::remove_snapshot(&conn, &snapshot.id) {
            eprintln!("Warning: failed to remove unused pre-sync snapshot: {e}");
        }
    }
    result
}

#[cfg(test)]
fn apply_sync_with_connection(
    conn: &Connection,
    project_uuid: Uuid,
    accepted_change_ids: Vec<String>,
    accepted_addition_ids: Vec<String>,
) -> Result<ReimportSummary, String> {
    apply_reviewed_sync(
        conn,
        project_uuid,
        accepted_change_ids,
        accepted_addition_ids,
        &[],
        &mut |_| Ok(()),
    )
}

fn apply_reviewed_sync(
    conn: &Connection,
    project_uuid: Uuid,
    accepted_change_ids: Vec<String>,
    accepted_addition_ids: Vec<String>,
    kept_conflict_ids: &[String],
    before_prose: &mut dyn FnMut(&Connection) -> Result<(), String>,
) -> Result<ReimportSummary, String> {
    // Get the existing project to find source path and type
    let project = db::get_project(conn, &project_uuid)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Project not found".to_string())?;

    let source_path = project
        .source_path
        .as_ref()
        .ok_or_else(|| "Project has no source path for sync".to_string())?;

    // Re-parse the source file based on source type
    let mut parsed = match project.source_type {
        crate::models::SourceType::Plottr => {
            parse_plottr_file(source_path).map_err(|e| e.to_string())?
        }
        crate::models::SourceType::YWriter => {
            let yw_parsed = parse_ywriter_file(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: yw_parsed.project,
                chapters: yw_parsed.chapters,
                scenes: yw_parsed.scenes,
                beats: yw_parsed.beats,
                characters: yw_parsed.characters,
                locations: yw_parsed.locations,
                scene_character_refs: yw_parsed.scene_character_refs,
                scene_location_refs: yw_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Scrivener => {
            return Err(
                "Scrivener import has been deprecated. This project cannot be synced.".to_string(),
            );
        }
        crate::models::SourceType::Longform => {
            let lf_parsed = parse_longform_index(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: lf_parsed.project,
                chapters: lf_parsed.chapters,
                scenes: lf_parsed.scenes,
                beats: lf_parsed.beats,
                characters: lf_parsed.characters,
                locations: lf_parsed.locations,
                scene_character_refs: lf_parsed.scene_character_refs,
                scene_location_refs: lf_parsed.scene_location_refs,
            }
        }
        crate::models::SourceType::Markdown => {
            let md_parsed = parse_markdown_outline(source_path).map_err(|e| e.to_string())?;
            crate::parsers::ParsedPlottr {
                project: md_parsed.project,
                chapters: md_parsed.chapters,
                scenes: md_parsed.scenes,
                beats: md_parsed.beats,
                characters: Vec::new(),
                locations: Vec::new(),
                scene_character_refs: Vec::new(),
                scene_location_refs: Vec::new(),
            }
        }
        crate::models::SourceType::NovelWriter => {
            let parsed = super::novelwriter_sync::load(conn, &project)?;
            return super::novelwriter_sync::apply(
                conn,
                &project,
                &parsed,
                &accepted_change_ids,
                &accepted_addition_ids,
                kept_conflict_ids,
                before_prose,
            );
        }
        crate::models::SourceType::Blank => {
            return Err("Blank projects have no source to reimport".to_string());
        }
    };
    reconcile_markdown(conn, &project, &mut parsed)?;

    let accepted_set: HashSet<String> = accepted_change_ids.into_iter().collect();
    let accepted_additions_set: HashSet<String> = accepted_addition_ids.into_iter().collect();

    let mut summary = ReimportSummary {
        chapters_added: 0,
        chapters_updated: 0,
        scenes_added: 0,
        scenes_updated: 0,
        beats_added: 0,
        beats_updated: 0,
        prose_preserved: 0,
        prose_updated: 0,
    };

    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;

    // Get existing DB data
    let db_chapters = db::get_chapters(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let chapter_source_to_db: HashMap<String, Chapter> = db_chapters
        .into_iter()
        .filter_map(|c| c.source_id.clone().map(|sid| (sid, c)))
        .collect();

    // Process chapters - always add new ones, only update if change is accepted
    for new_chapter in &parsed.chapters {
        if let Some(source_id) = &new_chapter.source_id {
            if let Some(existing) = chapter_source_to_db.get(source_id) {
                if existing.locked {
                    continue;
                }
                // Check if user accepted the title change
                let change_id = format!("chapter-title-{}", existing.id);
                if accepted_set.contains(&change_id) && existing.title != new_chapter.title {
                    db::update_chapter(conn, &existing.id, &new_chapter.title, existing.position)
                        .map_err(|e| e.to_string())?;
                    summary.chapters_updated += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("chapter-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let earlier = earlier_source_siblings(
                        parsed.chapters.iter().map(|c| (c.position, &c.source_id)),
                        new_chapter.position,
                    );
                    let position =
                        open_sync_slot(&tx, SyncSiblings::Chapters, &project_uuid, &earlier)?;
                    let chapter_to_insert = Chapter {
                        id: new_chapter.id,
                        project_id: project_uuid,
                        title: new_chapter.title.clone(),
                        position,
                        source_id: new_chapter.source_id.clone(),
                        archived: false,
                        locked: false,
                        is_part: new_chapter.is_part,
                        synopsis: None,
                        planning_status: PlanningStatus::Fixed,
                    };
                    db::insert_chapter(&tx, &chapter_to_insert).map_err(|e| e.to_string())?;
                    summary.chapters_added += 1;
                }
            }
        }
    }

    // Refresh chapter map after inserts
    let db_chapters = db::get_chapters(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let chapter_source_to_db: HashMap<String, &Chapter> = db_chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (sid.clone(), c)))
        .collect();

    // Build map from parsed chapter ID to parsed chapter source_id
    let parsed_chapter_id_to_source: HashMap<Uuid, String> = parsed
        .chapters
        .iter()
        .filter_map(|c| c.source_id.as_ref().map(|sid| (c.id, sid.clone())))
        .collect();

    // Get existing scenes
    let db_scenes = db::get_all_project_scenes(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let scene_source_to_db: HashMap<String, Scene> = db_scenes
        .into_iter()
        .filter_map(|s| s.source_id.clone().map(|sid| (sid, s)))
        .collect();

    // Process scenes
    for new_scene in &parsed.scenes {
        if let Some(source_id) = &new_scene.source_id {
            // Find the DB chapter this scene belongs to
            let parsed_chapter_source_id =
                parsed_chapter_id_to_source
                    .get(&new_scene.chapter_id)
                    .ok_or_else(|| "Scene references unknown chapter".to_string())?;
            let Some(db_chapter) = chapter_source_to_db.get(parsed_chapter_source_id) else {
                if !accepted_additions_set.contains(&format!("scene-{source_id}")) {
                    continue;
                }
                return Err(
                    "Could not find DB chapter for scene; accept its parent chapter first".into(),
                );
            };

            if db_chapter.locked {
                continue;
            }
            if let Some(existing) = scene_source_to_db.get(source_id).filter(|s| {
                !matches!(project.source_type, crate::models::SourceType::Markdown)
                    || s.chapter_id == db_chapter.id
            }) {
                if db::is_scene_locked(&tx, &existing.id).map_err(|e| e.to_string())? {
                    continue;
                }
                // Check which changes user accepted
                let mut new_title = existing.title.clone();
                let mut new_synopsis = existing.synopsis.clone();
                let mut updated = false;

                let title_change_id = format!("scene-title-{}", existing.id);
                if accepted_set.contains(&title_change_id) && existing.title != new_scene.title {
                    new_title = new_scene.title.clone();
                    updated = true;
                }

                let synopsis_change_id = format!("scene-synopsis-{}", existing.id);
                if accepted_set.contains(&synopsis_change_id)
                    && existing.synopsis != new_scene.synopsis
                {
                    new_synopsis = new_scene.synopsis.clone();
                    updated = true;
                }

                if updated {
                    db::update_scene(
                        conn,
                        &existing.id,
                        &new_title,
                        new_synopsis.as_deref(),
                        existing.position,
                        &existing.scene_type,
                        &existing.scene_status,
                    )
                    .map_err(|e| e.to_string())?;
                    summary.scenes_updated += 1;
                }
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("scene-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let earlier = earlier_source_siblings(
                        parsed
                            .scenes
                            .iter()
                            .filter(|s| s.chapter_id == new_scene.chapter_id)
                            .map(|s| (s.position, &s.source_id)),
                        new_scene.position,
                    );
                    let position =
                        open_sync_slot(&tx, SyncSiblings::Scenes, &db_chapter.id, &earlier)?;
                    let scene_to_insert = Scene {
                        id: new_scene.id,
                        chapter_id: db_chapter.id,
                        title: new_scene.title.clone(),
                        synopsis: new_scene.synopsis.clone(),
                        prose: None,
                        position,
                        source_id: new_scene.source_id.clone(),
                        archived: false,
                        locked: false,
                        scene_type: new_scene.scene_type,
                        scene_status: new_scene.scene_status,
                        planning_status: PlanningStatus::Fixed,
                        editor_mode: EditorMode::Beat,
                    };
                    db::insert_scene(&tx, &scene_to_insert).map_err(|e| e.to_string())?;
                    summary.scenes_added += 1;
                }
            }
        }
    }

    // Refresh scene map after inserts
    let db_scenes = db::get_all_project_scenes(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let scene_source_to_db: HashMap<String, &Scene> = db_scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (sid.clone(), s)))
        .collect();

    // Build map from parsed scene ID to parsed scene source_id
    let parsed_scene_id_to_source: HashMap<Uuid, String> = parsed
        .scenes
        .iter()
        .filter_map(|s| s.source_id.as_ref().map(|sid| (s.id, sid.clone())))
        .collect();

    // Get existing beats
    let db_beats = db::get_all_project_beats(&tx, &project_uuid).map_err(|e| e.to_string())?;
    let beat_source_to_db: HashMap<String, Beat> = db_beats
        .into_iter()
        .filter_map(|b| b.source_id.clone().map(|sid| (sid, b)))
        .collect();

    let parsed_scene_chapters: HashMap<Uuid, Uuid> =
        parsed.scenes.iter().map(|s| (s.id, s.chapter_id)).collect();
    // Process beats
    for new_beat in &parsed.beats {
        if let Some(source_id) = &new_beat.source_id {
            // A new scene may have been skipped because its chapter was locked.
            // Skip its descendants before requiring a database scene to exist.
            if parsed_scene_chapters
                .get(&new_beat.scene_id)
                .and_then(|id| parsed_chapter_id_to_source.get(id))
                .and_then(|sid| chapter_source_to_db.get(sid))
                .is_some_and(|c| c.locked)
            {
                continue;
            }
            // Find the DB scene this beat belongs to
            let parsed_scene_source_id = parsed_scene_id_to_source
                .get(&new_beat.scene_id)
                .ok_or_else(|| "Beat references unknown scene".to_string())?;
            let Some(db_scene) = scene_source_to_db.get(parsed_scene_source_id) else {
                if !accepted_additions_set.contains(&format!("beat-{source_id}")) {
                    continue;
                }
                return Err(
                    "Could not find DB scene for beat; accept its parent scene first".into(),
                );
            };

            if db::is_scene_locked(&tx, &db_scene.id).map_err(|e| e.to_string())? {
                continue;
            }
            if let Some(existing) = beat_source_to_db.get(source_id).filter(|b| {
                !matches!(project.source_type, crate::models::SourceType::Markdown)
                    || b.scene_id == db_scene.id
            }) {
                if db::is_scene_locked(&tx, &existing.scene_id).map_err(|e| e.to_string())? {
                    continue;
                }
                // Check if user accepted the content change
                let change_id = format!("beat-content-{}", existing.id);
                if accepted_set.contains(&change_id) && existing.content != new_beat.content {
                    db::update_beat(&tx, &existing.id, &new_beat.content, existing.position)
                        .map_err(|e| e.to_string())?;
                    summary.beats_updated += 1;
                }
                if existing.prose.is_some() {
                    summary.prose_preserved += 1;
                }
            } else {
                // Check if user accepted this addition
                let addition_id = format!("beat-{}", source_id);
                if accepted_additions_set.contains(&addition_id) {
                    let earlier = earlier_source_siblings(
                        parsed
                            .beats
                            .iter()
                            .filter(|b| b.scene_id == new_beat.scene_id)
                            .map(|b| (b.position, &b.source_id)),
                        new_beat.position,
                    );
                    let position =
                        open_sync_slot(&tx, SyncSiblings::Beats, &db_scene.id, &earlier)?;
                    let beat_to_insert = Beat {
                        id: new_beat.id,
                        scene_id: db_scene.id,
                        content: new_beat.content.clone(),
                        prose: None,
                        position,
                        source_id: new_beat.source_id.clone(),
                    };
                    db::insert_beat(&tx, &beat_to_insert).map_err(|e| e.to_string())?;
                    summary.beats_added += 1;
                }
            }
        }
    }

    if matches!(project.source_type, crate::models::SourceType::Markdown) {
        order_markdown_additions(&tx, project_uuid, &parsed, &accepted_additions_set)?;
    }
    db::update_project_modified(&tx, &project_uuid).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(summary)
}

/// A sibling list that sync inserts into.
#[derive(Clone, Copy)]
pub(super) enum SyncSiblings {
    Chapters,
    Scenes,
    Beats,
}

impl SyncSiblings {
    /// Table and parent column. Fixed strings, never user input.
    fn table(self) -> (&'static str, &'static str) {
        match self {
            Self::Chapters => ("chapters", "project_id"),
            Self::Scenes => ("scenes", "chapter_id"),
            Self::Beats => ("beats", "scene_id"),
        }
    }
}

/// Source ids of the siblings that precede `position` in the source, nearest
/// first.
pub(super) fn earlier_source_siblings<'a>(
    siblings: impl Iterator<Item = (i32, &'a Option<String>)>,
    position: i32,
) -> Vec<&'a str> {
    let mut earlier: Vec<_> = siblings
        .filter(|(p, _)| *p < position)
        .filter_map(|(p, sid)| sid.as_deref().map(|sid| (p, sid)))
        .collect();
    earlier.sort_by_key(|(p, _)| std::cmp::Reverse(*p));
    earlier.into_iter().map(|(_, sid)| sid).collect()
}

/// Make room for a sync addition and return the position it should take.
///
/// The new row goes straight after its nearest earlier source sibling that is
/// already under this parent (a matched row or one added earlier in the same
/// sync), or first if there is none. Every row at or after that slot moves
/// down one, so positions stay unique and the writer's existing order is kept
/// whatever order the additions are applied in.
///
/// Locked rows shift too. A lock stops a row being moved relative to the
/// writer's other rows; making room for a new row keeps that order, as
/// inserting a chapter from the outline does. Placing the addition after
/// locked rows instead would put it where the source does not have it.
pub(super) fn open_sync_slot(
    conn: &Connection,
    siblings: SyncSiblings,
    parent_id: &Uuid,
    earlier_sources: &[&str],
) -> Result<i32, String> {
    use rusqlite::OptionalExtension;
    let (table, parent) = siblings.table();
    let mut slot = 0;
    for source_id in earlier_sources {
        let anchor: Option<i32> = conn
            .query_row(
                &format!(
                    "SELECT MAX(position) FROM {table} WHERE {parent} = ?1 AND source_id = ?2"
                ),
                rusqlite::params![parent_id.to_string(), source_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();
        if let Some(anchor) = anchor {
            slot = anchor + 1;
            break;
        }
    }
    conn.execute(
        &format!(
            "UPDATE {table} SET position = position + 1 WHERE {parent} = ?1 AND position >= ?2"
        ),
        rusqlite::params![parent_id.to_string(), slot],
    )
    .map_err(|e| e.to_string())?;
    Ok(slot)
}

/// Only groups receiving accepted additions are reordered. Rejected additions
/// occupy no slots; matched nodes keep their identity and prose.
fn order_markdown_additions(
    conn: &Connection,
    project_id: Uuid,
    parsed: &crate::parsers::ParsedPlottr,
    accepted: &HashSet<String>,
) -> Result<(), String> {
    fn group(
        conn: &Connection,
        table: &str,
        kind: &str,
        source: Vec<&str>,
        existing: Vec<(Uuid, Option<String>)>,
        accepted: &HashSet<String>,
    ) -> Result<(), String> {
        if !source
            .iter()
            .any(|sid| accepted.contains(&format!("{kind}-{sid}")))
        {
            return Ok(());
        }
        let source_order: Vec<Uuid> = source
            .iter()
            .filter_map(|sid| {
                existing
                    .iter()
                    .find(|(_, id)| id.as_deref() == Some(sid))
                    .map(|(id, _)| *id)
            })
            .collect();
        let source_ids: HashSet<_> = source_order.iter().copied().collect();
        let mut source_order = source_order.into_iter();
        let mut ordered = Vec::new();
        for (id, _) in &existing {
            if source_ids.contains(id) {
                if let Some(next) = source_order.next() {
                    ordered.push(next);
                }
            } else {
                ordered.push(*id);
            }
        }
        ordered.extend(source_order);
        for (position, id) in ordered.iter().enumerate() {
            conn.execute(
                &format!("UPDATE {table} SET position = ?1 WHERE id = ?2"),
                rusqlite::params![position as i32, id.to_string()],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    let chapters = db::get_chapters(conn, &project_id).map_err(|e| e.to_string())?;
    group(
        conn,
        "chapters",
        "chapter",
        parsed
            .chapters
            .iter()
            .filter_map(|c| c.source_id.as_deref())
            .collect(),
        chapters
            .iter()
            .map(|c| (c.id, c.source_id.clone()))
            .collect(),
        accepted,
    )?;
    for chapter in &parsed.chapters {
        let Some(db_chapter) = chapters.iter().find(|c| c.source_id == chapter.source_id) else {
            continue;
        };
        let scenes = db::get_scenes(conn, &db_chapter.id).map_err(|e| e.to_string())?;
        let source_scenes: Vec<_> = parsed
            .scenes
            .iter()
            .filter(|s| s.chapter_id == chapter.id)
            .collect();
        group(
            conn,
            "scenes",
            "scene",
            source_scenes
                .iter()
                .filter_map(|s| s.source_id.as_deref())
                .collect(),
            scenes.iter().map(|s| (s.id, s.source_id.clone())).collect(),
            accepted,
        )?;
        for scene in source_scenes {
            let Some(db_scene) = scenes.iter().find(|s| s.source_id == scene.source_id) else {
                continue;
            };
            let beats = db::get_beats(conn, &db_scene.id).map_err(|e| e.to_string())?;
            group(
                conn,
                "beats",
                "beat",
                parsed
                    .beats
                    .iter()
                    .filter(|b| b.scene_id == scene.id)
                    .filter_map(|b| b.source_id.as_deref())
                    .collect(),
                beats.iter().map(|b| (b.id, b.source_id.clone())).collect(),
                accepted,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::truncate_string;

    #[test]
    fn unicode_preview_titles_do_not_panic_or_poison_the_connection_lock() {
        let lock = std::sync::Mutex::new(());
        {
            let _guard = lock.lock().unwrap();
            assert_eq!(truncate_string(&"界".repeat(17), 50), "界".repeat(17));
            assert_eq!(
                truncate_string(&"界".repeat(51), 50),
                format!("{}...", "界".repeat(50))
            );
            assert_eq!(truncate_string("😀é界", 2), "😀é...");
            assert_eq!(truncate_string("界", 0), "...");
        }
        assert!(lock.lock().is_ok());
    }
    #[test]
    fn test_truncate_string_shorter_than_limit() {
        let input = "Short text";
        assert_eq!(truncate_string(input, 20), "Short text");
    }

    #[test]
    fn test_truncate_string_exact_limit() {
        let input = "Exact";
        assert_eq!(truncate_string(input, 5), "Exact");
    }

    #[test]
    fn test_truncate_string_longer_than_limit() {
        let input = "This is a longer string";
        assert_eq!(truncate_string(input, 4), "This...");
    }
}

#[cfg(test)]
mod source_regression_tests {
    use super::*;
    use crate::models::Project;

    fn match_markdown_labels(
        kind: &str,
        parent: &str,
        incoming: Vec<&str>,
        existing: Vec<(&str, Option<&str>)>,
    ) -> Result<Vec<String>, String> {
        super::match_markdown_labels(kind, parent, incoming, existing, &mut HashSet::new())
    }

    fn markdown_fixture(
        path: &std::path::Path,
        source: &str,
    ) -> (Connection, crate::parsers::ParsedMarkdown) {
        std::fs::write(path, source).unwrap();
        let parsed = parse_markdown_outline(path).unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        db::insert_project(&conn, &parsed.project).unwrap();
        for c in &parsed.chapters {
            db::insert_chapter(&conn, c).unwrap();
        }
        for scene in &parsed.scenes {
            db::insert_scene(&conn, scene).unwrap();
        }
        for beat in &parsed.beats {
            db::insert_beat(&conn, beat).unwrap();
        }
        (conn, parsed)
    }

    #[test]
    fn local_markdown_nodes_remain_unlinked_and_do_not_block_additions() {
        for selective in [true, false] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("outline.md");
            let (conn, parsed) = markdown_fixture(&path, "# A\n## A\n- A\n");
            let chapter = Chapter::new(parsed.project.id, "Local chapter".into(), 1);
            db::insert_chapter(&conn, &chapter).unwrap();
            let mut scene = Scene::new(parsed.chapters[0].id, "Local scene".into(), None, 1);
            scene.prose = Some("Local page draft".into());
            db::insert_scene(&conn, &scene).unwrap();
            let mut beat = Beat::new(parsed.scenes[0].id, "Local beat".into(), 1);
            beat.prose = Some("Local beat draft".into());
            db::insert_beat(&conn, &beat).unwrap();
            std::fs::write(&path, "# X\n## X\n- X\n# A\n## X\n- X\n## A\n- X\n- A\n").unwrap();
            let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
            if selective {
                apply_sync_with_connection(
                    &conn,
                    parsed.project.id,
                    vec![],
                    preview.additions.iter().map(|a| a.id.clone()).collect(),
                )
                .unwrap();
            } else {
                reimport_project_with_connection(&conn, parsed.project.id).unwrap();
            }
            assert!(db::get_chapter_by_id(&conn, &chapter.id)
                .unwrap()
                .unwrap()
                .source_id
                .is_none());
            let current = db::get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
            assert!(current.source_id.is_none());
            assert_eq!(current.prose, scene.prose);
            let current = db::get_beat(&conn, &beat.id).unwrap().unwrap();
            assert!(current.source_id.is_none());
            assert_eq!(current.prose, beat.prose);
        }
    }

    #[test]
    fn archived_local_namesakes_do_not_make_live_local_adoption_ambiguous() {
        let live = Uuid::new_v4();
        let archived = Uuid::new_v4();
        let unlinked = HashSet::from([live, archived]);
        let candidates = markdown_candidates(
            &["Arrival"],
            vec![
                (live, "Arrival", Some("live-id"), false),
                (archived, "Arrival", Some("archived-id"), true),
            ],
            &unlinked,
        );
        assert_eq!(candidates, vec![("Arrival", Some("live-id"))]);
        let candidates = markdown_candidates(
            &["Arrival"],
            vec![
                (live, "Arrival", Some("first-archive"), true),
                (archived, "Arrival", Some("second-archive"), true),
            ],
            &unlinked,
        );
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn archived_markdown_nodes_are_not_recreated_or_adopted_by_same_named_local_nodes() {
        for archive_chapter in [true, false] {
            for source_keeps_archived in [true, false] {
                let temp = tempfile::tempdir().unwrap();
                let path = temp.path().join("outline.md");
                let (conn, parsed) = markdown_fixture(&path, "# A\n## A\n- A\n# B\n## B\n- B\n");
                db::update_scene_prose(&conn, &parsed.scenes[0].id, "Archived page draft").unwrap();
                db::update_beat_prose(&conn, &parsed.beats[0].id, "Archived beat draft").unwrap();
                if archive_chapter {
                    db::archive_chapter(&conn, &parsed.chapters[0].id).unwrap();
                    let mut archived_copy = Chapter::new(parsed.project.id, "A".into(), 3)
                        .with_source_id(Some("archived-copy".into()));
                    archived_copy.archived = true;
                    db::insert_chapter(&conn, &archived_copy).unwrap();
                    db::insert_chapter(&conn, &Chapter::new(parsed.project.id, "A".into(), 2))
                        .unwrap();
                } else {
                    db::archive_scene(&conn, &parsed.scenes[0].id).unwrap();
                    let mut archived_copy = Scene::new(parsed.chapters[0].id, "A".into(), None, 3)
                        .with_source_id(Some("archived-copy".into()));
                    archived_copy.archived = true;
                    db::insert_scene(&conn, &archived_copy).unwrap();
                    db::insert_scene(
                        &conn,
                        &Scene::new(parsed.chapters[0].id, "A".into(), None, 1),
                    )
                    .unwrap();
                }
                let source = if source_keeps_archived {
                    "# A\n## A\n- A\n- Hidden new beat\n# B\n## B\n- B\n# X\n## X\n- X\n"
                } else if archive_chapter {
                    "# B\n## B\n- B\n# X\n## X\n- X\n"
                } else {
                    "# A\n# B\n## B\n- B\n# X\n## X\n- X\n"
                };
                std::fs::write(&path, source).unwrap();
                let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
                assert_eq!(preview.additions.len(), 3);
                assert!(preview.additions.iter().all(|a| a.title == "X"));
                apply_sync_with_connection(
                    &conn,
                    parsed.project.id,
                    vec![],
                    preview.additions.iter().map(|a| a.id.clone()).collect(),
                )
                .unwrap();
                reimport_project_with_connection(&conn, parsed.project.id).unwrap();
                assert!(get_sync_preview_with_connection(&conn, parsed.project.id)
                    .unwrap()
                    .additions
                    .is_empty());
                let archived_scene = db::get_scene_by_id(&conn, &parsed.scenes[0].id)
                    .unwrap()
                    .unwrap();
                assert_eq!(archived_scene.prose.as_deref(), Some("Archived page draft"));
                assert_eq!(db::get_beats(&conn, &archived_scene.id).unwrap().len(), 1);
                assert_eq!(
                    db::get_beat(&conn, &parsed.beats[0].id)
                        .unwrap()
                        .unwrap()
                        .prose
                        .as_deref(),
                    Some("Archived beat draft")
                );
                if archive_chapter {
                    assert!(
                        db::get_chapter_by_id(&conn, &parsed.chapters[0].id)
                            .unwrap()
                            .unwrap()
                            .archived
                    );
                } else {
                    assert!(archived_scene.archived);
                }
            }
        }
    }

    #[test]
    fn removing_a_source_node_does_not_mistake_an_existing_namesake_for_a_move() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("outline.md");
        let (conn, parsed) = markdown_fixture(
            &path,
            "# A\n## Shared\n- Shared\n# B\n## Shared\n- Shared\n",
        );
        std::fs::write(&path, "# A\n# B\n## Shared\n- Shared\n").unwrap();
        assert!(get_sync_preview_with_connection(&conn, parsed.project.id)
            .unwrap()
            .additions
            .is_empty());
    }

    #[test]
    fn source_scene_and_beat_moves_require_explicit_alignment_without_duplicate_drafts() {
        for source in [
            "# A\n# B\n## B\n- B\n## Arrival\n- Knock\n",
            "# A\n## Arrival\n# B\n## B\n- B\n- Knock\n",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("outline.md");
            let (conn, parsed) =
                markdown_fixture(&path, "# A\n## Arrival\n- Knock\n# B\n## B\n- B\n");
            db::update_beat_prose(&conn, &parsed.beats[0].id, "Draft stays here").unwrap();
            std::fs::write(&path, source).unwrap();
            assert!(get_sync_preview_with_connection(&conn, parsed.project.id)
                .unwrap_err()
                .contains("move"));
            assert!(apply_sync_with_connection(&conn, parsed.project.id, vec![], vec![]).is_err());
            assert!(reimport_project_with_connection(&conn, parsed.project.id).is_err());
            assert_eq!(
                db::get_all_project_scenes(&conn, &parsed.project.id)
                    .unwrap()
                    .len(),
                2
            );
            let beat = db::get_beat(&conn, &parsed.beats[0].id).unwrap().unwrap();
            assert_eq!(beat.scene_id, parsed.scenes[0].id);
            assert_eq!(beat.prose.as_deref(), Some("Draft stays here"));
        }
    }

    #[test]
    fn duplicate_legacy_markdown_source_ids_are_repaired_atomically() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("outline.md");
        let (conn, parsed) =
            markdown_fixture(&path, "# A\n## A\n- A\n- B\n## B\n- C\n# B\n## C\n- D\n");
        for table in ["chapters", "scenes", "beats"] {
            conn.execute(
                &format!("UPDATE {table} SET source_id = 'legacy-duplicate'"),
                [],
            )
            .unwrap();
        }
        db::update_beat_prose(&conn, &parsed.beats[0].id, "Legacy prose").unwrap();
        let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
        assert!(preview.additions.is_empty());
        assert!(preview.changes.is_empty());
        reimport_project_with_connection(&conn, parsed.project.id).unwrap();
        let chapters = db::get_chapters(&conn, &parsed.project.id).unwrap();
        let scenes = db::get_all_project_scenes(&conn, &parsed.project.id).unwrap();
        let beats = db::get_all_project_beats(&conn, &parsed.project.id).unwrap();
        let ids: HashSet<_> = chapters
            .iter()
            .filter_map(|c| c.source_id.as_ref())
            .chain(scenes.iter().filter_map(|s| s.source_id.as_ref()))
            .chain(beats.iter().filter_map(|b| b.source_id.as_ref()))
            .collect();
        assert_eq!(ids.len(), chapters.len() + scenes.len() + beats.len());
        assert_eq!(
            db::get_beat(&conn, &parsed.beats[0].id)
                .unwrap()
                .unwrap()
                .prose
                .as_deref(),
            Some("Legacy prose")
        );
    }

    #[test]
    fn partial_markdown_additions_exclude_rejected_siblings_and_ancestors() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("outline.md");
        let (conn, parsed) =
            markdown_fixture(&path, "# A\n## A\n- A\n- B\n## B\n- A\n# B\n## B\n- B\n");
        std::fs::write(&path, "# X\n## X\n- X\n# A\n## X\n- X\n## A\n- X\n- A\n- Y\n- B\n## Y\n- Y\n## B\n- A\n# Y\n## Y\n- Y\n# B\n## B\n- B\n").unwrap();
        let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
        let additions = preview
            .additions
            .iter()
            .filter(|a| a.title == "Y")
            .map(|a| a.id.clone())
            .collect();
        apply_sync_with_connection(&conn, parsed.project.id, vec![], additions).unwrap();
        let chapters = db::get_chapters(&conn, &parsed.project.id).unwrap();
        assert_eq!(
            chapters
                .iter()
                .map(|c| c.title.as_str())
                .collect::<Vec<_>>(),
            ["A", "Y", "B"]
        );
        let scenes = db::get_scenes(&conn, &chapters[0].id).unwrap();
        assert_eq!(
            scenes.iter().map(|s| s.title.as_str()).collect::<Vec<_>>(),
            ["A", "Y", "B"]
        );
        let beats = db::get_beats(&conn, &scenes[0].id).unwrap();
        assert_eq!(
            beats.iter().map(|b| b.content.as_str()).collect::<Vec<_>>(),
            ["A", "Y", "B"]
        );
    }

    #[test]
    fn new_markdown_scene_cannot_reuse_the_id_of_a_scene_moved_to_another_chapter() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("outline.md");
        let (conn, parsed) = markdown_fixture(&path, "# C1\n## Existing\n- Old\n# C2\n## B\n- B\n");
        std::fs::write(
            &path,
            "# C1\n## A\n- A\n## Existing\n- Old\n# C2\n## B\n- B\n",
        )
        .unwrap();
        reimport_project_with_connection(&conn, parsed.project.id).unwrap();
        let original = db::get_scenes(&conn, &parsed.chapters[0].id)
            .unwrap()
            .remove(0);
        db::update_scene_prose(&conn, &original.id, "Moved scene sentinel").unwrap();
        db::move_scene_to_chapter(&conn, &original.id, &parsed.chapters[1].id, 1).unwrap();
        std::fs::write(
            &path,
            "# C1\n## Existing\n- Old\n# C2\n## B\n- B\n## A\n- A\n",
        )
        .unwrap();
        reimport_project_with_connection(&conn, parsed.project.id).unwrap();
        std::fs::write(
            &path,
            "# C1\n## A\n- New A\n## Existing\n- Old\n# C2\n## B\n- B\n## A\n- A\n",
        )
        .unwrap();
        let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
        assert_eq!(preview.additions.len(), 2);
        apply_sync_with_connection(
            &conn,
            parsed.project.id,
            vec![],
            preview.additions.iter().map(|a| a.id.clone()).collect(),
        )
        .unwrap();
        let scenes = db::get_all_project_scenes(&conn, &parsed.project.id).unwrap();
        let old = scenes.iter().find(|s| s.id == original.id).unwrap();
        assert_eq!(old.chapter_id, parsed.chapters[1].id);
        assert_eq!(old.prose.as_deref(), Some("Moved scene sentinel"));
        let new = scenes
            .iter()
            .find(|s| s.title == "A" && s.chapter_id == parsed.chapters[0].id)
            .unwrap();
        assert_ne!(new.source_id, old.source_id);
        assert_ne!(new.id, old.id);
        assert_eq!(new.prose, None);
    }

    #[test]
    fn markdown_insertions_keep_scene_and_beat_prose_attached() {
        for (selective, legacy) in [(true, false), (false, false), (true, true), (false, true)] {
            let temp = tempfile::tempdir().unwrap();
            let path = temp.path().join("outline.md");
            let original =
                "# Act One\n## Arrival\n- Knock\n- Answer\n# Act Two\n## Departure\n- Leave\n";
            std::fs::write(&path, original).unwrap();
            let parsed = parse_markdown_outline(&path).unwrap();
            let conn = Connection::open_in_memory().unwrap();
            db::initialize_schema(&conn).unwrap();
            db::insert_project(&conn, &parsed.project).unwrap();
            for c in &parsed.chapters {
                db::insert_chapter(&conn, c).unwrap();
            }
            for s in &parsed.scenes {
                db::insert_scene(&conn, s).unwrap();
                db::update_scene_prose(&conn, &s.id, &format!("Page {}", s.title)).unwrap();
            }
            for b in &parsed.beats {
                db::insert_beat(&conn, b).unwrap();
                db::update_beat_prose(&conn, &b.id, &format!("Prose {}", b.content)).unwrap();
            }
            if legacy {
                for table in ["chapters", "scenes", "beats"] {
                    conn.execute(&format!("UPDATE {table} SET source_id = NULL"), [])
                        .unwrap();
                }
                // Failed matching must not leave a partially backfilled project.
                std::fs::write(&path, original.replace("Knock", "Unresolved rename")).unwrap();
                assert!(get_sync_preview_with_connection(&conn, parsed.project.id).is_err());
                for chapter in db::get_chapters(&conn, &parsed.project.id).unwrap() {
                    assert!(chapter.source_id.is_none());
                }
                for scene in db::get_all_project_scenes(&conn, &parsed.project.id).unwrap() {
                    assert!(scene.source_id.is_none());
                }
                for beat in db::get_all_project_beats(&conn, &parsed.project.id).unwrap() {
                    assert!(beat.source_id.is_none());
                }
            }
            let changed = "# Prologue\n## Before\n- Start\n# Act One\n## New scene\n- New beat\n## Arrival\n- Before knock\n- Knock\n- Answer\n# Act Two\n## Departure\n- Leave\n";
            std::fs::write(&path, changed).unwrap();
            let preview = get_sync_preview_with_connection(&conn, parsed.project.id).unwrap();
            assert!(preview.changes.is_empty());
            assert_eq!(preview.additions.len(), 6);
            if selective {
                apply_sync_with_connection(
                    &conn,
                    parsed.project.id,
                    vec![],
                    preview.additions.iter().map(|a| a.id.clone()).collect(),
                )
                .unwrap();
            } else {
                reimport_project_with_connection(&conn, parsed.project.id).unwrap();
            }
            let actual_chapters = db::get_chapters(&conn, &parsed.project.id).unwrap();
            assert_eq!(
                actual_chapters
                    .iter()
                    .map(|c| c.title.as_str())
                    .collect::<Vec<_>>(),
                ["Prologue", "Act One", "Act Two"]
            );
            assert_eq!(
                actual_chapters
                    .iter()
                    .map(|c| c.position)
                    .collect::<Vec<_>>(),
                [0, 1, 2]
            );
            let actual_scenes = db::get_scenes(&conn, &parsed.chapters[0].id).unwrap();
            assert_eq!(
                actual_scenes
                    .iter()
                    .map(|s| s.title.as_str())
                    .collect::<Vec<_>>(),
                ["New scene", "Arrival"]
            );
            assert_eq!(
                actual_scenes.iter().map(|s| s.position).collect::<Vec<_>>(),
                [0, 1]
            );
            let actual_beats = db::get_beats(&conn, &parsed.scenes[0].id).unwrap();
            assert_eq!(
                actual_beats
                    .iter()
                    .map(|b| b.content.as_str())
                    .collect::<Vec<_>>(),
                ["Before knock", "Knock", "Answer"]
            );
            assert_eq!(
                actual_beats.iter().map(|b| b.position).collect::<Vec<_>>(),
                [0, 1, 2]
            );
            for scene in &parsed.scenes {
                let current = db::get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
                assert_eq!(current.title, scene.title);
                assert_eq!(current.prose, Some(format!("Page {}", scene.title)));
                assert_eq!(current.chapter_id, scene.chapter_id);
            }
            for beat in &parsed.beats {
                let current = db::get_beat(&conn, &beat.id).unwrap().unwrap();
                assert_eq!(current.content, beat.content);
                assert_eq!(current.prose, Some(format!("Prose {}", beat.content)));
                assert_eq!(current.scene_id, beat.scene_id);
            }
            assert!(get_sync_preview_with_connection(&conn, parsed.project.id)
                .unwrap()
                .additions
                .is_empty());
            std::fs::write(&path, changed.replace("Arrival", "Renamed")).unwrap();
            assert!(get_sync_preview_with_connection(&conn, parsed.project.id).is_err());
            assert!(apply_sync_with_connection(&conn, parsed.project.id, vec![], vec![]).is_err());
            assert!(reimport_project_with_connection(&conn, parsed.project.id).is_err());
            assert_eq!(
                db::get_scene_by_id(&conn, &parsed.scenes[0].id)
                    .unwrap()
                    .unwrap()
                    .title,
                "Arrival"
            );
        }
    }

    #[test]
    fn markdown_additions_never_reuse_a_renamed_nodes_id() {
        for kind in ["chapter", "scene", "beat"] {
            let first = match_markdown_labels(kind, "parent", vec!["A"], vec![])
                .unwrap()
                .remove(0);
            // The writer explicitly aligns A -> B, retaining the node's identity.
            let next =
                match_markdown_labels(kind, "parent", vec!["A", "B"], vec![("B", Some(&first))])
                    .unwrap();
            assert_ne!(next[0], first);
            assert_eq!(next[1], first);
            // A source label resembling our numeric collision suffix is also safe.
            let third = match_markdown_labels(
                kind,
                "parent",
                vec!["A:1", "A", "B"],
                vec![("A", Some(&next[0])), ("B", Some(&first))],
            )
            .unwrap();
            assert_eq!(third.iter().collect::<HashSet<_>>().len(), 3);
            assert_eq!(&third[1..], &next);
        }
    }

    #[test]
    fn markdown_duplicate_labels_require_resolution() {
        assert!(match_markdown_labels("scene", "chapter", vec!["Same", "Same"], vec![]).is_err());
        assert!(match_markdown_labels(
            "beat",
            "scene",
            vec!["Beat"],
            vec![("Beat", Some("a")), ("Beat", Some("b"))]
        )
        .is_err());
    }

    fn exercise(project: Project, chapters: Vec<Chapter>, scenes: Vec<Scene>, beats: Vec<Beat>) {
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        db::insert_project(&conn, &project).unwrap();
        for c in &chapters {
            db::insert_chapter(&conn, c).unwrap();
        }
        for s in &scenes {
            db::insert_scene(&conn, s).unwrap();
        }
        for b in &beats {
            db::insert_beat(&conn, b).unwrap();
        }
        for s in &scenes {
            db::update_scene_prose(&conn, &s.id, "<p>Local page prose</p>").unwrap();
        }
        for b in &beats {
            db::update_beat_prose(&conn, &b.id, "<p>Local beat prose</p>").unwrap();
        }
        let preview = get_sync_preview_with_connection(&conn, project.id).unwrap();
        assert!(preview.changes.iter().all(|c| c.field != "prose"));
        let chapter = chapters.iter().find(|c| c.source_id.is_some()).unwrap();
        conn.execute(
            "UPDATE chapters SET title = 'Local title' WHERE id = ?1",
            [chapter.id.to_string()],
        )
        .unwrap();
        if matches!(project.source_type, crate::models::SourceType::Markdown) {
            assert!(get_sync_preview_with_connection(&conn, project.id)
                .err()
                .unwrap()
                .contains("Ambiguous Markdown"));
            // Explicitly align the renamed heading before retrying an ambiguous sync.
            conn.execute(
                "UPDATE chapters SET title = ?1 WHERE id = ?2",
                [&chapter.title, &chapter.id.to_string()],
            )
            .unwrap();
        } else {
            let preview = get_sync_preview_with_connection(&conn, project.id).unwrap();
            let title = preview
                .changes
                .iter()
                .find(|c| c.db_id == chapter.id.to_string())
                .unwrap();
            assert_eq!(
                (&title.field, &title.current_value, &title.new_value),
                (&"title".into(), &"Local title".into(), &chapter.title)
            );
            let summary =
                apply_sync_with_connection(&conn, project.id, vec![title.id.clone()], vec![])
                    .unwrap();
            assert_eq!(summary.chapters_updated, 1);
            assert_eq!(summary.prose_updated, 0);
            // The serialized contract for old sources has no new summary field.
            assert!(serde_json::to_value(&summary)
                .unwrap()
                .get("prose_updated")
                .is_some());
        }
        reimport_project_with_connection(&conn, project.id).unwrap();
        for s in db::get_all_project_scenes(&conn, &project.id).unwrap() {
            assert_eq!(s.prose.as_deref(), Some("<p>Local page prose</p>"));
        }
        for b in db::get_all_project_beats(&conn, &project.id).unwrap() {
            assert_eq!(b.prose.as_deref(), Some("<p>Local beat prose</p>"));
        }
    }
    #[test]
    fn existing_sources_never_diff_or_overwrite_prose() {
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let p = parse_plottr_file(fixtures.join("hamlet.pltr")).unwrap();
        exercise(p.project, p.chapters, p.scenes, p.beats);
        let p = parse_ywriter_file(fixtures.join("hamlet.yw7")).unwrap();
        exercise(p.project, p.chapters, p.scenes, p.beats);
        let p = parse_markdown_outline(fixtures.join("hamlet.md")).unwrap();
        exercise(p.project, p.chapters, p.scenes, p.beats);
        let temp = tempfile::tempdir().unwrap();
        let index = temp.path().join("Index.md");
        std::fs::write(&index, "---\nlongform:\n  format: scenes\n  title: Test\n  sceneFolder: /\n  scenes:\n    - Opening\n---\n").unwrap();
        std::fs::write(
            temp.path().join("Opening.md"),
            "Incoming prose.\n\n<!-- kindling: beats -->\n- First beat\n",
        )
        .unwrap();
        let p = parse_longform_index(&index).unwrap();
        exercise(p.project, p.chapters, p.scenes, p.beats);
    }
    #[test]
    fn novelwriter_all_three_commands_dispatch_and_reimport_preserves_prose() {
        let (original, project) = crate::parsers::novelwriter::tests::fixture();
        let temp = tempfile::tempdir().unwrap();
        crate::parsers::novelwriter::export_novelwriter_project(
            &original,
            &project.id,
            temp.path(),
            &Default::default(),
        )
        .unwrap();
        let parsed = crate::parsers::novelwriter::parse_novelwriter_project(temp.path()).unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        super::super::import::insert_novelwriter(&conn, &parsed).unwrap();
        let project = parsed.project;
        let beat = &parsed.beats[0];
        let scene = &parsed.scenes[0];
        db::update_beat_prose(&conn, &beat.id, "<p>Local draft</p>").unwrap();
        // A kindling-only edit is not an incoming change.
        assert!(get_sync_preview_with_connection(&conn, project.id)
            .unwrap()
            .changes
            .is_empty());
        // Edit the same prose and scene title in novelWriter too.
        let file = temp
            .path()
            .join("content")
            .join(format!("{}.md", scene.source_id.as_deref().unwrap()));
        let text = std::fs::read_to_string(&file).unwrap();
        std::fs::write(
            &file,
            text.replace("door", "gate")
                .replace("### Arrival", "### Departure"),
        )
        .unwrap();
        conn.execute(
            "UPDATE scenes SET title = 'Landing' WHERE id = ?1",
            [scene.id.to_string()],
        )
        .unwrap();
        let preview = get_sync_preview_with_connection(&conn, project.id).unwrap();
        assert_eq!(preview.changes.len(), 2, "{:?}", preview.changes);
        assert!(preview.changes.iter().all(|c| c.conflict));
        // Reimport applies nothing that needs the writer's choice.
        reimport_project_with_connection(&conn, project.id).unwrap();
        assert_eq!(
            db::get_beats(&conn, &beat.scene_id).unwrap()[0]
                .prose
                .as_deref(),
            Some("<p>Local draft</p>")
        );
        assert_eq!(
            db::get_all_project_scenes(&conn, &project.id).unwrap()[0].title,
            "Landing"
        );
        let prose = preview.changes.iter().find(|c| c.field == "prose").unwrap();
        let title = preview.changes.iter().find(|c| c.field == "title").unwrap();
        // The pre-sync snapshot hook runs once, and only when current changes
        // replace prose; a stale or failed apply never reaches it.
        let snapshots = std::cell::Cell::new(0);
        let apply = |accepted: &[String], kept: &[String]| {
            apply_reviewed_sync(
                &conn,
                project.id,
                accepted.to_vec(),
                vec![],
                kept,
                &mut |_| {
                    snapshots.set(snapshots.get() + 1);
                    Ok(())
                },
            )
        };
        let result = apply(
            std::slice::from_ref(&prose.id),
            std::slice::from_ref(&title.id),
        );
        assert_eq!(result.unwrap().prose_updated, 1);
        assert_eq!(snapshots.get(), 1, "the pre-sync snapshot is taken");
        // The title conflict the writer saw unticked was settled in kindling's favour.
        assert!(get_sync_preview_with_connection(&conn, project.id)
            .unwrap()
            .changes
            .is_empty());
        assert_eq!(
            db::get_all_project_scenes(&conn, &project.id).unwrap()[0].title,
            "Landing"
        );
        // A stale retry is refused before a snapshot is ever written.
        let stale = apply(std::slice::from_ref(&prose.id), &[]);
        assert!(stale.err().unwrap().contains("changed after you reviewed"));
        assert_eq!(snapshots.get(), 1);
        // An unreadable folder fails before the snapshot too.
        std::fs::remove_file(temp.path().join("nwProject.nwx")).unwrap();
        assert!(apply(std::slice::from_ref(&prose.id), &[]).is_err());
        assert_eq!(snapshots.get(), 1);
    }
}

#[cfg(test)]
mod release_sync_tests {
    use super::*;
    use crate::models::{SceneStatus, SceneType};

    fn fixture() -> (Connection, crate::parsers::ParsedPlottr) {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.pltr");
        let p = parse_plottr_file(path).unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        db::insert_project(&conn, &p.project).unwrap();
        for c in &p.chapters {
            db::insert_chapter(&conn, c).unwrap();
        }
        for s in &p.scenes {
            db::insert_scene(&conn, s).unwrap();
        }
        for b in &p.beats {
            db::insert_beat(&conn, b).unwrap();
        }
        (conn, p)
    }

    #[test]
    fn accepted_title_preserves_unselected_scene_metadata_and_order() {
        let (conn, p) = fixture();
        let scene = &p.scenes[0];
        db::update_scene(
            &conn,
            &scene.id,
            "Local title",
            Some("Local synopsis"),
            42,
            &SceneType::Notes,
            &SceneStatus::Final,
        )
        .unwrap();
        let summary = apply_sync_with_connection(
            &conn,
            p.project.id,
            vec![format!("scene-title-{}", scene.id)],
            vec![],
        )
        .unwrap();
        let actual = db::get_scene_by_id(&conn, &scene.id).unwrap().unwrap();
        assert_eq!(summary.scenes_updated, 1);
        assert_eq!(actual.title, scene.title);
        assert_eq!(actual.synopsis.as_deref(), Some("Local synopsis"));
        assert_eq!(actual.position, 42);
        assert_eq!(actual.scene_type, SceneType::Notes);
        assert_eq!(actual.scene_status, SceneStatus::Final);
    }

    #[test]
    fn chapter_lock_is_checked_in_preview_apply_and_reimport() {
        let (conn, p) = fixture();
        let beat = &p.beats[0];
        let scene = p.scenes.iter().find(|s| s.id == beat.scene_id).unwrap();
        db::update_beat(&conn, &beat.id, "Protected local prompt", beat.position).unwrap();
        let id = format!("beat-content-{}", beat.id);
        assert!(get_sync_preview_with_connection(&conn, p.project.id)
            .unwrap()
            .changes
            .iter()
            .any(|c| c.id == id));
        db::lock_chapter(&conn, &scene.chapter_id).unwrap();
        assert!(!get_sync_preview_with_connection(&conn, p.project.id)
            .unwrap()
            .changes
            .iter()
            .any(|c| c.id == id));
        apply_sync_with_connection(&conn, p.project.id, vec![id.clone()], vec![]).unwrap();
        reimport_project_with_connection(&conn, p.project.id).unwrap();
        assert_eq!(
            db::get_beats(&conn, &scene.id)
                .unwrap()
                .iter()
                .find(|b| b.id == beat.id)
                .unwrap()
                .content,
            "Protected local prompt"
        );
        db::unlock_chapter(&conn, &scene.chapter_id).unwrap();
        apply_sync_with_connection(&conn, p.project.id, vec![id], vec![]).unwrap();
        assert_eq!(
            db::get_beats(&conn, &scene.id)
                .unwrap()
                .iter()
                .find(|b| b.id == beat.id)
                .unwrap()
                .content,
            beat.content
        );
    }

    #[test]
    fn locked_new_scene_does_not_abort_unrelated_sync_changes() {
        for selective in [false, true] {
            let (conn, p) = fixture();
            let removed = p
                .scenes
                .iter()
                .find(|s| p.beats.iter().any(|b| b.scene_id == s.id))
                .unwrap();
            let kept = p
                .chapters
                .iter()
                .find(|c| c.id != removed.chapter_id)
                .unwrap();
            db::delete_scene(&conn, &removed.id).unwrap();
            db::update_chapter(&conn, &kept.id, "Local title", kept.position).unwrap();
            let preview = get_sync_preview_with_connection(&conn, p.project.id).unwrap();
            assert!(preview.additions.iter().any(|a| a.item_type == "beat"));
            db::lock_chapter(&conn, &removed.chapter_id).unwrap();
            if selective {
                apply_sync_with_connection(
                    &conn,
                    p.project.id,
                    vec![format!("chapter-title-{}", kept.id)],
                    preview.additions.iter().map(|a| a.id.clone()).collect(),
                )
                .unwrap();
            } else {
                reimport_project_with_connection(&conn, p.project.id).unwrap();
            }
            assert_eq!(
                db::get_chapter_by_id(&conn, &kept.id)
                    .unwrap()
                    .unwrap()
                    .title,
                kept.title
            );
            assert!(db::get_scene_by_id(&conn, &removed.id).unwrap().is_none());
        }
    }

    /// Hamlet as a library imported before the source gained some items: the
    /// listed chapters, scenes and beats (by source id) are missing, and the
    /// remaining siblings are numbered contiguously as import numbered them.
    fn partial_hamlet_import(
        missing_chapters: &[&str],
        missing_scenes: &[&str],
        missing_beats: &[&str],
    ) -> (Connection, crate::parsers::ParsedPlottr) {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.pltr");
        let p = parse_plottr_file(path).unwrap();
        let conn = Connection::open_in_memory().unwrap();
        db::initialize_schema(&conn).unwrap();
        db::insert_project(&conn, &p.project).unwrap();
        let missing = |list: &[&str], sid: &Option<String>| {
            sid.as_deref().is_some_and(|sid| list.contains(&sid))
        };
        let mut chapters: Vec<_> = p
            .chapters
            .iter()
            .filter(|c| !missing(missing_chapters, &c.source_id))
            .collect();
        chapters.sort_by_key(|c| c.position);
        for (position, chapter) in chapters.iter().enumerate() {
            let mut old = (*chapter).clone();
            old.position = position as i32;
            db::insert_chapter(&conn, &old).unwrap();
            let mut scenes: Vec<_> = p
                .scenes
                .iter()
                .filter(|s| s.chapter_id == chapter.id && !missing(missing_scenes, &s.source_id))
                .collect();
            scenes.sort_by_key(|s| s.position);
            for (position, scene) in scenes.iter().enumerate() {
                let mut old = (*scene).clone();
                old.position = position as i32;
                db::insert_scene(&conn, &old).unwrap();
                let mut beats: Vec<_> = p
                    .beats
                    .iter()
                    .filter(|b| b.scene_id == scene.id && !missing(missing_beats, &b.source_id))
                    .collect();
                beats.sort_by_key(|b| b.position);
                for (position, beat) in beats.iter().enumerate() {
                    let mut old = (*beat).clone();
                    old.position = position as i32;
                    db::insert_beat(&conn, &old).unwrap();
                }
            }
        }
        (conn, p)
    }

    fn accept_all_additions(conn: &Connection, project_id: Uuid) -> ReimportSummary {
        let preview = get_sync_preview_with_connection(conn, project_id).unwrap();
        let additions = preview.additions.iter().map(|a| a.id.clone()).collect();
        apply_sync_with_connection(conn, project_id, vec![], additions).unwrap()
    }

    fn set_scene_position(conn: &Connection, scene: &Scene, position: i32) {
        db::update_scene(
            conn,
            &scene.id,
            &scene.title,
            scene.synopsis.as_deref(),
            position,
            &scene.scene_type,
            &scene.scene_status,
        )
        .unwrap();
    }

    fn scene_titles_and_positions(conn: &Connection, chapter_id: &Uuid) -> (Vec<String>, Vec<i32>) {
        db::get_scenes(conn, chapter_id)
            .unwrap()
            .into_iter()
            .map(|s| (s.title, s.position))
            .unzip()
    }

    const ACT1_SUMMARY: &str = "Hamlet learns the truth from the ghost of his father";
    const GUARDS: &str = "The guards see a ghost";
    const ANNOUNCEMENT: &str = "Claudius makes an announcement & Hamlet laments";
    const LAERTES: &str = "Laertes leaves for France";
    const AFTER_GHOST: &str = "Hamlet goes after the ghost of his father";
    const SPEAKS: &str = "Hamlet speaks with his father's ghost";

    #[test]
    fn plottr_card_additions_keep_the_writers_scene_order() {
        // A 1.2 project, imported before titled cards without a description
        // were imported: the five Summary-line scenes are missing.
        let (conn, p) = partial_hamlet_import(&[], &["11", "12", "13", "22", "24"], &[]);
        // The writer swaps the first two scenes of Act 1.
        let act1 = &p.chapters[0];
        let local = db::get_scenes(&conn, &act1.id).unwrap();
        set_scene_position(&conn, &local[0], 1);
        set_scene_position(&conn, &local[1], 0);

        assert_eq!(accept_all_additions(&conn, p.project.id).scenes_added, 5);

        // The new card is strictly first, where Plottr has it; the writer's
        // order follows untouched and no two scenes share a position.
        let (titles, positions) = scene_titles_and_positions(&conn, &act1.id);
        assert_eq!(
            titles,
            [
                ACT1_SUMMARY,
                ANNOUNCEMENT,
                GUARDS,
                LAERTES,
                AFTER_GHOST,
                SPEAKS
            ]
        );
        assert_eq!(positions, [0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn several_scene_additions_to_one_chapter_follow_their_source_neighbours() {
        // Act 1 in Plottr: SUMMARY GUARDS ANNOUNCEMENT LAERTES AFTER_GHOST SPEAKS.
        // The library lacks SUMMARY, ANNOUNCEMENT and SPEAKS.
        let (conn, p) = partial_hamlet_import(&[], &["12", "2", "5"], &[]);
        let act1 = &p.chapters[0];
        let local = db::get_scenes(&conn, &act1.id).unwrap();
        // The writer moves LAERTES after AFTER_GHOST, then locks GUARDS.
        let (guards, laertes, after_ghost) = (&local[0], &local[1], &local[2]);
        set_scene_position(&conn, after_ghost, 1);
        set_scene_position(&conn, laertes, 2);
        db::lock_scene(&conn, &guards.id).unwrap();

        assert_eq!(accept_all_additions(&conn, p.project.id).scenes_added, 3);

        // Each addition lands straight after its nearest earlier Plottr
        // sibling in the library (or first), and the locked scene shifts to
        // make room without changing its place among the writer's scenes.
        let (titles, positions) = scene_titles_and_positions(&conn, &act1.id);
        assert_eq!(
            titles,
            [
                ACT1_SUMMARY,
                GUARDS,
                ANNOUNCEMENT,
                AFTER_GHOST,
                SPEAKS,
                LAERTES
            ]
        );
        assert_eq!(positions, [0, 1, 2, 3, 4, 5]);
        assert!(db::is_scene_locked(&conn, &guards.id).unwrap());
    }

    #[test]
    fn chapter_and_beat_additions_take_their_own_positions() {
        let parsed = parse_plottr_file(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/hamlet.pltr"),
        )
        .unwrap();
        let act2 = parsed.chapters.iter().find(|c| c.title == "Act 2").unwrap();
        let act2_source = act2.source_id.clone().unwrap();
        let guards_source = parsed
            .scenes
            .iter()
            .find(|s| s.title == GUARDS)
            .and_then(|s| s.source_id.clone())
            .unwrap();
        let first_guards_beat = format!("{guards_source}:0");
        let (conn, p) = partial_hamlet_import(&[&act2_source], &[], &[&first_guards_beat]);

        let summary = accept_all_additions(&conn, p.project.id);
        assert_eq!(summary.chapters_added, 1);
        assert!(summary.beats_added > 1, "Act 2 beats plus the guards beat");

        let chapters = db::get_chapters(&conn, &p.project.id).unwrap();
        let titles: Vec<_> = chapters.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, ["Act 1", "Act 2", "Act 3", "Act 4", "Act 5"]);
        let positions: Vec<_> = chapters.iter().map(|c| c.position).collect();
        assert_eq!(positions, [0, 1, 2, 3, 4]);

        let guards = p.scenes.iter().find(|s| s.title == GUARDS).unwrap();
        let beats = db::get_beats(&conn, &guards.id).unwrap();
        let sources: Vec<_> = beats.iter().map(|b| b.source_id.clone().unwrap()).collect();
        let expected: Vec<_> = (0..beats.len())
            .map(|i| format!("{guards_source}:{i}"))
            .collect();
        assert_eq!(sources, expected);
        let positions: Vec<_> = beats.iter().map(|b| b.position).collect();
        assert_eq!(positions, (0..beats.len() as i32).collect::<Vec<_>>());
    }

    #[test]
    fn rejected_parent_additions_do_not_abort_unrelated_plottr_changes() {
        for reject_chapter in [true, false] {
            let (conn, p) = fixture();
            let kept = &p.chapters[0];
            let removed = p
                .scenes
                .iter()
                .find(|s| s.chapter_id != kept.id && p.beats.iter().any(|b| b.scene_id == s.id))
                .unwrap();
            if reject_chapter {
                db::delete_chapter(&conn, &removed.chapter_id).unwrap();
            } else {
                db::delete_scene(&conn, &removed.id).unwrap();
            }
            db::update_chapter(&conn, &kept.id, "Local title", kept.position).unwrap();
            let summary = apply_sync_with_connection(
                &conn,
                p.project.id,
                vec![format!("chapter-title-{}", kept.id)],
                vec![],
            )
            .unwrap();
            assert_eq!(summary.chapters_updated, 1);
            assert_eq!(summary.scenes_added, 0);
            assert_eq!(summary.beats_added, 0);
            assert!(db::get_scene_by_id(&conn, &removed.id).unwrap().is_none());
        }
    }
}
