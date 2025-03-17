use crate::models::{
    Integration, Credential, ExecutionRecord, 
    IntegrationType, IntegrationStatus, AuthType, ExecutionStatus, ScheduleType
};
use crate::error::{IntegrationError, IntegrationResult};
use crate::config::DatabaseConfig;
use crate::crypto::CryptoService;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, query, query_as};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

pub type DbPool = Pool<Postgres>;

/// Create database connection pool
pub async fn create_db_pool(config: &DatabaseConfig) -> IntegrationResult<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

#[derive(Clone)]
pub struct IntegrationRepository {
    pool: DbPool,
}

impl IntegrationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    // Create a new integration
    pub async fn create_integration(&self, integration: &Integration) -> IntegrationResult<()> {
        query!(
            r#"
            INSERT INTO integrations (
                id, name, description, integration_type, provider_id, status,
                config, created_at, updated_at, created_by, tags,
                schedule_type, schedule_config, last_execution, next_execution,
                error_message, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
            integration.id,
            integration.name,
            integration.description,
            integration.integration_type as _,
            integration.provider_id,
            integration.status as _,
            integration.config,
            integration.created_at,
            integration.updated_at,
            integration.created_by,
            &integration.tags,
            integration.schedule_type as _,
            integration.schedule_config,
            integration.last_execution,
            integration.next_execution,
            integration.error_message,
            serde_json::to_value(&integration.metadata)?,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Get integration by ID
    pub async fn get_integration_by_id(&self, id: &Uuid) -> IntegrationResult<Option<Integration>> {
        let row = query!(
            r#"
            SELECT 
                id, name, description, integration_type as "integration_type: IntegrationType", 
                provider_id, status as "status: IntegrationStatus", 
                config, created_at, updated_at, created_by, tags,
                schedule_type as "schedule_type: ScheduleType", schedule_config, 
                last_execution, next_execution, error_message, metadata
            FROM integrations
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
                
                Ok(Some(Integration {
                    id: r.id,
                    name: r.name,
                    description: r.description,
                    integration_type: r.integration_type,
                    provider_id: r.provider_id,
                    status: r.status,
                    config: r.config,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    created_by: r.created_by,
                    tags: r.tags,
                    schedule_type: r.schedule_type,
                    schedule_config: r.schedule_config,
                    last_execution: r.last_execution,
                    next_execution: r.next_execution,
                    error_message: r.error_message,
                    metadata,
                }))
            },
            None => Ok(None),
        }
    }
    
    // Update integration
    pub async fn update_integration(&self, integration: &Integration) -> IntegrationResult<()> {
        query!(
            r#"
            UPDATE integrations
            SET 
                name = $1,
                description = $2,
                status = $3,
                config = $4,
                updated_at = $5,
                tags = $6,
                schedule_type = $7,
                schedule_config = $8,
                last_execution = $9,
                next_execution = $10,
                error_message = $11,
                metadata = $12
            WHERE id = $13
            "#,
            integration.name,
            integration.description,
            integration.status as _,
            integration.config,
            integration.updated_at,
            &integration.tags,
            integration.schedule_type as _,
            integration.schedule_config,
            integration.last_execution,
            integration.next_execution,
            integration.error_message,
            serde_json::to_value(&integration.metadata)?,
            integration.id,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Delete integration
    pub async fn delete_integration(&self, id: &Uuid) -> IntegrationResult<()> {
        query!("DELETE FROM integrations WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    // List integrations with filtering
    pub async fn list_integrations(
        &self,
        integration_type: Option<&IntegrationType>,
        provider_id: Option<&str>,
        status: Option<&IntegrationStatus>,
        tag: Option<&str>,
        name_contains: Option<&str>,
        page: u64,
        per_page: u64
    ) -> IntegrationResult<(Vec<Integration>, u64)> {
        // Query with filtering and pagination
        let offset = (page - 1) * per_page;
        
        let rows = query!(
            r#"
            SELECT 
                id, name, description, integration_type as "integration_type: IntegrationType", 
                provider_id, status as "status: IntegrationStatus", 
                config, created_at, updated_at, created_by, tags,
                schedule_type as "schedule_type: ScheduleType", schedule_config, 
                last_execution, next_execution, error_message, metadata
            FROM integrations
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut integrations = Vec::with_capacity(rows.len());
        for r in rows {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            integrations.push(Integration {
                id: r.id,
                name: r.name,
                description: r.description,
                integration_type: r.integration_type,
                provider_id: r.provider_id,
                status: r.status,
                config: r.config,
                created_at: r.created_at,
                updated_at: r.updated_at,
                created_by: r.created_by,
                tags: r.tags,
                schedule_type: r.schedule_type,
                schedule_config: r.schedule_config,
                last_execution: r.last_execution,
                next_execution: r.next_execution,
                error_message: r.error_message,
                metadata,
            });
        }
        
        // Get total count
        let total_count = query!(
            r#"SELECT COUNT(*) as count FROM integrations"#
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        
        Ok((integrations, total_count as u64))
    }
    
    // Get scheduled integrations
    pub async fn get_scheduled_integrations(&self) -> IntegrationResult<Vec<Integration>> {
        let now = Utc::now();
        
        let rows = query!(
            r#"
            SELECT 
                id, name, description, integration_type as "integration_type: IntegrationType", 
                provider_id, status as "status: IntegrationStatus", 
                config, created_at, updated_at, created_by, tags,
                schedule_type as "schedule_type: ScheduleType", schedule_config, 
                last_execution, next_execution, error_message, metadata
            FROM integrations
            WHERE status = 'active'::integration_status 
              AND schedule_type != 'none'::schedule_type
              AND (next_execution IS NULL OR next_execution <= $1)
            ORDER BY next_execution ASC NULLS FIRST
            "#,
            now
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut integrations = Vec::with_capacity(rows.len());
        for r in rows {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            integrations.push(Integration {
                id: r.id,
                name: r.name,
                description: r.description,
                integration_type: r.integration_type,
                provider_id: r.provider_id,
                status: r.status,
                config: r.config,
                created_at: r.created_at,
                updated_at: r.updated_at,
                created_by: r.created_by,
                tags: r.tags,
                schedule_type: r.schedule_type,
                schedule_config: r.schedule_config,
                last_execution: r.last_execution,
                next_execution: r.next_execution,
                error_message: r.error_message,
                metadata,
            });
        }
        
        Ok(integrations)
    }
}

#[derive(Clone)]
pub struct CredentialRepository {
    pool: DbPool,
    crypto: CryptoService,
}

impl CredentialRepository {
    pub fn new(pool: DbPool, crypto: CryptoService) -> Self {
        Self { pool, crypto }
    }
    
    // Create a new credential
    pub async fn create_credential(&self, credential: &Credential) -> IntegrationResult<()> {
        query!(
            r#"
            INSERT INTO credentials (
                id, integration_id, auth_type, name, encrypted_data,
                created_at, updated_at, expires_at, last_used, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            credential.id,
            credential.integration_id,
            credential.auth_type as _,
            credential.name,
            credential.encrypted_data,
            credential.created_at,
            credential.updated_at,
            credential.expires_at,
            credential.last_used,
            serde_json::to_value(&credential.metadata)?
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Get credentials for an integration
    pub async fn get_credentials_for_integration(&self, integration_id: &Uuid) -> IntegrationResult<Vec<Credential>> {
        let rows = query!(
            r#"
            SELECT 
                id, integration_id, auth_type as "auth_type: AuthType", name, encrypted_data,
                created_at, updated_at, expires_at, last_used, metadata
            FROM credentials
            WHERE integration_id = $1
            "#,
            integration_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut credentials = Vec::with_capacity(rows.len());
        for r in rows {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            credentials.push(Credential {
                id: r.id,
                integration_id: r.integration_id,
                auth_type: r.auth_type,
                name: r.name,
                encrypted_data: r.encrypted_data,
                created_at: r.created_at,
                updated_at: r.updated_at,
                expires_at: r.expires_at,
                last_used: r.last_used,
                metadata,
            });
        }
        
        Ok(credentials)
    }
    
    // Get credential by ID
    pub async fn get_credential_by_id(&self, id: &Uuid) -> IntegrationResult<Option<Credential>> {
        let row = query!(
            r#"
            SELECT 
                id, integration_id, auth_type as "auth_type: AuthType", name, encrypted_data,
                created_at, updated_at, expires_at, last_used, metadata
            FROM credentials
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
                
                Ok(Some(Credential {
                    id: r.id,
                    integration_id: r.integration_id,
                    auth_type: r.auth_type,
                    name: r.name,
                    encrypted_data: r.encrypted_data,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    expires_at: r.expires_at,
                    last_used: r.last_used,
                    metadata,
                }))
            },
            None => Ok(None),
        }
    }
    
    // Update last used timestamp
    pub async fn update_credential_last_used(&self, id: &Uuid) -> IntegrationResult<()> {
        let now = Utc::now();
        
        query!(
            r#"
            UPDATE credentials
            SET last_used = $1
            WHERE id = $2
            "#,
            now,
            id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Delete credential
    pub async fn delete_credential(&self, id: &Uuid) -> IntegrationResult<()> {
        query!("DELETE FROM credentials WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    // Encrypt credential data
    pub fn encrypt_data(&self, data: &str) -> IntegrationResult<String> {
        self.crypto.encrypt(data)
    }
    
    // Decrypt credential data
    pub fn decrypt_data(&self, encrypted_data: &str) -> IntegrationResult<String> {
        self.crypto.decrypt(encrypted_data)
    }
}

#[derive(Clone)]
pub struct ExecutionRepository {
    pool: DbPool,
}

impl ExecutionRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    // Create a new execution record
    pub async fn create_execution_record(&self, record: &ExecutionRecord) -> IntegrationResult<()> {
        query!(
            r#"
            INSERT INTO executions (
                id, integration_id, status, started_at, completed_at,
                result_count, error_message, parameters, target, execution_time_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            record.id,
            record.integration_id,
            record.status as _,
            record.started_at,
            record.completed_at,
            record.result_count,
            record.error_message,
            record.parameters,
            record.target,
            record.execution_time_ms
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Update execution record
    pub async fn update_execution_record(&self, record: &ExecutionRecord) -> IntegrationResult<()> {
        query!(
            r#"
            UPDATE executions
            SET status = $1, completed_at = $2, result_count = $3, 
                error_message = $4, execution_time_ms = $5
            WHERE id = $6
            "#,
            record.status as _,
            record.completed_at,
            record.result_count,
            record.error_message,
            record.execution_time_ms,
            record.id
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // Get execution record by ID
    pub async fn get_execution_by_id(&self, id: &Uuid) -> IntegrationResult<Option<ExecutionRecord>> {
        let row = query!(
            r#"
            SELECT 
                id, integration_id, status as "status: ExecutionStatus", started_at, completed_at,
                result_count, error_message, parameters, target, execution_time_ms
            FROM executions
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => {
                Ok(Some(ExecutionRecord {
                    id: r.id,
                    integration_id: r.integration_id,
                    status: r.status,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    result_count: r.result_count,
                    error_message: r.error_message,
                    parameters: r.parameters,
                    target: r.target,
                    execution_time_ms: r.execution_time_ms,
                }))
            },
            None => Ok(None),
        }
    }
    
    // Get recent executions for an integration
    pub async fn get_recent_executions(&self, integration_id: &Uuid, limit: i64) -> IntegrationResult<Vec<ExecutionRecord>> {
        let rows = query!(
            r#"
            SELECT 
                id, integration_id, status as "status: ExecutionStatus", started_at, completed_at,
                result_count, error_message, parameters, target, execution_time_ms
            FROM executions
            WHERE integration_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
            integration_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut executions = Vec::with_capacity(rows.len());
        for r in rows {
            executions.push(ExecutionRecord {
                id: r.id,
                integration_id: r.integration_id,
                status: r.status,
                started_at: r.started_at,
                completed_at: r.completed_at,
                result_count: r.result_count,
                error_message: r.error_message,
                parameters: r.parameters,
                target: r.target,
                execution_time_ms: r.execution_time_ms,
            });
        }
        
        Ok(executions)
    }
}
