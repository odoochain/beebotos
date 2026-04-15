//! Hello Agent Example
//! 
//! This example demonstrates creating and running a simple agent.

use beebot_agents::{AgentRuntime, AgentConfig, AgentCapabilities};
use beebot_kernel::capabilities::{CapabilitySet, CapabilityLevel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize runtime
    let config = beebot_agents::runtime::RuntimeConfig::default();
    let mut runtime = AgentRuntime::new(config);
    
    // Configure agent
    let agent_config = AgentConfig {
        name: "HelloAgent".to_string(),
        description: "A simple greeting agent".to_string(),
        capabilities: vec!["greet".to_string()],
        skills: vec![],
        max_memory: 64 * 1024 * 1024,
        max_cpu_time: 60,
        auto_restart: false,
        log_level: "info".to_string(),
    };
    
    // Spawn agent
    let agent_id = runtime.spawn(agent_config).await?;
    println!("✅ Agent spawned: {}", agent_id);
    
    // Get agent reference
    let agent = runtime.get(&agent_id).unwrap();
    println!("Agent name: {}", agent.metadata.name);
    println!("Agent status: {:?}", agent.state);
    
    // Execute a simple task
    let task = beebot_agents::runtime::executor::Task {
        id: beebot_agents::types::TaskId::new(),
        name: "greet_task".to_string(),
        payload: beebot_agents::runtime::executor::TaskPayload::Code(
            "print('Hello from BeeBotOS!')".to_string()
        ),
        timeout: std::time::Duration::from_secs(30),
        max_memory: 1024 * 1024,
    };
    
    let result = runtime.execute(agent_id, task).await?;
    
    if result.success {
        println!("✅ Task completed successfully");
        println!("Output: {:?}", String::from_utf8_lossy(&result.output));
    } else {
        println!("❌ Task failed: {:?}", result.error);
    }
    
    // Cleanup
    runtime.terminate(&agent_id).await?;
    println!("👋 Agent terminated");
    
    Ok(())
}
