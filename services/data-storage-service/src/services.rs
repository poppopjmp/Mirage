use crate::models::{
    DataEntity, QueryParams, Relationship, StoreDataRequest, StoreRelationshipRequest,
};
use crate::repositories::{DataRepository, DbPool};
use chrono::Utc;
use elasticsearch::Elasticsearch;
use mirage_common::{Error, Result};
use mongodb::Database;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct StorageService {
    repo: Arc<DataRepository>,
    es_index_prefix: String,
}

impl StorageService {
    pub fn new(
        db_pool: DbPool,
        mongo_db: Database,
        es_client: Elasticsearch,
        es_index_prefix: String,
    ) -> Self {
        Self {
            repo: Arc::new(DataRepository::new(
                db_pool,
                mongo_db,
                es_client,
                es_index_prefix.clone(),
            )),
            es_index_prefix,
        }
    }

    pub async fn store_data(&self, req: StoreDataRequest) -> Result<Uuid> {
        // Validate request
        if req.value.is_empty() {
            return Err(Error::Validation(
                "Entity value cannot be empty".to_string(),
            ));
        }

        if req.entity_type.is_empty() {
            return Err(Error::Validation("Entity type cannot be empty".to_string()));
        }

        // Create DataEntity
        let entity = DataEntity {
            id: Uuid::new_v4(),
            source_module: req.source_module,
            scan_id: req.scan_id,
            entity_type: req.entity_type,
            value: req.value,
            data: req.data,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: req.metadata.unwrap_or_default(),
        };

        self.repo.store_entity(&entity).await
    }

    pub async fn get_data(&self, id: &Uuid) -> Result<DataEntity> {
        let entity = self
            .repo
            .get_entity(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Entity with ID {} not found", id)))?;

        Ok(entity)
    }

    pub async fn update_data(&self, id: &Uuid, data: serde_json::Value) -> Result<()> {
        // Get existing entity
        let mut entity = self
            .repo
            .get_entity(id)
            .await?
            .ok_or_else(|| Error::NotFound(format!("Entity with ID {} not found", id)))?;

        // Update data and timestamp
        entity.data = data;
        entity.updated_at = Utc::now();

        self.repo.update_entity(&entity).await
    }

    pub async fn delete_data(&self, id: &Uuid) -> Result<()> {
        let deleted = self.repo.delete_entity(id).await?;

        if deleted {
            Ok(())
        } else {
            Err(Error::NotFound(format!("Entity with ID {} not found", id)))
        }
    }

    pub async fn query_data(&self, params: QueryParams) -> Result<Vec<DataEntity>> {
        self.repo.query_entities(&params).await
    }

    pub async fn create_relationship(&self, req: StoreRelationshipRequest) -> Result<Uuid> {
        // Validate request
        if req.relationship_type.is_empty() {
            return Err(Error::Validation(
                "Relationship type cannot be empty".to_string(),
            ));
        }

        // Create Relationship
        let relationship = Relationship {
            id: Uuid::new_v4(),
            source_id: req.source_id,
            target_id: req.target_id,
            relationship_type: req.relationship_type,
            data: req.data,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.store_relationship(&relationship).await
    }

    pub async fn get_relationships_for_entity(
        &self,
        entity_id: &Uuid,
    ) -> Result<Vec<Relationship>> {
        // Get relationships where entity is either source or target
        let mut relationships = self.repo.find_relationships_by_source(entity_id).await?;
        let target_relationships = self.repo.find_relationships_by_target(entity_id).await?;

        relationships.extend(target_relationships);
        Ok(relationships)
    }
}
