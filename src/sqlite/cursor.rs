use std::path::Path;

use rusqlite::Connection;
use serde_json::{Map, Value};

use crate::error::{Result, TraceForgeError};
use crate::parser::RawEvent;

pub fn read_cursor_state_vscdb(path: &Path) -> Result<Vec<Vec<RawEvent>>> {
    let conn = Connection::open(path).map_err(|e| TraceForgeError::Io {
        path: path.display().to_string(),
        source: std::io::Error::other(format!("rusqlite: {e}")),
    })?;

    let mut stmt = conn
        .prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'")
        .map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(format!("rusqlite: {e}")),
        })?;

    let mut sessions: Vec<Vec<RawEvent>> = Vec::new();

    let rows = stmt
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: Option<String> = row.get(1)?;
            Ok((key, value))
        })
        .map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(format!("rusqlite: {e}")),
        })?;

    for row_result in rows {
        let (_key, value_opt) = match row_result {
            Ok(r) => r,
            Err(e) => {
                return Err(TraceForgeError::Io {
                    path: path.display().to_string(),
                    source: std::io::Error::other(format!("rusqlite: {e}")),
                });
            }
        };

        let value_blob = match value_opt {
            Some(v) => v,
            None => continue,
        };

        let composer: Value = match serde_json::from_str(&value_blob) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let composer_obj = match composer.as_object() {
            Some(o) => o,
            None => continue,
        };

        let composer_id = composer_obj
            .get("composerId")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let created_at = composer_obj
            .get("createdAt")
            .and_then(Value::as_number)
            .and_then(|n| n.as_i64());

        let conversation = match composer_obj.get("conversation").and_then(Value::as_array) {
            Some(arr) if !arr.is_empty() => arr,
            _ => continue,
        };

        let mut events: Vec<RawEvent> = Vec::new();

        let mut meta_map = Map::new();
        meta_map.insert("session_id".into(), Value::String(composer_id.to_string()));
        meta_map.insert("source".into(), Value::String("cursor".to_string()));
        meta_map.insert(
            "cursor_source_db".into(),
            Value::String(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
            ),
        );
        meta_map.insert(
            "cursor_key".into(),
            Value::String(
                path.to_string_lossy()
                    .rsplit('/')
                    .next()
                    .unwrap_or("state.vscdb")
                    .to_string(),
            ),
        );
        meta_map.insert(
            "cursor_storage_kind".into(),
            Value::String("vscdb".to_string()),
        );

        events.push(RawEvent {
            r#type: Some("cursor_session_meta".into()),
            raw: meta_map,
            message: composer_obj.get("model").map(|m| {
                let mut m2 = Map::new();
                m2.insert("model".into(), m.clone());
                Value::Object(m2)
            }),
            attachment: None,
            tool_use_result: None,
        });

        let tools_map = Map::new();
        events.push(RawEvent {
            r#type: Some("cursor_available_tools".into()),
            raw: tools_map,
            message: None,
            attachment: None,
            tool_use_result: None,
        });

        for msg in conversation {
            let msg_obj = match msg.as_object() {
                Some(o) => o,
                None => continue,
            };

            let msg_type = msg_obj.get("type").and_then(Value::as_i64).unwrap_or(0);
            let role = match msg_type {
                1 => "user",
                2 => "assistant",
                _ => continue,
            };

            let text = msg_obj.get("text").and_then(Value::as_str).unwrap_or("");
            if text.is_empty() && role != "assistant" {
                continue;
            }

            let mut msg_map = Map::new();
            let make_text_block = |text: &str| {
                let mut m = Map::new();
                m.insert("type".into(), Value::String("text".into()));
                m.insert("text".into(), Value::String(text.to_string()));
                Value::Object(m)
            };
            let content_blocks: Vec<Value> = if !text.is_empty() {
                vec![make_text_block(text)]
            } else {
                vec![]
            };

            if content_blocks.is_empty() {
                msg_map.insert("content".into(), Value::Array(vec![]));
            } else {
                msg_map.insert("content".into(), Value::Array(content_blocks));
            }

            let mut raw = Map::new();
            raw.insert("role".into(), Value::String(role.to_string()));
            if let Some(ts) = created_at {
                raw.insert("timestamp".into(), Value::String(ts.to_string()));
            }

            events.push(RawEvent {
                r#type: None,
                raw,
                message: Some(Value::Object(msg_map)),
                attachment: None,
                tool_use_result: None,
            });
        }

        if events.len() > 2 {
            sessions.push(events);
        }
    }

    Ok(sessions)
}
