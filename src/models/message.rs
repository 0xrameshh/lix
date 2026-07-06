use serde::{Deserialize, Serialize};

use super::types::Role;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "reasoning_content")]
    pub reasoning_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "teich_provider_error"
    )]
    pub teich_provider_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRef {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: ToolCallKind,
    pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCallKind {
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub kind: ToolEntryKind,
    pub function: ToolFunctionSchema,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolEntryKind {
    Function,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}
