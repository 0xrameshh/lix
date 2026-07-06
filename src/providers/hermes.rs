use crate::error::Result;
use crate::models::{Metadata, Step, SystemSubtype, TraceType};
use crate::parser::RawEvent;
use crate::providers::{hermes_meta, hermes_time, shared, NormalizedSession, Provider};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub struct HermesProvider;
pub struct ExternalAgentProvider;

impl Provider for HermesProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Hermes
    }
    fn matches(&self, events: &[RawEvent]) -> bool {
        matches_hermes(events)
    }
    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        normalize_hermes_inner(path, events, TraceType::Hermes)
    }
}

impl Provider for ExternalAgentProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::ExternalAgent
    }
    fn matches(&self, _events: &[RawEvent]) -> bool {
        false
    }
    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        normalize_hermes_inner(path, events, TraceType::ExternalAgent)
    }
}

fn matches_hermes(events: &[RawEvent]) -> bool {
    events.len() == 1
        && events[0].r#type.is_none()
        && events[0]
            .raw
            .get("messages")
            .and_then(Value::as_array)
            .is_some()
}

fn normalize_hermes_inner(
    path: &Path,
    events: Vec<RawEvent>,
    trace_type: TraceType,
) -> Result<NormalizedSession> {
    let mut steps: Vec<Step> = Vec::new();
    let mut tool_names: BTreeSet<String> = BTreeSet::new();
    let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut first_ts: Option<String> = None;
    let mut session_meta: BTreeMap<String, Value> = BTreeMap::new();

    let Some(ev) = events.into_iter().next() else {
        return Ok(NormalizedSession {
            trace_type,
            steps,
            metadata: Metadata {
                source_file: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
                session_id: path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                trace_type: trace_type.as_str().to_string(),
                model_provider: None,
                model: None,
                cwd: None,
                cli_version: None,
                turn_count: 0,
                usage: None,
                total_cost_usd: None,
                first_message_timestamp: None,
                extra: IndexMap::new(),
            },
            tool_names,
            argument_samples,
        });
    };

    // Collect session metadata from the event (all non-message fields)
    for (key, val) in &ev.raw {
        if key != "messages" {
            session_meta.insert(key.clone(), val.clone());
        }
    }

    // Extract messages from the raw event
    let msgs = match ev.raw.get("messages").and_then(Value::as_array) {
        Some(arr) => arr.clone(),
        None => vec![],
    };

    for msg in &msgs {
        let Some(obj) = msg.as_object() else { continue };
        let role = obj.get("role").and_then(Value::as_str).unwrap_or("");
        if role.is_empty() {
            continue;
        }
        let raw_content = obj.get("content");
        let content = match raw_content {
            Some(Value::String(s)) => s.clone(),
            Some(Value::Array(_)) => shared::first_text_block(raw_content),
            _ => String::new(),
        };
        if first_ts.is_none() {
            if let Some(ts) = obj.get("timestamp").and_then(|v| {
                v.as_str()
                    .map(String::from)
                    .or_else(|| v.as_f64().map(hermes_time::timestamp_to_iso))
            }) {
                first_ts = Some(ts);
            }
        }

        match shared::normalize_role(role) {
            "user" if !content.is_empty() => {
                steps.push(Step::User { content });
            }
            "assistant" => {
                // Check for inline reasoning via <think> tags
                let has_reasoning = obj
                    .get("reasoning_content")
                    .or_else(|| obj.get("reasoning"))
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .map(String::from);
                let reasoning_present = has_reasoning.is_some();

                // Check for tool_calls
                let tcs = obj.get("tool_calls").and_then(Value::as_array);
                let mut tool_calls_added = false;
                if let Some(tc_arr) = tcs {
                    for tc in tc_arr {
                        let Some(tc_obj) = tc.as_object() else {
                            continue;
                        };
                        let function = tc_obj.get("function").and_then(Value::as_object);
                        let (fname, fargs) = match function {
                            Some(func) => {
                                let name = func
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string();
                                let args = func
                                    .get("arguments")
                                    .cloned()
                                    .unwrap_or(Value::Object(Default::default()));
                                let args = shared::parse_function_arguments(&args);
                                (name, args)
                            }
                            None => {
                                let name = tc_obj
                                    .get("name")
                                    .and_then(Value::as_str)
                                    .unwrap_or("")
                                    .to_string();
                                (name, Value::Object(Default::default()))
                            }
                        };
                        if fname.is_empty() {
                            continue;
                        }
                        let id = tc_obj
                            .get("id")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string();
                        tool_calls_added = true;
                        tool_names.insert(fname.clone());
                        argument_samples
                            .entry(fname.clone())
                            .or_default()
                            .push(fargs.clone());
                        steps.push(Step::ToolCall {
                            id,
                            name: fname,
                            arguments: fargs,
                        });
                    }
                }

                // Handle reasoning step
                if let Some(r) = has_reasoning {
                    steps.push(Step::Thought {
                        content: r,
                        signature: None,
                    });
                }

                // Handle text content
                if !content.is_empty() {
                    steps.push(Step::AssistantText {
                        content,
                        api_error: None,
                    });
                } else if !tool_calls_added && !reasoning_present {
                    continue;
                }
            }
            "tool" => {
                let tcid = obj
                    .get("tool_call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let name = obj
                    .get("name")
                    .or_else(|| obj.get("tool_name"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown_tool");
                steps.push(Step::ToolResponse {
                    tool_call_id: tcid,
                    name: name.to_string(),
                    content,
                    is_error: None,
                });
            }
            "system" if !content.is_empty() => {
                steps.push(Step::SystemContext {
                    content,
                    subtype: SystemSubtype::Other,
                });
            }
            "user" | "system" => {}
            _ => {}
        }
    }

    let turn_count = steps
        .iter()
        .filter(|s| matches!(s, Step::User { .. }))
        .count();

    let get_str = |k: &str| {
        session_meta
            .get(k)
            .and_then(Value::as_str)
            .map(String::from)
    };
    let model = get_str("model");
    let sid = get_str("id");
    let cwd = get_str("cwd");
    let cli_version = get_str("cli_version");
    let usage: Option<Value> = None;

    let mut extra = hermes_meta::extract_extra(&session_meta);
    extra.insert("model_provider".into(), Value::Null);
    if cli_version.is_none() {
        extra.insert("cli_version".into(), Value::Null);
    }

    let metadata = Metadata {
        source_file: path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string(),
        session_id: sid
            .or_else(|| path.file_stem().and_then(|n| n.to_str()).map(String::from))
            .unwrap_or_default(),
        trace_type: trace_type.as_str().to_string(),
        model_provider: None,
        model,
        cwd,
        cli_version,
        turn_count,
        usage,
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
