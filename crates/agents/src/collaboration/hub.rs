//! Collaboration Hub

use super::{CollabMessage, CollabMessageType};
use crate::error::Result;
use std::collections::HashMap;
use uuid::Uuid;

/// Central hub for agent collaboration
pub struct CollaborationHub {
    agents: HashMap<Uuid, AgentInfo>,
}

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: Uuid,
    pub name: String,
    pub capabilities: Vec<String>,
}

impl CollaborationHub {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register(&mut self, info: AgentInfo) {
        self.agents.insert(info.id, info);
    }

    pub fn unregister(&mut self, id: Uuid) {
        self.agents.remove(&id);
    }

    pub async fn broadcast(&self, message: CollabMessage) -> Result<()> {
        for (id, _) in &self.agents {
            if Some(*id) != message.to {
                // Send to each agent
                tracing::info!("Broadcasting to {}", id);
            }
        }
        Ok(())
    }

    pub fn find_by_capability(&self, capability: &str) -> Vec<&AgentInfo> {
        self.agents
            .values()
            .filter(|a| a.capabilities.contains(&capability.to_string()))
            .collect()
    }
}

impl Default for CollaborationHub {
    fn default() -> Self {
        Self::new()
    }
}
