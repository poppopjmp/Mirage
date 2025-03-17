use actix_cors::Cors;
use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use actix_files as fs;
use tracing::info;

mod config;
mod handlers;
mod models;
mod services;
mod renderers;

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

    // Initialize visualization service
    let viz_service = web::Data::new(services::VisualizationService::new(
        http_client,
        config.clone()
    ));

    info!("Starting Visualization Service on port {}", config.server.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(viz_service.clone())
            .app_data(web::Data::new(config.clone()))
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::visualization_routes())
            )
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(fs::Files::new("/", "./static/templates").index_file("index.html"))
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
