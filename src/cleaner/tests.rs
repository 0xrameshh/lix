use crate::models::Step;

use super::*;

#[test]
fn cleans_openai_key() {
    let mut s = "export OPENAI_API_KEY=sk-proj-abcdefghijklmnopqrstuvwxyz1234567890".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:api_key>"));
    assert_eq!(r.api_keys, 1);
}

#[test]
fn cleans_home_path() {
    let mut s = "cwd=/Users/calebfahlgren/.openclaw/workspace".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(!s.contains("calebfahlgren"));
    assert!(s.contains("user_"));
    assert_eq!(r.home_paths, 1);
}

#[test]
fn cleans_email() {
    let mut s = "contact: john.doe+dev@example.com".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("@example.com"));
    assert!(s.contains("user_"));
    assert_eq!(r.emails, 1);
}

#[test]
fn cleans_anthropic_key() {
    let mut s = "ANTHROPIC_API_KEY=sk-ant-abcdefghijklmnopqrstuvwxyz1234567890abcd".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:api_key>"));
    assert_eq!(r.api_keys, 1);
}

#[test]
fn cleans_hf_key() {
    let mut s = "token=hf_abcdefghijklmnopqrstuvwxyzabcd".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:api_key>"));
    assert_eq!(r.api_keys, 1);
}

#[test]
fn cleans_windows_path() {
    let mut s = "cwd=C:\\Users\\jdoe\\projects".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(!s.contains("jdoe"));
    assert!(s.contains("user_"));
    assert_eq!(r.home_paths, 1);
}

#[test]
fn cleans_encoded_home_path() {
    let mut s = "-home-jdoe-".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(!s.contains("jdoe"));
    assert!(s.contains("user_"));
    assert_eq!(r.home_paths, 1);
}

#[test]
fn determinism() {
    let mut a = "/Users/alice/secret".into();
    let mut b = "/Users/alice/secret".into();
    let c = Cleaner::new();
    let mut ra = CleanReport::default();
    let mut rb = CleanReport::default();
    c.clean_text(&mut a, &mut ra);
    c.clean_text(&mut b, &mut rb);
    assert_eq!(a, b);
}

#[test]
fn cleans_jwt() {
    let mut s = "token=eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dGVzdC1zaWduYXR1cmU".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:jwt>"));
    assert_eq!(r.jwt, 1);
}

#[test]
fn cleans_bearer() {
    let mut s = "Authorization: Bearer a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:bearer>"));
    assert_eq!(r.bearer, 1);
}

#[test]
fn cleans_credential_url() {
    let mut s = "postgres://admin:supersecret123@db.example.com:5432/mydb".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted_user>"));
    assert!(s.contains("<redacted_password>"));
    assert_eq!(r.credential_url, 1);
}

#[test]
fn cleans_private_key_block() {
    let mut s =
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA1\n-----END RSA PRIVATE KEY-----".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:private_key_block>"));
    assert_eq!(r.private_key_block, 1);
}

#[test]
fn cleans_env_assignment() {
    let mut s = "MY_VAR = a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:env_value>"));
    assert_eq!(r.env_assignment, 1);
}

#[test]
fn cleans_query_secret() {
    let mut s = "?api_key=sk-proj-abcdefghijklmnopqrstuvwxyz1234567890&other=val".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:query_secret>"));
    assert_eq!(r.query_secret, 1);
}

#[test]
fn cleans_generic_secret() {
    let mut s = "password = abcdefghijklmnopqrstuvwxyz1234567890".into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:secret>"));
    assert_eq!(r.generic_secret, 1);
}

#[test]
fn clean_value_string() {
    let mut v = serde_json::json!("/Users/bob/secret");
    let mut r = CleanReport::default();
    Cleaner::new().clean_value(&mut v, &mut r);
    let s = v.as_str().unwrap();
    assert!(!s.contains("bob"));
    assert!(s.contains("user_"));
    assert_eq!(r.home_paths, 1);
}

#[test]
fn clean_value_nested_object() {
    let mut v = serde_json::json!({
        "user": "admin@example.com",
        "config": {
            "path": "/Users/alice/work"
        }
    });
    let mut r = CleanReport::default();
    Cleaner::new().clean_value(&mut v, &mut r);
    assert!(v.to_string().contains("@example.com"));
    assert!(!v.to_string().contains("alice"));
    assert!(r.home_paths >= 1);
    assert!(r.emails >= 1);
}

