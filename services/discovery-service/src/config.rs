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
pub struct RedisConfig {
    pub uri: String,
    pub key_prefix: String,
    pub service_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub success_threshold: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub health_check: HealthCheckConfig,
}

impl AppConfig {
    pub fn service_ttl(&self) -> Duration {
        Duration::from_secs(self.redis.service_ttl_seconds)
    }

    pub fn health_check_interval(&self) -> Duration {
        Duration::from_secs(self.health_check.interval_seconds)
    }

    pub fn health_check_timeout(&self) -> Duration {
        Duration::from_secs(self.health_check.timeout_seconds)
    }
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    let env = env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

    let config = Config::builder()
        .add_source(File::with_name("config/discovery-service/default"))
        .add_source(File::with_name(&format!("config/discovery-service/{}", env)).required(false))
        .add_source(config::Environment::with_prefix("MIRAGE_DISCOVERY"))
        .build()?;

    config.try_deserialize()
}
