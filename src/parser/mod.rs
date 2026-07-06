use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;
use serde_json::{Map, Value};

use crate::error::{Result, TraceForgeError};

#[derive(Debug, Clone, Deserialize)]
pub struct RawEvent {
    #[serde(default)]
    pub r#type: Option<String>,

    #[serde(flatten)]
    pub raw: serde_json::Map<String, Value>,

    #[serde(default)]
    pub message: Option<Value>,

    #[serde(default)]
    pub attachment: Option<Value>,

    #[serde(default)]
    pub tool_use_result: Option<Value>,
}

impl RawEvent {
    pub fn field(&self, key: &str) -> Option<&Value> {
        self.raw.get(key)
    }

    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.raw.get(key).and_then(Value::as_str)
    }

    pub fn field_bool(&self, key: &str) -> Option<bool> {
        self.raw.get(key).and_then(Value::as_bool)
    }

    pub fn field_f64(&self, key: &str) -> Option<f64> {
        self.raw.get(key).and_then(Value::as_f64)
    }

    pub fn full_map(&self) -> Map<String, Value> {
        let mut map = self.raw.clone();
        if let Some(ref t) = self.r#type {
            map.insert("type".into(), Value::String(t.clone()));
        }
        if let Some(ref v) = self.message {
            map.insert("message".into(), v.clone());
        }
        if let Some(ref v) = self.attachment {
            map.insert("attachment".into(), v.clone());
        }
        if let Some(ref v) = self.tool_use_result {
            map.insert("tool_use_result".into(), v.clone());
        }
        map
    }
}

pub struct LineReader {
    path: String,
    lines: std::io::Lines<BufReader<File>>,
    line_no: usize,
}

impl LineReader {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        Ok(Self {
            path: path.display().to_string(),
            lines: BufReader::new(file).lines(),
            line_no: 0,
        })
    }

    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn current_line_number(&self) -> usize {
        self.line_no
    }
}

impl Iterator for LineReader {
    type Item = Result<RawEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let line = match self.lines.next()? {
                Ok(l) => l,
                Err(e) => {
                    self.line_no += 1;
                    return Some(Err(TraceForgeError::Io {
                        path: self.path.clone(),
                        source: e,
                    }));
                }
            };
            self.line_no += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            return Some(
                serde_json::from_str(trimmed).map_err(|e| TraceForgeError::ParseLine {
                    path: self.path.clone(),
                    line: self.line_no,
                    source: e,
                }),
            );
        }
    }
}

pub fn read_all_events(path: &Path) -> Result<Vec<RawEvent>> {
    let reader = LineReader::open(path)?;
    let mut out = Vec::new();
    let mut had_error = false;

    for event in reader {
        match event {
            Ok(ev) => out.push(ev),
            Err(TraceForgeError::ParseLine { .. }) if out.is_empty() => {
                had_error = true;
                break;
            }
            Err(e) => return Err(e),
        }
    }

    if had_error && out.is_empty() {
        let content = std::fs::read_to_string(path).map_err(|e| TraceForgeError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        let trimmed = content.trim();
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            let val: serde_json::Value = serde_json::from_str(trimmed)?;
            return match val {
                serde_json::Value::Object(map) => Ok(vec![RawEvent {
                    r#type: None,
                    raw: map,
                    message: None,
                    attachment: None,
                    tool_use_result: None,
                }]),
                serde_json::Value::Array(arr) => {
                    let events = arr
                        .into_iter()
                        .filter_map(|item| match item {
                            serde_json::Value::Object(map) => Some(RawEvent {
                                r#type: None,
                                raw: map,
                                message: None,
                                attachment: None,
                                tool_use_result: None,
                            }),
                            _ => None,
                        })
                        .collect();
                    Ok(events)
                }
                _ => Err(TraceForgeError::ParseLine {
                    path: path.display().to_string(),
                    line: 1,
                    source: serde_json::from_str::<serde_json::Value>("")
                        .expect_err("this should always fail"),
                }),
            };
        }
    }

    Ok(out)
}

pub fn peek_events(path: &Path, n: usize) -> Result<Vec<RawEvent>> {
    let reader = LineReader::open(path)?;
    reader.take(n).collect::<Result<Vec<_>>>()
}

#[cfg(test)]
mod tests;
