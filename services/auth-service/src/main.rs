use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use log::info;

mod models;
mod routes;

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok", "service": "auth-service" }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("Starting auth-service on port 8001");

    HttpServer::new(move || {
        App::new()
            .route("/health", web::get().to(health_check))
            .configure(routes::config)
    })
    .bind(("0.0.0.0", 8001))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{test, App, web};
    use super::*;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(health_check))
        ).await;
        
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "auth-service");
    }

    #[actix_web::test]
    async fn test_auth_routes() {
        let app = test::init_service(
            App::new().configure(routes::config)
        ).await;
        
        // Test login endpoint
        let req = test::TestRequest::post().uri("/auth/login").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        // Test register endpoint
        let req = test::TestRequest::post().uri("/auth/register").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        
        // Test logout endpoint
        let req = test::TestRequest::post().uri("/auth/logout").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
