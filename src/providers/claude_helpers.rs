use serde_json::Value;

use crate::models::SyntheticReason;
use crate::parser::RawEvent;

pub fn text_from_content(content: Option<&Value>) -> Option<String> {
    match content {
        Some(Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(Value::Array(blocks)) => {
            let mut out = String::new();
            for b in blocks {
                let o = b.as_object()?;
                if o.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(t) = o.get("text").and_then(Value::as_str) {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(t);
                    }
                }
            }
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        }
        _ => None,
    }
}

pub fn reasoning_from_content(content: Option<&Value>) -> Option<(String, Option<String>)> {
    let blocks = content?.as_array()?;
    for b in blocks {
        let o = b.as_object()?;
        if o.get("type").and_then(Value::as_str) == Some("thinking") {
            let text = o
                .get("thinking")
                .and_then(Value::as_str)
                .filter(|t| !t.is_empty())
                .or_else(|| o.get("text").and_then(Value::as_str))
                .unwrap_or("")
                .to_string();
            if !text.is_empty() {
                let sig = o.get("signature").and_then(Value::as_str).map(String::from);
                return Some((text, sig));
            }
        }
    }
    None
}

pub struct ToolUseInfo {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

pub fn tool_uses_from_content(content: Option<&Value>) -> Vec<ToolUseInfo> {
    let Some(Value::Array(blocks)) = content else {
        return vec![];
    };
    let mut out = vec![];
    for b in blocks {
        let Some(o) = b.as_object() else { continue };
        if o.get("type").and_then(Value::as_str) != Some("tool_use") {
            continue;
        }
        let id = o
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let name = o
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let args = o
            .get("input")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        if !id.is_empty() && !name.is_empty() {
            out.push(ToolUseInfo {
                id,
                name,
                arguments: args,
            });
        }
    }
    out
}

pub fn tool_result_text(block: &serde_json::Map<String, Value>) -> String {
    let Some(c) = block.get("content") else {
        return String::new();
    };
    match c {
        Value::String(s) => s.clone(),
        Value::Array(blocks) => {
            let mut out = String::new();
            for b in blocks {
                let Some(o) = b.as_object() else { continue };
                if o.get("type").and_then(Value::as_str) == Some("text") {
                    if let Some(t) = o.get("text").and_then(Value::as_str) {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        out.push_str(t);
                    }
                }
            }
            out
        }
        _ => String::new(),
    }
}

pub fn is_synthetic_artifact(ev: &RawEvent, content: &str) -> Option<SyntheticReason> {
    if ev.field_bool("isApiErrorMessage").unwrap_or(false) {
        let kind = ev.field_str("error").unwrap_or("").to_lowercase();
        if content.to_lowercase().contains("session limit") {
            return if kind.contains("rate_limit") {
                Some(SyntheticReason::RateLimit)
            } else if kind.contains("session_limit") {
                Some(SyntheticReason::SessionLimit)
            } else {
                Some(SyntheticReason::ApiError {
                    kind: kind.to_string(),
                })
            };
        }
    }
    if let Some(msg) = ev.message.as_ref().and_then(|m| m.as_object()) {
        if msg.get("model").and_then(Value::as_str) == Some("<synthetic>")
            && content.trim() == "No response requested."
        {
            return Some(SyntheticReason::NoResponseRequested);
        }
    }
    None
}

pub fn is_local_command_artifact(text: &str) -> bool {
    let s = text.trim();
    s.starts_with("<local-command-caveat>")
        || s.starts_with("<local-command-stdout>")
        || s.starts_with("<local-command-stderr>")
}

pub fn promote_goal_command(text: &str) -> Option<String> {
    let s = text.trim();
    if let Some(rest) = s.strip_prefix("/goal") {
        let rest = rest.trim();
        if !rest.is_empty() {
            return Some(rest.to_string());
        }
    }
    None
}

pub fn system_context_text(ev: &RawEvent, subtype: &str) -> Option<String> {
    match subtype {
        "" | "local_command" | "turn_duration" => None,
        "stop_hook_summary" => {
            let items: Vec<&str> = ev
                .field("hookAdditionalContext")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if items.is_empty() {
                None
            } else {
                Some(format!(
                    "Claude Code stop hook context:\n{}",
                    items.join("\n")
                ))
            }
        }
        "away_summary" => ev
            .field_str("content")
            .map(|c| format!("Claude Code away summary:\n{}", c.trim())),
        "informational" => ev
            .field_str("content")
            .map(|c| format!("Claude Code notice:\n{}", c.trim())),
        _ => ev
            .field_str("content")
            .map(|c| format!("Claude Code system event ({subtype}):\n{}", c.trim())),
    }
}

pub fn should_drop_assistant_fragment(ev: &RawEvent) -> bool {
    if !matches!(ev.r#type.as_deref(), Some("assistant")) {
        return false;
    }
    let Some(msg) = ev.message.as_ref().and_then(Value::as_object) else {
        return false;
    };
    let content = msg.get("content");
    let blocks = match content {
        Some(Value::Array(b)) => b,
        _ => return false,
    };
    if blocks.is_empty() {
        return true;
    }
    if blocks.len() == 1 {
        if let Some(o) = blocks[0].as_object() {
            if o.get("type").and_then(Value::as_str) == Some("text") {
                let text = o.get("text").and_then(Value::as_str).unwrap_or("");
                if text.trim().is_empty() {
                    return true;
                }
            }
        }
    }
    false
}

pub fn capture_metadata(
    ev: &RawEvent,
    session_id: &mut Option<String>,
    _model: &mut Option<String>,
    cwd: &mut Option<String>,
    cli_version: &mut Option<String>,
) {
    if let Some(s) = ev
        .field_str("sessionId")
        .or_else(|| ev.field_str("session_id"))
    {
        session_id.get_or_insert_with(|| s.to_string());
    }
    if let Some(v) = ev.field_str("version") {
        cli_version.get_or_insert_with(|| v.to_string());
    }
    if let Some(c) = ev.field_str("cwd") {
        cwd.get_or_insert_with(|| c.to_string());
    }
}

pub fn capture_usage(ev: &RawEvent, usage: &mut Option<Value>, cost: &mut Option<f64>) {
    if let Some(u) = ev.field("usage") {
        usage.get_or_insert_with(|| u.clone());
    }
    if let Some(c) = ev.field_f64("total_cost_usd") {
        cost.get_or_insert_with(|| c);
    }
}

pub fn has_tool_result_blocks(content: Option<&Value>) -> bool {
    content.and_then(Value::as_array).is_some_and(|blocks| {
        blocks.iter().any(|b| {
            b.as_object()
                .and_then(|o| o.get("type"))
                .and_then(Value::as_str)
                == Some("tool_result")
        })
    })
}

pub fn user_content_text(ev: &RawEvent) -> Option<String> {
    let msg = ev.message.as_ref().and_then(Value::as_object)?;
    let content = text_from_content(msg.get("content"))?;
    let cmd = extract_local_command(&content);
    match cmd {
        Some((name, args)) if name == "/goal" && !args.is_empty() => Some(args.to_string()),
        Some(_) => None,
        None => Some(content),
    }
}

pub fn extract_local_command(text: &str) -> Option<(&str, &str)> {
    let s = text.trim();
    if !s.starts_with('/') {
        return None;
    }
    let end = s
        .find(|c: char| c.is_whitespace() || c == '\n')
        .unwrap_or(s.len());
    let name = &s[..end];
    let args = s[end..].trim();
    Some((name, args))
}

pub fn queued_command_content(ev: &RawEvent) -> Option<String> {
    let att = ev.attachment.as_ref().and_then(Value::as_object)?;
    if att.get("type").and_then(Value::as_str) != Some("queued_command") {
        return None;
    }
    let mode = att.get("commandMode").and_then(Value::as_str);
    if !matches!(mode, None | Some("prompt")) {
        return None;
    }
    att.get("prompt")
        .and_then(Value::as_str)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}
