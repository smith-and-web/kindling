use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub project_id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub field_type: String,
    pub options: Option<String>,
    pub default_value: Option<String>,
    pub position: i32,
    pub required: bool,
    pub visible: bool,
    pub created_at: String,
}

impl FieldDefinition {
    pub fn new(
        project_id: Uuid,
        entity_type: String,
        name: String,
        field_type: String,
        position: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            entity_type,
            name,
            field_type,
            options: None,
            default_value: None,
            position,
            required: false,
            visible: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub id: Uuid,
    pub field_definition_id: Uuid,
    pub entity_id: Uuid,
    pub value: Option<String>,
}

impl FieldValue {
    pub fn new(field_definition_id: Uuid, entity_id: Uuid, value: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            field_definition_id,
            entity_id,
            value,
        }
    }
}
