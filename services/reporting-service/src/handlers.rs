use actix_files::NamedFile;
use actix_web::{get, post, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::ReportRequest;
use crate::services::ReportService;

pub fn report_routes() -> actix_web::Scope {
    web::scope("/reports")
        .service(generate_report)
        .service(get_report)
        .service(list_templates)
}

#[post("/generate")]
async fn generate_report(
    request: web::Json<ReportRequest>,
    report_service: web::Data<ReportService>,
) -> Result<HttpResponse, Error> {
    let result = report_service
        .generate_report(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to generate report: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::ExternalApi(_) => actix_web::error::ErrorBadGateway(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[get("/templates")]
async fn list_templates(report_service: web::Data<ReportService>) -> Result<HttpResponse, Error> {
    let templates = report_service.get_available_templates();
    Ok(HttpResponse::Ok().json(templates))
}

#[get("/{id}")]
async fn get_report(
    id: web::Path<String>,
    report_service: web::Data<ReportService>,
) -> Result<NamedFile, Error> {
    let report_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid report ID format"))?;

    let file_path = report_service
        .get_report_file(&report_id)
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to get report file: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    NamedFile::open(file_path).map_err(|e| {
        tracing::error!("Failed to open report file: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })
}
