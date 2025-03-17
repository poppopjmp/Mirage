use crate::models::{ModuleModel, CreateModuleRequest, UpdateModuleRequest};
use crate::repositories::{DbPool, ModuleRepository};
use crate::config::ModuleStorageConfig;
use chrono::Utc;
use mirage_common::{Error, Result, models::Module};
use semver::Version;
use uuid::Uuid;
use std::sync::Arc;

use crate::models::{Module, ModuleStatus, ModuleUploadRequest, ModuleValidationResult, ModuleUpdateRequest};
use crate::repositories::ModuleRepository;
use crate::config::AppConfig;
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write};
use tokio::task;
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct ModuleService {
    repo: Arc<ModuleRepository>,
}

impl ModuleService {
    pub fn new(pool: DbPool, storage_config: ModuleStorageConfig) -> Self {
        Self {
            repo: Arc::new(ModuleRepository::new(pool, storage_config)),
        }
    }

    pub async fn list_modules(&self, limit: i64, offset: i64) -> Result<Vec<Module>> {
        let modules = self.repo.find_all(limit, offset).await?;
        Ok(modules.into_iter().map(|m| m.into()).collect())
    }

    pub async fn list_modules_by_capability(&self, capability: &str, limit: i64, offset: i64) -> Result<Vec<Module>> {
        let modules = self.repo.find_by_capability(capability, limit, offset).await?;
        Ok(modules.into_iter().map(|m| m.into()).collect())
    }

    pub async fn get_module(&self, id: &Uuid) -> Result<Module> {
        let module = self.repo.find_by_id(id).await?
            .ok_or_else(|| Error::NotFound(format!("Module with ID {} not found", id)))?;
        
        Ok(module.into())
    }

    pub async fn register_module(&self, req: CreateModuleRequest) -> Result<Module> {
        // Validate semver format
        let _ = Version::parse(&req.version)
            .map_err(|_| Error::Validation(format!("Invalid version format: {}", req.version)))?;
        
        // Check if a module with this name already exists
        if let Some(existing) = self.repo.find_by_name(&req.name).await? {
            // If it exists, compare versions
            let existing_version = Version::parse(&existing.version)
                .map_err(|_| Error::Internal(format!("Invalid version in database: {}", existing.version)))?;
            
            let new_version = Version::parse(&req.version)
                .map_err(|_| Error::Validation(format!("Invalid version format: {}", req.version)))?;
            
            if new_version <= existing_version {
                return Err(Error::Validation(format!(
                    "Module version {} is not newer than existing version {}", 
                    req.version, existing.version
                )));
            }
        }
        
        // Create new module
        let module = ModuleModel {
            id: Uuid::new_v4(),
            name: req.name,
            version: req.version,
            description: req.description,
            author: req.author,
            dependencies: req.dependencies,
            capabilities: req.capabilities,
            configuration: req.configuration,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created = self.repo.create(&module).await?;
        Ok(created.into())
    }

    pub async fn update_module(&self, id: &Uuid, req: UpdateModuleRequest) -> Result<Module> {
        // Get existing module
        let mut module = self.repo.find_by_id(id).await?
            .ok_or_else(|| Error::NotFound(format!("Module with ID {} not found", id)))?;
        
        // If version is provided, validate semver format
        if let Some(ref version) = req.version {
            let _ = Version::parse(version)
                .map_err(|_| Error::Validation(format!("Invalid version format: {}", version)))?;
            
            module.version = version.clone();
        }

        // Update other fields if provided
        if let Some(description) = req.description {
            module.description = description;
        }
        
        if let Some(dependencies) = req.dependencies {
            module.dependencies = dependencies;
        }
        
        if let Some(capabilities) = req.capabilities {
            module.capabilities = capabilities;
        }
        
        if let Some(configuration) = req.configuration {
            module.configuration = configuration;
        }

        let updated = self.repo.update(&module).await?;
        Ok(updated.into())
    }

    pub async fn delete_module(&self, id: &Uuid) -> Result<()> {
        let deleted = self.repo.delete(id).await?;
        
        if deleted {
            Ok(())
        } else {
            Err(Error::NotFound(format!("Module with ID {} not found", id)))
        }
    }
}

#[derive(Clone)]
pub struct ModuleRegistryService {
    repo: Arc<ModuleRepository>,
    config: Arc<AppConfig>,
}

impl ModuleRegistryService {
    pub fn new(repo: ModuleRepository, config: AppConfig) -> Self {
        Self {
            repo: Arc::new(repo),
            config: Arc::new(config),
        }
    }
    
