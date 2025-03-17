use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::{Error, ErrorUnauthorized},
    http, web, HttpMessage,
};
use actix_cors::Cors;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use chrono::Utc;
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Instant;
use uuid::Uuid;

// Re-export common middleware from external crates
pub use actix_cors;
pub use actix_web_httpauth;

// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub role: Option<String>,
    pub perms: Option<Vec<String>>,
}

// Authentication middleware
pub struct Authentication;

impl<S, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract bearer token
            let bearer = match BearerAuth::extract(&req).await {
                Ok(bearer) => bearer,
                Err(_) => {
                    return Err(ErrorUnauthorized("Missing or invalid authorization token"));
                }
            };

            // Validate JWT token
            let token = bearer.token();
            let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let key = jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes());
            let validation = jsonwebtoken::Validation::default();

            match jsonwebtoken::decode::<Claims>(token, &key, &validation) {
                Ok(token_data) => {
                    // Check if token is expired
                    let now = Utc::now().timestamp() as usize;
                    if token_data.claims.exp < now {
                        return Err(ErrorUnauthorized("Token expired"));
                    }

                    // Store claims in request extensions for handlers to access
                    req.extensions_mut().insert(token_data.claims);
                    service.call(req).await
                }
                Err(_) => Err(ErrorUnauthorized("Invalid token")),
            }
        })
    }
}

// Role-based authorization middleware
pub struct RoleAuthorization {
    roles: Vec<String>,
}

impl RoleAuthorization {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RoleAuthorization
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RoleAuthorizationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleAuthorizationMiddleware {
            service: Rc::new(service),
            roles: self.roles.clone(),
        }))
    }
}

pub struct RoleAuthorizationMiddleware<S> {
    service: Rc<S>,
    roles: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for RoleAuthorizationMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let roles = self.roles.clone();

        Box::pin(async move {
            // Get claims from request extensions (added by Authentication middleware)
            if let Some(claims) = req.extensions().get::<Claims>() {
                if let Some(role) = &claims.role {
                    if roles.contains(role) {
                        return service.call(req).await;
                    }
                }
                
                // Also check if user has admin role which always passes
                if claims.role.as_ref().map(|r| r == "admin").unwrap_or(false) {
                    return service.call(req).await;
                }
                
                Err(ErrorUnauthorized("Insufficient permissions"))
            } else {
                Err(ErrorUnauthorized("Missing authentication information"))
            }
        })
    }
}

// Request logger middleware
pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let start_time = Instant::now();
        let request_id = Uuid::new_v4().to_string();
        let method = req.method().clone();
        let path = req.path().to_string();
        let service = Rc::clone(&self.service);

        // Set request ID in request extensions for handler access
        req.extensions_mut().insert(request_id.clone());

        // Log the incoming request
        info!(
            "[{}] {} {} - Request started",
            request_id, method, path
        );

        Box::pin(async move {
            // Process the request
            let result = service.call(req).await;

            // Calculate elapsed time
            let elapsed = start_time.elapsed();

            match &result {
                Ok(res) => {
                    // Log successful response
                    info!(
                        "[{}] {} {} - Response status: {} - Elapsed: {:.2?}",
                        request_id,
                        method,
                        path,
                        res.status().as_u16(),
                        elapsed
                    );
                }
                Err(err) => {
                    // Log error response
                    error!(
                        "[{}] {} {} - Error: {} - Elapsed: {:.2?}",
                        request_id,
                        method,
                        path,
                        err,
                        elapsed
                    );
                }
            }

            result
        })
    }
}

// CORS middleware factory
pub fn cors_middleware() -> Cors {
    let allowed_origins = env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8080".to_string());
    
    let origins: Vec<&str> = allowed_origins.split(',').collect();
    
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ])
        .max_age(3600);
    
    // Add each origin to the CORS configuration
    for origin in origins {
        cors = cors.allowed_origin(origin.trim());
    }
    
    cors
}

// Helper function to extract Claims from a request
pub fn get_claims(req: &ServiceRequest) -> Option<Claims> {
    req.extensions().get::<Claims>().cloned()
}

// Helper function to extract user_id from Claims
pub fn get_user_id(req: &ServiceRequest) -> Option<String> {
    get_claims(req).map(|claims| claims.sub)
}

// Helper function to check if a user has a specific permission
pub fn has_permission(req: &ServiceRequest, permission: &str) -> bool {
    if let Some(claims) = get_claims(req) {
        // Admin role always has all permissions
        if claims.role.as_ref().map(|r| r == "admin").unwrap_or(false) {
            return true;
        }
        
        // Check specific permission
        if let Some(perms) = claims.perms {
            return perms.contains(&permission.to_string());
        }
    }
    
    false
}
