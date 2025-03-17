use actix_web::{web, HttpResponse, Responder, Error, post, get, put, delete};
use mirage_common::Error as CommonError;
use uuid::Uuid;

use crate::services::ConfigService;
use crate::models::{
    CreateConfigRequest, UpdateConfigRequest, CreateNamespaceRequest, ConfigQueryParams
};

pub fn config_routes() -> actix_web::Scope {
    web::scope("/config")
        .service(create_config)
        .service(get_config)
        .service(get_config_by_key)
        .service(update_config)
        .service(delete_config)
        .service(list_configs)
        .service(get_config_history)
        .service(create_namespace)
        .service(list_namespaces)
        .service(get_raw_config_value)
}

#[post("/items")]
async fn create_config(
    request: web::Json<CreateConfigRequest>,
    config_service: web::Data<ConfigService>,
    // In production, we'd extract user ID from auth headers
) -> Result<HttpResponse, Error> {
    // Mock user ID for testing
    let user_id = Some("config-api".to_string());
    
    let result = config_service.create_config(request.into_inner(), user_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::Conflict(_) => actix_web::error::ErrorConflict(e),
                _ => {
                    tracing::error!("Error creating configuration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Created().json(result))
}

#[get("/items/{id}")]
async fn get_config(
    id: web::Path<String>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let config_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid config ID format")
    })?;
    
    let config = config_service.get_config(config_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Error fetching configuration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(config))
}

#[get("/namespaces/{namespace}/items/{key}")]
async fn get_config_by_key(
    path: web::Path<(String, String)>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let (namespace, key) = path.into_inner();
    
    let config = config_service.get_config_by_key(&key, &namespace).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Error fetching configuration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(config))
}

#[put("/items/{id}")]
async fn update_config(
    id: web::Path<String>,
    request: web::Json<UpdateConfigRequest>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let config_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid config ID format")
    })?;
    
    // Mock user ID for testing
    let user_id = Some("config-api".to_string());
    
    let config = config_service.update_config(config_id, request.into_inner(), user_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => {
                    tracing::error!("Error updating configuration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(config))
}

#[delete("/items/{id}")]
async fn delete_config(
    id: web::Path<String>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let config_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid config ID format")
    })?;
    
    // Mock user ID for testing
    let user_id = Some("config-api");
    
    config_service.delete_config(config_id, user_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Error deleting configuration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[get("/items")]
async fn list_configs(
    query: web::Query<ConfigQueryParams>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    // Extract query parameters with defaults
    let namespace = query.namespace.as_deref();
    let tag = query.tag.as_deref();
    let key_contains = query.key_contains.as_deref();
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    
    let result = config_service.list_configs(
        namespace,
        tag,
        key_contains,
        page,
        per_page
    ).await
    .map_err(|e| {
        tracing::error!("Error listing configurations: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[get("/items/{id}/history")]
async fn get_config_history(
    id: web::Path<String>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let config_id = Uuid::parse_str(&id).map_err(|_| {
        actix_web::error::ErrorBadRequest("Invalid config ID format")
    })?;
    
    let history = config_service.get_config_history(config_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Error fetching configuration history: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(history))
}

#[post("/namespaces")]
async fn create_namespace(
    request: web::Json<CreateNamespaceRequest>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    // Mock user ID for testing
    let user_id = Some("config-api");
    
    let result = config_service.create_namespace(request.into_inner(), user_id).await
        .map_err(|e| {
            match e {
                CommonError::Conflict(_) => actix_web::error::ErrorConflict(e),
                _ => {
                    tracing::error!("Error creating namespace: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Created().json(result))
}

#[get("/namespaces")]
async fn list_namespaces(
    query: web::Query<ConfigQueryParams>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    
    let result = config_service.list_namespaces(page, per_page).await
        .map_err(|e| {
            tracing::error!("Error listing namespaces: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[get("/namespaces/{namespace}/items/{key}/raw")]
async fn get_raw_config_value(
    path: web::Path<(String, String)>,
    config_service: web::Data<ConfigService>,
) -> Result<HttpResponse, Error> {
    let (namespace, key) = path.into_inner();
    
    let value = config_service.get_raw_config_value(&key, &namespace).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Error fetching raw configuration value: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(value))
}
