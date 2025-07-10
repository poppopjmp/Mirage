use crate::config::RedisConfig;
use crate::error::{DiscoveryError, DiscoveryResult};
use crate::models::{ServiceInstance, ServiceQuery, ServiceStatus};
use chrono::Utc;
use redis::{AsyncCommands, Client as RedisClient, FromRedisValue, RedisResult};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct ServiceRepository {
    client: Arc<RedisClient>,
    config: RedisConfig,
}

impl ServiceRepository {
    pub fn new(client: RedisClient, config: RedisConfig) -> Self {
        Self {
            client: Arc::new(client),
            config,
        }
    }

    // Register a new service instance
    pub async fn register_service(&self, instance: &ServiceInstance) -> DiscoveryResult<()> {
        let mut conn = self.client.get_async_connection().await?;

        // Key for this specific service instance
        let instance_key = format!("{}:instance:{}", self.config.key_prefix, instance.id);

        // Key for the list of instances for this service type
        let service_key = format!("{}:service:{}", self.config.key_prefix, instance.name);

        // Serialize the instance
        let instance_json = serde_json::to_string(instance).map_err(|e| {
            DiscoveryError::Internal(format!("Failed to serialize instance: {}", e))
        })?;

        // Store the instance with an expiration (TTL)
        conn.set_ex(
            instance_key.clone(),
            instance_json,
            self.config.service_ttl_seconds as usize,
        )
        .await?;

        // Add instance ID to the set of instances for this service
        conn.sadd(service_key, &instance.id).await?;

        // Add service to the list of known services
        conn.sadd(
            format!("{}:services", self.config.key_prefix),
            &instance.name,
        )
        .await?;

        Ok(())
    }

    // Update a service instance (heartbeat)
    pub async fn update_service(&self, instance: &ServiceInstance) -> DiscoveryResult<()> {
        let mut conn = self.client.get_async_connection().await?;

        // Key for this specific service instance
        let instance_key = format!("{}:instance:{}", self.config.key_prefix, instance.id);

        // Check if instance exists
        let exists: bool = conn.exists(&instance_key).await?;
        if !exists {
            return Err(DiscoveryError::NotFound(format!(
                "Service instance {} not found",
                instance.id
            )));
        }

        // Serialize the instance
        let instance_json = serde_json::to_string(instance).map_err(|e| {
            DiscoveryError::Internal(format!("Failed to serialize instance: {}", e))
        })?;

        // Store the updated instance with an expiration (TTL)
        conn.set_ex(
            instance_key,
            instance_json,
            self.config.service_ttl_seconds as usize,
        )
        .await?;

        Ok(())
    }

    // Deregister a service instance
    pub async fn deregister_service(&self, instance_id: &str) -> DiscoveryResult<()> {
        let mut conn = self.client.get_async_connection().await?;

        // Get service name first
        let instance_key = format!("{}:instance:{}", self.config.key_prefix, instance_id);

        // Check if instance exists
        let instance_json: Option<String> = conn.get(&instance_key).await?;

        match instance_json {
            Some(json) => {
                // Parse the instance to get service name
                let instance: ServiceInstance = serde_json::from_str(&json).map_err(|e| {
                    DiscoveryError::Internal(format!("Failed to deserialize instance: {}", e))
                })?;

                // Service key
                let service_key = format!("{}:service:{}", self.config.key_prefix, instance.name);

                // Delete instance from Redis
                conn.del(&instance_key).await?;

                // Remove from service set
                conn.srem(service_key, instance_id).await?;

                // Check if this was the last instance of this service
                let count: u64 = conn.scard(service_key.clone()).await?;
                if count == 0 {
                    // Remove service from list of known services
                    conn.srem(
                        format!("{}:services", self.config.key_prefix),
                        instance.name,
                    )
                    .await?;
                    // Remove empty set
                    conn.del(service_key).await?;
                }

                Ok(())
            }
            None => Err(DiscoveryError::NotFound(format!(
                "Service instance {} not found",
                instance_id
            ))),
        }
    }

    // Get a service instance by ID
    pub async fn get_service_by_id(
        &self,
        instance_id: &str,
    ) -> DiscoveryResult<Option<ServiceInstance>> {
        let mut conn = self.client.get_async_connection().await?;

        let instance_key = format!("{}:instance:{}", self.config.key_prefix, instance_id);
        let instance_json: Option<String> = conn.get(&instance_key).await?;

        match instance_json {
            Some(json) => {
                let instance: ServiceInstance = serde_json::from_str(&json).map_err(|e| {
                    DiscoveryError::Internal(format!("Failed to deserialize instance: {}", e))
                })?;

                Ok(Some(instance))
            }
            None => Ok(None),
        }
    }

