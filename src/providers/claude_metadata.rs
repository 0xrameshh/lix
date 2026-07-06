use std::collections::BTreeSet;

use indexmap::IndexMap;

use serde_json::{Map, Value};

use crate::parser::RawEvent;

pub struct ExtraMetadataCollector {
    pub entrypoint: Option<String>,
    pub user_type: Option<String>,
    pub git_branch: Option<String>,
    pub mode: Option<String>,
    pub permission_mode: Option<String>,
    pub system_prompt: Option<String>,
    pub deferred_tool_names: BTreeSet<String>,
    pub mcp_instruction_names: BTreeSet<String>,
    pub attachment_types: BTreeSet<String>,
}

impl ExtraMetadataCollector {
    pub fn new() -> Self {
        Self {
            entrypoint: None,
            user_type: None,
            git_branch: None,
            mode: None,
            permission_mode: None,
            system_prompt: None,
            deferred_tool_names: BTreeSet::new(),
            mcp_instruction_names: BTreeSet::new(),
            attachment_types: BTreeSet::new(),
        }
    }

    pub fn capture_event(&mut self, ev: &RawEvent) {
        if self.entrypoint.is_none() {
            self.entrypoint = ev.field_str("entrypoint").map(String::from);
        }
        if self.user_type.is_none() {
            self.user_type = ev.field_str("userType").map(String::from);
        }
        if self.git_branch.is_none() {
            self.git_branch = ev.field_str("gitBranch").map(String::from);
        }
        match ev.r#type.as_deref() {
            Some("system") if self.system_prompt.is_none() => {
                self.system_prompt = ev.field_str("content").map(String::from);
            }
            Some("mode") if self.mode.is_none() => {
                self.mode = ev.field_str("mode").map(String::from);
            }
            Some("permission-mode") if self.permission_mode.is_none() => {
                self.permission_mode = ev.field_str("permissionMode").map(String::from);
            }
            _ => {}
        }
    }

    pub fn capture_attachment(&mut self, att: &Map<String, Value>, att_type: &str) {
        if !att_type.is_empty() {
            self.attachment_types.insert(att_type.to_string());
        }
        if att_type == "deferred_tools_delta" {
            if let Some(Value::Array(names)) = att.get("addedNames") {
                for n in names.iter().filter_map(Value::as_str) {
                    self.deferred_tool_names.insert(n.to_string());
                }
            }
        }
        if att_type == "mcp_instructions_delta" {
            if let Some(Value::Array(names)) = att.get("addedNames") {
                for n in names.iter().filter_map(Value::as_str) {
                    self.mcp_instruction_names.insert(n.to_string());
                }
            }
        }
    }

    pub fn into_extra(self, system_contexts_count: usize) -> IndexMap<String, Value> {
        let mut extra = IndexMap::new();
        if let Some(v) = self.entrypoint.filter(|s| !s.is_empty()) {
            extra.insert("entrypoint".into(), Value::String(v));
        }
        if let Some(v) = self.user_type.filter(|s| !s.is_empty()) {
            extra.insert("user_type".into(), Value::String(v));
        }
        if let Some(v) = self.git_branch.filter(|s| !s.is_empty()) {
            extra.insert("git_branch".into(), Value::String(v));
        }
        if let Some(v) = self.mode.filter(|s| !s.is_empty()) {
            extra.insert("mode".into(), Value::String(v));
        }
        if let Some(v) = self.permission_mode.filter(|s| !s.is_empty()) {
            extra.insert("permission_mode".into(), Value::String(v));
        }
        if let Some(v) = self.system_prompt.filter(|s| !s.is_empty()) {
            extra.insert("system_prompt".into(), Value::String(v));
        }
        if system_contexts_count > 0 {
            extra.insert(
                "claude_context_count".into(),
                Value::Number(system_contexts_count.into()),
            );
        }
        if !self.attachment_types.is_empty() {
            let arr: Vec<Value> = self
                .attachment_types
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            extra.insert("claude_attachment_types".into(), Value::Array(arr));
        }
        if !self.deferred_tool_names.is_empty() {
            let arr: Vec<Value> = self
                .deferred_tool_names
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            extra.insert("claude_deferred_tools".into(), Value::Array(arr));
        }
        if !self.mcp_instruction_names.is_empty() {
            let arr: Vec<Value> = self
                .mcp_instruction_names
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            extra.insert("claude_mcp_instruction_names".into(), Value::Array(arr));
        }
        extra
    }
}

impl Default for ExtraMetadataCollector {
    fn default() -> Self {
        Self::new()
    }
}
