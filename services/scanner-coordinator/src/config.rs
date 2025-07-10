use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

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
    pub task_queue_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchedulerConfig {
    pub interval_seconds: u64,
    pub max_concurrent_scans: usize,
    pub max_targets_per_batch: usize,
    pub retry_delay_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub scheduler: SchedulerConfig,
    pub module_registry: ServiceConfig,
    pub scan_orchestration: ServiceConfig,
    pub data_collection: ServiceConfig,
    pub data_storage: ServiceConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_SCANNER"))
        .build()?;

    config.try_deserialize()
}
