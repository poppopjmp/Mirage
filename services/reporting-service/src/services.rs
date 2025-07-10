use crate::config::AppConfig;
use crate::formatters::{
    CsvFormatter, ExcelFormatter, HtmlFormatter, JsonFormatter, MarkdownFormatter, PdfFormatter,
    ReportFormatter,
};
use crate::models::{
    EntityData, RelationshipData, Report, ReportFormat, ReportRequest, ReportTemplate,
    ReportTemplateContext, ReportType, VisualizationData,
};
use chrono::Utc;
use handlebars::Handlebars;
use mirage_common::Error;
use reqwest::Client;
use sanitize_filename::sanitize;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ReportService {
    client: Arc<Client>,
    config: Arc<AppConfig>,
    handlebars: Arc<Handlebars<'static>>,
}

impl ReportService {
    pub fn new(client: Client, config: AppConfig) -> Self {
        // Ensure output directory exists
        let output_dir = Path::new(&config.report.output_dir);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir).expect("Failed to create report output directory");
        }

        // Set up template registry
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);

        // Register templates from template directory
        let template_dir = Path::new(&config.report.template_dir);
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
            tracing::warn!(
                "Template directory does not exist: {}",
                config.report.template_dir
            );
        }

        // Register default templates from memory as a fallback
        handlebars
            .register_template_string(
                "summary_html",
                include_str!("../templates/summary_html.hbs"),
            )
            .expect("Failed to register default summary HTML template");
        handlebars
            .register_template_string(
                "detailed_html",
                include_str!("../templates/detailed_html.hbs"),
            )
            .expect("Failed to register default detailed HTML template");

        Self {
            client: Arc::new(client),
            config: Arc::new(config),
            handlebars: Arc::new(handlebars),
        }
    }

    pub async fn generate_report(
        &self,
        request: ReportRequest,
    ) -> Result<Report, mirage_common::Error> {
        // Validate request
        if request.entity_ids.is_empty() {
            return Err(Error::Validation(
                "At least one entity ID must be provided".into(),
            ));
        }

        if request.entity_ids.len() > self.config.report.max_entities_per_report {
            return Err(Error::Validation(format!(
                "Too many entities. Maximum allowed: {}, provided: {}",
                self.config.report.max_entities_per_report,
                request.entity_ids.len()
            )));
        }

        // Fetch data for each entity
        let entities = self.fetch_entities_data(&request.entity_ids).await?;

        // Generate visualizations if needed
        let visualizations = self.generate_visualizations(&entities, &request).await?;

        // Create template context
        let context = ReportTemplateContext {
            title: request.title.clone(),
            description: request.description.clone(),
            entities,
            generated_at: Utc::now(),
            generated_by: Some("Mirage OSINT Platform".to_string()),
            visualizations,
            custom_data: None,
        };

        // Select formatter based on requested format
        let formatter: Box<dyn ReportFormatter> = match request.format {
            ReportFormat::Html => Box::new(HtmlFormatter::new(self.handlebars.clone())),
            ReportFormat::Pdf => Box::new(PdfFormatter::new(self.handlebars.clone())),
            ReportFormat::Markdown => Box::new(MarkdownFormatter::new()),
            ReportFormat::Json => Box::new(JsonFormatter::new()),
            ReportFormat::Csv => Box::new(CsvFormatter::new()),
            ReportFormat::Excel => Box::new(ExcelFormatter::new()),
            ReportFormat::Text => {
                return Err(Error::Validation("Text format not yet implemented".into()))
            }
        };

        // Get template name based on report type
        let template_name = match &request.report_type {
            ReportType::Summary => "summary",
            ReportType::Detailed => "detailed",
            ReportType::Executive => "executive",
            ReportType::Technical => "technical",
            ReportType::Custom(name) => name,
        };

        // Generate content
        let (content, extension) = formatter.format(&context, template_name)?;

        // Create a unique, sanitized filename
        let base_filename = sanitize(&format!(
            "{}_{}",
            request.title.to_lowercase().replace(' ', "_"),
            Utc::now().format("%Y%m%d_%H%M%S")
        ));
        let report_id = Uuid::new_v4();
        let filename = format!("{}_{}.{}", base_filename, report_id, extension);
        let file_path = PathBuf::from(&self.config.report.output_dir).join(&filename);

        // Write to file
        let mut file = File::create(&file_path)
            .map_err(|e| Error::Internal(format!("Failed to create report file: {}", e)))?;
        file.write_all(&content)
            .map_err(|e| Error::Internal(format!("Failed to write to report file: {}", e)))?;

        // Get file size
        let file_size = file_path.metadata().map(|m| m.len()).unwrap_or(0);

        // Create report metadata
        let report = Report {
            id: report_id,
            title: request.title,
            description: request.description,
            created_at: Utc::now(),
            format: request.format,
            file_path: filename,
            file_size,
            entity_count: request.entity_ids.len(),
            generated_by: Some("Mirage OSINT Platform".to_string()),
        };

        Ok(report)
    }

    pub fn get_available_templates(&self) -> Vec<ReportTemplate> {
        let mut templates = Vec::new();

        // Add built-in templates
        templates.push(ReportTemplate {
            id: "summary".to_string(),
            name: "Summary Report".to_string(),
            description: "A brief summary of the entities and their relationships".to_string(),
            supported_formats: vec![
                ReportFormat::Html,
                ReportFormat::Pdf,
                ReportFormat::Markdown,
                ReportFormat::Json,
            ],
        });

        templates.push(ReportTemplate {
            id: "detailed".to_string(),
            name: "Detailed Report".to_string(),
            description: "A comprehensive report with all available entity details".to_string(),
            supported_formats: vec![
                ReportFormat::Html,
                ReportFormat::Pdf,
                ReportFormat::Markdown,
                ReportFormat::Json,
            ],
        });

        templates.push(ReportTemplate {
            id: "executive".to_string(),
            name: "Executive Summary".to_string(),
            description: "High-level overview suitable for executive briefing".to_string(),
            supported_formats: vec![
                ReportFormat::Html,
                ReportFormat::Pdf,
                ReportFormat::Markdown,
            ],
        });

        templates.push(ReportTemplate {
            id: "technical".to_string(),
            name: "Technical Report".to_string(),
            description: "Technical details with raw data and technical analysis".to_string(),
            supported_formats: vec![
                ReportFormat::Html,
                ReportFormat::Pdf,
                ReportFormat::Json,
                ReportFormat::Excel,
            ],
        });

        // Add custom templates from handlebars registry
        for name in self.handlebars.get_templates().keys() {
            if !templates.iter().any(|t| &t.id == name) {
                if !name.contains("_html") && !name.contains("_pdf") {
                    templates.push(ReportTemplate {
                        id: name.clone(),
                        name: name.replace('_', " "),
                        description: format!("Custom template: {}", name),
                        supported_formats: vec![
                            ReportFormat::Html,
                            ReportFormat::Pdf,
                            ReportFormat::Markdown,
                            ReportFormat::Json,
                        ],
                    });
                }
            }
        }

        templates
    }

    pub fn get_report_file(&self, report_id: &Uuid) -> Result<PathBuf, mirage_common::Error> {
        // In a real implementation, we would look up the report metadata in a database
        // For now, we'll just check if a file exists with this ID

        let report_dir = Path::new(&self.config.report.output_dir);

        if let Ok(entries) = fs::read_dir(report_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.contains(&report_id.to_string()) {
                        return Ok(entry.path());
                    }
                }
            }
        }

        Err(Error::NotFound(format!(
            "Report with ID {} not found",
            report_id
        )))
    }

    // Helper methods

    async fn fetch_entities_data(
        &self,
        entity_ids: &[Uuid],
    ) -> Result<Vec<EntityData>, mirage_common::Error> {
        let mut entities = Vec::new();

        for id in entity_ids {
            // Fetch entity data from storage service
            let url = format!("{}/api/v1/data/{}", self.config.data_storage.url, id);

            let response =
                self.client.get(&url).send().await.map_err(|e| {
                    Error::ExternalApi(format!("Failed to fetch entity data: {}", e))
                })?;

            if !response.status().is_success() {
                tracing::warn!(
                    "Failed to fetch entity data for ID {}: {}",
                    id,
                    response.status()
                );
                continue;
            }

            let entity_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| Error::ExternalApi(format!("Failed to parse entity data: {}", e)))?;

            // Fetch relationships
            let relationships_url = format!(
                "{}/api/v1/data/relationships/{}",
                self.config.data_storage.url, id
            );

            let relationships_response = self
                .client
                .get(&relationships_url)
                .send()
                .await
                .map_err(|e| Error::ExternalApi(format!("Failed to fetch relationships: {}", e)))?;

            let relationships: Vec<RelationshipData> =
                if relationships_response.status().is_success() {
                    relationships_response.json().await.map_err(|e| {
                        Error::ExternalApi(format!("Failed to parse relationships: {}", e))
                    })?
                } else {
                    Vec::new()
                };

            // Convert to EntityData
            if let (Some(entity_id), Some(entity_type), Some(value)) = (
                entity_data["id"]
                    .as_str()
                    .map(|s| Uuid::parse_str(s).ok())
                    .flatten(),
                entity_data["entity_type"].as_str(),
                entity_data["value"].as_str(),
            ) {
                let mut metadata = HashMap::new();
                if let Some(meta_obj) = entity_data["metadata"].as_object() {
                    for (k, v) in meta_obj {
                        if let Some(v_str) = v.as_str() {
                            metadata.insert(k.clone(), v_str.to_string());
                        }
                    }
                }

                let created_at = if let Some(ts) = entity_data["created_at"].as_str() {
                    chrono::DateTime::parse_from_rfc3339(ts)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                } else {
                    Utc::now()
                };

                entities.push(EntityData {
                    id: entity_id,
                    entity_type: entity_type.to_string(),
                    value: value.to_string(),
                    data: entity_data
                        .get("data")
                        .unwrap_or(&serde_json::Value::Null)
                        .clone(),
                    metadata,
                    created_at,
                    relationships,
                });
            }
        }

        Ok(entities)
    }

    async fn generate_visualizations(
        &self,
        entities: &[EntityData],
        request: &ReportRequest,
    ) -> Result<Vec<VisualizationData>, mirage_common::Error> {
        let mut visualizations = Vec::new();

        // If we have multiple entities, create a correlation graph
        if entities.len() > 1 {
            let entity_ids: Vec<Uuid> = entities.iter().map(|e| e.id).collect();

            // Request a graph visualization from the visualization service
            let viz_url = format!(
                "{}/api/v1/visualizations/graph",
                self.config.visualization.url
            );

            let viz_request = serde_json::json!({
                "entity_id": entity_ids[0],  // Use first entity as the starting point
                "format": "svg",
                "width": 800,
                "height": 600,
                "options": {
                    "title": format!("Relationships between {} entities", entity_ids.len())
                }
            });

            let response = self
                .client
                .post(&viz_url)
                .json(&viz_request)
                .send()
                .await
                .map_err(|e| {
                    Error::ExternalApi(format!("Failed to generate graph visualization: {}", e))
                })?;

            if response.status().is_success() {
                let viz_result: serde_json::Value = response.json().await.map_err(|e| {
                    Error::ExternalApi(format!("Failed to parse visualization result: {}", e))
                })?;

                if let (Some(viz_id), Some(data)) = (
                    viz_result["id"]
                        .as_str()
                        .map(|s| Uuid::parse_str(s).ok())
                        .flatten(),
                    viz_result["data"].as_str(),
                ) {
                    // In a real implementation, we would store this in a file and reference it
                    // For now, we'll use a dummy URL
                    visualizations.push(VisualizationData {
                        id: viz_id,
                        visualization_type: "graph".to_string(),
                        data_url: format!("/api/v1/visualizations/{}", viz_id),
                        title: Some("Entity Relationship Graph".to_string()),
                        description: Some(format!(
                            "Relationships between {} entities",
                            entity_ids.len()
                        )),
                    });
                }
            } else {
                tracing::warn!(
                    "Failed to generate graph visualization: {}",
                    response.status()
                );
            }
        }

        // If appropriate for the report type, add entity type distribution chart
        if matches!(
            request.report_type,
            ReportType::Summary | ReportType::Executive | ReportType::Detailed
        ) {
            let viz_url = format!(
                "{}/api/v1/visualizations/chart",
                self.config.visualization.url
            );

            let viz_request = serde_json::json!({
                "data_type": "pie",
                "entity_ids": entities.iter().map(|e| e.id).collect::<Vec<Uuid>>(),
                "format": "svg",
                "width": 500,
                "height": 400,
                "options": {
                    "title": "Entity Type Distribution"
                }
            });

            let response = self
                .client
                .post(&viz_url)
                .json(&viz_request)
                .send()
                .await
                .map_err(|e| {
                    Error::ExternalApi(format!("Failed to generate chart visualization: {}", e))
                })?;

            if response.status().is_success() {
                let viz_result: serde_json::Value = response.json().await.map_err(|e| {
                    Error::ExternalApi(format!("Failed to parse visualization result: {}", e))
                })?;

                if let (Some(viz_id), Some(_data)) = (
                    viz_result["id"]
                        .as_str()
                        .map(|s| Uuid::parse_str(s).ok())
                        .flatten(),
                    viz_result["data"].as_str(),
                ) {
                    visualizations.push(VisualizationData {
                        id: viz_id,
                        visualization_type: "pie",
                        data_url: format!("/api/v1/visualizations/{}", viz_id),
                        title: Some("Entity Type Distribution".to_string()),
                        description: Some(
                            "Distribution of entity types in the dataset".to_string(),
                        ),
                    });
                }
            } else {
                tracing::warn!(
                    "Failed to generate pie chart visualization: {}",
                    response.status()
                );
            }
        }

        Ok(visualizations)
    }
}
