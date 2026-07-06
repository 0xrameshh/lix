use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub mod builtins_codex;
pub mod builtins_cursor;
pub mod builtins_droid;
pub mod builtins_hermes;
pub mod builtins_pi;

use crate::models::{Tool, ToolEntryKind, ToolFunctionSchema, TraceType};

/// Build tool list by provider strategy:
/// - Claude Code: inferred params for observed tools (richer schemas from trace data)
/// - All other providers: hardcoded builtin params (complete schemas)
/// - Unobserved builtins: always included with hardcoded params
/// - Unknown observed tools: inferred params as fallback
pub fn build_tools(
    builtin_names: &[(String, String, Value)],
    observed_names: &BTreeSet<String>,
    argument_samples: &BTreeMap<String, Vec<Value>>,
) -> Vec<Tool> {
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut out: Vec<Tool> = Vec::new();

    // Index builtin descriptions for fallback
    let builtin_descs: BTreeMap<&str, &str> = builtin_names
        .iter()
        .map(|(n, d, _)| (n.as_str(), d.as_str()))
        .collect();

    for (name, desc, params) in builtin_names {
        // Builtins always use hardcoded params
        if seen.insert(name.clone()) {
            out.push(Tool {
                kind: ToolEntryKind::Function,
                function: ToolFunctionSchema {
                    name: name.clone(),
                    description: Some(desc.clone()),
                    parameters: params.clone(),
                },
            });
        }
    }

    // Observed tools not in builtins → inferred params
    for name in observed_names {
        if seen.insert(name.clone()) {
            let desc = builtin_descs.get(name.as_str()).map(|&d| d.to_string());
            let params = argument_samples
                .get(name)
                .map(|samples| infer_parameters(samples))
                .unwrap_or_else(empty_schema);
            out.push(Tool {
                kind: ToolEntryKind::Function,
                function: ToolFunctionSchema {
                    name: name.clone(),
                    description: desc,
                    parameters: params,
                },
            });
        }
    }

    out.sort_by(|a, b| a.function.name.cmp(&b.function.name));
    out
}

pub fn claude_code_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "Bash".into(),
            "Run a shell command.".into(),
            json!({"type": "object", "properties": {"command": {"type": "string"}, "description": {"type": "string"}, "timeout": {"type": "integer"}, "run_in_background": {"type": "boolean"}}, "required": ["command"], "additionalProperties": true}),
        ),
        (
            "BashOutput".into(),
            "Read output from a running background shell command.".into(),
            json!({"type": "object", "properties": {"bash_id": {"type": "string"}, "filter": {"type": "string"}}, "required": ["bash_id"], "additionalProperties": true}),
        ),
        (
            "Edit".into(),
            "Replace text in an existing file.".into(),
            json!({"type": "object", "properties": {"file_path": {"type": "string"}, "old_string": {"type": "string"}, "new_string": {"type": "string"}, "replace_all": {"type": "boolean"}}, "required": ["file_path", "old_string", "new_string"], "additionalProperties": true}),
        ),
        (
            "Glob".into(),
            "Find files by glob pattern.".into(),
            json!({"type": "object", "properties": {"pattern": {"type": "string"}, "path": {"type": "string"}}, "required": ["pattern"], "additionalProperties": true}),
        ),
        (
            "Grep".into(),
            "Search file contents by pattern.".into(),
            json!({"type": "object", "properties": {"pattern": {"type": "string"}, "path": {"type": "string"}, "include": {"type": "string"}, "output_mode": {"type": "string"}, "-A": {"type": "integer"}, "-B": {"type": "integer"}, "-C": {"type": "integer"}, "head_limit": {"type": "integer"}}, "required": ["pattern"], "additionalProperties": true}),
        ),
        (
            "KillBash".into(),
            "Stop a running background shell command.".into(),
            json!({"type": "object", "properties": {"shell_id": {"type": "string"}}, "required": ["shell_id"], "additionalProperties": true}),
        ),
        (
            "LS".into(),
            "List files and directories.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "ignore": {"type": "array", "items": {"type": "string"}}}, "required": ["path"], "additionalProperties": true}),
        ),
        (
            "MultiEdit".into(),
            "Apply multiple text replacements to one file.".into(),
            json!({"type": "object", "properties": {"file_path": {"type": "string"}, "edits": {"type": "array", "items": {"type": "object", "properties": {"old_string": {"type": "string"}, "new_string": {"type": "string"}, "replace_all": {"type": "boolean"}}, "required": ["old_string", "new_string"], "additionalProperties": true}}}, "required": ["file_path", "edits"], "additionalProperties": true}),
        ),
        (
            "NotebookEdit".into(),
            "Edit a Jupyter notebook cell.".into(),
            json!({"type": "object", "properties": {"notebook_path": {"type": "string"}, "cell_id": {"type": "string"}, "new_source": {"type": "string"}, "cell_type": {"type": "string"}, "edit_mode": {"type": "string"}}, "required": ["notebook_path", "new_source"], "additionalProperties": true}),
        ),
        (
            "NotebookRead".into(),
            "Read a Jupyter notebook.".into(),
            json!({"type": "object", "properties": {"notebook_path": {"type": "string"}}, "required": ["notebook_path"], "additionalProperties": true}),
        ),
        (
            "Read".into(),
            "Read a file.".into(),
            json!({"type": "object", "properties": {"file_path": {"type": "string"}, "offset": {"type": "integer"}, "limit": {"type": "integer"}}, "required": ["file_path"], "additionalProperties": true}),
        ),
        (
            "Task".into(),
            "Launch a subagent to complete a delegated task.".into(),
            json!({"type": "object", "properties": {"description": {"type": "string"}, "prompt": {"type": "string"}, "subagent_type": {"type": "string"}}, "required": ["prompt"], "additionalProperties": true}),
        ),
        (
            "TodoWrite".into(),
            "Create or update the task list.".into(),
            json!({"type": "object", "properties": {"todos": {"type": "array", "items": {"type": "object", "properties": {"content": {"type": "string"}, "status": {"type": "string"}, "priority": {"type": "string"}, "id": {"type": "string"}}, "required": ["content", "status", "priority", "id"], "additionalProperties": true}}}, "required": ["todos"], "additionalProperties": true}),
        ),
        (
            "WebFetch".into(),
            "Fetch web content from a URL.".into(),
            json!({"type": "object", "properties": {"url": {"type": "string"}, "prompt": {"type": "string"}}, "required": ["url", "prompt"], "additionalProperties": true}),
        ),
        (
            "WebSearch".into(),
            "Search the web.".into(),
            json!({"type": "object", "properties": {"query": {"type": "string"}, "allowed_domains": {"type": "array", "items": {"type": "string"}}, "blocked_domains": {"type": "array", "items": {"type": "string"}}}, "required": ["query"], "additionalProperties": true}),
        ),
        (
            "Write".into(),
            "Write a file.".into(),
            json!({"type": "object", "properties": {"file_path": {"type": "string"}, "content": {"type": "string"}}, "required": ["file_path", "content"], "additionalProperties": true}),
        ),
        (
            "ToolSearch".into(),
            "Load deferred Claude Desktop or MCP tools by search query.".into(),
            json!({"type": "object", "properties": {"query": {"type": "string"}, "max_results": {"type": "integer"}}, "required": ["query"], "additionalProperties": true}),
        ),
    ]
}

