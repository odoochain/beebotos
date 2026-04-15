//! Polygon PoS Module
//!
//! Polygon PoS is an EVM-compatible sidechain for Ethereum.
//! - Chain ID: 137 (Mainnet), 80001 (Mumbai Testnet), 80002 (Amoy Testnet)
//! - Block Time: ~2 seconds
//! - Consensus: Proof of Stake (PoS) with Bor validators
//!
//! ## Official Resources
//! - RPC: https://polygon-rpc.com
//! - Explorer: https://polygonscan.com
//! - Native Token: MATIC

pub mod client;
pub mod types;

// Re-export common types for convenience
pub use crate::chains::common::{
    format_native_token, parse_native_token, ContractCall, ContractDeploy, ContractInstance,
    EventFilter, EvmConfig, EvmError, EvmProvider, Mempool, TransactionBuilder,
    TransactionPriority as PolygonPriority,
};

pub const POLYGON_MAINNET_CHAIN_ID: u64 = 137;
pub const POLYGON_MUMBAI_CHAIN_ID: u64 = 80001;
pub const POLYGON_AMOY_CHAIN_ID: u64 = 80002;

/// Polygon Network Configuration
pub struct PolygonConfig {
    pub rpc_url: String,
    pub ws_url: Option<String>,
    pub chain_id: u64,
    pub confirmation_blocks: u64,
    pub gas_limit: u64,
    /// Whether to use EIP-1559 transactions
    pub use_eip1559: bool,
}

impl PolygonConfig {
    /// Polygon Mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            rpc_url: "https://polygon-rpc.com".to_string(),
            ws_url: Some("wss://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY".to_string()),
            chain_id: POLYGON_MAINNET_CHAIN_ID,
            confirmation_blocks: 20, // ~40 seconds
            gas_limit: 30_000_000,
            use_eip1559: true,
        }
    }

    /// Polygon Mumbai Testnet configuration (deprecated)
    pub fn mumbai() -> Self {
        Self {
            rpc_url: "https://rpc-mumbai.maticvigil.com".to_string(),
            ws_url: Some("wss://polygon-mumbai.g.alchemy.com/v2/YOUR_API_KEY".to_string()),
            chain_id: POLYGON_MUMBAI_CHAIN_ID,
            confirmation_blocks: 20,
            gas_limit: 30_000_000,
            use_eip1559: true,
        }
    }

    /// Polygon Amoy Testnet configuration (new testnet)
    pub fn amoy() -> Self {
        Self {
            rpc_url: "https://rpc-amoy.polygon.technology".to_string(),
            ws_url: Some("wss://polygon-amoy.g.alchemy.com/v2/YOUR_API_KEY".to_string()),
            chain_id: POLYGON_AMOY_CHAIN_ID,
            confirmation_blocks: 20,
            gas_limit: 30_000_000,
            use_eip1559: true,
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
        }
    }

    /// Custom configuration with specific RPC endpoint
    pub fn custom(rpc_url: &str, chain_id: u64) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            ws_url: None,
            chain_id,
            confirmation_blocks: 20,
            gas_limit: 30_000_000,
            use_eip1559: true,
        }
    }
}

/// Polygon Client
pub use client::PolygonClient;

/// Polygon-specific error type
pub type PolygonError = super::common::EvmError;

/// Polygon-specific constants
pub mod constants {
    /// Average block time in seconds
    pub const BLOCK_TIME_SECONDS: u64 = 2;

    /// Recommended confirmation blocks for safe transactions
    pub const SAFE_CONFIRMATION_BLOCKS: u64 = 20;

    /// Fast confirmation blocks (less safe)
    pub const FAST_CONFIRMATION_BLOCKS: u64 = 10;

    /// Native token symbol
    pub const NATIVE_TOKEN: &str = "MATIC";

    /// Native token decimals
    pub const NATIVE_TOKEN_DECIMALS: u8 = 18;

    /// Default gas limit for standard transactions
    pub const DEFAULT_GAS_LIMIT: u64 = 21000;

    /// Maximum gas limit per block
    pub const MAX_GAS_LIMIT_PER_BLOCK: u64 = 30_000_000;

    /// Root chain (Ethereum) block time
    pub const ROOT_CHAIN_BLOCK_TIME: u64 = 12;
}

/// Polygon network type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolygonNetwork {
    Mainnet,
    Mumbai,
    Amoy,
    Devnet,
}

impl PolygonNetwork {
    pub fn chain_id(&self) -> u64 {
        match self {
            PolygonNetwork::Mainnet => POLYGON_MAINNET_CHAIN_ID,
            PolygonNetwork::Mumbai => POLYGON_MUMBAI_CHAIN_ID,
            PolygonNetwork::Amoy => POLYGON_AMOY_CHAIN_ID,
            PolygonNetwork::Devnet => 1337,
        }
    }

    pub fn config(&self) -> PolygonConfig {
        match self {
            PolygonNetwork::Mainnet => PolygonConfig::mainnet(),
            PolygonNetwork::Mumbai => PolygonConfig::mumbai(),
            PolygonNetwork::Amoy => PolygonConfig::amoy(),
            PolygonNetwork::Devnet => PolygonConfig::devnet(),
        }
    }

    pub fn explorer_url(&self) -> &'static str {
        match self {
            PolygonNetwork::Mainnet => "https://polygonscan.com",
            PolygonNetwork::Mumbai => "https://mumbai.polygonscan.com",
            PolygonNetwork::Amoy => "https://amoy.polygonscan.com",
            PolygonNetwork::Devnet => "http://localhost:4000",
        }
    }
}
