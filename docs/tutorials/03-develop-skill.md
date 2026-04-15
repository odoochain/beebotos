# Tutorial: Developing Your First Skill

## Overview

This tutorial guides you through creating a custom skill for BeeBotOS agents.

## Prerequisites

- BeeBotOS CLI installed
- Rust toolchain (for WASM skills) or Python 3.10+
- Basic understanding of JSON schemas

## Step 1: Initialize the Skill

```bash
beebotos skill create my-analyzer --template rust
```

This creates a new skill directory:

```
my-analyzer/
├── Cargo.toml
├── skill.yaml
├── src/
│   └── lib.rs
└── README.md
```

## Step 2: Define the Schema

Edit `skill.yaml` to define inputs and outputs:

```yaml
name: my-analyzer
version: 0.1.0
description: Analyzes text sentiment

schema:
  input:
    type: object
    required: [text]
    properties:
      text:
        type: string
        maxLength: 1000
      language:
        type: string
        default: "en"
        
  output:
    type: object
    properties:
      sentiment:
        type: string
        enum: [positive, negative, neutral]
      confidence:
        type: number
        minimum: 0
        maximum: 1
      keywords:
        type: array
        items:
          type: string
```

## Step 3: Implement the Logic

Edit `src/lib.rs`:

```rust
use beebotos_sdk::{skill, Context, Result};
use serde_json::Value;

#[skill(name = "my-analyzer")]
pub async fn handle(ctx: Context, input: Value) -> Result<Value> {
    let text = input["text"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing text field"))?;
    
    // Simple sentiment analysis
    let positive_words = ["good", "great", "excellent", "happy"];
    let negative_words = ["bad", "terrible", "sad", "angry"];
    
    let lower = text.to_lowercase();
    let pos_count = positive_words.iter()
        .filter(|&&w| lower.contains(w))
        .count();
    let neg_count = negative_words.iter()
        .filter(|&&w| lower.contains(w))
        .count();
    
    let sentiment = if pos_count > neg_count {
        "positive"
    } else if neg_count > pos_count {
        "negative"
    } else {
        "neutral"
    };
    
    let confidence = calculate_confidence(pos_count, neg_count, text.len());
    
    Ok(serde_json::json!({
        "sentiment": sentiment,
        "confidence": confidence,
        "keywords": extract_keywords(&lower)
    }))
}

fn calculate_confidence(pos: usize, neg: usize, len: usize) -> f64 {
    let score = (pos + neg) as f64 / (len as f64 / 10.0).max(1.0);
    score.min(1.0)
}

fn extract_keywords(text: &str) -> Vec<String> {
    // Simple keyword extraction
    text.split_whitespace()
        .filter(|w| w.len() > 4)
        .map(|w| w.to_string())
        .collect()
}
```

## Step 4: Build the Skill

```bash
cd my-analyzer
cargo build --target wasm32-wasi --release
```

## Step 5: Test Locally

```bash
beebotos skill test ./my-analyzer \
  --input '{"text": "This is a great product!"}'
```

Expected output:
```json
{
  "sentiment": "positive",
  "confidence": 0.8,
  "keywords": ["great", "product"]
}
```

## Step 6: Package and Publish

```bash
# Package the skill
beebotos skill package ./my-analyzer --output my-analyzer-0.1.0.tar.gz

# Publish to marketplace (requires API key)
beebotos skill publish ./my-analyzer
```

## Advanced Features

### Accessing Configuration

```rust
#[skill(name = "my-analyzer")]
pub async fn handle(ctx: Context, input: Value) -> Result<Value> {
    // Access skill configuration
    let strict_mode = ctx.config()
        .get("strict_mode")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    
    // Access agent memory
    let history = ctx.memory()
        .search("previous_analysis", 5)
        .await?;
    
    // ... logic
}
```

### Making External Calls

```rust
use beebotos_sdk::http::Client;

pub async fn handle(ctx: Context, input: Value) -> Result<Value> {
    let client = Client::new();
    
    let response = client
        .post("https://api.example.com/analyze")
        .json(&input)
        .send()
        .await?;
    
    let result = response.json::<Value>().await?;
    Ok(result)
}
```

### Using LLM

```rust
pub async fn handle(ctx: Context, input: Value) -> Result<Value> {
    let llm = ctx.llm();
    
    let response = llm.complete()
        .with_prompt(format!("Analyze sentiment: {}", input["text"]))
        .with_temperature(0.3)
        .send()
        .await?;
    
    Ok(serde_json::json!({
        "analysis": response.text()
    }))
}
```

## Testing

Create `tests/integration_test.rs`:

```rust
#[tokio::test]
async fn test_sentiment_analysis() {
    let ctx = Context::new();
    let input = serde_json::json!({
        "text": "Amazing experience!"
    });
    
    let result = handle(ctx, input).await.unwrap();
    
    assert_eq!(result["sentiment"], "positive");
    assert!(result["confidence"].as_f64().unwrap() > 0.5);
}
```

Run tests:
```bash
cargo test
```

## Best Practices

1. **Validate inputs** - Always check required fields
2. **Handle errors** - Return meaningful error messages
3. **Limit resources** - Respect memory and time limits
4. **Document behavior** - Update README with examples
5. **Version properly** - Follow semantic versioning

## Next Steps

- [Skill Marketplace](./07-skill-marketplace.md)
- [Advanced WASM Features](./08-advanced-wasm.md)
- [Testing Strategies](./09-testing-skills.md)
