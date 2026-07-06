use serde_json::{Map, Value};

use crate::providers::shared;

pub fn codex_first_user_ts(ordered: &[Map<String, Value>]) -> Option<String> {
    for ev in ordered {
        let ev_type = ev.get("type").and_then(Value::as_str).unwrap_or("");
        if ev_type == "event_msg" {
            if let Some(payload) = ev.get("payload").and_then(Value::as_object) {
                if payload.get("type").and_then(Value::as_str) == Some("user_message") {
                    if let Some(ts) = ev.get("timestamp").and_then(Value::as_str) {
                        return Some(ts.to_string());
                    }
                }
            }
        }
    }
    for ev in ordered {
        if let Some(payload) = ev.get("payload").and_then(Value::as_object) {
            if payload.get("type").and_then(Value::as_str) == Some("message") {
                let role = payload.get("role").and_then(Value::as_str).unwrap_or("");
                if shared::normalize_role(role) == "user" {
                    if let Some(ts) = ev.get("timestamp").and_then(Value::as_str) {
                        return Some(ts.to_string());
                    }
                }
            }
        }
    }
    ordered.iter().find_map(|ev| {
        ev.get("timestamp")
            .and_then(Value::as_str)
            .map(String::from)
    })
}

pub fn payload_type(event: &Map<String, Value>) -> Option<&str> {
    if event.get("type")?.as_str()? != "response_item" {
        return None;
    }
    event.get("payload")?.as_object()?.get("type")?.as_str()
}

pub fn event_msg_payload_type(event: &Map<String, Value>) -> Option<&str> {
    if event.get("type")?.as_str()? != "event_msg" {
        return None;
    }
    event.get("payload")?.as_object()?.get("type")?.as_str()
}

pub fn is_assistant_message_event(event: &Map<String, Value>) -> bool {
    if payload_type(event) != Some("message") {
        return false;
    }
    event
        .get("payload")
        .and_then(|p| p.as_object())
        .and_then(|p| p.get("role"))
        .and_then(Value::as_str)
        == Some("assistant")
}

pub fn has_assistant_message_text(event: &Map<String, Value>) -> bool {
    if !is_assistant_message_event(event) {
        return false;
    }
    let payload = match event.get("payload").and_then(Value::as_object) {
        Some(p) => p,
        None => return false,
    };
    !shared::first_text_block(payload.get("content"))
        .trim()
        .is_empty()
}

pub fn is_tool_call_fragment(event: &Map<String, Value>) -> bool {
    matches!(
        payload_type(event),
        Some("function_call" | "custom_tool_call")
    )
}

pub fn is_output_fragment(event: &Map<String, Value>) -> bool {
    is_tool_call_fragment(event) || has_assistant_message_text(event)
}

pub fn is_prefix_marker(event: &Map<String, Value>) -> bool {
    matches!(
        event_msg_payload_type(event),
        Some("agent_reasoning" | "agent_message")
    )
}

pub fn is_reorderable_suffix_event(event: &Map<String, Value>, prefix_has_reasoning: bool) -> bool {
    if is_tool_call_fragment(event) {
        return true;
    }
    if !prefix_has_reasoning {
        return false;
    }
    is_assistant_message_event(event) || event_msg_payload_type(event) == Some("agent_message")
}

pub fn reasoning_text(event: &Map<String, Value>) -> Option<String> {
    if payload_type(event) != Some("reasoning") {
        return None;
    }
    let payload = event.get("payload")?.as_object()?;
    match payload.get("summary")? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Array(items) => {
            let parts: Vec<String> = items
                .iter()
                .filter_map(|item| item.as_object())
                .filter_map(|obj| obj.get("text").and_then(Value::as_str))
                .filter(|s| !s.trim().is_empty())
                .map(String::from)
                .collect();
            if parts.is_empty() {
                None
            } else {
                Some(parts.join("\n"))
            }
        }
        _ => None,
    }
}
