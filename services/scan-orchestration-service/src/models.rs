use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Created,
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: ScanStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub targets: Vec<ScanTarget>,
    pub modules: Vec<ScanModule>,
    pub priority: i32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub error_message: Option<String>,
    pub max_duration_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTarget {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub target_type: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub status: ScanTargetStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanTargetStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanModule {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub module_id: Uuid,
    pub module_name: String,
    pub module_version: String,
    pub order: i32,
    pub parameters: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ScanModuleStatus,
    pub results_summary: Option<ScanModuleResultsSummary>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanModuleStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanModuleResultsSummary {
    pub entities_found: i32,
    pub relationships_found: i32,
    pub entities_by_type: HashMap<String, i32>,
    pub runtime_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScanRequest {
    pub name: String,
    pub description: Option<String>,
    pub targets: Vec<CreateScanTargetRequest>,
    pub modules: Vec<CreateScanModuleRequest>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub max_duration_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScanTargetRequest {
    pub target_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScanModuleRequest {
    pub module_id: Uuid,
    pub order: Option<i32>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub max_duration_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanQueryParams {
    pub status: Option<ScanStatus>,
    pub created_by: Option<Uuid>,
    pub tag: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanModuleResult {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub module_id: Uuid,
    pub target_id: Uuid,
    pub entity_id: Option<Uuid>,
    pub relationship_id: Option<Uuid>,
    pub result_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
