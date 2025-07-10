//! CSV formatter for reports

use serde_json::Value;

pub fn format_to_csv(data: &Value, title: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut csv = format!("Report: {}\n", title);
    csv.push_str(&format!(
        "Generated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Simple CSV conversion for JSON data
    match data {
        Value::Array(arr) => {
            if let Some(first) = arr.first() {
                if let Value::Object(obj) = first {
                    // Headers
                    let headers: Vec<String> = obj.keys().cloned().collect();
                    csv.push_str(&headers.join(","));
                    csv.push('\n');

                    // Data rows
                    for item in arr {
                        if let Value::Object(row) = item {
                            let values: Vec<String> = headers
                                .iter()
                                .map(|h| {
                                    row.get(h).map(|v| format_csv_value(v)).unwrap_or_default()
                                })
                                .collect();
                            csv.push_str(&values.join(","));
                            csv.push('\n');
                        }
                    }
                }
            }
        }
        Value::Object(obj) => {
            csv.push_str("Key,Value\n");
            for (key, value) in obj {
                csv.push_str(&format!("{},{}\n", key, format_csv_value(value)));
            }
        }
        _ => {
            csv.push_str("Value\n");
            csv.push_str(&format!("{}\n", format_csv_value(data)));
        }
    }

    Ok(csv)
}

fn format_csv_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "".to_string(),
        _ => format!("\"{}\"", value.to_string().replace('"', "\"\"")),
    }
}
