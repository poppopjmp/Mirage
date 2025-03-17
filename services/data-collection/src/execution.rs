use crate::config::AppConfig;
use crate::models::{CollectionTask, TaskResult, Entity, Relationship};
use mirage_common::{Error, Result};
use reqwest::Client;
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

pub struct TaskExecutor {
    task: CollectionTask,
    client: Arc<Client>,
    config: Arc<AppConfig>,
}

impl TaskExecutor {
    pub fn new(task: CollectionTask, client: Arc<Client>, config: Arc<AppConfig>) -> Self {
        Self { task, client, config }
    }
    
    pub async fn execute(&self) -> Result<TaskResult> {
        // Set timeout if configured
        let max_duration = self.task.max_duration_seconds
            .unwrap_or_else(|| 300); // Default 5 minutes
            
        // Create a timeout future
        match timeout(
            Duration::from_secs(max_duration as u64),
            self.execute_internal()
        ).await {
            Ok(result) => result,
            Err(_) => Err(Error::Internal(format!(
                "Task execution timed out after {} seconds", 
                max_duration
            ))),
        }
    }
    
    async fn execute_internal(&self) -> Result<TaskResult> {
        // Prepare execution request
        let execution_request = serde_json::json!({
            "target": {
                "type": self.task.target.target_type,
                "value": self.task.target.value,
                "metadata": self.task.target.metadata,
                "entity_id": self.task.target.entity_id
            },
            "parameters": self.task.parameters,
            "task_id": self.task.id,
        });
        
        // Execute module via module registry
        let url = format!(
            "{}/api/v1/modules/{}/execute", 
            self.config.module_registry.url,
            self.task.module_id
        );
        
        let response = self.client.post(&url)
            .json(&execution_request)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to execute module: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ExternalApi(format!("Module execution failed: {} - {}", status, error_text)));
        }
        
        // Process results
        let execution_result: serde_json::Value = response.json()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to parse execution result: {}", e)))?;
            
        // Extract entities and relationships
        let entities = self.extract_entities(&execution_result)?;
        let relationships = self.extract_relationships(&execution_result)?;
        
        // Store data in data storage service
        self.store_results(&entities, &relationships).await?;
        
        // Create task result
        let result = TaskResult {
            task_id: self.task.id,
            entities,
            relationships,
            raw_data: Some(execution_result),
            created_at: Utc::now(),
        };
        
        Ok(result)
    }
    
    fn extract_entities(&self, result: &serde_json::Value) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();
        
        // Check if result contains an entities array
        if let Some(entities_array) = result["entities"].as_array() {
            for entity_value in entities_array {
                if let (Some(entity_type), Some(value)) = (
                    entity_value["entity_type"].as_str(),
                    entity_value["value"].as_str(),
                ) {
                    // Extract data
                    let mut data = HashMap::new();
                    if let Some(data_obj) = entity_value["data"].as_object() {
                        for (k, v) in data_obj {
                            data.insert(k.clone(), v.clone());
                        }
                    }
                    
                    // Extract metadata
                    let mut metadata = HashMap::new();
                    if let Some(meta_obj) = entity_value["metadata"].as_object() {
                        for (k, v) in meta_obj {
                            if let Some(v_str) = v.as_str() {
                                metadata.insert(k.clone(), v_str.to_string());
                            }
                        }
                    }
                    
                    // Create entity
                    let entity = Entity {
                        id: entity_value["id"]
                            .as_str()
                            .and_then(|id| Uuid::parse_str(id).ok()),
                        entity_type: entity_type.to_string(),
                        value: value.to_string(),
                        data,
                        metadata,
                        confidence: entity_value["confidence"]
                            .as_u64()
                            .unwrap_or(70) as u8,
                        source: self.task.module_name.clone(),
                    };
                    
                    entities.push(entity);
                }
            }
        }
        
        Ok(entities)
    }
    
    fn extract_relationships(&self, result: &serde_json::Value) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        
        // Check if result contains a relationships array
        if let Some(rel_array) = result["relationships"].as_array() {
            for rel_value in rel_array {
                if let (Some(rel_type), Some(source_value), Some(target_value)) = (
                    rel_value["relationship_type"].as_str(),
                    rel_value["source_value"].as_str(),
                    rel_value["target_value"].as_str(),
                ) {
                    // Extract data
                    let mut data = HashMap::new();
                    if let Some(data_obj) = rel_value["data"].as_object() {
                        for (k, v) in data_obj {
                            data.insert(k.clone(), v.clone());
                        }
                    }
                    
                    // Create relationship
                    let relationship = Relationship {
                        id: rel_value["id"]
                            .as_str()
                            .and_then(|id| Uuid::parse_str(id).ok()),
                        source_id: rel_value["source_id"]
                            .as_str()
                            .and_then(|id| Uuid::parse_str(id).ok()),
                        target_id: rel_value["target_id"]
                            .as_str()
                            .and_then(|id| Uuid::parse_str(id).ok()),
                        source_value: source_value.to_string(),
                        target_value: target_value.to_string(),
                        relationship_type: rel_type.to_string(),
                        data,
                        confidence: rel_value["confidence"]
                            .as_u64()
                            .unwrap_or(70) as u8,
                        source: self.task.module_name.clone(),
                    };
                    
                    relationships.push(relationship);
                }
            }
        }
        
        Ok(relationships)
    }
    
    async fn store_results(&self, entities: &[Entity], relationships: &[Relationship]) -> Result<()> {
        // Skip if no entities or relationships
        if entities.is_empty() && relationships.is_empty() {
            return Ok(());
        }
        
        // Store entities
        for entity in entities {
            self.store_entity(entity).await?;
        }
        
        // Store relationships
        for relationship in relationships {
            self.store_relationship(relationship).await?;
        }
        
        Ok(())
    }
    
    async fn store_entity(&self, entity: &Entity) -> Result<()> {
        // Skip if entity already has an ID (assuming it's already in storage)
        if entity.id.is_some() {
            return Ok(());
        }
        
        // Create entity in data storage
        let url = format!("{}/api/v1/data/entities", self.config.data_storage.url);
        
        let response = self.client.post(&url)
            .json(entity)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to store entity: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ExternalApi(format!("Data storage error: {} - {}", status, error_text)));
        }
        
        Ok(())
    }
    
    async fn store_relationship(&self, relationship: &Relationship) -> Result<()> {
        // Skip if relationship already has an ID
        if relationship.id.is_some() {
            return Ok(());
        }
        
        // Create relationship in data storage
        let url = format!("{}/api/v1/data/relationships", self.config.data_storage.url);
        
        let response = self.client.post(&url)
            .json(relationship)
            .send()
            .await
            .map_err(|e| Error::ExternalApi(format!("Failed to store relationship: {}", e)))?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(Error::ExternalApi(format!("Data storage error: {} - {}", status, error_text)));
        }
        
        Ok(())
    }
}
