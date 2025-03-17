use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataStorageConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VisualizationConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorrelationConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportConfig {
    pub output_dir: String, 
    pub template_dir: String,
    pub max_entities_per_report: usize,
    pub logo_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub data_storage: DataStorageConfig,
    pub visualization: VisualizationConfig,
    pub correlation: CorrelationConfig,
    pub report: ReportConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_REPORTING"))
        .build()?;
        
    config.try_deserialize()
}
