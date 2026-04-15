//! A2A (Agent-to-Agent) protocol implementation

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::a2a::message::{A2AMessage, MessageType};

/// A2A protocol handler
pub struct A2AProtocol {
    agent_id: String,
    handlers: HashMap<MessageType, Box<dyn MessageHandler>>,
    #[allow(dead_code)]
    pending_deals: HashMap<String, DealState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealState {
    pub deal_id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub service_id: String,
    pub price: u64,
    pub status: DealStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DealStatus {
    Proposed,
    Accepted,
    Funded,
    InProgress,
    Completed,
    Disputed,
    Cancelled,
}

pub trait MessageHandler: Send + Sync {
    fn handle(&self, message: &A2AMessage) -> Result<A2AMessage, A2AError>;
}

/// Task status for A2A tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum A2AError {
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Unauthorized sender")]
    Unauthorized,
    #[error("Deal not found")]
    DealNotFound,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Agent not found")]
    AgentNotFound,
    #[error("No valid endpoint")]
    NoValidEndpoint,
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Security error: {0}")]
    Security(String),
    #[error("Transport error: {0}")]
    Transport(String),
}

impl A2AProtocol {
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            handlers: HashMap::new(),
            pending_deals: HashMap::new(),
        }
    }

    /// Register message handler
    pub fn register_handler(&mut self, msg_type: MessageType, handler: Box<dyn MessageHandler>) {
        self.handlers.insert(msg_type, handler);
    }

    /// Process incoming message
    pub fn process_message(&self, message: A2AMessage) -> Result<A2AMessage, A2AError> {
        // Verify signature
        self.verify_signature(&message)?;

        // Route to handler
        if let Some(handler) = self.handlers.get(&message.msg_type) {
            handler.handle(&message)
        } else {
            Err(A2AError::InvalidFormat)
        }
    }

    /// Create service proposal
    pub fn create_proposal(
        &self,
        recipient: impl AsRef<str>,
        service_id: impl Into<String>,
        price: u64,
    ) -> A2AMessage {
        use super::message::MessagePayload;
        A2AMessage::new(
            MessageType::Negotiate,
            crate::types::AgentId::from_string(&self.agent_id),
            Some(crate::types::AgentId::from_string(recipient)),
            MessagePayload::Negotiate {
                offer: super::message::NegotiationOffer {
                    service: service_id.into(),
                    price,
                    terms: vec![],
                    valid_until: now() + 3600,
                },
            },
        )
    }

    /// Accept proposal
    pub fn accept_proposal(&self, deal_id: impl Into<String>) -> A2AMessage {
        use super::message::MessagePayload;
        A2AMessage::new(
            MessageType::Response,
            crate::types::AgentId::from_string(&self.agent_id),
            None, // broadcast
            MessagePayload::Response {
                success: true,
                data: Some(serde_json::json!({
                    "deal_id": deal_id.into(),
                    "accepted": true,
                })),
                error: None,
            },
        )
    }

    /// Verify message signature
    fn verify_signature(&self, message: &A2AMessage) -> Result<(), A2AError> {
        // In production, verify Ed25519 signature
        if message.signature.is_none() {
            return Err(A2AError::Unauthorized);
        }
        Ok(())
    }

    /// Negotiate price
    pub fn negotiate(
        &self,
        recipient: impl AsRef<str>,
        deal_id: impl Into<String>,
        counter_offer: u64,
    ) -> A2AMessage {
        use super::message::{MessagePayload, NegotiationOffer};
        A2AMessage::new(
            MessageType::Negotiate,
            crate::types::AgentId::from_string(&self.agent_id),
            Some(crate::types::AgentId::from_string(recipient)),
            MessagePayload::Negotiate {
                offer: NegotiationOffer {
                    service: deal_id.into(),
                    price: counter_offer,
                    terms: vec![],
                    valid_until: now() + 3600,
                },
            },
        )
    }
}

/// Service discovery handler
pub struct DiscoveryHandler;

impl MessageHandler for DiscoveryHandler {
    fn handle(&self, message: &A2AMessage) -> Result<A2AMessage, A2AError> {
        use super::message::MessagePayload;
        // Respond with available services
        Ok(A2AMessage::new(
            MessageType::Response,
            message.from.clone(),
            Some(message.from.clone()),
            MessagePayload::Response {
                success: true,
                data: Some(serde_json::json!({
                    "services": ["coding", "analysis", "communication"],
                })),
                error: None,
            },
        ))
    }
}

/// Commerce handler for deals
pub struct CommerceHandler {
    #[allow(dead_code)]
    escrow: Box<dyn Escrow>,
}

pub trait Escrow: Send + Sync {
    fn create_escrow(&self, buyer: &str, seller: &str, amount: u64) -> Result<String, A2AError>;
    fn release(&self, escrow_id: &str) -> Result<(), A2AError>;
    fn refund(&self, escrow_id: &str) -> Result<(), A2AError>;
}

impl MessageHandler for CommerceHandler {
    fn handle(&self, message: &A2AMessage) -> Result<A2AMessage, A2AError> {
        use super::message::MessagePayload;
        match message.msg_type {
            MessageType::Negotiate => {
                // Handle deal proposal
                Ok(A2AMessage::new(
                    MessageType::Response,
                    message.from.clone(),
                    Some(message.from.clone()),
                    MessagePayload::Response {
                        success: true,
                        data: Some(serde_json::json!({"status": "received"})),
                        error: None,
                    },
                ))
            }
            _ => Err(A2AError::InvalidFormat),
        }
    }
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
