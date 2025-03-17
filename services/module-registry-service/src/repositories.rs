use crate::config::{DatabaseConfig, ModuleStorageConfig};
use crate::models::{Module, ModuleModel, ModuleStatus};
use mirage_common::{Error, Result};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, query, query_as};
use std::fs;
use std::path::Path;
use uuid::Uuid;
use chrono::Utc;

pub type DbPool = Pool<Postgres>;

/// Create database connection pool
pub async fn create_db_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| Error::Database(format!("Database connection failed: {}", e)))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::Database(format!("Migration failed: {}", e)))?;

    Ok(pool)
}

pub struct ModuleRepository {
    pool: DbPool,
    storage_config: ModuleStorageConfig,
}

impl ModuleRepository {
    pub fn new(pool: DbPool, storage_config: ModuleStorageConfig) -> Self {
        Self { pool, storage_config }
    }

    pub async fn create(&self, module: &ModuleModel) -> Result<ModuleModel> {
        let created = sqlx::query_as!(
            ModuleModel,
            r#"
            INSERT INTO modules (id, name, version, description, author, dependencies, capabilities, configuration)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                     capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            "#,
            module.id,
            module.name,
            module.version,
            module.description,
            module.author,
            &module.dependencies as _,
            &module.capabilities as _,
            module.configuration
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create module: {}", e)))?;

        // Ensure the module storage directory exists
        let storage_dir = Path::new(&self.storage_config.path).join(module.name.clone());
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)
                .map_err(|e| Error::Internal(format!("Failed to create module directory: {}", e)))?;
        }

