use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use mirage_common::Result as CommonResult;

use crate::health::HealthService;
use crate::models::{
    ServiceHeartbeatRequest, ServiceQuery, ServiceRegistrationRequest, ServiceStatus,
};
use crate::services::DiscoveryService;

pub fn discovery_routes() -> actix_web::Scope {
    web::scope("/discovery")
        .service(register_service)
        .service(get_service)
        .service(get_services)
        .service(query_services)
        .service(get_service_instances)
        .service(send_heartbeat)
        .service(deregister_service)
        .service(get_service_health)
        .service(get_health)
}

#[post("/services")]
async fn register_service(
    request: web::Json<ServiceRegistrationRequest>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    let result = service
        .register_service(request.into_inner())
        .await
        .map_err(|e| match e {
            _ => {
                tracing::error!("Error registering service: {}", e);
                actix_web::error::ErrorInternalServerError(CommonError::from(e))
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[get("/services/{id}")]
async fn get_service(
    id: web::Path<String>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    let instance = service.get_service(&id).await.map_err(|e| match e {
        _ => {
            tracing::error!("Error getting service {}: {}", id, e);
            actix_web::error::ErrorInternalServerError(CommonError::from(e))
        }
    })?;

    Ok(HttpResponse::Ok().json(instance))
}

#[get("/services")]
async fn get_services(service: web::Data<DiscoveryService>) -> Result<HttpResponse, Error> {
    let services = service.get_all_services().await.map_err(|e| {
        tracing::error!("Error getting services: {}", e);
        actix_web::error::ErrorInternalServerError(CommonError::from(e))
    })?;

    Ok(HttpResponse::Ok().json(services))
}

#[post("/services/query")]
async fn query_services(
    query: web::Json<ServiceQuery>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    let services = service
        .query_services(query.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Error querying services: {}", e);
            actix_web::error::ErrorInternalServerError(CommonError::from(e))
        })?;

    Ok(HttpResponse::Ok().json(services))
}

#[get("/services/instances/{name}")]
async fn get_service_instances(
    name: web::Path<String>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    let instances = service
        .get_service_instances(&name)
        .await
        .map_err(|e| match e {
            _ => {
                tracing::error!("Error getting instances for service {}: {}", name, e);
                actix_web::error::ErrorInternalServerError(CommonError::from(e))
            }
        })?;

    Ok(HttpResponse::Ok().json(instances))
}

#[put("/services/{id}/heartbeat")]
async fn send_heartbeat(
    id: web::Path<String>,
    request: web::Json<ServiceHeartbeatRequest>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    let mut heartbeat_req = request.into_inner();
    heartbeat_req.id = id.into_inner();

    let result = service
        .heartbeat(heartbeat_req)
        .await
        .map_err(|e| match e {
            _ => {
                tracing::error!("Error sending heartbeat: {}", e);
                actix_web::error::ErrorInternalServerError(CommonError::from(e))
            }
        })?;

    Ok(HttpResponse::Ok().json(result))
}

#[delete("/services/{id}")]
async fn deregister_service(
    id: web::Path<String>,
    service: web::Data<DiscoveryService>,
) -> Result<HttpResponse, Error> {
    service.deregister_service(&id).await.map_err(|e| match e {
        _ => {
            tracing::error!("Error deregistering service {}: {}", id, e);
            actix_web::error::ErrorInternalServerError(CommonError::from(e))
        }
    })?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/health/{id}")]
async fn get_service_health(
    id: web::Path<String>,
    health_service: web::Data<HealthService>,
) -> Result<HttpResponse, Error> {
    let result = health_service.get_service_health(&id).await.map_err(|e| {
        tracing::error!("Error getting health for service {}: {}", id, e);
        actix_web::error::ErrorInternalServerError(CommonError::from(e))
    })?;

    match result {
        Some(health) => Ok(HttpResponse::Ok().json(health)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Health information for service {} not found", id)
        }))),
    }
}

#[get("/health")]
async fn get_health(health_service: web::Data<HealthService>) -> Result<HttpResponse, Error> {
    let results = health_service.get_health_results().await.map_err(|e| {
        tracing::error!("Error getting health results: {}", e);
        actix_web::error::ErrorInternalServerError(CommonError::from(e))
    })?;

    // Convert to a more structured response
    let health_results: Vec<_> = results.values().collect();

    // Calculate summary stats
    let total = health_results.len();
    let up = health_results
        .iter()
        .filter(|r| r.status == ServiceStatus::Up)
        .count();
    let down = health_results
        .iter()
        .filter(|r| r.status == ServiceStatus::Down)
        .count();
    let unknown = total - up - down;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "summary": {
            "total": total,
            "up": up,
            "down": down,
            "unknown": unknown,
        },
        "timestamp": chrono::Utc::now(),
        "services": health_results,
    })))
}
