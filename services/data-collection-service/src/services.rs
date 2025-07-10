use crate::config::AppConfig;
use crate::models::{
    BatchTaskRequest, BatchTaskResponse, CollectionTarget, CollectionTask, CreateTaskRequest,
    TaskResponse, TaskResult, TaskStatus, TaskType,
};
use crate::queue::TaskQueue;
use crate::repositories::{ResultRepository, TaskRepository};
use chrono::Utc;
use mirage_common::{Error, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct CollectionService {
    task_repo: Arc<TaskRepository>,
    result_repo: Arc<ResultRepository>,
    task_queue: Arc<TaskQueue>,
    http_client: Arc<Client>,
    config: Arc<AppConfig>,
}

impl CollectionService {
    pub fn new(
        task_repo: TaskRepository,
        result_repo: ResultRepository,
        task_queue: TaskQueue,
        http_client: Client,
        config: AppConfig,
    ) -> Self {
        Self {
            task_repo: Arc::new(task_repo),
            result_repo: Arc::new(result_repo),
            task_queue: Arc::new(task_queue),
            http_client: Arc::new(http_client),
            config: Arc::new(config),
        }
    }

    // Create a new collection task
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<TaskResponse> {
        // Generate a UUID for the task
        let task_id = Uuid::new_v4();

        // Create the target object
        let target = CollectionTarget {
            id: Uuid::new_v4(),
            target_type: request.target.target_type.clone(),
            value: request.target.value.clone(),
            metadata: request.target.metadata.unwrap_or_default(),
            entity_id: request.target.entity_id,
        };

        // Get module info from module registry
        let module_info = self.get_module_info(&request.module_id).await?;

        // Default values
        let priority = request.priority.unwrap_or(5); // 1-10, lower = higher priority
        let task_type = request.task_type.unwrap_or(TaskType::SingleTarget);

        // Create the task
        let task = CollectionTask {
            id: task_id,
            task_type,
            status: TaskStatus::Pending,
            priority,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
            target,
            module_id: request.module_id,
            module_name: module_info.name,
            module_version: module_info.version,
            parameters: request.parameters.unwrap_or_default(),
            scan_id: request.scan_id,
            created_by: None, // TODO: Add user context
            error_message: None,
            result_summary: None,
            max_duration_seconds: request.max_duration_seconds,
        };

        // Save task to database
        self.task_repo.create_task(&task).await?;

        // Add task to queue
        self.task_queue.enqueue_task(task_id, priority).await?;

        // Return response
        Ok(TaskResponse {
            id: task_id,
            status: TaskStatus::Pending,
            created_at: task.created_at,
            started_at: None,
            completed_at: None,
            target: task.target,
            module_name: task.module_name,
        })
    }

    // Create multiple collection tasks
    pub async fn create_batch_tasks(&self, request: BatchTaskRequest) -> Result<BatchTaskResponse> {
        if request.targets.is_empty() {
            return Err(Error::Validation(
                "At least one target must be provided".into(),
            ));
        }

        let mut responses = Vec::with_capacity(request.targets.len());

        for target in request.targets {
            let task_request = CreateTaskRequest {
                target,
                module_id: request.module_id,
                parameters: request.parameters.clone(),
                priority: request.priority,
                scan_id: request.scan_id,
                task_type: Some(TaskType::BatchTarget),
                max_duration_seconds: request.max_duration_seconds,
            };

            match self.create_task(task_request).await {
                Ok(response) => responses.push(response),
                Err(e) => {
                    tracing::error!("Failed to create batch task: {}", e);
                    // Continue processing other targets
                }
            }
        }

        Ok(BatchTaskResponse {
            tasks: responses.clone(),
            total_tasks: responses.len(),
        })
    }

    // Get task by ID
    pub async fn get_task(&self, task_id: Uuid) -> Result<TaskResponse> {
        // Find task in database
        let task = self
            .task_repo
            .get_task_by_id(&task_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?;

        // Convert to response
        Ok(TaskResponse {
            id: task.id,
            status: task.status,
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            target: task.target,
            module_name: task.module_name,
        })
    }

    // Get task result
    pub async fn get_task_result(&self, task_id: Uuid) -> Result<Option<TaskResult>> {
        self.result_repo.get_result_by_task_id(&task_id).await
    }

    // Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<TaskResponse> {
        // Check if task exists and is in a state that can be cancelled
        let task = self
            .task_repo
            .get_task_by_id(&task_id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Task with ID {} not found", task_id)))?;

        if task.status != TaskStatus::Pending && task.status != TaskStatus::Running {
            return Err(Error::Validation(format!(
                "Task cannot be cancelled in state {}",
                serde_json::to_string(&task.status).unwrap()
            )));
        }

        // If it's in the queue, try to remove it
        if task.status == TaskStatus::Pending {
            if let Ok(removed) = self.task_queue.remove_task(task_id).await {
                if !removed {
                    tracing::warn!("Task {} not found in queue", task_id);
                }
            }
        }

        // Update status in database
        self.task_repo
            .update_task_status(
                &task_id,
                TaskStatus::Cancelled,
                None,
                None,
                Some("Task cancelled by user".to_string()),
                None,
            )
            .await?;

        // Return updated task
        self.get_task(task_id).await
    }

    // List tasks with filtering
    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        module_id: Option<Uuid>,
        scan_id: Option<Uuid>,
        target_type: Option<String>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<TaskResponse>, u64)> {
        let (tasks, total) = self
            .task_repo
            .list_tasks(
                status,
                module_id,
                scan_id,
                None, // created_by - not implementing user context here
                target_type,
                page,
                per_page,
            )
            .await?;

        // Convert to responses
        let responses = tasks
            .into_iter()
            .map(|task| TaskResponse {
                id: task.id,
                status: task.status,
                created_at: task.created_at,
                started_at: task.started_at,
                completed_at: task.completed_at,
                target: task.target,
                module_name: task.module_name,
            })
            .collect();

        Ok((responses, total))
    }

    // Helper methods

    // Get module information from Module Registry
    async fn get_module_info(&self, module_id: &Uuid) -> Result<ModuleInfo> {
        let url = format!(
            "{}/api/v1/modules/{}",
            self.config.module_registry.url, module_id
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to fetch module info: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 {
                return Err(Error::NotFound(format!(
                    "Module with ID {} not found",
                    module_id
                )));
            } else {
                return Err(Error::ExternalApi(format!(
                    "Module registry error: {} - {}",
                    status, error_text
                )));
            }
        }

        let module_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to parse module data: {}", e)))?;

        Ok(ModuleInfo {
            id: *module_id,
            name: module_data["name"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            version: module_data["version"]
                .as_str()
                .unwrap_or("0.0.0")
                .to_string(),
        })
    }
}

// Helper struct for module info
#[derive(Debug, Clone)]
struct ModuleInfo {
    id: Uuid,
    name: String,
    version: String,
}
