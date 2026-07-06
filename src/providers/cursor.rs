use indexmap::IndexMap;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde_json::Value;

use crate::error::Result;
use crate::models::{Metadata, Step, SystemSubtype, TraceType};
use crate::parser::RawEvent;
use crate::providers::{claude_helpers, shared};
use crate::providers::{NormalizedSession, Provider};

pub struct CursorProvider;

impl Provider for CursorProvider {
    fn trace_type(&self) -> TraceType {
        TraceType::Cursor
    }

    fn matches(&self, events: &[RawEvent]) -> bool {
        events.iter().any(|ev| {
            matches!(
                ev.r#type.as_deref(),
                Some("cursor_session_meta" | "cursor_available_tools")
            )
        }) || events.iter().any(|ev| {
            ev.raw
                .get("raw_cursor")
                .and_then(Value::as_object)
                .is_some()
        })
    }

    fn normalize(&self, path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
        let mut steps: Vec<Step> = Vec::with_capacity(events.len());
        let mut tool_names: BTreeSet<String> = BTreeSet::new();
        let mut argument_samples: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        let mut tool_call_names: BTreeMap<String, String> = BTreeMap::new();
        let mut session_meta: Option<&RawEvent> = None;
        let mut first_ts: Option<String> = None;

        for ev in &events {
            let event_type = ev.r#type.as_deref().unwrap_or("");
            match event_type {
                "cursor_session_meta" => {
                    session_meta = Some(ev);
                    continue;
                }
                "cursor_available_tools" => {
                    continue;
                }
                "turn_ended" => continue,
                _ => {}
            }

            let role = ev.raw.get("role").and_then(Value::as_str).unwrap_or("");
            if role.is_empty() {
                continue;
            }

            let payload = ev.message.as_ref().and_then(Value::as_object);
            if payload.is_none() {
                continue;
            }
            let payload = payload.unwrap();
            let content_blocks = payload.get("content");

            if first_ts.is_none() {
                first_ts = ev.field_str("timestamp").map(String::from);
            }

            match role {
                "user" => {
                    if claude_helpers::has_tool_result_blocks(content_blocks) {
                        let Some(Value::Array(blocks)) = content_blocks else {
                            continue;
                        };
                        for b in blocks {
                            let Some(o) = b.as_object() else { continue };
                            if o.get("type").and_then(Value::as_str) != Some("tool_result") {
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
                    let content = claude_helpers::text_from_content(content_blocks);
                    if let Some(t) = content {
                        if !t.trim().is_empty() {
                            steps.push(Step::User { content: t });
                        }
                    }
                }
                "assistant" => {
                    if let Some((r, sig)) = claude_helpers::reasoning_from_content(content_blocks) {
                        steps.push(Step::Thought {
                            content: r,
                            signature: sig,
                        });
                    }

                    let tool_uses = claude_helpers::tool_uses_from_content(content_blocks);

                    for tu in &tool_uses {
                        let args = shared::normalize_json_like_value(&tu.arguments);
                        tool_call_names.insert(tu.id.clone(), tu.name.clone());
                        tool_names.insert(tu.name.clone());
                        argument_samples
                            .entry(tu.name.clone())
                            .or_default()
                            .push(args.clone());
                        steps.push(Step::ToolCall {
                            id: tu.id.clone(),
                            name: tu.name.clone(),
                            arguments: args,
                        });
                    }

                    let content = claude_helpers::text_from_content(content_blocks);
                    if let Some(t) = content {
                        if !t.trim().is_empty() {
                            steps.push(Step::AssistantText {
                                content: t,
                                api_error: None,
                            });
                        }
                    }
                }
                "system" => {
                    let content = claude_helpers::text_from_content(content_blocks);
                    if let Some(t) = content {
                        steps.push(Step::SystemContext {
                            content: t,
                            subtype: SystemSubtype::Other,
                        });
                    }
                }
                _ => {}
            }
        }

        let model = session_meta
            .and_then(|ev| {
                ev.message
                    .as_ref()
                    .and_then(Value::as_object)
                    .and_then(|o| o.get("model"))
                    .or_else(|| ev.field("model"))
            })
            .and_then(Value::as_str)
            .map(String::from);
        let sid = session_meta
            .and_then(|ev| ev.field("session_id"))
            .or_else(|| session_meta.and_then(|ev| ev.field("sessionId")))
            .and_then(|v| v.as_str().map(String::from));
        let source = session_meta
            .and_then(|ev| ev.field("source"))
            .and_then(Value::as_str);

        let turn_count = steps
            .iter()
            .filter(|s| matches!(s, Step::User { .. }))
            .count();

        let mut extra = IndexMap::new();
        if let Some(ref s) = source {
            extra.insert("source".into(), Value::String(s.to_string()));
        }
        for key in &[
            "cursor_scope",
            "cursor_workspace_id",
            "cursor_table",
            "cursor_key",
            "cursor_storage_kind",
            "cursor_source_db",
        ] {
            if let Some(v) = session_meta.and_then(|ev| ev.field(key)) {
                if let Some(s) = v.as_str().filter(|s| !s.is_empty()) {
                    extra.insert((*key).into(), Value::String(s.to_string()));
                }
            }
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
            trace_type: TraceType::Cursor.as_str().to_string(),
            model_provider: None,
            model,
            cwd: None,
            cli_version: None,
            turn_count,
            usage: None,
            total_cost_usd: None,
            first_message_timestamp: first_ts,
            extra,
        };

        Ok(NormalizedSession {
            trace_type: TraceType::Cursor,
            steps,
            metadata,
            tool_names,
            argument_samples,
        })
    }
}
