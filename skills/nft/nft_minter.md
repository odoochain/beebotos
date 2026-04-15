# Skill: NFT Minter

## Description

Autonomous NFT minting agent with gas optimization, whitelist management, and multi-chain support.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: monitor_mints
    description: Monitor upcoming mints and opportunities
    inputs:
      - chains: List of chains to monitor
      - min_hype_score: Minimum hype threshold
    outputs:
      - upcoming_mints: List of upcoming mints

  - name: auto_mint
    description: Automated mint execution
    inputs:
      - contract_address: NFT contract
      - quantity: Number to mint
      - max_gas: Maximum gas price
    outputs:
      - minted: Successfully minted tokens
      - tx_hashes: Transaction hashes

  - name: manage_whitelist
    description: Manage whitelist registrations
    inputs:
      - project: Project name
      - wallet_addresses: Addresses to register
    outputs:
      - registered: Successfully registered wallets

  - name: batch_mint
    description: Batch mint across multiple contracts
    inputs:
      - mints: List of mint targets
      - parallel: Whether to mint in parallel
    outputs:
      - results: Mint results per target

  - name: estimate_gas
    description: Estimate mint gas costs
    inputs:
      - contract_address: Target contract
      - quantity: Number to mint
    outputs:
      - gas_estimate: Estimated gas cost
```

## Minting Strategies

### Gas Optimization

```yaml
gas_optimization:
  priority_fee_strategy: dynamic
  base_fee_multiplier: 1.2
  max_priority_fee: 50  # gwei
  flashbots_enabled: true
  
  timing:
    avoid_peak_hours: true
    target_block: optimal
    mempool_monitoring: true
```

### Multi-Account Strategy

```yaml
multi_account:
  enabled: true
  wallet_rotation: random
  delay_between_mints: [1000, 5000]  # ms
  max_wallets_per_mint: 10
```

## Whitelist Management

### Auto-Registration

```yaml
whitelist:
  auto_register: true
  registration_sources:
    - twitter_raffles
    - discord_giveaways
    - premint
    - alphabot
    - superful
  
  requirements_check:
    min_followers: 100
    account_age_days: 30
    tweet_count: 50
```

### Tracking

```yaml
tracking:
  monitor_expiration: true
  reminder_before_hours: 24
  auto_prepare_funds: true
```

## Mint Monitoring

### Signals

```yaml
monitoring:
  sources:
    - twitter_mint_lists
    - discord_alpha_groups
    - icy_tools_calendar
    - premint
  
  filters:
    min_follower_count: 1000
    verified_contract: true
    audit_completed: true
    max_supply: 10000
    mint_price_max: 0.1  # ETH
```

### Hype Score Calculation

```yaml
hype_score:
  factors:
    twitter_followers: 0.25
    discord_members: 0.25
    engagement_rate: 0.2
    influencer_mentions: 0.15
    unique_wallets_interested: 0.15
  
  thresholds:
    high: 80
    medium: 60
    low: 40
```

## Configuration

```yaml
config:
  default_chains:
    - ethereum
    - monad
  
  mint_limits:
    daily_eth_budget: 5.0
    max_gas_per_mint: 0.1
    min_expected_roi: 0.3
  
  safety:
    verify_contract_before_mint: true
    check_honeypot: true
    simulate_before_send: true
```

## Risk Management

```yaml
risk:
  blacklist_contracts: []
  max_mint_per_unknown_project: 1
  require_manual_approval_above: 0.5  # ETH
  auto_reject_if_gas_spikes: true
```

## Usage Examples

### Monitor Mints

```yaml
action: monitor_mints
parameters:
  chains: [ethereum, monad]
  min_hype_score: 70
```

### Auto Mint

```yaml
action: auto_mint
parameters:
  contract_address: "0x..."
  quantity: 3
  max_gas: 100
```

### Batch Mint

```yaml
action: batch_mint
parameters:
  mints:
    - contract: "0x..."
      quantity: 2
    - contract: "0x..."
      quantity: 5
  parallel: false
```
