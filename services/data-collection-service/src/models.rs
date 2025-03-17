use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteModuleRequest {
    pub module_id: Uuid,
    pub target: String,
    pub scan_id: Option<Uuid>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionResult {
    pub id: Uuid,
    pub module_id: Uuid,
    pub scan_id: Option<Uuid>,
    pub target: String,
    pub status: CollectionStatus,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CollectionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RateLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub configuration_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    pub domain: String,
    pub limit_per_minute: u32,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingData {
    pub scan_id: Option<Uuid>,
    pub module_id: Uuid,
    pub target_id: Uuid,
    pub finding_type: String,
    pub data: serde_json::Value,
    pub confidence: u8,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionJob {
    pub id: Uuid,
    pub module_id: Uuid,
    pub scan_id: Option<Uuid>,
    pub target: String,
    pub configuration: Option<serde_json::Value>,
    pub status: CollectionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl CollectionJob {
    pub fn new(
        module_id: Uuid,
        target: String,
        scan_id: Option<Uuid>,
        configuration: Option<serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            module_id,
            scan_id,
            target,
            configuration,
            status: CollectionStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionTask {
    pub id: Uuid,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target: CollectionTarget,
    pub module_id: Uuid,
    pub module_name: String,
    pub module_version: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub scan_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub error_message: Option<String>,
    pub result_summary: Option<ResultSummary>,
    pub max_duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    SingleTarget,
    BatchTarget,
    EnrichEntity,
    MonitorTarget,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionTarget {
    pub id: Uuid,
    pub target_type: String,
    pub value: String,
    pub metadata: HashMap<String, String>,
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    pub entities_created: u32,
    pub relationships_created: u32,
    pub entities_updated: u32,
    pub data_size_bytes: u64,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub target: CollectionTargetRequest,
    pub module_id: Uuid,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub priority: Option<i32>,
    pub scan_id: Option<Uuid>,
    pub task_type: Option<TaskType>,
    pub max_duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionTargetRequest {
    pub target_type: String,
    pub value: String,
    pub metadata: Option<HashMap<String, String>>,
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target: CollectionTarget,
    pub module_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatusRequest {
    pub task_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskRequest {
    pub targets: Vec<CollectionTargetRequest>,
    pub module_id: Uuid,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub priority: Option<i32>,
    pub scan_id: Option<Uuid>,
    pub max_duration_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub entities: Vec<Entity>,
    pub relationships: Vec<Relationship>,
    pub raw_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Option<Uuid>,
    pub entity_type: String,
    pub value: String,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub confidence: u8,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Option<Uuid>,
    pub source_id: Option<Uuid>,
    pub target_id: Option<Uuid>,
    pub source_value: String,
    pub target_value: String,
    pub relationship_type: String,
    pub data: HashMap<String, serde_json::Value>,
    pub confidence: u8,
    pub source: String,
}
