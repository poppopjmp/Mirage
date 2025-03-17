use crate::config::WebhookConfig;
use crate::models::NotificationDelivery;
use mirage_common::{Error, Result};
use reqwest::Client;
use std::time::Duration;

pub struct WebhookChannel {
    config: WebhookConfig,
    client: Client,
}

impl WebhookChannel {
    pub fn new(config: &WebhookConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            config: config.clone(),
            client,
        }
    }
}

impl super::Channel for WebhookChannel {
    async fn send(&self, delivery: &NotificationDelivery, content: &str, subject: &str) -> Result<()> {
        // Create payload to send
        let payload = serde_json::json!({
            "subject": subject,
            "content": content,
            "notification_id": delivery.notification_id.to_string(),
            "delivery_id": delivery.id.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        
        // Get webhook URL from recipient field
        let webhook_url = &delivery.recipient;
        
        // Send POST request
        let response = self.client.post(webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to send webhook request: {}", e)))?;
            
        // Check response
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ExternalApi(format!("Webhook returned error ({}): {}", status, error_text)));
        }
        
        Ok(())
    }
}
