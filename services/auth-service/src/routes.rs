use actix_web::{web, HttpResponse, Responder};

pub async fn login() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "message": "Login endpoint" }))
}

pub async fn register() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "message": "Register endpoint" }))
}

pub async fn logout() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "message": "Logout endpoint" }))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post().to(login))
            .route("/register", web::post().to(register))
            .route("/logout", web::post().to(logout)),
    );
}
