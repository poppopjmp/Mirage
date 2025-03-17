use crate::models::{
    Scan, ScanStatus, ScanTarget, ScanTargetStatus, ScanModule, ScanModuleStatus
};
use crate::error::{ScannerError, ScannerResult};
use crate::config::DatabaseConfig;
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, query, query_as};
use std::collections::HashMap;
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

/// Create database connection pool
pub async fn create_db_pool(config: &DatabaseConfig) -> ScannerResult<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    Ok(pool)
}

pub struct ScanRepository {
    pool: DbPool,
}

impl ScanRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_scan(&self, scan: &Scan) -> ScannerResult<()> {
        query!(
            r#"
            INSERT INTO scans (
                id, name, description, status, created_by, created_at, updated_at,
                started_at, completed_at, priority, tags, metadata, 
                error_message, progress, estimated_completion_time
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            scan.id,
            scan.name,
            scan.description,
            scan.status as _,
            scan.created_by,
            scan.created_at,
            scan.updated_at,
            scan.started_at,
            scan.completed_at,
            scan.priority,
            &scan.tags,
            serde_json::to_value(&scan.metadata)?,
            scan.error_message,
            scan.progress,
            scan.estimated_completion_time,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_scan_by_id(&self, id: Uuid) -> ScannerResult<Option<Scan>> {
        let row = query!(
            r#"
            SELECT 
                id, name, description, status as "status!: ScanStatus", created_by, 
                created_at, updated_at, started_at, completed_at, priority,
                tags, metadata, error_message, progress, estimated_completion_time
            FROM scans
            WHERE id = $1
            "#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        
        match row {
            Some(r) => {
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
                
                Ok(Some(Scan {
                    id: r.id,
                    name: r.name,
                    description: r.description,
                    status: r.status,
                    created_by: r.created_by,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    started_at: r.started_at,
                    completed_at: r.completed_at,
                    priority: r.priority,
                    tags: r.tags,
                    metadata,
                    error_message: r.error_message,
                    progress: r.progress,
                    estimated_completion_time: r.estimated_completion_time,
                }))
            },
            None => Ok(None),
        }
    }
    
    pub async fn update_scan_status(
        &self, 
        id: Uuid, 
        status: ScanStatus,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        progress: Option<i32>,
        error_message: Option<String>
    ) -> ScannerResult<()> {
        query!(
            r#"
            UPDATE scans
            SET 
                status = $1,
                updated_at = $2,
                started_at = COALESCE($3, started_at),
                completed_at = COALESCE($4, completed_at),
                progress = COALESCE($5, progress),
                error_message = COALESCE($6, error_message)
            WHERE id = $7
            "#,
            status as _,
            Utc::now(),
            started_at,
            completed_at,
            progress,
            error_message,
            id,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn update_scan(&self, scan: &Scan) -> ScannerResult<()> {
        query!(
            r#"
            UPDATE scans
            SET 
                name = $1,
                description = $2,
                status = $3,
                updated_at = $4,
                priority = $5,
                tags = $6,
                metadata = $7,
                progress = $8,
                estimated_completion_time = $9,
                error_message = $10
            WHERE id = $11
            "#,
            scan.name,
            scan.description,
            scan.status as _,
            Utc::now(),
            scan.priority,
            &scan.tags,
            serde_json::to_value(&scan.metadata)?,
            scan.progress,
            scan.estimated_completion_time,
            scan.error_message,
            scan.id,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn list_scans(
        &self,
        status: Option<&ScanStatus>,
        created_by: Option<&Uuid>,
        tag: Option<&str>,
        created_after: Option<&DateTime<Utc>>,
        created_before: Option<&DateTime<Utc>>,
        page: u64,
        per_page: u64,
    ) -> ScannerResult<(Vec<Scan>, u64)> {
        // For simplicity, using a basic implementation
        let scans = query!(
            r#"
            SELECT 
                id, name, description, status as "status!: ScanStatus", created_by, 
                created_at, updated_at, started_at, completed_at, priority,
                tags, metadata, error_message, progress, estimated_completion_time
            FROM scans
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            per_page as i64,
            ((page - 1) * per_page) as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let total: i64 = query!(
            r#"
            SELECT COUNT(*) as count FROM scans
            "#
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        
        let mut result = Vec::with_capacity(scans.len());
        for r in scans {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            result.push(Scan {
                id: r.id,
                name: r.name,
                description: r.description,
                status: r.status,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
                started_at: r.started_at,
                completed_at: r.completed_at,
                priority: r.priority,
                tags: r.tags,
                metadata,
                error_message: r.error_message,
                progress: r.progress,
                estimated_completion_time: r.estimated_completion_time,
            });
        }
        
        Ok((result, total as u64))
    }
    
    pub async fn get_pending_scans(&self, limit: i64) -> ScannerResult<Vec<Scan>> {
        let scans = query!(
            r#"
            SELECT 
                id, name, description, status as "status!: ScanStatus", created_by, 
                created_at, updated_at, started_at, completed_at, priority,
                tags, metadata, error_message, progress, estimated_completion_time
            FROM scans
            WHERE status = 'created' OR status = 'queued'
            ORDER BY priority, created_at
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::with_capacity(scans.len());
        for r in scans {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            result.push(Scan {
                id: r.id,
                name: r.name,
                description: r.description,
                status: r.status,
                created_by: r.created_by,
                created_at: r.created_at,
                updated_at: r.updated_at,
                started_at: r.started_at,
                completed_at: r.completed_at,
                priority: r.priority,
                tags: r.tags,
                metadata,
                error_message: r.error_message,
                progress: r.progress,
                estimated_completion_time: r.estimated_completion_time,
            });
        }
        
        Ok(result)
    }
}

pub struct ScanTargetRepository {
    pool: DbPool,
}

impl ScanTargetRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_target(&self, target: &ScanTarget) -> ScannerResult<()> {
        query!(
            r#"
            INSERT INTO scan_targets (
                id, scan_id, target_type, value, status, created_at,
                updated_at, started_at, completed_at, error_message,
                metadata, result_count
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            target.id,
            target.scan_id,
            target.target_type,
            target.value,
            target.status as _,
            target.created_at,
            target.updated_at,
            target.started_at,
            target.completed_at,
            target.error_message,
            serde_json::to_value(&target.metadata)?,
            target.result_count,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_targets_for_scan(&self, scan_id: Uuid) -> ScannerResult<Vec<ScanTarget>> {
        let targets = query!(
            r#"
            SELECT 
                id, scan_id, target_type, value, status as "status!: ScanTargetStatus", 
                created_at, updated_at, started_at, completed_at, error_message,
                metadata, result_count
            FROM scan_targets
            WHERE scan_id = $1
            ORDER BY created_at
            "#,
            scan_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::with_capacity(targets.len());
        for r in targets {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            result.push(ScanTarget {
                id: r.id,
                scan_id: r.scan_id,
                target_type: r.target_type,
                value: r.value,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
                started_at: r.started_at,
                completed_at: r.completed_at,
                error_message: r.error_message,
                metadata,
                result_count: r.result_count,
            });
        }
        
        Ok(result)
    }
    
    pub async fn update_target_status(
        &self, 
        id: Uuid, 
        status: ScanTargetStatus,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        error_message: Option<String>,
        result_count: Option<i32>
    ) -> ScannerResult<()> {
        query!(
            r#"
            UPDATE scan_targets
            SET 
                status = $1,
                updated_at = $2,
                started_at = COALESCE($3, started_at),
                completed_at = COALESCE($4, completed_at),
                error_message = COALESCE($5, error_message),
                result_count = COALESCE($6, result_count)
            WHERE id = $7
            "#,
            status as _,
            Utc::now(),
            started_at,
            completed_at,
            error_message,
            result_count,
            id,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_pending_targets(&self, scan_id: Uuid) -> ScannerResult<Vec<ScanTarget>> {
        let targets = query!(
            r#"
            SELECT 
                id, scan_id, target_type, value, status as "status!: ScanTargetStatus", 
                created_at, updated_at, started_at, completed_at, error_message,
                metadata, result_count
            FROM scan_targets
            WHERE scan_id = $1 AND status = 'pending'
            ORDER BY created_at
            "#,
            scan_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::with_capacity(targets.len());
        for r in targets {
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)?;
            
            result.push(ScanTarget {
                id: r.id,
                scan_id: r.scan_id,
                target_type: r.target_type,
                value: r.value,
                status: r.status,
                created_at: r.created_at,
                updated_at: r.updated_at,
                started_at: r.started_at,
                completed_at: r.completed_at,
                error_message: r.error_message,
                metadata,
                result_count: r.result_count,
            });
        }
        
        Ok(result)
    }
}

pub struct ScanModuleRepository {
    pool: DbPool,
}

impl ScanModuleRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    
    pub async fn create_module(&self, module: &ScanModule) -> ScannerResult<()> {
        query!(
            r#"
            INSERT INTO scan_modules (
                id, scan_id, module_id, module_name, module_version,
                status, parameters, priority, depends_on, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            module.id,
            module.scan_id,
            module.module_id,
            module.module_name,
            module.module_version,
            module.status as _,
            serde_json::to_value(&module.parameters)?,
            module.priority,
            &module.depends_on,
            module.created_at,
            module.updated_at,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_modules_for_scan(&self, scan_id: Uuid) -> ScannerResult<Vec<ScanModule>> {
        let modules = query!(
            r#"
            SELECT 
                id, scan_id, module_id, module_name, module_version,
                status as "status!: ScanModuleStatus", parameters,
                priority, depends_on, created_at, updated_at
            FROM scan_modules
            WHERE scan_id = $1
            ORDER BY priority, created_at
            "#,
            scan_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        let mut result = Vec::with_capacity(modules.len());
        for r in modules {
            let parameters: HashMap<String, serde_json::Value> = serde_json::from_value(r.parameters)?;
            
            result.push(ScanModule {
                id: r.id,
                scan_id: r.scan_id,
                module_id: r.module_id,
                module_name: r.module_name,
                module_version: r.module_version,
                status: r.status,
                parameters,
                priority: r.priority,
                depends_on: r.depends_on,
                created_at: r.created_at,
                updated_at: r.updated_at,
            });
        }
        
        Ok(result)
    }
    
    pub async fn update_module_status(
        &self, 
        id: Uuid, 
        status: ScanModuleStatus,
    ) -> ScannerResult<()> {
        query!(
            r#"
            UPDATE scan_modules
            SET 
                status = $1,
                updated_at = $2
            WHERE id = $3
            "#,
            status as _,
            Utc::now(),
            id,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
