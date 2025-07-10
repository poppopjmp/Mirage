use crate::config::TemplateConfig;
use crate::models::NotificationType;
use handlebars::Handlebars;
use mirage_common::{Error, Result};
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub struct TemplateRegistry {
    handlebars: Arc<Handlebars<'static>>,
}

impl TemplateRegistry {
    pub fn new(config: &TemplateConfig) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register templates from template directory
        let template_dir = Path::new(&config.dir);
        if template_dir.exists() && template_dir.is_dir() {
            // Register each .hbs file in the template directory
            if let Ok(entries) = fs::read_dir(template_dir) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            if let Some(file_name) = entry.file_name().to_str() {
                                if file_name.ends_with(".hbs") {
                                    let template_name = file_name.trim_end_matches(".hbs");
                                    let template_path = entry.path();
                                    if let Err(e) = handlebars
                                        .register_template_file(template_name, &template_path)
                                    {
                                        tracing::error!(
                                            "Failed to register template '{}': {}",
                                            template_name,
                                            e
                                        );
                                    } else {
                                        tracing::info!("Registered template: {}", template_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            tracing::warn!("Template directory does not exist: {}", config.dir);
        }

        // Register default templates in memory as fallback
        handlebars
            .register_template_string(
                "new_entity_email",
                include_str!("../templates/new_entity_email.hbs"),
            )
            .expect("Failed to register new_entity_email template");
        handlebars
            .register_template_string(
                "new_relationship_email",
                include_str!("../templates/new_relationship_email.hbs"),
            )
            .expect("Failed to register new_relationship_email template");
        handlebars
            .register_template_string(
                "scan_complete_email",
                include_str!("../templates/scan_complete_email.hbs"),
            )
            .expect("Failed to register scan_complete_email template");
        handlebars
            .register_template_string(
                "alert_triggered_email",
                include_str!("../templates/alert_triggered_email.hbs"),
            )
            .expect("Failed to register alert_triggered_email template");
        handlebars
            .register_template_string(
                "system_alert_email",
                include_str!("../templates/system_alert_email.hbs"),
            )
            .expect("Failed to register system_alert_email template");

        // Also register subject templates
        handlebars
            .register_template_string(
                "new_entity_subject",
                "New Entity Detected: {{entity_type}} - {{value}}",
            )
            .expect("Failed to register new_entity_subject template");
        handlebars
            .register_template_string(
                "new_relationship_subject",
                "New Relationship Detected: {{relationship_type}}",
            )
            .expect("Failed to register new_relationship_subject template");
        handlebars
            .register_template_string("scan_complete_subject", "Scan Completed: {{scan_name}}")
            .expect("Failed to register scan_complete_subject template");
        handlebars
            .register_template_string("alert_triggered_subject", "Alert Triggered: {{alert_name}}")
            .expect("Failed to register alert_triggered_subject template");
        handlebars
            .register_template_string("system_alert_subject", "System Alert: {{alert_type}}")
            .expect("Failed to register system_alert_subject template");

        Self {
            handlebars: Arc::new(handlebars),
        }
    }

    pub fn render_subject(
        &self,
        notification_type: &NotificationType,
        data: &serde_json::Value,
    ) -> Result<String> {
        let template_name = match notification_type {
            NotificationType::NewEntity => "new_entity_subject",
            NotificationType::NewRelationship => "new_relationship_subject",
            NotificationType::ScanComplete => "scan_complete_subject",
            NotificationType::AlertTriggered => "alert_triggered_subject",
            NotificationType::SystemAlert => "system_alert_subject",
            NotificationType::Custom(name) => {
                // For custom types, check if a custom template exists, otherwise use a generic one
                let custom_name = format!("{}_subject", name);
                if self.handlebars.has_template(&custom_name) {
                    &custom_name
                } else {
                    "generic_subject"
                }
            }
        };

        let rendered = self.handlebars.render(template_name, data).map_err(|e| {
            Error::Internal(format!(
                "Failed to render subject template {}: {}",
                template_name, e
            ))
        })?;

        Ok(rendered)
    }

    pub fn render_content(
        &self,
        notification_type: &NotificationType,
        data: &serde_json::Value,
    ) -> Result<String> {
        let template_name = match notification_type {
            NotificationType::NewEntity => "new_entity_email",
            NotificationType::NewRelationship => "new_relationship_email",
            NotificationType::ScanComplete => "scan_complete_email",
            NotificationType::AlertTriggered => "alert_triggered_email",
            NotificationType::SystemAlert => "system_alert_email",
            NotificationType::Custom(name) => {
                // For custom types, check if a custom template exists, otherwise use a generic one
                let custom_name = format!("{}_email", name);
                if self.handlebars.has_template(&custom_name) {
                    &custom_name
                } else {
                    "generic_email"
                }
            }
        };

        let rendered = self.handlebars.render(template_name, data).map_err(|e| {
            Error::Internal(format!(
                "Failed to render content template {}: {}",
                template_name, e
            ))
        })?;

        Ok(rendered)
    }
}