    pub async fn register_module(&self, request: ModuleUploadRequest, module_data: Vec<u8>) -> Result<Module> {
        // Validate the module
        let validation = self.validate_module(&module_data, &request).await?;
        if (!validation.valid) {
            return Err(Error::Validation(format!(
                "Module validation failed: {:?}", 
                validation.errors
            )));
        }
        
        // Check if module with same name and version already exists
        if let Some(existing) = self.repo.find_by_name_version(&request.name, &request.version).await? {
            return Err(Error::Conflict(format!(
                "Module {} version {} already exists", 
                request.name, request.version
            )));
        }
        
        // Create module ID and file path
        let module_id = Uuid::new_v4();
        let file_name = format!(
            "{}_{}_v{}.wasm", 
            sanitize_filename(&request.name), 
            module_id, 
            sanitize_filename(&request.version)
        );
        let file_path = PathBuf::from(&self.config.storage.path).join(&file_name);
        
        // Create storage directory if it doesn't exist
        let storage_path = Path::new(&self.config.storage.path);
        if (!storage_path.exists()) {
            fs::create_dir_all(storage_path)
                .map_err(|e| Error::Internal(format!("Failed to create storage directory: {}", e)))?;
        }
        
        // Calculate hash
        let hash = calculate_sha256(&module_data);
        
        // Save module file
        let mut file = File::create(&file_path)
            .map_err(|e| Error::Internal(format!("Failed to create module file: {}", e)))?;
        file.write_all(&module_data)
            .map_err(|e| Error::Internal(format!("Failed to write module data: {}", e)))?;
        
        // Create module record
        let module = Module {
            id: module_id,
            name: request.name,
            version: request.version,
            description: request.description,
            author: request.author,
            license: request.license.unwrap_or_else(|| "Unknown".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            capabilities: request.capabilities,
            parameters: request.parameters.unwrap_or_default(),
            required_capabilities: request.required_capabilities,
            metadata: request.metadata.unwrap_or_default(),
            status: ModuleStatus::Testing, // New modules start in testing status
            file_path: file_name,
            hash,
        };
        
        // Save to repository
        self.repo.save(&module).await?;
        
        Ok(module)
    }
    
    pub async fn get_module(&self, id: &Uuid) -> Result<Module> {
        self.repo.find_by_id(id).await?
            .ok_or_else(|| Error::NotFound(format!("Module with ID {} not found", id)))
    }
    
    pub async fn list_modules(
        &self,
        name: Option<&str>,
        capability: Option<&str>,
        author: Option<&str>,
        status: Option<&ModuleStatus>,
        page: u64,
        per_page: u64,
    ) -> Result<(Vec<Module>, u64)> {
        self.repo.find_all(name, capability, author, status, page, per_page).await
    }
    
    pub async fn update_module(&self, id: &Uuid, update: ModuleUpdateRequest) -> Result<Module> {
        // Get existing module
        let mut module = self.get_module(id).await?;
        
        // Update fields
        if let Some(description) = update.description {
            module.description = description;
        }
        
        if let Some(parameters) = update.parameters {
            module.parameters = parameters;
        }
        
        if let Some(metadata) = update.metadata {
            // Merge metadata rather than replace
            for (k, v) in metadata {
                module.metadata.insert(k, v);
            }
        }
        
        if let Some(status) = update.status {
            module.status = status;
        }
        
        module.updated_at = Utc::now();
        
        // Save updated module
        self.repo.update(&module).await?;
        
        Ok(module)
    }
    
    pub async fn delete_module(&self, id: &Uuid) -> Result<()> {
        // Get module to check if it exists and get file path
        let module = self.get_module(id).await?;
        
        // Delete from repository
        self.repo.delete(id).await?;
        
        // Delete file
        let file_path = PathBuf::from(&self.config.storage.path).join(&module.file_path);
        if (file_path.exists()) {
            fs::remove_file(file_path)
                .map_err(|e| Error::Internal(format!("Failed to delete module file: {}", e)))?;
        }
        
        Ok(())
    }
    
    pub async fn download_module(&self, id: &Uuid) -> Result<(Module, Vec<u8>)> {
        // Get module
        let module = self.get_module(id).await?;
        
        // Read module file
        let file_path = PathBuf::from(&self.config.storage.path).join(&module.file_path);
        let mut file = File::open(&file_path)
            .map_err(|e| Error::Internal(format!("Failed to open module file: {}", e)))?;
            
        let mut data = Vec::new();
        file.read_to_end(&mut data)
            .map_err(|e| Error::Internal(format!("Failed to read module file: {}", e)))?;
            
        // Verify hash
        let hash = calculate_sha256(&data);
        if (hash != module.hash) {
            return Err(Error::Internal("Module file integrity check failed".to_string()));
        }
        
        Ok((module, data))
    }
    
    async fn validate_module(&self, module_data: &[u8], metadata: &ModuleUploadRequest) -> Result<ModuleValidationResult> {
        // Offload CPU-intensive validation to a separate thread
        let data = module_data.to_vec();
        let metadata_clone = metadata.clone();
        
        let validation_result = task::spawn_blocking(move || {
            // This would contain actual WebAssembly validation logic
            // For now, we'll do some basic checks
            
            let mut errors = Vec::new();
            let mut warnings = Vec::new();
            
            // Check module data size
            if (data.len() < 100) {
                errors.push("Module file is too small to be valid".to_string());
            }
            
            // Check if the file is a valid WebAssembly module
            if (data.len() < 8 || &data[0..8] != &[0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]) {
                errors.push("File does not have a valid WebAssembly header".to_string());
            }
            
            // Check metadata
            if (metadata_clone.name.is_empty()) {
                errors.push("Module name cannot be empty".to_string());
            }
            
            if (metadata_clone.version.is_empty()) {
                errors.push("Module version cannot be empty".to_string());
            }
            
            if (metadata_clone.capabilities.is_empty()) {
                warnings.push("Module has no declared capabilities".to_string());
            }
            
            // Additional validation would be performed here
            
            ModuleValidationResult {
                valid: errors.is_empty(),
                errors,
                warnings,
            }
        }).await
        .map_err(|e| Error::Internal(format!("Module validation failed: {}", e)))?;
        
        Ok(validation_result)
    }
}

// Helper functions
fn sanitize_filename(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if (c.is_alphanumeric() || c == '-' || c == '_' || c == '.') { c } else { '_' })
        .collect()
}

fn calculate_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    format!("{:x}", hash)
}
