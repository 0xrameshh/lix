use serde_json::{json, Value};

pub fn pi_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "bash".into(),
            "Run shell commands in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "command": {"type": "string"},
                    "cmd": {"type": "string"},
                    "cwd": {"type": "string"},
                    "description": {"type": "string"},
                    "timeout": {"type": "integer"}
                },
                "anyOf": [{"required": ["command"]}, {"required": ["cmd"]}],
                "additionalProperties": true
            }),
        ),
        (
            "read".into(),
            "Read file contents from the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "file_path": {"type": "string"},
                    "offset": {"type": "integer"},
                    "limit": {"type": "integer"}
                },
                "anyOf": [{"required": ["path"]}, {"required": ["file_path"]}],
                "additionalProperties": true
            }),
        ),
        (
            "read_file".into(),
            "Read file contents from the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"}
                },
                "required": ["path"],
                "additionalProperties": true
            }),
        ),
        (
            "write".into(),
            "Write file contents in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "file_path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["content"],
                "anyOf": [{"required": ["path"]}, {"required": ["file_path"]}],
                "additionalProperties": true
            }),
        ),
        (
            "write_file".into(),
            "Write file contents in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["path", "content"],
                "additionalProperties": true
            }),
        ),
        (
            "edit".into(),
            "Edit file contents in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "file_path": {"type": "string"},
                    "edits": {"type": "array"}
                },
                "required": ["edits"],
                "anyOf": [{"required": ["path"]}, {"required": ["file_path"]}],
                "additionalProperties": true
            }),
        ),
    ]
}

