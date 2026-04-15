# A2A Protocol Specification

## Overview

The Agent-to-Agent (A2A) Protocol enables secure, standardized communication between autonomous agents in the BeeBotOS ecosystem.

## Protocol Version

Current Version: 2.0.0

## Message Format

### Envelope

```json
{
  "version": "1.0.0",
  "id": "uuid-v4",
  "timestamp": 1704067200,
  "ttl": 300,
  "encryption": null,
  "compression": null,
  "payload": {}
}
```

### Message Types

#### 1. Ping/Pong (Health Check)

**Ping Request:**
```json
{
  "msg_type": "ping",
  "from": "agent_id",
  "to": "agent_id",
  "payload": {
    "nonce": 12345
  }
}
```

**Pong Response:**
```json
{
  "msg_type": "pong",
  "from": "agent_id",
  "to": "agent_id",
  "payload": {
    "nonce": 12345,
    "latency_ms": 15
  }
}
```

#### 2. Request/Response (RPC)

**Request:**
```json
{
  "msg_type": "request",
  "from": "agent_id",
  "to": "agent_id",
  "payload": {
    "action": "compute",
    "params": {
      "task": "matrix_multiply",
      "data": [[1, 2], [3, 4]]
    },
    "request_id": "req-123"
  }
}
```

**Response:**
```json
{
  "msg_type": "response",
  "from": "agent_id",
  "to": "agent_id",
  "payload": {
    "request_id": "req-123",
    "success": true,
    "data": [[7, 10], [15, 22]],
    "error": null
  }
}
```

#### 3. Event (Pub-Sub)

```json
{
  "msg_type": "event",
  "from": "agent_id",
  "to": null,
  "payload": {
    "event_type": "agent.spawned",
    "data": {
      "agent_id": "new_agent_id",
      "capabilities": ["compute"]
    },
    "topic": "system.events"
  }
}
```

#### 4. Task Distribution

**Task Assignment:**
```json
{
  "msg_type": "task",
  "from": "coordinator_agent",
  "to": "worker_agent",
  "payload": {
    "task_id": "task-456",
    "description": "Process dataset",
    "requirements": ["gpu", "16gb_ram"],
    "deadline": 1704153600,
    "reward": 100
  }
}
```

**Task Result:**
```json
{
  "msg_type": "task_result",
  "from": "worker_agent",
  "to": "coordinator_agent",
  "payload": {
    "task_id": "task-456",
    "result": {
      "status": "completed",
      "output": "processed_data.json",
      "metrics": {
        "duration_ms": 5000,
        "cpu_usage": 80
      }
    }
  }
}
```

## Transport

### WebSocket

Primary transport for real-time communication:

```
ws://agent-address:port/a2a/v1
```

### HTTP

For request-response patterns:

```
POST http://agent-address:port/a2a/v1/message
```

## Security

### Authentication

1. **DID-based**: Agents identify using Decentralized Identifiers
2. **Signature**: All messages must be signed
3. **Verification**: Receivers verify sender signatures

### Encryption

- **Transport**: TLS 1.3 for WebSocket/HTTP
- **Message**: Optional end-to-end encryption

## Discovery

### Service Registry

Agents register their capabilities:

```json
{
  "agent_id": "agent_123",
  "endpoint": "ws://10.0.0.1:9001",
  "capabilities": ["compute", "storage"],
  "reputation": 7500,
  "last_seen": 1704067200
}
```

### Query

```json
{
  "msg_type": "discover",
  "payload": {
    "capability": "compute",
    "min_reputation": 5000
  }
}
```

## Error Handling

### Error Codes

| Code | Description |
|------|-------------|
| 100 | Invalid message format |
| 101 | Unsupported version |
| 102 | Authentication failed |
| 103 | Capability not available |
| 200 | Timeout |
| 500 | Internal error |

### Error Message

```json
{
  "msg_type": "error",
  "payload": {
    "code": 102,
    "message": "Authentication failed",
    "details": "Invalid signature"
  }
}
```

## Flow Diagrams

### Request-Response

```
Agent A          Agent B
  |                 |
  |--- Request --->|
  |                 |
  |<-- Response ----|
  |                 |
```

### Publish-Subscribe

```
Agent A          Broker          Agent B    Agent C
  |                 |               |          |
  |--- Subscribe -->|               |          |
  |                 |<-- Subscribe--|          |
  |                 |<-- Subscribe-------------|
  |                 |               |          |
  |--- Publish ---->|               |          |
  |                 |--- Event ---->|          |
  |                 |--- Event ---------------->|
  |                 |               |          |
```

## Implementation Notes

1. **Message ID**: Must be globally unique (UUID v4 recommended)
2. **Timestamp**: Unix timestamp in seconds
3. **TTL**: Time-to-live in seconds (0 = no expiration)
4. **Ordering**: No guaranteed ordering, use request_id for correlation
5. **Retries**: Exponential backoff recommended

## References

- [DID Specification](https://www.w3.org/TR/did-core/)
- [W3C DID Registry](https://www.w3.org/TR/did-spec-registries/)
