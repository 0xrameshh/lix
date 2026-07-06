use super::*;
use serde_json::Value;

#[test]
fn trace_type_roundtrip() {
    for tt in &[
        TraceType::ClaudeCode,
        TraceType::Codex,
        TraceType::Pi,
        TraceType::Openclaw,
        TraceType::Hermes,
        TraceType::Cursor,
        TraceType::Droid,
        TraceType::ExternalAgent,
        TraceType::Chat,
        TraceType::Unknown,
    ] {
        let json = serde_json::to_string(tt).unwrap();
        let back: TraceType = serde_json::from_str(&json).unwrap();
        assert_eq!(*tt, back);
    }
}

#[test]
fn step_is_droppable() {
    assert!(Step::SyntheticArtifact {
        reason: SyntheticReason::RateLimit,
    }
    .is_droppable());
    assert!(!Step::User {
        content: "hi".into()
    }
    .is_droppable());
    assert!(!Step::AssistantText {
        content: "hello".into(),
        api_error: None,
    }
    .is_droppable());
}

#[test]
fn step_role_labels() {
    assert_eq!(Step::User { content: "".into() }.role_label(), "user");
    assert_eq!(
        Step::ToolCall {
            id: "".into(),
            name: "".into(),
            arguments: Value::Null
        }
        .role_label(),
        "tool_call"
    );
    assert_eq!(
        Step::SyntheticArtifact {
            reason: SyntheticReason::RateLimit
        }
        .role_label(),
        "synthetic"
    );
}

#[test]
fn message_roundtrip() {
    let m = Message {
        role: Role::Assistant,
        content: Some("Hello".into()),
        reasoning_content: Some("Let me think...".into()),
        tool_calls: Some(vec![ToolCallRef {
            id: "call_1".into(),
            kind: ToolCallKind::Function,
            function: ToolCallFunction {
                name: "Read".into(),
                arguments: serde_json::json!({"path": "README.md"}),
            },
        }]),
        tool_call_id: None,
        name: None,
        masked: None,
        teich_provider_error: None,
        is_error: None,
    };
    let json = serde_json::to_string(&m).unwrap();
    let back: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(back.role, Role::Assistant);
    assert_eq!(back.content.as_deref(), Some("Hello"));
    assert_eq!(back.tool_calls.as_ref().unwrap().len(), 1);
}

#[test]
fn training_example_complete() {
    let ex = TrainingExample {
        prompt: "Hi".into(),
        follow_up_prompts: vec![],
        messages: vec![
            Message {
                role: Role::User,
                content: Some("Hi".into()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
                masked: None,
                teich_provider_error: None,
                is_error: None,
            },
            Message {
                role: Role::Assistant,
                content: Some("Hello".into()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
                masked: None,
                teich_provider_error: None,
                is_error: None,
            },
        ],
        tools: vec![],
        metadata: Metadata::default(),
    };
    assert!(ex.is_complete());
}

#[test]
fn training_example_incomplete_ends_with_tool() {
    let ex = TrainingExample {
        prompt: "Read file".into(),
        follow_up_prompts: vec![],
        messages: vec![
            Message {
                role: Role::User,
                content: Some("Read file".into()),
                reasoning_content: None,
                tool_calls: None,
                tool_call_id: None,
                name: None,
                masked: None,
                teich_provider_error: None,
                is_error: None,
            },
            Message {
                role: Role::Tool,
                content: Some("file content".into()),
                tool_call_id: Some("call_1".into()),
                name: Some("Read".into()),
                reasoning_content: None,
                tool_calls: None,
                masked: None,
                teich_provider_error: None,
                is_error: None,
            },
        ],
        tools: vec![],
        metadata: Metadata::default(),
    };
    assert!(!ex.is_complete());
}

#[test]
fn training_example_empty_messages() {
    let ex = TrainingExample {
        prompt: String::new(),
        follow_up_prompts: vec![],
        messages: vec![],
        tools: vec![],
        metadata: Metadata::default(),
    };
    assert!(!ex.is_complete());
}

#[test]
fn metadata_default() {
    let m = Metadata::default();
    assert!(m.session_id.is_empty());
    assert_eq!(m.turn_count, 0);
    assert!(m.extra.is_empty());
}

#[test]
fn role_serde() {
    assert_eq!(serde_json::to_string(&Role::System).unwrap(), r#""system""#);
    assert_eq!(serde_json::to_string(&Role::User).unwrap(), r#""user""#);
    assert_eq!(
        serde_json::to_string(&Role::Assistant).unwrap(),
        r#""assistant""#
    );
    assert_eq!(serde_json::to_string(&Role::Tool).unwrap(), r#""tool""#);
}

#[test]
fn tool_entry_kind_serde() {
    assert_eq!(
        serde_json::to_string(&ToolEntryKind::Function).unwrap(),
        r#""function""#
    );
}

#[test]
fn system_subtype_from_attachment_type() {
    assert!(matches!(
        SystemSubtype::from_attachment_type("deferred_tools_delta"),
        SystemSubtype::DeferredToolsDelta
    ));
    assert!(matches!(
        SystemSubtype::from_attachment_type("unknown_type"),
        SystemSubtype::Other
    ));
}

#[test]
fn system_subtype_from_system_subtype() {
    assert!(matches!(
        SystemSubtype::from_system_subtype("stop_hook_summary"),
        SystemSubtype::StopHookSummary
    ));
    assert!(matches!(
        SystemSubtype::from_system_subtype("away_summary"),
        SystemSubtype::AwaySummary
    ));
}
