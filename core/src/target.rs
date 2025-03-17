use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Target {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl Target {
    pub fn new(id: &str, name: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
        }
    }
}

pub struct TargetManager {
    targets: HashMap<String, Target>,
}

impl TargetManager {
    pub fn new() -> Self {
        Self {
            targets: HashMap::new(),
        }
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.insert(target.id.clone(), target);
    }

    pub fn get_target(&self, target_id: &str) -> Option<&Target> {
        self.targets.get(target_id)
    }

    pub fn remove_target(&mut self, target_id: &str) {
        self.targets.remove(target_id);
    }

    pub fn list_targets(&self) -> Vec<&Target> {
        self.targets.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_target() {
        let mut manager = TargetManager::new();
        let target = Target::new("1", "Test Target", "This is a test target.");
        manager.add_target(target.clone());

        let retrieved_target = manager.get_target("1").unwrap();
        assert_eq!(retrieved_target, &target);
    }

    #[test]
    fn test_get_target() {
        let mut manager = TargetManager::new();
        let target = Target::new("1", "Test Target", "This is a test target.");
        manager.add_target(target.clone());

        let retrieved_target = manager.get_target("1").unwrap();
        assert_eq!(retrieved_target, &target);
    }

    #[test]
    fn test_remove_target() {
        let mut manager = TargetManager::new();
        let target = Target::new("1", "Test Target", "This is a test target.");
        manager.add_target(target.clone());

        manager.remove_target("1");
        assert!(manager.get_target("1").is_none());
    }

    #[test]
    fn test_list_targets() {
        let mut manager = TargetManager::new();
        let target1 = Target::new("1", "Test Target 1", "This is a test target 1.");
        let target2 = Target::new("2", "Test Target 2", "This is a test target 2.");
        manager.add_target(target1.clone());
        manager.add_target(target2.clone());

        let targets = manager.list_targets();
        assert_eq!(targets.len(), 2);
        assert!(targets.contains(&&target1));
        assert!(targets.contains(&&target2));
    }
}
