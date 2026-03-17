use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub parent_id: Option<Uuid>,
    pub position: i32,
    pub created_at: String,
}

impl Tag {
    pub fn new(
        project_id: Uuid,
        name: String,
        color: Option<String>,
        parent_id: Option<Uuid>,
        position: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            name,
            color,
            parent_id,
            position,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTag {
    pub tag_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedFilter {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub entity_type: String,
    pub filter_json: String,
    pub position: i32,
}

impl SavedFilter {
    pub fn new(
        project_id: Uuid,
        name: String,
        entity_type: String,
        filter_json: String,
        position: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            name,
            entity_type,
            filter_json,
            position,
        }
    }
}
