//! Spoke Agent

use super::CollabMessage;
use crate::error::Result;
use uuid::Uuid;

/// Spoke agent in hub-spoke collaboration
pub struct SpokeAgent {
    id: Uuid,
    hub_id: Option<Uuid>,
}

impl SpokeAgent {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            hub_id: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn connect(&mut self, hub_id: Uuid) {
        self.hub_id = Some(hub_id);
        tracing::info!("Spoke {} connected to hub {}", self.id, hub_id);
    }

    pub async fn send_to_hub(&self, message: CollabMessage) -> Result<()> {
        if let Some(hub) = self.hub_id {
            tracing::info!("Sending message to hub {}", hub);
            Ok(())
        } else {
            Err(crate::error::AgentError::configuration("Not connected to hub"))
        }
    }

    pub fn disconnect(&mut self) {
        self.hub_id = None;
    }
}

impl Default for SpokeAgent {
    fn default() -> Self {
        Self::new()
    }
}
