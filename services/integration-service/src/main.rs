use actix_web::{middleware::Logger, web, App, HttpServer};
use tracing::info;

mod config;
mod crypto;
mod error;
mod handlers;
mod models;
mod providers;
mod repositories;
mod scheduler;
mod services;

async fn health_check() -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "Hello, Integration Service!" }))
        .route("/integrations", get(list_integrations))
        .route("/integrations", post(create_integration));

    // run it
    let addr = "0.0.0.0:3001".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_integrations() -> &'static str {
    "List of available integrations"
}

async fn create_integration() -> &'static str {
    "Integration created successfully"
}
