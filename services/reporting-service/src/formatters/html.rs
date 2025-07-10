//! HTML formatter for reports

use serde_json::Value;

pub fn format_to_html(data: &Value, title: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .report-section {{ margin-bottom: 20px; }}
        .report-data {{ background-color: #f5f5f5; padding: 10px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <div class="report-section">
        <h2>Report Data</h2>
        <div class="report-data">
            <pre>{}</pre>
        </div>
    </div>
</body>
</html>"#,
        title,
        title,
        serde_json::to_string_pretty(data)?
    );

    Ok(html)
}
