use serde::{Deserialize, Serialize};

use super::message::{Message, Tool};
use super::metadata::Metadata;
use super::types::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingExample {
    pub prompt: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub follow_up_prompts: Vec<String>,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub tools: Vec<Tool>,
    pub metadata: Metadata,
}

impl TrainingExample {
    pub fn is_complete(&self) -> bool {
        self.messages
            .last()
            .map(|m| m.role != Role::Tool)
            .unwrap_or(false)
    }
}
