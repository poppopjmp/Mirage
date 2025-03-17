use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceStatus {
    Up,
    Down,
    Starting,
    Stopping,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInstance {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub metadata: HashMap<String, String>,
    pub health_check_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

impl ServiceInstance {
    pub fn new(
        name: &str,
        address: &str,
        port: u16,
        metadata: HashMap<String, String>,
        health_check_url: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let id = format!("{}-{}-{}", name, address, port);
        
        Self {
            id,
            name: name.to_string(),
            address: address.to_string(),
            port,
            status: ServiceStatus::Starting,
            metadata,
            health_check_url,
            registered_at: now,
            last_heartbeat: now,
        }
    }
    
    pub fn get_url(&self) -> String {
        format!("http://{}:{}", self.address, self.port)
    }
    
    pub fn get_health_url(&self) -> Option<String> {
        self.health_check_url.clone().or_else(|| {
            Some(format!("{}/health", self.get_url()))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistrationRequest {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub metadata: Option<HashMap<String, String>>,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub status: ServiceStatus,
    pub metadata: HashMap<String, String>,
    pub health_check_url: Option<String>,
    pub registered_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

impl From<ServiceInstance> for ServiceResponse {
    fn from(instance: ServiceInstance) -> Self {
        Self {
            id: instance.id,
            name: instance.name,
            address: instance.address,
            port: instance.port,
            status: instance.status,
            metadata: instance.metadata,
            health_check_url: instance.health_check_url,
            registered_at: instance.registered_at,
            last_heartbeat: instance.last_heartbeat,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub id: String,
    pub name: String,
    pub status: ServiceStatus,
    pub timestamp: DateTime<Utc>,
    pub response_time_ms: Option<u64>,
    pub error: Option<String>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHeartbeatRequest {
    pub id: String,
    pub status: ServiceStatus,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceQuery {
    pub name: Option<String>,
    pub status: Option<ServiceStatus>,
    pub metadata_key: Option<String>,
    pub metadata_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRegistry {
    pub services: Vec<ServiceResponse>,
    pub count: usize,
    pub timestamp: DateTime<Utc>,
}
