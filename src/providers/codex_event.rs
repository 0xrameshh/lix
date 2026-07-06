use crate::providers::{codex_reorder, shared};
use serde_json::{Map, Value};

pub fn normalize_single_event(mut event: Map<String, Value>) -> Map<String, Value> {
    if event.get("type").and_then(Value::as_str) != Some("response_item") {
        return event;
    }
    let payload = match event.get("payload") {
        Some(Value::Object(p)) => p,
        _ => return event,
    };
    let pt = match payload.get("type").and_then(Value::as_str) {
        Some(pt) => pt,
        None => return event,
    };

    match pt {
        "reasoning" => {
            let has_rich_summary = match payload.get("summary") {
                Some(Value::Array(items)) => items.iter().any(|item| {
                    item.as_object()
                        .and_then(|o| o.get("text"))
                        .and_then(Value::as_str)
                        .is_some_and(|s| !s.trim().is_empty())
                }),
                _ => false,
            };
            if !has_rich_summary {
                if let Some(summary) = codex_reorder::reasoning_text(&event) {
                    let mut new_payload = payload.clone();
                    new_payload.insert("summary".into(), Value::String(summary));
                    event.insert("payload".into(), Value::Object(new_payload));
                }
            }
        }
        "custom_tool_call" => {
            let name = match payload.get("name").and_then(Value::as_str) {
                Some(n) if !n.is_empty() => n.to_string(),
                _ => return event,
            };
            let call_id = match payload.get("call_id").and_then(Value::as_str) {
                Some(c) if !c.is_empty() => c.to_string(),
                _ => return event,
            };
            let raw_input = payload.get("input").cloned().unwrap_or(Value::Null);
            let args = if name == "apply_patch" {
                let input_val = match &raw_input {
                    Value::String(s) => {
                        shared::normalize_json_like_value(&Value::String(s.clone()))
                    }
                    v => shared::normalize_json_like_value(v),
                };
                let mut m = serde_json::Map::new();
                m.insert("patch".into(), input_val);
                Value::Object(m)
            } else {
                shared::parse_function_arguments(&raw_input)
            };
            let mut new_payload: Map<String, Value> = Map::new();
            for (k, v) in payload.iter() {
                if k != "type" && k != "input" {
                    new_payload.insert(k.clone(), v.clone());
                }
            }
            new_payload.insert("type".into(), Value::String("function_call".into()));
            new_payload.insert("name".into(), Value::String(name));
            new_payload.insert("call_id".into(), Value::String(call_id));
            new_payload.insert("arguments".into(), args_json(&args));
            event.insert("payload".into(), Value::Object(new_payload));
        }
        "custom_tool_call_output" => {
            let call_id = match payload.get("call_id").and_then(Value::as_str) {
                Some(c) if !c.is_empty() => c.to_string(),
                _ => return event,
            };
            let output = custom_tool_output_value(payload.get("output"));
            let mut new_payload: Map<String, Value> = Map::new();
            for (k, v) in payload.iter() {
                if k != "type" && k != "output" {
                    new_payload.insert(k.clone(), v.clone());
                }
            }
            new_payload.insert("type".into(), Value::String("function_call_output".into()));
            new_payload.insert("call_id".into(), Value::String(call_id));
            new_payload.insert("output".into(), output);
            event.insert("payload".into(), Value::Object(new_payload));
        }
        _ => {}
    }
    event
}

fn args_json(args: &Value) -> Value {
    match args {
        Value::Object(_) | Value::Array(_) => Value::String(args.to_string()),
        other => other.clone(),
    }
}

fn custom_tool_output_value(output: Option<&Value>) -> Value {
    match output {
        Some(Value::String(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return Value::String(String::new());
            }
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                if let Value::Object(ref obj) = parsed {
                    if let Some(Value::String(inner)) = obj.get("output") {
                        return Value::String(inner.clone());
                    }
                }
                shared::normalize_json_like_value(&parsed)
            } else {
                Value::String(s.clone())
            }
        }
        Some(Value::Array(blocks)) => {
            let text = shared::first_text_block(Some(&Value::Array(blocks.clone())));
            Value::String(text)
        }
        Some(v) => shared::normalize_json_like_value(v),
        None => Value::String(String::new()),
    }
}

