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

    pub fn parse(s: &str) -> Option<Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_as_str() {
        assert_eq!(SourceType::Scrivener.as_str(), "scrivener");
        assert_eq!(SourceType::Plottr.as_str(), "plottr");
        assert_eq!(SourceType::Markdown.as_str(), "markdown");
    }

    #[test]
    fn test_source_type_parse() {
        assert_eq!(SourceType::parse("scrivener"), Some(SourceType::Scrivener));
        assert_eq!(SourceType::parse("PLOTTR"), Some(SourceType::Plottr));
        assert_eq!(SourceType::parse("Markdown"), Some(SourceType::Markdown));
        assert_eq!(SourceType::parse("unknown"), None);
    }

    #[test]
    fn test_project_new() {
        let project = Project::new(
            "Test Project".to_string(),
            SourceType::Plottr,
            Some("/path/to/file.pltr".to_string()),
        );

        assert_eq!(project.name, "Test Project");
        assert_eq!(project.source_type, SourceType::Plottr);
        assert_eq!(project.source_path, Some("/path/to/file.pltr".to_string()));
        assert!(!project.id.is_nil());
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("Test".to_string(), SourceType::Markdown, None);

        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("Markdown"));
    }
}
