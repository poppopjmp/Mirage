use crate::channels::{get_channel, Channel};
use crate::config::AppConfig;
use crate::models::{
    CreateSubscriptionRequest, DeliveryStatusResponse, Notification, NotificationChannelRequest,
    NotificationDelivery, NotificationResponse, NotificationStatus, NotificationStatusResponse,
    SendNotificationRequest, Subscription, SubscriptionResponse,
};
use crate::repositories::{DbPool, NotificationRepository};
use crate::templates::TemplateRegistry;
use chrono::Utc;
use mirage_common::{Error, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationService {
    repo: Arc<NotificationRepository>,
    templates: Arc<TemplateRegistry>,
    config: Arc<AppConfig>,
}

impl NotificationService {
    pub fn new(pool: DbPool, config: AppConfig) -> Self {
        Self {
            repo: Arc::new(NotificationRepository::new(pool)),
            templates: Arc::new(TemplateRegistry::new(&config.templates)),
            config: Arc::new(config),
        }
    }

    // Send a new notification
    pub async fn send_notification(
        &self,
        request: SendNotificationRequest,
    ) -> Result<NotificationResponse> {
        // Validate request
        if request.channels.is_empty() {
            return Err(Error::Validation(
                "At least one notification channel must be provided".into(),
            ));
        }

        // Generate subject and content
        let (subject, content) = if let (Some(custom_subject), Some(custom_content)) =
            (&request.custom_subject, &request.custom_content)
        {
            // Use custom content directly if provided
            (custom_subject.clone(), custom_content.clone())
        } else if let Some(template_name) = &request.template_name {
            // TODO: Lookup custom template by name from template store
            // For now, fallback to default templates
            let subject = self.templates.render_subject(
                &request.notification_type,
                &serde_json::to_value(&request.data)?,
            )?;
            let content = self.templates.render_content(
                &request.notification_type,
                &serde_json::to_value(&request.data)?,
            )?;
            (subject, content)
        } else {
            // Use default templates based on notification type
            let subject = self.templates.render_subject(
                &request.notification_type,
                &serde_json::to_value(&request.data)?,
            )?;
            let content = self.templates.render_content(
                &request.notification_type,
                &serde_json::to_value(&request.data)?,
            )?;
            (subject, content)
        };

        // Create notification record
        let notification_id = Uuid::new_v4();
        let notification = Notification {
            id: notification_id,
            type_: request.notification_type,
            subject: subject.clone(),
            content: content.clone(),
            metadata: request.metadata.unwrap_or_default(),
            status: NotificationStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            processed_at: None,
        };

        // Save notification in database
        self.repo.create_notification(&notification).await?;

        // Create delivery records for each channel
        for channel_req in request.channels {
            let delivery = NotificationDelivery {
                id: Uuid::new_v4(),
                notification_id,
                channel: channel_req.channel,
                recipient: channel_req.recipient,
                status: NotificationStatus::Pending,
                error_message: None,
                retry_count: 0,
                next_retry_at: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                completed_at: None,
            };

            self.repo.create_delivery(&delivery).await?;
        }

        // Return notification ID
        Ok(NotificationResponse {
            notification_id,
            status: NotificationStatus::Pending,
        })
    }

    // Check status of a notification
    pub async fn get_notification_status(
        &self,
        notification_id: Uuid,
    ) -> Result<NotificationStatusResponse> {
        // Get notification
        let notification = self
            .repo
            .get_notification(&notification_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Notification with ID {} not found",
                    notification_id
                ))
            })?;

        // Get all deliveries for this notification
        let deliveries = self
            .repo
            .get_notification_deliveries(&notification_id)
            .await?;

        // Convert to delivery status responses
        let delivery_statuses: Vec<DeliveryStatusResponse> = deliveries
            .iter()
            .map(|d| DeliveryStatusResponse {
                delivery_id: d.id,
                channel: d.channel.clone(),
                recipient: d.recipient.clone(),
                status: d.status.clone(),
                error_message: d.error_message.clone(),
                retry_count: d.retry_count,
                completed_at: d.completed_at,
            })
            .collect();

        Ok(NotificationStatusResponse {
            notification_id,
            status: notification.status,
            created_at: notification.created_at,
            processed_at: notification.processed_at,
            deliveries: delivery_statuses,
        })
    }

    // Create a subscription for a notification type
    pub async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> Result<SubscriptionResponse> {
        // Create subscription record
        let subscription_id = Uuid::new_v4();
        let subscription = Subscription {
            id: subscription_id,
            notification_type: request.notification_type,
            channel: request.channel,
            recipient: request.recipient,
            filter_conditions: request.filter_conditions.unwrap_or_default(),
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save in database
        self.repo.create_subscription(&subscription).await?;

        Ok(SubscriptionResponse { subscription_id })
    }
}

// Worker function to process pending notifications
pub async fn start_notification_worker(pool: DbPool, config: AppConfig) {
    let repo = NotificationRepository::new(pool);
    let config = Arc::new(config);

    tracing::info!("Starting notification delivery worker");

    loop {
        // Process pending deliveries
        match process_pending_deliveries(&repo, &config).await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Processed {} notification deliveries", count);
                }
            }
            Err(e) => {
                tracing::error!("Error processing notification deliveries: {}", e);
            }
        }

        // Sleep before next poll
        time::sleep(Duration::from_secs(config.worker.poll_interval_seconds)).await;
    }
}

async fn process_pending_deliveries(
    repo: &NotificationRepository,
    config: &Arc<AppConfig>,
) -> Result<usize> {
    // Get pending deliveries
    let deliveries = repo
        .get_pending_deliveries(config.worker.batch_size as i64)
        .await?;

    if deliveries.is_empty() {
        return Ok(0);
    }

    let mut processed_count = 0;

    // Process each delivery
    for delivery in deliveries {
        // Get the associated notification
        if let Some(notification) = repo.get_notification(&delivery.notification_id).await? {
            // Get appropriate channel handler
            let channel = get_channel(&delivery.channel, config);

            // Attempt to send notification
            match channel
                .send(&delivery, &notification.content, &notification.subject)
                .await
            {
                Ok(_) => {
                    // Update delivery status to processed
                    repo.update_delivery_status(
                        &delivery.id,
                        NotificationStatus::Processed,
                        None,
                        None,
                        None,
                        Some(Utc::now()),
                    )
                    .await?;

                    processed_count += 1;
                }
                Err(e) => {
                    let retry_count = delivery.retry_count + 1;
                    let retry_delay =
                        config.webhook.retry_delay_seconds * (2_u64.pow(retry_count.min(5) as u32));
                    let next_retry = if retry_count < config.webhook.max_retries {
                        Some(Utc::now() + chrono::Duration::seconds(retry_delay as i64))
                    } else {
                        None
                    };

                    let status = if retry_count >= config.webhook.max_retries {
                        NotificationStatus::Failed
                    } else {
                        NotificationStatus::Pending
                    };

                    // Update delivery with error and retry information
                    repo.update_delivery_status(
                        &delivery.id,
                        status,
                        Some(format!("{}", e)),
                        Some(retry_count),
                        next_retry,
                        None,
                    )
                    .await?;

                    tracing::warn!(
                        "Failed to deliver notification {}: {} (retry {}/{})",
                        delivery.id,
                        e,
                        retry_count,
                        config.webhook.max_retries
                    );
                }
            }
        }
    }

    // Update notification status where all deliveries are complete
    // This would typically be done using a SQL query, but for simplicity we're not
    // implementing it fully here

    Ok(processed_count)
}
