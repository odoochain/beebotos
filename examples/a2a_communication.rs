//! A2A Communication Example
//!
//! Demonstrates agent-to-agent communication.

use beebot_agents::a2a::*;
use beebot_agents::types::AgentId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create A2A client
    let mut client = A2AClient::new().expect("Failed to create A2A client");
    
    // Register agent endpoints
    let agent1 = A2AEndpoint {
        agent_id: AgentId::new(),
        url: "http://localhost:9001".to_string(),
        capabilities: vec!["compute".to_string(), "storage".to_string()],
        reputation_score: 8000,
        last_seen: now(),
    };
    
    let agent2 = A2AEndpoint {
        agent_id: AgentId::new(),
        url: "http://localhost:9002".to_string(),
        capabilities: vec!["analyze".to_string(), "report".to_string()],
        reputation_score: 7500,
        last_seen: now(),
    };
    
    client.register_endpoint(agent1.clone());
    client.register_endpoint(agent2.clone());
    
    println!("✅ Registered {} agents", 2);
    
    // Discover agents by capability
    let compute_agents = client.discover_by_capability("compute");
    println!("\n🔍 Agents with 'compute' capability:");
    for agent in compute_agents {
        println!("  - {} (reputation: {})", agent.agent_id, agent.reputation_score);
    }
    
    // Get best agent for capability
    if let Some(best) = client.get_best_agent("compute") {
        println!("\n⭐ Best compute agent: {} (rep: {})", 
            best.agent_id, best.reputation_score);
    }
    
    // Send message
    println!("\n📤 Sending message...");
    let message = A2AMessage::new(
        MessageType::Request,
        agent1.agent_id,
        Some(agent2.agent_id),
        MessagePayload::Request {
            action: "analyze_data".to_string(),
            params: serde_json::json!({
                "dataset": "sales_q1_2024",
                "metrics": ["revenue", "growth"]
            }).as_object().unwrap().clone(),
        },
    ).with_priority(MessagePriority::High);
    
    // Note: In real usage, this would actually send over network
    println!("Message created:");
    println!("  Type: {:?}", message.msg_type);
    println!("  Priority: {:?}", message.priority);
    println!("  From: {}", message.from);
    println!("  To: {:?}", message.to);
    
    // Create A2A server
    println!("\n🖥️  Setting up A2A server...");
    let mut server = A2AServer::new(agent1.agent_id);
    
    // Register handlers
    server.on(MessageType::Request, |msg| {
        println!("Received request: {:?}", msg.payload);
        
        Ok(A2AMessage::new(
            MessageType::Response,
            msg.to.unwrap(),
            Some(msg.from),
            MessagePayload::Response {
                success: true,
                data: Some(serde_json::json!({"result": "analysis_complete"})),
                error: None,
            },
        ))
    });
    
    println!("✅ A2A server configured with handlers");
    
    // Simulate handling a message
    let test_request = A2AMessage::new(
        MessageType::Request,
        agent2.agent_id,
        Some(agent1.agent_id),
        MessagePayload::Request {
            action: "test".to_string(),
            params: Default::default(),
        },
    );
    
    if let Some(Ok(response)) = server.handle(test_request).await {
        println!("\n📥 Handler response:");
        println!("  Type: {:?}", response.msg_type);
    }
    
    Ok(())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
