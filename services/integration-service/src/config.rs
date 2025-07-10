use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub uri: String,
    pub prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    pub encryption_key: String,
    pub jwt_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    pub enabled: bool,
    pub execution_interval_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceEndpointConfig {
    pub data_collection_url: String,
    pub data_storage_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub security: SecurityConfig,
    pub scheduler: SchedulerConfig,
    pub api: ApiConfig,
    pub services: ServiceEndpointConfig,
}

impl AppConfig {
    pub fn api_timeout(&self) -> Duration {
        Duration::from_secs(self.api.timeout_seconds)
    }

    pub fn scheduler_interval(&self) -> Duration {
        Duration::from_secs(self.scheduler.execution_interval_seconds)
    }

    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.api.retry_delay_ms)
    }
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

    let config = Config::builder()
        .add_source(File::with_name("config/integration-service/default"))
        .add_source(File::with_name(&format!("config/integration-service/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_INTEGRATION"))
        .build()?;

    config.try_deserialize()
}
