use crate::config::{MongoDBConfig, RedisConfig};
use crate::models::{CollectionJob, CollectionStatus};
use chrono::{DateTime, Duration, Utc};
use futures::TryStreamExt;
use mirage_common::{Error, Result};
use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOptions, UpdateOptions},
    Client as MongoClient, Database,
};
use redis::{AsyncCommands, Client as RedisClient};
use std::time::Duration as StdDuration;
use uuid::Uuid;

pub async fn create_mongo_client(config: &MongoDBConfig) -> Result<Database> {
    let mut client_options = ClientOptions::parse(&config.uri)
        .await
        .map_err(|e| Error::Database(format!("MongoDB connection string parse error: {}", e)))?;

    client_options.app_name = Some("mirage-data-collection".to_string());

    let client = MongoClient::with_options(client_options)
        .map_err(|e| Error::Database(format!("MongoDB client creation error: {}", e)))?;

    // Ping the server to confirm connection is successful
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
        .map_err(|e| Error::Database(format!("MongoDB ping failed: {}", e)))?;

    Ok(client.database(&config.db_name))
}

pub fn create_redis_client(config: &RedisConfig) -> Result<RedisClient> {
    RedisClient::open(&config.uri)
        .map_err(|e| Error::Database(format!("Redis connection failed: {}", e)))
}

pub struct JobRepository {
    db: Database,
}

impl JobRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub async fn create_job(&self, job: &CollectionJob) -> Result<CollectionJob> {
        let collection = self.db.collection::<CollectionJob>("collection_jobs");

        collection
            .insert_one(job, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to insert job: {}", e)))?;

        Ok(job.clone())
    }

