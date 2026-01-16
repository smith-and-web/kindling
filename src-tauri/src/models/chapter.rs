use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub position: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub locked: bool,
}

impl Chapter {
    pub fn new(project_id: Uuid, title: String, position: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            title,
            position,
            source_id: None,
            archived: false,
            locked: false,
        }
    }

    pub fn with_source_id(mut self, source_id: Option<String>) -> Self {
        self.source_id = source_id;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_new() {
        let project_id = Uuid::new_v4();
        let chapter = Chapter::new(project_id, "Chapter One".to_string(), 0);

        assert_eq!(chapter.title, "Chapter One");
        assert_eq!(chapter.project_id, project_id);
        assert_eq!(chapter.position, 0);
        assert!(!chapter.id.is_nil());
    }

    #[test]
    fn test_chapter_serialization() {
        let chapter = Chapter::new(Uuid::new_v4(), "Test Chapter".to_string(), 1);
        let json = serde_json::to_string(&chapter).unwrap();
        assert!(json.contains("Test Chapter"));
    }
}
