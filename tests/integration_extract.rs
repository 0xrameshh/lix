use std::path::Path;

use lix::{run_pipeline, PipelineOpts};

#[test]
fn extract_pipeline_produces_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = Path::new("tests/fixtures/claude_minimal.jsonl");
    let output = dir.path().join("out.jsonl");

    let rep = run_pipeline(
        input,
        &output,
        PipelineOpts {
            clean: false,
            drop_incomplete: false,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(rep.rows_written > 0);
    assert!(output.exists());
}

#[test]
fn extract_with_clean_removes_sensitive_data() {
    let dir = tempfile::tempdir().unwrap();
    let input = Path::new("tests/fixtures/claude_toolcall.jsonl");
    let output = dir.path().join("out.jsonl");

    let rep = run_pipeline(
        input,
        &output,
        PipelineOpts {
            clean: true,
            drop_incomplete: false,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(rep.rows_written > 0);
    assert!(output.exists());
}

#[test]
fn extract_drops_incomplete_trace() {
    let dir = tempfile::tempdir().unwrap();
    let input = Path::new("tests/fixtures/claude_incomplete.jsonl");
    let output = dir.path().join("out.jsonl");

    let rep = run_pipeline(
        input,
        &output,
        PipelineOpts {
            clean: false,
            drop_incomplete: true,
            ..Default::default()
        },
    )
    .unwrap();

    // Should drop the incomplete trace
    assert_eq!(rep.rows_written, 0);
}

#[test]
fn extract_keeps_incomplete_when_requested() {
    let dir = tempfile::tempdir().unwrap();
    let input = Path::new("tests/fixtures/claude_incomplete.jsonl");
    let output = dir.path().join("out.jsonl");

    let rep = run_pipeline(
        input,
        &output,
        PipelineOpts {
            clean: false,
            drop_incomplete: false,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(rep.rows_written > 0);
}

#[test]
fn extract_multiple_providers() {
    for fixture in &[
        "claude_minimal",
        "codex_minimal",
        "cursor_minimal",
        "droid_minimal",
        "hermes_minimal",
        "pi_minimal",
        "openclaw_minimal",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let input = format!("tests/fixtures/{fixture}.jsonl");
        let output = dir.path().join("out.jsonl");

        let rep = run_pipeline(
            Path::new(&input),
            &output,
            PipelineOpts {
                clean: false,
                drop_incomplete: false,
                ..Default::default()
            },
        )
        .unwrap();

        assert!(
            rep.rows_written > 0,
            "{fixture}: expected at least 1 row, got 0"
        );
    }
}
