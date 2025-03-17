use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MongoConfig {
    pub uri: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub uri: String,
    pub queue_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModuleRegistryConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataStorageConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub min_workers: usize,
    pub max_workers: usize,
    pub queue_poll_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub mongodb: MongoConfig,
    pub redis: RedisConfig,
    pub module_registry: ModuleRegistryConfig,
    pub data_storage: DataStorageConfig,
    pub worker: WorkerConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_DATA_COLLECTION"))
        .build()?;
        
    config.try_deserialize()
}
