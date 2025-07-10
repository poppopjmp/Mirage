//! Health check utilities

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub checks: Vec<HealthCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: Option<u64>,
}

impl HealthStatus {
    pub fn new(service_name: &str, version: &str) -> Self {
        Self {
            status: "healthy".to_string(),
            service: service_name.to_string(),
            timestamp: Utc::now(),
            version: version.to_string(),
            checks: Vec::new(),
        }
    }

    pub fn add_check(
        &mut self,
        name: &str,
        status: &str,
        message: Option<String>,
        duration_ms: Option<u64>,
    ) {
        self.checks.push(HealthCheck {
            name: name.to_string(),
            status: status.to_string(),
            message,
            duration_ms,
        });

        // Update overall status based on checks
        if self.checks.iter().any(|check| check.status != "healthy") {
            self.status = "unhealthy".to_string();
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }
}
