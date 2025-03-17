use uuid::Uuid;
use redis::{Client, Commands, AsyncCommands};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use mirage_common::{Error, Result};

#[derive(Clone)]
pub struct TaskQueue {
    client: Client,
    queue_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub task_id: Uuid,
    pub priority: i32,
    pub enqueued_at: DateTime<Utc>,
}

impl TaskQueue {
    pub fn new(client: Client, queue_prefix: String) -> Self {
        Self { client, queue_prefix }
    }
    
    // Add a task to the queue with priority
    pub async fn enqueue_task(&self, task_id: Uuid, priority: i32) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;
            
        let queue_key = format!("{}:tasks", self.queue_prefix);
        
        // Create task data
        let task_data = QueuedTask {
            task_id,
            priority,
            enqueued_at: Utc::now(),
        };
        
        // Serialize task data
        let task_json = serde_json::to_string(&task_data)
            .map_err(|e| Error::Internal(format!("Failed to serialize task data: {}", e)))?;
        
        // Add to sorted set with priority as score (lower priority numbers = higher priority)
        conn.zadd::<_, _, _, i64>(queue_key, task_json, priority).await
            .map_err(|e| Error::Database(format!("Failed to add task to queue: {}", e)))?;
        
        Ok(())
    }
    
    // Get the next task from the queue
    pub async fn dequeue_task(&self) -> Result<Option<QueuedTask>> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;
            
        let queue_key = format!("{}:tasks", self.queue_prefix);
        
        // Get and remove the item with the lowest score (highest priority)
        let result: Option<String> = conn.zpopmin(queue_key, 1).await
            .map_err(|e| Error::Database(format!("Failed to pop task from queue: {}", e)))?;
            
        if let Some(task_json) = result {
            // Deserialize task data
            let task_data = serde_json::from_str::<QueuedTask>(&task_json)
                .map_err(|e| Error::Internal(format!("Failed to deserialize task data: {}", e)))?;
                
            Ok(Some(task_data))
        } else {
            // Queue is empty
            Ok(None)
        }
    }
    
    // Remove a task from the queue (if it exists)
    pub async fn remove_task(&self, task_id: Uuid) -> Result<bool> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;
            
        let queue_key = format!("{}:tasks", self.queue_prefix);
        
        // Get all tasks from the queue
        let tasks: Vec<String> = conn.zrange(queue_key.clone(), 0, -1).await
            .map_err(|e| Error::Database(format!("Failed to read tasks from queue: {}", e)))?;
        
        // Find the task with matching ID
        for task_json in tasks {
            if let Ok(task_data) = serde_json::from_str::<QueuedTask>(&task_json) {
                if task_data.task_id == task_id {
                    // Remove the task
                    let removed: u32 = conn.zrem(queue_key, task_json).await
                        .map_err(|e| Error::Database(format!("Failed to remove task from queue: {}", e)))?;
                    
                    return Ok(removed > 0);
                }
            }
        }
        
        // Task not found
        Ok(false)
    }
    
    // Get the number of tasks in the queue
    pub async fn queue_size(&self) -> Result<u64> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;
            
        let queue_key = format!("{}:tasks", self.queue_prefix);
        
        let size: u64 = conn.zcard(queue_key).await
            .map_err(|e| Error::Database(format!("Failed to get queue size: {}", e)))?;
            
        Ok(size)
    }
}
