use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use tracing::info;

mod config;
mod models;
mod repository;
mod services;
mod handlers;
mod health;
mod error;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = match config::load_config() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to load configuration",
            ));
        }
    };
    
    // Initialize Redis repository
    let redis_client = match redis::Client::open(&config.redis.uri) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other, 
                "Failed to connect to Redis",
            ));
        }
    };
    
    // Initialize service repository
    let repo = repository::ServiceRepository::new(redis_client.clone(), config.redis.clone());
    
    // Initialize HTTP client for health checks
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("Failed to create HTTP client");
        
    // Initialize services
    let discovery_service = web::Data::new(services::DiscoveryService::new(
        repo.clone(),
        config.clone(),
    ));
    
    let health_service = web::Data::new(health::HealthService::new(
        repo,
        http_client,
        config.health_check.clone(),
    ));
    
    // Start health check background task
    let health_service_clone = health_service.clone();
    tokio::spawn(async move {
        health::run_health_checker(health_service_clone.get_ref().clone()).await;
    });

    info!("Starting Discovery Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(discovery_service.clone())
            .app_data(health_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::discovery_routes())
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
