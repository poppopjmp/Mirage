use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Integration {
    id: Uuid,
    name: String,
    description: String,
    config: serde_json::Value,
}

type IntegrationDb = Mutex<Vec<Integration>>;

#[post("/integrations")]
async fn create_integration(integration: web::Json<Integration>, db: web::Data<IntegrationDb>) -> impl Responder {
    let mut integrations = db.lock().unwrap();
    let new_integration = Integration {
        id: Uuid::new_v4(),
        ..integration.into_inner()
    };
    integrations.push(new_integration.clone());
    HttpResponse::Ok().json(new_integration)
}

#[get("/integrations/{id}")]
async fn get_integration(id: web::Path<Uuid>, db: web::Data<IntegrationDb>) -> impl Responder {
    let integrations = db.lock().unwrap();
    if let Some(integration) = integrations.iter().find(|&integration| integration.id == *id) {
        HttpResponse::Ok().json(integration)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/integrations/{id}")]
async fn update_integration(id: web::Path<Uuid>, integration: web::Json<Integration>, db: web::Data<IntegrationDb>) -> impl Responder {
    let mut integrations = db.lock().unwrap();
    if let Some(existing_integration) = integrations.iter_mut().find(|integration| integration.id == *id) {
        *existing_integration = integration.into_inner();
        HttpResponse::Ok().json(existing_integration.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/integrations/{id}")]
async fn delete_integration(id: web::Path<Uuid>, db: web::Data<IntegrationDb>) -> impl Responder {
    let mut integrations = db.lock().unwrap();
    if let Some(pos) = integrations.iter().position(|integration| integration.id == *id) {
        integrations.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/integrations")]
async fn list_integrations(db: web::Data<IntegrationDb>) -> impl Responder {
    let integrations = db.lock().unwrap();
    HttpResponse::Ok().json(integrations.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let integration_db = web::Data::new(Mutex::new(Vec::<Integration>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(integration_db.clone())
            .service(create_integration)
            .service(get_integration)
            .service(update_integration)
            .service(delete_integration)
            .service(list_integrations)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
