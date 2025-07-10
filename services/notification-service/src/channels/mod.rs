use crate::config::AppConfig;
use crate::models::{NotificationChannel, NotificationDelivery};
use mirage_common::Result;

mod database;
mod email;
mod slack;
mod webhook;

pub use database::DatabaseChannel;
pub use email::EmailChannel;
pub use slack::SlackChannel;
pub use webhook::WebhookChannel;

pub trait Channel {
    async fn send(
        &self,
        delivery: &NotificationDelivery,
        content: &str,
        subject: &str,
    ) -> Result<()>;
}

pub fn get_channel(
    channel_type: &NotificationChannel,
    config: &AppConfig,
) -> Box<dyn Channel + Send + Sync> {
    match channel_type {
        NotificationChannel::Email => Box::new(EmailChannel::new(&config.email)),
        NotificationChannel::Webhook => Box::new(WebhookChannel::new(&config.webhook)),
        NotificationChannel::Slack => Box::new(SlackChannel::new(&config.slack)),
        NotificationChannel::Database => Box::new(DatabaseChannel::new()),
    }
}
