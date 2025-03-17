//! Common models used across services

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: Uuid,
    pub name: String,
    pub user_id: Uuid,
    pub status: ScanStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub targets: Vec<Uuid>,
    pub modules: Vec<Uuid>,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub module_id: Uuid,
    pub target_id: Uuid,
    pub finding_type: String,
    pub data: serde_json::Value,
    pub confidence: u8,
    pub timestamp: DateTime<Utc>,
}

// Common response models
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<T>,
    pub errors: Option<Vec<String>>,
}

// Common pagination models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u64, per_page: u64) -> Self {
        let total_pages = if per_page > 0 {
            (total + per_page - 1) / per_page
        } else {
            0
        };
        
        Self {
            items,
            total,
            page,
            per_page,
            total_pages,
        }
    }
}

// Module-related models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub parameters: HashMap<String, ParameterDefinition>,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ParameterType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Enum,
}

// User related models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: DateTime<Utc>,
}
