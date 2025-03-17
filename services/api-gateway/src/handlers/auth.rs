use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::AppState;

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
    pub user_id: String,
}

pub async fn login(
    data: web::Json<LoginRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // Forward to auth service
    if let Some(auth_url) = state.service_endpoints.get("auth") {
        match client.post(&format!("{}/api/v1/auth/login", auth_url))
            .json(&data)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<TokenResponse>().await {
                            Ok(token_data) => HttpResponse::Ok().json(token_data),
                            Err(_) => HttpResponse::InternalServerError().body("Failed to parse auth response")
                        }
                    } else {
                        HttpResponse::Unauthorized().body("Invalid credentials")
                    }
                },
                Err(_) => HttpResponse::ServiceUnavailable().body("Auth service unavailable")
            }
    } else {
        HttpResponse::InternalServerError().body("Auth service not configured")
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register(
    data: web::Json<RegisterRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // Forward to auth service
    if let Some(auth_url) = state.service_endpoints.get("auth") {
        match client.post(&format!("{}/api/v1/auth/register", auth_url))
            .json(&data)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        HttpResponse::Created().json(serde_json::json!({
                            "status": "success",
                            "message": "User registered successfully"
                        }))
                    } else {
                        match response.text().await {
                            Ok(error_text) => HttpResponse::BadRequest().body(error_text),
                            Err(_) => HttpResponse::BadRequest().body("Registration failed")
                        }
                    }
                },
                Err(_) => HttpResponse::ServiceUnavailable().body("Auth service unavailable")
            }
    } else {
        HttpResponse::InternalServerError().body("Auth service not configured")
    }
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

pub async fn refresh_token(
    data: web::Json<RefreshTokenRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let client = reqwest::Client::new();
    
    // Forward to auth service
    if let Some(auth_url) = state.service_endpoints.get("auth") {
        match client.post(&format!("{}/api/v1/auth/refresh", auth_url))
            .json(&data)
            .send()
            .await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<TokenResponse>().await {
                            Ok(token_data) => HttpResponse::Ok().json(token_data),
                            Err(_) => HttpResponse::InternalServerError().body("Failed to parse auth response")
                        }
                    } else {
                        HttpResponse::Unauthorized().body("Invalid refresh token")
                    }
                },
                Err(_) => HttpResponse::ServiceUnavailable().body("Auth service unavailable")
            }
    } else {
        HttpResponse::InternalServerError().body("Auth service not configured")
    }
}
