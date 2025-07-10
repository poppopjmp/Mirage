use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Types of visualizations supported by the service
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VisualizationType {
    Graph,
    Chart,
    Timeline,
    Map,
    Heatmap,
    Custom(String),
}

/// Types of chart visualizations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
    Area,
    Radar,
}

/// Graph visualization request
#[derive(Debug, Clone, Deserialize)]
pub struct GraphVisualizationRequest {
    pub title: String,
    pub description: Option<String>,
    pub data_source_id: Uuid,
    pub query: Option<String>,
    pub entity_types: Option<Vec<String>>,
    pub relationship_types: Option<Vec<String>>,
    pub max_nodes: Option<u32>,
    pub max_depth: Option<u32>,
    pub layout: Option<String>,
    pub style_options: Option<HashMap<String, serde_json::Value>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Chart visualization request
#[derive(Debug, Clone, Deserialize)]
pub struct ChartVisualizationRequest {
    pub title: String,
    pub description: Option<String>,
    pub data_source_id: Uuid,
    pub query: Option<String>,
    pub chart_type: ChartType,
    pub x_axis: String,
    pub y_axis: Vec<String>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
    pub style_options: Option<HashMap<String, serde_json::Value>>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Report generation request
#[derive(Debug, Clone, Deserialize)]
pub struct ReportGenerationRequest {
    pub title: String,
    pub description: Option<String>,
    pub visualizations: Vec<Uuid>,
    pub format: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Graph visualization request with rendering options
#[derive(Debug, Clone, Deserialize)]
pub struct RenderGraphRequest {
    pub correlation_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Chart visualization request with rendering options
#[derive(Debug, Clone, Deserialize)]
pub struct RenderChartRequest {
    pub entity_ids: Vec<Uuid>,
    pub data_type: String, // e.g., "timeline", "bar", "pie"
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: Option<String>,
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// Visualization model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Visualization {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub visualization_type: VisualizationType,
    pub data_source_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub config: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub thumbnail_url: Option<String>,
}

/// Visualization response
#[derive(Debug, Clone, Serialize)]
pub struct VisualizationResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub visualization_type: VisualizationType,
    pub data_source_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub config: serde_json::Value,
    pub thumbnail_url: Option<String>,
    pub render_url: String,
}

/// Report model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub visualizations: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub format: String,
    pub status: String,
    pub download_url: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub entity_type: String,
    pub value: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Visualization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationResult {
    pub id: Uuid,
    pub format: String,
    pub content_type: String,
    pub data: String, // Base64 encoded for binary formats, raw data for text formats
    pub created_at: chrono::DateTime<chrono::Utc>,
}
