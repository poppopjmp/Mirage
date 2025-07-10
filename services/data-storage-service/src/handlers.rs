use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{QueryParams, StoreDataRequest, StoreRelationshipRequest};
use crate::services::StorageService;

pub fn storage_routes() -> actix_web::Scope {
    web::scope("/data")
        .service(store_data)
        .service(get_data)
        .service(update_data)
        .service(delete_data)
        .service(query_data)
        .service(create_relationship)
        .service(get_relationships)
}

#[post("")]
async fn store_data(
    data: web::Json<StoreDataRequest>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let data_id = storage_service
        .store_data(data.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to store data: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(serde_json::json!({ "data_id": data_id })))
}

#[get("/{id}")]
async fn get_data(
    id: web::Path<String>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid data ID"))?;

    let data = storage_service.get_data(&id).await.map_err(|e| match e {
        CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
        _ => {
            tracing::error!("Failed to get data: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;

    Ok(HttpResponse::Ok().json(data))
}

#[put("/{id}")]
async fn update_data(
    id: web::Path<String>,
    data: web::Json<serde_json::Value>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid data ID"))?;

    storage_service
        .update_data(&id, data.into_inner())
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
            _ => {
                tracing::error!("Failed to update data: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[delete("/{id}")]
async fn delete_data(
    id: web::Path<String>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid data ID"))?;

    storage_service
        .delete_data(&id)
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to delete data: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("")]
async fn query_data(
    query: web::Query<QueryParams>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let data = storage_service
        .query_data(query.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to query data: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    Ok(HttpResponse::Ok().json(data))
}

#[post("/relationships")]
async fn create_relationship(
    data: web::Json<StoreRelationshipRequest>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let relationship_id = storage_service
        .create_relationship(data.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create relationship: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(serde_json::json!({ "relationship_id": relationship_id })))
}

#[get("/relationships/{id}")]
async fn get_relationships(
    id: web::Path<String>,
    storage_service: web::Data<StorageService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid entity ID"))?;

    let relationships = storage_service
        .get_relationships_for_entity(&id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get relationships: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    Ok(HttpResponse::Ok().json(relationships))
}
