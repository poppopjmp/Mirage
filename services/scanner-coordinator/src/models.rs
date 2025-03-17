use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanStatus {
    Created,
    Queued,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ScanStatus,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub priority: i32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub error_message: Option<String>,
    pub progress: Option<i32>,
    pub estimated_completion_time: Option<DateTime<Utc>>,
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
pub struct ScanTarget {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub target_type: String,
    pub value: String,
    pub status: ScanTargetStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
    pub result_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScanModuleStatus {
    Enabled,
    Disabled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanModule {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub module_id: Uuid,
    pub module_name: String,
    pub module_version: String,
    pub status: ScanModuleStatus,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: i32,
    pub depends_on: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScanRequest {
    pub name: String,
    pub description: Option<String>,
    pub targets: Vec<CreateTargetRequest>,
    pub modules: Vec<ModuleRequest>,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub schedule: Option<ScheduleConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTargetRequest {
    pub target_type: String,
    pub value: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRequest {
    pub module_id: Uuid,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub priority: Option<i32>,
    pub depends_on: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub start_time: Option<DateTime<Utc>>,
    pub frequency: Option<String>, // "once", "hourly", "daily", "weekly", etc.
    pub frequency_options: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ScanStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target_count: i32,
    pub completed_targets: i32,
    pub progress: Option<i32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanDetailResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ScanStatus,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub priority: i32,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub error_message: Option<String>,
    pub progress: Option<i32>,
    pub estimated_completion_time: Option<DateTime<Utc>>,
    pub targets: Vec<ScanTargetResponse>,
    pub modules: Vec<ScanModuleResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTargetResponse {
    pub id: Uuid,
    pub target_type: String,
    pub value: String,
    pub status: ScanTargetStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub result_count: Option<i32>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanModuleResponse {
    pub id: Uuid,
    pub module_id: Uuid,
    pub module_name: String,
    pub module_version: String,
    pub status: ScanModuleStatus,
    pub priority: i32,
    pub depends_on: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanQueryParams {
    pub status: Option<ScanStatus>,
    pub created_by: Option<Uuid>,
    pub tag: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub name_contains: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScanRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTargetRequest {
    pub targets: Vec<CreateTargetRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddModuleRequest {
    pub modules: Vec<ModuleRequest>,
}
