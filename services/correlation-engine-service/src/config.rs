use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphDatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataStorageConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    pub enable_background_correlation: bool,
    pub min_correlation_confidence: u8,
    pub max_correlation_depth: i32,
    pub max_entities_per_correlation: i32,
    pub enable_advanced_insights: bool,
    pub background_job_interval_seconds: u64,
    pub max_parallel_jobs: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub graph_database: GraphDatabaseConfig,
    pub data_storage: DataStorageConfig,
    pub engine: EngineConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_CORRELATION"))
        .build()?;

    config.try_deserialize()
}
