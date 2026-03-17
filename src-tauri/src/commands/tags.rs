use crate::commands::AppState;
use crate::db;
use crate::models::{EntityTag, SavedFilter, Tag};
use tauri::State;
use uuid::Uuid;

// ============================================================================
// Tag CRUD
// ============================================================================

#[tauri::command]
pub async fn get_tags(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Tag>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_tags(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_tag(
    project_id: String,
    name: String,
    color: Option<String>,
    parent_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Tag, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let parent_uuid = parent_id
        .map(|id| Uuid::parse_str(&id).map_err(|e| e.to_string()))
        .transpose()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let existing = db::get_tags(&conn, &project_uuid).map_err(|e| e.to_string())?;
    let next_position = existing.len() as i32;

    let tag = Tag::new(project_uuid, name, color, parent_uuid, next_position);
    db::create_tag(&conn, &tag).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(tag)
}

#[derive(serde::Deserialize)]
pub struct TagUpdate {
    pub name: Option<String>,
    pub color: Option<Option<String>>,
    pub parent_id: Option<Option<String>>,
    pub position: Option<i32>,
}

#[tauri::command]
pub async fn update_tag(
    tag_id: String,
    update: TagUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let parent_uuid = update
        .parent_id
        .map(|opt| {
            opt.map(|id| Uuid::parse_str(&id).map_err(|e| e.to_string()))
                .transpose()
        })
        .transpose()?;

    db::update_tag(
        &conn,
        &uuid,
        update.name.as_deref(),
        update.color.as_ref().map(|c| c.as_deref()),
        parent_uuid.as_ref().map(|p| p.as_ref()),
        update.position,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(
    tag_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_tag(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reorder_tags(
    tag_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuids: Vec<Uuid> = tag_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<Uuid>, String>>()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::reorder_tags(&conn, &uuids).map_err(|e| e.to_string())
}

// ============================================================================
// Entity Tagging
// ============================================================================

#[tauri::command]
pub async fn tag_entity(
    tag_id: String,
    entity_type: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tag_uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::tag_entity(&conn, &tag_uuid, &entity_type, &entity_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn untag_entity(
    tag_id: String,
    entity_type: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tag_uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::untag_entity(&conn, &tag_uuid, &entity_type, &entity_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_entity_tags(
    entity_type: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<Tag>, String> {
    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_entity_tags(&conn, &entity_type, &entity_uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_tag(
    tag_id: String,
    entity_type: String,
    entity_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tag_uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let entity_uuids: Vec<Uuid> = entity_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<Uuid>, String>>()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::bulk_tag(&conn, &tag_uuid, &entity_type, &entity_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn bulk_untag(
    tag_id: String,
    entity_type: String,
    entity_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tag_uuid = Uuid::parse_str(&tag_id).map_err(|e| e.to_string())?;
    let entity_uuids: Vec<Uuid> = entity_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<Uuid>, String>>()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::bulk_untag(&conn, &tag_uuid, &entity_type, &entity_uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_entity_tags(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntityTag>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_entity_tags_for_project(&conn, &uuid).map_err(|e| e.to_string())
}

// ============================================================================
// Filtering
// ============================================================================

#[tauri::command]
pub async fn filter_entities(
    project_id: String,
    entity_type: String,
    filter_json: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let ids = db::filter_entities(&conn, &uuid, &entity_type, &filter_json)
        .map_err(|e| e.to_string())?;
    Ok(ids.iter().map(|id| id.to_string()).collect())
}

#[tauri::command]
pub async fn save_filter(
    project_id: String,
    name: String,
    entity_type: String,
    filter_json: String,
    state: State<'_, AppState>,
) -> Result<SavedFilter, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let existing = db::get_saved_filters(&conn, &project_uuid).map_err(|e| e.to_string())?;
    let position = existing.len() as i32;

    let filter = SavedFilter::new(project_uuid, name, entity_type, filter_json, position);
    db::save_filter(&conn, &filter).map_err(|e| e.to_string())?;

    Ok(filter)
}

#[tauri::command]
pub async fn get_saved_filters(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<SavedFilter>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_saved_filters(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_saved_filter(
    filter_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&filter_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_saved_filter(&conn, &uuid).map_err(|e| e.to_string())
}
