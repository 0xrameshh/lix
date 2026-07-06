use regex::Regex;
use serde_json::Value;

use crate::models::Step;

mod helpers;
mod patterns;
mod report;
#[cfg(test)]
mod tests;

pub use helpers::{dummy_hex, dummy_username};
pub use patterns::{API_KEY_PATTERNS, SENSITIVE_KEYS};
pub use report::CleanReport;

#[derive(Clone)]
pub struct Cleaner {
    api_keys: Vec<(Regex, &'static str)>,
    home_path: Regex,
    encoded_home_path: Regex,
    email: Regex,
    jwt: Regex,
    bearer: Regex,
    generic_secret: Regex,
    credential_url: Regex,
    private_key_block: Regex,
    env_assignment: Regex,
    query_secret: Regex,
    sensitive_keys: &'static [&'static str],
}

impl Default for Cleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl Cleaner {
    pub fn new() -> Self {
        Self {
            api_keys: API_KEY_PATTERNS.iter().map(|(r, k)| (Regex::new(r).unwrap(), *k)).collect(),
            home_path: Regex::new(r#"(?P<prefix>(?:[A-Za-z]:)?[\\/]+(?:home|Users)[\\/]+)(?P<username>[^\\/:\s"'<>|]+)"#).unwrap(),
            encoded_home_path: Regex::new(r#"(?P<prefix>-(?:home|Users)-)(?P<username>[A-Za-z0-9._]+)(?:$|[-\\/\s"'])"#).unwrap(),
            email: Regex::new(r"(?P<local>[A-Za-z0-9._%+\-]+)@(?P<domain>[A-Za-z0-9.\-]+\.[A-Za-z]{2,})").unwrap(),
            jwt: Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").unwrap(),
            bearer: Regex::new(r"(?i)(\bBearer\s+)([A-Za-z0-9._~+/=-]{24,})").unwrap(),
            generic_secret: Regex::new(r#"(?i)(\b(?:[A-Za-z0-9]+[_-])*(?:api[_-]?key|token|secret|password)\b\\?["']?[^\S\r\n]*[:=][^\S\r\n]*\\?["']?)([A-Za-z0-9_~+/=-]{24,})"#).unwrap(),
            credential_url: Regex::new(r"(?i)\b(?P<scheme>[A-Za-z][A-Za-z0-9+.-]{1,32}://)(?P<user>[^:@/\s?#]*):(?P<password>[^@\s/?#]+)@").unwrap(),
            private_key_block: Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----").unwrap(),
            env_assignment: Regex::new(r"(?i)(\b(?:[A-Z][A-Z0-9_]{3,})\s*[:=]\s*)([A-Za-z0-9_~+/=-]{24,})").unwrap(),
            query_secret: Regex::new(r#"(?i)(?P<prefix>[?&](?P<name>[A-Za-z0-9_.-]*(?:api[_-]?key|token|secret|password|signature|sig|access[_-]?key|client[_-]?secret)[A-Za-z0-9_.-]*)=)(?P<value>[^&#\s"'<>()]+)"#).unwrap(),
            sensitive_keys: SENSITIVE_KEYS,
        }
    }

    pub fn clean_text(&self, text: &mut String, report: &mut CleanReport) {
        let mut out = String::with_capacity(text.len());
        std::mem::swap(&mut out, text);

        for (re, kind) in &self.api_keys {
            if re.is_match(&out) {
                out = re
                    .replace_all(&out, |c: &regex::Captures| {
                        let prefix = c.name("prefix").map(|m| m.as_str()).unwrap_or("");
                        format!("{}<redacted:api_key>", prefix)
                    })
                    .into_owned();
                report.bump(kind);
            }
        }

        if self.home_path.is_match(&out) {
            out = self
                .home_path
                .replace_all(&out, |c: &regex::Captures| {
                    let prefix = c.name("prefix").map(|m| m.as_str()).unwrap_or("");
                    let user = c.name("username").map(|m| m.as_str()).unwrap_or("");
                    format!("{}{}", prefix, dummy_username(user))
                })
                .into_owned();
            report.bump("home_path");
        }

        if self.encoded_home_path.is_match(&out) {
            out = self
                .encoded_home_path
                .replace_all(&out, |c: &regex::Captures| {
                    let prefix = c.name("prefix").map(|m| m.as_str()).unwrap_or("");
                    let user = c.name("username").map(|m| m.as_str()).unwrap_or("");
                    format!("{}{}", prefix, dummy_username(user))
                })
                .into_owned();
            report.bump("home_path");
        }

        if self.email.is_match(&out) {
            out = self
                .email
                .replace_all(&out, |c: &regex::Captures| {
                    let local = c.name("local").map(|m| m.as_str()).unwrap_or("");
                    format!("user_{}@example.com", dummy_hex(local))
                })
                .into_owned();
            report.bump("email");
        }

        for (re, label) in &[
            (&self.jwt, "jwt"),
            (&self.bearer, "bearer"),
            (&self.generic_secret, "generic_secret"),
            (&self.credential_url, "credential_url"),
            (&self.private_key_block, "private_key_block"),
            (&self.env_assignment, "env_assignment"),
            (&self.query_secret, "query_secret"),
        ] {
            if re.is_match(&out) {
                let replacement = match *label {
                    "jwt" => "<redacted:jwt>",
                    "bearer" => "${1}<redacted:bearer>",
                    "generic_secret" => "${1}<redacted:secret>",
                    "credential_url" => "${scheme}<redacted_user>:<redacted_password>@",
                    "private_key_block" => "<redacted:private_key_block>",
                    "env_assignment" => "${1}<redacted:env_value>",
                    "query_secret" => "${prefix}<redacted:query_secret>",
                    _ => "<redacted>",
                };
                out = re.replace_all(&out, replacement).into_owned();
                report.bump(label);
            }
        }

        *text = out;
    }

    pub fn clean_value(&self, value: &mut Value, report: &mut CleanReport) {
        match value {
            Value::String(s) => self.clean_text(s, report),
            Value::Array(arr) => {
                for v in arr {
                    self.clean_value(v, report);
                }
            }
            Value::Object(obj) => {
                for (k, v) in obj.iter_mut() {
                    if self.sensitive_keys.contains(&k.as_str()) && matches!(v, Value::String(_)) {
                        *v = Value::String("<redacted:sensitive_key>".to_string());
                        report.bump("env_assignment");
                    }
                    self.clean_value(v, report);
                }
            }
            _ => {}
        }
    }

    pub fn clean_metadata(&self, meta: &mut crate::models::Metadata, report: &mut CleanReport) {
        self.clean_text(&mut meta.source_file, report);
        self.clean_text(&mut meta.session_id, report);
        self.clean_text(&mut meta.trace_type, report);
        if let Some(ref mut val) = meta.model_provider {
            self.clean_text(val, report);
        }
        if let Some(ref mut val) = meta.model {
            self.clean_text(val, report);
        }
        if let Some(ref mut val) = meta.cwd {
            self.clean_text(val, report);
        }
        if let Some(ref mut val) = meta.cli_version {
            self.clean_text(val, report);
        }
        if let Some(ref mut val) = meta.usage {
            self.clean_value(val, report);
        }
        if let Some(ref mut val) = meta.first_message_timestamp {
            self.clean_text(val, report);
        }
        for val in meta.extra.values_mut() {
            self.clean_value(val, report);
        }
    }

    pub fn clean_step(&self, step: &mut Step, report: &mut CleanReport) {
        match step {
            Step::User { content }
            | Step::LlmOnly { content }
            | Step::Thought { content, .. }
            | Step::AssistantText { content, .. } => self.clean_text(content, report),
            Step::ToolCall {
                name, arguments, ..
            } => {
                self.clean_text(name, report);
                self.clean_value(arguments, report);
            }
            Step::ToolResponse { name, content, .. } => {
                self.clean_text(name, report);
                self.clean_text(content, report);
            }
            Step::SystemContext { content, .. } => self.clean_text(content, report),
            Step::SyntheticArtifact { .. } | Step::Telemetry { .. } => {}
        }
    }
}
