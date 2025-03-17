use actix_web::{web, HttpResponse, Responder, Error, post, get};
use mirage_common::Error as CommonError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::CollectionService;
use crate::models::{ExecuteModuleRequest, CollectionResult, CreateTaskRequest, BatchTaskRequest, TaskQueryParams};

pub fn collection_routes() -> actix_web::Scope {
    web::scope("/collection")
        .service(execute_module)
        .service(list_modules)
        .service(get_result)
        .service(create_task)
        .service(create_batch_tasks)
        .service(get_task)
        .service(get_task_result)
        .service(cancel_task)
        .service(list_tasks)
}

#[post("/execute")]
async fn execute_module(
    data: web::Json<ExecuteModuleRequest>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let job_id = collection_service.execute_module(data.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to execute module: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::RateLimit(_) => actix_web::error::ErrorTooManyRequests(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
        
    Ok(HttpResponse::Accepted().json(serde_json::json!({ "job_id": job_id })))
}

#[get("/modules")]
async fn list_modules(
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let modules = collection_service.list_available_modules().await
        .map_err(|e| {
            tracing::error!("Failed to list modules: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
        
    Ok(HttpResponse::Ok().json(modules))
}

#[get("/results/{id}")]
async fn get_result(
    id: web::Path<String>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid job ID")
    })?;
    
    let result = collection_service.get_result(&id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get result: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
        
    Ok(HttpResponse::Ok().json(result))
}

#[post("/tasks")]
async fn create_task(
    request: web::Json<CreateTaskRequest>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let result = collection_service.create_task(request.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to create task: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
    
    Ok(HttpResponse::Created().json(result))
}

#[post("/batch")]
async fn create_batch_tasks(
    request: web::Json<BatchTaskRequest>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let result = collection_service.create_batch_tasks(request.into_inner()).await
        .map_err(|e| {
            tracing::error!("Failed to create batch tasks: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;
    
    Ok(HttpResponse::Created().json(result))
}

#[get("/tasks/{id}")]
async fn get_task(
    id: web::Path<Uuid>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let task = collection_service.get_task(*id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get task {}: {}", id, e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(task))
}

#[get("/tasks/{id}/results")]
async fn get_task_result(
    id: web::Path<Uuid>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let result = collection_service.get_task_result(*id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get task result {}: {}", id, e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    match result {
        Some(task_result) => Ok(HttpResponse::Ok().json(task_result)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No results found for this task"
        })))
    }
}

#[post("/tasks/{id}/cancel")]
async fn cancel_task(
    id: web::Path<Uuid>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let result = collection_service.cancel_task(*id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => {
                    tracing::error!("Failed to cancel task {}: {}", id, e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[get("/tasks")]
async fn list_tasks(
    query: web::Query<TaskQueryParams>,
    collection_service: web::Data<CollectionService>,
) -> Result<HttpResponse, Error> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    
    let (tasks, total) = collection_service.list_tasks(
        query.status.clone(),
        query.module_id,
        query.scan_id,
        query.target_type.clone(),
        page,
        per_page
    ).await
        .map_err(|e| {
            tracing::error!("Failed to list tasks: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "items": tasks,
        "total": total,
        "page": page,
        "per_page": per_page,
        "pages": (total + per_page - 1) / per_page
    })))
}
