use actix_web::{web, HttpResponse, Responder, Error, post, get};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::CorrelationService;
use crate::models::{CorrelationRequest, BatchCorrelationRequest, PathFindingRequest};

pub fn correlation_routes() -> actix_web::Scope {
    web::scope("/correlation")
        .service(correlate_entity)
        .service(batch_correlate)
        .service(find_path)
        .service(get_job)
        .service(get_correlation_result)
}

#[post("/correlate")]
async fn correlate_entity(
    request: web::Json<CorrelationRequest>,
    correlation_service: web::Data<CorrelationService>,
) -> Result<HttpResponse, Error> {
    let result = correlation_service.generate_correlation(request.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to generate correlation: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[post("/batch")]
async fn batch_correlate(
    request: web::Json<BatchCorrelationRequest>,
    correlation_service: web::Data<CorrelationService>,
) -> Result<HttpResponse, Error> {
    let job = correlation_service.generate_batch_correlation(request.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to start batch correlation: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
    
    Ok(HttpResponse::Accepted().json(job))
}

#[post("/path")]
async fn find_path(
    request: web::Json<PathFindingRequest>,
    correlation_service: web::Data<CorrelationService>,
) -> Result<HttpResponse, Error> {
    let result = correlation_service.find_path(request.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to find path: {}", e);
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[get("/jobs/{id}")]
async fn get_job(
    id: web::Path<String>,
    correlation_service: web::Data<CorrelationService>,
) -> Result<HttpResponse, Error> {
    let job_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid job ID format")
    })?;
    
    let status = correlation_service.get_job_status(&job_id).await
        .map_err(|e| {
            tracing::error!("Failed to get job status: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
        
    match status {
        Some(job_status) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "id": job_id,
            "status": job_status,
        }))),
        None => Err(actix_web::error::ErrorNotFound(
            CommonError::NotFound(format!("Job with ID {} not found", job_id))
        )),
    }
}

#[get("/results/{id}")]
async fn get_correlation_result(
    id: web::Path<String>,
    _correlation_service: web::Data<CorrelationService>,
) -> Result<HttpResponse, Error> {
    let result_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid result ID format")
    })?;
    
    // In a real implementation, this would fetch the result from the database
    Err(actix_web::error::ErrorNotFound(format!("Result with ID {} not found", result_id)))
}
