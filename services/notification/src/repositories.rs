use crate::config::DatabaseConfig;
use crate::models::{Notification, NotificationDelivery, Subscription, NotificationStatus, NotificationType, NotificationChannel};
use chrono::{DateTime, Utc};
use mirage_common::{Error, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, query, query_as};
use uuid::Uuid;
use std::collections::HashMap;

pub type DbPool = Pool<Postgres>;

/// Create database connection pool
pub async fn create_db_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| Error::Database(format!("Database connection failed: {}", e)))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;

    Ok(pool)
}

pub struct NotificationRepository {
    pool: DbPool,
}

impl NotificationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_notification(&self, notification: &Notification) -> Result<Uuid> {
        let id = query!(
            r#"
            INSERT INTO notifications
                (id, type, subject, content, metadata, status, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            notification.id,
            notification.type_.to_string(),
            notification.subject,
            notification.content,
            serde_json::to_value(&notification.metadata)
                .map_err(|e| Error::Internal(format!("Failed to serialize metadata: {}", e)))?,
            notification.status.to_string(),
            notification.created_at,
            notification.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create notification: {}", e)))?
        .id;
        
        Ok(id)
    }
    
    pub async fn create_delivery(&self, delivery: &NotificationDelivery) -> Result<Uuid> {
        let id = query!(
            r#"
            INSERT INTO notification_deliveries
                (id, notification_id, channel, recipient, status, error_message, 
                 retry_count, next_retry_at, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
            "#,
            delivery.id,
            delivery.notification_id,
            delivery.channel.to_string(),
            delivery.recipient,
            delivery.status.to_string(),
            delivery.error_message,
            delivery.retry_count,
            delivery.next_retry_at,
            delivery.created_at,
            delivery.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create delivery: {}", e)))?
        .id;
        
        Ok(id)
    }
    
    pub async fn update_notification_status(
        &self, 
        notification_id: &Uuid, 
        status: NotificationStatus,
        processed_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut builder = query!(
            r#"
            UPDATE notifications
            SET status = $1, updated_at = $2
            "#,
            status.to_string(),
            Utc::now(),
        );
        
        if let Some(processed_time) = processed_at {
            builder = query!(
                r#"
                UPDATE notifications
                SET status = $1, updated_at = $2, processed_at = $3
                WHERE id = $4
                "#,
                status.to_string(),
                Utc::now(),
                processed_time,
                notification_id,
            );
        } else {
            builder = query!(
                r#"
                UPDATE notifications
                SET status = $1, updated_at = $2
                WHERE id = $3
                "#,
                status.to_string(),
                Utc::now(),
                notification_id,
            );
        }
        
        builder
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Failed to update notification status: {}", e)))?;
            
        Ok(())
    }
    
    pub async fn update_delivery_status(
        &self,
        delivery_id: &Uuid,
        status: NotificationStatus,
        error_message: Option<String>,
        retry_count: Option<u32>,
        next_retry_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        // Build the update statement dynamically based on what's provided
        let mut updates = Vec::new();
        let mut params = vec![status.to_string(), Utc::now().into(), *delivery_id];
        let mut param_index = 3;
        
        updates.push(format!("status = ${}", 1));
        updates.push(format!("updated_at = ${}", 2));
        
        if let Some(error) = error_message {
            updates.push(format!("error_message = ${}", param_index));
            params.push(error.into());
            param_index += 1;
        }
        
        if let Some(retries) = retry_count {
            updates.push(format!("retry_count = ${}", param_index));
            params.push(retries.into());
            param_index += 1;
        }
        
        if let Some(next_retry) = next_retry_at {
            updates.push(format!("next_retry_at = ${}", param_index));
            params.push(next_retry.into());
            param_index += 1;
        }
        
        if let Some(completed) = completed_at {
            updates.push(format!("completed_at = ${}", param_index));
            params.push(completed.into());
        }
        
        let update_clause = updates.join(", ");
        let sql = format!("UPDATE notification_deliveries SET {} WHERE id = $3", update_clause);
        
        sqlx::query(&sql)
            .bind(status.to_string())
            .bind(Utc::now())
            .bind(delivery_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Failed to update delivery status: {}", e)))?;
            
        Ok(())
    }
    
    pub async fn get_pending_deliveries(&self, limit: i64) -> Result<Vec<NotificationDelivery>> {
        let deliveries = sqlx::query_as!(
            NotificationDeliveryRecord,
            r#"
            SELECT 
                id, notification_id, channel, recipient, status, error_message,
                retry_count, next_retry_at, created_at, updated_at, completed_at
            FROM notification_deliveries
            WHERE status = 'pending' AND (next_retry_at IS NULL OR next_retry_at <= NOW())
            ORDER BY created_at ASC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch pending deliveries: {}", e)))?;
        
        // Convert database records to domain objects
        let result = deliveries
            .into_iter()
            .map(|record| NotificationDelivery {
                id: record.id,
                notification_id: record.notification_id,
                channel: NotificationChannel::from(record.channel),
                recipient: record.recipient,
                status: NotificationStatus::from(record.status),
                error_message: record.error_message,
                retry_count: record.retry_count as u32,
                next_retry_at: record.next_retry_at,
                created_at: record.created_at,
                updated_at: record.updated_at,
                completed_at: record.completed_at,
            })
            .collect();
            
        Ok(result)
    }
    
    pub async fn get_notification(&self, notification_id: &Uuid) -> Result<Option<Notification>> {
        let record = sqlx::query_as!(
            NotificationRecord,
            r#"
            SELECT 
                id, type as "type_", subject, content, 
                metadata, status, created_at, updated_at, processed_at
            FROM notifications
            WHERE id = $1
            "#,
            notification_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch notification: {}", e)))?;
        
        if let Some(record) = record {
            // Parse metadata JSON
            let metadata: HashMap<String, String> = serde_json::from_value(record.metadata)
                .unwrap_or_default();
                
            let notification = Notification {
                id: record.id,
                type_: NotificationType::from(record.type_),
                subject: record.subject,
                content: record.content,
                metadata,
                status: NotificationStatus::from(record.status),
                created_at: record.created_at,
                updated_at: record.updated_at,
                processed_at: record.processed_at,
            };
            
            Ok(Some(notification))
        } else {
            Ok(None)
        }
    }
    
    pub async fn get_notification_deliveries(&self, notification_id: &Uuid) -> Result<Vec<NotificationDelivery>> {
        let deliveries = sqlx::query_as!(
            NotificationDeliveryRecord,
            r#"
            SELECT 
                id, notification_id, channel, recipient, status, error_message,
                retry_count, next_retry_at, created_at, updated_at, completed_at
            FROM notification_deliveries
            WHERE notification_id = $1
            "#,
            notification_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch notification deliveries: {}", e)))?;
        
        // Convert database records to domain objects
        let result = deliveries
            .into_iter()
            .map(|record| NotificationDelivery {
                id: record.id,
                notification_id: record.notification_id,
                channel: NotificationChannel::from(record.channel),
                recipient: record.recipient,
                status: NotificationStatus::from(record.status),
                error_message: record.error_message,
                retry_count: record.retry_count as u32,
                next_retry_at: record.next_retry_at,
                created_at: record.created_at,
                updated_at: record.updated_at,
                completed_at: record.completed_at,
            })
            .collect();
            
        Ok(result)
    }
    
    pub async fn create_subscription(&self, subscription: &Subscription) -> Result<Uuid> {
        let id = query!(
            r#"
            INSERT INTO subscriptions
                (id, notification_type, channel, recipient, filter_conditions, active, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#,
            subscription.id,
            subscription.notification_type.to_string(),
            subscription.channel.to_string(),
            subscription.recipient,
            serde_json::to_value(&subscription.filter_conditions)
                .map_err(|e| Error::Internal(format!("Failed to serialize filter conditions: {}", e)))?,
            subscription.active,
            subscription.created_at,
            subscription.updated_at,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create subscription: {}", e)))?
        .id;
        
        Ok(id)
    }
    
    pub async fn get_subscriptions_for_notification_type(
        &self,
        notification_type: &NotificationType,
    ) -> Result<Vec<Subscription>> {
        let type_str = notification_type.to_string();
        
        let subscriptions = sqlx::query_as!(
            SubscriptionRecord,
            r#"
            SELECT 
                id, notification_type, channel, recipient, 
                filter_conditions, active, created_at, updated_at
            FROM subscriptions
            WHERE notification_type = $1 AND active = true
            "#,
            type_str
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch subscriptions: {}", e)))?;
        
        // Convert database records to domain objects
        let result = subscriptions
            .into_iter()
            .map(|record| {
                // Parse filter conditions JSON
                let filter_conditions: HashMap<String, String> = serde_json::from_value(record.filter_conditions)
                    .unwrap_or_default();
                    
                Subscription {
                    id: record.id,
                    notification_type: NotificationType::from(record.notification_type),
                    channel: NotificationChannel::from(record.channel),
                    recipient: record.recipient,
                    filter_conditions,
                    active: record.active,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                }
            })
            .collect();
            
        Ok(result)
    }
}

// Database Record structs
struct NotificationRecord {
    id: Uuid,
    type_: String,
    subject: String,
    content: String,
    metadata: serde_json::Value,
    status: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    processed_at: Option<DateTime<Utc>>,
}

struct NotificationDeliveryRecord {
    id: Uuid,
    notification_id: Uuid,
    channel: String,
    recipient: String,
    status: String,
    error_message: Option<String>,
    retry_count: i32,
    next_retry_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

struct SubscriptionRecord {
    id: Uuid,
    notification_type: String,
    channel: String,
    recipient: String,
    filter_conditions: serde_json::Value,
    active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Helper trait implementations
impl From<String> for NotificationStatus {
    fn from(value: String) -> Self {
        match value.as_str() {
            "pending" => NotificationStatus::Pending,
            "processed" => NotificationStatus::Processed,
            "failed" => NotificationStatus::Failed,
            "canceled" => NotificationStatus::Canceled,
            _ => NotificationStatus::Pending,
        }
    }
}
