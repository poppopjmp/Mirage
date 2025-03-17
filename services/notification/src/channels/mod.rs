use crate::config::AppConfig;
use crate::models::{NotificationChannel, NotificationDelivery};
use mirage_common::Result;

mod email;
mod webhook;
mod slack;
mod database;

pub use email::EmailChannel;
pub use webhook::WebhookChannel;
pub use slack::SlackChannel;
pub use database::DatabaseChannel;

pub trait Channel {
    async fn send(&self, delivery: &NotificationDelivery, content: &str, subject: &str) -> Result<()>;
}

pub fn get_channel(channel_type: &NotificationChannel, config: &AppConfig) -> Box<dyn Channel + Send + Sync> {
    match channel_type {
        NotificationChannel::Email => Box::new(EmailChannel::new(&config.email)),
        NotificationChannel::Webhook => Box::new(WebhookChannel::new(&config.webhook)),
        NotificationChannel::Slack => Box::new(SlackChannel::new(&config.slack)),
        NotificationChannel::Database => Box::new(DatabaseChannel::new()),
    }
}
