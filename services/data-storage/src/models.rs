use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    pub id: Uuid,
    pub source_module: Uuid,
    pub scan_id: Option<Uuid>,
    pub entity_type: String,
    pub value: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreDataRequest {
    pub source_module: Uuid,
    pub scan_id: Option<Uuid>,
    pub entity_type: String,
    pub value: String,
    pub data: serde_json::Value,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    pub entity_type: Option<String>,
    pub value: Option<String>,
    pub source_module: Option<Uuid>,
    pub scan_id: Option<Uuid>,
    pub from_date: Option<DateTime<Utc>>,
    pub to_date: Option<DateTime<Utc>>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
    pub data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreRelationshipRequest {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub relationship_type: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub confidence: u8,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: Uuid,
    pub relationship_type: String,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub source_value: String,
    pub target_value: String,
    pub data: HashMap<String, serde_json::Value>,
    pub confidence: u8,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityRequest {
    pub entity_type: String,
    pub value: String,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub metadata: Option<HashMap<String, String>>,
    pub confidence: Option<u8>,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityRequest {
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub metadata: Option<HashMap<String, String>>,
    pub confidence: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationshipRequest {
    pub relationship_type: String,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub confidence: Option<u8>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRelationshipRequest {
    pub data: Option<HashMap<String, serde_json::Value>>,
    pub confidence: Option<u8>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResponse {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
    pub data: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, String>,
    pub confidence: u8,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
    pub relationship_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipResponse {
    pub id: Uuid,
    pub relationship_type: String,
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub source_entity: Option<EntitySummary>,
    pub target_entity: Option<EntitySummary>,
    pub data: HashMap<String, serde_json::Value>,
    pub confidence: u8,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub id: Uuid,
    pub entity_type: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchParams {
    pub entity_type: Option<String>,
    pub value_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub confidence_min: Option<u8>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSearchParams {
    pub relationship_type: Option<String>,
    pub source_id: Option<Uuid>,
    pub target_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    pub confidence_min: Option<u8>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub pages: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportData {
    pub entities: Vec<CreateEntityRequest>,
    pub relationships: Vec<CreateRelationshipRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub entities_created: usize,
    pub entities_skipped: usize,
    pub relationships_created: usize,
    pub relationships_skipped: usize,
    pub errors: Vec<String>,
}
