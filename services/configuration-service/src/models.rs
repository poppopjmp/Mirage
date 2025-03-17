use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConfigValueType {
    String,
    Integer,
    Float,
    Boolean,
    Json,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItem {
    pub id: Uuid,
    pub key: String,
    pub namespace: String,
    pub value: serde_json::Value,
    pub value_type: ConfigValueType,
    pub description: Option<String>,
    pub version: i32,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    pub id: Uuid,
    pub config_id: Uuid,
    pub value: serde_json::Value,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigNamespace {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConfigRequest {
    pub key: String,
    pub namespace: String,
    pub value: serde_json::Value,
    pub value_type: ConfigValueType,
    pub description: Option<String>,
    pub is_secret: Option<bool>,
    pub schema: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfigRequest {
    pub value: serde_json::Value,
    pub description: Option<String>,
    pub is_secret: Option<bool>,
    pub schema: Option<serde_json::Value>,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, String>>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub id: Uuid,
    pub key: String,
    pub namespace: String,
    pub value: serde_json::Value,
    pub value_type: ConfigValueType,
    pub description: Option<String>,
    pub version: i32,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersionResponse {
    pub id: Uuid,
    pub config_id: Uuid,
    pub value: serde_json::Value,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigNamespaceResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub config_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigQueryParams {
    pub namespace: Option<String>,
    pub tag: Option<String>,
    pub key_contains: Option<String>,
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
pub struct AuditLog {
    pub id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub change_summary: Option<String>,
    pub service: Option<String>,
}
