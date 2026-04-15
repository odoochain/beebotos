//! Lifecycle Events

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    pub agent_id: Uuid,
    pub event_type: LifecycleEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEventType {
    Created,
    Initialized,
    Started,
    Paused,
    Resumed,
    Stopping,
    Stopped,
    Destroyed,
}

/// Lifecycle tracker
pub struct LifecycleTracker {
    events: Vec<LifecycleEvent>,
}

impl LifecycleTracker {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn record(&mut self, agent_id: Uuid, event_type: LifecycleEventType) {
        self.events.push(LifecycleEvent {
            agent_id,
            event_type,
            timestamp: chrono::Utc::now(),
        });
    }

    pub fn history(&self) -> &[LifecycleEvent] {
        &self.events
    }
}

impl Default for LifecycleTracker {
    fn default() -> Self {
        Self::new()
    }
}
