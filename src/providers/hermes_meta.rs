use indexmap::IndexMap;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn extract_extra(meta: &BTreeMap<String, Value>) -> IndexMap<String, Value> {
    let mut extra = IndexMap::new();
    let keys = [
        "parent_session_id",
        "system_prompt",
        "started_at",
        "ended_at",
        "end_reason",
        "message_count",
        "tool_call_count",
        "input_tokens",
        "output_tokens",
        "cache_read_tokens",
        "cache_write_tokens",
        "reasoning_tokens",
        "total_tokens",
        "estimated_cost_usd",
        "actual_cost_usd",
        "cost_status",
        "cost_source",
        "billing_provider",
        "billing_base_url",
        "billing_mode",
        "api_call_count",
        "teich_export_status",
        "teich_partial",
        "configured_model_provider",
        "configured_context_length",
        "hermes_source",
        "source",
        "total_cost",
    ];
    for k in &keys {
        extra.insert(k.to_string(), meta.get(*k).cloned().unwrap_or(Value::Null));
    }
    extra
}
