use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceSuggestion {
    pub reference_id: String,
    pub reference_type: String,
    pub reference_name: String,
    pub match_text: String,
    pub positions: Vec<usize>,
    pub confidence: f32,
}
