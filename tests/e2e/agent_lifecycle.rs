//! End-to-end test for agent lifecycle

use beebot_kernel::Kernel;
use beebot_agents::{AgentConfig, AgentRuntime};
use std::time::Duration;

#[tokio::test]
async fn test_agent_full_lifecycle() {
    // Initialize kernel
    let kernel = Kernel::new(Default::default()).await.unwrap();
    
    // Create agent runtime
    let runtime = AgentRuntime::new(kernel.clone()).await.unwrap();
    
    // Spawn agent
    let config = AgentConfig {
        name: "test-agent".to_string(),
        memory_limit: 64 * 1024 * 1024, // 64MB
        capabilities: vec!["L1_FileRead".to_string()],
        ..Default::default()
    };
    
    let agent_id = runtime.spawn_agent(config).await.unwrap();
    println!("Agent spawned: {:?}", agent_id);
    
    // Verify agent is running
    let status = runtime.get_agent_status(&agent_id).await.unwrap();
    assert!(status.is_running());
    
    // Send message to agent
    let message = beebot_agents::Message::new("ping", vec![]);
    runtime.send_message(&agent_id, message).await.unwrap();
    
    // Wait for processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Pause agent
    runtime.pause_agent(&agent_id).await.unwrap();
    let status = runtime.get_agent_status(&agent_id).await.unwrap();
    assert!(status.is_paused());
    
    // Resume agent
    runtime.resume_agent(&agent_id).await.unwrap();
    let status = runtime.get_agent_status(&agent_id).await.unwrap();
    assert!(status.is_running());
    
    // Terminate agent
    runtime.terminate_agent(&agent_id).await.unwrap();
    
    // Verify termination
    let result = runtime.get_agent_status(&agent_id).await;
    assert!(result.is_err() || !result.unwrap().is_running());
    
    println!("Agent lifecycle test passed!");
}

#[tokio::test]
async fn test_multiple_agents() {
    let kernel = Kernel::new(Default::default()).await.unwrap();
    let runtime = AgentRuntime::new(kernel.clone()).await.unwrap();
    
    let mut agents = vec![];
    
    // Spawn multiple agents
    for i in 0..5 {
        let config = AgentConfig {
            name: format!("agent-{}", i),
            ..Default::default()
        };
        let id = runtime.spawn_agent(config).await.unwrap();
        agents.push(id);
    }
    
    // Verify all agents exist
    assert_eq!(runtime.list_agents().await.unwrap().len(), 5);
    
    // Terminate all
    for id in agents {
        runtime.terminate_agent(&id).await.unwrap();
    }
}

#[tokio::test]
async fn test_agent_communication() {
    let kernel = Kernel::new(Default::default()).await.unwrap();
    let runtime = AgentRuntime::new(kernel.clone()).await.unwrap();
    
    // Spawn two agents
    let agent1 = runtime.spawn_agent(AgentConfig {
        name: "sender".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    let agent2 = runtime.spawn_agent(AgentConfig {
        name: "receiver".to_string(),
        ..Default::default()
    }).await.unwrap();
    
    // Send message from agent1 to agent2
    let message = beebot_agents::Message::new("greeting", vec![
        ("content", "Hello from agent1!"),
    ]);
    
    runtime.send_message_to(&agent1, &agent2, message).await.unwrap();
    
    // Wait and verify
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let inbox = runtime.get_inbox(&agent2).await.unwrap();
    assert_eq!(inbox.len(), 1);
    
    // Cleanup
    runtime.terminate_agent(&agent1).await.unwrap();
    runtime.terminate_agent(&agent2).await.unwrap();
}
