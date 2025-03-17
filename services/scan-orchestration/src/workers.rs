use crate::config::AppConfig;
use crate::services::{ScanService, ScanWorkerMessage};
use crate::execution::{ModuleExecutor, ExecutionResult};
use crate::repositories::{ScanRepository, ModuleRepository, ResultRepository};
use mirage_common::Error;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use uuid::Uuid;

pub async fn start_scan_worker(
    mut rx: mpsc::Receiver<ScanWorkerMessage>,
    scan_service: Arc<ScanService>,
    scan_repo: Arc<ScanRepository>,
    module_repo: Arc<ModuleRepository>,
    result_repo: Arc<ResultRepository>,
    config: Arc<AppConfig>,
) {
    tracing::info!("Starting scan worker");
    
    while let Some(message) = rx.recv().await {
        match message {
            ScanWorkerMessage::StartScan(scan_id) => {
                tracing::info!("Worker received StartScan message for scan {}", scan_id);
                tokio::spawn(process_scan(
                    scan_id,
                    scan_service.clone(),
                    scan_repo.clone(),
                    module_repo.clone(),
                    result_repo.clone(),
                    config.clone(),
                ));
            },
            ScanWorkerMessage::CancelScan(scan_id) => {
                tracing::info!("Worker received CancelScan message for scan {}", scan_id);
                // Attempt to cancel the scan
                if let Err(e) = scan_service.try_cancel_active_scan(scan_id).await {
                    tracing::error!("Failed to cancel scan {}: {}", scan_id, e);
                }
            },
            ScanWorkerMessage::Shutdown => {
                tracing::info!("Worker received Shutdown message, terminating");
                break;
            }
        }
    }
    
    tracing::info!("Scan worker terminated");
}

async fn process_scan(
    scan_id: Uuid,
    scan_service: Arc<ScanService>,
    scan_repo: Arc<ScanRepository>,
    module_repo: Arc<ModuleRepository>,
    result_repo: Arc<ResultRepository>,
    config: Arc<AppConfig>,
) {
    tracing::info!("Processing scan {}", scan_id);
    
    // Get scan details
    let scan = match scan_repo.get_scan_by_id(scan_id).await {
        Ok(Some(scan)) => scan,
        Ok(None) => {
            tracing::error!("Scan {} not found", scan_id);
            return;
        },
        Err(e) => {
            tracing::error!("Failed to fetch scan {}: {}", scan_id, e);
            return;
        }
    };
    
    // Mark scan as running
    if let Err(e) = scan_service.start_scan_execution(scan_id).await {
        tracing::error!("Failed to mark scan {} as running: {}", scan_id, e);
        return;
    }
    
    // Create module executor
    let mut executor = match ModuleExecutor::new(
        scan.clone(),
        module_repo.clone(),
        result_repo.clone(),
    ).await {
        Ok(executor) => executor,
        Err(e) => {
            tracing::error!("Failed to create executor for scan {}: {}", scan_id, e);
            // Mark scan as failed
            let _ = scan_service.complete_scan_execution(
                scan_id, 
                false, 
                Some(format!("Failed to start execution: {}", e))
            ).await;
            return;
        }
    };
    
    // Register active scan
    if let Err(e) = scan_service.register_active_scan(scan_id, executor.clone()).await {
        tracing::error!("Failed to register active scan {}: {}", scan_id, e);
        // Continue anyway
    }
    
    // Start execution
    if let Err(e) = executor.start().await {
        tracing::error!("Failed to start scan {}: {}", scan_id, e);
        // Mark scan as failed
        let _ = scan_service.complete_scan_execution(
            scan_id, 
            false, 
            Some(format!("Failed to start execution: {}", e))
        ).await;
        
        // Remove from active scans
        let _ = scan_service.remove_active_scan(scan_id).await;
        return;
    }
    
    // Wait for completion
    // In a real implementation, we would wait for the executor to signal completion
    // For now, we'll use a simple delay as a placeholder
    time::sleep(Duration::from_secs(5)).await;
    
    // Mark scan as complete (success)
    if let Err(e) = scan_service.complete_scan_execution(
        scan_id, 
        true, 
        None
    ).await {
        tracing::error!("Failed to mark scan {} as complete: {}", scan_id, e);
    }
    
    // Remove from active scans
    if let Err(e) = scan_service.remove_active_scan(scan_id).await {
        tracing::error!("Failed to remove active scan {}: {}", scan_id, e);
    }
    
    tracing::info!("Scan {} completed successfully", scan_id);
}
