# Skill: DeFi Yield Farmer

## Description

An autonomous DeFi yield farming agent that optimizes yield across multiple protocols and chains.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: analyze_opportunities
    description: Scan and analyze yield opportunities across protocols
    inputs:
      - protocols: List of protocols to analyze
      - min_apy: Minimum APY threshold
      - risk_tolerance: Risk tolerance level
    outputs:
      - opportunities: Ranked list of yield opportunities

  - name: execute_deposit
    description: Execute deposit into a yield protocol
    inputs:
      - protocol: Target protocol
      - amount: Amount to deposit
      - token: Token to deposit
    outputs:
      - tx_hash: Transaction hash
      - position_id: Position identifier

  - name: harvest_rewards
    description: Harvest rewards from active positions
    inputs:
      - position_ids: List of positions to harvest
    outputs:
      - harvested: Amount harvested per position

  - name: rebalance_portfolio
    description: Rebalance portfolio based on market conditions
    inputs:
      - target_allocation: Desired allocation percentages
      - slippage_tolerance: Maximum slippage allowed
    outputs:
      - rebalanced: Rebalancing summary

  - name: monitor_positions
    description: Monitor active positions for risks or opportunities
    outputs:
      - alerts: List of alerts and recommendations
```

## Supported Protocols

- **Lending**: Aave, Compound, Morpho
- **DEX AMM**: Uniswap V3, Curve, Balancer
- **Yield Aggregators**: Yearn, Beefy, Convex
- **Liquid Staking**: Lido, Rocket Pool, Frax
- **Monad Native**: Pending protocol launches

## Risk Management

### Parameters

```yaml
risk_management:
  max_position_size: 10000  # USD
  max_protocol_exposure: 0.25  # 25% per protocol
  max_slippage: 0.005  # 0.5%
  emergency_withdrawal_threshold: 0.15  # 15% loss
  rebalancing_interval_hours: 24
```

### Safety Checks

- Smart contract audit verification
- TVL thresholds
- Impermanent loss estimation
- Protocol exploit monitoring

## Configuration

```yaml
config:
  default_chain: monad
  supported_chains:
    - monad
    - ethereum
    - arbitrum
    - optimism
    - base
  gas_optimization: true
  auto_compound: true
  compounding_interval_hours: 168  # Weekly
```

## Usage Examples

### Analyze Opportunities

```yaml
action: analyze_opportunities
parameters:
  protocols: [aave, compound, morpho]
  min_apy: 0.05
  risk_tolerance: medium
```

### Execute Strategy

```yaml
action: execute_deposit
parameters:
  protocol: aave
  token: USDC
  amount: "5000"
```

## Events

```yaml
events:
  - name: OpportunityFound
    fields: [protocol, apy, tvl, risk_score]
  - name: PositionOpened
    fields: [position_id, protocol, amount, tx_hash]
  - name: RewardsHarvested
    fields: [position_id, reward_amount, token]
  - name: RiskAlert
    fields: [alert_type, severity, position_id, recommendation]
```

## API Integration

```yaml
apis:
  defillama: https://api.llama.fi
  defipulse: https://data-api.defipulse.com
  custom_oracle: https://api.beebotos.io/defi
```
