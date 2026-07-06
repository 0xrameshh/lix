use std::collections::BTreeSet;

use serde_json::Value;

pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

pub fn pi_tool_calls(
    content_blocks: Option<&Value>,
    invalid_ids: &BTreeSet<String>,
) -> Vec<ToolCallInfo> {
    let Some(Value::Array(blocks)) = content_blocks else {
        return vec![];
    };
    let mut out = vec![];
    for b in blocks {
        let Some(o) = b.as_object() else { continue };
        if o.get("type").and_then(Value::as_str) != Some("toolCall") {
            continue;
        }
        let id = o.get("id").and_then(Value::as_str).unwrap_or("");
        if id.is_empty() || invalid_ids.contains(id) {
            continue;
        }
        let name = o
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let args = o
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        out.push(ToolCallInfo {
            id: id.to_string(),
            name,
            arguments: args,
        });
    }
    out
}

pub fn pi_reasoning(content_blocks: Option<&Value>) -> Option<String> {
    let Value::Array(blocks) = content_blocks? else {
        return None;
    };
    for b in blocks {
        let o = b.as_object()?;
        if o.get("type").and_then(Value::as_str) == Some("thinking") {
            let t = o.get("thinking").and_then(Value::as_str).unwrap_or("");
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}
