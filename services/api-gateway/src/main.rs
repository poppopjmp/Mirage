use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::{Logger, NormalizePath};
use actix_web_httpauth::middleware::HttpAuthentication;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServiceResponse {
    status: String,
    data: Option<serde_json::Value>,
}

async fn validate_token(auth: BearerAuth) -> Result<Claims, actix_web::Error> {
    let token = auth.token();
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = jsonwebtoken::Validation::default();
    let token_data = jsonwebtoken::decode::<Claims>(token, &jsonwebtoken::DecodingKey::from_secret(secret.as_ref()), &validation)
        .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid token"))?;
    Ok(token_data.claims)
}

async fn route_request(path: web::Path<String>, req_body: String, services: web::Data<Arc<Mutex<HashMap<String, String>>>>) -> impl Responder {
    let service_name = path.into_inner();
    let services = services.lock().await;
    if let Some(service_url) = services.get(&service_name) {
        let client = reqwest::Client::new();
        let res = client.post(service_url)
            .body(req_body)
            .send()
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish())?;
        let res_body = res.text().await.map_err(|_| HttpResponse::InternalServerError().finish())?;
        HttpResponse::Ok().body(res_body)
    } else {
        HttpResponse::NotFound().body("Service not found")
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let services: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
    {
        let mut services = services.lock().await;
        services.insert("auth".to_string(), "http://auth-service:8080".to_string());
        services.insert("user-management".to_string(), "http://user-management-service:8080".to_string());
        services.insert("scan-orchestration".to_string(), "http://scan-orchestration-service:8080".to_string());
        services.insert("module-registry".to_string(), "http://module-registry-service:8080".to_string());
        services.insert("data-collection".to_string(), "http://data-collection-service:8080".to_string());
        services.insert("data-storage".to_string(), "http://data-storage-service:8080".to_string());
        services.insert("correlation-engine".to_string(), "http://correlation-engine-service:8080".to_string());
        services.insert("visualization".to_string(), "http://visualization-service:8080".to_string());
        services.insert("reporting".to_string(), "http://reporting-service:8080".to_string());
        services.insert("notification".to_string(), "http://notification-service:8080".to_string());
        services.insert("integration".to_string(), "http://integration-service:8080".to_string());
        services.insert("configuration".to_string(), "http://configuration-service:8080".to_string());
        services.insert("discovery".to_string(), "http://discovery-service:8080".to_string());
    }

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(NormalizePath::default())
            .wrap(HttpAuthentication::bearer(validate_token))
            .app_data(web::Data::new(services.clone()))
            .service(web::resource("/").to(index))
            .service(web::resource("/{service_name}").route(web::post().to(route_request)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
