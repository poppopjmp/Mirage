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

        // Log the incoming request
        info!(
            "[{}] {} {} - Request started at {}",
            request_id,
            method,
            path,
            Utc::now().to_rfc3339()
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
