# Skill: Market Maker

## Description

Autonomous market making agent for DEX liquidity provision and spread optimization.

## Capabilities

```yaml
functions:
  - name: analyze_orderbook
    description: Analyze orderbook depth and spreads
  - name: adjust_spread
    description: Dynamically adjust bid-ask spread
  - name: rebalance_inventory
    description: Rebalance token inventory
  - name: hedge_exposure
    description: Hedge directional exposure
```

## Configuration

```yaml
config:
  target_spread: 0.001
  max_position_size: 10000
  rebalance_threshold: 0.1
```
