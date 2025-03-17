use crate::models::{ServiceInstance, ServiceStatus, HealthCheckResult};
use crate::repository::ServiceRepository;
use crate::config::HealthCheckConfig;
use crate::error::{DiscoveryError, DiscoveryResult};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use chrono::Utc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, warn, error};

#[derive(Clone)]
pub struct HealthService {
    repo: Arc<ServiceRepository>,
    client: Arc<Client>,
    config: HealthCheckConfig,
    health_states: Arc<Mutex<HashMap<String, HealthCheckResult>>>,
}

impl HealthService {
    pub fn new(repo: ServiceRepository, client: Client, config: HealthCheckConfig) -> Self {
        Self {
            repo: Arc::new(repo),
            client: Arc::new(client),
            config,
            health_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    // Check the health of a single service instance
    pub async fn check_service_health(&self, instance: &ServiceInstance) -> DiscoveryResult<HealthCheckResult> {
        // Get the health check URL for this service
        let health_url = match instance.get_health_url() {
            Some(url) => url,
            None => return Ok(HealthCheckResult {
                id: instance.id.clone(),
                name: instance.name.clone(),
                status: ServiceStatus::Unknown,
                timestamp: Utc::now(),
                response_time_ms: None,
                error: Some("No health check URL defined".to_string()),
                consecutive_failures: 0,
                consecutive_successes: 0,
            }),
        };
        
        // Get current health state for this service from our cache
        let mut current_state = {
            let states = self.health_states.lock().await;
            states.get(&instance.id)
                .cloned()
                .unwrap_or_else(|| HealthCheckResult {
                    id: instance.id.clone(),
                    name: instance.name.clone(),
                    status: ServiceStatus::Unknown,
                    timestamp: Utc::now(),
                    response_time_ms: None,
                    error: None,
                    consecutive_failures: 0,
                    consecutive_successes: 0,
                })
        };
        
        // Make health check request with timeout
        let start_time = Instant::now();
        let result = self.client.get(&health_url)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await;
        
        let elapsed = start_time.elapsed();
        let response_time_ms = elapsed.as_millis() as u64;
        
        // Update health state based on response
        current_state.timestamp = Utc::now();
        current_state.response_time_ms = Some(response_time_ms);
        
        match result {
            Ok(response) => {
                if response.status().is_success() {
                    current_state.status = ServiceStatus::Up;
                    current_state.error = None;
                    current_state.consecutive_failures = 0;
                    current_state.consecutive_successes += 1;
                } else {
                    current_state.status = ServiceStatus::Down;
                    current_state.error = Some(format!(
                        "Health check failed with status code: {}", 
                        response.status()
                    ));
                    current_state.consecutive_failures += 1;
                    current_state.consecutive_successes = 0;
                }
            },
            Err(err) => {
                current_state.status = ServiceStatus::Down;
                current_state.error = Some(format!("Health check failed: {}", err));
                current_state.consecutive_failures += 1;
                current_state.consecutive_successes = 0;
            }
        }
        
        // Update status in repository if threshold is met
        let status_changed = if current_state.consecutive_failures >= self.config.failure_threshold {
            if instance.status != ServiceStatus::Down {
                self.repo.update_service_status(&instance.id, ServiceStatus::Down).await?;
                true
            } else {
                false
            }
        } else if current_state.consecutive_successes >= self.config.success_threshold {
            if instance.status != ServiceStatus::Up {
                self.repo.update_service_status(&instance.id, ServiceStatus::Up).await?;
                true
            } else {
                false
            }
        } else {
            false
        };
        
        // Update health state in our cache
        {
            let mut states = self.health_states.lock().await;
            states.insert(instance.id.clone(), current_state.clone());
        }
        
        // Log status changes
        if status_changed {
            info!(
                "Service {} status changed to {:?}. Response time: {}ms", 
                instance.name, 
                current_state.status, 
                response_time_ms
            );
        }
        
        Ok(current_state)
    }
    
    // Check health for all services
    pub async fn check_all_services(&self) -> DiscoveryResult<Vec<HealthCheckResult>> {
        let instances = self.repo.get_all_services().await?;
        let mut results = Vec::with_capacity(instances.len());
        
        for instance in instances {
            match self.check_service_health(&instance).await {
                Ok(result) => results.push(result),
                Err(err) => {
                    error!(
                        "Error checking health for service {}: {}", 
                        instance.name, 
                        err
                    );
                    // Still include a result with error information
                    results.push(HealthCheckResult {
                        id: instance.id.clone(),
                        name: instance.name.clone(),
                        status: ServiceStatus::Unknown,
                        timestamp: Utc::now(),
                        response_time_ms: None,
                        error: Some(format!("Health check error: {}", err)),
                        consecutive_failures: 0,
                        consecutive_successes: 0,
                    });
                }
            }
        }
        
        Ok(results)
    }
    
    // Get health check results
    pub async fn get_health_results(&self) -> DiscoveryResult<HashMap<String, HealthCheckResult>> {
        let states = self.health_states.lock().await;
        Ok(states.clone())
    }
    
    // Get health check result for a specific service
    pub async fn get_service_health(&self, service_id: &str) -> DiscoveryResult<Option<HealthCheckResult>> {
        let states = self.health_states.lock().await;
        Ok(states.get(service_id).cloned())
    }
}

// Background health check task
pub async fn run_health_checker(health_service: HealthService) {
    info!("Starting health checker background task");
    
    loop {
        match health_service.check_all_services().await {
            Ok(results) => {
                let up_count = results.iter().filter(|r| r.status == ServiceStatus::Up).count();
                let down_count = results.iter().filter(|r| r.status == ServiceStatus::Down).count();
                let unknown_count = results.iter().filter(|r| r.status == ServiceStatus::Unknown).count();
                
                info!(
                    "Health check completed for {} services: {} up, {} down, {} unknown",
                    results.len(), up_count, down_count, unknown_count
                );
            },
            Err(err) => {
                error!("Error during health check cycle: {}", err);
            }
        }
        
        // Also perform cleanup of expired services
        match health_service.repo.cleanup_expired_services().await {
            Ok(count) if count > 0 => {
                info!("Cleaned up {} expired service registrations", count);
            },
            Ok(_) => {},
            Err(err) => {
                error!("Error during expired services cleanup: {}", err);
            }
        }
        
        // Sleep until next check cycle
        let interval = health_service.config.interval_seconds;
        sleep(Duration::from_secs(interval)).await;
    }
}
