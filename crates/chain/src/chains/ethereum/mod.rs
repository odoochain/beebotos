//! Ethereum Module
//!
//! Ethereum is the leading smart contract platform.
//! - Chain ID: 1 (Mainnet), 11155111 (Sepolia Testnet), 17000 (Holesky Testnet)
//! - Block Time: ~12 seconds (post-Merge)
//! - Consensus: Proof of Stake (PoS)
//! - Finality: Single slot finality (~12.8 minutes for absolute finality)

pub mod client;
pub mod types;

// Re-export common types for convenience
pub use crate::chains::common::{
    format_native_token, parse_native_token, ContractCall, ContractDeploy, ContractInstance,
    EventFilter, EvmConfig, EvmError, EvmProvider, Mempool, TransactionBuilder,
    TransactionPriority,
};

pub const ETHEREUM_MAINNET_CHAIN_ID: u64 = 1;
pub const ETHEREUM_SEPOLIA_CHAIN_ID: u64 = 11_155_111;
pub const ETHEREUM_HOLESKY_CHAIN_ID: u64 = 17_000;
pub const ETHEREUM_KOVAN_CHAIN_ID: u64 = 42; // Deprecated

/// Ethereum Network Configuration
pub struct EthereumConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub chain_id: u64,
    pub confirmation_blocks: u64,
    pub gas_limit: u64,
    /// Whether to use EIP-1559 transactions
    pub use_eip1559: bool,
    /// Max fee per gas for EIP-1559 (in gwei)
    pub max_fee_per_gas_gwei: u64,
    /// Max priority fee per gas for EIP-1559 (in gwei)
    pub max_priority_fee_per_gas_gwei: u64,
}

impl EthereumConfig {
    /// Ethereum Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            rpc_url: "https://eth.llamarpc.com".to_string(),
            ws_url: Some("wss://ethereum.publicnode.com".to_string()),
            chain_id: ETHEREUM_MAINNET_CHAIN_ID,
            confirmation_blocks: 12, // ~2.4 minutes for safe
            gas_limit: 30_000_000,
            use_eip1559: true,
            max_fee_per_gas_gwei: 100,
            max_priority_fee_per_gas_gwei: 2,
        }
    }

    /// Ethereum Sepolia Testnet configuration
    pub fn sepolia() -> Self {
        Self {
            rpc_url: "https://rpc.sepolia.org".to_string(),
            ws_url: Some("wss://ethereum-sepolia.publicnode.com".to_string()),
            chain_id: ETHEREUM_SEPOLIA_CHAIN_ID,
            confirmation_blocks: 12,
            gas_limit: 30_000_000,
            use_eip1559: true,
            max_fee_per_gas_gwei: 50,
            max_priority_fee_per_gas_gwei: 1,
        }
    }

    /// Ethereum Holesky Testnet configuration
    pub fn holesky() -> Self {
        Self {
            rpc_url: "https://ethereum-holesky.publicnode.com".to_string(),
            ws_url: Some("wss://ethereum-holesky.publicnode.com".to_string()),
            chain_id: ETHEREUM_HOLESKY_CHAIN_ID,
            confirmation_blocks: 12,
            gas_limit: 30_000_000,
            use_eip1559: true,
            max_fee_per_gas_gwei: 50,
            max_priority_fee_per_gas_gwei: 1,
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
            use_eip1559: false,
            max_fee_per_gas_gwei: 10,
            max_priority_fee_per_gas_gwei: 1,
        }
    }

    /// Custom configuration with specific RPC endpoint
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            ws_url: None,
            chain_id,
            confirmation_blocks: 12,
            gas_limit: 30_000_000,
            use_eip1559: true,
            max_fee_per_gas_gwei: 100,
            max_priority_fee_per_gas_gwei: 2,
        }
    }
}

/// Ethereum Client
pub use client::EthereumClient;

/// Ethereum-specific error type
///
/// This is now a type alias to `EvmError` for consistency across chains.
/// Use `EvmError` directly for new code.
pub type EthereumError = super::common::EvmError;

/// Ethereum-specific error variants (for backward compatibility)
pub mod error_variants {
    use super::super::common::EvmError;

    pub fn provider_error(msg: impl Into<String>) -> EvmError {
        EvmError::ProviderError(msg.into())
    }

    pub fn contract_error(msg: impl Into<String>) -> EvmError {
        EvmError::ContractError(msg.into())
    }

    pub fn transaction_error(msg: impl Into<String>) -> EvmError {
        EvmError::TransactionError(msg.into())
    }
}

/// Ethereum-specific constants
pub mod constants {
    /// Average block time in seconds (post-Merge)
    pub const BLOCK_TIME_SECONDS: u64 = 12;

    /// Slots per epoch
    pub const SLOTS_PER_EPOCH: u64 = 32;

    /// Time to finality in epochs (2 epochs for finality)
    pub const EPOCHS_TO_FINALITY: u64 = 2;

    /// Safe confirmation blocks (2 epochs)
    pub const SAFE_CONFIRMATION_BLOCKS: u64 = 64;

    /// Finalized confirmation blocks
    pub const FINALIZED_CONFIRMATION_BLOCKS: u64 = 64;

    /// Native token symbol
    pub const NATIVE_TOKEN: &str = "ETH";

    /// Native token decimals
    pub const NATIVE_TOKEN_DECIMALS: u8 = 18;

    /// Default gas limit for standard transactions
    pub const DEFAULT_GAS_LIMIT: u64 = 21000;

    /// Maximum gas limit per block
    pub const MAX_GAS_LIMIT_PER_BLOCK: u64 = 30_000_000;
}

/// Ethereum network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EthereumNetwork {
    Mainnet,
    Sepolia,
    Holesky,
    Devnet,
}

impl EthereumNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            EthereumNetwork::Mainnet => ETHEREUM_MAINNET_CHAIN_ID,
            EthereumNetwork::Sepolia => ETHEREUM_SEPOLIA_CHAIN_ID,
            EthereumNetwork::Holesky => ETHEREUM_HOLESKY_CHAIN_ID,
            EthereumNetwork::Devnet => 1337,
        }
    }

    pub fn config(&self) -> EthereumConfig {
        match self {
            EthereumNetwork::Mainnet => EthereumConfig::mainnet(),
            EthereumNetwork::Sepolia => EthereumConfig::sepolia(),
            EthereumNetwork::Holesky => EthereumConfig::holesky(),
            EthereumNetwork::Devnet => EthereumConfig::devnet(),
        }
    }
}
