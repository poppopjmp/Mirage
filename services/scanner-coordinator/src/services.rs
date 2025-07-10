use crate::config::AppConfig;
use crate::error::{ScannerError, ScannerResult};
use crate::integrations::IntegrationService;
use crate::models::{
    CreateScanRequest, CreateTargetRequest, ModuleRequest, Scan, ScanDetailResponse, ScanModule,
    ScanModuleResponse, ScanModuleStatus, ScanResponse, ScanStatus, ScanTarget, ScanTargetResponse,
    ScanTargetStatus, UpdateScanRequest,
};
use crate::repositories::{ScanModuleRepository, ScanRepository, ScanTargetRepository};
use crate::scheduler::SchedulerService;
use chrono::Utc;
use mirage_common::{Error, Result};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ScannerService {
    scan_repo: ScanRepository,
    target_repo: ScanTargetRepository,
    module_repo: ScanModuleRepository,
    scheduler: SchedulerService,
    integration: IntegrationService,
    config: Arc<AppConfig>,
}

impl ScannerService {
    pub fn new(
        scan_repo: ScanRepository,
        target_repo: ScanTargetRepository,
        module_repo: ScanModuleRepository,
        scheduler: SchedulerService,
        integration: IntegrationService,
        config: AppConfig,
    ) -> Self {
        Self {
            scan_repo,
            target_repo,
            module_repo,
            scheduler,
            integration,
            config: Arc::new(config),
        }
    }

    /// Create a new scan
    pub async fn create_scan(
        &self,
        request: CreateScanRequest,
        user_id: Option<Uuid>,
    ) -> Result<ScanResponse> {
        // Validate request
        if request.targets.is_empty() {
            return Err(Error::Validation(
                "At least one target must be specified".into(),
            ));
        }

        if request.modules.is_empty() {
            return Err(Error::Validation(
                "At least one module must be specified".into(),
            ));
        }

        // Generate scan ID
        let scan_id = Uuid::new_v4();

        // Create scan object
        let now = Utc::now();
        let priority = request.priority.unwrap_or(5); // Default priority (1-10)
        let tags = request.tags.unwrap_or_default();
        let metadata = request.metadata.unwrap_or_default();

        // Create scan targets
        let targets = self.create_scan_targets(scan_id, &request.targets).await?;

        // Create scan modules
        let modules = self.create_scan_modules(scan_id, &request.modules).await?;

        // Create scan
        let scan = Scan {
            id: scan_id,
            name: request.name.clone(),
            description: request.description.clone(),
            status: ScanStatus::Created,
            created_by: user_id,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            priority,
            tags: tags.clone(),
            metadata,
            error_message: None,
            progress: None,
            estimated_completion_time: None,
        };

        // Store scan in database
        self.scan_repo
            .create_scan(&scan)
            .await
            .map_err(|e| Error::from(e))?;

        // Store targets in database
        for target in &targets {
            self.target_repo
                .create_target(target)
                .await
                .map_err(|e| Error::from(e))?;
        }

        // Store modules in database
        for module in &modules {
            self.module_repo
                .create_module(module)
                .await
                .map_err(|e| Error::from(e))?;
        }

        // If no schedule is provided, queue scan for immediate execution
        if request.schedule.is_none() {
            self.scheduler
                .enqueue_scan(scan_id, priority)
                .await
                .map_err(|e| Error::from(e))?;
        }

        // Create response
        let response = ScanResponse {
            id: scan_id,
            name: request.name,
            description: request.description,
            status: ScanStatus::Created,
            created_at: now,
            started_at: None,
            completed_at: None,
            target_count: targets.len() as i32,
            completed_targets: 0,
            progress: None,
            tags,
        };

        Ok(response)
    }

