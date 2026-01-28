use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Beat, Chapter, Character, Location, Project, ReferenceItem, Scene, SceneReferenceState,
};

/// Trigger type for snapshot creation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotTrigger {
    Manual,
    Export,
    Auto,
}

impl SnapshotTrigger {
    pub fn as_str(&self) -> &'static str {
        match self {
            SnapshotTrigger::Manual => "manual",
            SnapshotTrigger::Export => "export",
            SnapshotTrigger::Auto => "auto",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "manual" => Some(SnapshotTrigger::Manual),
            "export" => Some(SnapshotTrigger::Export),
            "auto" => Some(SnapshotTrigger::Auto),
            _ => None,
        }
    }
}

/// Metadata about a snapshot, stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: SnapshotTrigger,
    pub created_at: String,
    pub file_path: String,
    pub file_size: i64,
    pub uncompressed_size: Option<i64>,
    pub chapter_count: i32,
    pub scene_count: i32,
    pub beat_count: i32,
    pub word_count: Option<i32>,
    pub schema_version: i32,
}

impl SnapshotMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project_id: Uuid,
        name: String,
        description: Option<String>,
        trigger_type: SnapshotTrigger,
        file_path: String,
        file_size: i64,
        uncompressed_size: Option<i64>,
        chapter_count: i32,
        scene_count: i32,
        beat_count: i32,
        word_count: Option<i32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            name,
            description,
            trigger_type,
            created_at: chrono::Utc::now().to_rfc3339(),
            file_path,
            file_size,
            uncompressed_size,
            chapter_count,
            scene_count,
            beat_count,
            word_count,
            schema_version: 1,
        }
    }
}

/// Reference linking a scene to a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneCharacterRef {
    pub scene_id: Uuid,
    pub character_id: Uuid,
}

/// Reference linking a scene to a location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneLocationRef {
    pub scene_id: Uuid,
    pub location_id: Uuid,
}

/// Reference linking a scene to a reference item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneReferenceItemRef {
    pub scene_id: Uuid,
    pub reference_item_id: Uuid,
}

/// The full snapshot data stored in the compressed file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotData {
    pub version: i32,
    pub created_at: String,
    pub project: Project,
    pub chapters: Vec<Chapter>,
    pub scenes: Vec<Scene>,
    pub beats: Vec<Beat>,
    pub characters: Vec<Character>,
    pub locations: Vec<Location>,
    #[serde(default)]
    pub reference_items: Vec<ReferenceItem>,
    pub scene_character_refs: Vec<SceneCharacterRef>,
    pub scene_location_refs: Vec<SceneLocationRef>,
    #[serde(default)]
    pub scene_reference_item_refs: Vec<SceneReferenceItemRef>,
    #[serde(default)]
    pub scene_reference_states: Vec<SceneReferenceState>,
}

impl SnapshotData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        project: Project,
        chapters: Vec<Chapter>,
        scenes: Vec<Scene>,
        beats: Vec<Beat>,
        characters: Vec<Character>,
        locations: Vec<Location>,
        reference_items: Vec<ReferenceItem>,
        scene_character_refs: Vec<SceneCharacterRef>,
        scene_location_refs: Vec<SceneLocationRef>,
        scene_reference_item_refs: Vec<SceneReferenceItemRef>,
        scene_reference_states: Vec<SceneReferenceState>,
    ) -> Self {
        Self {
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            project,
            chapters,
            scenes,
            beats,
            characters,
            locations,
            reference_items,
            scene_character_refs,
            scene_location_refs,
            scene_reference_item_refs,
            scene_reference_states,
        }
    }

    /// Count total words across all prose content
    pub fn word_count(&self) -> i32 {
        let mut count = 0;

        // Count scene prose
        for scene in &self.scenes {
            if let Some(prose) = &scene.prose {
                count += prose.split_whitespace().count();
            }
        }

        // Count beat prose
        for beat in &self.beats {
            if let Some(prose) = &beat.prose {
                count += prose.split_whitespace().count();
            }
        }

        count as i32
    }
}

/// Mode for restoring a snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RestoreMode {
    /// Replace the current project with the snapshot data
    ReplaceCurrent,
    /// Create a new project from the snapshot
    CreateNew,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_trigger_as_str() {
        assert_eq!(SnapshotTrigger::Manual.as_str(), "manual");
        assert_eq!(SnapshotTrigger::Export.as_str(), "export");
        assert_eq!(SnapshotTrigger::Auto.as_str(), "auto");
    }

    #[test]
    fn test_snapshot_trigger_parse() {
        assert_eq!(
            SnapshotTrigger::parse("manual"),
            Some(SnapshotTrigger::Manual)
        );
        assert_eq!(
            SnapshotTrigger::parse("EXPORT"),
            Some(SnapshotTrigger::Export)
        );
        assert_eq!(SnapshotTrigger::parse("Auto"), Some(SnapshotTrigger::Auto));
        assert_eq!(SnapshotTrigger::parse("unknown"), None);
    }

    #[test]
    fn test_snapshot_metadata_new() {
        let project_id = Uuid::new_v4();
        let metadata = SnapshotMetadata::new(
            project_id,
            "Test Snapshot".to_string(),
            Some("A test description".to_string()),
            SnapshotTrigger::Manual,
            "/path/to/snapshot.json.gz".to_string(),
            1024,
            Some(10240),
            5,
            10,
            25,
            Some(5000),
        );

        assert_eq!(metadata.project_id, project_id);
        assert_eq!(metadata.name, "Test Snapshot");
        assert_eq!(metadata.trigger_type, SnapshotTrigger::Manual);
        assert_eq!(metadata.schema_version, 1);
    }
}
