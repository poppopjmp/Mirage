use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use tracing::info;

mod config;
mod handlers;
mod models;
mod repositories;
mod services;
mod workers;
mod execution;
mod module;
mod queue;

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
    
    // Initialize MongoDB connection
    let mongo_client = match mongodb::Client::with_uri_str(&config.mongodb.uri).await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to connect to MongoDB: {}", e);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to connect to MongoDB",
            ));
        }
    };
    
    let mongo_db = mongo_client.database(&config.mongodb.database);
    
    // Initialize Redis connection
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

    // Initialize repositories
    let task_repository = repositories::TaskRepository::new(mongo_db.clone());
    let result_repository = repositories::ResultRepository::new(mongo_db.clone());
    
    // Initialize task queue
    let task_queue = queue::TaskQueue::new(redis_client.clone(), config.redis.queue_prefix.clone());
    
    // Initialize HTTP client for external services
    let http_client = reqwest::Client::new();
    
    // Initialize services
    let collection_service = web::Data::new(services::CollectionService::new(
        task_repository.clone(),
        result_repository.clone(),
        task_queue.clone(),
        http_client.clone(),
        config.clone(),
    ));
    
    // Start worker pool
    let worker_config = config.worker.clone();
    let worker_task_repo = task_repository.clone();
    let worker_result_repo = result_repository.clone(); 
    let worker_task_queue = task_queue.clone();
    let worker_http_client = http_client.clone();
    let worker_app_config = config.clone();
    
    tokio::spawn(async move {
        workers::start_worker_pool(
            worker_task_repo,
            worker_result_repo,
            worker_task_queue,
            worker_http_client,
            worker_app_config,
            worker_config.min_workers,
            worker_config.max_workers,
            worker_config.queue_poll_interval_ms,
        ).await;
    });

    info!("Starting Data Collection Service on port {}", config.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(collection_service.clone())
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(handlers::collection_routes())
            )
    })
    .bind(format!("0.0.0.0:{}", config.server.port))?
    .run()
    .await
}