    pub async fn get_job(&self, job_id: &Uuid) -> Result<Option<CollectionJob>> {
        let collection = self.db.collection::<CollectionJob>("collection_jobs");

        let job = collection
            .find_one(doc! {"id": job_id.to_string()}, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to find job: {}", e)))?;

        Ok(job)
    }

    pub async fn update_job_status(&self, job_id: &Uuid, status: CollectionStatus) -> Result<()> {
        let collection = self.db.collection::<Document>("collection_jobs");

        let update = doc! {
            "$set": {
                "status": status.to_string(),
                "updatedAt": Utc::now(),
            }
        };

        collection
            .update_one(doc! {"id": job_id.to_string()}, update, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to update job status: {}", e)))?;

        Ok(())
    }

    pub async fn update_job_started(&self, job_id: &Uuid) -> Result<()> {
        let collection = self.db.collection::<Document>("collection_jobs");

        let update = doc! {
            "$set": {
                "status": CollectionStatus::Running.to_string(),
                "startedAt": Utc::now(),
                "updatedAt": Utc::now(),
            }
        };

        collection
            .update_one(doc! {"id": job_id.to_string()}, update, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to update job as started: {}", e)))?;

        Ok(())
    }

    pub async fn update_job_completed(
        &self,
        job_id: &Uuid,
        result: Option<serde_json::Value>,
    ) -> Result<()> {
        let collection = self.db.collection::<Document>("collection_jobs");

        let mut update_doc = doc! {
            "status": CollectionStatus::Completed.to_string(),
            "completedAt": Utc::now(),
            "updatedAt": Utc::now(),
        };

        if let Some(result) = result {
            update_doc.insert("result", result);
        }

        let update = doc! {
            "$set": update_doc
        };

        collection
            .update_one(doc! {"id": job_id.to_string()}, update, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to update job as completed: {}", e)))?;

        Ok(())
    }

    pub async fn update_job_failed(&self, job_id: &Uuid, error: &str) -> Result<()> {
        let collection = self.db.collection::<Document>("collection_jobs");

        let update = doc! {
            "$set": {
                "status": CollectionStatus::Failed.to_string(),
                "error": error,
                "completedAt": Utc::now(),
                "updatedAt": Utc::now(),
            }
        };

        collection
            .update_one(doc! {"id": job_id.to_string()}, update, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to update job as failed: {}", e)))?;

        Ok(())
    }

    pub async fn get_pending_jobs(&self, limit: i64) -> Result<Vec<CollectionJob>> {
        let collection = self.db.collection::<CollectionJob>("collection_jobs");

        let options = FindOptions::builder()
            .limit(limit)
            .sort(doc! { "createdAt": 1 })
            .build();

        let cursor = collection
            .find(
                doc! {"status": CollectionStatus::Pending.to_string()},
                options,
            )
            .await
            .map_err(|e| Error::Database(format!("Failed to query pending jobs: {}", e)))?;

        let jobs = cursor
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| Error::Database(format!("Failed to collect pending jobs: {}", e)))?;

        Ok(jobs)
    }
}

pub struct RateLimitRepository {
    client: RedisClient,
}

impl RateLimitRepository {
    pub fn new(client: RedisClient) -> Self {
        Self { client }
    }

    pub async fn check_rate_limit(&self, domain: &str, limit_per_minute: u32) -> Result<bool> {
        let key = format!("rate_limit:{}", domain);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        let current: Option<u32> = conn
            .get(&key)
            .await
            .map_err(|e| Error::Database(format!("Failed to get rate limit count: {}", e)))?;

        let current = current.unwrap_or(0);

        if current >= limit_per_minute {
            return Ok(false);
        }

        // Increment the counter
        let _: u32 = conn
            .incr(&key, 1)
            .await
            .map_err(|e| Error::Database(format!("Failed to increment rate limit: {}", e)))?;

        // Set expiration if it's a new key
        if current == 0 {
            let _: bool = conn
                .expire(&key, 60)
                .await
                .map_err(|e| Error::Database(format!("Failed to set expiration: {}", e)))?;
        }

        Ok(true)
    }

    pub async fn get_remaining_limit(&self, domain: &str, limit_per_minute: u32) -> Result<u32> {
        let key = format!("rate_limit:{}", domain);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        let current: Option<u32> = conn
            .get(&key)
            .await
            .map_err(|e| Error::Database(format!("Failed to get rate limit count: {}", e)))?;

        let current = current.unwrap_or(0);

        Ok(limit_per_minute.saturating_sub(current))
    }

    pub async fn get_reset_time(&self, domain: &str) -> Result<DateTime<Utc>> {
        let key = format!("rate_limit:{}", domain);
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| Error::Database(format!("Failed to get Redis connection: {}", e)))?;

        let ttl: i64 = conn
            .ttl(&key)
            .await
            .map_err(|e| Error::Database(format!("Failed to get TTL: {}", e)))?;

        if ttl <= 0 {
            Ok(Utc::now())
        } else {
            Ok(Utc::now() + Duration::seconds(ttl))
        }
    }
}

use crate::models::{
    CollectionTarget, CollectionTask, Entity, Relationship, ResultSummary, TaskResult, TaskStatus,
    TaskType,
};
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use mirage_common::{Error, Result};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Document},
    Collection, Database,
};
use uuid::Uuid;

pub struct TaskRepository {
    collection: Collection<Document>,
}

impl TaskRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("collection_tasks"),
        }
    }

    // Create a new collection task
    pub async fn create_task(&self, task: &CollectionTask) -> Result<Uuid> {
        // Convert task to BSON document
        let task_doc = self.task_to_document(task)?;

        // Insert into collection
        let result = self
            .collection
            .insert_one(task_doc, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to insert task: {}", e)))?;

        // Return the task ID
        Ok(task.id)
    }

    // Get a task by ID
    pub async fn get_task_by_id(&self, id: &Uuid) -> Result<Option<CollectionTask>> {
        let filter = doc! {"id": id.to_string()};

        let result = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch task: {}", e)))?;

        match result {
            Some(doc) => Ok(Some(self.document_to_task(&doc)?)),
            None => Ok(None),
        }
    }

    // Update task status
    pub async fn update_task_status(
        &self,
        id: &Uuid,
        status: TaskStatus,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        error_message: Option<String>,
        result_summary: Option<ResultSummary>,
    ) -> Result<Option<CollectionTask>> {
        let filter = doc! {"id": id.to_string()};

        // Build update document dynamically based on provided values
        let mut update_doc = doc! {
            "$set": {
                "status": status.to_string(),
                "updated_at": Utc::now(),
            }
        };

        if let Some(started) = started_at {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("started_at", bson::to_bson(&started).unwrap());
        }

        if let Some(completed) = completed_at {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("completed_at", bson::to_bson(&completed).unwrap());
        }

        if let Some(error) = error_message {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("error_message", error);
        }

        if let Some(summary) = result_summary {
            update_doc
                .get_document_mut("$set")
                .unwrap()
                .insert("result_summary", bson::to_bson(&summary).unwrap());
        }

        // Find and update options
        let options = FindOneAndUpdateOptions::builder()
            .return_document(ReturnDocument::After)
            .build();

        let result = self
            .collection
            .find_one_and_update(filter, update_doc, options)
            .await
            .map_err(|e| Error::Database(format!("Failed to update task status: {}", e)))?;

        match result {
            Some(doc) => Ok(Some(self.document_to_task(&doc)?)),
            None => Ok(None),
        }
    }

    // Get pending tasks (for worker to process)
    pub async fn get_pending_tasks(&self, limit: u64) -> Result<Vec<CollectionTask>> {
        let filter = doc! {
            "status": TaskStatus::Pending.to_string(),
        };

        let options = FindOptions::builder()
            .sort(doc! {"priority": 1})
            .limit(limit as i64)
            .build();

        let cursor = self
            .collection
            .find(filter, options)
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch pending tasks: {}", e)))?;

        let docs: Vec<Document> = cursor
            .try_collect()
            .await
            .map_err(|e| Error::Database(format!("Failed to collect task documents: {}", e)))?;

        let mut tasks = Vec::with_capacity(docs.len());
        for doc in docs {
            tasks.push(self.document_to_task(&doc)?);
        }

        Ok(tasks)
    }

    // List tasks with filtering and pagination
    pub async fn list_tasks(
        &self,
        status: Option<TaskStatus>,
        module_id: Option<Uuid>,
        scan_id: Option<Uuid>,
        created_by: Option<Uuid>,
        target_type: Option<String>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<CollectionTask>, u64)> {
        // Build filter
        let mut filter = doc! {};

        if let Some(s) = status {
            filter.insert("status", s.to_string());
        }

        if let Some(m) = module_id {
            filter.insert("module_id", m.to_string());
        }

        if let Some(s) = scan_id {
            filter.insert("scan_id", s.to_string());
        }

        if let Some(u) = created_by {
            filter.insert("created_by", u.to_string());
        }

        if let Some(t) = target_type {
            filter.insert("target.target_type", t);
        }

        // First, count total items
        let total = self
            .collection
            .count_documents(filter.clone(), None)
            .await
            .map_err(|e| Error::Database(format!("Failed to count tasks: {}", e)))?;

        // Then fetch paginated results
        let options = FindOptions::builder()
            .sort(doc! {"created_at": -1})
            .skip((page - 1) * per_page)
            .limit(per_page as i64)
            .build();

        let cursor = self
            .collection
            .find(filter, options)
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch tasks: {}", e)))?;

        let docs: Vec<Document> = cursor
            .try_collect()
            .await
            .map_err(|e| Error::Database(format!("Failed to collect task documents: {}", e)))?;

        let mut tasks = Vec::with_capacity(docs.len());
        for doc in docs {
            tasks.push(self.document_to_task(&doc)?);
        }

        Ok((tasks, total))
    }

    // Helper methods for document conversion

    fn task_to_document(&self, task: &CollectionTask) -> Result<Document> {
        let bson_value = bson::to_bson(task)
            .map_err(|e| Error::Internal(format!("Failed to serialize task: {}", e)))?;

        match bson_value {
            bson::Bson::Document(doc) => Ok(doc),
            _ => Err(Error::Internal("Failed to convert task to document".into())),
        }
    }

    fn document_to_task(&self, doc: &Document) -> Result<CollectionTask> {
        let task: CollectionTask = bson::from_document(doc.clone())
            .map_err(|e| Error::Internal(format!("Failed to deserialize task: {}", e)))?;

        Ok(task)
    }
}

pub struct ResultRepository {
    collection: Collection<Document>,
}

impl ResultRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("task_results"),
        }
    }

    // Save a task result
    pub async fn save_result(&self, result: &TaskResult) -> Result<Uuid> {
        // Convert result to BSON document
        let bson_value = bson::to_bson(result)
            .map_err(|e| Error::Internal(format!("Failed to serialize result: {}", e)))?;

        let doc = match bson_value {
            bson::Bson::Document(doc) => doc,
            _ => {
                return Err(Error::Internal(
                    "Failed to convert result to document".into(),
                ))
            }
        };

        // Insert into collection
        self.collection
            .insert_one(doc, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to insert result: {}", e)))?;

        Ok(result.task_id)
    }

    // Get result by task ID
    pub async fn get_result_by_task_id(&self, task_id: &Uuid) -> Result<Option<TaskResult>> {
        let filter = doc! {"task_id": task_id.to_string()};

        let result = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch result: {}", e)))?;

        match result {
            Some(doc) => {
                let task_result: TaskResult = bson::from_document(doc)
                    .map_err(|e| Error::Internal(format!("Failed to deserialize result: {}", e)))?;

                Ok(Some(task_result))
            }
            None => Ok(None),
        }
    }
}
