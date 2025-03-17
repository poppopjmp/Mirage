use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use chrono::Utc;
use futures::future::{ready, LocalBoxFuture, Ready};
use log::{debug, error, info};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Instant;
use uuid::Uuid;

// Global request metrics middleware
pub struct RequestMetrics;

impl<S, B> Transform<S, ServiceRequest> for RequestMetrics
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestMetricsMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestMetricsMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestMetricsMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestMetricsMiddleware<S>
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
        let path = req.path().to_string();
        let method = req.method().clone();
        let service = Rc::clone(&self.service);

        // Request ID for tracing
        let request_id = Uuid::new_v4().to_string();
        req.extensions_mut().insert(request_id.clone());

        Box::pin(async move {
            // Process the request
            let result = service.call(req).await;

            // Calculate elapsed time
            let elapsed = start_time.elapsed();

            // Log metrics information
            match &result {
                Ok(res) => {
                    // Record request success
                    let status = res.status();
                    info!(
                        "[METRICS][{}] {} {} - Status: {} - Elapsed: {:.2?}",
                        request_id, method, path, status.as_u16(), elapsed
                    );

                    // Here we would track in Prometheus or other metrics system:
                    // - Request count by path, method, status code
                    // - Request duration
                    // - Response size
                }
                Err(e) => {
                    // Record request failure
                    error!(
                        "[METRICS][{}] {} {} - Error: {} - Elapsed: {:.2?}",
                        request_id, method, path, e, elapsed
                    );

                    // Here we would track in Prometheus or other metrics system:
                    // - Error counts by path, method, error type
                    // - Failure request duration
                }
            }

            result
        })
    }
}

// Rate limiting middleware
pub struct RateLimiter {
    pub requests_per_minute: u32,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            requests_per_minute: self.requests_per_minute,
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    requests_per_minute: u32,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let limits = self.requests_per_minute;

        Box::pin(async move {
            // In a real implementation, we would use Redis to track rates
            // For now, we'll just allow all requests
            
            // Get client IP for rate tracking
            let ip = req.connection_info().peer_addr()
                .unwrap_or("unknown")
                .to_string();
                
            // Extract API key if present (for more granular rate limits)
            let api_key = req.headers().get("X-API-Key")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string());
                
            // Check rate limit (In real implementation)
            // let allowed = check_rate_limit(ip, api_key, limits).await;
            let allowed = true; // Mock implementation
                
            if allowed {
                service.call(req).await
            } else {
                Err(actix_web::error::ErrorTooManyRequests("Rate limit exceeded"))
            }
        })
    }
}

// Request tracing middleware
pub struct RequestTracing;

impl<S, B> Transform<S, ServiceRequest> for RequestTracing
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestTracingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestTracingMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestTracingMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestTracingMiddleware<S>
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
        let trace_id = req.headers().get("X-Trace-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_else(|| {
                // Generate a new trace ID if none provided
                Uuid::new_v4().to_string().as_str()
            })
            .to_string();
            
        // Store trace ID in request extensions
        req.extensions_mut().insert(trace_id.clone());
        
        // Add trace ID to response headers
        Box::pin(async move {
            let mut res = service.call(req).await?;
            
            // Add trace ID to response headers
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-trace-id"),
                actix_web::http::header::HeaderValue::from_str(&trace_id).unwrap(),
            );
            
            Ok(res)
        })
    }
}
