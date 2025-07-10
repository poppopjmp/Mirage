use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use mirage_common::{models::Module, Error as CommonError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{CreateModuleRequest, UpdateModuleRequest};
use crate::services::ModuleService;

pub fn module_routes() -> actix_web::Scope {
    web::scope("/modules")
        .service(list_modules)
        .service(get_module)
        .service(register_module)
        .service(update_module)
        .service(delete_module)
}

#[get("")]
async fn list_modules(
    module_service: web::Data<ModuleService>,
    query: web::Query<ListModulesQuery>,
) -> Result<HttpResponse, Error> {
    let modules = module_service
        .list_modules(query.limit.unwrap_or(100), query.offset.unwrap_or(0))
        .await
        .map_err(|e| {
            tracing::error!("Failed to list modules: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;

    Ok(HttpResponse::Ok().json(modules))
}

#[get("/{id}")]
async fn get_module(
    id: web::Path<String>,
    module_service: web::Data<ModuleService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid module ID"))?;

    let module = module_service.get_module(&id).await.map_err(|e| match e {
        CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
        _ => {
            tracing::error!("Failed to get module: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        }
    })?;

    Ok(HttpResponse::Ok().json(module))
}

#[post("")]
async fn register_module(
    data: web::Json<CreateModuleRequest>,
    module_service: web::Data<ModuleService>,
) -> Result<HttpResponse, Error> {
    let module = module_service
        .register_module(data.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to register module: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(module))
}

#[put("/{id}")]
async fn update_module(
    id: web::Path<String>,
    data: web::Json<UpdateModuleRequest>,
    module_service: web::Data<ModuleService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid module ID"))?;

    let module = module_service
        .update_module(&id, data.into_inner())
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
            _ => {
                tracing::error!("Failed to update module: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(module))
}

#[delete("/{id}")]
async fn delete_module(
    id: web::Path<String>,
    module_service: web::Data<ModuleService>,
) -> Result<HttpResponse, Error> {
    let id =
        Uuid::parse_str(&id).map_err(|_| actix_web::error::ErrorBadRequest("Invalid module ID"))?;

    module_service
        .delete_module(&id)
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to delete module: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize)]
struct ListModulesQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    capability: Option<String>,
}
