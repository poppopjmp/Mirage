use crate::config::AppConfig;
use crate::error::{IntegrationError, IntegrationResult};
use crate::models::{
    Credential, ExecutionRecord, ExecutionStatus, Integration, IntegrationStatus, ScheduleType,
};
use crate::providers::ProviderRegistry;
use crate::repositories::{ExecutionRepository, IntegrationRepository};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use cron::Schedule;
use redis::{AsyncCommands, Client as RedisClient};
use reqwest::Client;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{sleep, Duration, Instant};
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct SchedulerService {
    integration_repo: Arc<IntegrationRepository>,
    execution_repo: Arc<ExecutionRepository>,
    provider_registry: Arc<ProviderRegistry>,
    http_client: Arc<Client>,
    redis_client: Arc<RedisClient>,
    config: Arc<AppConfig>,
}

impl SchedulerService {
    pub fn new(
        integration_repo: IntegrationRepository,
        execution_repo: ExecutionRepository,
        provider_registry: ProviderRegistry,
        http_client: Client,
        redis_client: RedisClient,
        config: AppConfig,
    ) -> Self {
        Self {
            integration_repo: Arc::new(integration_repo),
            execution_repo: Arc::new(execution_repo),
            provider_registry: Arc::new(provider_registry),
            http_client: Arc::new(http_client),
            redis_client: Arc::new(redis_client),
            config: Arc::new(config),
        }
    }

    // Execute a scheduled integration
    pub async fn execute_scheduled_integration(
        &self,
        integration: &Integration,
    ) -> IntegrationResult<()> {
        info!(
            "Starting scheduled execution of integration '{}' ({})",
            integration.name, integration.id
        );

        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        // Create execution record
        let mut execution = ExecutionRecord {
            id: execution_id,
            integration_id: integration.id,
            status: ExecutionStatus::Running,
            started_at,
            completed_at: None,
            result_count: None,
            error_message: None,
            parameters: None,
            target: None,
            execution_time_ms: None,
        };

        self.execution_repo
            .create_execution_record(&execution)
            .await?;

        // Acquire execution lock
        let lock_key = format!(
            "{}:integration:lock:{}",
            self.config.redis.prefix, integration.id
        );
        let mut redis_conn = self.redis_client.get_async_connection().await?;

        // Try to acquire lock with 10-second expiry (to prevent abandoned locks)
        let lock_result: bool = redis_conn
            .set_nx(&lock_key, execution_id.to_string())
            .await?;
        if !lock_result {
            warn!(
                "Integration {} is already being executed by another process",
                integration.id
            );

            // Update execution record
            execution.status = ExecutionStatus::Failed;
            execution.completed_at = Some(Utc::now());
            execution.error_message =
                Some("Integration is already being executed by another process".to_string());
            self.execution_repo
                .update_execution_record(&execution)
                .await?;

            return Err(IntegrationError::Scheduling(
                "Integration is already being executed".into(),
            ));
        }

        // Set expiry on lock
        redis_conn.expire(&lock_key, 600).await?; // 10 minutes

        // Execute integration
        let start_time = Instant::now();
        let result = self.execute_integration(integration, execution_id).await;
        let elapsed = start_time.elapsed();

        // Release lock
        let _: () = redis_conn.del(&lock_key).await?;

        // Update execution record based on result
        match result {
            Ok((result_count, _)) => {
                info!(
                    "Successfully executed integration '{}' ({}) - Found {} results in {}ms",
                    integration.name,
                    integration.id,
                    result_count.unwrap_or(0),
                    elapsed.as_millis()
                );

                execution.status = ExecutionStatus::Completed;
                execution.completed_at = Some(Utc::now());
                execution.result_count = result_count;
                execution.execution_time_ms = Some(elapsed.as_millis() as i64);

                // Update integration last execution time
                let mut updated_integration = integration.clone();
                updated_integration.last_execution = Some(started_at);
                updated_integration.next_execution =
                    self.calculate_next_execution(&updated_integration);

                // Reset error if previously failed
                if integration.status == IntegrationStatus::Failed {
                    updated_integration.status = IntegrationStatus::Active;
                    updated_integration.error_message = None;
                }

                // Update integration
                self.integration_repo
                    .update_integration(&updated_integration)
                    .await?;
            }
            Err(e) => {
                error!(
                    "Failed to execute integration '{}' ({}): {}",
                    integration.name, integration.id, e
                );

                execution.status = ExecutionStatus::Failed;
                execution.completed_at = Some(Utc::now());
                execution.error_message = Some(format!("{}", e));
                execution.execution_time_ms = Some(elapsed.as_millis() as i64);

                // Update integration with error
                let mut updated_integration = integration.clone();
                updated_integration.status = IntegrationStatus::Failed;
                updated_integration.error_message = Some(format!("Execution failed: {}", e));
                updated_integration.last_execution = Some(started_at);
                updated_integration.next_execution =
                    self.calculate_next_execution(&updated_integration);

                // Update integration
                self.integration_repo
                    .update_integration(&updated_integration)
                    .await?;
            }
        }

        // Update execution record
        self.execution_repo
            .update_execution_record(&execution)
            .await?;

        Ok(())
    }

