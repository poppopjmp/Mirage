//! Configuration for API Gateway service

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub auth_service_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Config {
            server_host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("API_GATEWAY_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()?,
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:8001".to_string()),
            jwt_secret: env::var("JWT_SECRET")?,
        })
    }
}
