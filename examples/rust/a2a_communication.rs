use beebotos_agents::a2a::{A2AClient, AgentCard, Endpoint, Protocol};

#[tokio::main]
async fn main() {
    println!("Initializing A2A Communication Example...");

    let client = A2AClient::new().expect("Failed to create A2A client");

    let agent_card = AgentCard {
        id: "agent_001".to_string(),
        name: "Example Agent".to_string(),
        description: "An example A2A agent".to_string(),
        version: "1.0.0".to_string(),
        capabilities: vec![],
        endpoints: vec![
            Endpoint {
                protocol: Protocol::Http,
                address: "localhost".to_string(),
                port: 8080,
            },
        ],
        authentication: beebotos_agents::a2a::AuthenticationMethod::None,
        metadata: std::collections::HashMap::new(),
    };

    client.discovery().register_agent(agent_card);
    println!("Agent registered successfully");
}
