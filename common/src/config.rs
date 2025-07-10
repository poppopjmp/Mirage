//! Configuration management for Mirage services

use serde::{Deserialize, Serialize};
use std::env;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt_secret: String,
    pub log_level: String,
}

impl ServiceConfig {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| Error::Config("DATABASE_URL not set".to_string()))?;
        
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8000".to_string())
            .parse()
            .map_err(|_| Error::Config("Invalid PORT value".to_string()))?;
            
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| Error::Config("JWT_SECRET not set".to_string()))?;
            
        let log_level = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
        
        let max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .map_err(|_| Error::Config("Invalid DB_MAX_CONNECTIONS value".to_string()))?;

        Ok(ServiceConfig {
            server: ServerConfig { host, port },
            database: DatabaseConfig {
                url: database_url,
                max_connections,
            },
            jwt_secret,
            log_level,
        })
    }
}

pub fn load_config() -> Result<ServiceConfig> {
    dotenv::dotenv().ok();
    ServiceConfig::from_env()
}