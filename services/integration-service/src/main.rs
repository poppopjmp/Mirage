use actix_web::{web, App, HttpServer, middleware::Logger};
use tracing::info;

mod config;
mod models;
mod repositories;
mod services;
mod handlers;
mod providers;
mod scheduler;
mod crypto;
mod error;

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging with tracing
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = match config::load_config() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load configuration: {}", e),
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
                format!("Failed to connect to database: {}", e),
            ));
        }
    };
    
    // Initialize Redis client
    let redis_client = match redis::Client::open(&config.redis.uri) {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to connect to Redis: {}", e),
            ));
        }
    };
    
    // Create HTTP client for external requests
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");
    
    // Initialize repositories
    let integration_repo = repositories::IntegrationRepository::new(db_pool.clone());
    let credential_repo = repositories::CredentialRepository::new(
        db_pool.clone(), 
        crypto::CryptoService::new(&config.security.encryption_key)
    );
    let execution_repo = repositories::ExecutionRepository::new(db_pool);
    
    // Initialize provider registry
    let provider_registry = providers::ProviderRegistry::new();
    
    // Initialize services
    let integration_service = web::Data::new(services::IntegrationService::new(
        integration_repo.clone(),
        credential_repo.clone(),
        provider_registry.clone(),
        http_client.clone(),
        config.clone(),
    ));
    
    let scheduler_service = web::Data::new(scheduler::SchedulerService::new(
        integration_repo,
        execution_repo,
        provider_registry,
        http_client,
        redis_client,
        config.clone(),
    ));
    
    // Start the background scheduler
    let scheduler = scheduler_service.clone();
    tokio::spawn(async move {
        scheduler::run_scheduler(scheduler.get_ref().clone()).await;
    });

    info!("Starting Integration Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(integration_service.clone())
            .app_data(scheduler_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::integration_routes())
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
