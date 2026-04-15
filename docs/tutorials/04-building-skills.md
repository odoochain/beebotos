# Tutorial 4: Building Custom Skills

Learn how to create reusable, composable skills that extend agent capabilities.

## Overview

By the end of this tutorial, you will:
- Understand the skill architecture
- Create skills in Rust and WebAssembly
- Register and version skills
- Compose skills into workflows
- Share skills with the community

## Prerequisites

- Completed [Tutorial 2: Create Your First Agent](02-create-first-agent.md)
- Basic understanding of WebAssembly (WASM)
- Rust programming knowledge

## What is a Skill?

A skill is a modular, reusable capability that agents can acquire:

```
┌─────────────────────────────────────────────────────────────┐
│                         Skill                                │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Manifest   │  │   Handler    │  │   Schema     │      │
│  │  skill.yaml  │  │   (WASM)     │  │  schema.json │      │
│  │              │  │              │  │              │      │
│  │ - Metadata   │  │ - Logic      │  │ - Inputs     │      │
│  │ - Triggers   │  │ - State      │  │ - Outputs    │      │
│  │ - Parameters │  │ - External   │  │ - Examples   │      │
│  │ - Depends_on │  │   calls      │  │              │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

## Skill Architecture

### Skill Manifest

```yaml
# skill.yaml
name: web-search
version: 1.2.0
description: Search the web using various search engines
author: beebotos-team
license: MIT
tags: [search, web, data]

# Skill triggers - when is this skill invoked?
triggers:
  - type: intent
    value: search
  - type: intent
    value: find
  - type: keyword
    value: "search for"
  - type: pattern
    value: "(find|look up|search) (.+)"

# Input parameters
parameters:
  - name: query
    type: string
    required: true
    description: The search query
    
  - name: engine
    type: string
    required: false
    default: google
    enum: [google, bing, duckduckgo]
    description: Search engine to use
    
  - name: limit
    type: integer
    required: false
    default: 10
    minimum: 1
    maximum: 100
    description: Maximum number of results

# Output schema
returns:
  type: object
  properties:
    results:
      type: array
      items:
        type: object
        properties:
          title:
            type: string
          url:
            type: string
            format: uri
          snippet:
            type: string
          score:
            type: number
    total:
      type: integer
    engine:
      type: string

# Resource requirements
resources:
  memory_mb: 64
  timeout_ms: 10000
  
# Dependencies on other skills or external services
depends_on:
  skills: []
  apis:
    - name: google_custom_search
      required: false
    - name: bing_search
      required: false

# Examples for documentation and testing
examples:
  - name: basic search
    input:
      query: "Rust programming language"
    output:
      results:
        - title: "Rust Programming Language"
          url: "https://www.rust-lang.org"
          snippet: "A language empowering everyone..."
          score: 0.95
      total: 1
      engine: google
      
  - name: limited search
    input:
      query: "machine learning"
      limit: 5
    output:
      results: []
      total: 5
```

### Skill Interface

```rust
// Trait that all skills must implement
#[async_trait]
pub trait Skill: Send + Sync {
    /// Skill metadata
    fn metadata(&self) -> &SkillMetadata;
    
    /// Initialize the skill with configuration
    async fn initialize(&mut self, config: SkillConfig) -> Result<()>;
    
    /// Execute the skill
    async fn execute(&self, input: SkillInput, context: SkillContext) -> Result<SkillOutput>;
    
    /// Check if skill can handle the given input
    fn can_handle(&self, input: &SkillInput) -> bool;
    
    /// Get skill health status
    async fn health(&self) -> HealthStatus;
}

/// Input to skill execution
pub struct SkillInput {
    pub parameters: HashMap<String, Value>,
    pub context: ExecutionContext,
}

/// Output from skill execution
pub struct SkillOutput {
    pub result: Value,
    pub metadata: ExecutionMetadata,
}

