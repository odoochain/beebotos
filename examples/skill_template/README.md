# BeeBotOS Skill Template

A template for creating custom skills for BeeBotOS agents.

## Quick Start

1. **Copy the template:**
   ```bash
   cp -r examples/skill_template my-skill
   cd my-skill
   ```

2. **Update metadata:**
   ```toml
   # Cargo.toml
   [package]
   name = "my-awesome-skill"
   version = "0.1.0"
   ```

3. **Implement your logic in `src/lib.rs`**

4. **Build:**
   ```bash
   cargo build --target wasm32-wasi --release
   ```

5. **Deploy:**
   ```bash
   beebot skill deploy target/wasm32-wasi/release/my_awesome_skill.wasm
   ```

## Skill Structure

```
my-skill/
├── Cargo.toml          # Dependencies and metadata
├── README.md           # Documentation
├── src/
│   └── lib.rs         # Skill implementation
└── tests/             # Tests (optional)
```

## API Reference

### Message Handling

```rust
pub async fn handle_message(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError>
```

### Available Context Methods

- `ctx.log(msg)` - Log a message
- `ctx.storage_get(key)` - Get from storage
- `ctx.storage_set(key, value)` - Set in storage
- `ctx.http_request(req)` - Make HTTP request
- `ctx.emit_event(event)` - Emit event

### Message Fields

- `action` - The action to perform
- `params` - JSON parameters
- `sender` - Sender agent ID
- `timestamp` - Unix timestamp

## Examples

### Simple Echo Skill

```rust
async fn handle_message(&self, _ctx: &Context, msg: Message) -> Result<Message, SkillError> {
    Ok(Message::response(&msg, msg.params))
}
```

### HTTP API Skill

```rust
async fn fetch_weather(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError> {
    let city = msg.params.get("city").unwrap_or("London");
    let url = format!("https://api.weather.com/v1/current?city={}", city);
    
    let response = ctx.http_get(&url).await?;
    let weather: WeatherResponse = response.json().await?;
    
    Ok(Message::response(&msg, serde_json::to_value(weather)?))
}
```

### State Management

```rust
async fn increment_counter(&self, ctx: &Context, msg: Message) -> Result<Message, SkillError> {
    let current: u64 = ctx.storage_get("counter").await?.unwrap_or(0);
    let next = current + 1;
    ctx.storage_set("counter", &next).await?;
    
    Ok(Message::response(&msg, serde_json::json!({"counter": next})))
}
```

## Best Practices

1. **Validate inputs** - Always check parameters
2. **Handle errors** - Return proper error types
3. **Log important events** - Use ctx.log()
4. **Limit resource usage** - Set timeouts and limits
5. **Document actions** - List supported actions

## Testing

```bash
# Unit tests
cargo test

# Integration test with runtime
cargo test --features integration

# Build for WASM
cargo build --target wasm32-wasi
```

## Publishing

1. Create a release on GitHub
2. Tag with `skill-v{version}`
3. Upload `.wasm` file
4. Submit to ClawHub marketplace

## License

MIT - See LICENSE file
