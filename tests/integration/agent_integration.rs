//! Agent integration tests

use beebot_agents::{AgentRuntime, AgentConfig, AgentCapabilities};
use beebot_agents::runtime::executor::{Task, TaskPayload};

#[tokio::test]
async fn test_agent_spawning() {
    let config = beebot_agents::runtime::RuntimeConfig::default();
    let mut runtime = AgentRuntime::new(config);
    
    let agent_config = AgentConfig {
        name: "TestAgent".to_string(),
        description: "Integration test agent".to_string(),
        capabilities: vec!["compute".to_string()],
        skills: vec![],
        max_memory: 64 * 1024 * 1024,
        max_cpu_time: 60,
        auto_restart: false,
        log_level: "info".to_string(),
    };
    
    let agent_id = runtime.spawn(agent_config).await
        .expect("Failed to spawn agent");
    
    let agent = runtime.get(&agent_id).expect("Agent not found");
    assert_eq!(agent.metadata.name, "TestAgent");
    
    runtime.terminate(&agent_id).await.unwrap();
}

#[tokio::test]
async fn test_agent_task_execution() {
    let config = beebot_agents::runtime::RuntimeConfig::default();
    let mut runtime = AgentRuntime::new(config);
    
    let agent_config = AgentConfig::default();
    let agent_id = runtime.spawn(agent_config).await.unwrap();
    
    let task = Task {
        id: beebot_agents::types::TaskId::new(),
        name: "test_task".to_string(),
        payload: TaskPayload::Code("print('hello')".to_string()),
        timeout: std::time::Duration::from_secs(10),
        max_memory: 1024 * 1024,
    };
    
    let result = runtime.execute(agent_id, task).await
        .expect("Task execution failed");
    
    assert!(result.success);
    
    runtime.terminate(&agent_id).await.unwrap();
}

#[tokio::test]
async fn test_multiple_agents() {
    let config = beebot_agents::runtime::RuntimeConfig::default();
    let mut runtime = AgentRuntime::new(config);
    
    let mut agents = vec![];
    
    for i in 0..5 {
        let agent_config = AgentConfig {
            name: format!("Agent{}", i),
            ..Default::default()
        };
        
        let id = runtime.spawn(agent_config).await.unwrap();
        agents.push(id);
    }
    
    assert_eq!(agents.len(), 5);
    
    let all_agents = runtime.list();
    assert_eq!(all_agents.len(), 5);
    
    for id in agents {
        runtime.terminate(&id).await.unwrap();
    }
}

#[tokio::test]
async fn test_agent_reputation() {
    let config = beebot_agents::runtime::RuntimeConfig::default();
    let mut runtime = AgentRuntime::new(config);
    
    let agent_config = AgentConfig::default();
    let agent_id = runtime.spawn(agent_config).await.unwrap();
    
    {
        let agent = runtime.get_mut(&agent_id).unwrap();
        
        // Record successes
        for _ in 0..5 {
            agent.record_success();
        }
        
        // Record failures
        for _ in 0..2 {
            agent.record_failure();
        }
    }
    
    let agent = runtime.get(&agent_id).unwrap();
    assert_eq!(agent.tasks_completed, 5);
    assert_eq!(agent.tasks_failed, 2);
    assert_eq!(agent.success_rate(), 5.0 / 7.0);
    
    runtime.terminate(&agent_id).await.unwrap();
}
