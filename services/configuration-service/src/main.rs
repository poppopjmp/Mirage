use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use tracing::info;

mod config;
mod models;
mod repositories;
mod services;
mod handlers;
mod validation;
mod audit;

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
    
    // Initialize Redis connection for caching
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
    
    // Initialize repositories
    let config_repo = repositories::ConfigRepository::new(db_pool.clone());
    let audit_repo = repositories::AuditRepository::new(db_pool.clone());
    
    // Initialize services
    let audit_service = web::Data::new(audit::AuditService::new(audit_repo));
    let config_service = web::Data::new(services::ConfigService::new(
        config_repo,
        redis_client,
        audit_service.get_ref().clone(),
        config.clone(),
    ));

    info!("Starting Configuration Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(config_service.clone())
            .app_data(audit_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::config_routes())
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
