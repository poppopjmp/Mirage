use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVisualizationRequest {
    pub correlation_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    pub format: Option<String>, // "svg", "png", "json", etc.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartVisualizationRequest {
    pub data_type: String, // "timeline", "bar", "pie", etc.
    pub entity_ids: Vec<Uuid>,
    pub format: Option<String>, // "svg", "png", "json", etc.
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportGenerationRequest {
    pub entity_ids: Vec<Uuid>,
    pub report_type: String, // "summary", "detailed", "executive", etc.
    pub format: Option<String>, // "pdf", "html", etc.
    pub options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub value: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResult {
    pub id: Uuid,
    pub format: String,
    pub content_type: String,
    pub data: String, // Base64 encoded for binary formats, raw data for text formats
    pub created_at: chrono::DateTime<chrono::Utc>,
}
