mod common;

use std::fs;
use std::path::Path;

use lix::{run_pipeline, PipelineOpts};

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

#[test]
fn extracts_minimal_trace() {
    let (data, report) = run_extract(&common::fixture("claude_minimal.jsonl"));
    assert_eq!(report.rows_written, 1, "should produce 1 row");
    assert!(report.files_ok >= 1, "should mark file as ok");
    assert!(!data.is_empty(), "output should not be empty");

    let line: serde_json::Value = serde_json::from_slice(&data).unwrap();
    assert_eq!(line["metadata"]["trace_type"], "claude-code");
    assert_eq!(line["metadata"]["source_file"], "claude_minimal.jsonl");
    assert!(line["messages"].as_array().unwrap().len() >= 2);
}

#[test]
fn extracts_toolcall_trace() {
    let (data, report) = run_extract(&common::fixture("claude_toolcall.jsonl"));
    assert_eq!(report.rows_written, 1);
    assert!(report.files_ok >= 1);

    let line: serde_json::Value = serde_json::from_slice(&data).unwrap();
    let messages = line["messages"].as_array().unwrap();
    let tool_names: Vec<&str> = messages.iter().filter_map(|m| m["name"].as_str()).collect();
    assert!(
        tool_names.contains(&"unknown_tool"),
        "tool response name should be 'unknown_tool', got: {tool_names:?}"
    );
}

#[test]
fn drops_synthetic_only_trace() {
    let (data, report) = run_extract(&common::fixture("claude_synthetic_only.jsonl"));
    assert_eq!(
        report.rows_written, 0,
        "synthetic-only trace should produce no output"
    );
    assert!(report.files_dropped >= 1);
    assert!(data.is_empty());
}

#[test]
fn handles_mixed_synthetic_trace() {
    let (data, report) = run_extract(&common::fixture("claude_synthetic_limit.jsonl"));
    assert_eq!(
        report.rows_written, 1,
        "trace with user + synthetic assistant should be valid"
    );
    assert!(!data.is_empty());
}

#[test]
fn drops_incomplete_trace() {
    let (data, report) = run_extract(&common::fixture("claude_incomplete.jsonl"));
    assert_eq!(report.rows_written, 0);
    assert!(report.files_dropped >= 1);
    assert!(data.is_empty());
}

#[test]
fn clean_removes_sensitive_data() {
    let opts = PipelineOpts {
        clean: true,
        ..PipelineOpts::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.jsonl");
    let report = run_pipeline(&common::fixture("claude_minimal.jsonl"), &output, opts).unwrap();
    assert_eq!(report.rows_written, 1);

    let data = fs::read(&output).unwrap();
    let line: serde_json::Value = serde_json::from_slice(&data).unwrap();
    let text = serde_json::to_string(&line).unwrap();
    assert!(
        !text.contains("testuser"),
        "home path username should be redacted"
    );
    assert!(
        report.total_replacements > 0,
        "cleaning should have performed replacements"
    );
}

#[test]
fn no_clean_preserves_data() {
    let opts = PipelineOpts {
        clean: false,
        ..PipelineOpts::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.jsonl");
    let report = run_pipeline(&common::fixture("claude_minimal.jsonl"), &output, opts).unwrap();
    assert_eq!(report.rows_written, 1);

    let data = fs::read(&output).unwrap();
    let text = String::from_utf8_lossy(&data);
    assert!(
        text.contains("testuser"),
        "without cleaning, username should remain"
    );
    assert_eq!(report.total_replacements, 0);
}

#[test]
fn model_filter_drops_non_matching() {
    let opts = PipelineOpts {
        model_filter: Some("nonexistent".into()),
        ..PipelineOpts::default()
    };
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.jsonl");
    let report = run_pipeline(&common::fixture("claude_minimal.jsonl"), &output, opts).unwrap();
    assert_eq!(report.rows_written, 0);
}

#[test]
fn report_metrics_are_correct() {
    let (data, report) = run_extract(&common::fixture("claude_minimal.jsonl"));
    assert!(report.files_total >= 1);
    assert!(report.elapsed_secs > 0.0);

    let line: serde_json::Value = serde_json::from_slice(&data).unwrap();
    let metadata = &line["metadata"];
    assert!(
        metadata["session_id"].as_str().unwrap_or("") == "test-session-1"
            || metadata["session_id"].as_str().unwrap_or("") == "claude_minimal"
    );
}

#[test]
fn handles_empty_input_file() {
    let dir = tempfile::tempdir().unwrap();
    let empty_file = dir.path().join("empty.jsonl");
    fs::write(&empty_file, "").unwrap();
    let output = dir.path().join("out.jsonl");
    let report = run_pipeline(&empty_file, &output, PipelineOpts::default()).unwrap();
    assert_eq!(report.rows_written, 0);
    assert!(report.files_dropped >= 1);
    assert!(std::fs::metadata(&output).is_err() || fs::read(&output).unwrap().is_empty());
}
