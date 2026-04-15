//! P2P message types

use serde::{Deserialize, Serialize};

/// P2P message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub msg_type: MessageType,
    /// Topic
    pub topic: String,
    /// Payload
    pub payload: Vec<u8>,
    /// Sender
    pub sender: String,
    /// Timestamp
    pub timestamp: u64,
    /// TTL in seconds
    pub ttl: u32,
    /// Target
    #[serde(skip)]
    pub target: crate::MessageTarget,
}

/// Message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Agent heartbeat
    Heartbeat,
    /// Task announcement
    TaskAnnouncement,
    /// Task result
    TaskResult,
    /// Capability advertisement
    CapabilityAdvert,
    /// Discovery request
    DiscoveryRequest,
    /// Discovery response
    DiscoveryResponse,
    /// Direct message
    DirectMessage,
    /// Broadcast event
    Event,
}

impl P2PMessage {
    /// Create new message
    pub fn new(msg_type: MessageType, topic: impl Into<String>, payload: Vec<u8>) -> Self {
        use uuid::Uuid;
        use std::time::{SystemTime, UNIX_EPOCH};

        Self {
            id: Uuid::new_v4().to_string(),
            msg_type,
            topic: topic.into(),
            payload,
            sender: String::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            ttl: 300, // 5 minutes default
            target: crate::MessageTarget::Broadcast,
        }
    }

    /// Set sender
    pub fn from(mut self, sender: impl Into<String>) -> Self {
        self.sender = sender.into();
        self
    }

    /// Set TTL
    pub fn with_ttl(mut self, ttl: u32) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set target
    pub fn to(mut self, target: crate::MessageTarget) -> Self {
        self.target = target;
        self
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    /// Check if message is expired
    pub fn is_expired(&self) -> bool {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.timestamp + self.ttl as u64
    }
}

/// Agent announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAnnouncement {
    pub agent_id: String,
    pub capabilities: Vec<String>,
    pub services: Vec<String>,
    pub endpoint: Option<String>,
}

/// Task announcement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAnnouncement {
    pub task_id: String,
    pub task_type: String,
    pub requirements: TaskRequirements,
    pub reward: u64,
    pub deadline: u64,
}

/// Task requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub min_capabilities: Vec<String>,
    pub min_reputation: u32,
    pub max_latency_ms: u32,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub agent_id: String,
    pub success: bool,
    pub output: Vec<u8>,
    pub execution_time_ms: u64,
}

/// Capability query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityQuery {
    pub query_id: String,
    pub capabilities: Vec<String>,
    pub max_results: usize,
}

/// Capability response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    pub query_id: String,
    pub agents: Vec<AgentInfo>,
}

/// Agent info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub agent_id: String,
    pub peer_id: String,
    pub capabilities: Vec<String>,
    pub reputation: u32,
}
