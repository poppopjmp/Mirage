use crate::config::AppConfig;
use crate::execution::TaskExecutor;
use crate::models::{CollectionTask, ResultSummary, TaskResult, TaskStatus};
use crate::queue::TaskQueue;
use crate::repositories::{ResultRepository, TaskRepository};
use chrono::Utc;
use mirage_common::{Error, Result};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio::time;
use uuid::Uuid;

// Start a worker pool for processing tasks
pub async fn start_worker_pool(
    task_repo: TaskRepository,
    result_repo: ResultRepository,
    task_queue: TaskQueue,
    http_client: Client,
    config: AppConfig,
    min_workers: usize,
    max_workers: usize,
    poll_interval_ms: u64,
) {
    tracing::info!(
        "Starting worker pool with min={}, max={} workers",
        min_workers,
        max_workers
    );

    // Create a channel for workers to report task completion
    let (completion_tx, mut completion_rx) =
        mpsc::channel::<(Uuid, bool, Option<String>, Option<TaskResult>)>(100);

    // Create shared repository instances
    let task_repo = Arc::new(task_repo);
    let result_repo = Arc::new(result_repo);
    let config = Arc::new(config);
    let http_client = Arc::new(http_client);

    // Set up active tasks tracking
    let active_tasks = Arc::new(RwLock::new(HashMap::new()));

    // Set up worker semaphore to limit concurrent workers to max_workers
    let worker_semaphore = Arc::new(Semaphore::new(max_workers));

    // Set up global task queue lock to ensure only one worker can dequeue at a time
    let queue_lock = Arc::new(Mutex::new(()));

    // Start worker monitoring task
    let monitor_task_repo = task_repo.clone();
    let monitor_queue = task_queue.clone();
    let monitor_active_tasks = active_tasks.clone();
    let monitor_semaphore = worker_semaphore.clone();
    tokio::spawn(async move {
        // Ensure at least min_workers are always running
        loop {
            let queue_size = match monitor_queue.queue_size().await {
                Ok(size) => size,
                Err(e) => {
                    tracing::error!("Failed to get queue size: {}", e);
                    0
                }
            };

            let active_count = monitor_active_tasks.read().await.len();

            // If we have less active workers than minimum and there are tasks in the queue,
            // start more workers up to min_workers
            if active_count < min_workers && queue_size > 0 {
                tracing::info!(
                    "Starting additional workers: active={}, min={}",
                    active_count,
                    min_workers
                );

                // Calculate how many workers to start
                let workers_to_start = min_workers - active_count;

                for _ in 0..workers_to_start {
                    // Try to acquire semaphore permit
                    if let Ok(permit) = monitor_semaphore.clone().try_acquire_owned() {
                        let worker_task_repo = monitor_task_repo.clone();
                        let worker_queue = monitor_queue.clone();
                        let worker_active_tasks = monitor_active_tasks.clone();
                        let worker_queue_lock = queue_lock.clone();

                        tokio::spawn(async move {
                            // Start worker process
                            process_tasks(
                                worker_task_repo,
                                worker_queue,
                                worker_active_tasks,
                                worker_queue_lock,
                                permit,
                            )
                            .await;
                        });
                    }
                }
            }

            time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Start completion handler
    let completion_task_repo = task_repo.clone();
    let completion_result_repo = result_repo.clone();
    tokio::spawn(async move {
        while let Some((task_id, success, error_message, task_result)) = completion_rx.recv().await
        {
            // Update task status
            let status = if success {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            };

            // Calculate result summary if successful
            let result_summary = if let Some(result) = &task_result {
                Some(ResultSummary {
                    entities_created: result.entities.len() as u32,
                    relationships_created: result.relationships.len() as u32,
                    entities_updated: 0, // Would need additional tracking
                    data_size_bytes: serde_json::to_string(&result).unwrap_or_default().len()
                        as u64,
                    execution_time_ms: 0, // Would need timing info
                })
            } else {
                None
            };

            // Update task status in database
            if let Err(e) = completion_task_repo
                .update_task_status(
                    &task_id,
                    status,
                    None,
                    Some(Utc::now()),
                    error_message,
                    result_summary,
                )
                .await
            {
                tracing::error!("Failed to update task status: {}", e);
            }

            // If successful and we have a result, store it
            if success && let Some(result) = task_result {
                if let Err(e) = completion_result_repo.save_result(&result).await {
                    tracing::error!("Failed to save task result: {}", e);
                }
            }
        }
    });

    // Start task processing loop
    let processing_task_repo = task_repo.clone();
    let processing_queue = task_queue.clone();
    let processing_semaphore = worker_semaphore.clone();
    let processing_queue_lock = queue_lock.clone();
    let processing_active_tasks = active_tasks.clone();
    tokio::spawn(async move {
        loop {
            // Check if we have available capacity to process more tasks
            let permits_available = processing_semaphore.available_permits();
            let active_count = processing_active_tasks.read().await.len();

            if permits_available > 0 && active_count < max_workers {
                // Try to get next task from queue
                let queue_task = {
                    let _lock = processing_queue_lock.lock().await;
                    match processing_queue.dequeue_task().await {
                        Ok(Some(task)) => task,
                        Ok(None) => {
                            // Queue is empty, sleep and try again
                            time::sleep(Duration::from_millis(poll_interval_ms)).await;
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Failed to dequeue task: {}", e);
                            time::sleep(Duration::from_millis(poll_interval_ms)).await;
                            continue;
                        }
                    }
                };

                // Get task details from database
                let task = match processing_task_repo
                    .get_task_by_id(&queue_task.task_id)
                    .await
                {
                    Ok(Some(task)) => task,
                    Ok(None) => {
                        tracing::error!(
                            "Task {} from queue not found in database",
                            queue_task.task_id
                        );
                        continue;
                    }
                    Err(e) => {
                        tracing::error!("Failed to get task {}: {}", queue_task.task_id, e);
                        continue;
                    }
                };

                // Try to acquire worker permit
                if let Ok(permit) = processing_semaphore.clone().try_acquire_owned() {
                    // Start a worker to process this task
                    let worker_task_repo = processing_task_repo.clone();
                    let worker_active_tasks = processing_active_tasks.clone();
                    let worker_completion_tx = completion_tx.clone();
                    let worker_http_client = http_client.clone();
                    let worker_config = config.clone();

                    tokio::spawn(async move {
                        // Add to active tasks
                        {
                            let mut active = worker_active_tasks.write().await;
                            active.insert(task.id, ());
                        }

                        // Update task status to running
                        if let Err(e) = worker_task_repo
                            .update_task_status(
                                &task.id,
                                TaskStatus::Running,
                                Some(Utc::now()),
                                None,
                                None,
                                None,
                            )
                            .await
                        {
                            tracing::error!("Failed to update task status: {}", e);
                        }

                        // Create task executor
                        let executor =
                            TaskExecutor::new(task.clone(), worker_http_client, worker_config);

                        // Execute task
                        let (success, error, result) = match executor.execute().await {
                            Ok(result) => (true, None, Some(result)),
                            Err(e) => (false, Some(format!("{}", e)), None),
                        };

                        // Send completion message
                        if let Err(e) = worker_completion_tx
                            .send((task.id, success, error, result))
                            .await
                        {
                            tracing::error!("Failed to send completion message: {}", e);
                        }

                        // Remove from active tasks
                        {
                            let mut active = worker_active_tasks.write().await;
                            active.remove(&task.id);
                        }

                        // Drop permit to release worker
                        drop(permit);
                    });
                }
            } else {
                // No capacity available, sleep and try again
                time::sleep(Duration::from_millis(poll_interval_ms)).await;
            }
        }
    });
}

// Process tasks from the queue
async fn process_tasks(
    task_repo: Arc<TaskRepository>,
    task_queue: TaskQueue,
    active_tasks: Arc<RwLock<HashMap<Uuid, ()>>>,
    queue_lock: Arc<Mutex<()>>,
    _permit: tokio::sync::OwnedSemaphorePermit,
) {
    loop {
        // Check if there are any tasks to process
        let task = {
            // Ensure only one worker dequeues at a time
            let _lock = queue_lock.lock().await;

            match task_queue.dequeue_task().await {
                Ok(Some(queued_task)) => {
                    // Get full task details
                    match task_repo.get_task_by_id(&queued_task.task_id).await {
                        Ok(Some(task)) => task,
                        Ok(None) => {
                            tracing::error!(
                                "Task ID {} from queue not found in database",
                                queued_task.task_id
                            );
                            continue;
                        }
                        Err(e) => {
                            tracing::error!("Error fetching task {}: {}", queued_task.task_id, e);
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    // No tasks in queue, sleep before checking again
                    time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
                Err(e) => {
                    tracing::error!("Failed to dequeue task: {}", e);
                    time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        };

        // Skip if task is already being processed
        {
            let active = active_tasks.read().await;
            if active.contains_key(&task.id) {
                tracing::warn!("Task {} is already being processed, skipping", task.id);
                continue;
            }
        }

        // Process the task in a separate task to keep this loop running
        break;
    }
}
