# BeeBotOS Message Bus

[![Crates.io](https://img.shields.io/crates/v/beebotos-message-bus)](https://crates.io/crates/beebotos-message-bus)
[![Documentation](https://docs.rs/beebotos-message-bus/badge.svg)](https://docs.rs/beebotos-message-bus)
[![License](https://img.shields.io/crates/l/beebotos-message-bus)](LICENSE)

A high-performance, unified message bus for BeeBotOS with multiple transport backends, designed for agent-based distributed systems.

## Features

- 🚀 **Multiple Transports**: In-memory, Redis, and gRPC cluster federation
- 📨 **Pub/Sub**: Topic-based messaging with wildcard support (`+`, `#`)
- 🔄 **Request-Reply**: Built-in RPC pattern
- 📊 **Observability**: Metrics, tracing, and health checks
- 🔒 **Type Safety**: Strongly-typed messages with serde
- 🌐 **Distributed**: Automatic cluster formation with gRPC
- 🛠️ **CLI Tool**: Command-line interface for debugging and operations

## Quick Start

### Add Dependency

```toml
[dependencies]
beebotos-message-bus = "1.0"
```

### Basic Usage

```rust
use beebotos_message_bus::{MessageBus, MemoryTransport, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create message bus with in-memory transport
    let bus = MemoryTransport::new();
    
    // Subscribe to events
    let (sub_id, mut stream) = bus.subscribe("agent/+/task/+").await?;
    
    // Publish a message
    let msg = Message::new("agent/123/task/start", serde_json::json!({
        "task_id": "task-456",
        "action": "analyze"
    }))?;
    bus.publish("agent/123/task/start", msg).await?;
    
    // Receive the message
    if let Some(message) = stream.recv().await {
        println!("Received: {:?}", message);
    }
    
    // Cleanup
    bus.unsubscribe(sub_id).await?;
    
    Ok(())
}
```

### Request-Reply Pattern

```rust
// Server side
let bus = MemoryTransport::new();
let (sub_id, mut stream) = bus.subscribe("rpc/echo").await?;

tokio::spawn(async move {
    while let Some(msg) = stream.recv().await {
        if let Some(reply_to) = &msg.metadata.reply_to {
            let response = Message::new(reply_to, msg.payload);
            let _ = bus.respond(&msg.id, response).await;
        }
    }
});

// Client side
let request = Message::new("rpc/echo", b"hello".to_vec());
let response = bus.request("rpc/echo", request, Duration::from_secs(5)).await?;
```

### Cluster Mode with gRPC

```rust
use beebotos_message_bus::{GrpcTransport, GrpcConfig};

let config = GrpcConfig {
    bind_addr: "0.0.0.0:50051".parse()?,
    cluster_addrs: vec!["192.168.1.100:50051".parse()?],
    node_id: "node-1".to_string(),
    ..Default::default()
};

let bus = GrpcTransport::new(config).await?;

// Messages automatically route across cluster
bus.publish("agent/123/task/start", msg).await?;
```

## CLI Tool

Install and use the `mbus-cli` tool:

```bash
# Start a server
cargo run --bin mbus-cli -- server --bind 0.0.0.0:50051

# Publish a message
mbus-cli publish agent/123/status '{"state":"active"}'

# Subscribe to topics
mbus-cli subscribe "agent/+/task/+" --format pretty

# Send request
mbus-cli request agent/123/task '{"type":"analyze"}' -t 10

# Check health
mbus-cli health

# View stats
mbus-cli stats --watch
```

## Transport Backends

| Feature | Memory | Redis | gRPC |
|---------|--------|-------|------|
| Single-node | ✅ | ✅ | ✅ |
| Multi-node | ❌ | ✅ | ✅ |
| Persistence | ❌ | ⚠️ | ❌ |
| Clustering | ❌ | ❌ | ✅ |
| Auto-discovery | ❌ | ❌ | ✅ |
| Request-Reply | ✅ | ✅ | ✅ |
| Wildcards | ✅ | ✅ | ✅ |
| Priorities | ✅ | ❌ | ✅ |

### Memory Transport
Best for single-node deployments and testing.

```rust
let bus = MemoryTransport::new();
```

### Redis Transport
For multi-node deployments with Redis as message broker.

```rust
use beebotos_message_bus::RedisBusBuilder;

let bus = RedisBusBuilder::new("redis://localhost:6379")
    .with_connection_pool_size(10)
    .build()
    .await?;
```

### gRPC Transport
For distributed clusters with automatic node discovery.

```rust
use beebotos_message_bus::{GrpcTransport, GrpcConfig};

let config = GrpcConfig {
    bind_addr: "0.0.0.0:50051".parse()?,
    cluster_addrs: vec!["192.168.1.100:50051".parse()?],
    ..Default::default()
};

let bus = GrpcTransport::new(config).await?;
```

## Topic Patterns

The message bus supports MQTT-style topic patterns:

- **Exact match**: `agent/123/task/start` matches `agent/123/task/start`
- **Single-level wildcard (`+`)**: `agent/+/task/start` matches any agent ID
- **Multi-level wildcard (`#`)**: `agent/#` matches any topic under `agent/`

```rust
// Subscribe to all agent status updates
bus.subscribe("agent/+/status").await?;

// Subscribe to everything
bus.subscribe("#").await?;

// Subscribe to specific agent's all events
bus.subscribe("agent/123/#").await?;
```

## Message Priorities

Messages can have priorities from 0 (lowest) to 9 (highest):

```rust
let mut msg = Message::new("urgent", payload);
msg.metadata.priority = 9; // Highest priority

let mut msg = Message::new("background", payload);
msg.metadata.priority = 0; // Lowest priority
```

## OpenTelemetry Tracing

Automatic distributed tracing support:

```rust
use beebotos_message_bus::TraceContext;

// Create trace context
let trace_ctx = TraceContext::new("trace-123", "span-456", true);

// Inject into message
let mut msg = Message::new("topic", payload);
msg.metadata.trace_context = Some(trace_ctx);

// Extract on receiver side
if let Some(ctx) = &msg.metadata.trace_context {
    println!("Trace ID: {}", ctx.trace_id);
}
```

## Metrics

Built-in metrics collection (requires `metrics` feature):

```rust
use beebotos_message_bus::MessageBusMetrics;

let metrics = MessageBusMetrics::new("message_bus");
let bus = MemoryTransport::with_metrics(metrics);

// Metrics available:
// - message_bus_published_total
// - message_bus_delivered_total
// - message_bus_latency_seconds
// - message_bus_subscriptions_active
```

## Performance

| Operation | Memory | Redis (Local) | gRPC (Local) |
|-----------|--------|---------------|--------------|
| Publish/sec | 2M+ | 50K | 100K |
| Latency (p99) | <1µs | 1-5ms | 1-2ms |
| Request latency | 5µs | 10-20ms | 5-10ms |

*Benchmarks on AMD Ryzen 9 5950X, 64GB RAM*

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │  Core   │  │ Agents  │  │ Kernel  │  │     Chain       │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
└───────┼────────────┼────────────┼────────────────┼──────────┘
        │            │            │                │
        └────────────┴─────┬──────┴────────────────┘
                           │
┌──────────────────────────┼──────────────────────────────────┐
│  ┌───────────────────────┴───────────────────────┐           │
│  │              Message Bus Trait                 │           │
│  └───────────────────────┬───────────────────────┘           │
│                          │                                   │
│  ┌───────────────────────┼───────────────────────┐           │
│  │              Transport Layer                   │           │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │           │
│  │  │ Memory  │  │  Redis  │  │    gRPC     │   │           │
│  │  └─────────┘  └─────────┘  └─────────────┘   │           │
│  └───────────────────────────────────────────────┘           │
└──────────────────────────────────────────────────────────────┘
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `memory` | In-memory transport (default) |
| `redis-transport` | Redis transport backend |
| `json-codec` | JSON serialization (default) |
| `msgpack-codec` | MessagePack serialization |
| `metrics` | Metrics collection |
| `persistence` | Message persistence |
| `cli` | CLI tool |
| `full` | All features enabled |

## Migration from Legacy EventBus

Use the compatibility adapters for gradual migration:

```rust
use beebotos_message_bus::compat::CoreEventBusAdapter;

// Wrap new bus with adapter
let new_bus = MemoryTransport::new();
let adapter = CoreEventBusAdapter::new(new_bus).await?;

// Use like old EventBus
adapter.publish("event", data).await?;
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Please read [CONTRIBUTING.md](../../CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## Acknowledgments

- Inspired by [NATS](https://nats.io/) and [MQTT](https://mqtt.org/)
- Built with [Tokio](https://tokio.rs/) and [Redis](https://redis.io/)
