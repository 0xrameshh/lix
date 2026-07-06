use std::path::Path;

use crate::error::Result;
use crate::models::TraceType;
use crate::parser::RawEvent;
use crate::providers::claude_normalize;
use crate::providers::{NormalizedSession, Provider};

pub struct ClaudeCodeProvider;

impl Provider for ClaudeCodeProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::ClaudeCode
    }

    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().take(20).any(|ev| {
            if matches!(
                ev.r#type.as_deref(),
                Some("user" | "assistant" | "system" | "attachment")
            ) && (ev.message.is_some() || ev.attachment.is_some())
            {
                return true;
            }
            if ev.r#type.as_deref() == Some("tool_use") && ev.field_str("tool_name").is_some() {
                return true;
            }
            if ev.r#type.is_none()
                && ev.field_str("display").is_some()
                && ev.field_str("sessionId").is_some()
            {
                return true;
            }
            false
        })
    }

    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        claude_normalize::normalize_events(path, events)
    }
}
