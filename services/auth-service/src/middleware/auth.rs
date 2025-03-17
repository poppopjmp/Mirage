use crate::models::Claims;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

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
            // Extract bearer token from the Authorization header
            let bearer = match BearerAuth::extract(&req).await {
                Ok(bearer) => bearer,
                Err(_) => {
                    return Err(actix_web::error::ErrorUnauthorized("Missing or invalid authorization token"));
                }
            };

            // Validate JWT token
            let token = bearer.token();
            let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
            let key = DecodingKey::from_secret(jwt_secret.as_bytes());
            let validation = Validation::default();

            match decode::<Claims>(token, &key, &validation) {
                Ok(token_data) => {
                    // Check if token is expired
                    let now = chrono::Utc::now().timestamp() as usize;
                    if token_data.claims.exp < now {
                        return Err(actix_web::error::ErrorUnauthorized("Token expired"));
                    }

                    // Store claims in request extensions for handlers to access
                    req.extensions_mut().insert(token_data.claims);
                    service.call(req).await
                }
                Err(_) => Err(actix_web::error::ErrorUnauthorized("Invalid token")),
            }
        })
    }
}
