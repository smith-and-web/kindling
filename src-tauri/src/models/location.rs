use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub attributes: HashMap<String, String>,
    pub source_id: Option<String>,
}

impl Location {
    pub fn new(project_id: Uuid, name: String, description: Option<String>, source_id: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            name,
            description,
            attributes: HashMap::new(),
            source_id,
        }
    }

    pub fn with_attributes(mut self, attributes: HashMap<String, String>) -> Self {
        self.attributes = attributes;
        self
    }
}
