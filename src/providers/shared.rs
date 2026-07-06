#![allow(dead_code)]

use serde_json::Value;

use crate::models::{Message, Role};

pub fn normalize_role(role: &str) -> &str {
    match role {
        "developer" => "system",
        "model" => "assistant",
        other => other,
    }
}

pub fn is_user_role(role: &str) -> bool {
    normalize_role(role) == "user"
}

pub fn first_text_block(content: Option<&Value>) -> String {
    let blocks = match content {
        Some(Value::Array(b)) => b,
        _ => return String::new(),
    };
    let mut parts = Vec::new();
    for b in blocks {
        let Some(o) = b.as_object() else { continue };
        let t = o.get("type").and_then(Value::as_str).unwrap_or("");
        if matches!(t, "text" | "input_text" | "output_text") {
            if let Some(text) = o.get("text").and_then(Value::as_str) {
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }
    }
    if parts.is_empty() {
        return String::new();
    }
    parts.join("\n").trim().to_string()
}

pub fn parse_function_arguments(args: &Value) -> Value {
    match args {
        Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone())),
        other => other.clone(),
    }
}

pub fn normalize_json_like_value(v: &Value) -> Value {
    match v {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.starts_with('{') || trimmed.starts_with('[') {
                serde_json::from_str(trimmed).unwrap_or_else(|_| Value::String(s.clone()))
            } else {
                Value::String(s.clone())
            }
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), normalize_json_like_value(v));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.iter().map(normalize_json_like_value).collect()),
        other => other.clone(),
    }
}

pub fn append_or_merge_assistant_message(messages: &mut Vec<Message>, new: Message) {
    if new.role != Role::Assistant {
        messages.push(new);
        return;
    }
    if let Some(last) = messages.last_mut() {
        if last.role == Role::Assistant {
            if let Some(c) = new.content {
                if last.content.as_deref().is_none_or(|s| s.is_empty()) {
                    last.content = Some(c);
                } else if let Some(ref mut lc) = last.content {
                    lc.push('\n');
                    lc.push_str(&c);
                }
            }
            if new.reasoning_content.is_some() {
                last.reasoning_content = new.reasoning_content.or(last.reasoning_content.take());
            }
            if let Some(tc) = new.tool_calls {
                last.tool_calls.get_or_insert_with(Vec::new).extend(tc);
            }
            if new.teich_provider_error.is_some() {
                last.teich_provider_error = new
                    .teich_provider_error
                    .or(last.teich_provider_error.take());
            }
            return;
        }
    }
    messages.push(new);
}

pub fn reasoning_summary(payload: &Value) -> Option<String> {
    let o = payload.as_object()?;
    if let Some(text) = o.get("reasoning_summary").and_then(Value::as_str) {
        if !text.is_empty() {
            return Some(text.to_string());
        }
    }
    None
}

pub fn epoch_ms_to_iso(ms: f64) -> String {
    let ms = ms as i64;
    let secs = ms / 1000;
    let micros = (ms % 1000).abs() * 1000;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    let (y, m, d) = civil_from_days(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
        y, m, d, hours, minutes, seconds, micros
    )
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = y + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

pub fn message_content_and_inline_reasoning(content: &Value) -> (String, Option<String>) {
    match content {
        Value::String(s) => (s.clone(), None),
        Value::Array(blocks) => {
            let mut parts = Vec::new();
            let mut reasoning = None;
            for b in blocks {
                let Some(o) = b.as_object() else { continue };
                let t = o.get("type").and_then(Value::as_str).unwrap_or("");
                match t {
                    "text" | "input_text" | "output_text" => {
                        if let Some(text) = o.get("text").and_then(Value::as_str) {
                            if !text.is_empty() {
                                parts.push(text.to_string());
                            }
                        }
                    }
                    "reasoning" | "thinking" => {
                        let text = o
                            .get("thinking")
                            .or_else(|| o.get("text"))
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if !text.is_empty() {
                            reasoning = Some(text.to_string());
                        }
                    }
                    _ => {}
                }
            }
            (parts.join("\n"), reasoning)
        }
        _ => (String::new(), None),
    }
}
