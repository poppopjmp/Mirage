use actix_web::{
    web, App, HttpServer, middleware::Logger,
    HttpResponse, Responder,
};
use tracing::info;

mod config;
mod models;
mod repository;
mod services;
mod handlers;
mod health;
mod error;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}

use axum::{
    routing::{get, post},
    Router,
};

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/", get(|| async { "Hello, Discovery Service!" }))
        .route("/services", get(list_services))
        .route("/services", post(register_service));

    // run it
    let addr = "0.0.0.0:3000".parse().unwrap();
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_services() -> &'static str {
    "List of registered services"
}

async fn register_service() -> &'static str {
    "Service registered successfully"
}
