use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use tracing::info;

mod analysis;
mod config;
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

    // Set up database connections
    let graph_db = match repositories::create_graph_db(&config.graph_database).await {
        Ok(db) => db,
        Err(e) => {
            tracing::error!("Failed to connect to graph database: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to graph database",
            ));
        }
    };

    // Set up HTTP client for external services
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Initialize correlation service
    let correlation_service = web::Data::new(services::CorrelationService::new(
        graph_db.clone(),
        http_client.clone(),
        config.clone(),
    ));

    // Start background correlation tasks if enabled
    if config.engine.enable_background_correlation {
        let worker_service = correlation_service.clone();
        tokio::spawn(async move {
            services::start_background_correlation(worker_service).await;
        });
    }

    info!("Starting Correlation Engine on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(correlation_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::correlation_routes()),
            )
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
