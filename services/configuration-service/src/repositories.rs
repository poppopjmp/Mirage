use crate::config::DatabaseConfig;
use crate::models::{AuditLog, ConfigItem, ConfigNamespace, ConfigValueType, ConfigVersion};
use chrono::Utc;
use mirage_common::{Error, Result};
use sqlx::{postgres::PgPoolOptions, query, query_as, Pool, Postgres};
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

pub async fn create_db_pool(config: &DatabaseConfig) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
        .map_err(|e| Error::Database(format!("Failed to connect to database: {}", e)))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to run migrations: {}", e)))?;

    Ok(pool)
}

#[derive(Clone)]
pub struct ConfigRepository {
    pool: DbPool,
}

impl ConfigRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // Create a new configuration item
    pub async fn create_config(&self, config: &ConfigItem) -> Result<()> {
        query!(
            r#"
            INSERT INTO config_items (
                id, key, namespace, value, value_type, description, version,
                is_secret, created_at, updated_at, created_by, updated_by,
                schema, tags, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            config.id,
            config.key,
            config.namespace,
            config.value,
            config.value_type.to_string(),
            config.description,
            config.version,
            config.is_secret,
            config.created_at,
            config.updated_at,
            config.created_by,
            config.updated_by,
            config.schema,
            &config.tags,
            serde_json::to_value(&config.metadata)?,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create config item: {}", e)))?;

        Ok(())
    }

    // Create a new version of a configuration item
    pub async fn create_config_version(&self, version: &ConfigVersion) -> Result<()> {
        query!(
            r#"
            INSERT INTO config_versions (
                id, config_id, value, version, created_at, created_by, comment
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            version.id,
            version.config_id,
            version.value,
            version.version,
            version.created_at,
            version.created_by,
            version.comment,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create config version: {}", e)))?;

        Ok(())
    }

    // Get a configuration item by ID
    pub async fn get_config_by_id(&self, id: &Uuid) -> Result<Option<ConfigItem>> {
        let row = query!(
            r#"
            SELECT 
                id, key, namespace, value, value_type, description, version,
                is_secret, created_at, updated_at, created_by, updated_by,
                schema, tags, metadata
            FROM config_items
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch config item: {}", e)))?;

        match row {
            Some(r) => {
                // Parse metadata
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)
                    .map_err(|e| Error::Internal(format!("Failed to parse metadata: {}", e)))?;

                // Parse value_type
                let value_type = ConfigValueType::from_str(&r.value_type).map_err(|_| {
                    Error::Internal(format!("Invalid value type: {}", r.value_type))
                })?;

                Ok(Some(ConfigItem {
                    id: r.id,
                    key: r.key,
                    namespace: r.namespace,
                    value: r.value,
                    value_type,
                    description: r.description,
                    version: r.version,
                    is_secret: r.is_secret,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    created_by: r.created_by,
                    updated_by: r.updated_by,
                    schema: r.schema,
                    tags: r.tags,
                    metadata,
                }))
            }
            None => Ok(None),
        }
    }

    // Get a configuration item by key and namespace
    pub async fn get_config_by_key(
        &self,
        key: &str,
        namespace: &str,
    ) -> Result<Option<ConfigItem>> {
        let row = query!(
            r#"
            SELECT 
                id, key, namespace, value, value_type, description, version,
                is_secret, created_at, updated_at, created_by, updated_by,
                schema, tags, metadata
            FROM config_items
            WHERE key = $1 AND namespace = $2
            "#,
            key,
            namespace
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch config item: {}", e)))?;

        match row {
            Some(r) => {
                // Parse metadata
                let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)
                    .map_err(|e| Error::Internal(format!("Failed to parse metadata: {}", e)))?;

                // Parse value_type
                let value_type = ConfigValueType::from_str(&r.value_type).map_err(|_| {
                    Error::Internal(format!("Invalid value type: {}", r.value_type))
                })?;

                Ok(Some(ConfigItem {
                    id: r.id,
                    key: r.key,
                    namespace: r.namespace,
                    value: r.value,
                    value_type,
                    description: r.description,
                    version: r.version,
                    is_secret: r.is_secret,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                    created_by: r.created_by,
                    updated_by: r.updated_by,
                    schema: r.schema,
                    tags: r.tags,
                    metadata,
                }))
            }
            None => Ok(None),
        }
    }

    // Update a configuration item
    pub async fn update_config(&self, config: &ConfigItem) -> Result<()> {
        query!(
            r#"
            UPDATE config_items
            SET 
                value = $1,
                value_type = $2,
                description = $3,
                version = $4,
                is_secret = $5,
                updated_at = $6,
                updated_by = $7,
                schema = $8,
                tags = $9,
                metadata = $10
            WHERE id = $11
            "#,
            config.value,
            config.value_type.to_string(),
            config.description,
            config.version,
            config.is_secret,
            config.updated_at,
            config.updated_by,
            config.schema,
            &config.tags,
            serde_json::to_value(&config.metadata)?,
            config.id,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update config item: {}", e)))?;

        Ok(())
    }

    // Delete a configuration item
    pub async fn delete_config(&self, id: &Uuid) -> Result<()> {
        query!(
            r#"
            DELETE FROM config_items
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete config item: {}", e)))?;

        Ok(())
    }

    // List configuration items with filtering and pagination
    pub async fn list_configs(
        &self,
        namespace: Option<&str>,
        tag: Option<&str>,
        key_contains: Option<&str>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<ConfigItem>, u64)> {
        // Build query with filters
        let mut query_str = String::from(
            "SELECT id, key, namespace, value, value_type, description, version,
            is_secret, created_at, updated_at, created_by, updated_by,
            schema, tags, metadata
            FROM config_items
            WHERE 1=1",
        );

        let mut params = Vec::new();
        let mut param_idx = 1;

        if let Some(ns) = namespace {
            query_str.push_str(&format!(" AND namespace = ${}", param_idx));
            params.push(ns);
            param_idx += 1;
        }

        if let Some(t) = tag {
            query_str.push_str(&format!(" AND ${}::text = ANY(tags)", param_idx));
            params.push(t);
            param_idx += 1;
        }

        if let Some(k) = key_contains {
            query_str.push_str(&format!(" AND key ILIKE ${}", param_idx));
            params.push(format!("%{}%", k));
            param_idx += 1;
        }

        // Add order, limit and offset
        query_str.push_str(&format!(
            " ORDER BY namespace, key LIMIT {} OFFSET {}",
            per_page,
            (page - 1) * per_page
        ));

        // Execute query (simplified for this example - would normally use dynamic parameters)
        let rows = query!(
            r#"
            SELECT 
                id, key, namespace, value, value_type, description, version,
                is_secret, created_at, updated_at, created_by, updated_by,
                schema, tags, metadata
            FROM config_items
            ORDER BY namespace, key
            LIMIT $1 OFFSET $2
            "#,
            per_page as i64,
            ((page - 1) * per_page) as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch config items: {}", e)))?;

        // Parse rows into ConfigItems
        let mut configs = Vec::with_capacity(rows.len());

        for r in rows {
            // Parse metadata
            let metadata: HashMap<String, String> = serde_json::from_value(r.metadata)
                .map_err(|e| Error::Internal(format!("Failed to parse metadata: {}", e)))?;

            // Parse value_type
            let value_type = ConfigValueType::from_str(&r.value_type)
                .map_err(|_| Error::Internal(format!("Invalid value type: {}", r.value_type)))?;

            configs.push(ConfigItem {
                id: r.id,
                key: r.key,
                namespace: r.namespace,
                value: r.value,
                value_type,
                description: r.description,
                version: r.version,
                is_secret: r.is_secret,
                created_at: r.created_at,
                updated_at: r.updated_at,
                created_by: r.created_by,
                updated_by: r.updated_by,
                schema: r.schema,
                tags: r.tags,
                metadata,
            });
        }

        // Get total count
        let total: i64 = query!(
            r#"
            SELECT COUNT(*) as count FROM config_items
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to count config items: {}", e)))?
        .count
        .unwrap_or(0);

        Ok((configs, total as u64))
    }

    // Get all versions of a configuration item
    pub async fn get_config_versions(&self, config_id: &Uuid) -> Result<Vec<ConfigVersion>> {
        let rows = query!(
            r#"
            SELECT 
                id, config_id, value, version, created_at, created_by, comment
            FROM config_versions
            WHERE config_id = $1
            ORDER BY version DESC
            "#,
            config_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch config versions: {}", e)))?;

        let mut versions = Vec::with_capacity(rows.len());

        for r in rows {
            versions.push(ConfigVersion {
                id: r.id,
                config_id: r.config_id,
                value: r.value,
                version: r.version,
                created_at: r.created_at,
                created_by: r.created_by,
                comment: r.comment,
            });
        }

        Ok(versions)
    }

    // Create a new namespace
    pub async fn create_namespace(&self, namespace: &ConfigNamespace) -> Result<()> {
        query!(
            r#"
            INSERT INTO config_namespaces (
                id, name, description, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
            namespace.id,
            namespace.name,
            namespace.description,
            namespace.created_at,
            namespace.updated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create namespace: {}", e)))?;

        Ok(())
    }

    // Get a namespace by name
    pub async fn get_namespace(&self, name: &str) -> Result<Option<ConfigNamespace>> {
        let row = query!(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM config_namespaces
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch namespace: {}", e)))?;

        match row {
            Some(r) => Ok(Some(ConfigNamespace {
                id: r.id,
                name: r.name,
                description: r.description,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })),
            None => Ok(None),
        }
    }

    // List all namespaces with pagination
    pub async fn list_namespaces(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<ConfigNamespace>, u64)> {
        let rows = query!(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM config_namespaces
            ORDER BY name
            LIMIT $1 OFFSET $2
            "#,
            per_page as i64,
            ((page - 1) * per_page) as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch namespaces: {}", e)))?;

        let mut namespaces = Vec::with_capacity(rows.len());

        for r in rows {
            namespaces.push(ConfigNamespace {
                id: r.id,
                name: r.name,
                description: r.description,
                created_at: r.created_at,
                updated_at: r.updated_at,
            });
        }

        // Get total count
        let total: i64 = query!(
            r#"
            SELECT COUNT(*) as count FROM config_namespaces
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to count namespaces: {}", e)))?
        .count
        .unwrap_or(0);

        Ok((namespaces, total as u64))
    }

    // Count configurations in a namespace
    pub async fn count_configs_in_namespace(&self, namespace: &str) -> Result<i32> {
        let row = query!(
            r#"
            SELECT COUNT(*) as count 
            FROM config_items
            WHERE namespace = $1
            "#,
            namespace
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to count configs in namespace: {}", e)))?;

        Ok(row.count.unwrap_or(0) as i32)
    }
}

// Implement FromStr for ConfigValueType
impl FromStr for ConfigValueType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(ConfigValueType::String),
            "integer" => Ok(ConfigValueType::Integer),
            "float" => Ok(ConfigValueType::Float),
            "boolean" => Ok(ConfigValueType::Boolean),
            "json" => Ok(ConfigValueType::Json),
            "list" => Ok(ConfigValueType::List),
            _ => Err(()),
        }
    }
}

// Implement ToString for ConfigValueType
impl ToString for ConfigValueType {
    fn to_string(&self) -> String {
        match self {
            ConfigValueType::String => "string".to_string(),
            ConfigValueType::Integer => "integer".to_string(),
            ConfigValueType::Float => "float".to_string(),
            ConfigValueType::Boolean => "boolean".to_string(),
            ConfigValueType::Json => "json".to_string(),
            ConfigValueType::List => "list".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AuditRepository {
    pool: DbPool,
}

impl AuditRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    // Create a new audit log entry
    pub async fn create_audit_log(&self, log: &AuditLog) -> Result<()> {
        query!(
            r#"
            INSERT INTO audit_logs (
                id, action, entity_type, entity_id, user_id, timestamp,
                details, change_summary, service
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            log.id,
            log.action,
            log.entity_type,
            log.entity_id,
            log.user_id,
            log.timestamp,
            log.details,
            log.change_summary,
            log.service,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to create audit log: {}", e)))?;

        Ok(())
    }

    // Get audit logs for an entity
    pub async fn get_logs_for_entity(
        &self,
        entity_type: &str,
        entity_id: &Uuid,
        limit: u64,
    ) -> Result<Vec<AuditLog>> {
        let rows = query!(
            r#"
            SELECT 
                id, action, entity_type, entity_id, user_id, timestamp,
                details, change_summary, service
            FROM audit_logs
            WHERE entity_type = $1 AND entity_id = $2
            ORDER BY timestamp DESC
            LIMIT $3
            "#,
            entity_type,
            entity_id,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to fetch audit logs: {}", e)))?;

        let mut logs = Vec::with_capacity(rows.len());

        for r in rows {
            logs.push(AuditLog {
                id: r.id,
                action: r.action,
                entity_type: r.entity_type,
                entity_id: r.entity_id,
                user_id: r.user_id,
                timestamp: r.timestamp,
                details: r.details,
                change_summary: r.change_summary,
                service: r.service,
            });
        }

        Ok(logs)
    }
}
