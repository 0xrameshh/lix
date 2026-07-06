use crate::error::Result;
use crate::models::{Metadata, Step, SystemSubtype, TraceType};
use crate::parser::RawEvent;
use crate::providers::{codex_event, codex_reorder, shared, NormalizedSession, Provider};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct CodexProvider;

impl Provider for CodexProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Codex
    }
    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().any(|ev| {
            matches!(
                ev.r#type.as_deref(),
                Some("session_meta" | "turn_context" | "response_item" | "event_msg")
            )
        })
    }
    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        normalize_codex(path, events)
    }
}

fn normalize_codex(path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
    let mut session_id = None;
    let mut model = None;
    let mut model_provider = None;
    let mut source = None;
    let mut cwd = None;
    let mut cli_version = None;
    let mut system_instructions: Option<String> = None;
    let mut turn_ids = Vec::new();
    let mut first_ts = None;
    let mut prompt_set = false;

    let maps: Vec<_> = events.iter().map(|ev| ev.full_map()).collect();
    let normalized: Vec<_> = maps
        .into_iter()
        .map(codex_event::normalize_single_event)
        .collect();
    let ordered = codex_event::reorder_events(&normalized);

    for ev in &events {
        if ev.r#type.as_deref() == Some("session_meta") {
            if let Some(payload) = ev.raw.get("payload").and_then(Value::as_object) {
                if session_id.is_none() {
                    session_id = payload.get("id").and_then(Value::as_str).map(String::from);
                }
                if model_provider.is_none() {
                    model_provider = payload
                        .get("model_provider")
                        .and_then(Value::as_str)
                        .map(String::from);
                }
                if source.is_none() {
                    source = payload
                        .get("source")
                        .and_then(Value::as_str)
                        .map(String::from);
                }
                if cwd.is_none() {
                    cwd = payload.get("cwd").and_then(Value::as_str).map(String::from);
                }
                if cli_version.is_none() {
                    cli_version = payload
                        .get("cli_version")
                        .and_then(Value::as_str)
                        .map(String::from);
                }
                if system_instructions.is_none() {
                    system_instructions = payload
                        .get("base_instructions")
                        .and_then(Value::as_object)
                        .and_then(|bi| bi.get("text"))
                        .and_then(Value::as_str)
                        .map(String::from);
                }
            }
        }
        if ev.r#type.as_deref() == Some("turn_context") {
            if let Some(payload) = ev.raw.get("payload").and_then(Value::as_object) {
                if let Some(tid) = payload.get("turn_id").and_then(Value::as_str) {
                    if turn_ids.is_empty() || turn_ids.last().map(|l| l != tid).unwrap_or(true) {
                        turn_ids.push(tid.to_string());
                    }
                }
                if cwd.is_none() {
                    cwd = payload.get("cwd").and_then(Value::as_str).map(String::from);
                }
                if model.is_none() {
                    model = payload
                        .get("model")
                        .and_then(Value::as_str)
                        .map(String::from);
                }
            }
        }
    }
    first_ts = first_ts.or_else(|| codex_reorder::codex_first_user_ts(&ordered));

    let mut steps: Vec<Step> = Vec::new();
    let mut tool_names: BTreeSet<String> = BTreeSet::new();
    let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut tok_call_ids: BTreeMap<String, String> = BTreeMap::new();

    if let Some(ref text) = system_instructions {
        let already_present = steps
            .iter()
            .any(|s| matches!(s, Step::SystemContext { content, .. } if content == text));
        if !already_present {
            steps.push(Step::SystemContext {
                content: text.clone(),
                subtype: SystemSubtype::Other,
            });
        }
    }

    for ev in &ordered {
        if ev.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }
        let payload = match ev.get("payload").and_then(Value::as_object) {
            Some(p) => p,
            None => continue,
        };
        let pt = match payload.get("type").and_then(Value::as_str) {
            Some(pt) => pt,
            None => continue,
        };

        match pt {
            "message" => {
                let role = payload.get("role").and_then(Value::as_str).unwrap_or("");
                let text = match payload.get("content").and_then(Value::as_array) {
                    Some(c) => shared::first_text_block(Some(&Value::Array(c.clone()))),
                    None => String::new(),
                };
                match shared::normalize_role(role) {
                    "user" if !text.is_empty() => {
                        if !prompt_set && is_codex_runtime_context(&text) {
                            continue;
                        }
                        prompt_set = true;
                        steps.push(Step::User { content: text })
                    }
                    "assistant" if !text.is_empty() => {
                        steps.push(Step::AssistantText {
                            content: text,
                            api_error: None,
                        });
                    }
                    "system" if !text.is_empty() => {
                        steps.push(Step::SystemContext {
                            content: text,
                            subtype: SystemSubtype::Other,
                        });
                    }
                    _ => {}
                }
            }
            "function_call" => {
                let name = payload
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let cid = payload
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let args = shared::parse_function_arguments(
                    &payload.get("arguments").cloned().unwrap_or(Value::Null),
                );
                if !name.is_empty() {
                    tok_call_ids.insert(cid.clone(), name.clone());
                    tool_names.insert(name.clone());
                    argument_samples
                        .entry(name.clone())
                        .or_default()
                        .push(args.clone());
                    steps.push(Step::ToolCall {
                        id: cid,
                        name,
                        arguments: args,
                    });
                }
            }
            "function_call_output" => {
                let cid = payload
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let text = match payload.get("output").or_else(|| payload.get("content")) {
                    Some(Value::String(s)) => s.clone(),
                    Some(Value::Array(blocks)) => {
                        shared::first_text_block(Some(&Value::Array(blocks.clone())))
                    }
                    _ => String::new(),
                };
                let name = tok_call_ids
                    .get(&cid)
                    .cloned()
                    .unwrap_or_else(|| "unknown_tool".into());
                let is_error = payload.get("is_error").and_then(Value::as_bool);
                steps.push(Step::ToolResponse {
                    tool_call_id: cid,
                    name,
                    content: text,
                    is_error,
                });
            }
            "reasoning" => {
                if let Some(r) = codex_reorder::reasoning_text(ev) {
                    steps.push(Step::Thought {
                        content: r,
                        signature: None,
                    });
                }
            }
            _ => {}
        }
    }

    let turn_count = turn_ids.len();

    let mut extra = IndexMap::new();
    if let Some(ref s) = source {
        extra.insert("source".into(), Value::String(s.to_string()));
    }
    if let Some(ref s) = system_instructions {
        extra.insert(
            "system_prompt".into(),
            Value::String(s.trim_end().to_string()),
        );
    }

    let metadata = Metadata {
        source_file: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        session_id: session_id.unwrap_or_else(|| {
            path.file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string()
        }),
        trace_type: TraceType::Codex.as_str().to_string(),
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
        trace_type: TraceType::Codex,
        steps,
        metadata,
        tool_names,
        argument_samples,
    })
}

pub fn is_codex_runtime_context(text: &str) -> bool {
    text.trim_start().starts_with("<environment_context>")
        || text.trim_start().starts_with("<user_instructions>")
}
