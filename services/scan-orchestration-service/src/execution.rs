use crate::models::{Scan, ScanModule, ScanTarget, ScanModuleStatus, ScanTargetStatus, ScanModuleResult};
use crate::repositories::{ModuleRepository, ResultRepository};
use mirage_common::{Error, Result};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::{self, Duration};
use uuid::Uuid;
use chrono::Utc;

// Message types for module execution
pub enum ExecutionMessage {
    Start,
    Cancel,
}

// Result types from module execution
pub enum ExecutionResult {
    Success(Vec<ScanModuleResult>),
    Failure(String),
    Cancelled,
}

pub struct ModuleExecutor {
    tx: mpsc::Sender<ExecutionMessage>,
    cancel_tx: Option<oneshot::Sender<()>>,
}

impl ModuleExecutor {
    pub async fn new(
        scan: Scan,
        module_repo: Arc<ModuleRepository>,
        result_repo: Arc<ResultRepository>,
    ) -> Result<Self> {
        let (tx, rx) = mpsc::channel(10);
        let (result_tx, result_rx) = oneshot::channel();
        
        // Spawn a background task to handle module execution
        tokio::spawn(run_scan_execution(
            scan.clone(),
            rx,
            result_tx,
            module_repo.clone(),
            result_repo.clone(),
        ));
        
        Ok(Self {
            tx,
            cancel_tx: None,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        // Send start message
        self.tx.send(ExecutionMessage::Start).await
            .map_err(|e| Error::Internal(format!("Failed to send start message: {}", e)))?;
            
        Ok(())
    }
    
    pub async fn cancel(&self) -> Result<()> {
        // Send cancel message
        self.tx.send(ExecutionMessage::Cancel).await
            .map_err(|e| Error::Internal(format!("Failed to send cancel message: {}", e)))?;
            
        // Additionally, if we have a cancel channel, send the signal there as well
        if let Some(cancel_tx) = &self.cancel_tx {
            let _ = cancel_tx.send(());
        }
            
        Ok(())
    }
}

async fn run_scan_execution(
    mut scan: Scan,
    mut rx: mpsc::Receiver<ExecutionMessage>,
    result_tx: oneshot::Sender<ExecutionResult>,
    module_repo: Arc<ModuleRepository>,
    result_repo: Arc<ResultRepository>,
) {
    // Wait for start message
    match rx.recv().await {
        Some(ExecutionMessage::Start) => {
            // Proceed with execution
        },
        Some(ExecutionMessage::Cancel) => {
            // Cancelled before starting
            let _ = result_tx.send(ExecutionResult::Cancelled);
            return;
        },
        None => {
            // Channel closed
            let _ = result_tx.send(ExecutionResult::Failure("Execution channel closed".to_string()));
            return;
        }
    }
    
    // Create a cancel channel
    let (cancel_tx, cancel_rx) = oneshot::channel();
    
    // Process each module in order
    let mut all_results = Vec::new();
    let mut success = true;
    let mut error_message = None;
    
    'module_loop: for module in &mut scan.modules {
        // Update module status to Running
        module.status = ScanModuleStatus::Running;
        module.started_at = Some(Utc::now());
        
        // Execute module against each target
        for target in &mut scan.targets {
            // Check for cancellation
            if cancel_rx.try_recv().is_ok() {
                success = false;
                error_message = Some("Scan execution was cancelled".to_string());
                break 'module_loop;
            }
            
            // Update target status
            target.status = ScanTargetStatus::InProgress;
            
            // Execute module against target
            match execute_module_on_target(
                module,
                target,
                &module_repo,
                &result_repo,
            ).await {
                Ok(results) => {
                    // Store results
                    all_results.extend(results);
                    target.status = ScanTargetStatus::Completed;
                    target.processed_at = Some(Utc::now());
                },
                Err(e) => {
                    // Target failed
                    target.status = ScanTargetStatus::Failed;
                    target.error_message = Some(format!("Failed to process target: {}", e));
                    
                    // But continue with other targets
                    tracing::warn!("Failed to execute module {} on target {}: {}", 
                        module.id, target.id, e);
                }
            }
        }
        
        // Update module status to Completed
        module.status = ScanModuleStatus::Completed;
        module.completed_at = Some(Utc::now());
        
        // TODO: Add module results summary
    }
    
    // Send result
    if success {
        let _ = result_tx.send(ExecutionResult::Success(all_results));
    } else {
        let _ = result_tx.send(ExecutionResult::Failure(
            error_message.unwrap_or_else(|| "Unknown error".to_string())
        ));
    }
}

async fn execute_module_on_target(
    module: &ScanModule,
    target: &ScanTarget,
    module_repo: &Arc<ModuleRepository>,
    result_repo: &Arc<ResultRepository>,
) -> Result<Vec<ScanModuleResult>> {
    // Get module from repository
    let module_info = module_repo.get_module_info(&module.module_id).await?
        .ok_or_else(|| Error::NotFound(format!("Module {} not found", module.module_id)))?;
    
    // Create execution request to send to module registry
    let execution_request = serde_json::json!({
        "module_id": module.module_id,
        "target": {
            "id": target.id,
            "type": target.target_type,
            "value": target.value
        },
        "parameters": module.parameters,
        "scan_id": module.scan_id
    });
    
    // Execute module via module registry API
    let execution_results = module_repo.execute_module(execution_request).await?;
    
    // Process and store results
    let mut scan_results = Vec::new();
    
    if let Some(results_array) = execution_results.as_array() {
        for result in results_array {
            let result_id = Uuid::new_v4();
            let result_type = result["type"].as_str().unwrap_or("unknown").to_string();
            
            let entity_id = result["entity_id"].as_str()
                .and_then(|id| Uuid::parse_str(id).ok());
                
            let relationship_id = result["relationship_id"].as_str()
                .and_then(|id| Uuid::parse_str(id).ok());
            
            let scan_result = ScanModuleResult {
                id: result_id,
                scan_id: module.scan_id,
                module_id: module.module_id,
                target_id: target.id,
                entity_id,
                relationship_id,
                result_type,
                data: result.clone(),
                created_at: Utc::now(),
            };
            
            // Save result
            result_repo.save_result(&scan_result).await?;
            
            scan_results.push(scan_result);
        }
    }
    
    Ok(scan_results)
}
