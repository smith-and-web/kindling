use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beat {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub content: String,
    pub prose: Option<String>,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

impl Beat {
    pub fn new(scene_id: Uuid, content: String, position: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            scene_id,
            content,
            prose: None,
            position,
            source_id: None,
        }
    }

    pub fn with_source_id(mut self, source_id: Option<String>) -> Self {
        self.source_id = source_id;
        self
    }
}
