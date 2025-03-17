use actix_cors::Cors;
use actix_web::http;
use std::env;

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
