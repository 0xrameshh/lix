use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceType {
    ClaudeCode,
    Codex,
    Pi,
    Openclaw,
    Hermes,
    Cursor,
    Droid,
    ExternalAgent,
    Chat,
    #[default]
    Unknown,
}

impl TraceType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude-code",
            Self::Codex => "codex",
            Self::Pi => "pi",
            Self::Openclaw => "openclaw",
            Self::Hermes => "hermes",
            Self::Cursor => "cursor",
            Self::Droid => "droid",
            Self::ExternalAgent => "external-agent",
            Self::Chat => "chat",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    User {
        content: String,
    },
    LlmOnly {
        content: String,
    },
    Thought {
        content: String,
        signature: Option<String>,
    },
    AssistantText {
        content: String,
        api_error: Option<String>,
    },
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResponse {
        tool_call_id: String,
        name: String,
        content: String,
        is_error: Option<bool>,
    },
    SystemContext {
        content: String,
        subtype: SystemSubtype,
    },
    SyntheticArtifact {
        reason: SyntheticReason,
    },
    Telemetry {
        event_type: String,
        payload: Option<serde_json::Value>,
    },
}

impl Step {
    pub fn is_droppable(&self) -> bool {
        matches!(self, Self::SyntheticArtifact { .. })
    }

    pub fn role_label(&self) -> &'static str {
        match self {
            Self::User { .. } | Self::LlmOnly { .. } => "user",
            Self::Thought { .. } => "thought",
            Self::AssistantText { .. } => "assistant_text",
            Self::ToolCall { .. } => "tool_call",
            Self::ToolResponse { .. } => "tool_response",
            Self::SystemContext { .. } => "system_context",
            Self::SyntheticArtifact { .. } => "synthetic",
            Self::Telemetry { .. } => "telemetry",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemSubtype {
    DeferredToolsDelta,
    McpInstructionsDelta,
    SkillListing,
    CommandPermissions,
    DateChange,
    HookContext,
    AwaySummary,
    SessionRecap,
    PlanModeExit,
    EditedTextFile,
    TaskReminder,
    StopHookSummary,
    Informational,
    Other,
}

impl SystemSubtype {
    pub fn from_attachment_type(s: &str) -> Self {
        match s {
            "deferred_tools_delta" => Self::DeferredToolsDelta,
            "mcp_instructions_delta" => Self::McpInstructionsDelta,
            "skill_listing" => Self::SkillListing,
            "command_permissions" => Self::CommandPermissions,
            "date_change" => Self::DateChange,
            "hook_additional_context" => Self::HookContext,
            "edited_text_file" => Self::EditedTextFile,
            "task_reminder" => Self::TaskReminder,
            "plan_mode_exit" => Self::PlanModeExit,
            _ => Self::Other,
        }
    }

    pub fn from_system_subtype(s: &str) -> Self {
        match s {
            "stop_hook_summary" => Self::StopHookSummary,
            "away_summary" => Self::AwaySummary,
            "informational" => Self::Informational,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyntheticReason {
    RateLimit,
    SessionLimit,
    NoResponseRequested,
    ApiError { kind: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Developer,
    User,
    Assistant,
    Tool,
}
