use serde_json::Value;

pub fn attachment_context(att: &serde_json::Map<String, Value>, att_type: &str) -> Option<String> {
    match att_type {
        "deferred_tools_delta" => {
            let names: Vec<&str> = att
                .get("addedNames")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if names.is_empty() {
                return None;
            }
            Some(format!(
                "Claude Code deferred tools available through ToolSearch:\n{}",
                names
                    .iter()
                    .map(|n| format!("- {n}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
        "mcp_instructions_delta" => {
            let names: Vec<&str> = att
                .get("addedNames")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let blocks: Vec<&str> = att
                .get("addedBlocks")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            let mut parts: Vec<String> = vec![];
            if !names.is_empty() {
                parts.push(format!(
                    "Claude MCP instructions added for: {}",
                    names.join(", ")
                ));
            }
            parts.extend(blocks.iter().map(|&s| s.to_string()));
            let out = parts.join("\n\n");
            if out.is_empty() {
                None
            } else {
                Some(out)
            }
        }
        "skill_listing" => att
            .get("content")
            .and_then(Value::as_str)
            .map(|c| format!("Claude Code available skills:\n{}", c.trim())),
        "command_permissions" => {
            let allowed: Vec<&str> = att
                .get("allowedTools")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if allowed.is_empty() {
                Some("Claude Code command permissions allow no additional tools.".into())
            } else {
                Some(format!(
                    "Claude Code command permissions allow: {}",
                    allowed.join(", ")
                ))
            }
        }
        "date_change" => att
            .get("newDate")
            .and_then(Value::as_str)
            .map(|d| format!("Current date: {}", d.trim())),
        "hook_additional_context" => {
            let items: Vec<&str> = att
                .get("content")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if items.is_empty() {
                return None;
            }
            let name = att.get("hookName").and_then(Value::as_str);
            let header = match name {
                Some(n) if !n.is_empty() => format!("Claude Code hook context ({})", n.trim()),
                _ => "Claude Code hook context".into(),
            };
            Some(format!("{}:\n{}", header, items.join("\n")))
        }
        "edited_text_file" => {
            let snippet = att.get("snippet").and_then(Value::as_str)?;
            if snippet.trim().is_empty() {
                return None;
            }
            let filename = att.get("filename").and_then(Value::as_str);
            match filename {
                Some(f) if !f.is_empty() => Some(format!(
                    "Claude Code edited file context for {}:\n{}",
                    f.trim(),
                    snippet.trim()
                )),
                _ => Some(format!(
                    "Claude Code edited file context:\n{}",
                    snippet.trim()
                )),
            }
        }
        "task_reminder" => {
            let items: Vec<&str> = att
                .get("content")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if items.is_empty() {
                None
            } else {
                Some(format!("Claude Code task reminder:\n{}", items.join("\n")))
            }
        }
        "plan_mode_exit" => {
            let plan_path = att.get("planFilePath").and_then(Value::as_str);
            match plan_path {
                Some(p) if !p.is_empty() => Some(format!(
                    "Claude Code exited plan mode. Plan file: {}",
                    p.trim()
                )),
                _ => Some("Claude Code exited plan mode.".into()),
            }
        }
        _ => None,
    }
}
