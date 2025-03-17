use crate::config::AppConfig;
use crate::models::{
    Scan, ScanTarget, ScanModule, ScanStatus, ScanTargetStatus, ScanModuleStatus,
    CreateScanRequest, UpdateScanRequest, ScanModuleResultsSummary, ScanModuleResult,
    CreateScanTargetRequest, CreateScanModuleRequest
};
use crate::repositories::{ScanRepository, ModuleRepository, ResultRepository};
use crate::execution::ModuleExecutor;
use mirage_common::{Error, Result};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

// Channel messages for scan worker communication
#[derive(Debug)]
pub enum ScanWorkerMessage {
    StartScan(Uuid),
    CancelScan(Uuid),
    Shutdown,
}

#[derive(Clone)]
pub struct ScanService {
    scan_repo: Arc<ScanRepository>,
    module_repo: Arc<ModuleRepository>,
    result_repo: Arc<ResultRepository>,
    config: Arc<AppConfig>,
    scan_tx: mpsc::Sender<ScanWorkerMessage>,
    active_scans: Arc<RwLock<HashMap<Uuid, ModuleExecutor>>>,
}

impl ScanService {
    pub fn new(
        scan_repo: ScanRepository, 
        module_repo: ModuleRepository,
        result_repo: ResultRepository,
        config: AppConfig,
        scan_tx: mpsc::Sender<ScanWorkerMessage>,
    ) -> Self {
        Self {
            scan_repo: Arc::new(scan_repo),
            module_repo: Arc::new(module_repo),
            result_repo: Arc::new(result_repo),
            config: Arc::new(config),
            scan_tx,
            active_scans: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // Create a new scan
    pub async fn create_scan(&self, request: CreateScanRequest, user_id: Uuid) -> Result<Scan> {
        // Validate request
        if request.targets.is_empty() {
            return Err(Error::Validation("At least one target must be specified".into()));
        }
        
        if request.modules.is_empty() {
            return Err(Error::Validation("At least one module must be specified".into()));
        }
        
        // Create scan ID
        let scan_id = Uuid::new_v4();
        
        // Set up default values
        let now = Utc::now();
        let priority = request.priority.unwrap_or(5); // Default priority (1-10)
        let tags = request.tags.unwrap_or_default();
        let metadata = request.metadata.unwrap_or_default();
        let max_duration = request.max_duration_minutes;
        
        // Determine initial status (Created or Scheduled)
        let status = if request.scheduled_at.is_some() {
            ScanStatus::Scheduled
        } else {
            ScanStatus::Created
        };
        
        // Create scan targets
        let targets = self.create_scan_targets(scan_id, &request.targets).await?;
        
        // Create scan modules
        let modules = self.create_scan_modules(scan_id, &request.modules).await?;
        
        // Create scan object
        let scan = Scan {
            id: scan_id,
            name: request.name,
            description: request.description,
            created_by: user_id,
            created_at: now,
            updated_at: now,
            status,
            scheduled_at: request.scheduled_at,
            started_at: None,
            completed_at: None,
            targets,
            modules,
            priority,
            tags,
            metadata,
            error_message: None,
            max_duration_minutes: max_duration,
        };
        
        // Save scan to repository
        self.scan_repo.create_scan(&scan).await?;
        
        // If scan is not scheduled for later, queue it for immediate execution
        if request.scheduled_at.is_none() {
            self.scan_tx.send(ScanWorkerMessage::StartScan(scan_id)).await
                .map_err(|e| Error::Internal(format!("Failed to queue scan: {}", e)))?;
        }
        
        Ok(scan)
    }
    
    // Update an existing scan (only allowed if scan is not yet running)
    pub async fn update_scan(&self, scan_id: Uuid, request: UpdateScanRequest) -> Result<Scan> {
        // Get current scan
        let mut scan = self.get_scan(scan_id).await?;
        
        // Check if scan can be modified
        if scan.status != ScanStatus::Created && scan.status != ScanStatus::Scheduled {
            return Err(Error::Validation(format!(
                "Cannot update scan in status: {}. Only Created or Scheduled scans can be updated.", 
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
        
        if let Some(scheduled_at) = request.scheduled_at {
            scan.scheduled_at = Some(scheduled_at);
            // If the scan was Created, change to Scheduled
            if scan.status == ScanStatus::Created {
                scan.status = ScanStatus::Scheduled;
            }
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
        
        if let Some(max_duration) = request.max_duration_minutes {
            scan.max_duration_minutes = Some(max_duration);
        }
        
        scan.updated_at = Utc::now();
        
        // Save updated scan
        self.scan_repo.update_scan(&scan).await?;
        
        Ok(scan)
    }
    
    // Get a scan by ID
    pub async fn get_scan(&self, scan_id: Uuid) -> Result<Scan> {
        self.scan_repo.get_scan_by_id(scan_id).await?
            .ok_or_else(|| Error::NotFound(format!("Scan with ID {} not found", scan_id)))
    }
    
    // List scans with optional filtering
    pub async fn list_scans(
        &self,
        status: Option<ScanStatus>,
        created_by: Option<Uuid>,
        tag: Option<&str>,
        created_after: Option<DateTime<Utc>>,
        created_before: Option<DateTime<Utc>>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<Scan>, u64)> {
        self.scan_repo.list_scans(
            status.as_ref(),
            created_by.as_ref(),
            tag,
            created_after.as_ref(),
            created_before.as_ref(),
            page,
            per_page
        ).await
    }
    
    // Start a scan (if it's in Created or Scheduled status)
    pub async fn start_scan(&self, scan_id: Uuid) -> Result<Scan> {
        // Get current scan
        let scan = self.get_scan(scan_id).await?;
        
        // Check if scan can be started
        if scan.status != ScanStatus::Created && scan.status != ScanStatus::Scheduled {
            return Err(Error::Validation(format!(
                "Cannot start scan in status: {}. Only Created or Scheduled scans can be started.", 
                serde_json::to_string(&scan.status).unwrap()
            )));
        }
        
        // Queue the scan for execution
        self.scan_tx.send(ScanWorkerMessage::StartScan(scan_id)).await
            .map_err(|e| Error::Internal(format!("Failed to queue scan: {}", e)))?;
        
        // Return updated scan
        self.get_scan(scan_id).await
    }
    
    // Cancel a running scan
    pub async fn cancel_scan(&self, scan_id: Uuid) -> Result<Scan> {
        // Get current scan
        let scan = self.get_scan(scan_id).await?;
        
        // Check if scan can be cancelled
        if scan.status != ScanStatus::Running && scan.status != ScanStatus::Scheduled {
            return Err(Error::Validation(format!(
                "Cannot cancel scan in status: {}. Only Running or Scheduled scans can be cancelled.", 
                serde_json::to_string(&scan.status).unwrap()
            )));
        }
        
        // Send cancellation message
        self.scan_tx.send(ScanWorkerMessage::CancelScan(scan_id)).await
            .map_err(|e| Error::Internal(format!("Failed to cancel scan: {}", e)))?;
        
        // For scheduled scans, update status directly
        if scan.status == ScanStatus::Scheduled {
            self.scan_repo.update_scan_status(
                scan_id, 
                ScanStatus::Cancelled,
                None,
                None,
                None,
                Some("Scan cancelled before execution".to_string())
            ).await?;
        }
        
        // Return updated scan
        self.get_scan(scan_id).await
    }
    
    // Get scan results 
    pub async fn get_scan_results(&self, scan_id: Uuid) -> Result<Vec<ScanModuleResult>> {
        self.result_repo.get_results_for_scan(scan_id).await
    }
    
    // Helper methods
    
    // Create scan targets from requests
    async fn create_scan_targets(&self, scan_id: Uuid, target_requests: &[CreateScanTargetRequest]) -> Result<Vec<ScanTarget>> {
        let mut targets = Vec::with_capacity(target_requests.len());
        
        for request in target_requests {
            let target = ScanTarget {
                id: Uuid::new_v4(),
                scan_id,
                target_type: request.target_type.clone(),
                value: request.value.clone(),
                created_at: Utc::now(),
                processed_at: None,
                status: ScanTargetStatus::Pending,
                error_message: None,
            };
            
            targets.push(target);
        }
        
        Ok(targets)
    }
    
    // Create scan modules from requests
    async fn create_scan_modules(&self, scan_id: Uuid, module_requests: &[CreateScanModuleRequest]) -> Result<Vec<ScanModule>> {
        let mut modules = Vec::with_capacity(module_requests.len());
        
        // Sort by order if specified, otherwise use request order
        let mut ordered_requests = module_requests.to_vec();
        ordered_requests.sort_by_key(|req| req.order.unwrap_or(0));
        
        for (idx, request) in ordered_requests.iter().enumerate() {
            // Get module info from registry
            let module_info = self.module_repo.get_module_info(&request.module_id).await?
                .ok_or_else(|| Error::NotFound(format!("Module with ID {} not found", request.module_id)))?;
                
            // Create scan module
            let module = ScanModule {
                id: Uuid::new_v4(),
                scan_id,
                module_id: request.module_id,
                module_name: module_info.name,
                module_version: module_info.version,
                order: request.order.unwrap_or(idx as i32),
                parameters: request.parameters.clone().unwrap_or_default(),
                created_at: Utc::now(),
                started_at: None,
                completed_at: None,
                status: ScanModuleStatus::Pending,
                results_summary: None,
                error_message: None,
            };
            
            modules.push(module);
        }
        
        // Sort by order
        modules.sort_by_key(|m| m.order);
        
        Ok(modules)
    }
    
    // Methods for scan worker to update scan status
    
    pub(crate) async fn start_scan_execution(&self, scan_id: Uuid) -> Result<()> {
        // Update scan status to Running
        self.scan_repo.update_scan_status(
            scan_id, 
            ScanStatus::Running,
            None,
            Some(Utc::now()),
            None,
            None
        ).await?;
        
        Ok(())
    }
    
    pub(crate) async fn complete_scan_execution(&self, scan_id: Uuid, success: bool, error_message: Option<String>) -> Result<()> {
        let status = if success { ScanStatus::Completed } else { ScanStatus::Failed };
        
        self.scan_repo.update_scan_status(
            scan_id, 
            status,
            None,
            None,
            Some(Utc::now()),
            error_message
        ).await?;
        
        Ok(())
    }
    
    pub(crate) async fn register_active_scan(&self, scan_id: Uuid, executor: ModuleExecutor) -> Result<()> {
        // Save executor in active scans
        let mut active_scans = self.active_scans.write().await;
        active_scans.insert(scan_id, executor);
        
        Ok(())
    }
    
    pub(crate) async fn remove_active_scan(&self, scan_id: Uuid) -> Result<()> {
        // Remove from active scans
        let mut active_scans = self.active_scans.write().await;
        active_scans.remove(&scan_id);
        
        Ok(())
    }
    
    pub(crate) async fn try_cancel_active_scan(&self, scan_id: Uuid) -> Result<bool> {
        // Get the executor for the scan
        let active_scans = self.active_scans.read().await;
        
        if let Some(executor) = active_scans.get(&scan_id) {
            executor.cancel().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
