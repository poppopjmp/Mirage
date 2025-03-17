#[macro_use] extern crate rocket;

use rocket::{Build, Rocket};
use rocket::fairing::AdHoc;
use rocket::serde::json::{Json, Value, json};
use rocket::State;
use mirage_common::models::User;

mod config;
mod handlers;
mod models;
mod repositories;
mod services;

#[get("/health")]
fn health_check() -> Value {
    json!({ "status": "ok" })
}

#[launch]
async fn rocket() -> Rocket<Build> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load configuration
    let config = match config::load_config() {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            panic!("Failed to load configuration: {}", e);
        }
    };
    
    let db_pool = match repositories::create_db_pool(&config.database).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            panic!("Failed to connect to database: {}", e);
        }
    };
    
    let user_service = services::UserService::new(db_pool.clone());
    
    tracing::info!("Starting User Management Service on port {}", config.server.port);

    rocket::build()
        .manage(db_pool)
        .manage(user_service)
        .manage(config)
        .mount("/api/v1", routes![
            health_check,
        ])
        .mount("/api/v1/users", handlers::user_routes())
        .mount("/api/v1/teams", handlers::team_routes())
        .mount("/api/v1/roles", handlers::role_routes())
}
