use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    SocialMedia,
    SearchEngine,
    ThreatIntel,
    DarkWeb,
    PublicDatabase,
    SecurityTool,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationStatus {
    Active,
    Inactive,
    Failed,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    ApiKey,
    OAuth1,
    OAuth2,
    Basic,
    Bearer,
    Certificate,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleType {
    None,
    Once,
    Interval,
    Cron,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub integration_type: IntegrationType,
    pub provider_id: String,
    pub status: IntegrationStatus,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
    pub schedule_type: ScheduleType,
    pub schedule_config: Option<serde_json::Value>,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub auth_type: AuthType,
    pub name: String,
    pub encrypted_data: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_count: Option<i32>,
    pub error_message: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub target: Option<String>,
    pub execution_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIntegrationRequest {
    pub name: String,
    pub description: Option<String>,
    pub integration_type: IntegrationType,
    pub provider_id: String,
    pub config: serde_json::Value,
    pub tags: Option<Vec<String>>,
    pub schedule_type: ScheduleType,
    pub schedule_config: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntegrationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<IntegrationStatus>,
    pub config: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub schedule_type: Option<ScheduleType>,
    pub schedule_config: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub integration_type: IntegrationType,
    pub provider_id: String,
    pub status: IntegrationStatus,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub schedule_type: ScheduleType,
    pub schedule_config: Option<serde_json::Value>,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
    pub has_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRequest {
    pub auth_type: AuthType,
    pub name: String,
    pub data: serde_json::Value,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialResponse {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub auth_type: AuthType,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub parameters: Option<serde_json::Value>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResponse {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result_count: Option<i32>,
    pub error_message: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub target: Option<String>,
    pub execution_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationQueryParams {
    pub integration_type: Option<IntegrationType>,
    pub provider_id: Option<String>,
    pub status: Option<IntegrationStatus>,
    pub tag: Option<String>,
    pub name_contains: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub pages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAuth {
    pub api_key: String,
    pub header_name: String,
    pub additional_headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearerAuth {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Auth {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_at: Option<i64>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth1Auth {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub token: String,
    pub token_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub auth_types: Vec<AuthType>,
    pub supported_targets: Vec<String>,
    pub config_schema: serde_json::Value,
    pub metadata: HashMap<String, String>,
}
