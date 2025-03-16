use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportTemplate {
    id: Uuid,
    name: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Report {
    id: Uuid,
    template_id: Uuid,
    custom_data: String,
    generated_at: String,
}

type ReportTemplateDb = Mutex<Vec<ReportTemplate>>;
type ReportDb = Mutex<Vec<Report>>;

#[post("/templates")]
async fn create_template(template: web::Json<ReportTemplate>, db: web::Data<ReportTemplateDb>) -> impl Responder {
    let mut templates = db.lock().unwrap();
    let new_template = ReportTemplate {
        id: Uuid::new_v4(),
        ..template.into_inner()
    };
    templates.push(new_template.clone());
    HttpResponse::Ok().json(new_template)
}

#[get("/templates/{id}")]
async fn get_template(id: web::Path<Uuid>, db: web::Data<ReportTemplateDb>) -> impl Responder {
    let templates = db.lock().unwrap();
    if let Some(template) = templates.iter().find(|&template| template.id == *id) {
        HttpResponse::Ok().json(template)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/templates/{id}")]
async fn update_template(id: web::Path<Uuid>, template: web::Json<ReportTemplate>, db: web::Data<ReportTemplateDb>) -> impl Responder {
    let mut templates = db.lock().unwrap();
    if let Some(existing_template) = templates.iter_mut().find(|template| template.id == *id) {
        existing_template.name = template.name.clone();
        existing_template.content = template.content.clone();
        HttpResponse::Ok().json(existing_template.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/templates/{id}")]
async fn delete_template(id: web::Path<Uuid>, db: web::Data<ReportTemplateDb>) -> impl Responder {
    let mut templates = db.lock().unwrap();
    if let Some(pos) = templates.iter().position(|template| template.id == *id) {
        templates.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/reports")]
async fn generate_report(report: web::Json<Report>, template_db: web::Data<ReportTemplateDb>, report_db: web::Data<ReportDb>) -> impl Responder {
    let templates = template_db.lock().unwrap();
    if let Some(template) = templates.iter().find(|&template| template.id == report.template_id) {
        let new_report = Report {
            id: Uuid::new_v4(),
            template_id: report.template_id,
            custom_data: report.custom_data.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        };
        let mut reports = report_db.lock().unwrap();
        reports.push(new_report.clone());
        HttpResponse::Ok().json(new_report)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/reports/{id}")]
async fn get_report(id: web::Path<Uuid>, db: web::Data<ReportDb>) -> impl Responder {
    let reports = db.lock().unwrap();
    if let Some(report) = reports.iter().find(|&report| report.id == *id) {
        HttpResponse::Ok().json(report)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/reports/{id}")]
async fn delete_report(id: web::Path<Uuid>, db: web::Data<ReportDb>) -> impl Responder {
    let mut reports = db.lock().unwrap();
    if let Some(pos) = reports.iter().position(|report| report.id == *id) {
        reports.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[get("/reports")]
async fn list_reports(db: web::Data<ReportDb>) -> impl Responder {
    let reports = db.lock().unwrap();
    HttpResponse::Ok().json(reports.clone())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let template_db = web::Data::new(Mutex::new(Vec::<ReportTemplate>::new()));
    let report_db = web::Data::new(Mutex::new(Vec::<Report>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(template_db.clone())
            .app_data(report_db.clone())
            .service(create_template)
            .service(get_template)
            .service(update_template)
            .service(delete_template)
            .service(generate_report)
            .service(get_report)
            .service(delete_report)
            .service(list_reports)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
