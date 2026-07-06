use serde_json::json;
use serde_json::Value;

pub fn codex_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "apply_patch".into(),
            "Apply a unified patch to files in the workspace.".into(),
            json!({"type": "object", "properties": {"patch": {"type": "string"}}, "required": ["patch"], "additionalProperties": true}),
        ),
        (
            "bash".into(),
            "Run shell commands in the workspace.".into(),
            json!({"type": "object", "properties": {"command": {"type": "string"}, "timeout_ms": {"type": "integer"}, "workdir": {"type": "string"}}, "required": ["command"], "additionalProperties": true}),
        ),
        (
            "exec_command".into(),
            "Run a shell command in the workspace.".into(),
            json!({"type": "object", "properties": {"cmd": {"type": "string"}, "workdir": {"type": "string"}, "yield_time_ms": {"type": "integer"}, "max_output_tokens": {"type": "integer"}, "shell": {"type": "string"}, "login": {"type": "boolean"}, "tty": {"type": "boolean"}, "justification": {"type": "string"}, "prefix_rule": {"type": "array"}, "sandbox_permissions": {"type": "string"}}, "required": ["cmd"], "additionalProperties": true}),
        ),
        (
            "update_plan".into(),
            "Update the current task plan.".into(),
            json!({"type": "object", "properties": {"explanation": {"type": "string"}, "plan": {"type": "array"}}, "required": ["plan"], "additionalProperties": true}),
        ),
        (
            "view_image".into(),
            "Inspect a local image file.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "detail": {"type": "string"}}, "required": ["path"], "additionalProperties": true}),
        ),
    ]
}
