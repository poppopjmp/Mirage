use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Service {
    id: Uuid,
    name: String,
    url: String,
    status: String,
}

type ServiceDb = Mutex<Vec<Service>>;

#[post("/services")]
async fn register_service(service: web::Json<Service>, db: web::Data<ServiceDb>) -> impl Responder {
    let mut services = db.lock().unwrap();
    let new_service = Service {
        id: Uuid::new_v4(),
        status: "UP".to_string(),
        ..service.into_inner()
    };
    services.push(new_service.clone());
    HttpResponse::Ok().json(new_service)
}

#[get("/services/{id}")]
async fn get_service(id: web::Path<Uuid>, db: web::Data<ServiceDb>) -> impl Responder {
    let services = db.lock().unwrap();
    if let Some(service) = services.iter().find(|&service| service.id == *id) {
        HttpResponse::Ok().json(service)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/services/{id}")]
async fn update_service(id: web::Path<Uuid>, service: web::Json<Service>, db: web::Data<ServiceDb>) -> impl Responder {
    let mut services = db.lock().unwrap();
    if let Some(existing_service) = services.iter_mut().find(|service| service.id == *id) {
        *existing_service = service.into_inner();
        HttpResponse::Ok().json(existing_service.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/services/{id}")]
async fn delete_service(id: web::Path<Uuid>, db: web::Data<ServiceDb>) -> impl Responder {
    let mut services = db.lock().unwrap();
    if let Some(pos) = services.iter().position(|service| service.id == *id) {
        services.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/services")]
async fn list_services(db: web::Data<ServiceDb>) -> impl Responder {
    let services = db.lock().unwrap();
    HttpResponse::Ok().json(services.clone())
}

#[get("/services/health")]
async fn health_check(db: web::Data<ServiceDb>) -> impl Responder {
    let services = db.lock().unwrap();
    let health_status: Vec<_> = services.iter().map(|service| {
        let status = if service.status == "UP" { "Healthy" } else { "Unhealthy" };
        (service.name.clone(), status)
    }).collect();
    HttpResponse::Ok().json(health_status)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let service_db = web::Data::new(Mutex::new(Vec::<Service>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(service_db.clone())
            .service(register_service)
            .service(get_service)
            .service(update_service)
            .service(delete_service)
            .service(list_services)
            .service(health_check)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
