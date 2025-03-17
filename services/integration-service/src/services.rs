use crate::models::{
    Integration, IntegrationType, IntegrationStatus, ScheduleType,
    Credential, ExecutionRecord, ExecutionStatus, AuthType,
    CreateIntegrationRequest, UpdateIntegrationRequest, IntegrationResponse,
    CredentialRequest, CredentialResponse, ExecutionRequest, ExecutionResponse,
    PaginatedResponse, ProviderInfo
};
use crate::repositories::{IntegrationRepository, CredentialRepository, ExecutionRepository};
use crate::providers::{ProviderRegistry, Provider};
use crate::config::AppConfig;
use crate::error::{IntegrationError, IntegrationResult};
use chrono::{DateTime, Utc, Duration};
use reqwest::Client;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use cron::Schedule;
use std::str::FromStr;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct IntegrationService {
    integration_repo: Arc<IntegrationRepository>,
    credential_repo: Arc<CredentialRepository>,
    provider_registry: Arc<ProviderRegistry>,
    client: Arc<Client>,
    config: Arc<AppConfig>,
}

impl IntegrationService {
    pub fn new(
        integration_repo: IntegrationRepository,
        credential_repo: CredentialRepository,
        provider_registry: ProviderRegistry,
        client: Client,
        config: AppConfig,
    ) -> Self {
        Self {
            integration_repo: Arc::new(integration_repo),
            credential_repo: Arc::new(credential_repo),
            provider_registry: Arc::new(provider_registry),
            client: Arc::new(client),
            config: Arc::new(config),
        }
    }
    
