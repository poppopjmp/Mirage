use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{http, middleware, web, App, HttpResponse, HttpServer, Responder};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

mod auth;
mod config;
mod handlers;
mod models;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    role: Option<String>,
    perms: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResponse {
    status: String,
    data: Option<serde_json::Value>,
}

#[derive(Clone)]
struct AppState {
    service_endpoints: HashMap<String, String>,
    auth_cache: Arc<Mutex<HashMap<String, Claims>>>,
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
        .json(serde_json::json!({ "status": "ok", "version": env!("CARGO_PKG_VERSION") }))
}

async fn validate_token(
    req: actix_web::dev::ServiceRequest,
    auth: BearerAuth,
) -> Result<actix_web::dev::ServiceRequest, actix_web::Error> {
    let token = auth.token();
    let app_state = req.app_data::<web::Data<AppState>>().unwrap();

    // Check cache first
    {
        let cache = app_state.auth_cache.lock().await;
        if let Some(claims) = cache.get(token) {
            // Check if token is expired
            let now = chrono::Utc::now().timestamp() as usize;
            if claims.exp > now {
                // Token valid, proceed
                return Ok(req);
            }
        }
    }

    // Not in cache or expired, validate with auth service
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = jsonwebtoken::Validation::default();

    match jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
        &validation,
    ) {
        Ok(token_data) => {
            // Add to cache
            {
                let mut cache = app_state.auth_cache.lock().await;
                cache.insert(token.to_string(), token_data.claims.clone());
            }
            Ok(req)
        }
        Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Load service configuration
    let mut service_endpoints = HashMap::new();
    service_endpoints.insert("auth".to_string(), "http://auth-service:8081".to_string());
    service_endpoints.insert(
        "user-management".to_string(),
        "http://user-management-service:8082".to_string(),
    );
    service_endpoints.insert(
        "scan-orchestration".to_string(),
        "http://scan-orchestration-service:8083".to_string(),
    );
    service_endpoints.insert(
        "module-registry".to_string(),
        "http://module-registry-service:8084".to_string(),
    );
    service_endpoints.insert(
        "data-collection".to_string(),
        "http://data-collection-service:8085".to_string(),
    );
    service_endpoints.insert(
        "data-storage".to_string(),
        "http://data-storage-service:8086".to_string(),
    );
    service_endpoints.insert(
        "correlation-engine".to_string(),
        "http://correlation-engine-service:8087".to_string(),
    );
    service_endpoints.insert(
        "visualization".to_string(),
        "http://visualization-service:8088".to_string(),
    );
    service_endpoints.insert(
        "reporting".to_string(),
        "http://reporting-service:8089".to_string(),
    );
    service_endpoints.insert(
        "notification".to_string(),
        "http://notification-service:8090".to_string(),
    );
    service_endpoints.insert(
        "integration".to_string(),
        "http://integration-service:8091".to_string(),
    );
    service_endpoints.insert(
        "configuration".to_string(),
        "http://configuration-service:8092".to_string(),
    );
    service_endpoints.insert(
        "discovery".to_string(),
        "http://discovery-service:8093".to_string(),
    );

    let app_state = web::Data::new(AppState {
        service_endpoints,
        auth_cache: Arc::new(Mutex::new(HashMap::new())),
    });

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validate_token);
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .wrap(middleware::Compress::default())
            // Public routes
            .service(
                web::scope("/api/v1")
                    .route("/health", web::get().to(health_check))
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/refresh", web::post().to(handlers::auth::refresh_token)),
                    ),
            )
            // Protected routes
            .service(
                web::scope("/api/v1")
                    .wrap(auth)
                    .service(
                        web::scope("/users")
                            .route("", web::get().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::get().to(handlers::proxy::proxy_request))
                            .route("", web::post().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::put().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::delete().to(handlers::proxy::proxy_request)),
                    )
                    .service(
                        web::scope("/scans")
                            .route("", web::get().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::get().to(handlers::proxy::proxy_request))
                            .route("", web::post().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::put().to(handlers::proxy::proxy_request))
                            .route("/{id}", web::delete().to(handlers::proxy::proxy_request)),
                    ), // Add more service routes as needed
            )
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
