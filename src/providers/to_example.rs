use crate::models::{
    Message, Role, Step, Tool, ToolCallFunction, ToolCallKind, ToolCallRef, TrainingExample,
};
use crate::providers::NormalizedSession;

pub fn to_training_example(
    session: NormalizedSession,
    tools: Vec<Tool>,
    drop_incomplete: bool,
) -> Option<TrainingExample> {
    let mut messages: Vec<Message> = Vec::with_capacity(session.steps.len());
    let mut prompt = String::new();
    let mut pending_api_error: Option<String> = None;

    for step in &session.steps {
        match step {
            Step::User { content } => {
                pending_api_error = None;
                if prompt.is_empty() {
                    prompt = content.clone();
                }
                messages.push(msg_user(content));
            }
            Step::LlmOnly { content } => {
                pending_api_error = None;
                messages.push(msg_user(content));
            }
            Step::Thought { content, .. } => {
                try_merge_reasoning(&mut messages, content);
            }
            Step::AssistantText { content, api_error } => {
                let effective_error = api_error.clone().or_else(|| pending_api_error.take());
                try_merge_text(&mut messages, content, &effective_error);
            }
            Step::ToolCall {
                id,
                name,
                arguments,
            } => {
                try_merge_tool_call(
                    &mut messages,
                    ToolCallRef {
                        id: id.clone(),
                        kind: ToolCallKind::Function,
                        function: ToolCallFunction {
                            name: name.clone(),
                            arguments: arguments.clone(),
                        },
                    },
                );
            }
            Step::ToolResponse {
                tool_call_id,
                name,
                content,
                is_error,
            } => {
                if *is_error == Some(true) {
                    pending_api_error = Some("true".to_string());
                } else {
                    pending_api_error = None;
                }
                messages.push(msg_tool(
                    tool_call_id,
                    name,
                    content,
                    is_error.filter(|e| *e),
                ));
            }
            Step::SystemContext { content, .. } => {
                messages.push(Message {
                    role: Role::System,
                    content: Some(content.clone()),
                    masked: None,
                    reasoning_content: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    teich_provider_error: None,
                    is_error: None,
                });
            }
            Step::SyntheticArtifact { .. } | Step::Telemetry { .. } => {}
        }
    }

    if prompt.is_empty() {
        prompt = messages
            .iter()
            .find_map(|m| {
                if m.role == Role::User {
                    m.content.clone()
                } else {
                    None
                }
            })
            .unwrap_or_default();
    }

    let ex = TrainingExample {
        prompt,
        follow_up_prompts: vec![],
        messages,
        tools,
        metadata: session.metadata,
    };
    if !drop_incomplete || ex.is_complete() {
        Some(ex)
    } else {
        None
    }
}

fn msg_user(content: &str) -> Message {
    Message {
        role: Role::User,
        content: Some(content.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
        masked: None,
        teich_provider_error: None,
        is_error: None,
    }
}

fn msg_tool(id: &str, name: &str, content: &str, is_error: Option<bool>) -> Message {
    Message {
        role: Role::Tool,
        content: Some(content.to_string()),
        tool_call_id: Some(id.to_string()),
        name: Some(name.to_string()),
        reasoning_content: None,
        tool_calls: None,
        masked: None,
        teich_provider_error: None,
        is_error,
    }
}

fn try_merge_reasoning(messages: &mut Vec<Message>, content: &str) {
    if let Some(last) = messages.last_mut() {
        if last.role == Role::Assistant && last.reasoning_content.is_none() {
            last.reasoning_content = Some(content.to_string());
            return;
        }
    }
    messages.push(Message {
        role: Role::Assistant,
        content: Some(String::new()),
        reasoning_content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        name: None,
        masked: None,
        teich_provider_error: None,
        is_error: None,
    });
}

fn try_merge_text(messages: &mut Vec<Message>, content: &str, api_error: &Option<String>) {
    if let Some(last) = messages.last_mut() {
        if last.role == Role::Assistant && last.content.as_deref().is_none_or(|c| c.is_empty()) {
            last.content = Some(content.to_string());
            if let Some(err) = api_error {
                last.teich_provider_error = Some(err.clone());
            }
            return;
        }
    }
    messages.push(Message {
        role: Role::Assistant,
        content: Some(content.to_string()),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
        masked: None,
        teich_provider_error: api_error.clone(),
        is_error: None,
    });
}

fn try_merge_tool_call(messages: &mut Vec<Message>, tc: ToolCallRef) {
    if let Some(last) = messages.last_mut() {
        if last.role == Role::Assistant {
            last.tool_calls.get_or_insert_with(Vec::new).push(tc);
            return;
        }
    }
    messages.push(Message {
        role: Role::Assistant,
        content: Some(String::new()),
        reasoning_content: None,
        tool_calls: Some(vec![tc]),
        tool_call_id: None,
        name: None,
        masked: None,
        teich_provider_error: None,
        is_error: None,
    });
}
