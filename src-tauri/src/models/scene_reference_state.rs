use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneReferenceState {
    pub scene_id: Uuid,
    pub reference_type: String,
    pub reference_id: Uuid,
    pub position: i32,
    pub expanded: bool,
}
