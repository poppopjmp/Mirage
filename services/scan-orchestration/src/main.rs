use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use mirage_common::models::Scan;
use tracing::info;

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
    
    let db_pool = match repositories::create_db_pool(&config.database).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to database",
            ));
        }
    };

    let scan_service = services::ScanService::new(db_pool.clone());
    
    info!("Starting Scan Orchestration Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(scan_service.clone()))
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::scan_routes())
            )
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
