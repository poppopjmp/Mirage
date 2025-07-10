use crate::config::SlackConfig;
use crate::models::NotificationDelivery;
use mirage_common::{Error, Result};
use reqwest::Client;

pub struct SlackChannel {
    config: SlackConfig,
    client: Client,
}

impl SlackChannel {
    pub fn new(config: &SlackConfig) -> Self {
        let client = Client::new();
        Self {
            config: config.clone(),
            client,
        }
    }
}

impl super::Channel for SlackChannel {
    async fn send(
        &self,
        delivery: &NotificationDelivery,
        content: &str,
        subject: &str,
    ) -> Result<()> {
        // Determine channel - either recipient field or default from config
        let channel = if delivery.recipient.starts_with('#') || delivery.recipient.starts_with('@')
        {
            delivery.recipient.clone()
        } else {
            self.config.default_channel.clone()
        };

        // Create Slack message payload
        let payload = serde_json::json!({
            "channel": channel,
            "text": subject,
            "blocks": [
                {
                    "type": "header",
                    "text": {
                        "type": "plain_text",
                        "text": subject
                    }
                },
                {
                    "type": "section",
                    "text": {
                        "type": "mrkdwn",
                        "text": content
                    }
                },
                {
                    "type": "context",
                    "elements": [
                        {
                            "type": "mrkdwn",
                            "text": format!("*Notification ID:* {}", delivery.notification_id)
                        }
                    ]
                }
            ]
        });

        // Send to Slack webhook
        let response = self
            .client
            .post(&self.config.webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to send Slack message: {}", e)))?;

        // Check response
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Error::ExternalApi(format!(
                "Slack API returned error ({}): {}",
                status, error_text
            )));
        }

        Ok(())
    }
}
