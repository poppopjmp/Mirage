use crate::config::EmailConfig;
use crate::models::NotificationDelivery;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use mirage_common::{Error, Result};

pub struct EmailChannel {
    config: EmailConfig,
}

impl EmailChannel {
    pub fn new(config: &EmailConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }
}

impl super::Channel for EmailChannel {
    async fn send(
        &self,
        delivery: &NotificationDelivery,
        content: &str,
        subject: &str,
    ) -> Result<()> {
        // Create message
        let message = Message::builder()
            .from(
                format!("{} <{}>", self.config.from_name, self.config.from_address)
                    .parse()
                    .unwrap(),
            )
            .to(delivery.recipient.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(content.to_string())
            .map_err(|e| Error::Internal(format!("Failed to create email: {}", e)))?;

        // Set up credentials
        let creds = Credentials::new(self.config.username.clone(), self.config.password.clone());

        // Open a remote connection to the SMTP server
        let mailer = SmtpTransport::relay(&self.config.smtp_server)
            .map_err(|e| Error::Internal(format!("Failed to create SMTP transport: {}", e)))?
            .credentials(creds)
            .port(self.config.smtp_port)
            .build();

        // Send the email
        mailer
            .send(&message)
            .map_err(|e| Error::ExternalApi(format!("Failed to send email: {}", e)))?;

        Ok(())
    }
}
