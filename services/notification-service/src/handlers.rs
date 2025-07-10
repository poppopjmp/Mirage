use actix_web::{get, post, web, Error, HttpResponse, Responder};
use mirage_common::Error as CommonError;
use uuid::Uuid;

use crate::models::{CreateSubscriptionRequest, SendNotificationRequest};
use crate::services::NotificationService;

pub fn notification_routes() -> actix_web::Scope {
    web::scope("/notifications")
        .service(send_notification)
        .service(get_notification_status)
        .service(create_subscription)
}

#[post("")]
async fn send_notification(
    request: web::Json<SendNotificationRequest>,
    notification_service: web::Data<NotificationService>,
) -> Result<HttpResponse, Error> {
    let result = notification_service
        .send_notification(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to send notification: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}

#[get("/{id}")]
async fn get_notification_status(
    id: web::Path<String>,
    notification_service: web::Data<NotificationService>,
) -> Result<HttpResponse, Error> {
    let notification_id = Uuid::parse_str(&id)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid notification ID format"))?;

    let status = notification_service
        .get_notification_status(notification_id)
        .await
        .map_err(|e| match e {
            CommonError::NotFound(_) => actix_web::error::ErrorNotFound(e),
            _ => {
                tracing::error!("Failed to get notification status: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            }
        })?;

    Ok(HttpResponse::Ok().json(status))
}

#[post("/subscriptions")]
async fn create_subscription(
    request: web::Json<CreateSubscriptionRequest>,
    notification_service: web::Data<NotificationService>,
) -> Result<HttpResponse, Error> {
    let result = notification_service
        .create_subscription(request.into_inner())
        .await
        .map_err(|e| {
            tracing::error!("Failed to create subscription: {}", e);
            match e {
                CommonError::Validation(_) => actix_web::error::ErrorBadRequest(e),
                _ => actix_web::error::ErrorInternalServerError(e),
            }
        })?;

    Ok(HttpResponse::Created().json(result))
}
