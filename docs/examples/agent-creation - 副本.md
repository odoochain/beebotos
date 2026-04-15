# Agent Creation Examples

This guide provides practical examples for creating agents in BeeBotOS.

## Basic Agent

### Creating a Simple Agent

```rust
use beebotos::agents::{Agent, AgentConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent configuration
    let config = AgentConfig::new()
        .with_name("Simple Agent")
        .with_description("A basic autonomous agent")
        .with_capability("text_processing")
        .with_capability("data_analysis");
    
    // Initialize agent
    let mut agent = Agent::create(config).await?;
    
    // Start agent
    agent.start().await?;
    
    println!("Agent {} is running!", agent.id());
    
    // Run for a while
    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    
    // Stop agent
    agent.stop().await?;
    
    Ok(())
}
```

### Using CLI

```bash
# Create agent from template
beebotos agent create \
  --name "My Agent" \
  --template basic \
  --capabilities text_processing,data_analysis

# Start agent
beebotos agent start <agent-id>

# Check status
beebotos agent status <agent-id>
```

## Specialized Agents

### Trading Agent

```python
from beebotos import Agent, AgentConfig
from beebotos.skills import TradingSkill

config = AgentConfig(
    name="Alpha Trader",
    description="Autonomous crypto trading agent",
    skills=[
        {
            "name": "trading",
            "config": {
                "exchange": "binance",
                "paper_trading": True,
                "max_position": 0.1
            }
        }
    ],
    personality={
        "risk_tolerance": "medium",
        "trading_style": "swing"
    }
)

agent = Agent.create(config)
agent.start()

# Execute trading strategy
result = agent.execute({
    "action": "analyze_market",
    "symbol": "BTC/USDT"
})

print(result)
```

### Data Analysis Agent

```typescript
import { Agent, AgentConfig } from '@beebotos/sdk';

const config: AgentConfig = {
  name: 'Data Analyst',
  description: 'Analyzes datasets and generates insights',
  skills: [
    {
      name: 'data_analysis',
      config: {
        supported_formats: ['csv', 'json', 'parquet'],
        max_file_size: '100MB'
      }
    },
    {
      name: 'visualization',
      config: {
        chart_types: ['line', 'bar', 'scatter', 'heatmap']
      }
    }
  ]
};

const agent = await Agent.create(config);
await agent.start();

// Analyze dataset
const analysis = await agent.execute({
  action: 'analyze_dataset',
  file: 'sales_data.csv',
  analysis_type: 'trend'
});

console.log(analysis.insights);
```

## Multi-Agent Systems

### Creating a Swarm

```rust
use beebotos::agents::{Swarm, SwarmConfig, AgentRole};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let swarm_config = SwarmConfig::new()
        .with_name("Research Swarm")
        .with_coordination("hierarchical")
        .add_agent(AgentRole::Leader, 1)
        .add_agent(AgentRole::Worker, 5)
        .add_agent(AgentRole::Specialist, 2);
    
    let mut swarm = Swarm::create(swarm_config).await?;
    
    // Deploy swarm with task
    let task = Task::new("Research blockchain scalability solutions");
    let result = swarm.execute(task).await?;
    
    println!("Swarm completed task: {:?}", result);
    
    Ok(())
}
```

### Agent Collaboration

```python
from beebotos import AgentNetwork

# Create network
network = AgentNetwork()

# Add agents
researcher = network.add_agent(
    name="Researcher",
    capabilities=["web_search", "summarization"]
)

writer = network.add_agent(
    name="Writer", 
    capabilities=["writing", "editing"]
)

# Define workflow
workflow = network.create_workflow()
workflow.add_step(researcher, "research_topic")
workflow.add_step(writer, "write_article", depends_on=["research_topic"])

# Execute
result = workflow.execute({
    "topic": "Artificial General Intelligence"
})
```

## Advanced Configuration

### Custom Behavior

```rust
use beebotos::agents::{Agent, Behavior, Action};

struct CustomBehavior {
    learning_rate: f64,
}

#[async_trait]
impl Behavior for CustomBehavior {
    async fn decide(&self, context: &Context) -> Action {
        // Custom decision logic
        if context.confidence < 0.5 {
            Action::RequestHelp
        } else {
            Action::Execute
        }
    }
    
    async fn learn(&mut self, feedback: &Feedback) {
        // Update based on feedback
        self.learning_rate *= 0.99;
    }
}

let agent = Agent::builder()
    .with_behavior(Box::new(CustomBehavior {
        learning_rate: 0.1
    }))
    .build()
    .await?;
```

