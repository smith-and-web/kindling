use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryNote {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub content: String,
    /// Tags stored as JSON array, e.g. ["character", "plot"]
    pub tags: Vec<String>,
    pub position: i32,
    pub created_at: String,
}