/// Context available during skill execution
pub struct SkillContext {
    pub agent: AgentContext,
    pub memory: MemoryAccess,
    pub tools: ToolRegistry,
    pub llm: LlmClient,
}
```

## Step 1: Create a Skill Project

### Initialize

```bash
beebotos skill init weather-skill
cd weather-skill
```

This creates:

```
weather-skill/
├── skill.yaml          # Skill manifest
├── Cargo.toml          # Rust package
├── src/
│   └── lib.rs          # Skill implementation
├── schema.json         # JSON schema for validation
├── examples/           # Example inputs/outputs
└── tests/
    └── integration.rs  # Skill tests
```

### Define Schema

```json
// schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Weather Skill Schema",
  "type": "object",
  "required": ["location"],
  "properties": {
    "location": {
      "type": "string",
      "description": "City name or coordinates"
    },
    "units": {
      "type": "string",
      "enum": ["metric", "imperial"],
      "default": "metric"
    },
    "forecast_days": {
      "type": "integer",
      "minimum": 1,
      "maximum": 14,
      "default": 1
    }
  },
  "returns": {
    "type": "object",
    "required": ["temperature", "conditions"],
    "properties": {
      "temperature": {
        "type": "number",
        "description": "Current temperature"
      },
      "feels_like": {
        "type": "number"
      },
      "humidity": {
        "type": "integer",
        "minimum": 0,
        "maximum": 100
      },
      "conditions": {
        "type": "string",
        "enum": ["sunny", "cloudy", "rainy", "snowy", "stormy"]
      },
      "forecast": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "date": {
              "type": "string",
              "format": "date"
            },
            "high": {
              "type": "number"
            },
            "low": {
              "type": "number"
            },
            "conditions": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}
```

## Step 2: Implement the Skill

### Rust Implementation

```rust
// src/lib.rs
use beebotos_skill::prelude::*;
use serde::{Deserialize, Serialize};

// Skill metadata
const METADATA: SkillMetadata = SkillMetadata {
    name: "weather",
    version: "1.0.0",
    description: "Get current weather and forecasts",
};

// Input structure
#[derive(Debug, Deserialize)]
pub struct WeatherInput {
    pub location: String,
    #[serde(default = "default_units")]
    pub units: String,
    #[serde(default = "default_forecast_days")]
    pub forecast_days: u8,
}

fn default_units() -> String { "metric".to_string() }
fn default_forecast_days() -> u8 { 1 }

// Output structure
#[derive(Debug, Serialize)]
pub struct WeatherOutput {
    pub location: String,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: u8,
    pub conditions: String,
    pub forecast: Vec<ForecastDay>,
}

#[derive(Debug, Serialize)]
pub struct ForecastDay {
    pub date: String,
    pub high: f64,
    pub low: f64,
    pub conditions: String,
}

// Skill implementation
pub struct WeatherSkill {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

#[async_trait]
impl Skill for WeatherSkill {
    fn metadata(&self) -> &SkillMetadata {
        &METADATA
    }

    async fn initialize(&mut self, config: SkillConfig) -> Result<()> {
        self.api_key = config
            .get_string("api_key")?
            .ok_or_else(|| Error::missing_config("api_key"))?;
        
        self.base_url = config
            .get_string("base_url")?
            .unwrap_or_else(|| "https://api.weather.com/v1".to_string());
        
        Ok(())
    }

    async fn execute(&self, input: SkillInput, _ctx: SkillContext) -> Result<SkillOutput> {
        // Parse and validate input
        let input: WeatherInput = input.parse()?;
        
        // Fetch current weather
        let current = self.fetch_current(&input.location, &input.units).await?;
        
        // Fetch forecast if requested
        let forecast = if input.forecast_days > 1 {
            self.fetch_forecast(&input.location, &input.units, input.forecast_days).await?
        } else {
            vec![]
        };
        
        // Build output
        let output = WeatherOutput {
            location: input.location,
            temperature: current.temp,
            feels_like: current.feels_like,
            humidity: current.humidity,
            conditions: current.conditions,
            forecast,
        };
        
        Ok(SkillOutput::new(output)?)
    }

    fn can_handle(&self, input: &SkillInput) -> bool {
        input.has_param("location")
    }

    async fn health(&self) -> HealthStatus {
        match self.check_api_health().await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) => HealthStatus::Unhealthy(e.to_string()),
        }
    }
}

