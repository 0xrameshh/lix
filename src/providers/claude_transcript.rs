use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::error::Result;
use crate::models::{Metadata, Step, TraceType};
use crate::parser::RawEvent;
use crate::providers::claude_metadata::ExtraMetadataCollector;
use crate::providers::NormalizedSession;

/// Check if events are Claude Code transcript export format
/// (type: "user"/"tool_use"/"tool_result"/"assistant" with content directly on the event).
pub fn is_transcript_format(events: &[RawEvent]) -> bool {
    events.iter().take(20).any(|ev| {
        // Has tool_use with tool_name
        if ev.r#type.as_deref() == Some("tool_use") && ev.field_str("tool_name").is_some() {
            return true;
        }
        // Has user/assistant with content string, no message object
        if matches!(ev.r#type.as_deref(), Some("user" | "assistant"))
            && ev.raw.get("content").and_then(Value::as_str).is_some()
            && ev.message.is_none()
        {
            return true;
        }
        false
    })
}

/// Check if events are history.jsonl format (no type, display + sessionId).
pub fn is_history_format(events: &[RawEvent]) -> bool {
    events.iter().take(20).any(|ev| {
        ev.r#type.is_none()
            && ev.field_str("display").is_some()
            && ev.field_str("sessionId").is_some()
    })
}

/// Normalize a transcript-format event stream into training steps.
pub fn normalize_transcript(path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
    let mut steps: Vec<Step> = Vec::with_capacity(events.len());
    let mut tool_names: BTreeSet<String> = BTreeSet::new();
    let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut first_ts: Option<String> = None;
    let model: Option<String> = None;
    let session_id: Option<String> = None;
    let mut extra_collector = ExtraMetadataCollector::new();

    for ev in events {
        let event_type = ev.r#type.as_deref().unwrap_or("");
        if first_ts.is_none() && matches!(event_type, "user" | "assistant") {
            first_ts = ev.field_str("timestamp").map(String::from);
        }
        extra_collector.capture_event(&ev);

        match event_type {
            "user" => {
                let content = ev
                    .raw
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if !content.is_empty() {
                    steps.push(Step::User { content });
                }
            }
            "assistant" => {
                let content = ev
                    .raw
                    .get("content")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if !content.is_empty() {
                    steps.push(Step::AssistantText {
                        content,
                        api_error: None,
                    });
                }
            }
            "tool_use" => {
                let name = ev.field_str("tool_name").unwrap_or("unknown").to_string();
                let args = ev.raw.get("tool_input").cloned().unwrap_or(Value::Null);
                let id = format!("transcript_{}", steps.len());
                tool_names.insert(name.clone());
                argument_samples
                    .entry(name.clone())
                    .or_default()
                    .push(args.clone());
                steps.push(Step::ToolCall {
                    id,
                    name,
                    arguments: args,
                });
            }
            "tool_result" => {
                let name = ev.field_str("tool_name").unwrap_or("unknown").to_string();
                let content = ev
                    .raw
                    .get("tool_output")
                    .map(|v| serde_json::to_string(v).unwrap_or_default())
                    .unwrap_or_default();
                let id = format!("transcript_{}", steps.len());
                let is_error = ev.field_bool("is_error");
                steps.push(Step::ToolResponse {
                    tool_call_id: id,
                    name,
                    content,
                    is_error,
                });
            }
            _ => {
                steps.push(Step::Telemetry {
                    event_type: event_type.to_string(),
                    payload: Some(serde_json::to_value(&ev.raw).unwrap_or(Value::Null)),
                });
            }
        }
    }

    let turn_count = steps
        .iter()
        .filter(|s| matches!(s, Step::User { .. }))
        .count();
    let metadata = Metadata {
        source_file: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        session_id: session_id
            .or_else(|| path.file_stem().and_then(|n| n.to_str()).map(String::from))
            .unwrap_or_default(),
        trace_type: TraceType::ClaudeCode.as_str().to_string(),
        model_provider: Some("anthropic".to_string()),
        model,
        cwd: None,
        cli_version: None,
        turn_count,
        usage: None,
        total_cost_usd: None,
        first_message_timestamp: first_ts,
        extra: extra_collector.into_extra(0),
    };
    Ok(NormalizedSession {
        trace_type: TraceType::ClaudeCode,
        steps,
        metadata,
        tool_names,
        argument_samples,
    })
}

/// Normalize history.jsonl format into training steps.
pub fn normalize_history(path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
    let mut steps: Vec<Step> = Vec::with_capacity(events.len());
    let model: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut first_ts: Option<String> = None;
    let mut extra_collector = ExtraMetadataCollector::new();

    for ev in events {
        let event_type = ev.r#type.as_deref().unwrap_or("");
        if first_ts.is_none() {
            first_ts = ev
                .raw
                .get("timestamp")
                .and_then(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)))
                .map(|ts| ts.to_string());
        }
        extra_collector.capture_event(&ev);

        if let Some(sid) = ev.field_str("sessionId") {
            session_id.get_or_insert_with(|| sid.to_string());
        }

        match event_type {
            "" => {
                let content = ev.field_str("display").unwrap_or("").to_string();
                if !content.is_empty() {
                    if content.starts_with('/') {
                        steps.push(Step::Telemetry {
                            event_type: "command".into(),
                            payload: Some(serde_json::to_value(&ev.raw).unwrap_or(Value::Null)),
                        });
                    } else {
                        steps.push(Step::User { content });
                    }
                }
            }
            _ => {
                steps.push(Step::Telemetry {
                    event_type: event_type.to_string(),
                    payload: Some(serde_json::to_value(&ev.raw).unwrap_or(Value::Null)),
                });
            }
        }
    }

    let turn_count = steps
        .iter()
        .filter(|s| matches!(s, Step::User { .. }))
        .count();
    let metadata = Metadata {
        source_file: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        session_id: session_id.unwrap_or_default(),
        trace_type: TraceType::ClaudeCode.as_str().to_string(),
        model_provider: Some("anthropic".to_string()),
        model,
        cwd: None,
        cli_version: None,
        turn_count,
        usage: None,
        total_cost_usd: None,
        first_message_timestamp: first_ts,
        extra: extra_collector.into_extra(0),
    };
    Ok(NormalizedSession {
        trace_type: TraceType::ClaudeCode,
        steps,
        metadata,
        tool_names: BTreeSet::new(),
        argument_samples: BTreeMap::new(),
    })
}
