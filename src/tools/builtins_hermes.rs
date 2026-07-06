use serde_json::json;
use serde_json::Value;

pub fn hermes_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "delegate_task".into(),
            "Spawn isolated Hermes subagents for delegated tasks.".into(),
            json!({"type": "object", "properties": {"goal": {"type": "string"}, "context": {"type": "string"}, "toolsets": {"type": "array", "items": {"type": "string"}}, "tasks": {"type": "array", "items": {"type": "object", "properties": {"goal": {"type": "string"}, "context": {"type": "string"}, "toolsets": {"type": "array", "items": {"type": "string"}}, "acp_command": {"type": "string"}, "acp_args": {"type": "array", "items": {"type": "string"}}, "role": {"type": "string", "enum": ["leaf", "orchestrator"]}}, "required": ["goal"], "additionalProperties": true}}, "role": {"type": "string", "enum": ["leaf", "orchestrator"]}, "acp_command": {"type": "string"}, "acp_args": {"type": "array", "items": {"type": "string"}}}, "additionalProperties": true}),
        ),
        (
            "memory".into(),
            "Add, replace, or remove durable Hermes memory entries.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string", "enum": ["add", "replace", "remove"]}, "target": {"type": "string", "enum": ["memory", "user"]}, "content": {"type": "string"}, "old_text": {"type": "string"}}, "required": ["action", "target"], "additionalProperties": true}),
        ),
        (
            "patch".into(),
            "Apply targeted file edits or multi-file patches.".into(),
            json!({"type": "object", "properties": {"mode": {"type": "string", "enum": ["replace", "patch"]}, "path": {"type": "string"}, "old_string": {"type": "string"}, "new_string": {"type": "string"}, "replace_all": {"type": "boolean"}, "patch": {"type": "string"}, "cross_profile": {"type": "boolean"}}, "required": ["mode"], "additionalProperties": true}),
        ),
        (
            "process".into(),
            "Manage background processes started by Hermes terminal calls.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string", "enum": ["list", "poll", "log", "wait", "kill", "write", "submit", "close"]}, "session_id": {"type": "string"}, "data": {"type": "string"}, "timeout": {"type": "integer"}, "offset": {"type": "integer"}, "limit": {"type": "integer"}}, "required": ["action"], "additionalProperties": true}),
        ),
        (
            "read_file".into(),
            "Read a text file with optional pagination.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "offset": {"type": "integer"}, "limit": {"type": "integer"}}, "required": ["path"], "additionalProperties": true}),
        ),
        (
            "search_files".into(),
            "Search file contents or filenames.".into(),
            json!({"type": "object", "properties": {"pattern": {"type": "string"}, "target": {"type": "string", "enum": ["content", "files"]}, "path": {"type": "string"}, "file_glob": {"type": "string"}, "limit": {"type": "integer"}, "offset": {"type": "integer"}, "output_mode": {"type": "string", "enum": ["content", "files_only", "count"]}, "context": {"type": "integer"}}, "required": ["pattern"], "additionalProperties": true}),
        ),
        (
            "session_search".into(),
            "Search or inspect previous Hermes sessions.".into(),
            json!({"type": "object", "properties": {"query": {"type": "string"}, "limit": {"type": "integer"}, "sort": {"type": "string", "enum": ["newest", "oldest"]}, "session_id": {"type": "string"}, "around_message_id": {"type": "integer"}, "window": {"type": "integer"}, "role_filter": {"type": "string"}, "profile": {"type": "string"}}, "additionalProperties": true}),
        ),
        (
            "skill_manage".into(),
            "Create, patch, edit, delete, or update files for Hermes skills.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string", "enum": ["create", "patch", "edit", "delete", "write_file", "remove_file"]}, "name": {"type": "string"}, "content": {"type": "string"}, "old_string": {"type": "string"}, "new_string": {"type": "string"}, "replace_all": {"type": "boolean"}, "category": {"type": "string"}, "file_path": {"type": "string"}, "file_content": {"type": "string"}, "absorbed_into": {"type": "string"}}, "required": ["action", "name"], "additionalProperties": true}),
        ),
        (
            "skill_view".into(),
            "View a Hermes skill or one of its linked files.".into(),
            json!({"type": "object", "properties": {"name": {"type": "string"}, "file_path": {"type": "string"}}, "required": ["name"], "additionalProperties": true}),
        ),
        (
            "skills_list".into(),
            "List available Hermes skills.".into(),
            json!({"type": "object", "properties": {"category": {"type": "string"}}, "additionalProperties": true}),
        ),
        (
            "terminal".into(),
            "Run shell commands in the Hermes environment.".into(),
            json!({"type": "object", "properties": {"command": {"type": "string"}, "background": {"type": "boolean"}, "timeout": {"type": "integer"}, "workdir": {"type": "string"}, "pty": {"type": "boolean"}, "notify_on_complete": {"type": "boolean"}, "watch_patterns": {"type": "array", "items": {"type": "string"}}}, "required": ["command"], "additionalProperties": true}),
        ),
        (
            "vision_analyze".into(),
            "Load an image for visual analysis.".into(),
            json!({"type": "object", "properties": {"image_url": {"type": "string"}, "question": {"type": "string"}}, "required": ["image_url", "question"], "additionalProperties": true}),
        ),
        (
            "write_file".into(),
            "Write content to a file, replacing existing contents.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}, "cross_profile": {"type": "boolean"}}, "required": ["path", "content"], "additionalProperties": true}),
        ),
    ]
}