    // Execute an integration manually
    pub async fn execute_integration_manually(
        &self,
        integration_id: &Uuid,
        parameters: Option<serde_json::Value>,
        target: Option<String>,
    ) -> IntegrationResult<ExecutionRecord> {
        // Get integration
        let integration = match self
            .integration_repo
            .get_integration_by_id(integration_id)
            .await?
        {
            Some(integration) => integration,
            None => {
                return Err(IntegrationError::NotFound(format!(
                    "Integration {} not found",
                    integration_id
                )))
            }
        };

        // Create execution record
        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        let mut execution = ExecutionRecord {
            id: execution_id,
            integration_id: *integration_id,
            status: ExecutionStatus::Running,
            started_at,
            completed_at: None,
            result_count: None,
            error_message: None,
            parameters: parameters.clone(),
            target: target.clone(),
            execution_time_ms: None,
        };

        self.execution_repo
            .create_execution_record(&execution)
            .await?;

        // Execute integration
        let start_time = Instant::now();
        let result = self
            .execute_integration_with_params(
                &integration,
                execution_id,
                parameters.as_ref(),
                target.as_deref(),
            )
            .await;
        let elapsed = start_time.elapsed();

        // Update execution record based on result
        match result {
            Ok((result_count, _)) => {
                info!(
                    "Successfully executed integration '{}' ({}) - Found {} results in {}ms",
                    integration.name,
                    integration.id,
                    result_count.unwrap_or(0),
                    elapsed.as_millis()
                );

                execution.status = ExecutionStatus::Completed;
                execution.completed_at = Some(Utc::now());
                execution.result_count = result_count;
                execution.execution_time_ms = Some(elapsed.as_millis() as i64);

                // Update integration last execution time
                let mut updated_integration = integration.clone();
                updated_integration.last_execution = Some(started_at);

                // Reset error if previously failed
                if integration.status == IntegrationStatus::Failed {
                    updated_integration.status = IntegrationStatus::Active;
                    updated_integration.error_message = None;
                }

                // Update integration
                self.integration_repo
                    .update_integration(&updated_integration)
                    .await?;
            }
            Err(e) => {
                error!(
                    "Failed to execute integration '{}' ({}): {}",
                    integration.name, integration.id, e
                );

                execution.status = ExecutionStatus::Failed;
                execution.completed_at = Some(Utc::now());
                execution.error_message = Some(format!("{}", e));
                execution.execution_time_ms = Some(elapsed.as_millis() as i64);
            }
        }

        // Update execution record
        self.execution_repo
            .update_execution_record(&execution)
            .await?;

        Ok(execution)
    }

