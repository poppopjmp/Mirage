use actix_web::middleware::Logger;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid; // P1372

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Scan {
    id: Uuid,
    name: String,
    target: String,
    status: String,
}

type ScanDb = Mutex<Vec<Scan>>;

#[post("/scans")]
async fn create_scan(scan: web::Json<Scan>, db: web::Data<ScanDb>) -> impl Responder {
    let mut scans = db.lock().unwrap();
    let new_scan = Scan {
        id: Uuid::new_v4(),
        status: "created".to_string(),
        ..scan.into_inner()
    };
    scans.push(new_scan.clone());
    info!("Created new scan: {:?}", new_scan); // P1372
    HttpResponse::Ok().json(new_scan)
}

#[get("/scans/{id}")]
async fn get_scan(id: web::Path<Uuid>, db: web::Data<ScanDb>) -> impl Responder {
    let scans = db.lock().unwrap();
    if let Some(scan) = scans.iter().find(|&scan| scan.id == *id) {
        info!("Fetched scan: {:?}", scan); // P1372
        HttpResponse::Ok().json(scan)
    } else {
        warn!("Scan not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[put("/scans/{id}")]
async fn update_scan(
    id: web::Path<Uuid>,
    scan: web::Json<Scan>,
    db: web::Data<ScanDb>,
) -> impl Responder {
    let mut scans = db.lock().unwrap();
    if let Some(existing_scan) = scans.iter_mut().find(|scan| scan.id == *id) {
        *existing_scan = scan.into_inner();
        info!("Updated scan: {:?}", existing_scan); // P1372
        HttpResponse::Ok().json(existing_scan.clone())
    } else {
        warn!("Scan not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[delete("/scans/{id}")]
async fn delete_scan(id: web::Path<Uuid>, db: web::Data<ScanDb>) -> impl Responder {
    let mut scans = db.lock().unwrap();
    if let Some(pos) = scans.iter().position(|scan| scan.id == *id) {
        scans.remove(pos);
        info!("Deleted scan: {:?}", id); // P1372
        HttpResponse::NoContent().finish()
    } else {
        warn!("Scan not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[get("/scans")]
async fn list_scans(db: web::Data<ScanDb>) -> impl Responder {
    let scans = db.lock().unwrap();
    info!("Listing all scans"); // P1372
    HttpResponse::Ok().json(scans.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Event {
    id: Uuid,
    scan_id: Uuid,
    event_type: String,
    data: String,
    timestamp: u64,
}

type EventDb = Mutex<Vec<Event>>;

#[post("/events")]
async fn create_event(event: web::Json<Event>, db: web::Data<EventDb>) -> impl Responder {
    let mut events = db.lock().unwrap();
    let new_event = Event {
        id: Uuid::new_v4(),
        ..event.into_inner()
    };
    events.push(new_event.clone());
    info!("Created new event: {:?}", new_event); // P1372
    HttpResponse::Ok().json(new_event)
}

#[get("/events/{id}")]
async fn get_event(id: web::Path<Uuid>, db: web::Data<EventDb>) -> impl Responder {
    let events = db.lock().unwrap();
    if let Some(event) = events.iter().find(|&event| event.id == *id) {
        info!("Fetched event: {:?}", event); // P1372
        HttpResponse::Ok().json(event)
    } else {
        warn!("Event not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[put("/events/{id}")]
async fn update_event(
    id: web::Path<Uuid>,
    event: web::Json<Event>,
    db: web::Data<EventDb>,
) -> impl Responder {
    let mut events = db.lock().unwrap();
    if let Some(existing_event) = events.iter_mut().find(|event| event.id == *id) {
        *existing_event = event.into_inner();
        info!("Updated event: {:?}", existing_event); // P1372
        HttpResponse::Ok().json(existing_event.clone())
    } else {
        warn!("Event not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[delete("/events/{id}")]
async fn delete_event(id: web::Path<Uuid>, db: web::Data<EventDb>) -> impl Responder {
    let mut events = db.lock().unwrap();
    if let Some(pos) = events.iter().position(|event| event.id == *id) {
        events.remove(pos);
        info!("Deleted event: {:?}", id); // P1372
        HttpResponse::NoContent().finish()
    } else {
        warn!("Event not found: {:?}", id); // P1372
        HttpResponse::NotFound().finish()
    }
}

#[get("/events")]
async fn list_events(db: web::Data<EventDb>) -> impl Responder {
    let events = db.lock().unwrap();
    info!("Listing all events"); // P1372
    HttpResponse::Ok().json(events.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let scan_db = web::Data::new(Mutex::new(Vec::<Scan>::new()));
    let event_db = web::Data::new(Mutex::new(Vec::<Event>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(scan_db.clone())
            .app_data(event_db.clone())
            .service(create_scan)
            .service(get_scan)
            .service(update_scan)
            .service(delete_scan)
            .service(list_scans)
            .service(create_event)
            .service(get_event)
            .service(update_event)
            .service(delete_event)
            .service(list_events)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
