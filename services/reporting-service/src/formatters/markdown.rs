//! Markdown formatter for reports

use serde_json::Value;

pub fn format_to_markdown(data: &Value, title: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut markdown = format!("# {}\n\n", title);
    markdown.push_str(&format!(
        "**Generated:** {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    markdown.push_str("## Report Data\n\n");
    markdown.push_str("```json\n");
    markdown.push_str(&serde_json::to_string_pretty(data)?);
    markdown.push_str("\n```\n");

    Ok(markdown)
}

pub fn create_markdown_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut table = String::new();

    // Headers
    table.push_str("| ");
    for header in headers {
        table.push_str(header);
        table.push_str(" | ");
    }
    table.push('\n');

    // Separator
    table.push_str("| ");
    for _ in headers {
        table.push_str("--- | ");
    }
    table.push('\n');

    // Rows
    for row in rows {
        table.push_str("| ");
        for cell in row {
            table.push_str(cell);
            table.push_str(" | ");
        }
        table.push('\n');
    }

    table
}
