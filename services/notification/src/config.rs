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
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebhookConfig {
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SlackConfig {
    pub webhook_url: String,
    pub default_channel: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateConfig {
    pub dir: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub poll_interval_seconds: u64,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub webhook: WebhookConfig,
    pub slack: SlackConfig,
    pub templates: TemplateConfig,
    pub worker: WorkerConfig,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());
    
    let config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_NOTIFICATION"))
        .build()?;
        
    config.try_deserialize()
}
