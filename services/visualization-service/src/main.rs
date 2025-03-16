use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Visualization {
    id: Uuid,
    name: String,
    graph_data: String,
    filters: Vec<String>,
    created_at: String,
}

type VisualizationDb = Mutex<Vec<Visualization>>;

#[post("/visualizations")]
async fn create_visualization(visualization: web::Json<Visualization>, db: web::Data<VisualizationDb>) -> impl Responder {
    let mut visualizations = db.lock().unwrap();
    let new_visualization = Visualization {
        id: Uuid::new_v4(),
        ..visualization.into_inner()
    };
    visualizations.push(new_visualization.clone());
    HttpResponse::Ok().json(new_visualization)
}

#[get("/visualizations/{id}")]
async fn get_visualization(id: web::Path<Uuid>, db: web::Data<VisualizationDb>) -> impl Responder {
    let visualizations = db.lock().unwrap();
    if let Some(visualization) = visualizations.iter().find(|&visualization| visualization.id == *id) {
        HttpResponse::Ok().json(visualization)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/visualizations/{id}")]
async fn update_visualization(id: web::Path<Uuid>, visualization: web::Json<Visualization>, db: web::Data<VisualizationDb>) -> impl Responder {
    let mut visualizations = db.lock().unwrap();
    if let Some(existing_visualization) = visualizations.iter_mut().find(|visualization| visualization.id == *id) {
        *existing_visualization = visualization.into_inner();
        HttpResponse::Ok().json(existing_visualization.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/visualizations/{id}")]
async fn delete_visualization(id: web::Path<Uuid>, db: web::Data<VisualizationDb>) -> impl Responder {
    let mut visualizations = db.lock().unwrap();
    if let Some(pos) = visualizations.iter().position(|visualization| visualization.id == *id) {
        visualizations.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/visualizations")]
async fn list_visualizations(db: web::Data<VisualizationDb>) -> impl Responder {
    let visualizations = db.lock().unwrap();
    HttpResponse::Ok().json(visualizations.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let visualization_db = web::Data::new(Mutex::new(Vec::<Visualization>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(visualization_db.clone())
            .service(create_visualization)
            .service(get_visualization)
            .service(update_visualization)
            .service(delete_visualization)
            .service(list_visualizations)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
