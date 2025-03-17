use actix_web::{web, HttpResponse, Responder, Error, post, get, put, delete};
use mirage_common::Error as CommonError;
use uuid::Uuid;

use crate::services::IntegrationService;
use crate::scheduler::SchedulerService;
use crate::models::{
    CreateIntegrationRequest, UpdateIntegrationRequest, IntegrationStatus, IntegrationQueryParams,
    CredentialRequest, ExecutionRequest
};

pub fn integration_routes() -> actix_web::Scope {
    web::scope("/integrations")
        .service(create_integration)
        .service(get_integration)
        .service(list_integrations)
        .service(update_integration)
        .service(delete_integration)
        .service(list_providers)
        .service(create_credential)
        .service(get_credentials)
        .service(delete_credential)
        .service(execute_integration)
        .service(get_execution)
        .service(get_recent_executions)
}

#[post("")]
async fn create_integration(
    request: web::Json<CreateIntegrationRequest>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    // Mock user ID for demonstration
    let user_id = Some("00000000-0000-0000-0000-000000000000".to_string());
    
    let result = service.create_integration(request.into_inner(), user_id).await
        .map_err(|e| {
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to create integration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Created().json(result))
}

#[get("/{id}")]
async fn get_integration(
    path: web::Path<Uuid>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    
    let integration = service.get_integration(id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get integration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(integration))
}

#[get("")]
async fn list_integrations(
    query: web::Query<IntegrationQueryParams>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    
    let result = service.list_integrations(
        query.integration_type.as_ref(),
        query.provider_id.as_deref(),
        query.status.as_ref(),
        query.tag.as_deref(),
        query.name_contains.as_deref(),
        page,
        per_page
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to list integrations: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    Ok(HttpResponse::Ok().json(result))
}

#[put("/{id}")]
async fn update_integration(
    path: web::Path<Uuid>,
    request: web::Json<UpdateIntegrationRequest>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    
    let integration = service.update_integration(id, request.into_inner()).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => {
                    tracing::error!("Failed to update integration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(integration))
}

#[delete("/{id}")]
async fn delete_integration(
    path: web::Path<Uuid>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    
    service.delete_integration(id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to delete integration: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[get("/providers")]
async fn list_providers(
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let providers = service.list_providers().await
        .map_err(|e| {
            tracing::error!("Failed to list providers: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(providers))
}

#[post("/{id}/credentials")]
async fn create_credential(
    path: web::Path<Uuid>,
    request: web::Json<CredentialRequest>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let integration_id = path.into_inner();
    
    let credential = service.create_credential(integration_id, request.into_inner()).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => {
                    tracing::error!("Failed to create credential: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Created().json(credential))
}

#[get("/{id}/credentials")]
async fn get_credentials(
    path: web::Path<Uuid>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let integration_id = path.into_inner();
    
    let credentials = service.get_credentials(integration_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get credentials: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(credentials))
}

#[delete("/{integration_id}/credentials/{credential_id}")]
async fn delete_credential(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let (integration_id, credential_id) = path.into_inner();
    
    service.delete_credential(integration_id, credential_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to delete credential: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::NoContent().finish())
}

#[post("/{id}/execute")]
async fn execute_integration(
    path: web::Path<Uuid>,
    request: web::Json<ExecutionRequest>,
    scheduler: web::Data<SchedulerService>,
) -> Result<HttpResponse, Error> {
    let integration_id = path.into_inner();
    
    let execution = scheduler.execute_integration_manually(
        &integration_id, 
        request.parameters.clone(), 
        request.target.clone()
    )
    .await
    .map_err(|e| {
        match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to execute integration: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        }
    })?;
    
    Ok(HttpResponse::Accepted().json(execution))
}

#[get("/{integration_id}/executions/{execution_id}")]
async fn get_execution(
    path: web::Path<(Uuid, Uuid)>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let (integration_id, execution_id) = path.into_inner();
    
    let execution = service.get_execution(integration_id, execution_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get execution: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(execution))
}

#[get("/{id}/executions")]
async fn get_recent_executions(
    path: web::Path<Uuid>,
    service: web::Data<IntegrationService>,
) -> Result<HttpResponse, Error> {
    let integration_id = path.into_inner();
    
    let executions = service.get_recent_executions(integration_id).await
        .map_err(|e| {
            match e {
                CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
                _ => {
                    tracing::error!("Failed to get executions: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                }
            }
        })?;
    
    Ok(HttpResponse::Ok().json(executions))
}
