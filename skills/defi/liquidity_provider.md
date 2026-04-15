# Skill: Liquidity Provider

## Description

Automated liquidity provision across DEXs with impermanent loss protection and dynamic rebalancing.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: analyze_pools
    description: Analyze liquidity pools for optimal entry
    inputs:
      - dex: Target DEX
      - token_pair: Token pair
      - investment_amount: Total investment
    outputs:
      - pool_analysis: Pool metrics and recommendations

  - name: add_liquidity
    description: Add liquidity to a pool
    inputs:
      - pool_address: Pool contract address
      - token_a: First token
      - token_b: Second token
      - amount_a: Amount of token A
      - amount_b: Amount of token B
    outputs:
      - lp_tokens: LP token amount received
      - tx_hash: Transaction hash

  - name: remove_liquidity
    description: Remove liquidity from a pool
    inputs:
      - pool_address: Pool contract address
      - lp_amount: Amount of LP tokens to remove
    outputs:
      - token_a_received: Token A amount
      - token_b_received: Token B amount
      - tx_hash: Transaction hash

  - name: rebalance_position
    description: Rebalance liquidity position
    inputs:
      - position_id: Position to rebalance
      - new_range: New price range (for concentrated liquidity)
    outputs:
      - new_position: Updated position details

  - name: calculate_il
    description: Calculate impermanent loss
    inputs:
      - entry_price_a: Entry price of token A
      - entry_price_b: Entry price of token B
      - current_price_a: Current price of token A
      - current_price_b: Current price of token B
    outputs:
      - il_percentage: Impermanent loss percentage
```

## Supported DEXs

- **Uniswap V3**: Concentrated liquidity
- **Curve**: Stable and volatile pools
- **Balancer**: Weighted pools
- **Trader Joe**: Monad native
- **Camelot**: Arbitrum native

## Concentrated Liquidity Strategy

### Auto-Range Strategy

```yaml
auto_range:
  enabled: true
  width_percentage: 0.5  # ±50% of current price
  rebalance_threshold: 0.1  # Rebalance when price moves 10%
  auto_compound_fees: true
```

### Rebalancing Logic

```yaml
rebalancing:
  triggers:
    - price_outside_range
    - il_exceeds_threshold
    - fee_apy_below_target
  actions:
    - remove_liquidity
    - swap_to_optimal_ratio
    - add_liquidity_new_range
```

## Impermanent Loss Protection

### Hedging Strategies

```yaml
il_protection:
  strategies:
    - name: options_hedge
      provider: hegic
      coverage: 0.5
    - name: perp_hedge
      provider: gmx
      leverage: 2
    - name: delta_neutral
      description: Maintain delta neutral via lending
```

### Monitoring

```yaml
monitoring:
  check_interval_minutes: 5
  alerts:
    - il_exceeds_5_percent
    - price_outside_range
    - fee_apy_drops_below_10
```

## Configuration

```yaml
config:
  max_slippage: 0.005
  deadline_minutes: 20
  min_liquidity_usd: 100000
  preferred_fee_tiers:
    - 0.05  # For stable pairs
    - 0.3   # For standard pairs
    - 1.0   # For exotic pairs
```

## Risk Management

```yaml
risk_params:
  max_single_pool_allocation: 0.3
  max_il_tolerance: 0.1
  emergency_exit_enabled: true
  auto_reduce_on_volatility: true
```

## Usage Examples

### Add Liquidity to Uniswap V3

```yaml
action: add_liquidity
parameters:
  dex: uniswap_v3
  pool_address: "0x..."
  token_a: USDC
  token_b: ETH
  amount_a: "10000"
  amount_b: "5"
  tick_lower: -1000
  tick_upper: 1000
```

### Analyze Pool Before Entry

```yaml
action: analyze_pools
parameters:
  dex: curve
  token_pair: [USDC, USDT]
  investment_amount: 50000
```
