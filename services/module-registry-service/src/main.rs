use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Module {
    id: Uuid,
    name: String,
    version: String,
    config: String,
    dependencies: Vec<Uuid>,
}

type ModuleDb = Mutex<Vec<Module>>;

#[post("/modules")]
async fn register_module(module: web::Json<Module>, db: web::Data<ModuleDb>) -> impl Responder {
    let mut modules = db.lock().unwrap();
    let new_module = Module {
        id: Uuid::new_v4(),
        ..module.into_inner()
    };
    modules.push(new_module.clone());
    HttpResponse::Ok().json(new_module)
}

#[get("/modules/{id}")]
async fn get_module(id: web::Path<Uuid>, db: web::Data<ModuleDb>) -> impl Responder {
    let modules = db.lock().unwrap();
    if let Some(module) = modules.iter().find(|&module| module.id == *id) {
        HttpResponse::Ok().json(module)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/modules/{id}")]
async fn update_module(id: web::Path<Uuid>, module: web::Json<Module>, db: web::Data<ModuleDb>) -> impl Responder {
    let mut modules = db.lock().unwrap();
    if let Some(existing_module) = modules.iter_mut().find(|module| module.id == *id) {
        *existing_module = module.into_inner();
        HttpResponse::Ok().json(existing_module.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/modules/{id}")]
async fn delete_module(id: web::Path<Uuid>, db: web::Data<ModuleDb>) -> impl Responder {
    let mut modules = db.lock().unwrap();
    if let Some(pos) = modules.iter().position(|module| module.id == *id) {
        modules.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/modules")]
async fn list_modules(db: web::Data<ModuleDb>) -> impl Responder {
    let modules = db.lock().unwrap();
    HttpResponse::Ok().json(modules.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let module_db = web::Data::new(Mutex::new(Vec::<Module>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(module_db.clone())
            .service(register_module)
            .service(get_module)
            .service(update_module)
            .service(delete_module)
            .service(list_modules)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
