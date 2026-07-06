use indexmap::IndexMap;
use std::path::Path;

use rusqlite::Connection;
use serde_json::{Map, Value};

use crate::error::{Result, TraceForgeError};
use crate::parser::RawEvent;

pub fn read_hermes_state_db(path: &Path) -> Result<Vec<Vec<RawEvent>>> {
    let conn = Connection::open(path).map_err(|e| TraceForgeError::Io {
        path: path.display().to_string(),
        source: std::io::Error::other(format!("rusqlite: {e}")),
    })?;

    let has_sessions: bool = conn
        .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='sessions'")
        .and_then(|mut s| s.query_row([], |r| r.get::<_, i64>(0)))
        .map(|c| c > 0)
        .unwrap_or(false);
    let has_messages: bool = conn
        .prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='messages'")
        .and_then(|mut s| s.query_row([], |r| r.get::<_, i64>(0)))
        .map(|c| c > 0)
        .unwrap_or(false);
    if !has_sessions || !has_messages {
        return Ok(vec![]);
    }

    let mut stmt = conn
        .prepare(
            "SELECT id, source, model, system_prompt, started_at, ended_at,
                    end_reason, title, cwd, estimated_cost_usd,
                    input_tokens, output_tokens, reasoning_tokens,
                    cache_read_tokens, cache_write_tokens
             FROM sessions
             ORDER BY started_at",
        )
        .map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(format!("rusqlite: {e}")),
        })?;

    let mut msg_stmt = conn
        .prepare(
            "SELECT role, content, tool_call_id, tool_calls, tool_name,
                    timestamp, finish_reason, reasoning, reasoning_content
             FROM messages
             WHERE session_id = ?1 AND active = 1
             ORDER BY timestamp",
        )
        .map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(format!("rusqlite: {e}")),
        })?;

    let mut sessions: Vec<Vec<RawEvent>> = Vec::new();

    let rows = stmt
        .query_map([], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                source: row.get(1)?,
                model: row.get(2)?,
                system_prompt: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                end_reason: row.get(6)?,
                title: row.get(7)?,
                cwd: row.get(8)?,
                estimated_cost_usd: row.get(9)?,
                input_tokens: row.get(10)?,
                output_tokens: row.get(11)?,
                reasoning_tokens: row.get(12)?,
                cache_read_tokens: row.get(13)?,
                cache_write_tokens: row.get(14)?,
            })
        })
        .map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: std::io::Error::other(format!("rusqlite: {e}")),
        })?;

    for row_result in rows {
        let row = match row_result {
            Ok(r) => r,
            Err(e) => {
                return Err(TraceForgeError::Io {
                    path: path.display().to_string(),
                    source: std::io::Error::other(format!("rusqlite: {e}")),
                });
            }
        };

        let mut messages: Vec<Value> = Vec::new();

        let msg_rows = msg_stmt
            .query_map([&row.id], |msg_row| {
                Ok(MessageRow {
                    role: msg_row.get(0)?,
                    content: msg_row.get(1)?,
                    tool_call_id: msg_row.get(2)?,
                    tool_calls: msg_row.get(3)?,
                    tool_name: msg_row.get(4)?,
                    timestamp: msg_row.get(5)?,
                    finish_reason: msg_row.get(6)?,
                    reasoning: msg_row.get(7)?,
                    reasoning_content: msg_row.get(8)?,
                })
            })
            .map_err(|e| TraceForgeError::Io {
                path: path.display().to_string(),
                source: std::io::Error::other(format!("rusqlite: {e}")),
            })?;

        for msg_result in msg_rows {
            let msg_row = match msg_result {
                Ok(m) => m,
                Err(e) => {
                    return Err(TraceForgeError::Io {
                        path: path.display().to_string(),
                        source: std::io::Error::other(format!("rusqlite: {e}")),
                    });
                }
            };

            let mut msg_map = Map::new();
            msg_map.insert("role".into(), Value::String(msg_row.role));

            if let Some(ref c) = msg_row.content {
                msg_map.insert("content".into(), Value::String(c.clone()));
            }

            if let Some(ref tcid) = msg_row.tool_call_id {
                msg_map.insert("tool_call_id".into(), Value::String(tcid.clone()));
            }

            if let Some(ref tn) = msg_row.tool_name {
                msg_map.insert("tool_name".into(), Value::String(tn.clone()));
            }

            if let Some(ref tcs) = msg_row.tool_calls {
                if let Ok(val) = serde_json::from_str::<Value>(tcs) {
                    msg_map.insert("tool_calls".into(), val);
                }
            }

            if let Some(ref ts) = msg_row.timestamp {
                msg_map.insert("timestamp".into(), Value::String(ts.to_string()));
            }

            if let Some(ref r) = msg_row.reasoning {
                msg_map.insert("reasoning".into(), Value::String(r.clone()));
            }
            if let Some(ref rc) = msg_row.reasoning_content {
                msg_map.insert("reasoning_content".into(), Value::String(rc.clone()));
            }

            if let Some(ref fr) = msg_row.finish_reason {
                msg_map.insert("finish_reason".into(), Value::String(fr.clone()));
            }

            messages.push(Value::Object(msg_map));
        }

        let mut raw = Map::new();
        raw.insert("messages".into(), Value::Array(messages));
        raw.insert("id".into(), Value::String(row.id.clone()));
        if let Some(s) = row.source {
            raw.insert("source".into(), Value::String(s));
        }
        if let Some(m) = row.model {
            raw.insert("model".into(), Value::String(m));
        }
        if let Some(sp) = row.system_prompt {
            raw.insert("system_prompt".into(), Value::String(sp));
        }
        if let Some(ts) = row.started_at {
            if let Some(n) = serde_json::Number::from_f64(ts) {
                raw.insert("started_at".into(), Value::Number(n));
            }
        }
        if let Some(ts) = row.ended_at {
            if let Some(n) = serde_json::Number::from_f64(ts) {
                raw.insert("ended_at".into(), Value::Number(n));
            }
        }
        if let Some(er) = row.end_reason {
            raw.insert("end_reason".into(), Value::String(er));
        }
        if let Some(t) = row.title {
            raw.insert("title".into(), Value::String(t));
        }
        if let Some(c) = row.cwd {
            raw.insert("cwd".into(), Value::String(c));
        }
        if let Some(cost) = row.estimated_cost_usd {
            if let Some(n) = serde_json::Number::from_f64(cost) {
                raw.insert("estimated_cost_usd".into(), Value::Number(n));
            }
        }

        let mut extra = IndexMap::new();
        if let Some(t) = row.input_tokens {
            extra.insert("input_tokens".into(), Value::Number(t.into()));
        }
        if let Some(t) = row.output_tokens {
            extra.insert("output_tokens".into(), Value::Number(t.into()));
        }
        if let Some(t) = row.reasoning_tokens {
            extra.insert("reasoning_tokens".into(), Value::Number(t.into()));
        }
        if let Some(t) = row.cache_read_tokens {
            extra.insert("cache_read_tokens".into(), Value::Number(t.into()));
        }
        if let Some(t) = row.cache_write_tokens {
            extra.insert("cache_write_tokens".into(), Value::Number(t.into()));
        }
        if !extra.is_empty() {
            let map: Map<String, Value> = extra.into_iter().collect();
            raw.insert("usage".into(), Value::Object(map));
        }

        sessions.push(vec![RawEvent {
            r#type: None,
            raw,
            message: None,
            attachment: None,
            tool_use_result: None,
        }]);
    }

    Ok(sessions)
}

struct SessionRow {
    id: String,
    source: Option<String>,
    model: Option<String>,
    system_prompt: Option<String>,
    started_at: Option<f64>,
    ended_at: Option<f64>,
    end_reason: Option<String>,
    title: Option<String>,
    cwd: Option<String>,
    estimated_cost_usd: Option<f64>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    reasoning_tokens: Option<i64>,
    cache_read_tokens: Option<i64>,
    cache_write_tokens: Option<i64>,
}

struct MessageRow {
    role: String,
    content: Option<String>,
    tool_call_id: Option<String>,
    tool_calls: Option<String>,
    tool_name: Option<String>,
    timestamp: Option<f64>,
    finish_reason: Option<String>,
    reasoning: Option<String>,
    reasoning_content: Option<String>,
}
