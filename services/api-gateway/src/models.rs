//! Data models for API Gateway service

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRoute {
    pub path: String,
    pub service_url: String,
    pub method: String,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyRequest {
    pub path: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyResponse {
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub services: std::collections::HashMap<String, ServiceHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<u64>,
    pub last_check: chrono::DateTime<chrono::Utc>,
}
