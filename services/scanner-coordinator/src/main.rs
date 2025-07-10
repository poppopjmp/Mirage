use actix_web::{middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use tracing::info;

mod config;
mod error;
mod handlers;
mod integrations;
mod models;
mod repositories;
mod scheduler;
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

    // Initialize database connection
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

    // Initialize repositories
    let scan_repo = repositories::ScanRepository::new(db_pool.clone());
    let target_repo = repositories::ScanTargetRepository::new(db_pool.clone());
    let module_repo = repositories::ScanModuleRepository::new(db_pool.clone());

    // Initialize HTTP client for external services
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    // Initialize Redis client for task queues
    let redis_client = match redis::Client::open(config.redis.uri.clone()) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to Redis",
            ));
        }
    };

    // Initialize services
    let integration_service =
        integrations::IntegrationService::new(http_client.clone(), config.clone());

    let scheduler_service = scheduler::SchedulerService::new(
        redis_client.clone(),
        scan_repo.clone(),
        target_repo.clone(),
        integration_service.clone(),
        config.clone(),
    );

    let scanner_service = web::Data::new(services::ScannerService::new(
        scan_repo,
        target_repo,
        module_repo,
        scheduler_service.clone(),
        integration_service,
        config.clone(),
    ));

    // Start scheduler background task
    let scheduler_config = config.clone();
    tokio::spawn(async move {
        scheduler::run_scheduler(scheduler_service, scheduler_config).await;
    });

    info!(
        "Starting Scanner Coordinator on port {}",
        config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .app_data(scanner_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::scanner_routes()),
            )
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