#[test]
fn clean_value_array() {
    let mut v = serde_json::json!(["/Users/dave/project", "dave@example.com"]);
    let mut r = CleanReport::default();
    Cleaner::new().clean_value(&mut v, &mut r);
    assert!(!v.to_string().contains("dave"));
    assert!(v.to_string().contains("@example.com"));
}

#[test]
fn clean_step_user_content() {
    let mut step = Step::User {
        content: "my password is sk-proj-abcdefghijklmnopqrstuvwxyz1234567890".into(),
    };
    let mut r = CleanReport::default();
    Cleaner::new().clean_step(&mut step, &mut r);
    let Step::User { content } = &step else {
        panic!("expected User")
    };
    assert!(content.contains("<redacted:api_key>"));
    assert_eq!(r.api_keys, 1);
}

#[test]
fn clean_step_tool_call() {
    let mut step = Step::ToolCall {
        id: "call_1".into(),
        name: "Read".into(),
        arguments: serde_json::json!({"path": "/Users/test/readme.md"}),
    };
    let mut r = CleanReport::default();
    Cleaner::new().clean_step(&mut step, &mut r);
    let debug = format!("{step:?}");
    assert!(!debug.contains("test"), "username should be redacted");
    assert_eq!(r.home_paths, 1);
}

#[test]
fn clean_step_tool_response() {
    let mut step = Step::ToolResponse {
        tool_call_id: "call_1".into(),
        name: "Read".into(),
        content: "Hello, john@example.com".into(),
        is_error: None,
    };
    let mut r = CleanReport::default();
    Cleaner::new().clean_step(&mut step, &mut r);
    let debug = format!("{step:?}");
    assert!(debug.contains("@example.com"));
    assert_eq!(r.emails, 1);
}

#[test]
fn clean_step_sensitive_key_in_object() {
    let mut step = Step::ToolCall {
        id: "call_1".into(),
        name: "Auth".into(),
        arguments: serde_json::json!({"api_key": "sk-proj-abcdefghijklmnopqrstuvwxyz1234567890"}),
    };
    let mut r = CleanReport::default();
    Cleaner::new().clean_step(&mut step, &mut r);
    let debug = format!("{step:?}");
    assert!(
        debug.contains("<redacted:sensitive_key>"),
        "sensitive key value should be redacted"
    );
}

#[test]
fn clean_metadata_fields() {
    let mut meta = crate::models::Metadata {
        source_file: "/Users/testuser/project/trace.jsonl".into(),
        session_id: "sess_abc123".into(),
        trace_type: "claude-code".into(),
        model_provider: Some("anthropic".into()),
        model: Some("claude-opus-4".into()),
        cwd: Some("/Users/testuser/project".into()),
        cli_version: Some("2.1.77".into()),
        usage: Some(serde_json::json!({"input_tokens": 10})),
        turn_count: 5,
        first_message_timestamp: Some("2026-01-01T00:00:00Z".into()),
        total_cost_usd: None,
        extra: indexmap::IndexMap::from([
            ("branch".into(), serde_json::json!("main")),
            ("email".into(), serde_json::json!("test@example.com")),
        ]),
    };
    let mut r = CleanReport::default();
    Cleaner::new().clean_metadata(&mut meta, &mut r);
    assert!(!meta.source_file.contains("testuser"));
    assert!(!meta.cwd.as_ref().unwrap().contains("testuser"));
    assert!(r.home_paths >= 2);
}

#[test]
fn multiple_keys_time() {
    let mut s = "\
        sk-proj-abc123def456ghi789jkl012mno345pqr\n\
        /Users/tester/.config\n\
        admin@evil.com\
    "
    .into();
    let mut r = CleanReport::default();
    Cleaner::new().clean_text(&mut s, &mut r);
    assert!(s.contains("<redacted:api_key>"));
    assert!(s.contains("user_"));
    assert!(s.contains("@example.com"));
    assert_eq!(r.api_keys, 1);
    assert_eq!(r.home_paths, 1);
    assert_eq!(r.emails, 1);
}
