use actix_web::{web, App, HttpServer, Responder};
use log::info;
mod middleware;
mod models;
mod routes;
use actix_web::middleware::Logger;
use middleware::auth::Authentication;
use middleware::cors::cors_middleware;
use middleware::logging::RequestLogger;
async fn health_check() -> impl Responder {
    "OK"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Starting auth-service");

    HttpServer::new(move || {
        App::new()
            .wrap(cors_middleware())
            .wrap(Authentication {})
            .wrap(RequestLogger {})
            .wrap(Logger::default())
            .route("/health", web::get().to(health_check))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8001))?
    .run()
    .await
}
