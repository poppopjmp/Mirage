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
pub struct MongoDBConfig {
    pub uri: String,
    pub db_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ElasticsearchConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub index_prefix: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataRetentionConfig {
    pub enabled: bool,
    pub retention_days: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub mongodb: MongoDBConfig,
    pub elasticsearch: ElasticsearchConfig,
    pub data_retention: DataRetentionConfig,
    pub entity_types_whitelist: Option<Vec<String>>,
    pub relationship_types_whitelist: Option<Vec<String>>,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_DATA_STORAGE"))
        .build()?;

    config.try_deserialize()
}
