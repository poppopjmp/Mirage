use crate::models::ReportTemplateContext;
use mirage_common::Result;
use std::sync::Arc;
use handlebars::Handlebars;

// Re-export formatters
mod html;
mod pdf;
mod markdown;
mod json;
mod csv;
mod excel;

pub use html::HtmlFormatter;
pub use pdf::PdfFormatter;
pub use markdown::MarkdownFormatter;
pub use json::JsonFormatter;
pub use csv::CsvFormatter;
pub use excel::ExcelFormatter;

// Common trait for all formatters
pub trait ReportFormatter {
    fn format(&self, context: &ReportTemplateContext, template_name: &str) -> Result<(Vec<u8>, String)>;
}

// HTML formatter implementation
pub struct HtmlFormatterImpl {
    handlebars: Arc<Handlebars<'static>>,
}

impl HtmlFormatterImpl {
    pub fn new(handlebars: Arc<Handlebars<'static>>) -> Self {
        Self { handlebars }
    }
}

impl ReportFormatter for HtmlFormatterImpl {
    fn format(&self, context: &ReportTemplateContext, template_name: &str) -> Result<(Vec<u8>, String)> {
        // Use template_name + _html suffix or fallback to a default
        let template_key = if self.handlebars.has_template(&format!("{}_html", template_name)) {
            format!("{}_html", template_name)
        } else if self.handlebars.has_template(template_name) {
            template_name.to_string()
        } else {
            "summary_html".to_string() // Default template
        };
        
        let rendered = self.handlebars.render(&template_key, &context)
            .map_err(|e| mirage_common::Error::Internal(format!("Template rendering error: {}", e)))?;
        
        Ok((rendered.into_bytes(), "html".to_string()))
    }
}
