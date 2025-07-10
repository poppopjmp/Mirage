use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use tracing::info;

mod channels;
mod config;
mod handlers;
mod models;
mod repositories;
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

    // Set up database connection pool
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

    // Initialize notification service
    let notification_service = web::Data::new(services::NotificationService::new(
        db_pool.clone(),
        config.clone(),
    ));

    // Initialize background worker for processing notification queue
    let worker_pool = db_pool.clone();
    let worker_config = config.clone();
    tokio::spawn(async move {
        services::start_notification_worker(worker_pool, worker_config).await;
    });

    info!(
        "Starting Notification Service on port {}",
        config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(notification_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::notification_routes()),
            )
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
