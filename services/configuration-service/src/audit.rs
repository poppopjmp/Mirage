use crate::models::AuditLog;
use crate::repositories::AuditRepository;
use chrono::Utc;
use mirage_common::Result;
use serde_json::Value;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    repo: AuditRepository,
}

impl AuditService {
    pub fn new(repo: AuditRepository) -> Self {
        Self { repo }
    }

    // Log creation action
    pub async fn log_create(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        user_id: Option<&str>,
        details: &Value,
    ) -> Result<()> {
        self.log_action("create", entity_type, entity_id, user_id, details, None)
            .await
    }

    // Log update action
    pub async fn log_update(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        user_id: Option<&str>,
        details: &Value,
        summary: Option<String>,
    ) -> Result<()> {
        self.log_action("update", entity_type, entity_id, user_id, details, summary)
            .await
    }

    // Log delete action
    pub async fn log_delete(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        user_id: Option<&str>,
        details: &Value,
    ) -> Result<()> {
        self.log_action("delete", entity_type, entity_id, user_id, details, None)
            .await
    }

    // Generic action logging
    async fn log_action(
        &self,
        action: &str,
        entity_type: &str,
        entity_id: &Uuid,
        user_id: Option<&str>,
        details: &Value,
        change_summary: Option<String>,
    ) -> Result<()> {
        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            action: action.to_string(),
            entity_type: entity_type.to_string(),
            entity_id: *entity_id,
            user_id: user_id.map(String::from),
            timestamp: Utc::now(),
            details: details.clone(),
            change_summary,
            service: Some("configuration-service".to_string()),
        };

        self.repo.create_audit_log(&audit_log).await
    }

    // Get audit logs for entity
    pub async fn get_audit_logs_for_entity(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        limit: u64,
    ) -> Result<Vec<AuditLog>> {
        self.repo
            .get_logs_for_entity(entity_type, entity_id, limit)
            .await
    }
}
