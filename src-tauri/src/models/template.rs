use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateScene {
    pub title: String,
    #[serde(default)]
    pub synopsis: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateChapter {
    pub title: String,
    #[serde(default)]
    pub synopsis: Option<String>,
    #[serde(default)]
    pub scenes: Vec<TemplateScene>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePart {
    pub title: String,
    #[serde(default)]
    pub children: Vec<TemplateChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_project_types")]
    pub project_types: Vec<String>,
    pub structure: Vec<TemplatePart>,
    #[serde(default)]
    pub bundled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

fn default_project_types() -> Vec<String> {
    vec!["novel".to_string(), "screenplay".to_string()]
}

impl StoryTemplate {
    pub fn new_user(id: Uuid, name: String, structure: Vec<TemplatePart>) -> Self {
        Self {
            id: id.to_string(),
            name,
            source: None,
            description: None,
            project_types: default_project_types(),
            structure,
            bundled: false,
            created_at: Some(chrono::Utc::now().to_rfc3339()),
        }
    }
}
