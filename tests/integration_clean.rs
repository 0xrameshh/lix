use std::path::Path;

use lix::{Cleaner, LineReader};

#[test]
fn cleaner_redacts_api_keys() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    std::fs::write(
        &input,
        r#"{"type":"user","content":"my key is sk-proj-abcdefghijklmnopqrstuvwxyz1234567890"}"#,
    )
    .unwrap();

    let reader = LineReader::open(&input).unwrap();
    let cleaner = Cleaner::new();
    let mut cleaned = String::new();

    for line in reader {
        let ev = line.unwrap();
        let json_str = serde_json::to_string(&ev.raw).unwrap();
        let mut text = json_str;
        cleaner.clean_text(&mut text, &mut Default::default());
        cleaned.push_str(&text);
    }

    assert!(cleaned.contains("<redacted:api_key>"));
}

#[test]
fn cleaner_redacts_home_paths() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    std::fs::write(&input, r#"{"cwd":"/Users/testuser/project"}"#).unwrap();

    let reader = LineReader::open(&input).unwrap();
    let cleaner = Cleaner::new();
    let mut cleaned = String::new();

    for line in reader {
        let ev = line.unwrap();
        let json_str = serde_json::to_string(&ev.raw).unwrap();
        let mut text = json_str;
        cleaner.clean_text(&mut text, &mut Default::default());
        cleaned.push_str(&text);
    }

    assert!(!cleaned.contains("testuser"));
    assert!(cleaned.contains("user_"));
}

#[test]
fn cleaner_handles_empty_input() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("empty.jsonl");
    std::fs::write(&input, "").unwrap();

    let reader = LineReader::open(&input).unwrap();
    let cleaner = Cleaner::new();
    let mut count = 0;

    for line in reader {
        let ev = line.unwrap();
        let json_str = serde_json::to_string(&ev.raw).unwrap();
        let mut text = json_str;
        cleaner.clean_text(&mut text, &mut Default::default());
        count += 1;
    }

    assert_eq!(count, 0);
}

#[test]
fn cleaner_multiple_patterns_in_one_line() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.jsonl");
    std::fs::write(
        &input,
        r#"{"msg":"sk-proj-abcdefghijklmnopqrstuvwxyz1234567890 /Users/bob bob@example.com"}"#,
    )
    .unwrap();

    let reader = LineReader::open(&input).unwrap();
    let cleaner = Cleaner::new();
    let mut report = lix::CleanReport::default();

    for line in reader {
        let ev = line.unwrap();
        let json_str = serde_json::to_string(&ev.raw).unwrap();
        let mut text = json_str;
        cleaner.clean_text(&mut text, &mut report);
    }

    assert!(report.api_keys > 0, "api_keys should be > 0");
    assert!(report.home_paths > 0, "home_paths should be > 0");
    assert!(report.emails > 0, "emails should be > 0");
}
