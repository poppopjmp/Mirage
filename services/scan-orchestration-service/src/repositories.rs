//! Database repositories for scan orchestration service

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRecord {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub targets: Vec<String>,
    pub modules: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ScanRepository {
    // Placeholder - in real implementation would contain database pool
    scans: HashMap<Uuid, ScanRecord>,
}

impl ScanRepository {
    pub fn new() -> Self {
        Self {
            scans: HashMap::new(),
        }
    }

    pub async fn create_scan(
        &mut self,
        scan: ScanRecord,
    ) -> Result<ScanRecord, Box<dyn std::error::Error>> {
        self.scans.insert(scan.id, scan.clone());
        Ok(scan)
    }

    pub async fn get_scan(
        &self,
        id: Uuid,
    ) -> Result<Option<ScanRecord>, Box<dyn std::error::Error>> {
        Ok(self.scans.get(&id).cloned())
    }

    pub async fn list_scans(&self) -> Result<Vec<ScanRecord>, Box<dyn std::error::Error>> {
        Ok(self.scans.values().cloned().collect())
    }

    pub async fn update_scan_status(
        &mut self,
        id: Uuid,
        status: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(scan) = self.scans.get_mut(&id) {
            scan.status = status;
            scan.updated_at = chrono::Utc::now();
        }
        Ok(())
    }

    pub async fn delete_scan(&mut self, id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        self.scans.remove(&id);
        Ok(())
    }
}
