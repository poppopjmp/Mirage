//! JSON formatter for reports

use serde_json::Value;

pub fn format_to_json(data: &Value, title: &str) -> Result<String, Box<dyn std::error::Error>> {
    let report = serde_json::json!({
        "title": title,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "data": data
    });

    Ok(serde_json::to_string_pretty(&report)?)
}

pub fn format_to_json_compact(
    data: &Value,
    title: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let report = serde_json::json!({
        "title": title,
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "data": data
    });

    Ok(serde_json::to_string(&report)?)
}