        Ok(created)
    }

    pub async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<ModuleModel>> {
        let modules = sqlx::query_as!(
            ModuleModel,
            r#"
            SELECT id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                  capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            FROM modules
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch modules: {}", e)))?;

        Ok(modules)
    }

    pub async fn find_by_capability(&self, capability: &str, limit: i64, offset: i64) -> Result<Vec<ModuleModel>> {
        let modules = sqlx::query_as!(
            ModuleModel,
            r#"
            SELECT id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                   capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            FROM modules
            WHERE $1 = ANY(capabilities)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#,
            capability,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch modules by capability: {}", e)))?;

        Ok(modules)
    }

    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<ModuleModel>> {
        let module = sqlx::query_as!(
            ModuleModel,
            r#"
            SELECT id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                   capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            FROM modules
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find module: {}", e)))?;

        Ok(module)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<ModuleModel>> {
        let module = sqlx::query_as!(
            ModuleModel,
            r#"
            SELECT id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                   capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            FROM modules
            WHERE name = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find module by name: {}", e)))?;

        Ok(module)
    }

    pub async fn update(&self, module: &ModuleModel) -> Result<ModuleModel> {
        let updated = sqlx::query_as!(
            ModuleModel,
            r#"
            UPDATE modules
            SET version = $2, description = $3, author = $4, dependencies = $5, 
                capabilities = $6, configuration = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, version, description, author, dependencies as "dependencies: Vec<String>", 
                     capabilities as "capabilities: Vec<String>", configuration, created_at, updated_at
            "#,
            module.id,
            module.version,
            module.description,
            module.author,
            &module.dependencies as _,
            &module.capabilities as _,
            module.configuration
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update module: {}", e)))?;

        Ok(updated)
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        // First, get the module name for directory cleanup
        let module = self.find_by_id(id).await?
            .ok_or_else(|| Error::NotFound(format!("Module with ID {} not found", id)))?;

        let result = sqlx::query!(
            r#"
            DELETE FROM modules
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete module: {}", e)))?;

        // If deletion was successful, remove the module directory
        if result.rows_affected() > 0 {
            let storage_dir = Path::new(&self.storage_config.path).join(module.name);
            if storage_dir.exists() {
                fs::remove_dir_all(storage_dir)
                    .map_err(|e| Error::Internal(format!("Failed to delete module directory: {}", e)))?;
            }
        }

        Ok(result.rows_affected() > 0)
    }

    pub async fn save(&self, module: &Module) -> Result<Uuid> {
        let id = sqlx::query!(
            r#"
            INSERT INTO modules (
                id, name, version, description, author, license, 
                created_at, updated_at, capabilities, parameters,
                required_capabilities, metadata, status, file_path, hash
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id
            "#,
            module.id,
            module.name,
            module.version,
            module.description,
            module.author,
            module.license,
            module.created_at,
            module.updated_at,
            &module.capabilities as _,
            serde_json::to_value(&module.parameters).map_err(|e| Error::Internal(format!("Failed to serialize parameters: {}", e)))?,
            &module.required_capabilities as _,
            serde_json::to_value(&module.metadata).map_err(|e| Error::Internal(format!("Failed to serialize metadata: {}", e)))?,
            module.status.to_string(),
            module.file_path,
            module.hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to save module: {}", e)))?
        .id;
        
        Ok(id)
    }
    
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<Module>> {
        let record = sqlx::query_as!(
            ModuleRecord,
            r#"
            SELECT 
                id, name, version, description, author, license, 
                created_at, updated_at, capabilities, parameters,
                required_capabilities, metadata, status, file_path, hash
            FROM modules
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch module: {}", e)))?;
        
        match record {
            Some(r) => Ok(Some(self.record_to_module(r)?)),
            None => Ok(None),
        }
    }
    
    pub async fn find_by_name_version(&self, name: &str, version: &str) -> Result<Option<Module>> {
        let record = sqlx::query_as!(
            ModuleRecord,
            r#"
            SELECT 
                id, name, version, description, author, license, 
                created_at, updated_at, capabilities, parameters,
                required_capabilities, metadata, status, file_path, hash
            FROM modules
            WHERE name = $1 AND version = $2
            "#,
            name,
            version
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch module: {}", e)))?;
        
        match record {
            Some(r) => Ok(Some(self.record_to_module(r)?)),
            None => Ok(None),
        }
    }
    
    pub async fn find_all(
        &self,
        name: Option<&str>,
        capability: Option<&str>,
        author: Option<&str>,
        status: Option<&ModuleStatus>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<Module>, u64)> {
        // Build dynamic WHERE clause based on filters
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;
        
        if let Some(n) = name {
            conditions.push(format!("name ILIKE ${}", param_index));
            params.push(format!("%{}%", n));
            param_index += 1;
        }
        
        if let Some(c) = capability {
            conditions.push(format!("${}::text = ANY(capabilities)", param_index));
            params.push(c.to_string());
            param_index += 1;
        }
        
        if let Some(a) = author {
            conditions.push(format!("author ILIKE ${}", param_index));
            params.push(format!("%{}%", a));
            param_index += 1;
        }
        
        if let Some(s) = status {
            conditions.push(format!("status = ${}", param_index));
            params.push(s.to_string());
            param_index += 1;
        }
        
        // Construct WHERE clause
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };
        
        // Count total matching modules
        let count_sql = format!(
            "SELECT COUNT(*) AS count FROM modules {}", 
            where_clause
        );
        
        let count: i64 = sqlx::query_scalar(&count_sql)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Failed to count modules: {}", e)))?;
        
        // Calculate pagination
        let offset = (page - 1) * per_page;
        let limit = per_page;
        
        // Query modules with pagination
        let query_sql = format!(
            r#"
            SELECT 
                id, name, version, description, author, license, 
                created_at, updated_at, capabilities, parameters,
                required_capabilities, metadata, status, file_path, hash
            FROM modules
            {}
            ORDER BY created_at DESC
            LIMIT {} OFFSET {}
            "#,
            where_clause, limit, offset
        );
        
        let records = sqlx::query_as::<_, ModuleRecord>(&query_sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("Failed to fetch modules: {}", e)))?;
        
        // Convert records to modules
        let mut modules = Vec::with_capacity(records.len());
        for record in records {
            modules.push(self.record_to_module(record)?);
        }
        
        Ok((modules, count as u64))
    }
    
    pub async fn update(&self, module: &Module) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE modules
            SET 
                description = $1,
                updated_at = $2,
                capabilities = $3,
                parameters = $4,
                required_capabilities = $5,
                metadata = $6,
                status = $7
            WHERE id = $8
            "#,
            module.description,
            module.updated_at,
            &module.capabilities as _,
            serde_json::to_value(&module.parameters).map_err(|e| Error::Internal(format!("Failed to serialize parameters: {}", e)))?,
            &module.required_capabilities as _,
            serde_json::to_value(&module.metadata).map_err(|e| Error::Internal(format!("Failed to serialize metadata: {}", e)))?,
            module.status.to_string(),
            module.id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update module: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn delete(&self, id: &Uuid) -> Result<()> {
        let result = sqlx::query!(
            r#"
            DELETE FROM modules
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete module: {}", e)))?;
        
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(format!("Module with ID {} not found", id)));
        }
        
        Ok(())
    }
    
    // Convert database record to domain model
    fn record_to_module(&self, record: ModuleRecord) -> Result<Module> {
        let parameters = serde_json::from_value(record.parameters)
            .map_err(|e| Error::Internal(format!("Failed to deserialize parameters: {}", e)))?;
            
        let metadata = serde_json::from_value(record.metadata)
            .map_err(|e| Error::Internal(format!("Failed to deserialize metadata: {}", e)))?;
            
        let status = match record.status.as_str() {
            "active" => ModuleStatus::Active,
            "disabled" => ModuleStatus::Disabled,
            "deprecated" => ModuleStatus::Deprecated,
            "testing" => ModuleStatus::Testing,
            _ => ModuleStatus::Testing,
        };
        
        Ok(Module {
            id: record.id,
            name: record.name,
            version: record.version,
            description: record.description,
            author: record.author,
            license: record.license,
            created_at: record.created_at,
            updated_at: record.updated_at,
            capabilities: record.capabilities,
            parameters,
            required_capabilities: record.required_capabilities,
            metadata,
            status,
            file_path: record.file_path,
            hash: record.hash,
        })
    }
}

struct ModuleRecord {
    id: Uuid,
    name: String,
    version: String,
    description: String,
    author: String,
    license: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    capabilities: Vec<String>,
    parameters: serde_json::Value,
    required_capabilities: Vec<String>,
    metadata: serde_json::Value,
    status: String,
    file_path: String,
    hash: String,
}
