use crate::config::AppConfig;
use crate::error::{ScannerError, ScannerResult};
use crate::integrations::IntegrationService;
use crate::models::{Scan, ScanModule, ScanModuleStatus, ScanStatus, ScanTarget, ScanTargetStatus};
use crate::repositories::{ScanRepository, ScanTargetRepository};
use chrono::Utc;
use redis::{AsyncCommands, Client as RedisClient, Commands};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const SCAN_QUEUE_KEY: &str = "mirage:scanner:scan_queue";
const TARGET_QUEUE_PREFIX: &str = "mirage:scanner:target_queue:";

pub struct SchedulerService {
    redis_client: RedisClient,
    scan_repo: ScanRepository,
    target_repo: ScanTargetRepository,
    integration_service: IntegrationService,
    config: AppConfig,
}

impl SchedulerService {
    pub fn new(
        redis_client: RedisClient,
        scan_repo: ScanRepository,
        target_repo: ScanTargetRepository,
        integration_service: IntegrationService,
        config: AppConfig,
    ) -> Self {
        Self {
            redis_client,
            scan_repo,
            target_repo,
            integration_service,
            config,
        }
    }

    /// Enqueue a scan for execution
    pub async fn enqueue_scan(&self, scan_id: Uuid, priority: i32) -> ScannerResult<()> {
        let mut conn = self.redis_client.get_async_connection().await?;

        // Add scan ID to sorted set with priority as score
        // Lower priority numbers = higher priority
        conn.zadd::<_, _, _, i64>(SCAN_QUEUE_KEY, scan_id.to_string(), priority)
            .await?;

        // Update scan status to queued
        self.scan_repo
            .update_scan_status(scan_id, ScanStatus::Queued, None, None, None, None)
            .await?;

        Ok(())
    }

    /// Start processing a scan
    pub async fn start_scan(&self, scan_id: Uuid) -> ScannerResult<()> {
        // Get scan details
        let scan = match self.scan_repo.get_scan_by_id(scan_id).await? {
            Some(scan) => scan,
            None => {
                return Err(ScannerError::NotFound(format!(
                    "Scan {} not found",
                    scan_id
                )))
            }
        };

        // Check if scan can be started
        if scan.status != ScanStatus::Created && scan.status != ScanStatus::Queued {
            return Err(ScannerError::Validation(format!(
                "Cannot start scan in status: {}",
                serde_json::to_string(&scan.status).unwrap()
            )));
        }

        // Get scan targets
        let targets = self.target_repo.get_targets_for_scan(scan_id).await?;
        if targets.is_empty() {
            return Err(ScannerError::Validation("Scan has no targets".into()));
        }

        // Update scan status to running
        self.scan_repo
            .update_scan_status(
                scan_id,
                ScanStatus::Running,
                Some(Utc::now()),
                None,
                Some(0), // Initial progress
                None,
            )
            .await?;

        // Enqueue targets for processing
        self.enqueue_scan_targets(scan_id, &targets).await?;

        Ok(())
    }

    /// Enqueue targets for a scan
    async fn enqueue_scan_targets(
        &self,
        scan_id: Uuid,
        targets: &[ScanTarget],
    ) -> ScannerResult<()> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let queue_key = format!("{}{}", TARGET_QUEUE_PREFIX, scan_id);

        // Add target IDs to scan's target queue
        for target in targets {
            conn.rpush::<_, _, i64>(&queue_key, target.id.to_string())
                .await?;
        }

