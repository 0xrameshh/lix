use crate::error::Result;
use crate::models::{Metadata, Step, SystemSubtype, TraceType};
use crate::parser::{LineReader, RawEvent};
use crate::providers::{pi_helpers, shared, NormalizedSession, Provider};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct PiProvider;

impl Provider for PiProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Pi
    }

    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().any(|ev| {
            matches!(
                ev.r#type.as_deref(),
                Some("session" | "session_info" | "model_change" | "thinking_level_change")
            )
        })
    }

    fn normalize(&self, _path: &Path, _events: Vec<RawEvent>) -> Result<NormalizedSession> {
        self.normalize_file(_path)
    }

    fn normalize_file(&self, path: &Path) -> Result<NormalizedSession> {
        self.normalize_stream(path, TraceType::Pi)
    }
}

impl PiProvider {
    fn normalize_stream(&self, path: &Path, trace_type: TraceType) -> Result<NormalizedSession> {
        let mut steps: Vec<Step> = Vec::new();
        let mut tool_names: BTreeSet<String> = BTreeSet::new();
        let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut model: Option<String> = None;
        let mut model_provider: Option<String> = None;
        let mut cli_version: Option<String> = None;
        let mut first_ts: Option<String> = None;
        let mut extra: IndexMap<String, serde_json::Value> = IndexMap::new();
        let mut invalid_tool_call_ids: BTreeSet<String> = BTreeSet::new();

        // First pass: stream file, collect metadata + invalid tool call IDs
        for ev in LineReader::open(path)? {
            let ev = ev?;
            let payload = ev.message.as_ref().and_then(Value::as_object);
            if ev.r#type.as_deref() == Some("message") {
                if let Some(p) = payload {
                    if p.get("role").and_then(Value::as_str) == Some("toolResult") {
                        let tcid = p.get("toolCallId").and_then(Value::as_str).unwrap_or("");
                        if !tcid.is_empty() {
                            let name = p.get("toolName").and_then(Value::as_str);
                            let content = shared::first_text_block(p.get("content"));
                            if let Some(n) = name {
                                if content.trim() == format!("Tool {n} not found") {
                                    invalid_tool_call_ids.insert(tcid.to_string());
                                }
                            } else if content.trim() == "Tool  not found" {
                                invalid_tool_call_ids.insert(tcid.to_string());
                            }
                        }
                    }
                }
            }
            if ev.r#type.as_deref() == Some("session") {
                if let Some(s) = ev.field_str("id") {
                    session_id.get_or_insert_with(|| s.to_string());
                }
                if let Some(c) = ev.field_str("cwd") {
                    cwd.get_or_insert_with(|| c.to_string());
                }
                if let Some(v) = ev.field_str("version") {
                    cli_version = Some(v.to_string());
                }
            }
            if ev.r#type.as_deref() == Some("model_change") {
                if model.is_none() {
                    if let Some(m) = ev.field_str("modelId") {
                        model = Some(m.to_string());
                    }
                }
                if model_provider.is_none() {
                    if let Some(p) = ev.field_str("provider") {
                        model_provider = Some(p.to_string());
                    }
                }
            }
            if ev.r#type.as_deref() == Some("thinking_level_change") {
                if let Some(lvl) = ev.field_str("thinkingLevel") {
                    extra
                        .entry("thinking_level".to_string())
                        .or_insert_with(|| serde_json::Value::String(lvl.to_string()));
                }
            }
            if ev.r#type.as_deref() == Some("session_info") {
                if let Some(n) = ev.field_str("name") {
                    extra
                        .entry("session_names".to_string())
                        .and_modify(|v| {
                            if let Some(arr) = v.as_array_mut() {
                                arr.push(serde_json::Value::String(n.to_string()));
                            }
                        })
                        .or_insert_with(|| serde_json::json!([n]));
                    extra.insert(
                        "session_name".to_string(),
                        serde_json::Value::String(n.to_string()),
                    );
                }
            }
        }

        // Second pass: stream file again, build steps
        for ev in LineReader::open(path)? {
            let ev = ev?;
            let event_type = ev.r#type.as_deref().unwrap_or("");
            if event_type == "custom" {
                continue;
            }
            if event_type != "message" {
                continue;
            }

            let Some(payload) = ev.message.as_ref().and_then(Value::as_object) else {
                continue;
            };
            let role = payload.get("role").and_then(Value::as_str).unwrap_or("");
            if role.is_empty() {
                continue;
            }
            if first_ts.is_none() && shared::is_user_role(role) {
                first_ts = payload
                    .get("timestamp")
                    .and_then(Value::as_number)
                    .and_then(|n| n.as_f64())
                    .map(shared::epoch_ms_to_iso)
                    .or_else(|| ev.field_str("timestamp").map(String::from));
            }

            if role == "toolResult" {
                let tcid = payload
                    .get("toolCallId")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                if tcid.is_empty() || invalid_tool_call_ids.contains(tcid) {
                    continue;
                }
                let name = payload
                    .get("toolName")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown_tool");
                let content = shared::first_text_block(payload.get("content"));
                let is_error = payload.get("isError").and_then(Value::as_bool);
                steps.push(Step::ToolResponse {
                    tool_call_id: tcid.to_string(),
                    name: name.to_string(),
                    content,
                    is_error,
                });
                continue;
            }

            let normalized_role = shared::normalize_role(role);
            let content_blocks = payload.get("content");
            let content = shared::first_text_block(content_blocks);

            if normalized_role == "user" {
                if !content.is_empty() {
                    steps.push(Step::User { content });
                }
                continue;
            }

            if normalized_role == "assistant" {
                let has_reasoning = pi_helpers::pi_reasoning(content_blocks);
                let reasoning_present = has_reasoning.is_some();
                if let Some(r) = has_reasoning {
                    steps.push(Step::Thought {
                        content: r,
                        signature: None,
                    });
                }

                let tool_calls = pi_helpers::pi_tool_calls(content_blocks, &invalid_tool_call_ids);
                for tc in &tool_calls {
                    let args = shared::normalize_json_like_value(&tc.arguments);
                    tool_names.insert(tc.name.clone());
                    argument_samples
                        .entry(tc.name.clone())
                        .or_default()
                        .push(args.clone());
                    steps.push(Step::ToolCall {
                        id: tc.id.clone(),
                        name: tc.name.clone(),
                        arguments: args,
                    });
                }

                if !content.is_empty() {
                    steps.push(Step::AssistantText {
                        content,
                        api_error: None,
                    });
                } else if tool_calls.is_empty() && !reasoning_present {
                    continue;
                }
                continue;
            }

            if normalized_role == "system" && !content.is_empty() {
                steps.push(Step::SystemContext {
                    content,
                    subtype: SystemSubtype::Other,
                });
            }
        }

        let turn_count = steps
            .iter()
            .filter(|s| matches!(s, Step::User { .. }))
            .count();

        if cli_version.is_none() {
            extra.insert("cli_version".into(), serde_json::Value::Null);
        }
        let metadata = Metadata {
            source_file: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            session_id: session_id
                .or_else(|| path.file_stem().and_then(|n| n.to_str()).map(String::from))
                .unwrap_or_default(),
            trace_type: trace_type.as_str().to_string(),
            model_provider,
            model,
            cwd,
            cli_version,
            turn_count,
            usage: None,
            total_cost_usd: None,
            first_message_timestamp: first_ts,
            extra,
        };

        Ok(NormalizedSession {
            trace_type,
            steps,
            metadata,
            tool_names,
            argument_samples,
        })
    }
}

pub struct OpenclawProvider;

impl Provider for OpenclawProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Openclaw
    }

    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().any(|ev| {
            ev.r#type.as_deref() == Some("session")
                && ev.field_str("cwd").is_some_and(|c| c.contains(".openclaw"))
        })
    }

    fn normalize(&self, _path: &Path, _events: Vec<RawEvent>) -> Result<NormalizedSession> {
        self.normalize_file(_path)
    }

    fn normalize_file(&self, path: &Path) -> Result<NormalizedSession> {
        PiProvider.normalize_stream(path, TraceType::Openclaw)
    }
}
