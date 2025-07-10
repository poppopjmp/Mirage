use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use tracing::info;

mod error;
mod handlers;
mod models;
mod repositories;
mod services;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = match std::env::var("CONFIG_PATH") {
        Ok(path) => {
            let config_str = std::fs::read_to_string(path)?;
            serde_json::from_str(&config_str).expect("Failed to parse configuration")
        }
        Err(_) => {
            tracing::warn!("CONFIG_PATH not set, using default configuration");
            serde_json::json!({
                "server": {
                    "host": "0.0.0.0",
                    "port": 8088
                },
                "correlation_engine": {
                    "url": "http://correlation-engine-service:8087"
                }
            })
        }
    };

    // Create HTTP client for external communication
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Initialize visualization service
    let viz_service = web::Data::new(services::VisualizationService::new(
        http_client,
        config["correlation_engine"]["url"]
            .as_str()
            .unwrap_or("http://localhost:8087")
            .to_string(),
    ));

    let port = config["server"]["port"].as_u64().unwrap_or(8088);
    let host = config["server"]["host"].as_str().unwrap_or("0.0.0.0");

    info!("Starting Visualization Service on {}:{}", host, port);

    HttpServer::new(move || {
        App::new()
            .app_data(viz_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::visualization_routes()),
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}
