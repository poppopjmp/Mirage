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
    pub cache_ttl_seconds: u64,
    pub prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub service_name: String,
    pub audit_enabled: bool,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_CONFIG"))
        .build()?;
        
    config.try_deserialize()
}
