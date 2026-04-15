# Skill: Bridge Operator

## Description

Autonomous cross-chain bridge operator for asset transfers.

## Capabilities

```yaml
functions:
  - name: monitor_bridges
    description: Monitor bridge status
  - name: execute_transfer
    description: Execute cross-chain transfer
```

## Configuration

```yaml
config:
  default_bridge: "native"
  max_transfer_amount: 1000000
```
