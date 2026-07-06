mod common;

use lix::{run_pipeline, PipelineOpts};
use std::fs;
use std::path::Path;

fn run_extract(input: &Path) -> (Vec<u8>, lix::ExtractionReport) {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.jsonl");
    let report = run_pipeline(input, &output, PipelineOpts::default()).unwrap();
    let data = if output.exists() {
        fs::read(&output).unwrap()
    } else {
        vec![]
    };
    (data, report)
}

fn read_first_row(data: &[u8]) -> serde_json::Value {
    serde_json::from_slice(data).unwrap()
}

// ── Droid ──────────────────────────────────────────────────────────

#[test]
fn droid_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("droid_minimal.jsonl"));
    assert_eq!(report.rows_written, 1, "droid should produce 1 row");
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "droid");
    let msgs = row["messages"].as_array().unwrap();
    assert!(
        msgs.len() > 1,
        "droid should have 2+ messages, got {}",
        msgs.len()
    );
}

#[test]
fn droid_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_droid_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "droid");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 60, "droid full session should have 60 messages");
    let tools: Vec<&str> = msgs.iter().filter_map(|m| m["name"].as_str()).collect();
    for expected in &[
        "AskUser",
        "Create",
        "Execute",
        "Glob",
        "Read",
        "Skill",
        "TodoWrite",
        "WebSearch",
    ] {
        assert!(
            tools.contains(expected),
            "droid should have tool {expected}"
        );
    }
}

// ── Cursor ─────────────────────────────────────────────────────────

#[test]
fn cursor_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("cursor_minimal.jsonl"));
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "cursor");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 4, "cursor minimal should have 4 messages");
}

#[test]
fn cursor_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_cursor_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "cursor");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 4, "cursor full session should have 4 messages");
    let tools: Vec<&str> = msgs.iter().filter_map(|m| m["name"].as_str()).collect();
    assert!(tools.contains(&"Shell"));
}

// ── Pi ─────────────────────────────────────────────────────────────

#[test]
fn pi_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("pi_minimal.jsonl"));
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "pi");
    let msgs = row["messages"].as_array().unwrap();
    assert!(!msgs.is_empty());
}

#[test]
fn pi_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_pi_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "pi");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 319, "pi full session should have 319 messages");
    let tools: Vec<&str> = msgs.iter().filter_map(|m| m["name"].as_str()).collect();
    for expected in &["bash", "edit", "read", "write"] {
        assert!(tools.contains(expected), "pi should have tool {expected}");
    }
}

// ── Openclaw ───────────────────────────────────────────────────────

#[test]
fn openclaw_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("openclaw_minimal.jsonl"));
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "openclaw");
    let msgs = row["messages"].as_array().unwrap();
    assert!(!msgs.is_empty());
}

#[test]
fn openclaw_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_openclaw_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "openclaw");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(
        msgs.len(),
        3,
        "openclaw full session should have 3 messages"
    );
}

// ── Hermes ─────────────────────────────────────────────────────────

#[test]
fn hermes_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("hermes_minimal.jsonl"));
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "hermes");
    let msgs = row["messages"].as_array().unwrap();
    assert!(!msgs.is_empty());
}

#[test]
fn hermes_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_hermes_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "hermes");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(
        msgs.len(),
        11,
        "hermes full session should have 11 messages"
    );
    let tools: Vec<&str> = msgs.iter().filter_map(|m| m["name"].as_str()).collect();
    assert!(
        tools.contains(&"browser_navigate"),
        "hermes should have tool browser_navigate"
    );
}

// ── Codex ──────────────────────────────────────────────────────────

#[test]
fn codex_minimal_extracts_correctly() {
    let (data, report) = run_extract(&common::fixture("codex_minimal.jsonl"));
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "codex");
    let msgs = row["messages"].as_array().unwrap();
    assert!(!msgs.is_empty());
}

#[test]
fn codex_full_session_produces_output() {
    let path = Path::new("/tmp/teich/examples/example_codex_session.jsonl");
    if !path.exists() {
        return;
    }
    let (data, report) = run_extract(path);
    assert_eq!(report.rows_written, 1);
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "codex");
    let msgs = row["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 35, "codex full session should have 35 messages");
    let tools: Vec<&str> = msgs.iter().filter_map(|m| m["name"].as_str()).collect();
    for expected in &["apply_patch", "exec_command", "write_stdin"] {
        assert!(
            tools.contains(expected),
            "codex should have tool {expected}"
        );
    }
}

// ── Provider detection ─────────────────────────────────────────────

#[test]
fn codex_detection_over_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("codex_like.jsonl");
    let mut content = String::new();
    content.push_str(r#"{"type":"session_meta","payload":{"id":"s1","cwd":"/home"}}"#);
    content.push('\n');
    content.push_str(r#"{"type":"response_item","payload":{"type":"message","role":"user","content":[{"type":"text","text":"hello"}]}}"#);
    content.push('\n');
    content.push_str(r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"text","text":"hi"}]}}"#);
    content.push('\n');
    fs::write(&path, &content).unwrap();
    let (data, report) = run_extract(&path);
    assert_eq!(
        report.rows_written, 1,
        "codex-like events should be detected as codex"
    );
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "codex");
}

#[test]
fn hermes_detection_requires_export_session_fields() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hermes_single.jsonl");
    let obj = serde_json::json!({
        "id": "test-hermes",
        "model": "gpt-4o",
        "source": "cli",
        "messages": [
            {"role": "user", "content": "hello"},
            {"role": "assistant", "content": "hi"}
        ]
    });
    fs::write(&path, obj.to_string() + "\n").unwrap();
    let (data, report) = run_extract(&path);
    assert_eq!(
        report.rows_written, 1,
        "hermes export session should be detected"
    );
    let row = read_first_row(&data);
    assert_eq!(row["metadata"]["trace_type"], "hermes");
}