impl WeatherSkill {
    async fn fetch_current(&self, location: &str, units: &str) -> Result<CurrentWeather> {
        let url = format!(
            "{}/current?location={}&units={}&apikey={}",
            self.base_url,
            urlencoding::encode(location),
            units,
            self.api_key
        );
        
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(10))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(Error::api_error(
                format!("Weather API error: {}", response.status())
            ));
        }
        
        response.json::<CurrentWeather>().await.map_err(Into::into)
    }

    async fn check_api_health(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}

// Register the skill
skill_export!(WeatherSkill);
```

### WebAssembly Compilation

```toml
# Cargo.toml
[package]
name = "weather-skill"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
beebotos-skill = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
urlencoding = "2.1"

[profile.release]
opt-level = 3
lto = true
```

## Step 3: Test the Skill

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weather_skill() {
        let mut skill = WeatherSkill::new();
        
        let config = SkillConfig::new()
            .with("api_key", "test_key")
            .with("base_url", "https://mock.api.com");
        
        skill.initialize(config).await.unwrap();
        
        let input = SkillInput::new()
            .with("location", "New York")
            .with("units", "metric");
        
        // Mock the HTTP client for testing
        let output = skill.execute(input, SkillContext::mock()).await.unwrap();
        
        assert!(output.get::<f64>("temperature").is_ok());
    }

    #[test]
    fn test_input_validation() {
        // Valid input
        let valid = serde_json::json!({
            "location": "London",
            "units": "metric",
            "forecast_days": 5
        });
        
        assert!(validate_input(&valid).is_ok());
        
        // Invalid - missing location
        let invalid = serde_json::json!({
            "units": "metric"
        });
        
        assert!(validate_input(&invalid).is_err());
    }
}
```

### Integration Tests

```rust
// tests/integration.rs
use beebotos_skill::testing::*;

#[tokio::test]
async fn test_skill_registration() {
    let skill = load_skill("../skill.yaml").await.unwrap();
    
    assert_eq!(skill.metadata().name, "weather");
    assert_eq!(skill.metadata().version, "1.0.0");
}

#[tokio::test]
async fn test_skill_execution() {
    let runner = SkillRunner::new("weather-skill.wasm")
        .with_config("api_key", env!("WEATHER_API_KEY"));
    
    let result = runner.execute(json!({
        "location": "Tokyo",
        "units": "metric"
    })).await.unwrap();
    
    assert!(result.get("temperature").is_some());
    assert!(result.get("conditions").is_some());
}
```

Run tests:

```bash
# Build WASM target
rustup target add wasm32-wasi
cargo build --target wasm32-wasi --release

# Run tests
cargo test
beebotos skill test --skill weather-skill.wasm
```

## Step 4: Register and Publish

### Local Registration

```bash
# Register skill locally
beebotos skill register ./skill.yaml

# Verify registration
beebotos skill list
```

### Publish to Registry

```bash
# Build release package
beebotos skill package --output weather-skill-v1.0.0.tar.gz

# Publish to public registry
beebotos skill publish weather-skill-v1.0.0.tar.gz \
    --registry https://skills.beebotos.dev

# Or publish privately
beebotos skill publish weather-skill-v1.0.0.tar.gz \
    --private \
    --org my-organization
```

### Version Management

```bash
# Create new version
beebotos skill version bump --type minor
# Creates v1.1.0

# Deprecate old version
beebotos skill deprecate weather@1.0.0 \
    --reason "Security vulnerability in dependencies"

# Yank version
beebotos skill yank weather@1.0.0
```

## Step 5: Compose Skills

### Skill Workflows