fn reassign_timestamps(
    events: &[Map<String, Value>],
    sources: &[Map<String, Value>],
) -> Vec<Map<String, Value>> {
    let ts_keys = ["timestamp", "created_at", "createdAt"];
    events
        .iter()
        .zip(sources.iter())
        .map(|(ev, src)| {
            let mut e = ev.clone();
            let mut assigned = false;
            for k in &ts_keys {
                if let Some(ts) = src.get(*k) {
                    e.insert(k.to_string(), ts.clone());
                    assigned = true;
                } else if e.contains_key(*k) {
                    e.remove(*k);
                }
            }
            if !assigned {
                for k in &ts_keys {
                    e.remove(*k);
                }
            }
            e
        })
        .collect()
}

fn rotate_timestamps(
    first: &Map<String, Value>,
    following: &[Map<String, Value>],
) -> Vec<Map<String, Value>> {
    let events: Vec<_> = std::iter::once(first.clone())
        .chain(following.iter().cloned())
        .collect();
    let sources: Vec<_> = following
        .iter()
        .cloned()
        .chain(std::iter::once(first.clone()))
        .collect();
    reassign_timestamps(&events, &sources)
}

fn assistant_prefix_block(
    events: &[Map<String, Value>],
    start: usize,
) -> (Vec<Map<String, Value>>, usize) {
    let mut block = Vec::new();
    let mut saw_assistant = false;
    let mut saw_message = false;
    let mut idx = start;

    while idx < events.len() {
        let ev = &events[idx];
        if codex_reorder::is_prefix_marker(ev) {
            block.push(ev.clone());
            idx += 1;
            continue;
        }
        match codex_reorder::payload_type(ev) {
            Some("reasoning") => {
                if saw_message {
                    break;
                }
                block.push(ev.clone());
                saw_assistant = true;
                idx += 1;
                continue;
            }
            Some("message") if codex_reorder::is_assistant_message_event(ev) => {
                block.push(ev.clone());
                saw_assistant = true;
                saw_message = true;
                idx += 1;
                continue;
            }
            _ => {}
        }
        break;
    }

    if !saw_assistant {
        return (Vec::new(), start);
    }
    (block, idx)
}

pub fn reorder_events(normalized: &[Map<String, Value>]) -> Vec<Map<String, Value>> {
    let mut ordered: Vec<Map<String, Value>> = Vec::new();
    let mut idx = 0;

    while idx < normalized.len() {
        let (prefix_block, next_idx) = assistant_prefix_block(normalized, idx);
        if !prefix_block.is_empty() {
            let prefix_has_reasoning = prefix_block
                .iter()
                .any(|ev| codex_reorder::payload_type(ev) == Some("reasoning"));

            let mut suffix = Vec::new();
            while let Some(last) = ordered.last() {
                if codex_reorder::is_reorderable_suffix_event(last, prefix_has_reasoning) {
                    suffix.insert(0, ordered.pop().unwrap());
                } else {
                    break;
                }
            }

            if !suffix.is_empty() {
                let sources: Vec<_> = suffix
                    .iter()
                    .cloned()
                    .chain(prefix_block.iter().cloned())
                    .collect();
                let reordered: Vec<_> = prefix_block
                    .iter()
                    .cloned()
                    .chain(suffix.iter().cloned())
                    .collect();
                ordered.extend(reassign_timestamps(&reordered, &sources));
            } else {
                ordered.extend(prefix_block);
            }
            idx = next_idx;
            continue;
        }

        let event = normalized[idx].clone();
        if codex_reorder::reasoning_text(&event).is_some() {
            let mut suffix = Vec::new();
            while let Some(last) = ordered.last() {
                if codex_reorder::is_output_fragment(last) {
                    suffix.insert(0, ordered.pop().unwrap());
                } else {
                    break;
                }
            }
            if !suffix.is_empty() {
                ordered.extend(rotate_timestamps(&event, &suffix));
                idx += 1;
                continue;
            }
        }

        ordered.push(event);
        idx += 1;
    }
    ordered
}
