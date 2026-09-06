use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A local reading/writing position, independent of project content and snapshots.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionState {
    pub project_id: Uuid,
    pub current_scene_id: Option<Uuid>,
    /// Derived from the scene on read, so moving a scene does not break resume.
    #[serde(default)]
    pub current_chapter_id: Option<Uuid>,
    pub current_beat_id: Option<Uuid>,
    pub cursor_position: Option<u32>,
    pub scroll_position: Option<f64>,
    pub editor_scroll_position: Option<f64>,
    pub last_opened_at: Option<String>,
}
