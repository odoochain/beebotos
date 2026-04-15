//! Heartbeat Scheduler
//!
//! Wakes agents every 30 minutes for proactive reasoning (OpenClaw feature).

use std::collections::HashMap;
use std::time::Duration;

use chrono::Timelike;
use serde::{Deserialize, Serialize};
use tokio::time::interval;

use crate::session::SessionKey;

/// Heartbeat scheduler
pub struct HeartbeatScheduler {
    interval: Duration,
    active_hours: (u8, u8),
    checklists: HashMap<String, HeartbeatChecklist>,
    handlers: Vec<Box<dyn HeartbeatHandler>>,
}

/// Heartbeat checklist (HEARTBEAT.md content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatChecklist {
    pub path: String,
    pub items: Vec<CheckItem>,
    pub last_check: u64,
}

/// Checklist item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckItem {
    pub id: String,
    pub description: String,
    pub priority: Priority,
    pub completed: bool,
}

/// Priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Heartbeat handler trait
#[async_trait::async_trait]
pub trait HeartbeatHandler: Send + Sync {
    async fn on_heartbeat(&self, session_key: &SessionKey, checklist: &HeartbeatChecklist);
}

impl HeartbeatScheduler {
    /// Default heartbeat interval (30 minutes)
    pub const DEFAULT_INTERVAL: Duration = Duration::from_secs(30 * 60);

    pub fn new() -> Self {
        Self {
            interval: Self::DEFAULT_INTERVAL,
            active_hours: (0, 23),
            checklists: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_active_hours(mut self, start: u8, end: u8) -> Self {
        self.active_hours = (start, end);
        self
    }

    pub fn register(&mut self, session_key: SessionKey, checklist: HeartbeatChecklist) {
        self.checklists.insert(session_key.to_string(), checklist);
    }

    pub fn add_handler(&mut self, handler: Box<dyn HeartbeatHandler>) {
        self.handlers.push(handler);
    }

    pub async fn start(&self) {
        let mut ticker = interval(self.interval);

        loop {
            ticker.tick().await;
            self.process_heartbeats().await;
        }
    }

    async fn process_heartbeats(&self) {
        let current_hour = chrono::Local::now().hour() as u8;

        // Check if within active hours
        if current_hour < self.active_hours.0 || current_hour > self.active_hours.1 {
            return;
        }

        for (session_key_str, checklist) in &self.checklists {
            if let Ok(session_key) = SessionKey::parse(session_key_str) {
                for handler in &self.handlers {
                    handler.on_heartbeat(&session_key, checklist).await;
                }
            }
        }
    }
}

impl Default for HeartbeatScheduler {
    fn default() -> Self {
        Self::new()
    }
}
