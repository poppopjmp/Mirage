use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DataRecord {
    id: Uuid,
    version: u32,
    data: String,
}

type DataDb = Mutex<Vec<DataRecord>>;

#[post("/data")]
async fn create_data(record: web::Json<DataRecord>, db: web::Data<DataDb>) -> impl Responder {
    let mut data = db.lock().unwrap();
    let new_record = DataRecord {
        id: Uuid::new_v4(),
        version: 1,
        ..record.into_inner()
    };
    data.push(new_record.clone());
    HttpResponse::Ok().json(new_record)
}

#[get("/data/{id}")]
async fn get_data(id: web::Path<Uuid>, db: web::Data<DataDb>) -> impl Responder {
    let data = db.lock().unwrap();
    if let Some(record) = data.iter().find(|&record| record.id == *id) {
        HttpResponse::Ok().json(record)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/data/{id}")]
async fn update_data(id: web::Path<Uuid>, record: web::Json<DataRecord>, db: web::Data<DataDb>) -> impl Responder {
    let mut data = db.lock().unwrap();
    if let Some(existing_record) = data.iter_mut().find(|record| record.id == *id) {
        existing_record.version += 1;
        existing_record.data = record.data.clone();
        HttpResponse::Ok().json(existing_record.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/data/{id}")]
async fn delete_data(id: web::Path<Uuid>, db: web::Data<DataDb>) -> impl Responder {
    let mut data = db.lock().unwrap();
    if let Some(pos) = data.iter().position(|record| record.id == *id) {
        data.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/data")]
async fn list_data(db: web::Data<DataDb>) -> impl Responder {
    let data = db.lock().unwrap();
    HttpResponse::Ok().json(data.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let data_db = web::Data::new(Mutex::new(Vec::<DataRecord>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(data_db.clone())
            .service(create_data)
            .service(get_data)
            .service(update_data)
            .service(delete_data)
            .service(list_data)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
