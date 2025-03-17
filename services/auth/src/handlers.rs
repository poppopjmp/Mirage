use actix_web::{web, HttpResponse, Responder, Error, post, get};
use mirage_common::Result as CommonResult;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::{self, AuthService};
use crate::config::AppConfig;

pub fn auth_routes() -> actix_web::Scope {
    web::scope("/auth")
        .service(login)
        .service(register)
        .service(refresh_token)
        .service(logout)
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user_id: Uuid,
}

#[post("/login")]
async fn login(
    data: web::Json<LoginRequest>,
    auth_service: web::Data<AuthService>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let login_result = auth_service.login(&data.username, &data.password).await
        .map_err(|e| {
            tracing::error!("Login error: {}", e);
            actix_web::error::ErrorUnauthorized(e)
        })?;
        
    // Generate tokens
    let access_token = services::generate_token(
        &login_result.user_id,
        &login_result.roles,
        &config.jwt.secret,
        config.jwt.expiration,
    ).map_err(|e| {
        tracing::error!("Token generation error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    let refresh_token = services::generate_refresh_token(
        &login_result.user_id,
        &config.jwt.secret,
        config.jwt.refresh_expiration,
    ).map_err(|e| {
        tracing::error!("Refresh token generation error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    let response = TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: config.jwt.expiration,
        user_id: login_result.user_id,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/register")]
async fn register(
    data: web::Json<RegisterRequest>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, Error> {
    let user = auth_service.register(&data.username, &data.email, &data.password).await
        .map_err(|e| {
            tracing::error!("Registration error: {}", e);
            actix_web::error::ErrorBadRequest(e)
        })?;
    
    Ok(HttpResponse::Created().json(user))
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[post("/refresh")]
async fn refresh_token(
    data: web::Json<RefreshTokenRequest>,
    auth_service: web::Data<AuthService>,
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, Error> {
    let refresh_result = auth_service.refresh_token(&data.refresh_token).await
        .map_err(|e| {
            tracing::error!("Token refresh error: {}", e);
            actix_web::error::ErrorUnauthorized(e)
        })?;
    
    let access_token = services::generate_token(
        &refresh_result.user_id,
        &refresh_result.roles,
        &config.jwt.secret,
        config.jwt.expiration,
    ).map_err(|e| {
        tracing::error!("Token generation error: {}", e);
        actix_web::error::ErrorInternalServerError(e)
    })?;
    
    let response = TokenResponse {
        access_token,
        refresh_token: data.refresh_token.clone(),
        token_type: "Bearer".to_string(),
        expires_in: config.jwt.expiration,
        user_id: refresh_result.user_id,
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

#[post("/logout")]
async fn logout(
    data: web::Json<LogoutRequest>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, Error> {
    auth_service.logout(&data.refresh_token).await
        .map_err(|e| {
            tracing::error!("Logout error: {}", e);
            actix_web::error::ErrorInternalServerError(e)
        })?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({ "status": "success" })))
}