    /// Get details of a scan
    pub async fn get_scan(&self, scan_id: Uuid) -> Result<ScanDetailResponse> {
        // Get scan
        let scan = self
            .scan_repo
            .get_scan_by_id(scan_id)
            .await
            .map_err(|e| Error::from(e))?
            .ok_or_else(|| Error::NotFound(format!("Scan with ID {} not found", scan_id)))?;

        // Get targets
        let targets = self
            .target_repo
            .get_targets_for_scan(scan_id)
            .await
            .map_err(|e| Error::from(e))?;

        // Get modules
        let modules = self
            .module_repo
            .get_modules_for_scan(scan_id)
            .await
            .map_err(|e| Error::from(e))?;

        // Count completed targets
        let completed_targets = targets
            .iter()
            .filter(|t| t.status == ScanTargetStatus::Completed)
            .count();

        // Convert to response types
        let target_responses: Vec<ScanTargetResponse> = targets
            .into_iter()
            .map(|t| ScanTargetResponse {
                id: t.id,
                target_type: t.target_type,
                value: t.value,
                status: t.status,
                created_at: t.created_at,
                started_at: t.started_at,
                completed_at: t.completed_at,
                error_message: t.error_message,
                result_count: t.result_count,
                metadata: t.metadata,
            })
            .collect();

        let module_responses: Vec<ScanModuleResponse> = modules
            .into_iter()
            .map(|m| ScanModuleResponse {
                id: m.id,
                module_id: m.module_id,
                module_name: m.module_name,
                module_version: m.module_version,
                status: m.status,
                priority: m.priority,
                depends_on: m.depends_on,
            })
            .collect();

        // Create detail response
        let response = ScanDetailResponse {
            id: scan.id,
            name: scan.name,
            description: scan.description,
            status: scan.status,
            created_by: scan.created_by.map(|id| id.to_string()),
            created_at: scan.created_at,
            updated_at: scan.updated_at,
            started_at: scan.started_at,
            completed_at: scan.completed_at,
            priority: scan.priority,
            tags: scan.tags,
            metadata: scan.metadata,
            error_message: scan.error_message,
            progress: scan.progress,
            estimated_completion_time: scan.estimated_completion_time,
            targets: target_responses,
            modules: module_responses,
        };

        Ok(response)
    }

    /// List scans with optional filtering
    pub async fn list_scans(
        &self,
        status: Option<ScanStatus>,
        created_by: Option<Uuid>,
        tag: Option<String>,
        created_after: Option<chrono::DateTime<Utc>>,
        created_before: Option<chrono::DateTime<Utc>>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<ScanResponse>, u64)> {
        // Query database
        let (scans, total) = self
            .scan_repo
            .list_scans(
                status.as_ref(),
                created_by.as_ref(),
                tag.as_deref(),
                created_after.as_ref(),
                created_before.as_ref(),
                page,
                per_page,
            )
            .await
            .map_err(|e| Error::from(e))?;

        // Convert to response type
        let responses = scans
            .into_iter()
            .map(|s| {
                // Get target count and completed target count
                // In a real implementation, this would be more efficient with a JOIN
                let target_count = 0; // Placeholder
                let completed_targets = 0; // Placeholder

                ScanResponse {
                    id: s.id,
                    name: s.name,
                    description: s.description,
                    status: s.status,
                    created_at: s.created_at,
                    started_at: s.started_at,
                    completed_at: s.completed_at,
                    target_count,
                    completed_targets,
                    progress: s.progress,
                    tags: s.tags,
                }
            })
            .collect();

        Ok((responses, total))
    }

    /// Update an existing scan
    pub async fn update_scan(
        &self,
        scan_id: Uuid,
        request: UpdateScanRequest,
    ) -> Result<ScanResponse> {
        // Get current scan
        let mut scan = self
            .scan_repo
            .get_scan_by_id(scan_id)
            .await
            .map_err(|e| Error::from(e))?
            .ok_or_else(|| Error::NotFound(format!("Scan with ID {} not found", scan_id)))?;

        // Check if scan can be modified
        if scan.status != ScanStatus::Created && scan.status != ScanStatus::Queued {
            return Err(Error::Validation(format!(
                "Cannot update scan in status: {}. Only 'created' or 'queued' scans can be updated.",
                serde_json::to_string(&scan.status).unwrap()
            )));
        }

        // Update fields if provided
        if let Some(name) = request.name {
            scan.name = name;
        }

        if let Some(description) = request.description {
            scan.description = description;
        }

        if let Some(priority) = request.priority {
            scan.priority = priority;
        }

        if let Some(tags) = request.tags {
            scan.tags = tags;
        }

        if let Some(metadata) = request.metadata {
            // Merge metadata rather than replace
            for (k, v) in metadata {
                scan.metadata.insert(k, v);
            }
        }

        scan.updated_at = Utc::now();

        // Save updated scan
        self.scan_repo
            .update_scan(&scan)
            .await
            .map_err(|e| Error::from(e))?;

        // Create response
        // Get target count and completed target count
        // In a real implementation, this would be more efficient with a JOIN
        let targets = self
            .target_repo
            .get_targets_for_scan(scan_id)
            .await
            .map_err(|e| Error::from(e))?;

        let target_count = targets.len() as i32;
        let completed_targets = targets
            .iter()
            .filter(|t| t.status == ScanTargetStatus::Completed)
            .count() as i32;

        let response = ScanResponse {
            id: scan.id,
            name: scan.name,
            description: scan.description,
            status: scan.status,
            created_at: scan.created_at,
            started_at: scan.started_at,
            completed_at: scan.completed_at,
            target_count,
            completed_targets,
            progress: scan.progress,
            tags: scan.tags,
        };

        Ok(response)
    }

