use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use actix_web_httpauth::middleware::HttpAuthentication;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

async fn login() -> impl Responder {
    // Implement login logic
    HttpResponse::Ok().body("Login")
}

async fn refresh() -> impl Responder {
    // Implement token refresh logic
    HttpResponse::Ok().body("Refresh")
}

async fn logout() -> impl Responder {
    // Implement logout logic
    HttpResponse::Ok().body("Logout")
}

async fn validate_token(token: &str) -> bool {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let validation = Validation::default();
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &validation).is_ok()
}

async fn generate_token(user_id: &str) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: 10000000000, // Set expiration time
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(HttpAuthentication::bearer(validate_token))
            .service(web::resource("/").to(index))
            .service(web::resource("/login").to(login))
            .service(web::resource("/refresh").to(refresh))
            .service(web::resource("/logout").to(logout))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
