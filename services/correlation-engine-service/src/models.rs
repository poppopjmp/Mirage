use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityNode {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub confidence: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub confidence: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationRequest {
    pub entity_id: Uuid,
    pub max_depth: Option<i32>,
    pub min_confidence: Option<u8>,
    pub include_entities: Option<Vec<String>>,
    pub exclude_entities: Option<Vec<String>>,
    pub include_relationships: Option<Vec<String>>,
    pub exclude_relationships: Option<Vec<String>>,
    pub max_entities: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCorrelationRequest {
    pub entity_ids: Vec<Uuid>,
    pub max_depth: Option<i32>,
    pub min_confidence: Option<u8>,
    pub include_entities: Option<Vec<String>>,
    pub exclude_entities: Option<Vec<String>>,
    pub include_relationships: Option<Vec<String>>,
    pub exclude_relationships: Option<Vec<String>>,
    pub max_entities: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationResult {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub query_entity_id: Uuid,
    pub depth: i32,
    pub node_count: i32,
    pub relationship_count: i32,
    pub nodes: Vec<GraphNode>,
    pub relationships: Vec<GraphRelationship>,
    pub insights: Vec<CorrelationInsight>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
    pub data: HashMap<String, serde_json::Value>,
    pub source: String,
    pub confidence: u8,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub importance: f32, // Computed value based on centrality metrics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    pub id: Uuid,
    pub relationship_type: String,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub data: HashMap<String, serde_json::Value>,
    pub source: String,
    pub confidence: u8,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub strength: f32, // Computed value based on relationship analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationInsight {
    pub insight_type: InsightType,
    pub title: String,
    pub description: String,
    pub severity: InsightSeverity,
    pub entities: Vec<Uuid>,
    pub relationships: Vec<Uuid>,
    pub confidence: u8,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    ClusterDetected,
    UnusualConnection,
    PatternFound,
    TemporalAnomaly,
    DuplicateEntity,
    HighValueAsset,
    PotentialPivot,
    IndirectConnection,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum InsightSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindingRequest {
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub max_depth: Option<i32>,
    pub min_confidence: Option<u8>,
    pub include_entity_types: Option<Vec<String>>,
    pub exclude_entity_types: Option<Vec<String>>,
    pub include_relationship_types: Option<Vec<String>>,
    pub exclude_relationship_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityPath {
    pub path_length: i32,
    pub nodes: Vec<GraphNode>,
    pub relationships: Vec<GraphRelationship>,
    pub total_confidence: f32,
    pub path_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathFindingResult {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub source_entity_id: Uuid,
    pub target_entity_id: Uuid,
    pub paths: Vec<EntityPath>,
    pub insights: Vec<CorrelationInsight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchRequest {
    pub pattern_type: String,
    pub parameters: serde_json::Value,
    pub min_confidence: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatchResult {
    pub pattern_type: String,
    pub matches: Vec<PatternMatch>,
    pub match_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub nodes: Vec<EntityNode>,
    pub relationships: Vec<Relationship>,
    pub confidence: u8,
    pub match_properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityEnrichmentRequest {
    pub entity_id: Uuid,
    pub enrichment_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: Uuid,
    pub job_type: AnalysisJobType,
    pub entity_ids: Vec<Uuid>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisJobType {
    Correlation,
    PathFinding,
    CommunityDetection,
    AnomalyDetection,
    EntityEnrichment,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityImportance {
    pub entity_id: Uuid,
    pub degree_centrality: f32,
    pub betweenness_centrality: f32,
    pub pagerank: f32,
    pub total_score: f32,
}

impl AnalysisJob {
    pub fn new(job_type: AnalysisJobType, parameters: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            job_type,
            entity_ids: Vec::new(),
            parameters: HashMap::new(),
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result_id: None,
            error: None,
        }
    }
}
