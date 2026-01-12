use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub title: String,
    pub synopsis: Option<String>,
    pub prose: Option<String>,
    pub position: i32,
}

impl Scene {
    pub fn new(chapter_id: Uuid, title: String, synopsis: Option<String>, position: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            chapter_id,
            title,
            synopsis,
            prose: None,
            position,
        }
    }
}
