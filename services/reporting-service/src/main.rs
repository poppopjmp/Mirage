use actix_files as fs;
use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use tracing::info;

mod config;
mod formatters;
mod handlers;
mod models;
mod services;
mod templates;

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

    // Initialize HTTP client for services
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Initialize reporting service
    let report_service = web::Data::new(services::ReportService::new(http_client, config.clone()));

    info!("Starting Reporting Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(report_service.clone())
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::report_routes()),
            )
            .service(fs::Files::new("/reports", &config.report.output_dir).show_files_listing())
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
