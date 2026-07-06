use indexmap::IndexMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::error::Result;
use crate::models::{Metadata, Step, TraceType};
use crate::parser::RawEvent;
use crate::providers::{claude_helpers, shared};
use crate::providers::{NormalizedSession, Provider};

pub struct DroidProvider;

impl Provider for DroidProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Droid
    }

    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().any(|ev| {
            ev.r#type.as_deref() == Some("session_start")
                && ev.field_str("cwd").is_some()
                && ev.raw.get("version").is_some()
        })
    }

    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        let mut steps: Vec<Step> = Vec::with_capacity(events.len());
        let mut tool_names: BTreeSet<String> = BTreeSet::new();
        let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let mut tool_call_names: BTreeMap<String, String> = BTreeMap::new();
        let mut session_id: Option<String> = None;
        let mut cwd: Option<String> = None;
        let mut title: Option<String> = None;
        let mut first_ts: Option<String> = None;

        for ev in events {
            let event_type = ev.r#type.as_deref().unwrap_or("");
            match event_type {
                "session_start" => {
                    if let Some(s) = ev.field_str("id") {
                        session_id = Some(s.to_string());
                    }
                    if let Some(s) = ev.field_str("cwd") {
                        cwd = Some(s.to_string());
                    }
                    let t = ev
                        .field_str("sessionTitle")
                        .or_else(|| ev.field_str("title"));
                    if let Some(s) = t {
                        title = Some(s.to_string());
                    }
                }
                "message" => {
                    let Some(msg) = ev.message.as_ref().and_then(Value::as_object) else {
                        continue;
                    };
                    let visibility = msg.get("visibility").and_then(Value::as_str).unwrap_or("");
                    if visibility == "user_only" {
                        continue;
                    }
                    if first_ts.is_none() {
                        first_ts = ev.field_str("timestamp").map(String::from);
                    }
                    let role = msg.get("role").and_then(Value::as_str).unwrap_or("");
                    let content_blocks = msg.get("content");

                    if role == "user" {
                        if let Some(Value::Array(blocks)) = content_blocks {
                            if blocks.iter().any(|b| {
                                b.as_object()
                                    .and_then(|o| o.get("type"))
                                    .and_then(Value::as_str)
                                    == Some("tool_result")
                            }) {
                                for b in blocks {
                                    let Some(o) = b.as_object() else { continue };
                                    if o.get("type").and_then(Value::as_str) != Some("tool_result")
                                    {
                                        continue;
                                    }
                                    let tcid = o
                                        .get("tool_use_id")
                                        .or_else(|| o.get("tool_call_id"))
                                        .and_then(Value::as_str)
                                        .unwrap_or("")
                                        .to_string();
                                    let name = tool_call_names
                                        .get(&tcid)
                                        .cloned()
                                        .unwrap_or_else(|| "unknown_tool".into());
                                    steps.push(Step::ToolResponse {
                                        tool_call_id: tcid,
                                        name,
                                        content: claude_helpers::tool_result_text(o),
                                        is_error: None,
                                    });
                                }
                                continue;
                            }
                        }
                        let content = claude_helpers::text_from_content(content_blocks);
                        if let Some(t) = content {
                            if !t.trim().is_empty() {
                                if visibility == "llm_only" {
                                    steps.push(Step::LlmOnly { content: t });
                                } else {
                                    steps.push(Step::User { content: t });
                                }
                            }
                        }
                        continue;
                    }

                    if role == "assistant" {
                        let content = claude_helpers::text_from_content(content_blocks);
                        let reasoning = claude_helpers::reasoning_from_content(content_blocks);
                        let tool_uses = claude_helpers::tool_uses_from_content(content_blocks);

                        if let Some((r, sig)) = reasoning {
                            steps.push(Step::Thought {
                                content: r,
                                signature: sig,
                            });
                        }
                        if let Some(t) = content {
                            if !t.trim().is_empty() {
                                steps.push(Step::AssistantText {
                                    content: t,
                                    api_error: None,
                                });
                            }
                        }
                        for tu in tool_uses {
                            let args = shared::normalize_json_like_value(&tu.arguments);
                            tool_call_names.insert(tu.id.clone(), tu.name.clone());
                            tool_names.insert(tu.name.clone());
                            argument_samples
                                .entry(tu.name.clone())
                                .or_default()
                                .push(args.clone());
                            steps.push(Step::ToolCall {
                                id: tu.id,
                                name: tu.name,
                                arguments: args,
                            });
                        }
                        continue;
                    }
                }
                _ => {}
            }
        }

        let settings = load_droid_settings(path);
        let model = settings
            .get("model")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(String::from);
        let provider_lock = settings
            .get("providerLock")
            .and_then(Value::as_str)
            .filter(|s| !s.trim().is_empty())
            .map(String::from);

        let turn_count = steps
            .iter()
            .filter(|s| matches!(s, Step::User { .. } | Step::LlmOnly { .. }))
            .count();
        let mut extra = IndexMap::new();
        if let Some(ref t) = title {
            extra.insert("title".into(), Value::String(t.clone()));
        }
        if let Some(v) = settings
            .get("reasoningEffort")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            extra.insert("reasoning_effort".into(), Value::String(v.to_string()));
        }
        if let Some(v) = settings
            .get("autonomyLevel")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            extra.insert("autonomy_level".into(), Value::String(v.to_string()));
        }
        let usage = settings
            .get("tokenUsage")
            .and_then(Value::as_object)
            .cloned();
        let usage_json = usage.map(Value::Object);

        if model.is_none() {
            extra.insert("model".into(), Value::Null);
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
            trace_type: TraceType::Droid.as_str().to_string(),
            model_provider: provider_lock.or(Some("factory".to_string())),
            model,
            cwd,
            cli_version: None,
            turn_count,
            usage: usage_json,
            total_cost_usd: None,
            first_message_timestamp: first_ts,
            extra,
        };

        Ok(NormalizedSession {
            trace_type: TraceType::Droid,
            steps,
            metadata,
            tool_names,
            argument_samples,
        })
    }
}

fn load_droid_settings(path: &Path) -> Value {
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let settings_path = parent.join(format!("{stem}.settings.json"));
    let content = match std::fs::read_to_string(&settings_path) {
        Ok(c) => c,
        Err(_) => return Value::Object(Default::default()),
    };
    serde_json::from_str(&content).unwrap_or(Value::Object(Default::default()))
}
