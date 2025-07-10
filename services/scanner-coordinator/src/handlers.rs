use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use uuid::Uuid;

use crate::models::{
    AddModuleRequest, AddTargetRequest, CreateScanRequest, ScanQueryParams, ScanStatus,
    UpdateScanRequest,
};
use crate::services::ScannerService;

pub fn scanner_routes() -> actix_web::Scope {
    web::scope("/scanner")
        .service(create_scan)
        .service(get_scan)
        .service(list_scans)
        .service(update_scan)
        .service(start_scan)
        .service(cancel_scan)
        .service(add_targets)
        .service(add_modules)
}

#[post("/scans")]
async fn create_scan(
    request: web::Json<CreateScanRequest>,
    service: web::Data<ScannerService>,
    // In production, we'd extract user ID from auth headers/context
) -> Result<HttpResponse, Error> {
    // Mock user ID for demonstration
    let user_id = Some(Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap());

    let result = service
        .create_scan(request.into_inner(), user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create scan: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[get("/scans/{id}")]
async fn get_scan(
    id: web::Path<String>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    let scan = service.get_scan(scan_id).await.map_err(|e| match e {
        CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
        _ => {
            tracing::error!("Failed to get scan: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;

    Ok(HttpResponse::Ok().json(scan))
}

#[get("/scans")]
async fn list_scans(
    query: web::Query<ScanQueryParams>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    // Extract query parameters with defaults
    let status = query.status.clone();
    let created_by = query.created_by;
    let tag = query.tag.clone();
    let created_after = query.created_after;
    let created_before = query.created_before;
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);

    let (scans, total) = service
        .list_scans(
            status,
            created_by,
            tag,
            created_after,
            created_before,
            page,
            per_page,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to list scans: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    // Return response with pagination metadata
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": scans,
        "total": total,
        "page": page,
        "per_page": per_page,
        "pages": (total + per_page - 1) / per_page
    })))
}

#[put("/scans/{id}")]
async fn update_scan(
    id: web::Path<String>,
    request: web::Json<UpdateScanRequest>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    let scan = service
        .update_scan(scan_id, request.into_inner())
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
            _ => {
                tracing::error!("Failed to update scan: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(scan))
}

#[post("/scans/{id}/start")]
async fn start_scan(
    id: web::Path<String>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    let scan = service.start_scan(scan_id).await.map_err(|e| match e {
        CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
        CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
        _ => {
            tracing::error!("Failed to start scan: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;

    Ok(HttpResponse::Ok().json(scan))
}

#[post("/scans/{id}/cancel")]
async fn cancel_scan(
    id: web::Path<String>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    let scan = service.cancel_scan(scan_id).await.map_err(|e| match e {
        CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
        CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
        _ => {
            tracing::error!("Failed to cancel scan: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;

    Ok(HttpResponse::Ok().json(scan))
}

#[post("/scans/{id}/targets")]
async fn add_targets(
    id: web::Path<String>,
    request: web::Json<AddTargetRequest>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    // This would call service.add_targets(), which we haven't implemented yet
    // For now, return a not implemented error
    Err(actix_web::error::ErrorNotImplemented(
        "Adding targets to an existing scan is not yet implemented",
    ))
}

#[post("/scans/{id}/modules")]
async fn add_modules(
    id: web::Path<String>,
    request: web::Json<AddModuleRequest>,
    service: web::Data<ScannerService>,
) -> Result<HttpResponse, Error> {
    let scan_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid scan ID format"))?;

    // This would call service.add_modules(), which we haven't implemented yet
    // For now, return a not implemented error
    Err(actix_web::error::ErrorNotImplemented(
        "Adding modules to an existing scan is not yet implemented",
    ))
}
