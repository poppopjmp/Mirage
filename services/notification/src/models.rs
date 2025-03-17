use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    NewEntity,
    NewRelationship,
    ScanComplete,
    AlertTriggered,
    SystemAlert,
    Custom(String),
}

impl ToString for NotificationType {
    fn to_string(&self) -> String {
        match self {
            NotificationType::NewEntity => "new_entity".to_string(),
            NotificationType::NewRelationship => "new_relationship".to_string(),
            NotificationType::ScanComplete => "scan_complete".to_string(),
            NotificationType::AlertTriggered => "alert_triggered".to_string(),
            NotificationType::SystemAlert => "system_alert".to_string(),
            NotificationType::Custom(name) => format!("custom_{}", name),
        }
    }
}

impl From<String> for NotificationType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "new_entity" => NotificationType::NewEntity,
            "new_relationship" => NotificationType::NewRelationship,
            "scan_complete" => NotificationType::ScanComplete,
            "alert_triggered" => NotificationType::AlertTriggered,
            "system_alert" => NotificationType::SystemAlert,
            _ => {
                if value.starts_with("custom_") {
                    NotificationType::Custom(value.replace("custom_", ""))
                } else {
                    NotificationType::Custom(value)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationChannel {
    Email,
    Webhook,
    Slack,
    Database,
}

impl ToString for NotificationChannel {
    fn to_string(&self) -> String {
        match self {
            NotificationChannel::Email => "email".to_string(),
            NotificationChannel::Webhook => "webhook".to_string(),
            NotificationChannel::Slack => "slack".to_string(),
            NotificationChannel::Database => "database".to_string(),
        }
    }
}

impl From<String> for NotificationChannel {
    fn from(value: String) -> Self {
        match value.as_str() {
            "email" => NotificationChannel::Email,
            "webhook" => NotificationChannel::Webhook,
            "slack" => NotificationChannel::Slack,
            "database" => NotificationChannel::Database,
            _ => NotificationChannel::Database, // Default to database for unknown channels
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationStatus {
    Pending,
    Processed,
    Failed,
    Canceled,
}

impl ToString for NotificationStatus {
    fn to_string(&self) -> String {
        match self {
            NotificationStatus::Pending => "pending".to_string(),
            NotificationStatus::Processed => "processed".to_string(),
            NotificationStatus::Failed => "failed".to_string(),
            NotificationStatus::Canceled => "canceled".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub type_: NotificationType,
    pub subject: String,
    pub content: String,
    pub metadata: HashMap<String, String>,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationDelivery {
    pub id: Uuid,
    pub notification_id: Uuid,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    pub name: String,
    pub description: String,
    pub subject_template: String,
    pub content_template: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationRequest {
    pub notification_type: NotificationType,
    pub channels: Vec<NotificationChannelRequest>,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: Option<HashMap<String, String>>,
    pub template_name: Option<String>,
    pub custom_subject: Option<String>,
    pub custom_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelRequest {
    pub channel: NotificationChannel,
    pub recipient: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notification_id: Uuid,
    pub status: NotificationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationStatusResponse {
    pub notification_id: Uuid,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub deliveries: Vec<DeliveryStatusResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryStatusResponse {
    pub delivery_id: Uuid,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub status: NotificationStatus,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub filter_conditions: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub notification_type: NotificationType,
    pub channel: NotificationChannel,
    pub recipient: String,
    pub filter_conditions: HashMap<String, String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub subscription_id: Uuid,
}
