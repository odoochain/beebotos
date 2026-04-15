//! A2A Protocol E2E Tests
//!
//! 🟠 HIGH FIX: Updated to use current A2AClient API

use beebotos_agents::a2a::{A2AClient, A2AMessage, MessageType};
use beebotos_agents::types::AgentId;

#[tokio::test]
async fn test_a2a_client_creation() {
    // 🟠 HIGH FIX: A2AClient::new() now returns Result
    let client = A2AClient::new().expect("Failed to create A2A client");
    
    // Verify discovery service is available
    let discovery = client.discovery();
    assert!(discovery.find_agent_by_id("nonexistent").is_none());
}

#[tokio::test]
async fn test_a2a_message_creation() {
    // Create agents
    let agent1 = AgentId::from_string("agent1");
    let agent2 = AgentId::from_string("agent2");
    
    // Create message
    let message = A2AMessage::new(
        MessageType::Request,
        agent1,
        Some(agent2),
        beebotos_agents::a2a::message::MessagePayload::Request {
            action: "test".to_string(),
            params: serde_json::json!({"key": "value"}).as_object().unwrap().clone(),
        },
    );
    
    assert_eq!(message.msg_type, MessageType::Request);
    assert!(message.signature.is_none());
}

#[tokio::test]
async fn test_a2a_message_signing() {
    let client = A2AClient::new().expect("Failed to create A2A client");
    
    let agent1 = AgentId::from_string("agent1");
    let agent2 = AgentId::from_string("agent2");
    
    let message = A2AMessage::new(
        MessageType::Request,
        agent1,
        Some(agent2),
        beebotos_agents::a2a::message::MessagePayload::Request {
            action: "test".to_string(),
            params: serde_json::json!({}).as_object().unwrap().clone(),
        },
    );
    
    // 🟠 HIGH FIX: signature is now Option<Vec<u8>>
    let signed_message = message.sign(vec![1, 2, 3, 4]);
    assert!(signed_message.signature.is_some());
    assert_eq!(signed_message.signature.unwrap(), vec![1, 2, 3, 4]);
}

// TODO: Implement tests for the following A2A features:
// - Capability announcement and discovery
// - Intent-based negotiation
// - Task assignment and execution
// These require additional A2AClient methods to be implemented.