### Memory Configuration

```yaml
# agent-config.yaml
name: Memory Agent
memory:
  short_term:
    capacity: 100
    decay_rate: 0.1
  long_term:
    storage: vector_db
    embedding_model: text-embedding-3
    consolidation_interval: 3600
  episodic:
    enabled: true
    retention_days: 30
```

```rust
use beebotos::agents::Agent;

let agent = Agent::from_config_file("agent-config.yaml").await?;
agent.start().await?;
```

### Personality Setup

```python
from beebotos.personality import OCEAN

personality = OCEAN(
    openness=0.8,          # Curious, creative
    conscientiousness=0.7,  # Organized, careful
    extraversion=0.6,       # Outgoing, energetic
    agreeableness=0.75,     # Cooperative, trusting
    neuroticism=0.3         # Calm, stable
)

agent = Agent.create(
    name="Creative Assistant",
    personality=personality,
    communication_style="friendly_professional"
)
```

## Agent with External APIs

### Integrating Third-Party Services

```typescript
import { Agent, APISkill } from '@beebotos/sdk';

const weatherAPI = new APISkill({
  name: 'weather',
  baseUrl: 'https://api.weather.com',
  auth: {
    type: 'api_key',
    key: process.env.WEATHER_API_KEY
  },
  endpoints: [
    {
      name: 'get_forecast',
      path: '/forecast',
      method: 'GET',
      parameters: ['location', 'days']
    }
  ]
});

const agent = await Agent.create({
  name: 'Weather Assistant',
  skills: [weatherAPI]
});

const forecast = await agent.execute({
  action: 'weather.get_forecast',
  location: 'New York',
  days: 5
});
```

## Agent Monitoring

### Adding Observability

```rust
use beebotos::agents::{Agent, Telemetry};

let telemetry = Telemetry::new()
    .with_metrics("prometheus")
    .with_tracing("jaeger")
    .with_logging("structured");

let agent = Agent::create(config)
    .with_telemetry(telemetry)
    .await?;

// Metrics are automatically collected
// - Task execution time
// - Decision confidence
// - Memory usage
// - Skill utilization
```

### Event Handling

```python
from beebotos import Agent, Event

agent = Agent.create(name="Event Driven Agent")

@agent.on_event("task.completed")
def handle_task_complete(event: Event):
    print(f"Task {event.task_id} completed!")
    # Send notification, update metrics, etc.

@agent.on_event("error")
def handle_error(event: Event):
    print(f"Error occurred: {event.error}")
    # Log error, alert operators, etc.

agent.start()
```

## Testing Agents

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use beebotos::agents::testing::*;

    #[tokio::test]
    async fn test_agent_decision() {
        let agent = create_test_agent().await;
        
        let context = TestContext::new()
            .with_input("What is 2+2?")
            .with_expected_capability("calculation");
        
        let action = agent.decide(&context).await;
        
        assert!(matches!(action, Action::Execute));
    }
}
```

### Simulation Testing

```python
from beebotos.testing import Simulator

simulator = Simulator()

# Create test scenario
scenario = simulator.create_scenario()
scenario.add_user_input("Hello, can you help me?")
scenario.expect_response(containing="Hello")

# Run simulation
results = simulator.run(agent, scenario)
assert results.success_rate > 0.95
```

## Deployment

### Docker Deployment

```dockerfile
FROM beebotos/agent-runtime:latest

COPY ./my_agent ./agent
COPY config.yaml ./config.yaml

EXPOSE 8080

CMD ["beebotos", "agent", "start", "--config", "./config.yaml"]
```

```bash
docker build -t my-agent .
docker run -p 8080:8080 my-agent
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-agent
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-agent
  template:
    metadata:
      labels:
        app: my-agent
    spec:
      containers:
      - name: agent
        image: my-agent:latest
        env:
        - name: BEE_API_KEY
          valueFrom:
            secretKeyRef:
              name: agent-secrets
              key: api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
```