```yaml
# workflow.yaml
name: trip-planner
version: 1.0.0
description: Plan a trip using multiple skills

workflow:
  steps:
    - id: get_weather
      skill: weather
      input:
        location: $.destination
        forecast_days: 7
      
    - id: find_hotels
      skill: hotel-search
      input:
        location: $.destination
        check_in: $.check_in
        check_out: $.check_out
      depends_on: [get_weather]
      
    - id: find_flights
      skill: flight-search
      input:
        from: $.origin
        to: $.destination
        date: $.check_in
      depends_on: [get_weather]
      
    - id: create_itinerary
      skill: itinerary-generator
      input:
        weather: $.get_weather
        hotels: $.find_hotels
        flights: $.find_flights
      depends_on: [find_hotels, find_flights]

  output:
    weather: $.get_weather
    hotels: $.find_hotels
    flights: $.find_flights
    itinerary: $.create_itinerary
```

### Conditional Logic

```yaml
workflow:
  steps:
    - id: check_weather
      skill: weather
      
    - id: indoor_activities
      skill: activity-search
      condition: $.check_weather.conditions == "rainy"
      input:
        type: indoor
        location: $.destination
        
    - id: outdoor_activities
      skill: activity-search
      condition: $.check_weather.conditions != "rainy"
      input:
        type: outdoor
        location: $.destination
```

## Step 6: Use Skills in Agents

### Runtime Skill Loading

```rust
use beebotos_agent::skills::SkillManager;

impl MyAgent {
    async fn load_skills(&mut self) -> Result<()> {
        let mut skills = SkillManager::new();
        
        // Load from registry
        skills.load_from_registry("weather", "^1.0").await?;
        skills.load_from_registry("hotel-search", "^2.0").await?;
        
        // Load local skill
        skills.load_local("./custom-skill.wasm").await?;
        
        self.skills = skills;
        Ok(())
    }

    async fn handle_weather_query(&self, query: &str) -> Result<Response> {
        // Extract location from query
        let location = self.extract_location(query).await?;
        
        // Execute skill
        let result = self.skills
            .get("weather")?
            .execute(json!({
                "location": location,
                "forecast_days": 3
            }))
            .await?;
        
        Ok(Response::success(result))
    }
}
```

## Advanced Topics

### Skill Sandboxing

```rust
use beebotos_skill::sandbox::{Sandbox, ResourceLimits};

let sandbox = Sandbox::new()
    .with_limits(ResourceLimits {
        memory_mb: 64,
        cpu_ms: 1000,
        network: NetworkPolicy::Restricted(vec![
            "api.weather.com".to_string(),
        ]),
        filesystem: FilesystemPolicy::ReadOnly,
    });

let result = sandbox.run(skill, input).await?;
```

### Skill Chaining

```rust
// Chain multiple skills together
let pipeline = SkillPipeline::new()
    .add("translate", json!({"target_lang": "en"}))
    .add("summarize", json!({"max_length": 100}))
    .add("sentiment", json!({}));

let result = pipeline.execute(input_text).await?;
```

## Best Practices

1. **Keep skills focused**: One skill = one capability
2. **Validate inputs**: Use JSON Schema for strict validation
3. **Handle errors gracefully**: Provide meaningful error messages
4. **Document thoroughly**: Include examples in manifest
5. **Version carefully**: Follow semantic versioning
6. **Test extensively**: Unit, integration, and end-to-end tests
7. **Monitor performance**: Track execution time and resource usage
8. **Secure external calls**: Validate and sanitize all external data

## Skill Registry

Browse available skills:

```bash
# Search skills
beebotos skill search "weather"

# View skill details
beebotos skill info weather

# Install skill
beebotos skill install weather@^1.0
```

## Complete Example

View the complete skill example:

```bash
git clone https://github.com/beebotos/examples.git
cd examples/weather-skill
beebotos skill build
beebotos skill test
```

## Next Steps

- [Skill Development Guide](../guides/skill-development.md)
- [Skill Registry](https://skills.beebotos.dev)
- [Skill SDK Reference](../api/skill-sdk.md)

---

Congratulations! You can now build reusable skills that extend agent capabilities and share them with the community.
