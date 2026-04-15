//! Beechain Module
//!
//! Beechain is a high-performance EVM-compatible Layer 1 blockchain.
//! - Chain ID: 3188 (Mainnet)
//! - Block Time: ~0.4 seconds
//! - TPS: 10,000 transactions per second
//! - Finality: ~0.8 seconds (2 blocks)
//! - Consensus: Parallel EVM Execution
//! - Network Series: Ethereum series
//!
//! ## Official Resources
//! - RPC: https://rpc.beechain.ai
//! - Explorer: https://scan.beechain.ai
//! - Native Token: BKC (Beechain Coin)

pub mod client;
pub mod types;

// Re-export common types for convenience
pub use crate::chains::common::{
    format_native_token, parse_native_token, ContractCall, ContractDeploy, ContractInstance,
    EventFilter, EvmConfig, EvmError, EvmProvider, Mempool, TransactionBuilder,
    TransactionPriority as BeechainPriority,
};

pub const BEECHAIN_MAINNET_CHAIN_ID: u64 = 3188;
pub const BEECHAIN_TESTNET_CHAIN_ID: u64 = 30188;

/// Block time in seconds (0.4s = 400ms)
pub const BLOCK_TIME_SECONDS: f64 = 0.4;

/// Target TPS (transactions per second)
pub const TARGET_TPS: u32 = 10_000;

/// Finality time in seconds (~0.8s = 2 blocks)
pub const FINALITY_TIME_SECONDS: f64 = 0.8;

/// Confirmation blocks for finality
pub const FINALITY_CONFIRMATION_BLOCKS: u64 = 2;

/// Safe confirmation blocks
pub const SAFE_CONFIRMATION_BLOCKS: u64 = 3;

/// Beechain Network Configuration
pub struct BeechainConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub chain_id: u64,
    pub confirmation_blocks: u64,
    pub gas_limit: u64,
    /// Block time in milliseconds
    pub block_time_ms: u64,
    /// Enable parallel execution optimizations
    pub parallel_execution: bool,
}

impl BeechainConfig {
    /// Beechain Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            rpc_url: "https://rpc.beechain.ai".to_string(),
            ws_url: Some("wss://rpc.beechain.ai/ws".to_string()),
            chain_id: BEECHAIN_MAINNET_CHAIN_ID,
            confirmation_blocks: FINALITY_CONFIRMATION_BLOCKS,
            gas_limit: 30_000_000,
            block_time_ms: 400,
            parallel_execution: true,
        }
    }

    /// Beechain Testnet configuration
    pub fn testnet() -> Self {
        Self {
            rpc_url: "https://testnet-rpc.beechain.ai".to_string(),
            ws_url: Some("wss://testnet-rpc.beechain.ai/ws".to_string()),
            chain_id: BEECHAIN_TESTNET_CHAIN_ID,
            confirmation_blocks: FINALITY_CONFIRMATION_BLOCKS,
            gas_limit: 30_000_000,
            block_time_ms: 400,
            parallel_execution: true,
        }
    }

    /// Local development configuration
    pub fn devnet() -> Self {
        Self {
            rpc_url: "http://localhost:8545".to_string(),
            ws_url: Some("ws://localhost:8546".to_string()),
            chain_id: 1337,
            confirmation_blocks: 1,
            gas_limit: 30_000_000,
            block_time_ms: 400,
            parallel_execution: false,
        }
    }

    /// Custom configuration with specific RPC endpoint
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            ws_url: None,
            chain_id,
            confirmation_blocks: FINALITY_CONFIRMATION_BLOCKS,
            gas_limit: 30_000_000,
            block_time_ms: 400,
            parallel_execution: true,
        }
    }

    /// Get estimated time to finality in seconds
    pub fn finality_time_seconds(&self) -> f64 {
        self.confirmation_blocks as f64 * (self.block_time_ms as f64 / 1000.0)
    }

    /// Get estimated TPS capacity
    pub fn tps_capacity(&self) -> u32 {
        if self.parallel_execution {
            TARGET_TPS
        } else {
            1000 // Conservative estimate without parallel execution
        }
    }
}

