use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_web::middleware::Logger;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Notification {
    id: Uuid,
    user_id: Uuid,
    message: String,
    delivered: bool,
    created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationPreference {
    user_id: Uuid,
    email: bool,
    sms: bool,
    webhook: bool,
}

type NotificationDb = Mutex<Vec<Notification>>;
type NotificationPreferenceDb = Mutex<Vec<NotificationPreference>>;

#[post("/notifications")]
async fn create_notification(notification: web::Json<Notification>, db: web::Data<NotificationDb>) -> impl Responder {
    let mut notifications = db.lock().unwrap();
    let new_notification = Notification {
        id: Uuid::new_v4(),
        delivered: false,
        created_at: chrono::Utc::now().to_rfc3339(),
        ..notification.into_inner()
    };
    notifications.push(new_notification.clone());
    HttpResponse::Ok().json(new_notification)
}

#[get("/notifications/{id}")]
async fn get_notification(id: web::Path<Uuid>, db: web::Data<NotificationDb>) -> impl Responder {
    let notifications = db.lock().unwrap();
    if let Some(notification) = notifications.iter().find(|&notification| notification.id == *id) {
        HttpResponse::Ok().json(notification)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[put("/notifications/{id}")]
async fn update_notification(id: web::Path<Uuid>, notification: web::Json<Notification>, db: web::Data<NotificationDb>) -> impl Responder {
    let mut notifications = db.lock().unwrap();
    if let Some(existing_notification) = notifications.iter_mut().find(|notification| notification.id == *id) {
        existing_notification.message = notification.message.clone();
        existing_notification.delivered = notification.delivered;
        HttpResponse::Ok().json(existing_notification.clone())
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[delete("/notifications/{id}")]
async fn delete_notification(id: web::Path<Uuid>, db: web::Data<NotificationDb>) -> impl Responder {
    let mut notifications = db.lock().unwrap();
    if let Some(pos) = notifications.iter().position(|notification| notification.id == *id) {
        notifications.remove(pos);
        HttpResponse::NoContent().finish()
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[post("/preferences")]
async fn set_preferences(preferences: web::Json<NotificationPreference>, db: web::Data<NotificationPreferenceDb>) -> impl Responder {
    let mut prefs = db.lock().unwrap();
    if let Some(existing_pref) = prefs.iter_mut().find(|pref| pref.user_id == preferences.user_id) {
        existing_pref.email = preferences.email;
        existing_pref.sms = preferences.sms;
        existing_pref.webhook = preferences.webhook;
        HttpResponse::Ok().json(existing_pref.clone())
    } else {
        let new_pref = NotificationPreference {
            ..preferences.into_inner()
        };
        prefs.push(new_pref.clone());
        HttpResponse::Ok().json(new_pref)
    }
}

#[get("/preferences/{user_id}")]
async fn get_preferences(user_id: web::Path<Uuid>, db: web::Data<NotificationPreferenceDb>) -> impl Responder {
    let prefs = db.lock().unwrap();
    if let Some(pref) = prefs.iter().find(|&pref| pref.user_id == *user_id) {
        HttpResponse::Ok().json(pref)
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let notification_db = web::Data::new(Mutex::new(Vec::<Notification>::new()));
    let preference_db = web::Data::new(Mutex::new(Vec::<NotificationPreference>::new()));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(notification_db.clone())
            .app_data(preference_db.clone())
            .service(create_notification)
            .service(get_notification)
            .service(update_notification)
            .service(delete_notification)
            .service(set_preferences)
            .service(get_preferences)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
