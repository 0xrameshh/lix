use super::*;
use std::io::Write;

#[test]
fn parse_valid_jsonl() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.jsonl");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(
        f,
        r#"{{"type":"user","message":{{"role":"user","content":"hi"}}}}"#
    )
    .unwrap();
    writeln!(
        f,
        r#"{{"type":"assistant","message":{{"role":"assistant","content":"hello"}}}}"#
    )
    .unwrap();

    let events = read_all_events(&path).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].r#type.as_deref(), Some("user"));
    assert_eq!(events[1].r#type.as_deref(), Some("assistant"));
}

#[test]
fn parse_skips_empty_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.jsonl");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"type":"user"}}"#).unwrap();
    writeln!(f).unwrap();
    writeln!(f, "  ").unwrap();
    writeln!(f, r#"{{"type":"assistant"}}"#).unwrap();

    let events = read_all_events(&path).unwrap();
    assert_eq!(events.len(), 2);
}

#[test]
fn parse_handles_malformed_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.jsonl");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"type":"user"}}"#).unwrap();
    writeln!(f, "not-json").unwrap();

    let result = read_all_events(&path);
    assert!(result.is_err());
    if let Err(TraceForgeError::ParseLine { line, .. }) = result {
        assert_eq!(line, 2);
    } else {
        panic!("expected ParseLine error");
    }
}

#[test]
fn parse_nonexistent_file() {
    let result = read_all_events(Path::new("/nonexistent/path.jsonl"));
    assert!(result.is_err());
    assert!(matches!(result, Err(TraceForgeError::Io { .. })));
}

#[test]
fn raw_event_deserializes_type() {
    let json = r#"{"type":"assistant"}"#;
    let ev: RawEvent = serde_json::from_str(json).unwrap();
    assert_eq!(ev.r#type.as_deref(), Some("assistant"));
    assert!(ev.message.is_none());
}

#[test]
fn raw_event_deserializes_flattened_fields() {
    let json = r#"{"type":"user","sessionId":"s1","timestamp":"t1"}"#;
    let ev: RawEvent = serde_json::from_str(json).unwrap();
    assert_eq!(ev.field_str("sessionId"), Some("s1"));
    assert_eq!(ev.field_str("timestamp"), Some("t1"));
}

#[test]
fn raw_event_deserializes_message() {
    let json = r#"{"type":"assistant","message":{"role":"assistant","content":"hello"}}"#;
    let ev: RawEvent = serde_json::from_str(json).unwrap();
    assert!(ev.message.is_some());
    assert_eq!(
        ev.message
            .as_ref()
            .and_then(|m| m.get("role"))
            .and_then(Value::as_str),
        Some("assistant")
    );
}

#[test]
fn raw_event_field_accessors() {
    let json = r#"{"type":"user","count":42,"active":true,"cost":2.5}"#;
    let ev: RawEvent = serde_json::from_str(json).unwrap();
    assert_eq!(ev.field("count").and_then(Value::as_i64), Some(42));
    assert_eq!(ev.field_bool("active"), Some(true));
    assert!((ev.field_f64("cost").unwrap() - 2.5).abs() < 0.001);
}

#[test]
fn reports_line_number_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.jsonl");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, r#"{{"type":"user"}}"#).unwrap();
    writeln!(f, r#"{{"type":"assistant"}}"#).unwrap();
    writeln!(f, "bad-json").unwrap();

    let result = read_all_events(&path);
    assert!(result.is_err());
    match result {
        Err(TraceForgeError::ParseLine { line, .. }) => assert_eq!(line, 3),
        _ => panic!("expected ParseLine error on line 3"),
    }
}

#[test]
fn line_reader_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.jsonl");
    std::fs::write(&path, "").unwrap();
    let events = read_all_events(&path).unwrap();
    assert!(events.is_empty());
}
