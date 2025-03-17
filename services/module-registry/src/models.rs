use chrono::{DateTime, Utc};
use mirage_common::models::Module as CommonModule;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use mirage_common::models::ParameterDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleModel {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub configuration: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ModuleModel> for CommonModule {
    fn from(model: ModuleModel) -> Self {
        CommonModule {
            id: model.id,
            name: model.name,
            version: model.version,
            description: model.description,
            author: model.author,
            dependencies: model.dependencies,
            capabilities: model.capabilities,
            configuration: model.configuration,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub capabilities: Option<Vec<String>>,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub parameters: HashMap<String, ParameterDefinition>,
    pub required_capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub status: ModuleStatus,
    pub file_path: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    Active,
    Disabled,
    Deprecated,
    Testing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUploadRequest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub capabilities: Vec<String>,
    pub required_capabilities: Vec<String>,
    pub parameters: Option<HashMap<String, ParameterDefinition>>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleResponse {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub parameters: HashMap<String, ParameterDefinition>,
    pub required_capabilities: Vec<String>,
    pub metadata: HashMap<String, String>,
    pub status: ModuleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleQueryParams {
    pub name: Option<String>,
    pub capability: Option<String>,
    pub author: Option<String>,
    pub status: Option<ModuleStatus>,
    pub page: Option<u64>,
    pub per_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUpdateRequest {
    pub description: Option<String>,
    pub parameters: Option<HashMap<String, ParameterDefinition>>,
    pub metadata: Option<HashMap<String, String>>,
    pub status: Option<ModuleStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
