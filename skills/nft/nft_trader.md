# Skill: NFT Trader

## Description

Autonomous NFT trading agent with sniping, flipping, and collection analysis capabilities.

## Version

- **Version**: 1.0.0
- **Updated**: 2025-03-10

## Capabilities

### Core Functions

```yaml
functions:
  - name: snipe_listings
    description: Monitor and snipe underpriced listings
    inputs:
      - collection: Target collection
      - max_price: Maximum price willing to pay
      - traits: Desired traits (optional)
    outputs:
      - sniped_items: List of successfully sniped NFTs

  - name: analyze_collection
    description: Deep analysis of NFT collection
    inputs:
      - contract_address: Collection contract
      - analysis_depth: quick | standard | deep
    outputs:
      - report: Collection analysis report

  - name: estimate_floor_price
    description: Predict floor price movements
    inputs:
      - collection: Target collection
      - timeframe: Prediction timeframe
    outputs:
      - prediction: Price prediction with confidence

  - name: batch_bid
    description: Place batch bids on multiple items
    inputs:
      - collection: Target collection
      - bid_price: Bid price per item
      - quantity: Number of bids
    outputs:
      - bids_placed: Successfully placed bids

  - name: track_whale_wallets
    description: Monitor whale wallet movements
    inputs:
      - wallets: List of whale addresses
      - collections: Collections to track
    outputs:
      - alerts: Whale activity alerts
```

## Marketplaces

- **OpenSea**
- **Blur**
- **LooksRare**
- **Magic Eden** (Solana/Monad)
- **Tensor** (Solana)

## Sniping Strategy

### Configuration

```yaml
sniping:
  mode: aggressive | balanced | conservative
  max_gas_premium: 50  # gwei
  flashbots_enabled: true
  private_mempool: true
  
  filters:
    min_profit_margin: 0.15  # 15%
    max_listing_age_seconds: 30
    exclude_flagged: true
    min_trait_rarity: rare
```

### Auto-Snipe Conditions

```yaml
auto_snipe:
  triggers:
    - price_below_floor_percent: 80
    - rare_trait_at_floor: true
    - whale_listing_detected: true
    - collection_volume_spike: 2.0
```

## Collection Analysis

### Metrics

```yaml
analysis_metrics:
  floor_price:
    current: float
    change_24h: float
    change_7d: float
    all_time_high: float
  
  volume:
    volume_24h: float
    volume_7d: float
    volume_30d: float
  
  holders:
    unique_holders: int
    unique_percent: float
    whale_count: int
  
  listings:
    listed_count: int
    listed_percent: float
    listing_trend: increasing | stable | decreasing
```

### Rarity Scoring

```yaml
rarity:
  providers:
    - rarity_sniper
    - trait_sniper
    - openrarity
  
  custom_weights:
    trait_rarity: 0.4
    trait_count: 0.2
    statistical_rarity: 0.4
```

## Trading Strategies

### Flip Strategy

```yaml
flip:
  hold_duration_max: 48  # hours
  min_profit_target: 0.2
  max_loss_tolerance: 0.05
  gas_optimization: true
```

### Collection Betting

```yaml
collection_betting:
  entry_signals:
    - volume_increase_7d: 2.0
    - social_mentions_spike: 3.0
    - holder_growth_rate: 0.1
  
  exit_signals:
    - profit_target: 0.5
    - stop_loss: -0.15
    - momentum_loss: true
```

## Risk Management

```yaml
risk:
  max_position_per_collection: 0.2
  max_daily_spend: 10.0  # ETH
  max_open_bids: 50
  blacklist_collections: []
  scam_detection: true
```

## API Integrations

```yaml
apis:
  market_data:
    - reservoir
    - nftgo
    - icy_tools
  
  rarity:
    - rarity_sniper
    - trait_sniper
  
  social:
    - lunarcrush
    - twitter_api
```

## Usage Examples

### Snipe Rare Items

```yaml
action: snipe_listings
parameters:
  collection: "0x..."
  max_price: "1.5"
  traits:
    - trait_type: Background
      value: Gold
```

### Analyze Collection

```yaml
action: analyze_collection
parameters:
  contract_address: "0x..."
  analysis_depth: deep
```

### Batch Bid

```yaml
action: batch_bid
parameters:
  collection: "0x..."
  bid_price: "0.8"
  quantity: 10
```
