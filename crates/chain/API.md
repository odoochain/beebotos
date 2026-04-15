# BeeBotOS Chain API Documentation

## Overview

The `beebotos-chain` crate provides blockchain integration and DAO governance capabilities for the BeeBotOS platform. It supports Ethereum-compatible chains including Monad.

## Table of Contents

- [Getting Started](#getting-started)
- [Core Types](#core-types)
- [Configuration](#configuration)
- [Wallet Management](#wallet-management)
- [Contract Interactions](#contract-interactions)
- [DAO Operations](#dao-operations)
- [Event Streaming](#event-streaming)
- [Metrics](#metrics)
- [Security](#security)
- [Error Handling](#error-handling)

## Getting Started

### Basic Setup

```rust
use beebotos_chain::config::ChainConfig;
use beebotos_chain::compat::AlloyClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure chain connection
    let config = ChainConfig::new("https://rpc.testnet.monad.xyz", 10143)?;
    
    // Create client
    let client = AlloyClient::new(&config.rpc_url).await?;
    
    // Get block number
    let block_number = client.get_block_number().await?;
    println!("Current block: {}", block_number);
    
    Ok(())
}
```

## Core Types

### Addresses and Hashes

```rust
use beebotos_chain::compat::{Address, B256, U256, TxHash};

// Create address from string
let addr: Address = "0x1234567890123456789012345678901234567890".parse()?;

// Create U256 from various types
let amount = U256::from(1000);
let amount = U256::from_str_radix("1000000000000000000", 10)?; // 1 ETH in wei

// Create transaction hash
let tx_hash = B256::from([1u8; 32]);
```

### Result Type

All operations return `beebotos_chain::Result<T>`:

```rust
use beebotos_chain::{Result, ChainError};

fn may_fail() -> Result<u64> {
    // On success
    Ok(42)
    
    // On error
    // Err(ChainError::Connection("Failed".to_string()))
}
```

## Configuration

### ChainConfig

```rust
use beebotos_chain::config::ChainConfig;

// Basic configuration
let config = ChainConfig::new("https://rpc.example.com", 1337)?;

// With contract addresses
let config = ChainConfig::new("https://rpc.example.com", 1337)?
    .with_dao_address("0x...")
    .with_token_address("0x...")
    .with_treasury_address("0x...")
    .with_identity_registry_address("0x...");

// From environment variables
let config = ChainConfig::from_env()?;

// Predefined configurations
let local = ChainConfig::local();
let testnet = ChainConfig::monad_testnet()?;
```

### Validation

```rust
// Validate all settings
config.validate()?;

// Validate contract addresses are configured
config.validate_contract_addresses()?;

// Get typed addresses
let dao_addr = config.get_dao_address()?;
```

## Wallet Management

### HD Wallet

```rust
use beebotos_chain::wallet::{HDWallet, Wallet};
use beebotos_chain::compat::U256;

// Generate new mnemonic
let mnemonic = HDWallet::generate_mnemonic(12)?;

// Create from existing mnemonic
let wallet = HDWallet::from_mnemonic(&mnemonic)?;

// Derive accounts
let account0 = wallet.derive_account(0, Some("Primary".to_string()))?;
let account1 = wallet.derive_account(1, None)?;

// Encrypt mnemonic
let encrypted = wallet.export_encrypted("password")?;
let decrypted = HDWallet::decrypt_mnemonic(&encrypted, "password")?;
```

### Simple Wallet

```rust
// Random wallet
let wallet = Wallet::random();
let address = wallet.address();

// From private key
let wallet = Wallet::from_key(&[1u8; 32])?;

// Sign message
let signature = wallet.sign_message(b"Hello").await?;
```

## Contract Interactions

### Contract Caller

```rust
use beebotos_chain::contracts::{ContractCaller, CallOptions};
use beebotos_chain::metrics::ContractMetrics;

// Create caller
let caller = ContractCaller::new(provider, contract_address);

// With metrics
let metrics = ContractMetrics::new("MyContract", metrics_collector);
let caller = caller.with_metrics(metrics);

// Call function (read-only)
let result: MyReturnType = caller.call(&my_call, CallOptions::default().static_call()).await?;

// Send transaction
let receipt = caller.send_transaction(&my_call, CallOptions::default()).await?;

// Estimate gas
let gas = caller.estimate_gas(&my_call, CallOptions::default()).await?;
```

### Contract Deployment

```rust
use beebotos_chain::contracts::ContractDeployer;

let deployer = ContractDeployer::new(provider);
let result = deployer.deploy(bytecode, Some(constructor_args)).await?;
println!("Deployed at: {}", result.contract_address);
```

## DAO Operations

### DAO Client

```rust
use beebotos_chain::dao::{DAOClient, ProposalBuilder, VoteType};

// Create client
let dao = DAOClient::new(provider, dao_address);

// With signer
let dao = dao.with_signer(signer);

// Get proposal count
let count = dao.proposal_count().await?;

// Get proposal
let proposal = dao.get_proposal(1).await?;

// Cast vote
// dao.cast_vote(1, VoteType::For).await?;
```

### Proposal Builder

```rust
use beebotos_chain::dao::{ProposalBuilder, ProposalType};

let builder = ProposalBuilder::new("My Proposal")
    .proposal_type(ProposalType::Standard)
    .add_transfer(recipient_address, amount)
    .add_contract_call(target, calldata, "function signature");

let (targets, values, calldatas, description) = builder.build();
```

## Event Streaming

### WebSocket Subscription

```rust
use beebotos_chain::monad::events::{EventListener, EventFilter, SubscriptionType};

let filter = EventFilter::new()
    .from_block(1000)
    .address(contract_address);

let listener = EventListener::new(provider, filter)
    .with_subscription_type(SubscriptionType::WebSocket);

let mut stream = listener.stream_events().await?;

while let Some(log) = stream.next().await {
    println!("New event: {:?}", log);
}
```

### Polling Fallback

```rust
let listener = EventListener::new(provider, filter)
    .with_subscription_type(SubscriptionType::Polling);
```

### Event Processing

```rust
use beebotos_chain::monad::events::EventHandler;

struct MyHandler;

#[async_trait::async_trait]
impl EventHandler for MyHandler {
    async fn on_proposal_created(&self, event: ProposalCreated) {
        println!("New proposal: {}", event.proposalId);
    }
    
    async fn on_vote_cast(&self, event: VoteCast) {
        println!("Vote cast: {}", event.proposalId);
    }
}

let processor = EventProcessor::new(listener, Box::new(MyHandler));
processor.start().await?;
```

## Metrics

### Metrics Collector

```rust
use beebotos_chain::metrics::{MetricsCollector, OperationType};

let metrics = MetricsCollector::new();

// Record operation
metrics.record_operation(OperationType::Transaction, true, duration);

// Record transaction
metrics.record_transaction("transfer", true, Some(duration));

// Update chain state
metrics.set_gas_price(20.0);
metrics.set_block_number(1000);
metrics.set_pending_transactions(5);
```

### Prometheus Integration

```rust
#[cfg(feature = "prometheus")]
{
    let handle = beebotos_chain::init_prometheus_exporter("0.0.0.0:9090")?;
    // Metrics available at http://localhost:9090/metrics
}
```

### Contract Metrics

```rust
use beebotos_chain::metrics::ContractMetrics;

let contract_metrics = ContractMetrics::new("AgentDAO", metrics_collector);
contract_metrics.record_function_call("propose", true, duration);
contract_metrics.record_gas_usage("propose", 150000);
```

## Security

### Security Validator

```rust
use beebotos_chain::security::SecurityValidator;

let validator = SecurityValidator::new()
    .max_gas_price(100_000_000_000) // 100 gwei
    .max_value(10_000_000_000_000_000_000) // 10 ETH
    .blacklist(suspicious_address)
    .whitelist_mode()
    .whitelist(allowed_address);

// Validate transaction
validator.validate_transaction(to, value, gas_price, &data)?;
validator.validate_contract_address(contract_addr)?;
```

### Reentrancy Guard

```rust
use beebotos_chain::security::ReentrancyGuard;

static GUARD: ReentrancyGuard = ReentrancyGuard::new();

async fn withdraw() -> Result<()> {
    let _lock = GUARD.try_lock()?;
    
    // Perform withdrawal - protected against reentrancy
    
    Ok(()) // Lock released automatically
}
```

### Nonce Manager

```rust
use beebotos_chain::security::NonceManager;

let nonce_manager = NonceManager::new();

// Get next nonce
let nonce = nonce_manager.get_next_nonce(address);

// Validate specific nonce
nonce_manager.validate_nonce(address, expected_nonce)?;

// Set custom nonce
nonce_manager.set_nonce(address, 100);
```

### Rate Limiting

```rust
use beebotos_chain::security::CallRateLimiter;

let limiter = CallRateLimiter::new(10, 60); // 10 calls per minute

limiter.check()?; // Check if call is allowed
```

### Security Audit

```rust
use beebotos_chain::security::SecurityAudit;

SecurityAudit::log_event("LOGIN", "User authenticated");
SecurityAudit::log_suspicious("Multiple failed attempts", Some(address));
SecurityAudit::log_violation("Reentrancy detected", true);
```

## Error Handling

### Error Types

```rust
use beebotos_chain::ChainError;

match error {
    ChainError::Connection(msg) => println!("Connection failed: {}", msg),
    ChainError::InvalidAddress(addr) => println!("Invalid address: {}", addr),
    ChainError::InsufficientBalance => println!("Insufficient funds"),
    ChainError::NotImplemented(feature) => println!("Not implemented: {}", feature),
    _ => println!("Other error: {}", error),
}
```

### Retry Logic

```rust
use beebotos_chain::compat::retry::{with_retry, RetryConfig};

let config = RetryConfig::default();
let result = with_retry(&config, || async {
    // Operation that might fail temporarily
    provider.get_balance(address).await
}).await?;
```

### Circuit Breaker

```rust
use beebotos_chain::compat::retry::CircuitBreaker;

let mut cb = CircuitBreaker::new(5, 3, 60); // 5 failures, 3 successes, 60s timeout

if cb.allow_request() {
    match operation().await {
        Ok(_) => cb.record_success(),
        Err(_) => cb.record_failure(),
    }
}
```

## Best Practices

### 1. Always validate inputs

```rust
use beebotos_chain::security::InputSanitizer;

let sanitized = InputSanitizer::sanitize_address(user_input)?;
```

### 2. Use metrics for monitoring

```rust
metrics.record_operation(OperationType::ContractCall, result.is_ok(), duration);
```

### 3. Handle errors properly

```rust
match result {
    Ok(value) => value,
    Err(ChainError::NotImplemented(_)) => {
        // Feature not yet available
        default_value
    }
    Err(e) => return Err(e.into()),
}
```

### 4. Use WebSocket for real-time events when available

```rust
let listener = EventListener::new(provider, filter)
    .with_subscription_type(SubscriptionType::WebSocket);
// Falls back to polling automatically if WebSocket unavailable
```

## Examples

See the `examples/` directory for complete working examples:

- `basic_connection.rs` - Basic chain connection
- `wallet_management.rs` - Wallet operations
- `contract_interaction.rs` - Contract calls
- `dao_operations.rs` - DAO governance
- `event_streaming.rs` - Real-time events

## Feature Flags

- `prometheus` - Enable Prometheus metrics export (enabled by default)
- `testing` - Enable test utilities

## Version

Current version: `1.0.0`