pub fn openclaw_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "agents_list".into(),
            "List agent ids allowed for subagent spawning.".into(),
            json!({"type": "object", "properties": {}, "additionalProperties": true}),
        ),
        (
            "apply_patch".into(),
            "Apply a patch to one or more files using the OpenClaw apply_patch format.".into(),
            json!({"type": "object", "properties": {"input": {"type": "string"}, "patch": {"type": "string"}}, "additionalProperties": true, "anyOf": [{"required": ["input"]}, {"required": ["patch"]}]}),
        ),
        (
            "browser".into(),
            "Control a web browser.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "url": {"type": "string"}, "selector": {"type": "string"}, "text": {"type": "string"}, "target": {"type": "string"}, "profile": {"type": "string"}, "timeoutMs": {"type": "integer"}, "node": {"type": "string"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "canvas".into(),
            "Present, evaluate, or snapshot the OpenClaw Canvas.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "url": {"type": "string"}, "javaScript": {"type": "string"}, "jsonl": {"type": "string"}, "jsonlPath": {"type": "string"}, "outputFormat": {"type": "string"}, "timeoutMs": {"type": "integer"}, "delayMs": {"type": "integer"}, "quality": {"type": "number"}, "maxWidth": {"type": "integer"}, "node": {"type": "string"}, "target": {"type": "string"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "cron".into(),
            "Manage cron jobs and wake events.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "includeDisabled": {"type": "boolean"}, "job": {"type": "object"}, "jobId": {"type": "string"}, "patch": {"type": "object"}, "text": {"type": "string"}, "mode": {"type": "string"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "edit".into(),
            "Make precise edits to files in the workspace.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "file_path": {"type": "string"}, "old_string": {"type": "string"}, "new_string": {"type": "string"}, "oldText": {"type": "string"}, "newText": {"type": "string"}, "edits": {"type": "array"}, "replace_all": {"type": "boolean"}}, "additionalProperties": true}),
        ),
        (
            "exec".into(),
            "Run shell commands in the OpenClaw environment.".into(),
            json!({"type": "object", "properties": {"command": {"type": "string"}, "cmd": {"type": "string"}, "cwd": {"type": "string"}, "workdir": {"type": "string"}, "env": {"type": "object"}, "yieldMs": {"type": "integer"}, "timeout": {"type": "integer"}, "timeoutSec": {"type": "integer"}, "background": {"type": "boolean"}, "pty": {"type": "boolean"}, "host": {"type": "string"}, "node": {"type": "string"}, "security": {"type": "string"}, "ask": {"type": "string"}}, "additionalProperties": true, "anyOf": [{"required": ["command"]}, {"required": ["cmd"]}]}),
        ),
        (
            "find".into(),
            "Find files by glob pattern.".into(),
            json!({"type": "object", "properties": {"pattern": {"type": "string"}, "glob": {"type": "string"}, "path": {"type": "string"}, "limit": {"type": "integer"}}, "additionalProperties": true, "anyOf": [{"required": ["pattern"]}, {"required": ["glob"]}]}),
        ),
        (
            "gateway".into(),
            "Restart, apply config, or run updates on the OpenClaw gateway.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "config": {"type": "object"}, "patch": {"type": "object"}, "command": {"type": "string"}, "timeoutMs": {"type": "integer"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "grep".into(),
            "Search file contents for patterns.".into(),
            json!({"type": "object", "properties": {"pattern": {"type": "string"}, "query": {"type": "string"}, "path": {"type": "string"}, "include": {"type": "string"}, "glob": {"type": "string"}, "case_sensitive": {"type": "boolean"}, "context": {"type": "integer"}, "head_limit": {"type": "integer"}, "output_mode": {"type": "string"}}, "additionalProperties": true, "anyOf": [{"required": ["pattern"]}, {"required": ["query"]}]}),
        ),
        (
            "image".into(),
            "Analyze an image with the configured image model.".into(),
            json!({"type": "object", "properties": {"image": {"type": "string"}, "image_url": {"type": "string"}, "prompt": {"type": "string"}, "model": {"type": "string"}, "maxBytesMb": {"type": "number"}}, "additionalProperties": true, "anyOf": [{"required": ["image"]}, {"required": ["image_url"]}]}),
        ),
        (
            "ls".into(),
            "List directory contents.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "limit": {"type": "integer"}}, "additionalProperties": true}),
        ),
        (
            "message".into(),
            "Send messages and channel actions.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "to": {"type": "string"}, "target": {"type": "string"}, "message": {"type": "string"}, "content": {"type": "string"}, "text": {"type": "string"}, "channel": {"type": "string"}, "thread": {"type": "string"}, "attachments": {"type": "array"}, "buttons": {"type": "array"}}, "additionalProperties": true}),
        ),
        (
            "nodes".into(),
            "List, describe, notify, capture, or run commands on paired nodes.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "node": {"type": "string"}, "requestId": {"type": "string"}, "title": {"type": "string"}, "body": {"type": "string"}, "priority": {"type": "string"}, "delivery": {"type": "string"}, "facing": {"type": "string"}, "deviceId": {"type": "string"}, "duration": {"type": "number"}, "durationMs": {"type": "integer"}, "includeAudio": {"type": "boolean"}, "fps": {"type": "number"}, "screenIndex": {"type": "integer"}, "outPath": {"type": "string"}, "command": {"type": "string"}, "cwd": {"type": "string"}, "env": {"type": "object"}, "timeoutMs": {"type": "integer"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "process".into(),
            "Manage background exec sessions.".into(),
            json!({"type": "object", "properties": {"action": {"type": "string"}, "sessionId": {"type": "string"}, "session_id": {"type": "string"}, "data": {"type": "string"}, "keys": {"type": "array", "items": {"type": "string"}}, "text": {"type": "string"}, "offset": {"type": "integer"}, "limit": {"type": "integer"}, "timeout": {"type": "integer"}, "eof": {"type": "boolean"}}, "additionalProperties": true, "required": ["action"]}),
        ),
        (
            "read".into(),
            "Read file contents from the workspace.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "file_path": {"type": "string"}, "offset": {"type": "integer"}, "limit": {"type": "integer"}, "image_quality": {"type": "string"}}, "additionalProperties": true, "anyOf": [{"required": ["path"]}, {"required": ["file_path"]}]}),
        ),
        (
            "session_status".into(),
            "Show a status card for a session.".into(),
            json!({"type": "object", "properties": {"sessionKey": {"type": "string"}, "sessionId": {"type": "string"}, "model": {"type": "string"}}, "additionalProperties": true}),
        ),
        (
            "sessions_history".into(),
            "Fetch history for another session or sub-agent.".into(),
            json!({"type": "object", "properties": {"sessionKey": {"type": "string"}, "sessionId": {"type": "string"}, "limit": {"type": "integer"}, "includeTools": {"type": "boolean"}}, "additionalProperties": true, "anyOf": [{"required": ["sessionKey"]}, {"required": ["sessionId"]}]}),
        ),
        (
            "sessions_list".into(),
            "List other sessions.".into(),
            json!({"type": "object", "properties": {"kind": {"type": "string"}, "kinds": {"type": "array", "items": {"type": "string"}}, "limit": {"type": "integer"}, "activeMinutes": {"type": "integer"}, "messageLimit": {"type": "integer"}}, "additionalProperties": true}),
        ),
        (
            "sessions_send".into(),
            "Send a message to another session or sub-agent.".into(),
            json!({"type": "object", "properties": {"sessionKey": {"type": "string"}, "sessionId": {"type": "string"}, "agentId": {"type": "string"}, "label": {"type": "string"}, "message": {"type": "string"}, "timeoutSeconds": {"type": "number"}}, "additionalProperties": true, "required": ["message"]}),
        ),
        (
            "sessions_spawn".into(),
            "Spawn a sub-agent session.".into(),
            json!({"type": "object", "properties": {"task": {"type": "string"}, "label": {"type": "string"}, "agentId": {"type": "string"}, "model": {"type": "string"}, "thinking": {"type": "string"}, "runTimeoutSeconds": {"type": "number"}, "timeoutSeconds": {"type": "number"}, "cleanup": {"type": "boolean"}}, "additionalProperties": true, "required": ["task"]}),
        ),
        (
            "tts".into(),
            "Speak text through a configured text-to-speech channel.".into(),
            json!({"type": "object", "properties": {"text": {"type": "string"}, "channel": {"type": "string"}, "voice": {"type": "string"}}, "additionalProperties": true, "required": ["text"]}),
        ),
        (
            "web_fetch".into(),
            "Fetch and extract readable content from a URL.".into(),
            json!({"type": "object", "properties": {"url": {"type": "string"}, "extractMode": {"type": "string"}, "maxChars": {"type": "integer"}}, "additionalProperties": true, "required": ["url"]}),
        ),
        (
            "web_search".into(),
            "Search the web.".into(),
            json!({"type": "object", "properties": {"query": {"type": "string"}, "count": {"type": "integer"}, "country": {"type": "string"}, "search_lang": {"type": "string"}, "ui_lang": {"type": "string"}, "freshness": {"type": "string"}}, "additionalProperties": true, "required": ["query"]}),
        ),
        (
            "write".into(),
            "Create or overwrite files in the workspace.".into(),
            json!({"type": "object", "properties": {"path": {"type": "string"}, "file_path": {"type": "string"}, "content": {"type": "string"}}, "additionalProperties": true, "required": ["content"], "anyOf": [{"required": ["path"]}, {"required": ["file_path"]}]}),
        ),
    ]
}
