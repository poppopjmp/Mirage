use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Represents a target in the system with id, name, and description.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    id: String,
    name: String,
    description: String,
}

impl Target {
    /// Create a new target with the given id, name, and description.
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Target {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        }
    }

    /// Get the target id.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the target name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the target description.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Update the target name.
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    /// Update the target description.
    pub fn set_description(&mut self, description: &str) {
        self.description = description.to_string();
    }
}

/// Manages targets, including adding, retrieving, and removing targets.
#[derive(Debug)]
pub struct TargetManager {
    targets: HashMap<String, Target>,
}

impl TargetManager {
    /// Create a new target manager.
    pub fn new() -> Self {
        TargetManager {
            targets: HashMap::new(),
        }
    }

    /// Add a target to the manager.
    pub fn add_target(&mut self, target: Target) -> bool {
        if self.targets.contains_key(&target.id) {
            return false;
        }
        self.targets.insert(target.id().to_string(), target);
        true
    }

    /// Get a target by id.
    pub fn get_target(&self, id: &str) -> Option<&Target> {
        self.targets.get(id)
    }

    /// Remove a target by id.
    pub fn remove_target(&mut self, id: &str) -> bool {
        self.targets.remove(id).is_some()
    }

    /// List all targets.
    pub fn list_targets(&self) -> Vec<&Target> {
        self.targets.values().collect()
    }

    /// Count targets.
    pub fn count_targets(&self) -> usize {
        self.targets.len()
    }

    /// Update a target's name and description.
    pub fn update_target(&mut self, id: &str, name: &str, description: &str) -> bool {
        if let Some(target) = self.targets.get_mut(id) {
            target.set_name(name);
            target.set_description(description);
            true
        } else {
            false
        }
    }
}

impl Default for TargetManager {
    fn default() -> Self {
        Self::new()
    }
}
