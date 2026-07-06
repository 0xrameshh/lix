use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

use crate::error::Result;
use crate::models::{Metadata, Step, SyntheticReason, TraceType};
use crate::parser::RawEvent;
use crate::providers::{
    claude_context, claude_helpers, claude_metadata::ExtraMetadataCollector, claude_transcript,
    NormalizedSession,
};

pub fn normalize_events(path: &Path, events: Vec<RawEvent>) -> Result<NormalizedSession> {
    if claude_transcript::is_transcript_format(&events) {
        return claude_transcript::normalize_transcript(path, events);
    }
    if claude_transcript::is_history_format(&events) {
        return claude_transcript::normalize_history(path, events);
    }
    let mut steps: Vec<Step> = Vec::with_capacity(events.len());
    let mut tool_names: BTreeSet<String> = BTreeSet::new();
    let mut argument_samples: std::collections::BTreeMap<String, Vec<Value>> =
        std::collections::BTreeMap::new();
    let mut model: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut cli_version: Option<String> = None;
    let mut session_id: Option<String> = None;
    let mut usage: Option<Value> = None;
    let mut total_cost_usd: Option<f64> = None;
    let mut first_ts: Option<String> = None;

    let mut seen_system: BTreeSet<String> = BTreeSet::new();
    let mut extra_collector = ExtraMetadataCollector::new();

    for ev in events {
        let event_type = ev.r#type.as_deref().unwrap_or("");

        claude_helpers::capture_metadata(
            &ev,
            &mut session_id,
            &mut model,
            &mut cwd,
            &mut cli_version,
        );
        claude_helpers::capture_usage(&ev, &mut usage, &mut total_cost_usd);
        extra_collector.capture_event(&ev);
        match event_type {
            "queue-operation" | "last-prompt" => {
                steps.push(Step::Telemetry {
                    event_type: event_type.to_string(),
                    payload: Some(serde_json::to_value(&ev.raw).unwrap_or(Value::Null)),
                });
            }
            "system" => {
                let subtype = ev.field_str("subtype").unwrap_or("");
                if subtype == "init" {
                    if let Some(m) = ev.field_str("model") {
                        model = Some(m.to_string());
                    }
                }
                if let Some(ctx) = claude_helpers::system_context_text(&ev, subtype) {
                    if seen_system.insert(ctx.clone()) {
                        steps.push(Step::SystemContext {
                            content: ctx,
                            subtype: crate::models::SystemSubtype::from_system_subtype(subtype),
                        });
                    }
                }
            }
            "attachment" => {
                if let Some(att) = ev.attachment.as_ref().and_then(Value::as_object) {
                    let att_type = att.get("type").and_then(Value::as_str).unwrap_or("");
                    extra_collector.capture_attachment(att, att_type);
                    if att_type == "deferred_tools_delta" {
                        if let Some(Value::Array(names)) = att.get("addedNames") {
                            for n in names.iter().filter_map(Value::as_str) {
                                tool_names.insert(n.to_string());
                            }
                        }
                    }
                    if let Some(ctx) = claude_context::attachment_context(att, att_type) {
                        if seen_system.insert(ctx.clone()) {
                            steps.push(Step::SystemContext {
                                content: ctx,
                                subtype: crate::models::SystemSubtype::from_attachment_type(
                                    att_type,
                                ),
                            });
                        }
                    }
                    if let Some(qc) = claude_helpers::queued_command_content(&ev) {
                        steps.push(Step::User { content: qc });
                    }
                }
                if let Some(sid) = ev
                    .field_str("sessionId")
                    .or_else(|| ev.field_str("session_id"))
                {
                    session_id.get_or_insert_with(|| sid.to_string());
                }
            }
            "user" => {
                if let Some(msg) = ev.message.as_ref().and_then(Value::as_object) {
                    if claude_helpers::has_tool_result_blocks(msg.get("content")) {
                        if let Some(Value::Array(blocks)) = msg.get("content") {
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
                                let tool_name = "unknown_tool".to_string();
                                steps.push(Step::ToolResponse {
                                    tool_call_id: tcid,
                                    name: tool_name,
                                    content: claude_helpers::tool_result_text(o),
                                    is_error: None,
                                });
                            }
                        }
                        continue;
                    }
                }
                let content_text = claude_helpers::user_content_text(&ev);
                if let Some(t) = content_text {
                    if ev.field_bool("isMeta").unwrap_or(false) {
                        continue;
                    }
                    if claude_helpers::is_local_command_artifact(&t) {
                        continue;
                    }
                    first_ts
                        .get_or_insert_with(|| ev.field_str("timestamp").unwrap_or("").to_string());
                    if let Some(goal) = claude_helpers::promote_goal_command(&t) {
                        steps.push(Step::User { content: goal });
                    } else {
                        steps.push(Step::User { content: t });
                    }
                }
            }
            "assistant" => {
                let Some(msg) = ev.message.as_ref().and_then(Value::as_object) else {
                    continue;
                };
                if let Some(m) = msg.get("model").and_then(Value::as_str) {
                    model = Some(m.to_string());
                }
                if let Some(u) = msg.get("usage") {
                    usage = Some(u.clone());
                }

                let ct = msg.get("content");
                let content_text = claude_helpers::text_from_content(ct);
                let reasoning = claude_helpers::reasoning_from_content(ct);
                let tool_uses = claude_helpers::tool_uses_from_content(ct);

                if claude_helpers::should_drop_assistant_fragment(&ev)
                    && reasoning.is_none()
                    && tool_uses.is_empty()
                {
                    continue;
                }

                let synthetic_reason = content_text
                    .as_ref()
                    .and_then(|text| claude_helpers::is_synthetic_artifact(&ev, text));
                if let Some(reason) = &synthetic_reason {
                    steps.push(Step::SyntheticArtifact {
                        reason: reason.clone(),
                    });
                    if matches!(reason, SyntheticReason::NoResponseRequested) {
                        continue;
                    }
                }

                first_ts.get_or_insert_with(|| ev.field_str("timestamp").unwrap_or("").to_string());

                if let Some((r, sig)) = reasoning {
                    steps.push(Step::Thought {
                        content: r,
                        signature: sig,
                    });
                }
                if let Some(t) = content_text {
                    if !t.trim().is_empty() {
                        let api_error = ev
                            .field_str("error")
                            .filter(|e| !e.trim().is_empty())
                            .or_else(|| {
                                ev.field_bool("isApiErrorMessage")
                                    .unwrap_or(false)
                                    .then_some("true")
                            })
                            .map(String::from);
                        steps.push(Step::AssistantText {
                            content: t,
                            api_error,
                        });
                    }
                }
                for tu in tool_uses {
                    tool_names.insert(tu.name.clone());
                    argument_samples
                        .entry(tu.name.clone())
                        .or_default()
                        .push(tu.arguments.clone());
                    steps.push(Step::ToolCall {
                        id: tu.id,
                        name: tu.name,
                        arguments: tu.arguments,
                    });
                }
            }
            "result" => {
                if let Some(u) = ev.field("usage") {
                    usage = Some(u.clone());
                }
                if let Some(c) = ev.field_f64("total_cost_usd") {
                    total_cost_usd = Some(c);
                }
                if let Some(r) = ev.field_str("result") {
                    let trimmed = r.trim();
                    if !trimmed.is_empty() {
                        let duplicate = steps
                            .iter()
                            .rev()
                            .find_map(|s| match s {
                                Step::AssistantText { content, .. } => {
                                    Some(content.trim() == trimmed)
                                }
                                _ => None,
                            })
                            .unwrap_or(false);
                        if !duplicate {
                            steps.push(Step::AssistantText {
                                content: trimmed.to_string(),
                                api_error: None,
                            });
                        }
                    }
                }
                steps.push(Step::Telemetry {
                    event_type: "result".into(),
                    payload: Some(serde_json::to_value(&ev.raw).unwrap_or(Value::Null)),
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
    let mut extra = extra_collector.into_extra(seen_system.len());
    if cwd.is_none() {
        extra.insert("cwd".into(), Value::Null);
    }
    if cli_version.is_none() {
        extra.insert("cli_version".into(), Value::Null);
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
        trace_type: TraceType::ClaudeCode.as_str().to_string(),
        model_provider: Some("anthropic".to_string()),
        model,
        cwd,
        cli_version,
        turn_count,
        usage,
        total_cost_usd,
        first_message_timestamp: first_ts,
        extra,
    };
    Ok(NormalizedSession {
        trace_type: TraceType::ClaudeCode,
        steps,
        metadata,
        tool_names,
        argument_samples,
    })
}