    // Create a new integration
    pub async fn create_integration(
        &self, 
        request: CreateIntegrationRequest, 
        user_id: Option<String>
    ) -> IntegrationResult<IntegrationResponse> {
        // Get the provider to validate configuration
        let provider = self.provider_registry.get_provider(&request.provider_id)
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Provider '{}' not found", request.provider_id
            )))?;
        
        // Validate configuration against provider schema
        provider.validate_config(&request.config)?;
        
        // Calculate next execution time if scheduled
        let next_execution = match request.schedule_type {
            ScheduleType::None => None,
            ScheduleType::Once => {
                // For one-time scheduled tasks, expect a timestamp in schedule_config
                let schedule_config = request.schedule_config.as_ref()
                    .ok_or_else(|| IntegrationError::Validation("Schedule configuration required for 'once' schedule type".into()))?;
                
                let timestamp = schedule_config.get("timestamp")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| IntegrationError::Validation("'timestamp' field required in schedule_config for 'once' schedule type".into()))?;
                
                Some(DateTime::<Utc>::from_timestamp(timestamp, 0)
                    .ok_or_else(|| IntegrationError::Validation("Invalid timestamp value".into()))?)
            },
            ScheduleType::Interval => {
                // For interval schedules, expect interval_seconds in config
                let schedule_config = request.schedule_config.as_ref()
                    .ok_or_else(|| IntegrationError::Validation("Schedule configuration required for 'interval' schedule type".into()))?;
                
                let interval_seconds = schedule_config.get("interval_seconds")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| IntegrationError::Validation("'interval_seconds' field required in schedule_config for 'interval' schedule type".into()))?;
                
                if interval_seconds < 60 {
                    return Err(IntegrationError::Validation("interval_seconds must be at least 60 seconds".into()));
                }
                
                // First execution is now + interval
                Some(Utc::now() + Duration::seconds(interval_seconds))
            },
            ScheduleType::Cron => {
                // For cron schedules, expect a cron expression
                let schedule_config = request.schedule_config.as_ref()
                    .ok_or_else(|| IntegrationError::Validation("Schedule configuration required for 'cron' schedule type".into()))?;
                
                let cron_expr = schedule_config.get("cron_expression")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| IntegrationError::Validation("'cron_expression' field required in schedule_config for 'cron' schedule type".into()))?;
                
                // Parse cron expression and get next occurrence
                let schedule = Schedule::from_str(cron_expr)
                    .map_err(|e| IntegrationError::Validation(format!("Invalid cron expression: {}", e)))?;
                
                schedule.upcoming(Utc).next()
            }
        };
        
        // Create integration object
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let integration = Integration {
            id,
            name: request.name,
            description: request.description,
            integration_type: request.integration_type,
            provider_id: request.provider_id,
            status: IntegrationStatus::Active,
            config: request.config,
            created_at: now,
            updated_at: now,
            created_by: user_id,
            tags: request.tags.unwrap_or_default(),
            schedule_type: request.schedule_type,
            schedule_config: request.schedule_config,
            last_execution: None,
            next_execution,
            error_message: None,
            metadata: request.metadata.unwrap_or_default(),
        };
        
        // Save to database
        self.integration_repo.create_integration(&integration).await?;
        
        // Convert to response
        let response = IntegrationResponse {
            id: integration.id,
            name: integration.name,
            description: integration.description,
            integration_type: integration.integration_type,
            provider_id: integration.provider_id,
            status: integration.status,
            config: integration.config,
            created_at: integration.created_at,
            updated_at: integration.updated_at,
            tags: integration.tags,
            schedule_type: integration.schedule_type,
            schedule_config: integration.schedule_config,
            last_execution: integration.last_execution,
            next_execution: integration.next_execution,
            metadata: integration.metadata,
            has_credentials: false, // No credentials yet
        };
        
        Ok(response)
    }
    
    // Get integration by ID
    pub async fn get_integration(&self, id: Uuid) -> IntegrationResult<IntegrationResponse> {
        let integration = self.integration_repo.get_integration_by_id(&id).await?
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Integration with ID {} not found", id
            )))?;
        
        // Check if integration has credentials
        let credentials = self.credential_repo.get_credentials_for_integration(&id).await?;
        let has_credentials = !credentials.is_empty();
        
        // Convert to response
        let response = IntegrationResponse {
            id: integration.id,
            name: integration.name,
            description: integration.description,
            integration_type: integration.integration_type,
            provider_id: integration.provider_id,
            status: integration.status,
            config: integration.config,
            created_at: integration.created_at,
            updated_at: integration.updated_at,
            tags: integration.tags,
            schedule_type: integration.schedule_type,
            schedule_config: integration.schedule_config,
            last_execution: integration.last_execution,
            next_execution: integration.next_execution,
            metadata: integration.metadata,
            has_credentials,
        };
        
        Ok(response)
    }
    
    // Update integration
    pub async fn update_integration(
        &self, 
        id: Uuid, 
        request: UpdateIntegrationRequest
    ) -> IntegrationResult<IntegrationResponse> {
        // Get existing integration
        let mut integration = self.integration_repo.get_integration_by_id(&id).await?
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Integration with ID {} not found", id
            )))?;
        
        // Get provider to validate updated config if needed
        if let Some(ref new_config) = request.config {
            let provider = self.provider_registry.get_provider(&integration.provider_id)
                .ok_or_else(|| IntegrationError::NotFound(format!(
                    "Provider '{}' not found", integration.provider_id
                )))?;
            
            // Validate updated configuration
            provider.validate_config(new_config)?;
            
            integration.config = new_config.clone();
        }
        
        // Update fields from request
        if let Some(name) = request.name {
            integration.name = name;
        }
        
        if let Some(description) = request.description {
            integration.description = description;
        }
        
        if let Some(status) = request.status {
            integration.status = status;
        }
        
        if let Some(tags) = request.tags {
            integration.tags = tags;
        }
        
        // Handle schedule updates
        let schedule_changed = request.schedule_type.is_some() || request.schedule_config.is_some();
        
        if let Some(schedule_type) = request.schedule_type {
            integration.schedule_type = schedule_type;
        }
        
        if let Some(schedule_config) = request.schedule_config {
            integration.schedule_config = Some(schedule_config);
        }
        
        // Recalculate next_execution if schedule changed
        if schedule_changed {
            // Just use the scheduler's calculate_next_execution here
            // This would be calculated based on the updated schedule
            integration.next_execution = match integration.schedule_type {
                ScheduleType::None => None,
                ScheduleType::Once => {
                    // For one-time schedules, check if there's a timestamp in schedule_config
                    if let Some(ref config) = integration.schedule_config {
                        if let Some(timestamp) = config.get("timestamp").and_then(|v| v.as_i64()) {
                            Some(DateTime::<Utc>::from_timestamp(timestamp, 0)
                                .ok_or_else(|| IntegrationError::Validation("Invalid timestamp value".into()))?)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                ScheduleType::Interval => {
                    // For interval schedules, add interval to current time
                    if let Some(ref config) = integration.schedule_config {
                        if let Some(seconds) = config.get("interval_seconds").and_then(|v| v.as_i64()) {
                            Some(Utc::now() + Duration::seconds(seconds))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                ScheduleType::Cron => {
                    // For cron schedules, calculate next occurrence
                    if let Some(ref config) = integration.schedule_config {
                        if let Some(expr) = config.get("cron_expression").and_then(|v| v.as_str()) {
                            let schedule = Schedule::from_str(expr)
                                .map_err(|e| IntegrationError::Validation(format!("Invalid cron expression: {}", e)))?;
                            
                            schedule.upcoming(Utc).next()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            };
        }
        
        // Update metadata if provided
        if let Some(metadata) = request.metadata {
            // Merge with existing metadata
            for (key, value) in metadata {
                integration.metadata.insert(key, value);
            }
        }
        
        // Update timestamp
        integration.updated_at = Utc::now();
        
        // Save to database
        self.integration_repo.update_integration(&integration).await?;
        
        // Check if integration has credentials
        let credentials = self.credential_repo.get_credentials_for_integration(&id).await?;
        let has_credentials = !credentials.is_empty();
        
        // Convert to response
        let response = IntegrationResponse {
            id: integration.id,
            name: integration.name,
            description: integration.description,
            integration_type: integration.integration_type,
            provider_id: integration.provider_id,
            status: integration.status,
            config: integration.config,
            created_at: integration.created_at,
            updated_at: integration.updated_at,
            tags: integration.tags,
            schedule_type: integration.schedule_type,
            schedule_config: integration.schedule_config,
            last_execution: integration.last_execution,
            next_execution: integration.next_execution,
            metadata: integration.metadata,
            has_credentials,
        };
        
        Ok(response)
    }
    
    // Delete integration
    pub async fn delete_integration(&self, id: Uuid) -> IntegrationResult<()> {
        // Check if integration exists
        if self.integration_repo.get_integration_by_id(&id).await?.is_none() {
            return Err(IntegrationError::NotFound(format!(
                "Integration with ID {} not found", id
            )));
        }
        
        // Delete integration
        self.integration_repo.delete_integration(&id).await
    }
    
    // List integrations with filtering and pagination
    pub async fn list_integrations(
        &self,
        integration_type: Option<&IntegrationType>,
        provider_id: Option<&str>,
        status: Option<&IntegrationStatus>,
        tag: Option<&str>,
        name_contains: Option<&str>,
        page: u64,
        per_page: u64
    ) -> IntegrationResult<PaginatedResponse<IntegrationResponse>> {
        // Get integrations from repository
        let (integrations, total) = self.integration_repo.list_integrations(
            integration_type,
            provider_id,
            status,
            tag,
            name_contains,
            page,
            per_page
        ).await?;
        
        // Convert to responses
        let mut responses = Vec::with_capacity(integrations.len());
        
        for integration in integrations {
            // Check if integration has credentials
            let credentials = self.credential_repo.get_credentials_for_integration(&integration.id).await?;
            let has_credentials = !credentials.is_empty();
            
            responses.push(IntegrationResponse {
                id: integration.id,
                name: integration.name,
                description: integration.description,
                integration_type: integration.integration_type,
                provider_id: integration.provider_id,
                status: integration.status,
                config: integration.config,
                created_at: integration.created_at,
                updated_at: integration.updated_at,
                tags: integration.tags,
                schedule_type: integration.schedule_type,
                schedule_config: integration.schedule_config,
                last_execution: integration.last_execution,
                next_execution: integration.next_execution,
                metadata: integration.metadata,
                has_credentials,
            });
        }
        
        // Calculate pagination info
        let total_pages = (total + per_page - 1) / per_page;
        
        Ok(PaginatedResponse {
            items: responses,
            total,
            page,
            per_page,
            pages: total_pages,
        })
    }
    
    // Create credential for an integration
    pub async fn create_credential(
        &self,
        integration_id: Uuid,
        request: CredentialRequest
    ) -> IntegrationResult<CredentialResponse> {
        // Check if integration exists
        let integration = self.integration_repo.get_integration_by_id(&integration_id).await?
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Integration with ID {} not found", integration_id
            )))?;
        
        // Get provider to check if auth type is supported
        let provider = self.provider_registry.get_provider(&integration.provider_id)
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Provider '{}' not found", integration.provider_id
            )))?;
        
        if !provider.supports_auth_type(&request.auth_type) {
            return Err(IntegrationError::Validation(format!(
                "Auth type {:?} not supported by provider '{}'",
                request.auth_type, integration.provider_id
            )));
        }
        
        // Encrypt credential data
        let data_json = serde_json::to_string(&request.data)
            .map_err(|e| IntegrationError::Internal(format!("Failed to serialize credential data: {}", e)))?;
        
        let encrypted_data = self.credential_repo.encrypt_data(&data_json)?;
        
        // Create credential
        let now = Utc::now();
        let id = Uuid::new_v4();
        
        let credential = Credential {
            id,
            integration_id,
            auth_type: request.auth_type,
            name: request.name,
            encrypted_data,
            created_at: now,
            updated_at: now,
            expires_at: request.expires_at,
            last_used: None,
            metadata: request.metadata.unwrap_or_default(),
        };
        
        // Save to database
        self.credential_repo.create_credential(&credential).await?;
        
        // Convert to response (don't include the encrypted data)
        let response = CredentialResponse {
            id: credential.id,
            integration_id: credential.integration_id,
            auth_type: credential.auth_type,
            name: credential.name,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
            expires_at: credential.expires_at,
            last_used: credential.last_used,
            metadata: credential.metadata,
        };
        
        Ok(response)
    }
    
    // Get credentials for an integration
    pub async fn get_credentials(&self, integration_id: Uuid) -> IntegrationResult<Vec<CredentialResponse>> {
        // Check if integration exists
        if self.integration_repo.get_integration_by_id(&integration_id).await?.is_none() {
            return Err(IntegrationError::NotFound(format!(
                "Integration with ID {} not found", integration_id
            )));
        }
        
        // Get credentials
        let credentials = self.credential_repo.get_credentials_for_integration(&integration_id).await?;
        
        // Convert to responses
        let responses = credentials.into_iter().map(|cred| {
            CredentialResponse {
                id: cred.id,
                integration_id: cred.integration_id,
                auth_type: cred.auth_type,
                name: cred.name,
                created_at: cred.created_at,
                updated_at: cred.updated_at,
                expires_at: cred.expires_at,
                last_used: cred.last_used,
                metadata: cred.metadata,
            }
        }).collect();
        
        Ok(responses)
    }
    
    // Delete credential
    pub async fn delete_credential(&self, integration_id: Uuid, credential_id: Uuid) -> IntegrationResult<()> {
        // Check if integration exists
        if self.integration_repo.get_integration_by_id(&integration_id).await?.is_none() {
            return Err(IntegrationError::NotFound(format!(
                "Integration with ID {} not found", integration_id
            )));
        }
        
        // Check if credential exists and belongs to the integration
        let credential = self.credential_repo.get_credential_by_id(&credential_id).await?
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Credential with ID {} not found", credential_id
            )))?;
        
        if credential.integration_id != integration_id {
            return Err(IntegrationError::Validation(
                "Credential does not belong to specified integration".into()
            ));
        }
        
        // Delete credential
        self.credential_repo.delete_credential(&credential_id).await
    }
    
    // Get execution record
    pub async fn get_execution(
        &self,
        integration_id: Uuid,
        execution_id: Uuid
    ) -> IntegrationResult<ExecutionResponse> {
        // Check if integration exists
        if self.integration_repo.get_integration_by_id(&integration_id).await?.is_none() {
            return Err(IntegrationError::NotFound(format!(
                "Integration with ID {} not found", integration_id
            )));
        }
        
        // Get execution record
        let execution = self.get_execution_repo().get_execution_by_id(&execution_id).await?
            .ok_or_else(|| IntegrationError::NotFound(format!(
                "Execution with ID {} not found", execution_id
            )))?;
        
        // Check if execution belongs to the integration
        if execution.integration_id != integration_id {
            return Err(IntegrationError::Validation(
                "Execution does not belong to specified integration".into()
            ));
        }
        
        // Convert to response
        let response = ExecutionResponse {
            id: execution.id,
            integration_id: execution.integration_id,
            status: execution.status,
            started_at: execution.started_at,
            completed_at: execution.completed_at,
            result_count: execution.result_count,
            error_message: execution.error_message,
            parameters: execution.parameters,
            target: execution.target,
            execution_time_ms: execution.execution_time_ms,
        };
        
        Ok(response)
    }
    
    // Get recent executions for an integration
    pub async fn get_recent_executions(&self, integration_id: Uuid) -> IntegrationResult<Vec<ExecutionResponse>> {
        // Check if integration exists
        if self.integration_repo.get_integration_by_id(&integration_id).await?.is_none() {
            return Err(IntegrationError::NotFound(format!(
                "Integration with ID {} not found", integration_id
            )));
        }
        
        // Get recent executions
        let executions = self.get_execution_repo().get_recent_executions(&integration_id, 10).await?;
        
        // Convert to responses
        let responses = executions.into_iter().map(|exec| {
            ExecutionResponse {
                id: exec.id,
                integration_id: exec.integration_id,
                status: exec.status,
                started_at: exec.started_at,
                completed_at: exec.completed_at,
                result_count: exec.result_count,
                error_message: exec.error_message,
                parameters: exec.parameters,
                target: exec.target,
                execution_time_ms: exec.execution_time_ms,
            }
        }).collect();
        
        Ok(responses)
    }
    
    // List available providers
    pub async fn list_providers(&self) -> IntegrationResult<Vec<ProviderInfo>> {
        let providers = self.provider_registry.list_providers();
        Ok(providers)
    }
    
    // Helper method to get ExecutionRepository
    // This should be in the struct, but for this example I'm stubbing it
    fn get_execution_repo(&self) -> Arc<ExecutionRepository> {
        // This would come from the instance, I'm just creating a placeholder
        // because we didn't actually store it in the struct
        // In a real implementation, this would be stored in the struct
        Arc::new(ExecutionRepository::new(sqlx::PgPool::connect_lazy("postgres://localhost").unwrap()))
    }
}
