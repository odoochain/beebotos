//! Thread Management
//!
//! Manages conversation threads across platforms.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::{Message, PlatformType, Result};

/// Conversation thread
#[derive(Debug, Clone)]
pub struct Thread {
    pub id: Uuid,
    pub platform: PlatformType,
    pub external_id: String,
    pub participants: Vec<String>,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

impl Thread {
    pub fn new(platform: PlatformType, external_id: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            platform,
            external_id: external_id.into(),
            participants: Vec::new(),
            messages: Vec::new(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }

    pub fn add_participant(&mut self, participant: impl Into<String>) {
        let p = participant.into();
        if !self.participants.contains(&p) {
            self.participants.push(p);
        }
    }
}

/// Thread manager
pub struct ThreadManager {
    threads: HashMap<Uuid, Thread>,
    platform_index: HashMap<(PlatformType, String), Uuid>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            threads: HashMap::new(),
            platform_index: HashMap::new(),
        }
    }

    /// Create a new thread
    pub fn create_thread(
        &mut self,
        platform: PlatformType,
        external_id: impl Into<String>,
    ) -> &Thread {
        let thread = Thread::new(platform, external_id);
        let id = thread.id;
        let key = (platform, thread.external_id.clone());

        self.threads.insert(id, thread);
        self.platform_index.insert(key, id);

        self.threads.get(&id).unwrap()
    }

    /// Get thread by ID
    pub fn get_thread(&self, id: Uuid) -> Option<&Thread> {
        self.threads.get(&id)
    }

    /// Get thread by platform and external ID
    pub fn get_by_external(&self, platform: PlatformType, external_id: &str) -> Option<&Thread> {
        self.platform_index
            .get(&(platform, external_id.to_string()))
            .and_then(|id| self.threads.get(id))
    }

    /// Add message to thread
    pub fn add_message(&mut self, thread_id: Uuid, message: Message) -> Result<()> {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            thread.add_message(message);
            Ok(())
        } else {
            Err(crate::error::AgentError::AgentNotFound(format!(
                "Thread {}",
                thread_id
            )))
        }
    }

    /// List all threads
    pub fn list_threads(&self) -> Vec<&Thread> {
        self.threads.values().collect()
    }

    /// Archive old threads
    pub fn archive_old_threads(&mut self, before: DateTime<Utc>) -> usize {
        let to_archive: Vec<Uuid> = self
            .threads
            .iter()
            .filter(|(_, t)| t.updated_at < before)
            .map(|(id, _)| *id)
            .collect();

        for id in &to_archive {
            if let Some(thread) = self.threads.get_mut(id) {
                thread
                    .metadata
                    .insert("archived".to_string(), "true".to_string());
            }
        }

        to_archive.len()
    }
}

impl Default for ThreadManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_creation() {
        let mut manager = ThreadManager::new();
        let thread = manager.create_thread(PlatformType::Slack, "C12345");
        assert_eq!(thread.platform, PlatformType::Slack);
        assert_eq!(thread.external_id, "C12345");
    }

    #[test]
    fn test_get_by_external() {
        let mut manager = ThreadManager::new();
        manager.create_thread(PlatformType::Slack, "C12345");

        let found = manager.get_by_external(PlatformType::Slack, "C12345");
        assert!(found.is_some());
    }
}
