# A2A Protocol Specification v1.0

## Overview

The Agent-to-Agent (A2A) Protocol enables autonomous AI agents to discover, negotiate, and collaborate with each other in a decentralized manner.

## Core Concepts

### Agents

Agents are autonomous entities that:
- Expose capabilities through standardized interfaces
- Can discover and interact with other agents
- Participate in economic transactions
- Maintain reputation scores

### Capabilities

A capability represents a service an agent can provide:

```protobuf
message Capability {
  string uri = 1;              // Unique identifier
  string name = 2;             // Human-readable name
  string version = 3;          // Semantic version
  
  // Input/Output schemas
  Schema input_schema = 4;
  Schema output_schema = 5;
  
  // Pricing
  PricingModel pricing = 6;
  
  // Quality metrics
  QualityMetrics quality = 7;
}
```

### Intents

An intent represents a desired outcome:

```protobuf
message Intent {
  string id = 1;
  IntentType type = 2;
  repeated Constraint constraints = 3;
  repeated Preference preferences = 4;
}
```

## Protocol Flow

### 1. Discovery

```
┌─────────┐                    ┌─────────┐
│ Agent A │ ──Discover───────→ │ Registry│
│         │                    │         │
│         │ ←─Capabilities──── │         │
└─────────┘                    └─────────┘
```

**Methods:**
- `announce_capability` - Register a capability
- `discover` - Search for capabilities
- `query` - Get specific agent info

### 2. Negotiation

```
┌─────────┐                    ┌─────────┐
│ Agent A │ ──Offer───────────→│ Agent B │
│ (Buyer) │                    │(Seller) │
│         │ ←─Counter/Ack───── │         │
│         │ ──Accept/Reject───→│         │
└─────────┘                    └─────────┘
```

**Negotiation Pattern:**
1. Intent broadcast or direct request
2. Capability matching
3. Terms negotiation (price, deadline, quality)
4. Agreement formation

### 3. Execution

```
┌─────────┐                    ┌─────────┐
│ Agent A │ ──Task Assignment─→│ Agent B │
│         │                    │         │
│         │ ←─Progress Updates─│         │
│         │                    │         │
│         │ ←─Delivery─────────│         │
└─────────┘                    └─────────┘
```

### 4. Settlement

```
┌─────────┐     ┌──────────┐     ┌─────────┐
│ Agent A │────→│  Escrow  │────→│ Agent B │
│         │     │ Contract │     │         │
│         │←────│          │←────│         │
└─────────┘     └──────────┘     └─────────┘
```

## Message Types

### Discovery Messages

| Message | Direction | Description |
|---------|-----------|-------------|
| `CapabilityAnnouncement` | Broadcast | Announce available capability |
| `DiscoveryQuery` | Request | Search for capabilities |
| `DiscoveryResponse` | Response | Return matching capabilities |

### Negotiation Messages

| Message | Direction | Description |
|---------|-----------|-------------|
| `Intent` | To target | Express intent |
| `Offer` | Bidirectional | Propose terms |
| `CounterOffer` | Bidirectional | Counter proposal |
| `Accept` | Bidirectional | Accept terms |
| `Reject` | Bidirectional | Reject terms |

### Execution Messages

| Message | Direction | Description |
|---------|-----------|-------------|
| `TaskAssignment` | To provider | Assign task |
| `ProgressUpdate` | To client | Report progress |
| `Delivery` | To client | Deliver results |
| `Confirmation` | To provider | Confirm receipt |

## Security

### Authentication

- Agents identified by DID
- All messages cryptographically signed
- mTLS for transport security

### Authorization

- Capability-based access control
- Reputation-based trust
- Smart contract enforcement

### Privacy

- End-to-end encryption optional
- Selective disclosure of capabilities
- Zero-knowledge proofs for verification

## Economic Model

### Pricing Strategies

1. **Fixed** - Set price for service
2. **Dynamic** - Market-based pricing
3. **Negotiable** - Bilateral negotiation
4. **Subscription** - Recurring payments

### Payment Flow

1. Buyer locks funds in escrow
2. Service provider executes task
3. Delivery confirmed by buyer or arbiter
4. Funds released to provider

## Implementation Requirements

### Minimum Requirements

- Support for WebSocket transport
- JSON/Protobuf message encoding
- ECDSA signature verification
- Basic reputation tracking

### Recommended Features

- gRPC support
- libp2p integration
- Multiple payment tokens
- Advanced reputation algorithms

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-03-10 | Initial specification |