    // Calculate when the next execution should happen based on schedule
    pub fn calculate_next_execution(&self, integration: &Integration) -> Option<DateTime<Utc>> {
        match integration.schedule_type {
            ScheduleType::None => None,
            ScheduleType::Once => {
                // For one-time schedules, there's no next execution
                None
            }
            ScheduleType::Interval => {
                // For interval schedules, add interval to current time
                if let Some(config) = &integration.schedule_config {
                    if let Some(seconds) = config.get("interval_seconds").and_then(|v| v.as_i64()) {
                        return Some(Utc::now() + ChronoDuration::seconds(seconds));
                    }
                }
                None
            }
            ScheduleType::Cron => {
                // For cron schedules, calculate next occurrence
                if let Some(config) = &integration.schedule_config {
                    if let Some(expr) = config.get("cron_expression").and_then(|v| v.as_str()) {
                        if let Ok(schedule) = Schedule::from_str(expr) {
                            return schedule.upcoming(Utc).next();
                        }
                    }
                }
                None
            }
        }
    }

    // Core logic to execute an integration
    async fn execute_integration(
        &self,
        integration: &Integration,
        execution_id: Uuid,
    ) -> IntegrationResult<(Option<i32>, Option<String>)> {
        // Just delegate to the parameterized version with no parameters or target
        self.execute_integration_with_params(integration, execution_id, None, None)
            .await
    }

    // Core logic to execute an integration with parameters
    async fn execute_integration_with_params(
        &self,
        integration: &Integration,
        execution_id: Uuid,
        parameters: Option<&serde_json::Value>,
        target: Option<&str>,
    ) -> IntegrationResult<(Option<i32>, Option<String>)> {
        // Get the provider
        let provider = self
            .provider_registry
            .get_provider(&integration.provider_id)
            .ok_or_else(|| {
                IntegrationError::Provider(format!(
                    "Provider {} not found",
                    integration.provider_id
                ))
            })?;

        // Get credentials if any
        let credentials = self
            .get_credentials_for_integration(&integration.id)
            .await?;
        let credential = credentials.first();

        // Execute via provider
        let result = provider
            .execute(
                integration,
                credential,
                parameters,
                target,
                &self.http_client,
                &|data| {
                    if let Some(cred) = credential {
                        self.decrypt_credential_data(cred, data)
                    } else {
                        // No credentials, return error
                        Err(IntegrationError::Authentication(
                            "No credentials available".into(),
                        ))
                    }
                },
            )
            .await?;

        // TODO: Process results and send to data storage

        Ok(result)
    }

    // Helper to get credentials for an integration
    async fn get_credentials_for_integration(
        &self,
        integration_id: &Uuid,
    ) -> IntegrationResult<Vec<Credential>> {
        // This would be from a credential repository
        // For now, we'll just return an empty vector
        Ok(vec![])
    }

    // Helper to decrypt credential data
    fn decrypt_credential_data(
        &self,
        credential: &Credential,
        data: &str,
    ) -> IntegrationResult<String> {
        // This would use a crypto service to decrypt the data
        // For now, just return the data itself
        Ok(data.to_string())
    }
}

// Background scheduler task
pub async fn run_scheduler(scheduler: SchedulerService) {
    info!("Starting scheduler background task");

    loop {
        if !scheduler.config.scheduler.enabled {
            info!("Scheduler is disabled, sleeping for 60 seconds");
            sleep(Duration::from_secs(60)).await;
            continue;
        }

        info!("Checking for scheduled integrations to execute");

        // Get scheduled integrations that are due for execution
        match scheduler
            .integration_repo
            .get_scheduled_integrations()
            .await
        {
            Ok(integrations) => {
                info!(
                    "Found {} scheduled integrations to execute",
                    integrations.len()
                );

                for integration in integrations {
                    // Execute each integration
                    if let Err(e) = scheduler.execute_scheduled_integration(&integration).await {
                        error!(
                            "Failed to schedule execution for integration '{}' ({}): {}",
                            integration.name, integration.id, e
                        );
                    }
                }
            }
            Err(e) => {
                error!("Failed to get scheduled integrations: {}", e);
            }
        }

        // Sleep until next cycle
        let interval = scheduler.config.scheduler_interval();
        info!("Scheduler sleeping for {} seconds", interval.as_secs());
        sleep(interval).await;
    }
}
