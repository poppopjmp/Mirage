use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRequest {
    pub title: String,
    pub description: Option<String>,
    pub entity_ids: Vec<Uuid>,
    pub report_type: ReportType,
    pub format: ReportFormat,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Summary,
    Detailed,
    Executive,
    Technical,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportFormat {
    Html,
    Pdf,
    Markdown,
    Text,
    Json,
    Csv,
    Excel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub format: ReportFormat,
    pub file_path: String,
    pub file_size: u64,
    pub entity_count: usize,
    pub generated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supported_formats: Vec<ReportFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
    pub data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub relationships: Vec<RelationshipData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipData {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplateContext {
    pub title: String,
    pub description: Option<String>,
    pub entities: Vec<EntityData>,
    pub generated_at: DateTime<Utc>,
    pub generated_by: Option<String>,
    pub visualizations: Vec<VisualizationData>,
    pub custom_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub id: Uuid,
    pub visualization_type: String,
    pub data_url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}
