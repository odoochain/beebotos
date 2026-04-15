use beebotos_agents::{Agent, AgentBuilder, AgentConfig};

#[tokio::main]
async fn main() {
    println!("Creating BeeBotOS Agent...");

    let agent = AgentBuilder::new("ExampleAgent")
        .description("A simple example agent")
        .with_capability("chat")
        .with_model("openai", "gpt-4")
        .build();

    println!("Agent created: {}", agent.get_config().name);
    println!("Capabilities: {:?}", agent.get_config().capabilities);

    let task = beebotos_agents::Task {
        id: "task_001".to_string(),
        task_type: "conversation".to_string(),
        input: "Hello, BeeBotOS!".to_string(),
        parameters: std::collections::HashMap::new(),
    };

    match agent.execute_task(task).await {
        Ok(result) => println!("Task completed: {}", result.output),
        Err(e) => println!("Task failed: {}", e),
    }
}
