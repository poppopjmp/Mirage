//! Authentication middleware for API Gateway

use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    // Placeholder implementation - just validate that token exists
    let token = credentials.token();

    if !token.is_empty() {
        // In real implementation, this would validate JWT with auth service
        req.extensions_mut().insert(token.to_string());
        Ok(req)
    } else {
        Err(actix_web::error::ErrorUnauthorized("Invalid token"))
    }
}

pub async fn verify_token(token: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Placeholder implementation
    // In real implementation, this would call the auth service
    Ok(!token.is_empty())
}
