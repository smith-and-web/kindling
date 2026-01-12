use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub position: i32,
}

impl Chapter {
    pub fn new(project_id: Uuid, title: String, position: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            title,
            position,
        }
    }
}
