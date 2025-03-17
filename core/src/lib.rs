pub mod event;
pub mod helpers;
pub mod target;

use event::{Event, EventHandler};
use helpers::{is_valid_email, is_valid_ip, is_valid_url};
use target::{Target, TargetManager};

pub struct Core {
    event_handler: EventHandler,
    target_manager: TargetManager,
}

impl Core {
    pub fn new() -> Self {
        Self {
            event_handler: EventHandler::new(),
            target_manager: TargetManager::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) {
        self.event_handler.add_event(event);
    }

    pub fn get_events(&self, event_type: &str) -> Option<&Vec<Event>> {
        self.event_handler.get_events(event_type)
    }

    pub fn add_target(&mut self, target: Target) {
        self.target_manager.add_target(target);
    }

    pub fn get_target(&self, target_id: &str) -> Option<&Target> {
        self.target_manager.get_target(target_id)
    }
}
