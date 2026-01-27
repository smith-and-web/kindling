use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub reference_type: String,
    pub name: String,
    pub description: Option<String>,
    pub attributes: HashMap<String, String>,
    pub source_id: Option<String>,
}

impl ReferenceItem {
    pub fn new(
        project_id: Uuid,
        reference_type: String,
        name: String,
        description: Option<String>,
        source_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            reference_type,
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