pub fn provider_builtins(tt: TraceType) -> Vec<(String, String, Value)> {
    match tt {
        TraceType::ClaudeCode => claude_code_builtins(),
        TraceType::Droid => builtins_droid::droid_builtins(),
        TraceType::Cursor => builtins_cursor::cursor_builtins(),
        TraceType::Pi => builtins_pi::pi_builtins(),
        TraceType::Codex => builtins_codex::codex_builtins(),
        TraceType::Openclaw => builtins_pi::openclaw_builtins(),
        TraceType::Hermes | TraceType::ExternalAgent => builtins_hermes::hermes_builtins(),
        _ => vec![],
    }
}

pub fn infer_parameters(samples: &[Value]) -> Value {
    if samples.is_empty() {
        return empty_schema();
    }
    let mut props: Map<String, Value> = Map::new();
    for sample in samples {
        if let Value::Object(o) = sample {
            for (k, v) in o {
                props.entry(k.clone()).or_insert_with(|| infer_type(v));
            }
        }
    }
    let mut required: Vec<String> = props.keys().cloned().collect();
    required.sort();
    json!({
        "type": "object",
        "properties": props,
        "required": required,
        "additionalProperties": true
    })
}

fn infer_type(v: &Value) -> Value {
    match v {
        Value::Null => json!({"type": "string"}),
        Value::Bool(_) => json!({"type": "boolean"}),
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                json!({"type": "integer"})
            } else {
                json!({"type": "number"})
            }
        }
        Value::String(_) => json!({"type": "string"}),
        Value::Array(_) => json!({"type": "array"}),
        Value::Object(_) => json!({"type": "object"}),
    }
}

fn empty_schema() -> Value {
    json!({"type": "object", "properties": {}, "required": [], "additionalProperties": true})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_parameters_from_samples() {
        let s1 = json!({"a": 1, "b": "hello"});
        let s2 = json!({"a": 2, "c": true});
        let result = infer_parameters(&[s1, s2]);
        let props = result["properties"].as_object().unwrap();
        assert!(props.contains_key("a"));
        assert!(props.contains_key("b"));
        assert!(props.contains_key("c"));
        assert_eq!(result["type"], "object");
    }

    #[test]
    fn builtin_tools_always_present() {
        let builtins = claude_code_builtins();
        let names: BTreeSet<_> = builtins.iter().map(|(n, _, _)| n.clone()).collect();
        assert!(names.contains("Bash"));
        assert!(names.contains("Read"));
        assert!(names.contains("Write"));
        assert!(names.contains("Edit"));
        assert!(names.contains("Glob"));
        assert!(names.contains("Grep"));
    }
}
