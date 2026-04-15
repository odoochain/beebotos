# Skill: Arbitrageur

## Description

Cross-DEX and cross-chain arbitrage detection and execution.

## Capabilities

```yaml
functions:
  - name: scan_opportunities
    description: Scan for arbitrage opportunities
  - name: calculate_profit
    description: Calculate net profit after fees
  - name: execute_arbitrage
    description: Execute flash loan arbitrage
```

## Configuration

```yaml
config:
  min_profit_basis_points: 10
  max_slippage: 0.005
  use_flash_loans: true
```
