use serde_json::{json, Value};

/// 10 droid-specific builtin tools.
pub fn droid_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "Read".into(),
            "Read a file from the filesystem.".into(),
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "offset": {"type": "number"},
                    "limit": {"type": "number"},
                    "image_quality": {"type": "string"}
                },
                "required": ["file_path"],
                "additionalProperties": true
            }),
        ),
        (
            "Edit".into(),
            "Replace a string in a file.".into(),
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "old_str": {"type": "string"},
                    "new_str": {"type": "string"}
                },
                "required": ["file_path", "old_str", "new_str"],
                "additionalProperties": true
            }),
        ),
        (
            "Create".into(),
            "Create a new file with the given content.".into(),
            json!({
                "type": "object",
                "properties": {
                    "file_path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["file_path", "content"],
                "additionalProperties": true
            }),
        ),
        (
            "Execute".into(),
            "Run a shell command.".into(),
            json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "summary": {"type": "string"},
                    "riskLevel": {"type": "string"},
                    "riskLevelReason": {"type": "string"},
                    "timeout": {"type": "number"},
                    "fireAndForget": {"type": "boolean"}
                },
                "required": ["command"],
                "additionalProperties": true
            }),
        ),
        (
            "LS".into(),
            "List the contents of a directory.".into(),
            json!({
                "type": "object",
                "properties": {
                    "directory_path": {"type": "string"}
                },
                "required": ["directory_path"],
                "additionalProperties": true
            }),
        ),
        (
            "Glob".into(),
            "Find files matching glob patterns.".into(),
            json!({
                "type": "object",
                "properties": {
                    "patterns": {"type": "array", "items": {"type": "string"}},
                    "folder": {"type": "string"}
                },
                "required": ["patterns"],
                "additionalProperties": true
            }),
        ),
        (
            "Grep".into(),
            "Search file contents for a pattern.".into(),
            json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "output_mode": {"type": "string"},
                    "case_insensitive": {"type": "boolean"},
                    "fixed_string": {"type": "boolean"},
                    "head_limit": {"type": "number"}
                },
                "required": ["pattern"],
                "additionalProperties": true
            }),
        ),
        (
            "TodoWrite".into(),
            "Update the task list.".into(),
            json!({
                "type": "object",
                "properties": {
                    "todos": {"type": "string"}
                },
                "required": ["todos"],
                "additionalProperties": true
            }),
        ),
        (
            "FetchUrl".into(),
            "Fetch the contents of a URL.".into(),
            json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"}
                },
                "required": ["url"],
                "additionalProperties": true
            }),
        ),
        (
            "Skill".into(),
            "Invoke a skill.".into(),
            json!({
                "type": "object",
                "properties": {
                    "skill": {"type": "string"}
                },
                "required": ["skill"],
                "additionalProperties": true
            }),
        ),
    ]
}
