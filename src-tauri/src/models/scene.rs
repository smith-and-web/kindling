use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SceneType {
    #[default]
    Normal,
    Notes,
    Todo,
    Unused,
}

impl SceneType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SceneType::Normal => "normal",
            SceneType::Notes => "notes",
            SceneType::Todo => "todo",
            SceneType::Unused => "unused",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "notes" => SceneType::Notes,
            "todo" => SceneType::Todo,
            "unused" => SceneType::Unused,
            _ => SceneType::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum SceneStatus {
    #[default]
    Draft,
    Revised,
    Final,
}

impl SceneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SceneStatus::Draft => "draft",
            SceneStatus::Revised => "revised",
            SceneStatus::Final => "final",
        }
    }

    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "revised" => SceneStatus::Revised,
            "final" => SceneStatus::Final,
            _ => SceneStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub title: String,
    pub synopsis: Option<String>,
    pub prose: Option<String>,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub scene_type: SceneType,
    #[serde(default)]
    pub scene_status: SceneStatus,
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
            source_id: None,
            archived: false,
            locked: false,
            scene_type: SceneType::Normal,
            scene_status: SceneStatus::Draft,
        }
    }

    pub fn with_source_id(mut self, source_id: Option<String>) -> Self {
        self.source_id = source_id;
        self
    }
}
