use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceType {
    Scrivener,
    Plottr,
    Markdown,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::Scrivener => "scrivener",
            SourceType::Plottr => "plottr",
            SourceType::Markdown => "markdown",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "scrivener" => Some(SourceType::Scrivener),
            "plottr" => Some(SourceType::Plottr),
            "markdown" => Some(SourceType::Markdown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub source_type: SourceType,
    pub source_path: Option<String>,
    pub created_at: String,
    pub modified_at: String,
}

impl Project {
    pub fn new(name: String, source_type: SourceType, source_path: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4(),
            name,
            source_type,
            source_path,
            created_at: now.clone(),
            modified_at: now,
        }
    }
}