/// Beechain Client
pub use client::BeechainClient;

/// Beechain-specific error type
///
/// This is now a type alias to `EvmError` for consistency across chains.
/// Beechain-specific error variants are handled via the `Other` variant.
pub type BeechainError = super::common::EvmError;

/// Beechain-specific error helper functions
pub mod error_helpers {
    use super::super::common::EvmError;

    pub fn parallel_execution_unavailable() -> EvmError {
        EvmError::ProviderError("Parallel execution not available".into())
    }

    pub fn high_contention() -> EvmError {
        EvmError::ProviderError("High transaction contention detected".into())
    }

    pub fn block_time_deviation() -> EvmError {
        EvmError::ProviderError("Block time deviation detected, network may be congested".into())
    }
}

/// Beechain-specific constants
pub mod constants {
    // use super::*;

    /// Native token symbol
    pub const NATIVE_TOKEN: &str = "BKC";

    /// Native token decimals
    pub const NATIVE_TOKEN_DECIMALS: u8 = 18;

    /// Explorer URL
    pub const EXPLORER_URL: &str = "https://scan.beechain.ai";

    /// Block time in milliseconds
    pub const BLOCK_TIME_MS: u64 = 400;

    /// Blocks per second (2.5 blocks per second)
    pub const BLOCKS_PER_SECOND: f64 = 2.5;

    /// Blocks per minute
    pub const BLOCKS_PER_MINUTE: u64 = 150;

    /// Blocks per hour
    pub const BLOCKS_PER_HOUR: u64 = 9_000;

    /// Blocks per day
    pub const BLOCKS_PER_DAY: u64 = 216_000;

    /// Maximum gas per block
    pub const MAX_GAS_PER_BLOCK: u64 = 30_000_000;

    /// Target gas per second (with parallel execution)
    pub const TARGET_GAS_PER_SECOND: u64 = 75_000_000; // 30M * 2.5

    /// Minimum gas price in gwei
    pub const MIN_GAS_PRICE_GWEI: u64 = 1;

    /// Default gas price in gwei
    pub const DEFAULT_GAS_PRICE_GWEI: u64 = 5;
}

/// Beechain network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeechainNetwork {
    Mainnet,
    Testnet,
    Devnet,
}

impl BeechainNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            BeechainNetwork::Mainnet => BEECHAIN_MAINNET_CHAIN_ID,
            BeechainNetwork::Testnet => BEECHAIN_TESTNET_CHAIN_ID,
            BeechainNetwork::Devnet => 1337,
        }
    }

    pub fn config(&self) -> BeechainConfig {
        match self {
            BeechainNetwork::Mainnet => BeechainConfig::mainnet(),
            BeechainNetwork::Testnet => BeechainConfig::testnet(),
            BeechainNetwork::Devnet => BeechainConfig::devnet(),
        }
    }

    pub fn explorer_url(&self) -> &'static str {
        match self {
            BeechainNetwork::Mainnet => constants::EXPLORER_URL,
            BeechainNetwork::Testnet => "https://testnet-scan.beechain.ai",
            BeechainNetwork::Devnet => "http://localhost:4000",
        }
    }
}

/// Transaction status with parallel execution info
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Pending,
    InParallelQueue,
    Executing,
    Confirmed,
    Finalized,
    Failed,
}

/// Performance metrics for Beechain
#[derive(Debug, Clone, Default)]
pub struct BeechainMetrics {
    /// Current TPS
    pub current_tps: f64,
    /// Average block time (ms)
    pub avg_block_time_ms: f64,
    /// Number of parallel execution threads
    pub parallel_threads: u32,
    /// Queue depth
    pub queue_depth: u64,
    /// Network utilization (0.0 - 1.0)
    pub utilization: f64,
}
