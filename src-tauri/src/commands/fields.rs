use crate::commands::AppState;
use crate::db;
use crate::models::{FieldDefinition, FieldValue};
use tauri::State;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FieldDefinitionCreate {
    pub entity_type: String,
    pub name: String,
    pub field_type: String,
    pub options: Option<String>,
    pub default_value: Option<String>,
    pub required: Option<bool>,
    pub visible: Option<bool>,
}

#[derive(serde::Deserialize)]
pub struct FieldDefinitionUpdate {
    pub name: String,
    pub field_type: String,
    pub options: Option<String>,
    pub default_value: Option<String>,
    pub required: Option<bool>,
    pub visible: Option<bool>,
}

#[tauri::command]
pub async fn get_field_definitions(
    project_id: String,
    entity_type: String,
    state: State<'_, AppState>,
) -> Result<Vec<FieldDefinition>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_field_definitions(&conn, &uuid, &entity_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_all_field_definitions(
    project_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FieldDefinition>, String> {
    let uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_all_field_definitions(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_field_definition(
    project_id: String,
    definition: FieldDefinitionCreate,
    state: State<'_, AppState>,
) -> Result<FieldDefinition, String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let existing = db::get_field_definitions(&conn, &project_uuid, &definition.entity_type)
        .map_err(|e| e.to_string())?;
    let next_position = existing.len() as i32;

    let mut def = FieldDefinition::new(
        project_uuid,
        definition.entity_type,
        definition.name,
        definition.field_type,
        next_position,
    );
    def.options = definition.options;
    def.default_value = definition.default_value;
    def.required = definition.required.unwrap_or(false);
    def.visible = definition.visible.unwrap_or(true);

    db::create_field_definition(&conn, &def).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;

    Ok(def)
}

#[tauri::command]
pub async fn update_field_definition(
    project_id: String,
    definition_id: String,
    definition: FieldDefinitionUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let def_uuid = Uuid::parse_str(&definition_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    db::update_field_definition(
        &conn,
        &def_uuid,
        &definition.name,
        &definition.field_type,
        definition.options.as_deref(),
        definition.default_value.as_deref(),
        definition.required.unwrap_or(false),
        definition.visible.unwrap_or(true),
    )
    .map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn delete_field_definition(
    project_id: String,
    definition_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let def_uuid = Uuid::parse_str(&definition_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::delete_field_definition(&conn, &def_uuid).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn reorder_field_definitions(
    project_id: String,
    definition_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let project_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let uuids: Vec<Uuid> = definition_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<Uuid>, String>>()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::reorder_field_definitions(&conn, &uuids).map_err(|e| e.to_string())?;
    db::update_project_modified(&conn, &project_uuid).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_field_values(
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<FieldValue>, String> {
    let uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_field_values(&conn, &uuid).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_field_values_bulk(
    entity_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<FieldValue>, String> {
    let uuids: Vec<Uuid> = entity_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|e| e.to_string()))
        .collect::<Result<Vec<Uuid>, String>>()?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_field_values_bulk(&conn, &uuids).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_field_value(
    field_definition_id: String,
    entity_id: String,
    value: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let def_uuid = Uuid::parse_str(&field_definition_id).map_err(|e| e.to_string())?;
    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::set_field_value(&conn, &def_uuid, &entity_uuid, value.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_field_value(
    field_definition_id: String,
    entity_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let def_uuid = Uuid::parse_str(&field_definition_id).map_err(|e| e.to_string())?;
    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::clear_field_value(&conn, &def_uuid, &entity_uuid).map_err(|e| e.to_string())
}
