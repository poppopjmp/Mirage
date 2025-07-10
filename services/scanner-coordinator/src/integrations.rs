use crate::config::{AppConfig, ServiceConfig};
use crate::error::{ScannerError, ScannerResult};
use crate::models::{CreateTargetRequest, ScanTarget};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct IntegrationService {
    client: Arc<Client>,
    config: Arc<AppConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CollectionTaskRequest {
    target: CollectionTarget,
    module_id: Uuid,
    parameters: Option<HashMap<String, serde_json::Value>>,
    priority: Option<i32>,
    scan_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CollectionTarget {
    target_type: String,
    value: String,
    metadata: Option<HashMap<String, String>>,
    entity_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskResponse {
    id: Uuid,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleInfo {
    id: Uuid,
    name: String,
    version: String,
    description: Option<String>,
    target_types: Vec<String>,
    parameters: HashMap<String, ParameterInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ParameterInfo {
    name: String,
    description: Option<String>,
    required: bool,
    #[serde(rename = "type")]
    param_type: String,
    default: Option<serde_json::Value>,
}

impl IntegrationService {
    pub fn new(client: Client, config: AppConfig) -> Self {
        Self {
            client: Arc::new(client),
            config: Arc::new(config),
        }
    }

    pub async fn create_collection_task(
        &self,
        target: &ScanTarget,
        module_id: Uuid,
        parameters: Option<HashMap<String, serde_json::Value>>,
        priority: Option<i32>,
    ) -> ScannerResult<Uuid> {
        // Create the request body
        let request = CollectionTaskRequest {
            target: CollectionTarget {
                target_type: target.target_type.clone(),
                value: target.value.clone(),
                metadata: Some(target.metadata.clone()),
                entity_id: None, // We don't have an entity ID at this point
            },
            module_id,
            parameters,
            priority,
            scan_id: Some(target.scan_id),
        };

        // Send request to data collection service
        let url = format!(
            "{}/api/v1/collection/tasks",
            self.config.data_collection.url
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await?;
            return Err(ScannerError::Integration(format!(
                "Failed to create collection task: {} - {}",
                status, error
            )));
        }

        let task_response: TaskResponse = response.json().await?;

        Ok(task_response.id)
    }

    pub async fn create_batch_collection_tasks(
        &self,
        targets: &[ScanTarget],
        module_id: Uuid,
        parameters: Option<HashMap<String, serde_json::Value>>,
        priority: Option<i32>,
    ) -> ScannerResult<Vec<Uuid>> {
        // Convert scan targets to collection targets
        let collection_targets: Vec<CollectionTarget> = targets
            .iter()
            .map(|t| CollectionTarget {
                target_type: t.target_type.clone(),
                value: t.value.clone(),
                metadata: Some(t.metadata.clone()),
                entity_id: None,
            })
            .collect();

        // Create batch request
        let request = serde_json::json!({
            "targets": collection_targets,
            "module_id": module_id,
            "parameters": parameters,
            "priority": priority,
            "scan_id": targets.first().map(|t| t.scan_id)
        });

        // Send request to data collection service
        let url = format!(
            "{}/api/v1/collection/batch",
            self.config.data_collection.url
        );

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await?;
            return Err(ScannerError::Integration(format!(
                "Failed to create batch collection tasks: {} - {}",
                status, error
            )));
        }

        // Parse response to get task IDs
        let batch_response: serde_json::Value = response.json().await?;

        let mut task_ids = Vec::new();
        if let Some(tasks) = batch_response["tasks"].as_array() {
            for task in tasks {
                if let Some(id_str) = task["id"].as_str() {
                    if let Ok(id) = Uuid::parse_str(id_str) {
                        task_ids.push(id);
                    }
                }
            }
        }

        Ok(task_ids)
    }

    pub async fn get_module_info(&self, module_id: &Uuid) -> ScannerResult<ModuleInfo> {
        // Fetch module info from module registry
        let url = format!(
            "{}/api/v1/modules/{}",
            self.config.module_registry.url, module_id
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error = response.text().await?;
            return Err(ScannerError::Integration(format!(
                "Failed to fetch module info: {} - {}",
                status, error
            )));
        }

        let module_info: ModuleInfo = response.json().await?;

        Ok(module_info)
    }
}
