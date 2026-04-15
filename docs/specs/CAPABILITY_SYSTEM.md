# Capability System Specification

## Overview

The BeeBotOS Capability System provides fine-grained, hierarchical access control for agent operations.

## Capability Levels

### Hierarchy (L0 - L10)

| Level | Name | Description | TEE Required |
|-------|------|-------------|--------------|
| L0 | Local Compute | Sandboxed computation only | No |
| L1 | File Read | Read-only filesystem access | No |
| L2 | File Write | Read/write filesystem access | No |
| L3 | Network Out | Outbound network connections | No |
| L4 | Network In | Accept inbound connections | No |
| L5 | Spawn Limited | Spawn up to 10 child agents | No |
| L6 | Spawn Unlimited | Unlimited agent spawning | No |
| L7 | Chain Read | Read blockchain state | No |
| L8 | Chain Write Low | Transactions < 1 ETH | No |
| L9 | Chain Write High | High-value transactions | Yes |
| L10 | System Admin | Full system control | Yes |

## Capability Model

### Structure

```rust
pub struct CapabilitySet {
    pub max_level: CapabilityLevel,
    pub permissions: HashSet<String>,
    pub expires_at: Option<u64>,
    pub delegable: bool,
}
```

### Permissions

Granular permissions within each level:

```
compute.local
compute.gpu
compute.distributed

filesystem.read.{path}
filesystem.write.{path}

network.outbound.{host}
network.inbound.{port}

agents.spawn.{max_count}
agents.terminate.own
agents.terminate.others

chain.read.{contract}
chain.write.{contract}.{value_limit}

system.shutdown
system.upgrade
```

## Time Decay

High-level capabilities decay over time:

```rust
pub struct DecayingCapability {
    pub level: CapabilityLevel,
    pub granted_at: u64,
    pub decay_rate: DecayRate,
}

pub enum DecayRate {
    Slow,    // 1 level per day
    Normal,  // 1 level per hour
    Fast,    // 1 level per 10 minutes
}
```

## Delegation

### Rules

1. **Attenuation**: Can only delegate subset of capabilities
2. **Depth**: Maximum delegation chain depth (default: 3)
3. **Revocation**: Original grantor can revoke at any time
4. **Expiration**: Delegated capabilities inherit expiration

### Delegation Chain

```
Root Authority
    ↓ (L10, expires: 1 day)
System Agent
    ↓ (L8, expires: 1 hour)
User Agent
    ↓ (L5, expires: 10 min)
Child Agent
```

## Verification

### Process

1. Check if capability is expired
2. Verify level meets requirement
3. Check specific permission
4. Verify delegation chain (if applicable)

### Code Example

```rust
impl CapabilitySet {
    pub fn verify(&self, required: CapabilityLevel) -> Result<()> {
        if self.is_expired() {
            return Err(CapabilityError::Expired);
        }
        if self.max_level < required {
            return Err(CapabilityError::Insufficient {
                required,
                current: self.max_level,
            });
        }
        Ok(())
    }
}
```

## System Calls Mapping

| Syscall | Required Level |
|---------|---------------|
| SpawnAgent | L5 |
| TerminateAgent | L6 |
| SendMessage | L3 |
| ReceiveMessage | L4 |
| OpenFile | L1 |
| WriteFile | L2 |
| QueryChain | L7 |
| SubmitTransaction | L8 |
| CreateProposal | L9 |
| SystemShutdown | L10 |

## Escalation

### Automatic Escalation

Certain conditions trigger automatic escalation:
- Reputation score > 8000
- Successful task completion rate > 95%
- No security violations in 30 days

### Manual Request

```rust
pub struct EscalationRequest {
    pub requested_level: CapabilityLevel,
    pub justification: String,
    pub duration_seconds: u64,
}
```

## Security Considerations

1. **Principle of Least Privilege**: Grant minimum necessary capabilities
2. **Time Limiting**: Always set expiration
3. **Auditing**: Log all capability checks and escalations
4. **Revocation**: Implement fast revocation for compromised agents

## Best Practices

### For Developers

```rust
// Good: Specific permission
let caps = CapabilitySet::standard()
    .with_permission("network.outbound.api.example.com");

// Bad: Too broad
let caps = CapabilitySet::full();
```

### For Users

1. Review agent capability requests carefully
2. Set short expiration times
3. Monitor capability usage
4. Revoke unused capabilities

## References

- [Capability-based security](https://en.wikipedia.org/wiki/Capability-based_security)
- [Object-capability model](https://en.wikipedia.org/wiki/Object-capability_model)