    // Get all instances of a service by name
    pub async fn get_service_instances(
        &self,
        service_name: &str,
    ) -> DiscoveryResult<Vec<ServiceInstance>> {
        let mut conn = self.client.get_async_connection().await?;

        let service_key = format!("{}:service:{}", self.config.key_prefix, service_name);
        let instance_ids: Vec<String> = conn.smembers(&service_key).await?;

        let mut instances = Vec::with_capacity(instance_ids.len());

        for id in instance_ids {
            let instance_key = format!("{}:instance:{}", self.config.key_prefix, id);
            let instance_json: Option<String> = conn.get(&instance_key).await?;

            if let Some(json) = instance_json {
                let instance: ServiceInstance = serde_json::from_str(&json).map_err(|e| {
                    DiscoveryError::Internal(format!("Failed to deserialize instance: {}", e))
                })?;

                instances.push(instance);
            }
        }

        Ok(instances)
    }

    // Get all service names
    pub async fn get_service_names(&self) -> DiscoveryResult<Vec<String>> {
        let mut conn = self.client.get_async_connection().await?;

        let services_key = format!("{}:services", self.config.key_prefix);
        let names: Vec<String> = conn.smembers(&services_key).await?;

        Ok(names)
    }

    // Get all service instances
    pub async fn get_all_services(&self) -> DiscoveryResult<Vec<ServiceInstance>> {
        let service_names = self.get_service_names().await?;
        let mut all_instances = Vec::new();

        for name in service_names {
            let instances = self.get_service_instances(&name).await?;
            all_instances.extend(instances);
        }

        Ok(all_instances)
    }

    // Update service status
    pub async fn update_service_status(
        &self,
        instance_id: &str,
        status: ServiceStatus,
    ) -> DiscoveryResult<()> {
        let instance = match self.get_service_by_id(instance_id).await? {
            Some(mut instance) => {
                instance.status = status;
                instance.last_heartbeat = Utc::now();
                instance
            }
            None => {
                return Err(DiscoveryError::NotFound(format!(
                    "Service instance {} not found",
                    instance_id
                )))
            }
        };

        self.update_service(&instance).await
    }

    // Query services by criteria
    pub async fn query_services(
        &self,
        query: &ServiceQuery,
    ) -> DiscoveryResult<Vec<ServiceInstance>> {
        let all_instances = if let Some(name) = &query.name {
            self.get_service_instances(name).await?
        } else {
            self.get_all_services().await?
        };

        // Filter by status
        let filtered = all_instances
            .into_iter()
            .filter(|instance| {
                // Filter by status if specified
                if let Some(status) = &query.status {
                    if &instance.status != status {
                        return false;
                    }
                }

                // Filter by metadata key/value if specified
                if let Some(key) = &query.metadata_key {
                    if !instance.metadata.contains_key(key) {
                        return false;
                    }

                    // If value is also specified, check it matches
                    if let Some(value) = &query.metadata_value {
                        if let Some(instance_value) = instance.metadata.get(key) {
                            if instance_value != value {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }

                true
            })
            .collect();

        Ok(filtered)
    }

    // Clean up expired service instances
    pub async fn cleanup_expired_services(&self) -> DiscoveryResult<usize> {
        let mut conn = self.client.get_async_connection().await?;
        let service_names = self.get_service_names().await?;

        let mut removed_count = 0;

        for name in service_names {
            let service_key = format!("{}:service:{}", self.config.key_prefix, name);
            let instance_ids: Vec<String> = conn.smembers(&service_key).await?;

            for id in instance_ids {
                let instance_key = format!("{}:instance:{}", self.config.key_prefix, id);
                let exists: bool = conn.exists(&instance_key).await?;

                if !exists {
                    // Instance expired from Redis TTL, remove it from the service set
                    conn.srem(&service_key, &id).await?;
                    removed_count += 1;
                }
            }

            // Check if this service has any instances left
            let count: u64 = conn.scard(&service_key).await?;
            if count == 0 {
                // Remove service from list of known services
                conn.srem(format!("{}:services", self.config.key_prefix), &name)
                    .await?;
                // Remove empty set
                conn.del(&service_key).await?;
            }
        }

        Ok(removed_count)
    }
}