    /// Start a scan (if it's in Created or Queued status)
    pub async fn start_scan(&self, scan_id: Uuid) -> Result<ScanResponse> {
        // Get current scan
        let scan = self
            .scan_repo
            .get_scan_by_id(scan_id)
            .await
            .map_err(|e| Error::from(e))?
            .ok_or_else(|| Error::NotFound(format!("Scan with ID {} not found", scan_id)))?;

        // Check if scan can be started
        if scan.status != ScanStatus::Created && scan.status != ScanStatus::Queued {
            return Err(Error::Validation(format!(
                "Cannot start scan in status: {}. Only 'created' or 'queued' scans can be started.",
                serde_json::to_string(&scan.status).unwrap()
            )));
        }

        // Queue the scan for execution
        self.scheduler
            .enqueue_scan(scan_id, scan.priority)
            .await
            .map_err(|e| Error::from(e))?;

        // Return updated scan
        self.get_scan(scan_id).await.map(|detail| ScanResponse {
            id: detail.id,
            name: detail.name,
            description: detail.description,
            status: detail.status,
            created_at: detail.created_at,
            started_at: detail.started_at,
            completed_at: detail.completed_at,
            target_count: detail.targets.len() as i32,
            completed_targets: detail
                .targets
                .iter()
                .filter(|t| t.status == ScanTargetStatus::Completed)
                .count() as i32,
            progress: detail.progress,
            tags: detail.tags,
        })
    }

    /// Cancel a scan
    pub async fn cancel_scan(&self, scan_id: Uuid) -> Result<ScanResponse> {
        // Get current scan
        let scan = self
            .scan_repo
            .get_scan_by_id(scan_id)
            .await
            .map_err(|e| Error::from(e))?
            .ok_or_else(|| Error::NotFound(format!("Scan with ID {} not found", scan_id)))?;

        // Check if scan can be cancelled
        if scan.status != ScanStatus::Created
            && scan.status != ScanStatus::Queued
            && scan.status != ScanStatus::Running
        {
            return Err(Error::Validation(format!(
                "Cannot cancel scan in status: {}. Only 'created', 'queued', or 'running' scans can be cancelled.",
                serde_json::to_string(&scan.status).unwrap()
            )));
        }

        // Update scan status
        self.scan_repo
            .update_scan_status(
                scan_id,
                ScanStatus::Cancelled,
                None,
                Some(Utc::now()),
                None,
                Some("Scan cancelled by user".to_string()),
            )
            .await
            .map_err(|e| Error::from(e))?;

        // Return updated scan
        self.get_scan(scan_id).await.map(|detail| ScanResponse {
            id: detail.id,
            name: detail.name,
            description: detail.description,
            status: detail.status,
            created_at: detail.created_at,
            started_at: detail.started_at,
            completed_at: detail.completed_at,
            target_count: detail.targets.len() as i32,
            completed_targets: detail
                .targets
                .iter()
                .filter(|t| t.status == ScanTargetStatus::Completed)
                .count() as i32,
            progress: detail.progress,
            tags: detail.tags,
        })
    }

    // Helper methods

    /// Create scan targets from request
    async fn create_scan_targets(
        &self,
        scan_id: Uuid,
        requests: &[CreateTargetRequest],
    ) -> Result<Vec<ScanTarget>> {
        let mut targets = Vec::with_capacity(requests.len());

        for request in requests {
            let target = ScanTarget {
                id: Uuid::new_v4(),
                scan_id,
                target_type: request.target_type.clone(),
                value: request.value.clone(),
                status: ScanTargetStatus::Pending,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                started_at: None,
                completed_at: None,
                error_message: None,
                metadata: request.metadata.clone().unwrap_or_default(),
                result_count: None,
            };

            targets.push(target);
        }

        Ok(targets)
    }

    /// Create scan modules from request
    async fn create_scan_modules(
        &self,
        scan_id: Uuid,
        requests: &[ModuleRequest],
    ) -> Result<Vec<ScanModule>> {
        let mut modules = Vec::with_capacity(requests.len());

        for (index, request) in requests.iter().enumerate() {
            // Fetch module info to validate it exists
            let module_info = self
                .integration
                .get_module_info(&request.module_id)
                .await
                .map_err(|e| Error::from(e))?;

            // Create module
            let module = ScanModule {
                id: Uuid::new_v4(),
                scan_id,
                module_id: request.module_id,
                module_name: module_info.name,
                module_version: module_info.version,
                status: ScanModuleStatus::Enabled,
                parameters: request.parameters.clone().unwrap_or_default(),
                priority: request.priority.unwrap_or((index as i32) + 1),
                depends_on: request.depends_on.clone().unwrap_or_default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            modules.push(module);
        }

        Ok(modules)
    }
}
