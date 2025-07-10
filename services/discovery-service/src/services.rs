use crate::config::AppConfig;
use crate::error::{DiscoveryError, DiscoveryResult};
use crate::models::{
    ServiceHeartbeatRequest, ServiceInstance, ServiceQuery, ServiceRegistrationRequest,
    ServiceRegistry, ServiceResponse, ServiceStatus,
};
use crate::repository::ServiceRepository;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[derive(Clone)]
pub struct DiscoveryService {
    repo: Arc<ServiceRepository>,
    config: Arc<AppConfig>,
}

impl DiscoveryService {
    pub fn new(repo: ServiceRepository, config: AppConfig) -> Self {
        Self {
            repo: Arc::new(repo),
            config: Arc::new(config),
        }
    }

    // Register a new service
    pub async fn register_service(
        &self,
        request: ServiceRegistrationRequest,
    ) -> DiscoveryResult<ServiceResponse> {
        // Validate request
        if request.name.is_empty() {
            return Err(DiscoveryError::Validation(
                "Service name cannot be empty".into(),
            ));
        }

        if request.address.is_empty() {
            return Err(DiscoveryError::Validation(
                "Service address cannot be empty".into(),
            ));
        }

        if request.port == 0 {
            return Err(DiscoveryError::Validation(
                "Service port cannot be zero".into(),
            ));
        }

        // Create service instance
        let instance = ServiceInstance::new(
            &request.name,
            &request.address,
            request.port,
            request.metadata.unwrap_or_default(),
            request.health_check_url,
        );

        // Register in repository
        self.repo.register_service(&instance).await?;

        // Convert to response
        Ok(instance.into())
    }

    // Update service heartbeat
    pub async fn heartbeat(
        &self,
        request: ServiceHeartbeatRequest,
    ) -> DiscoveryResult<ServiceResponse> {
        // Get existing service
        let mut instance = match self.repo.get_service_by_id(&request.id).await? {
            Some(instance) => instance,
            None => {
                return Err(DiscoveryError::NotFound(format!(
                    "Service instance {} not found",
                    request.id
                )))
            }
        };

        // Update fields
        instance.status = request.status;
        instance.last_heartbeat = Utc::now();

        // Update metadata if provided
        if let Some(metadata) = request.metadata {
            // Merge metadata rather than replace
            for (k, v) in metadata {
                instance.metadata.insert(k, v);
            }
        }

        // Update in repository
        self.repo.update_service(&instance).await?;

        // Convert to response
        Ok(instance.into())
    }

    // Deregister a service
    pub async fn deregister_service(&self, instance_id: &str) -> DiscoveryResult<()> {
        self.repo.deregister_service(instance_id).await
    }

    // Get service by ID
    pub async fn get_service(&self, instance_id: &str) -> DiscoveryResult<ServiceResponse> {
        match self.repo.get_service_by_id(instance_id).await? {
            Some(instance) => Ok(instance.into()),
            None => Err(DiscoveryError::NotFound(format!(
                "Service instance {} not found",
                instance_id
            ))),
        }
    }

    // Get all instances of a service
    pub async fn get_service_instances(
        &self,
        service_name: &str,
    ) -> DiscoveryResult<Vec<ServiceResponse>> {
        let instances = self.repo.get_service_instances(service_name).await?;

        if instances.is_empty() {
            return Err(DiscoveryError::NotFound(format!(
                "No instances found for service {}",
                service_name
            )));
        }

        Ok(instances.into_iter().map(ServiceResponse::from).collect())
    }

    // Get all services
    pub async fn get_all_services(&self) -> DiscoveryResult<ServiceRegistry> {
        let instances = self.repo.get_all_services().await?;
        let responses: Vec<ServiceResponse> =
            instances.into_iter().map(ServiceResponse::from).collect();

        Ok(ServiceRegistry {
            services: responses.clone(),
            count: responses.len(),
            timestamp: Utc::now(),
        })
    }

    // Query services
    pub async fn query_services(&self, query: ServiceQuery) -> DiscoveryResult<ServiceRegistry> {
        let instances = self.repo.query_services(&query).await?;
        let responses: Vec<ServiceResponse> =
            instances.into_iter().map(ServiceResponse::from).collect();

        Ok(ServiceRegistry {
            services: responses.clone(),
            count: responses.len(),
            timestamp: Utc::now(),
        })
    }

    // Clean up expired services
    pub async fn cleanup_expired(&self) -> DiscoveryResult<usize> {
        self.repo.cleanup_expired_services().await
    }
}
