use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Configuration {
    id: Uuid,
    key: String,
    value: String,
    environment: String,
}

type ConfigDb = Mutex<Vec<Configuration>>;

#[post("/configurations")]
async fn create_configuration(config: web::Json<Configuration>, db: web::Data<ConfigDb>) -> impl Responder {
    let mut configs = db.lock().unwrap();
    let new_config = Configuration {
        id: Uuid::new_v4(),
        ..config.into_inner()
    };
    configs.push(new_config.clone());
    HttpResponse::Ok().json(new_config)
}

#[get("/configurations/{id}")]
async fn get_configuration(id: web::Path<Uuid>, db: web::Data<ConfigDb>) -> impl Responder {
    let configs = db.lock().unwrap();
    if let Some(config) = configs.iter().find(|&config| config.id == *id) {
        HttpResponse::Ok().json(config)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/configurations/{id}")]
async fn update_configuration(id: web::Path<Uuid>, config: web::Json<Configuration>, db: web::Data<ConfigDb>) -> impl Responder {
    let mut configs = db.lock().unwrap();
    if let Some(existing_config) = configs.iter_mut().find(|config| config.id == *id) {
        *existing_config = config.into_inner();
        HttpResponse::Ok().json(existing_config.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/configurations/{id}")]
async fn delete_configuration(id: web::Path<Uuid>, db: web::Data<ConfigDb>) -> impl Responder {
    let mut configs = db.lock().unwrap();
    if let Some(pos) = configs.iter().position(|config| config.id == *id) {
        configs.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/configurations")]
async fn list_configurations(db: web::Data<ConfigDb>) -> impl Responder {
    let configs = db.lock().unwrap();
    HttpResponse::Ok().json(configs.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config_db = web::Data::new(Mutex::new(Vec::<Configuration>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(config_db.clone())
            .service(create_configuration)
            .service(get_configuration)
            .service(update_configuration)
            .service(delete_configuration)
            .service(list_configurations)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
