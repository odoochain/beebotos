# Tutorial 2: Creating Skills

## Introduction

Skills are WebAssembly modules that extend agent capabilities. This tutorial shows you how to create and use skills.

## What is a Skill?

A skill is a compiled WASM module that:
- Implements specific functionality
- Runs in a sandboxed environment
- Can be dynamically loaded
- Has a defined interface

## Step 1: Create a New Skill

Create a new Rust project:

```bash
cargo new --lib my_skill
cd my_skill
```

## Step 2: Configure for WASM

Edit `Cargo.toml`:

```toml
[package]
name = "my_skill"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
beebot-skills = { path = "path/to/beebotos/crates/skills" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[profile.release]
opt-level = 'z'
lto = true
```

## Step 3: Implement the Skill

Edit `src/lib.rs`:

```rust
use beebot_skills::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Input {
    pub numbers: Vec<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
    pub sum: f64,
    pub average: f64,
}

#[skill]
pub fn calculate_stats(input: Input) -> Result<Output, String> {
    if input.numbers.is_empty() {
        return Err("Empty input".to_string());
    }
    
    let sum: f64 = input.numbers.iter().sum();
    let average = sum / input.numbers.len() as f64;
    
    Ok(Output { sum, average })
}
```

## Step 4: Compile to WASM

```bash
cargo build --target wasm32-unknown-unknown --release
```

The compiled skill will be at:
`target/wasm32-unknown-unknown/release/my_skill.wasm`

## Step 5: Register the Skill

Create a skill manifest (`skill.json`):

```json
{
  "name": "my_skill",
  "version": "0.1.0",
  "description": "Calculates statistics",
  "author": "Your Name",
  "capabilities": ["compute"],
  "wasm_path": "my_skill.wasm",
  "schema": {
    "inputs": [
      {
        "name": "numbers",
        "param_type": "array",
        "required": true,
        "description": "List of numbers"
      }
    ],
    "outputs": [
      {
        "name": "sum",
        "param_type": "number",
        "required": true
      },
      {
        "name": "average",
        "param_type": "number",
        "required": true
      }
    ]
  }
}
```

## Step 6: Use the Skill

```rust
use beebot_agents::skills::SkillRegistry;

fn main() {
    let mut registry = SkillRegistry::new();
    
    // Load skill
    let manifest = SkillLoader::load("skill.json").unwrap();
    registry.register(manifest);
    
    // Execute skill
    let params = serde_json::json!({
        "numbers": [1.0, 2.0, 3.0, 4.0, 5.0]
    });
    
    let result = registry.execute("my_skill", params).unwrap();
    println!("Result: {:?}", result);
}
```

## Best Practices

1. **Keep skills small and focused**
2. **Handle errors gracefully**
3. **Document the interface**
4. **Version your skills**
5. **Test thoroughly**

## Advanced Topics

### Stateful Skills

```rust
#[skill]
pub fn counter(input: CounterInput) -> Result<CounterOutput, String> {
    // Access persistent storage
    let current = storage::get("count").unwrap_or(0);
    let new_count = current + input.increment;
    storage::set("count", new_count)?;
    
    Ok(CounterOutput { count: new_count })
}
```

### Skills with External Calls

```rust
#[skill]
pub async fn fetch_data(input: FetchInput) -> Result<FetchOutput, String> {
    // Make HTTP request
    let response = http::get(&input.url).await?;
    let data = response.json().await?;
    
    Ok(FetchOutput { data })
}
```

## Next Steps

- Tutorial 3: Agent Communication
- Tutorial 4: DAO Participation
- Check out example skills in `examples/skills/`
