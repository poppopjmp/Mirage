use crate::config::{DatabaseConfig, ElasticsearchConfig, MongoDBConfig};
use crate::models::{DataEntity, QueryParams, Relationship};
use chrono::Utc;
use elasticsearch::{http::transport::Transport, Elasticsearch, SearchParts};
use futures::TryStreamExt;
use mirage_common::{Error, Result};
use mongodb::{
    bson::{doc, to_document, Document},
    options::{ClientOptions, FindOptions},
    Client as MongoClient, Database,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::ops::Range;
use uuid::Uuid;

pub type DbPool = Pool<Postgres>;

/// Create PostgreSQL database connection pool
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

/// Create MongoDB client
pub async fn create_mongo_client(config: &MongoDBConfig) -> Result<Database> {
    let mut client_options = ClientOptions::parse(&config.uri)
        .await
        .map_err(|e| Error::Database(format!("MongoDB connection string parse error: {}", e)))?;

    client_options.app_name = Some("mirage-data-storage".to_string());

    let client = MongoClient::with_options(client_options)
        .map_err(|e| Error::Database(format!("MongoDB client creation error: {}", e)))?;

    // Ping the server to confirm connection is successful
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await
        .map_err(|e| Error::Database(format!("MongoDB ping failed: {}", e)))?;

    Ok(client.database(&config.db_name))
}

/// Create Elasticsearch client
pub fn create_elasticsearch_client(config: &ElasticsearchConfig) -> Result<Elasticsearch> {
    let transport = Transport::single_node(&config.url)
        .map_err(|e| Error::Database(format!("Elasticsearch connection failed: {}", e)))?;

    Ok(Elasticsearch::new(transport))
}

pub struct DataRepository {
    db_pool: DbPool,
    mongo_db: Database,
    es_client: Elasticsearch,
    es_index_prefix: String,
}

impl DataRepository {
    pub fn new(
        db_pool: DbPool,
        mongo_db: Database,
        es_client: Elasticsearch,
        es_index_prefix: String,
    ) -> Self {
        Self {
            db_pool,
            mongo_db,
            es_client,
            es_index_prefix,
        }
    }

    pub async fn store_entity(&self, entity: &DataEntity) -> Result<Uuid> {
        // Store metadata and core info in PostgreSQL
        let record_id = sqlx::query!(
            r#"
            INSERT INTO entities (id, source_module, scan_id, entity_type, value, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
            entity.id,
            entity.source_module,
            entity.scan_id,
            entity.entity_type,
            entity.value,
            entity.created_at,
            entity.updated_at,
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to store entity in SQL: {}", e)))?
        .id;

        // Store full data structure in MongoDB
        let mongo_collection = self.mongo_db.collection::<DataEntity>("entities");
        mongo_collection
            .insert_one(entity, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to store entity in MongoDB: {}", e)))?;

        // Index in Elasticsearch for searching
        let es_index = format!("{}_entities", self.es_index_prefix);
        let es_doc = serde_json::to_value(entity)
            .map_err(|e| Error::Internal(format!("Failed to serialize entity: {}", e)))?;

        self.es_client
            .index(elasticsearch::IndexParts::index_id(
                &es_index,
                &entity.id.to_string(),
            ))
            .body(es_doc)
            .send()
            .await
            .map_err(|e| {
                Error::Database(format!("Failed to index entity in Elasticsearch: {}", e))
            })?;

        Ok(record_id)
    }

    pub async fn get_entity(&self, id: &Uuid) -> Result<Option<DataEntity>> {
        let collection = self.mongo_db.collection::<DataEntity>("entities");

        let entity = collection
            .find_one(doc! {"id": id.to_string()}, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to find entity: {}", e)))?;

        Ok(entity)
    }

    pub async fn update_entity(&self, entity: &DataEntity) -> Result<()> {
        // Update metadata in PostgreSQL
        sqlx::query!(
            r#"
            UPDATE entities
            SET source_module = $2, entity_type = $3, value = $4, updated_at = $5
            WHERE id = $1
            "#,
            entity.id,
            entity.source_module,
            entity.entity_type,
            entity.value,
            entity.updated_at,
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to update entity in SQL: {}", e)))?;

        // Update full data in MongoDB
        let collection = self.mongo_db.collection::<Document>("entities");
        let entity_doc = mongodb::bson::to_document(entity)
            .map_err(|e| Error::Internal(format!("Failed to serialize entity: {}", e)))?;

        collection
            .replace_one(doc! {"id": entity.id.to_string()}, entity_doc, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to update entity in MongoDB: {}", e)))?;

        // Update Elasticsearch index
        let es_index = format!("{}_entities", self.es_index_prefix);
        let es_doc = serde_json::to_value(entity)
            .map_err(|e| Error::Internal(format!("Failed to serialize entity: {}", e)))?;

        self.es_client
            .index(elasticsearch::IndexParts::index_id(
                &es_index,
                &entity.id.to_string(),
            ))
            .body(es_doc)
            .send()
            .await
            .map_err(|e| {
                Error::Database(format!("Failed to index entity in Elasticsearch: {}", e))
            })?;

        Ok(())
    }

    pub async fn delete_entity(&self, id: &Uuid) -> Result<bool> {
        // Delete from PostgreSQL
        let result = sqlx::query!(
            r#"
            DELETE FROM entities WHERE id = $1
            "#,
            id
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to delete entity from SQL: {}", e)))?;

        // Delete from MongoDB
        let collection = self.mongo_db.collection::<Document>("entities");
        collection
            .delete_one(doc! {"id": id.to_string()}, None)
            .await
            .map_err(|e| Error::Database(format!("Failed to delete entity from MongoDB: {}", e)))?;

        // Delete from Elasticsearch
        let es_index = format!("{}_entities", self.es_index_prefix);
        self.es_client
            .delete(elasticsearch::DeleteParts::index_id(
                &es_index,
                &id.to_string(),
            ))
            .send()
            .await
            .map_err(|e| {
                Error::Database(format!("Failed to delete entity from Elasticsearch: {}", e))
            })?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn query_entities(&self, params: &QueryParams) -> Result<Vec<DataEntity>> {
        // For complex queries, we'll use Elasticsearch
        let es_index = format!("{}_entities", self.es_index_prefix);

        // Build Elasticsearch query
        let mut query = serde_json::json!({
            "query": {
                "bool": {
                    "must": []
                }
            },
            "size": params.limit.unwrap_or(100),
            "from": params.offset.unwrap_or(0),
            "sort": [
                { "created_at": { "order": "desc" } }
            ]
        });

        let must = query["query"]["bool"]["must"].as_array_mut().unwrap();

        // Add query conditions based on params
        if let Some(ref entity_type) = params.entity_type {
            must.push(serde_json::json!({
                "term": { "entity_type": entity_type }
            }));
        }

        if let Some(ref value) = params.value {
            must.push(serde_json::json!({
                "match": { "value": value }
            }));
        }

        if let Some(ref source_module) = params.source_module {
            must.push(serde_json::json!({
                "term": { "source_module": source_module.to_string() }
            }));
        }

        if let Some(ref scan_id) = params.scan_id {
            must.push(serde_json::json!({
                "term": { "scan_id": scan_id.to_string() }
            }));
        }

        // Date range
        if params.from_date.is_some() || params.to_date.is_some() {
            let mut range = serde_json::json!({
                "range": {
                    "created_at": {}
                }
            });

            if let Some(from_date) = params.from_date {
                range["range"]["created_at"]["gte"] =
                    serde_json::Value::String(from_date.to_rfc3339());
            }

            if let Some(to_date) = params.to_date {
                range["range"]["created_at"]["lte"] =
                    serde_json::Value::String(to_date.to_rfc3339());
            }

            must.push(range);
        }

        // Execute search
        let response = self
            .es_client
            .search(SearchParts::index(&[&es_index]))
            .body(query)
            .send()
            .await
            .map_err(|e| Error::Database(format!("Failed to search entities: {}", e)))?;

        // Parse response
        let response_body = response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::Database(format!("Failed to parse search response: {}", e)))?;

        // Extract hits and convert to DataEntity objects
        let hits = response_body["hits"]["hits"]
            .as_array()
            .ok_or_else(|| Error::Internal("Invalid search response format".to_string()))?;

        let mut entities = Vec::new();
        for hit in hits {
            let source = hit["_source"].clone();
            let entity: DataEntity = serde_json::from_value(source)
                .map_err(|e| Error::Internal(format!("Failed to deserialize entity: {}", e)))?;
            entities.push(entity);
        }

        Ok(entities)
    }

    // Relationship methods
    pub async fn store_relationship(&self, relationship: &Relationship) -> Result<Uuid> {
        // Store in PostgreSQL
        let record_id = sqlx::query!(
            r#"
            INSERT INTO relationships (id, source_id, target_id, relationship_type, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
            relationship.id,
            relationship.source_id,
            relationship.target_id,
            relationship.relationship_type,
            relationship.created_at,
            relationship.updated_at,
        )
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to store relationship in SQL: {}", e)))?
        .id;

        // Store extra data in MongoDB if present
        if let Some(ref data) = relationship.data {
            let mongo_collection = self.mongo_db.collection::<Document>("relationships");
            let mut doc = Document::new();
            doc.insert("id", relationship.id.to_string());
            doc.insert("source_id", relationship.source_id.to_string());
            doc.insert("target_id", relationship.target_id.to_string());
            doc.insert("relationship_type", &relationship.relationship_type);
            doc.insert(
                "data",
                mongodb::bson::to_bson(data).map_err(|e| {
                    Error::Internal(format!("Failed to serialize relationship data: {}", e))
                })?,
            );
            doc.insert("created_at", relationship.created_at);
            doc.insert("updated_at", relationship.updated_at);

            mongo_collection.insert_one(doc, None).await.map_err(|e| {
                Error::Database(format!(
                    "Failed to store relationship data in MongoDB: {}",
                    e
                ))
            })?;
        }

        Ok(record_id)
    }

    pub async fn find_relationships_by_source(
        &self,
        source_id: &Uuid,
    ) -> Result<Vec<Relationship>> {
        let relationships = sqlx::query_as!(
            RelationshipRecord,
            r#"
            SELECT id, source_id, target_id, relationship_type, created_at, updated_at
            FROM relationships
            WHERE source_id = $1
            "#,
            source_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find relationships: {}", e)))?;

        // Convert to Relationship objects and fetch additional data if needed
        let mut results = Vec::new();
        for record in relationships {
            let mut relationship = Relationship {
                id: record.id,
                source_id: record.source_id,
                target_id: record.target_id,
                relationship_type: record.relationship_type,
                data: None,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };

            // Get additional data from MongoDB if available
            let mongo_collection = self.mongo_db.collection::<Document>("relationships");
            if let Some(doc) = mongo_collection
                .find_one(doc! {"id": relationship.id.to_string()}, None)
                .await
                .map_err(|e| Error::Database(format!("Failed to fetch relationship data: {}", e)))?
            {
                if let Ok(data_bson) = doc.get_document("data") {
                    if let Ok(data) =
                        mongodb::bson::from_bson(mongodb::bson::Bson::Document(data_bson.clone()))
                    {
                        relationship.data = Some(data);
                    }
                }
            }

            results.push(relationship);
        }

        Ok(results)
    }

    pub async fn find_relationships_by_target(
        &self,
        target_id: &Uuid,
    ) -> Result<Vec<Relationship>> {
        let relationships = sqlx::query_as!(
            RelationshipRecord,
            r#"
            SELECT id, source_id, target_id, relationship_type, created_at, updated_at
            FROM relationships
            WHERE target_id = $1
            "#,
            target_id
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| Error::Database(format!("Failed to find relationships: {}", e)))?;

        // Same process as above to get complete relationship objects
        let mut results = Vec::new();
        for record in relationships {
            let mut relationship = Relationship {
                id: record.id,
                source_id: record.source_id,
                target_id: record.target_id,
                relationship_type: record.relationship_type,
                data: None,
                created_at: record.created_at,
                updated_at: record.updated_at,
            };

            let mongo_collection = self.mongo_db.collection::<Document>("relationships");
            if let Some(doc) = mongo_collection
                .find_one(doc! {"id": relationship.id.to_string()}, None)
                .await
                .map_err(|e| Error::Database(format!("Failed to fetch relationship data: {}", e)))?
            {
                if let Ok(data_bson) = doc.get_document("data") {
                    if let Ok(data) =
                        mongodb::bson::from_bson(mongodb::bson::Bson::Document(data_bson.clone()))
                    {
                        relationship.data = Some(data);
                    }
                }
            }

            results.push(relationship);
        }

        Ok(results)
    }
}

// Internal struct for SQL query results
struct RelationshipRecord {
    id: Uuid,
    source_id: Uuid,
    target_id: Uuid,
    relationship_type: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}
