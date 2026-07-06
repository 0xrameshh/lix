use serde_json::json;
use serde_json::Value;

pub fn cursor_builtins() -> Vec<(String, String, Value)> {
    vec![
        (
            "Shell".into(),
            "Run shell commands.".into(),
            json!({
                "type": "object", "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }),
        ),
        (
            "read_file".into(),
            "Read file contents from the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "target_file": {"type": "string"},
                    "start_line_one_indexed": {"type": "integer"},
                    "end_line_one_indexed_inclusive": {"type": "integer"},
                    "explanation": {"type": "string"},
                    "should_read_entire_file": {"type": "boolean"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "read_file_v2".into(),
            "Read file contents from the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "target_file": {"type": "string"},
                    "offset": {"type": "integer"},
                    "limit": {"type": "integer"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "list_dir".into(),
            "List directory contents.".into(),
            json!({
                "type": "object", "properties": {
                    "relative_workspace_path": {"type": "string"},
                    "target_directory": {"type": "string"},
                    "directory_path": {"type": "string"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "run_terminal_cmd".into(),
            "Run a terminal command in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "command": {"type": "string"},
                    "is_background": {"type": "boolean"},
                    "require_user_approval": {"type": "boolean"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "run_terminal_command_v2".into(),
            "Run a terminal command in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "command": {"type": "string"},
                    "working_directory": {"type": "string"},
                    "cwd": {"type": "string"},
                    "timeout_ms": {"type": "integer"},
                    "is_background": {"type": "boolean"},
                    "skip_approval": {"type": "boolean"},
                    "description": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "edit_file".into(),
            "Edit a file in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "target_file": {"type": "string"},
                    "path": {"type": "string"},
                    "instructions": {"type": "string"},
                    "code_edit": {"type": "string"},
                    "blocking": {"type": "boolean"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "edit_file_v2".into(),
            "Edit a file in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "target_file": {"type": "string"},
                    "relative_workspace_path": {"type": "string"},
                    "instructions": {"type": "string"},
                    "streaming_content": {"type": "string"},
                    "code_edit": {"type": "string"},
                    "no_codeblock": {"type": "boolean"},
                    "cloud_agent_edit": {"type": "boolean"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "search_replace".into(),
            "Replace matching text in a file.".into(),
            json!({
                "type": "object", "properties": {
                    "file_path": {"type": "string"},
                    "path": {"type": "string"},
                    "old_string": {"type": "string"},
                    "new_string": {"type": "string"},
                    "replace_all": {"type": "boolean"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "apply_patch".into(),
            "Apply a patch to files in the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "patch": {"type": "string"},
                    "file_path": {"type": "string"},
                    "target_file": {"type": "string"},
                    "instructions": {"type": "string"},
                    "code_edit": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "delete_file".into(),
            "Delete a file from the workspace.".into(),
            json!({
                "type": "object", "properties": {
                    "path": {"type": "string"},
                    "target_file": {"type": "string"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "file_search".into(),
            "Search for files by fuzzy path or filename.".into(),
            json!({
                "type": "object", "properties": {
                    "query": {"type": "string"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "glob_file_search".into(),
            "Search for files by glob pattern.".into(),
            json!({
                "type": "object", "properties": {
                    "glob_pattern": {"type": "string"},
                    "globPattern": {"type": "string"},
                    "target_directory": {"type": "string"},
                    "targetDirectory": {"type": "string"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "grep_search".into(),
            "Search workspace text with grep-like filters.".into(),
            json!({
                "type": "object", "properties": {
                    "query": {"type": "string"},
                    "include_pattern": {"type": "string"},
                    "exclude_pattern": {"type": "string"},
                    "case_sensitive": {"type": "boolean"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "ripgrep_raw_search".into(),
            "Run a raw ripgrep search.".into(),
            json!({
                "type": "object", "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"},
                    "glob": {"type": "string"},
                    "case_insensitive": {"type": "boolean"},
                    "output_mode": {"type": "string"},
                    "head_limit": {"type": "integer"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "codebase_search".into(),
            "Search the codebase semantically.".into(),
            json!({
                "type": "object", "properties": {
                    "query": {"type": "string"},
                    "target_directories": {"type": "array", "items": {"type": "string"}},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "read_lints".into(),
            "Read diagnostics or lints for files.".into(),
            json!({
                "type": "object", "properties": {
                    "paths": {"type": "array", "items": {"type": "string"}},
                    "file_paths": {"type": "array", "items": {"type": "string"}}
                },
                "additionalProperties": true
            }),
        ),
        (
            "todo_write".into(),
            "Create or update a task checklist.".into(),
            json!({
                "type": "object", "properties": {
                    "todos": {"type": "array"},
                    "merge": {"type": "boolean"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "web_search".into(),
            "Search the web.".into(),
            json!({
                "type": "object", "properties": {
                    "search_term": {"type": "string"},
                    "searchTerm": {"type": "string"},
                    "query": {"type": "string"},
                    "explanation": {"type": "string"}
                },
                "additionalProperties": true
            }),
        ),
        (
            "web_fetch".into(),
            "Fetch a web page.".into(),
            json!({
                "type": "object", "properties": {
                    "url": {"type": "string"},
                    "urls": {"type": "array", "items": {"type": "string"}}
                },
                "additionalProperties": true
            }),
        ),
    ]
}
