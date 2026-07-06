pub mod claude;
pub mod claude_context;
pub mod claude_helpers;
pub mod claude_metadata;
pub mod claude_normalize;
pub mod claude_transcript;
pub mod codex;
pub mod codex_event;
pub mod codex_reorder;
pub mod cursor;
pub mod droid;
pub mod hermes;
pub mod hermes_meta;
pub mod hermes_time;
pub mod pi;
pub mod pi_helpers;
pub mod shared;
pub mod to_example;

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use crate::error::{Result, TraceForgeError};
use crate::models::{Metadata, Step, TraceType};
use crate::parser::RawEvent;

pub struct NormalizedSession {
    pub trace_type: TraceType,
    pub steps: Vec<Step>,
    pub metadata: Metadata,
    pub tool_names: BTreeSet<String>,
    pub argument_samples: std::collections::BTreeMap<String, Vec<Value>>,
}

pub trait Provider: Send + Sync {
    fn trace_type(&self) -> TraceType;
    fn matches(&self, events: &[RawEvent]) -> bool;
    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession>;
    fn normalize_file(&self, path: &Path) -> Result<NormalizedSession> {
        let events = crate::parser::read_all_events(path)?;
        self.normalize(path, events)
    }
}

const CURSOR_METADATA_KEYS: &[&str] = &[
    "cursor_headers",
    "cursor_key",
    "cursor_message_count",
    "cursor_row",
    "cursor_scope",
    "cursor_source_db",
    "cursor_storage_kind",
    "cursor_table",
    "cursor_workspace_id",
];

fn is_openclaw_session_header(ev: &RawEvent) -> bool {
    ev.r#type.as_deref() == Some("session")
        && ev.field_str("cwd").is_some_and(|c| c.contains(".openclaw"))
}

fn is_hermes_conversation(ev: &RawEvent) -> bool {
    ev.raw.get("from").and_then(Value::as_str).is_some()
        && ev.raw.get("value").and_then(Value::as_str).is_some()
}

fn is_hermes_trace_row(ev: &RawEvent) -> bool {
    ev.field_str("id").is_some() && ev.raw.get("traces").and_then(Value::as_array).is_some()
}

fn is_cursor_trace_row(ev: &RawEvent) -> bool {
    if matches!(
        ev.r#type.as_deref(),
        Some("cursor_session_meta" | "cursor_available_tools")
    ) {
        return true;
    }
    if let Some(meta) = ev.raw.get("metadata").and_then(Value::as_object) {
        if let Some(tt) = meta.get("trace_type").and_then(Value::as_str) {
            if tt.trim().to_lowercase().replace('-', "_") == "cursor" {
                return true;
            }
        }
        if CURSOR_METADATA_KEYS.iter().any(|k| meta.contains_key(*k)) {
            return true;
        }
    }
    ev.raw
        .get("raw_cursor")
        .and_then(Value::as_object)
        .is_some()
}

fn is_hermes_export_session(ev: &RawEvent) -> bool {
    let src = ev.field_str("source");
    let started = ev.raw.get("started_at");
    let parent = ev.field_str("parent_session_id");
    ev.raw.get("messages").and_then(Value::as_array).is_some()
        && (src == Some("cli")
            || started.and_then(Value::as_f64).is_some()
            || started.and_then(Value::as_u64).is_some()
            || parent.is_some())
}

fn external_session_source(ev: &RawEvent) -> Option<String> {
    let payload = ev.raw.get("payload")?.as_object()?;
    let source = payload.get("source")?.as_str()?;
    Some(source.trim().to_lowercase())
}