        Ok(())
    }

    /// Get the next target to process
    pub async fn get_next_target(&self, scan_id: Uuid) -> ScannerResult<Option<ScanTarget>> {
        let mut conn = self.redis_client.get_async_connection().await?;
        let queue_key = format!("{}{}", TARGET_QUEUE_PREFIX, scan_id);

        // Get and remove first target from queue
        let target_id: Option<String> = conn.lpop(&queue_key).await?;

        if let Some(target_id_str) = target_id {
            // Parse target ID
            let target_id = match Uuid::parse_str(&target_id_str) {
                Ok(id) => id,
                Err(_) => {
                    return Err(ScannerError::Internal(format!(
                        "Invalid target ID in queue: {}",
                        target_id_str
                    )))
                }
            };

            // Get target details from database
            let targets = self.target_repo.get_targets_for_scan(scan_id).await?;
            let target = targets.into_iter().find(|t| t.id == target_id);

            Ok(target)
        } else {
            // Queue is empty
            Ok(None)
        }
    }

    /// Process a target against available modules
    pub async fn process_target(&self, target: &ScanTarget, module_id: Uuid) -> ScannerResult<()> {
        // Update target status to in-progress
        self.target_repo
            .update_target_status(
                target.id,
                ScanTargetStatus::InProgress,
                Some(Utc::now()),
                None,
                None,
                None,
            )
            .await?;

        // Create collection task
        let task_id = self
            .integration_service
            .create_collection_task(
                target, module_id, None, // parameters
                None, // priority
            )
            .await?;

        // Store task ID in target metadata (in a real system we would have a better data model)
        // This is a simplification for this example
        let mut metadata = target.metadata.clone();
        metadata.insert("collection_task_id".to_string(), task_id.to_string());

        // Update target status to completed
        // In a real implementation, we would track task completion asynchronously
        self.target_repo
            .update_target_status(
                target.id,
                ScanTargetStatus::Completed,
                None,
                Some(Utc::now()),
                None,
                Some(1), // result_count
            )
            .await?;

        Ok(())
    }

    /// Complete a scan
    pub async fn complete_scan(
        &self,
        scan_id: Uuid,
        success: bool,
        error_message: Option<String>,
    ) -> ScannerResult<()> {
        let status = if success {
            ScanStatus::Completed
        } else {
            ScanStatus::Failed
        };

        self.scan_repo
            .update_scan_status(
                scan_id,
                status,
                None,
                Some(Utc::now()),
                Some(100), // 100% progress
                error_message,
            )
            .await?;

        Ok(())
    }

    /// Check if scan is complete (all targets processed)
    pub async fn check_scan_completion(&self, scan_id: Uuid) -> ScannerResult<bool> {
        let targets = self.target_repo.get_targets_for_scan(scan_id).await?;

        // Check if all targets are either completed or failed
        let all_processed = targets.iter().all(|t| {
            t.status == ScanTargetStatus::Completed
                || t.status == ScanTargetStatus::Failed
                || t.status == ScanTargetStatus::Skipped
        });

        // Calculate completion percentage
        let total = targets.len();
        let completed = targets
            .iter()
            .filter(|t| {
                t.status == ScanTargetStatus::Completed
                    || t.status == ScanTargetStatus::Failed
                    || t.status == ScanTargetStatus::Skipped
            })
            .count();

        if total > 0 {
            let progress = (completed * 100) / total;

            // Update scan progress
            self.scan_repo
                .update_scan_status(
                    scan_id,
                    if all_processed {
                        ScanStatus::Completed
                    } else {
                        ScanStatus::Running
                    },
                    None,
                    if all_processed {
                        Some(Utc::now())
                    } else {
                        None
                    },
                    Some(progress as i32),
                    None,
                )
                .await?;
        }

        Ok(all_processed)
    }
}

/// Background scheduler process
pub async fn run_scheduler(scheduler: SchedulerService, config: AppConfig) {
    tracing::info!("Starting scan scheduler");

    let interval = config.scheduler.interval_seconds;
    loop {
        // Process pending scans
        if let Err(e) = process_pending_scans(&scheduler).await {
            tracing::error!("Error processing pending scans: {}", e);
        }

        // Sleep before next check
        time::sleep(Duration::from_secs(interval)).await;
    }
}

/// Process pending scans in the queue
async fn process_pending_scans(scheduler: &SchedulerService) -> ScannerResult<()> {
    let mut conn = scheduler.redis_client.get_async_connection().await?;

    // Get highest priority scan from queue
    let scan_id: Option<String> = conn.zpopmin(SCAN_QUEUE_KEY).await?;

    if let Some(scan_id_str) = scan_id {
        match Uuid::parse_str(&scan_id_str) {
            Ok(scan_id) => {
                // Start the scan
                if let Err(e) = scheduler.start_scan(scan_id).await {
                    tracing::error!("Failed to start scan {}: {}", scan_id, e);
                }

                // Process scan targets
                if let Err(e) = process_scan_targets(scheduler, scan_id).await {
                    tracing::error!("Failed to process scan targets for scan {}: {}", scan_id, e);

                    // Mark scan as failed
                    let _ = scheduler
                        .complete_scan(
                            scan_id,
                            false,
                            Some(format!("Failed to process scan targets: {}", e)),
                        )
                        .await;
                }
            }
            Err(e) => {
                tracing::error!("Invalid scan ID in queue: {}, error: {}", scan_id_str, e);
            }
        }
    }

    Ok(())
}

/// Process targets for a scan
async fn process_scan_targets(scheduler: &SchedulerService, scan_id: Uuid) -> ScannerResult<()> {
    let mut success = true;

    // Get modules for this scan
    let modules = scheduler
        .scan_repo
        .get_scan_by_id(scan_id)
        .await?
        .map(|s| {
            // In a real implementation, we would get modules from the database
            vec![Uuid::new_v4()] // Placeholder module ID
        })
        .unwrap_or_default();

    if modules.is_empty() {
        return Err(ScannerError::Validation(
            "No modules configured for scan".into(),
        ));
    }

    // Process each target against each module
    loop {
        // Get next target from queue
        let target = match scheduler.get_next_target(scan_id).await? {
            Some(target) => target,
            None => break, // No more targets
        };

        // Process target against each module
        for module_id in &modules {
            if let Err(e) = scheduler.process_target(&target, *module_id).await {
                tracing::error!(
                    "Failed to process target {} with module {}: {}",
                    target.id,
                    module_id,
                    e
                );
                success = false;
            }
        }
    }

    // Check if scan is complete
    let is_complete = scheduler.check_scan_completion(scan_id).await?;

    if is_complete {
        // Mark scan as complete
        scheduler.complete_scan(scan_id, success, None).await?;
    }

    Ok(())
}
