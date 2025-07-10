use crate::audit::AuditService;
use crate::config::AppConfig;
use crate::models::{
    ConfigItem, ConfigNamespace, ConfigNamespaceResponse, ConfigResponse, ConfigValueType,
    ConfigVersion, ConfigVersionResponse, CreateConfigRequest, CreateNamespaceRequest,
    PaginatedResponse, UpdateConfigRequest,
};
use crate::repositories::ConfigRepository;
use crate::validation::ConfigValidator;
use chrono::Utc;
use mirage_common::{Error, Result};
use redis::{AsyncCommands, Client as RedisClient, Commands};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ConfigService {
    repo: Arc<ConfigRepository>,
    redis_client: Arc<RedisClient>,
    audit_service: Arc<AuditService>,
    validator: Arc<ConfigValidator>,
    config: Arc<AppConfig>,
}

impl ConfigService {
    pub fn new(
        repo: ConfigRepository,
        redis_client: RedisClient,
        audit_service: AuditService,
        config: AppConfig,
    ) -> Self {
        Self {
            repo: Arc::new(repo),
            redis_client: Arc::new(redis_client),
            audit_service: Arc::new(audit_service),
            validator: Arc::new(ConfigValidator::new()),
            config: Arc::new(config),
        }
    }

    // Create a new configuration item
    pub async fn create_config(
        &self,
        request: CreateConfigRequest,
        user_id: Option<String>,
    ) -> Result<ConfigResponse> {
        // Validate namespace
        let namespace = self
            .repo
            .get_namespace(&request.namespace)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Namespace '{}' not found", request.namespace))
            })?;

        // Check if config with same key+namespace already exists
        if self
            .repo
            .get_config_by_key(&request.key, &request.namespace)
            .await?
            .is_some()
        {
            return Err(Error::Conflict(format!(
                "Configuration with key '{}' in namespace '{}' already exists",
                request.key, request.namespace
            )));
        }

        // Validate value against schema if provided
        if let Some(schema) = &request.schema {
            self.validator
                .validate_against_schema(&request.value, schema)?;
        }

        // Validate value type
        self.validator
            .validate_value_type(&request.value, &request.value_type)?;

        // Create config item
        let now = Utc::now();
        let config_id = Uuid::new_v4();

        let config_item = ConfigItem {
            id: config_id,
            key: request.key.clone(),
            namespace: request.namespace.clone(),
            value: request.value.clone(),
            value_type: request.value_type,
            description: request.description,
            version: 1,
            is_secret: request.is_secret.unwrap_or(false),
            created_at: now,
            updated_at: now,
            created_by: user_id.clone(),
            updated_by: user_id.clone(),
            schema: request.schema,
            tags: request.tags.unwrap_or_default(),
            metadata: request.metadata.unwrap_or_default(),
        };

        // Save to repository
        self.repo.create_config(&config_item).await?;

        // Create initial version record
        let version = ConfigVersion {
            id: Uuid::new_v4(),
            config_id,
            value: request.value.clone(),
            version: 1,
            created_at: now,
            created_by: user_id.clone(),
            comment: Some("Initial version".to_string()),
        };

        self.repo.create_config_version(&version).await?;

        // Add to cache
        self.cache_config(&config_item).await?;

        // Log audit event
        self.audit_service
            .log_create(
                "config_item",
                &config_id,
                user_id.as_deref(),
                &serde_json::to_value(&config_item).unwrap_or_default(),
            )
            .await?;

        // Map to response
        let response = ConfigResponse {
            id: config_item.id,
            key: config_item.key,
            namespace: config_item.namespace,
            value: self.mask_secret_value(&config_item.value, config_item.is_secret),
            value_type: config_item.value_type,
            description: config_item.description,
            version: config_item.version,
            is_secret: config_item.is_secret,
            created_at: config_item.created_at,
            updated_at: config_item.updated_at,
            tags: config_item.tags,
            metadata: config_item.metadata,
            schema: config_item.schema,
        };

        Ok(response)
    }

    // Get configuration by ID
    pub async fn get_config(&self, config_id: Uuid) -> Result<ConfigResponse> {
        let config = self
            .repo
            .get_config_by_id(&config_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Configuration with ID '{}' not found", config_id))
            })?;

        let response = ConfigResponse {
            id: config.id,
            key: config.key,
            namespace: config.namespace,
            value: self.mask_secret_value(&config.value, config.is_secret),
            value_type: config.value_type,
            description: config.description,
            version: config.version,
            is_secret: config.is_secret,
            created_at: config.created_at,
            updated_at: config.updated_at,
            tags: config.tags,
            metadata: config.metadata,
            schema: config.schema,
        };

        Ok(response)
    }

    // Get configuration by key and namespace
    pub async fn get_config_by_key(&self, key: &str, namespace: &str) -> Result<ConfigResponse> {
        // Try getting from cache first
        if let Some(config) = self.get_from_cache(key, namespace).await? {
            let response = ConfigResponse {
                id: config.id,
                key: config.key,
                namespace: config.namespace,
                value: self.mask_secret_value(&config.value, config.is_secret),
                value_type: config.value_type,
                description: config.description,
                version: config.version,
                is_secret: config.is_secret,
                created_at: config.created_at,
                updated_at: config.updated_at,
                tags: config.tags,
                metadata: config.metadata,
                schema: config.schema,
            };

            return Ok(response);
        }

        // Get from repository if not in cache
        let config = self
            .repo
            .get_config_by_key(key, namespace)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "Configuration with key '{}' in namespace '{}' not found",
                    key, namespace
                ))
            })?;

        // Cache the result
        self.cache_config(&config).await?;

        let response = ConfigResponse {
            id: config.id,
            key: config.key,
            namespace: config.namespace,
            value: self.mask_secret_value(&config.value, config.is_secret),
            value_type: config.value_type,
            description: config.description,
            version: config.version,
            is_secret: config.is_secret,
            created_at: config.created_at,
            updated_at: config.updated_at,
            tags: config.tags,
            metadata: config.metadata,
            schema: config.schema,
        };

        Ok(response)
    }

    // Update configuration
    pub async fn update_config(
        &self,
        config_id: Uuid,
        request: UpdateConfigRequest,
        user_id: Option<String>,
    ) -> Result<ConfigResponse> {
        // Get current config
        let mut config = self
            .repo
            .get_config_by_id(&config_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Configuration with ID '{}' not found", config_id))
            })?;

        // Validate value against schema if provided
        if let Some(schema) = request.schema.as_ref().or(config.schema.as_ref()) {
            self.validator
                .validate_against_schema(&request.value, schema)?;
        }

        // Validate value type
        self.validator
            .validate_value_type(&request.value, &config.value_type)?;

        // Store old value for auditing
        let old_value = config.value.clone();

        // Update fields
        config.value = request.value;
        config.updated_at = Utc::now();
        config.updated_by = user_id.clone();
        config.version += 1;

        if let Some(description) = request.description {
            config.description = Some(description);
        }

        if let Some(is_secret) = request.is_secret {
            config.is_secret = is_secret;
        }

        if let Some(schema) = request.schema {
            config.schema = Some(schema);
        }

        if let Some(tags) = request.tags {
            config.tags = tags;
        }

        if let Some(metadata) = request.metadata {
            // Merge metadata rather than replace
            for (k, v) in metadata {
                config.metadata.insert(k, v);
            }
        }

        // Create version record
        let version = ConfigVersion {
            id: Uuid::new_v4(),
            config_id,
            value: config.value.clone(),
            version: config.version,
            created_at: config.updated_at,
            created_by: user_id.clone(),
            comment: request.comment,
        };

        // Save changes
        self.repo.update_config(&config).await?;
        self.repo.create_config_version(&version).await?;

        // Update cache
        self.cache_config(&config).await?;

        // Log audit event
        let change_details = serde_json::json!({
            "old_value": old_value,
            "new_value": config.value,
            "version": config.version,
        });

        self.audit_service
            .log_update(
                "config_item",
                &config_id,
                user_id.as_deref(),
                &change_details,
                Some(format!(
                    "Updated value and incremented version to {}",
                    config.version
                )),
            )
            .await?;

        // Map to response
        let response = ConfigResponse {
            id: config.id,
            key: config.key,
            namespace: config.namespace,
            value: self.mask_secret_value(&config.value, config.is_secret),
            value_type: config.value_type,
            description: config.description,
            version: config.version,
            is_secret: config.is_secret,
            created_at: config.created_at,
            updated_at: config.updated_at,
            tags: config.tags,
            metadata: config.metadata,
            schema: config.schema,
        };

        Ok(response)
    }

    // Delete configuration
    pub async fn delete_config(&self, config_id: Uuid, user_id: Option<&str>) -> Result<()> {
        // Get config first to ensure it exists and for audit purposes
        let config = self
            .repo
            .get_config_by_id(&config_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Configuration with ID '{}' not found", config_id))
            })?;

        // Delete config
        self.repo.delete_config(&config_id).await?;

        // Remove from cache
        self.invalidate_cache(&config.key, &config.namespace)
            .await?;

        // Log audit event
        self.audit_service
            .log_delete(
                "config_item",
                &config_id,
                user_id,
                &serde_json::to_value(&config).unwrap_or_default(),
            )
            .await?;

        Ok(())
    }

    // List configurations with pagination and filtering
    pub async fn list_configs(
        &self,
        namespace: Option<&str>,
        tag: Option<&str>,
        key_contains: Option<&str>,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedResponse<ConfigResponse>> {
        let (configs, total) = self
            .repo
            .list_configs(namespace, tag, key_contains, page, per_page)
            .await?;

        // Map to responses
        let items: Vec<ConfigResponse> = configs
            .into_iter()
            .map(|config| ConfigResponse {
                id: config.id,
                key: config.key,
                namespace: config.namespace,
                value: self.mask_secret_value(&config.value, config.is_secret),
                value_type: config.value_type,
                description: config.description,
                version: config.version,
                is_secret: config.is_secret,
                created_at: config.created_at,
                updated_at: config.updated_at,
                tags: config.tags,
                metadata: config.metadata,
                schema: config.schema,
            })
            .collect();

        let pages = (total + per_page - 1) / per_page;

        let response = PaginatedResponse {
            items,
            total,
            page,
            per_page,
            pages,
        };

        Ok(response)
    }

    // Get configuration history
    pub async fn get_config_history(&self, config_id: Uuid) -> Result<Vec<ConfigVersionResponse>> {
        // Get all versions
        let versions = self.repo.get_config_versions(&config_id).await?;

        // Get config to check if it's a secret
        let config = self
            .repo
            .get_config_by_id(&config_id)
            .await?
            .ok_or_else(|| {
                Error::NotFound(format!("Configuration with ID '{}' not found", config_id))
            })?;

        // Map to responses
        let responses: Vec<ConfigVersionResponse> = versions
            .into_iter()
            .map(|version| ConfigVersionResponse {
                id: version.id,
                config_id: version.config_id,
                value: self.mask_secret_value(&version.value, config.is_secret),
                version: version.version,
                created_at: version.created_at,
                created_by: version.created_by,
                comment: version.comment,
            })
            .collect();

        Ok(responses)
    }

    // Create a new namespace
    pub async fn create_namespace(
        &self,
        request: CreateNamespaceRequest,
        user_id: Option<&str>,
    ) -> Result<ConfigNamespaceResponse> {
        // Check if namespace exists
        if self.repo.get_namespace(&request.name).await?.is_some() {
            return Err(Error::Conflict(format!(
                "Namespace '{}' already exists",
                request.name
            )));
        }

        // Create namespace
        let now = Utc::now();
        let namespace = ConfigNamespace {
            id: Uuid::new_v4(),
            name: request.name.clone(),
            description: request.description,
            created_at: now,
            updated_at: now,
        };

        self.repo.create_namespace(&namespace).await?;

        // Log audit event
        self.audit_service
            .log_create(
                "config_namespace",
                &namespace.id,
                user_id,
                &serde_json::to_value(&namespace).unwrap_or_default(),
            )
            .await?;

        // Return response
        let response = ConfigNamespaceResponse {
            id: namespace.id,
            name: namespace.name,
            description: namespace.description,
            created_at: namespace.created_at,
            updated_at: namespace.updated_at,
            config_count: 0,
        };

        Ok(response)
    }

    // List namespaces
    pub async fn list_namespaces(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<PaginatedResponse<ConfigNamespaceResponse>> {
        let (namespaces, total) = self.repo.list_namespaces(page, per_page).await?;

        // Get config counts for each namespace
        let mut namespace_responses = Vec::with_capacity(namespaces.len());

        for namespace in namespaces {
            let count = self
                .repo
                .count_configs_in_namespace(&namespace.name)
                .await?;

            namespace_responses.push(ConfigNamespaceResponse {
                id: namespace.id,
                name: namespace.name,
                description: namespace.description,
                created_at: namespace.created_at,
                updated_at: namespace.updated_at,
                config_count: count,
            });
        }

        let pages = (total + per_page - 1) / per_page;

        let response = PaginatedResponse {
            items: namespace_responses,
            total,
            page,
            per_page,
            pages,
        };

        Ok(response)
    }

    // Get configuration value without any wrapper
    pub async fn get_raw_config_value(
        &self,
        key: &str,
        namespace: &str,
    ) -> Result<serde_json::Value> {
        let config = self.get_config_by_key(key, namespace).await?;
        Ok(config.value)
    }

    // Get strongly-typed configuration with default fallback
    pub async fn get_config_value<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
        namespace: &str,
        default: T,
    ) -> Result<T> {
        match self.get_config_by_key(key, namespace).await {
            Ok(config) => serde_json::from_value::<T>(config.value)
                .map_err(|e| Error::Internal(format!("Failed to deserialize config value: {}", e))),
            Err(Error::NotFound(_)) => Ok(default),
            Err(e) => Err(e),
        }
    }

    // Helper methods

    // Mask secret value in responses
    fn mask_secret_value(&self, value: &serde_json::Value, is_secret: bool) -> serde_json::Value {
        if is_secret {
            match value {
                serde_json::Value::String(_) => serde_json::Value::String("*****".to_string()),
                serde_json::Value::Object(obj) => {
                    let mut masked_obj = serde_json::Map::new();
                    for (k, v) in obj {
                        masked_obj
                            .insert(k.clone(), serde_json::Value::String("*****".to_string()));
                    }
                    serde_json::Value::Object(masked_obj)
                }
                _ => serde_json::Value::String("*****".to_string()),
            }
        } else {
            value.clone()
        }
    }

    // Get cache key for config item
    fn cache_key(&self, key: &str, namespace: &str) -> String {
        format!("{}:config:{}:{}", self.config.redis.prefix, namespace, key)
    }

    // Cache a config item
    async fn cache_config(&self, config: &ConfigItem) -> Result<()> {
        let key = self.cache_key(&config.key, &config.namespace);

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| Error::Internal(format!("Redis connection error: {}", e)))?;

        let value = serde_json::to_string(config)
            .map_err(|e| Error::Internal(format!("Failed to serialize config: {}", e)))?;

        conn.set_ex::<_, _, ()>(&key, value, self.config.redis.cache_ttl_seconds as usize)
            .await
            .map_err(|e| Error::Internal(format!("Redis set error: {}", e)))?;

        Ok(())
    }

    // Get config from cache
    async fn get_from_cache(&self, key: &str, namespace: &str) -> Result<Option<ConfigItem>> {
        let cache_key = self.cache_key(key, namespace);

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| Error::Internal(format!("Redis connection error: {}", e)))?;

        let value: Option<String> = conn
            .get(&cache_key)
            .await
            .map_err(|e| Error::Internal(format!("Redis get error: {}", e)))?;

        if let Some(json) = value {
            let config = serde_json::from_str::<ConfigItem>(&json).map_err(|e| {
                Error::Internal(format!("Failed to deserialize cached config: {}", e))
            })?;

            Ok(Some(config))
        } else {
            Ok(None)
        }
    }

    // Invalidate cache entry
    async fn invalidate_cache(&self, key: &str, namespace: &str) -> Result<()> {
        let cache_key = self.cache_key(key, namespace);

        let mut conn = self
            .redis_client
            .get_async_connection()
            .await
            .map_err(|e| Error::Internal(format!("Redis connection error: {}", e)))?;

        conn.del::<_, ()>(&cache_key)
            .await
            .map_err(|e| Error::Internal(format!("Redis delete error: {}", e)))?;

        Ok(())
    }
}
