use indexmap::IndexMap;
use serde::{
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Metadata {
    pub source_file: String,
    pub session_id: String,
    pub trace_type: String,
    pub model_provider: Option<String>,
    pub model: Option<String>,
    pub cwd: Option<String>,
    pub cli_version: Option<String>,
    pub turn_count: usize,
    #[serde(default)]
    pub extra: IndexMap<String, serde_json::Value>,
    pub usage: Option<serde_json::Value>,
    pub total_cost_usd: Option<f64>,
    pub first_message_timestamp: Option<String>,
}

impl Serialize for Metadata {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_entry("source_file", &self.source_file)?;
        map.serialize_entry("session_id", &self.session_id)?;
        map.serialize_entry("trace_type", &self.trace_type)?;

        if let Some(ref v) = self.model_provider {
            map.serialize_entry("model_provider", v)?;
        } else if let Some(v) = self.extra.get("model_provider") {
            map.serialize_entry("model_provider", v)?;
        }
        if let Some(ref v) = self.model {
            map.serialize_entry("model", v)?;
        } else if let Some(v) = self.extra.get("model") {
            map.serialize_entry("model", v)?;
        }
        if let Some(ref v) = self.cwd {
            map.serialize_entry("cwd", v)?;
        } else if let Some(v) = self.extra.get("cwd") {
            map.serialize_entry("cwd", v)?;
        }
        if let Some(ref v) = self.cli_version {
            map.serialize_entry("cli_version", v)?;
        } else if let Some(v) = self.extra.get("cli_version") {
            map.serialize_entry("cli_version", v)?;
        }

        map.serialize_entry("turn_count", &self.turn_count)?;

        let emitted: std::collections::BTreeSet<&str> =
            ["model_provider", "model", "cwd", "cli_version"]
                .into_iter()
                .collect();
        let mut sorted_extra: Vec<(&str, &serde_json::Value)> = self
            .extra
            .iter()
            .filter(|(k, _)| !emitted.contains(k.as_str()))
            .map(|(k, v)| (k.as_str(), v))
            .collect();
        sorted_extra.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in &sorted_extra {
            map.serialize_entry(k, v)?;
        }

        if let Some(ref v) = self.usage {
            map.serialize_entry("usage", v)?;
        }
        if let Some(ref v) = self.total_cost_usd {
            map.serialize_entry("total_cost_usd", v)?;
        }
        if let Some(ref v) = self.first_message_timestamp {
            map.serialize_entry("first_message_timestamp", v)?;
        }

        map.end()
    }
}
