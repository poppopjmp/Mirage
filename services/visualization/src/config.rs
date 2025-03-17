use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorrelationServiceConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataStorageConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisualizationConfig {
    pub output_dir: String,
    pub max_nodes: usize,
    pub default_graph_width: u32,
    pub default_graph_height: u32,
    pub default_chart_width: u32,
    pub default_chart_height: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub correlation_service: CorrelationServiceConfig,
    pub data_storage: DataStorageConfig,
    pub visualization: VisualizationConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_VISUALIZATION"))
        .build()?;
        
    config.try_deserialize()
}