/// 13-step priority detection.
pub fn detect(events: &[RawEvent]) -> TraceType {
    // Step 1: openclaw — session + .openclaw in cwd
    for ev in events {
        if is_openclaw_session_header(ev) {
            return TraceType::Openclaw;
        }
    }
    // Steps 2-13: scan all events
    for ev in events {
        // Step 2: hermes conversation format
        if is_hermes_conversation(ev) {
            return TraceType::Hermes;
        }
        // Step 3: hermes trace row
        if is_hermes_trace_row(ev) {
            return TraceType::Hermes;
        }
        // Step 4: cursor
        if is_cursor_trace_row(ev) {
            return TraceType::Cursor;
        }
        // Step 5-7: external_session_meta
        if ev.r#type.as_deref() == Some("external_session_meta") {
            let source = external_session_source(ev);
            match source.as_deref() {
                // Step 5: claude_code
                Some("claude" | "claude-code" | "claude_code") => return TraceType::ClaudeCode,
                // Step 6: hermes
                Some("hermes" | "hermes-agent" | "hermes_agent") => return TraceType::Hermes,
                // Step 7: external_agent
                _ => return TraceType::ExternalAgent,
            }
        }
        // Step 8: hermes (type == "hermes_session_meta" or export session or has role)
        if ev.r#type.as_deref() == Some("hermes_session_meta") {
            return TraceType::Hermes;
        }
        if is_hermes_export_session(ev) {
            return TraceType::Hermes;
        }
        // Hermes session file: single JSON with messages + session_start
        if ev.r#type.is_none()
            && ev.raw.get("messages").and_then(Value::as_array).is_some()
            && ev
                .raw
                .get("session_start")
                .and_then(Value::as_str)
                .is_some()
        {
            return TraceType::Hermes;
        }
        if ev.r#type.is_none() && ev.raw.get("role").and_then(Value::as_str).is_some() {
            return TraceType::Hermes;
        }
        // Step 9: claude_code
        if matches!(
            ev.r#type.as_deref(),
            Some("assistant" | "user" | "system" | "result")
        ) && (ev.field_str("session_id").is_some() || ev.field_str("sessionId").is_some())
        {
            return TraceType::ClaudeCode;
        }
        // Step 9a: claude_code transcript export (tool_use with tool_name)
        if ev.r#type.as_deref() == Some("tool_use") && ev.field_str("tool_name").is_some() {
            return TraceType::ClaudeCode;
        }
        // Step 9b: claude_code history.jsonl format (sessionId, display, no type)
        if ev.r#type.is_none()
            && ev.field_str("sessionId").is_some()
            && ev.field_str("display").is_some()
        {
            return TraceType::ClaudeCode;
        }
        // Step 9c: claude_code transcript format (user/assistant with content string, no message)
        if matches!(ev.r#type.as_deref(), Some("user" | "assistant"))
            && ev.raw.get("content").and_then(Value::as_str).is_some()
            && ev.message.is_none()
        {
            return TraceType::ClaudeCode;
        }
        // Step 10: codex
        if matches!(
            ev.r#type.as_deref(),
            Some("session_meta" | "turn_context" | "response_item" | "event_msg")
        ) {
            return TraceType::Codex;
        }
        // Step 11: droid
        if ev.r#type.as_deref() == Some("session_start")
            && ev.field_str("cwd").is_some()
            && ev.raw.get("version").is_some()
        {
            return TraceType::Droid;
        }
        // Step 12: pi
        if matches!(
            ev.r#type.as_deref(),
            Some(
                "session"
                    | "message"
                    | "session_info"
                    | "model_change"
                    | "thinking_level_change"
                    | "compaction"
                    | "branch_summary"
                    | "custom"
                    | "custom_message"
                    | "label"
            )
        ) {
            return TraceType::Pi;
        }
    }
    // Step 13: default to codex
    TraceType::Codex
}

pub fn for_type(tt: TraceType) -> Result<Box<dyn Provider>> {
    match tt {
        TraceType::ClaudeCode => Ok(Box::new(claude::ClaudeCodeProvider)),
        TraceType::Cursor => Ok(Box::new(cursor::CursorProvider)),
        TraceType::Droid => Ok(Box::new(droid::DroidProvider)),
        TraceType::Pi => Ok(Box::new(pi::PiProvider)),
        TraceType::Openclaw => Ok(Box::new(pi::OpenclawProvider)),
        TraceType::Hermes => Ok(Box::new(hermes::HermesProvider)),
        TraceType::ExternalAgent => Ok(Box::new(hermes::ExternalAgentProvider)),
        TraceType::Codex => Ok(Box::new(codex::CodexProvider)),
        TraceType::Chat | TraceType::Unknown => Err(TraceForgeError::UnsupportedProvider(
            tt.as_str().to_string(),
        )),
    }
}
