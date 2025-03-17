use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub url: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub output_dir: String,
    pub default_graph_width: u32,
    pub default_graph_height: u32,
    pub default_chart_width: u32,
    pub default_chart_height: u32,
    pub max_nodes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub correlation_service: ServiceConfig,
    pub data_storage: ServiceConfig,
    pub visualization: VisualizationConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8088,
                workers: None,
            },
            correlation_service: ServiceConfig {
                url: "http://correlation-engine-service:8087".to_string(),
                timeout_secs: Some(30),
            },
            data_storage: ServiceConfig {
                url: "http://data-storage-service:8086".to_string(),
                timeout_secs: Some(30),
            },
            visualization: VisualizationConfig {
                output_dir: "/tmp/mirage/visualizations".to_string(),
                default_graph_width: 1200,
                default_graph_height: 800,
                default_chart_width: 1000,
                default_chart_height: 600,
                max_nodes: 500,
            },
        }
    }
}

pub fn load_config(path: Option<PathBuf>) -> Result<AppConfig, anyhow::Error> {
    if let Some(path) = path {
        if path.exists() {
            let config_str = std::fs::read_to_string(path)?;
            let config: AppConfig = serde_json::from_str(&config_str)?;
            return Ok(config);
        }
    }
    
    // Load from environment variables if no file is specified or the file doesn't exist
    let config = AppConfig {
        server: ServerConfig {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8088),
            workers: std::env::var("SERVER_WORKERS").ok().and_then(|w| w.parse().ok()),
        },
        correlation_service: ServiceConfig {
            url: std::env::var("CORRELATION_ENGINE_URL").unwrap_or_else(|_| "http://correlation-engine-service:8087".to_string()),
            timeout_secs: std::env::var("CORRELATION_ENGINE_TIMEOUT").ok().and_then(|t| t.parse().ok()),
        },
        data_storage: ServiceConfig {
            url: std::env::var("DATA_STORAGE_URL").unwrap_or_else(|_| "http://data-storage-service:8086".to_string()),
            timeout_secs: std::env::var("DATA_STORAGE_TIMEOUT").ok().and_then(|t| t.parse().ok()),
        },
        visualization: VisualizationConfig {
            output_dir: std::env::var("VISUALIZATION_OUTPUT_DIR").unwrap_or_else(|_| "/tmp/mirage/visualizations".to_string()),
            default_graph_width: std::env::var("VISUALIZATION_DEFAULT_GRAPH_WIDTH").ok().and_then(|w| w.parse().ok()).unwrap_or(1200),
            default_graph_height: std::env::var("VISUALIZATION_DEFAULT_GRAPH_HEIGHT").ok().and_then(|h| h.parse().ok()).unwrap_or(800),
            default_chart_width: std::env::var("VISUALIZATION_DEFAULT_CHART_WIDTH").ok().and_then(|w| w.parse().ok()).unwrap_or(1000),
            default_chart_height: std::env::var("VISUALIZATION_DEFAULT_CHART_HEIGHT").ok().and_then(|h| h.parse().ok()).unwrap_or(600),
            max_nodes: std::env::var("VISUALIZATION_MAX_NODES").ok().and_then(|n| n.parse().ok()).unwrap_or(500),
        },
    };
    
    Ok(config)
}
