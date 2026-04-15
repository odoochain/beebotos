//! Chain Module Constants
//!
//! Centralized constants for the beebotos-chain module.
//! All magic numbers should be defined here and imported where needed.

use std::time::Duration;

// ============================================================================
// Gas Limits
// ============================================================================

/// Default gas limit for transactions
pub const DEFAULT_GAS_LIMIT: u64 = 30_000_000;

/// Minimum gas limit (ETH transfer)
pub const MIN_GAS_LIMIT: u64 = 21_000;

/// Maximum gas limit
pub const MAX_GAS_LIMIT: u64 = 30_000_000;

/// Standard ERC20 transfer gas limit
pub const ERC20_TRANSFER_GAS: u64 = 65_000;

/// Standard ERC20 approve gas limit
pub const ERC20_APPROVE_GAS: u64 = 55_000;

/// ETH/native token transfer gas limit
pub const NATIVE_TRANSFER_GAS: u64 = 21_000;

/// Contract deployment base gas
pub const CONTRACT_DEPLOY_BASE_GAS: u64 = 100_000;

/// Multicall base gas
pub const MULTICALL_BASE_GAS: u64 = 50_000;

// ============================================================================
// Gas Prices (in wei)
// ============================================================================

/// Default gas price (5 gwei)
pub const DEFAULT_GAS_PRICE_WEI: u128 = 5_000_000_000;

/// Maximum acceptable gas price (1000 gwei)
pub const MAX_GAS_PRICE_WEI: u128 = 1_000_000_000_000;

/// High gas price threshold (500 gwei)
pub const HIGH_GAS_PRICE_WEI: u128 = 500_000_000_000;

/// One Gwei in wei
pub const ONE_GWEI: u128 = 1_000_000_000;

// ============================================================================
// Confirmation Blocks
// ============================================================================

/// Default confirmation blocks for mainnet
pub const DEFAULT_CONFIRMATION_BLOCKS: u64 = 12;

/// Fast confirmation blocks (for testnets)
pub const FAST_CONFIRMATION_BLOCKS: u64 = 1;

/// No confirmation (local devnet)
pub const NO_CONFIRMATION_BLOCKS: u64 = 0;

// ============================================================================
// Block & Time Constants
// ============================================================================

/// Average block time in seconds (Ethereum mainnet)
pub const AVERAGE_BLOCK_TIME_SECS: u64 = 12;

/// Maximum block age for health check (seconds)
pub const MAX_BLOCK_AGE_SECS: u64 = 60;

/// Standard voting period (~1 week in blocks)
pub const STANDARD_VOTING_PERIOD_BLOCKS: u64 = 40_320;

/// Fast track voting period (~1 day in blocks)
pub const FAST_TRACK_VOTING_PERIOD_BLOCKS: u64 = 5_760;

/// Emergency voting period (~15 minutes in blocks)
pub const EMERGENCY_VOTING_PERIOD_BLOCKS: u64 = 60;

// ============================================================================
// Token Decimals
// ============================================================================

/// Standard ERC20 decimals
pub const STANDARD_TOKEN_DECIMALS: u8 = 18;

/// Wei per ETH (10^18)
pub const WEI_PER_ETH: u128 = 1_000_000_000_000_000_000;

/// Gwei per ETH (10^9)
pub const GWEI_PER_ETH: u128 = 1_000_000_000;

// ============================================================================
// Address & Data Limits
// ============================================================================

/// Ethereum address length in characters (with 0x prefix)
pub const ETH_ADDRESS_LENGTH: usize = 42;

/// Ethereum address length in bytes (without 0x prefix)
pub const ETH_ADDRESS_BYTES: usize = 20;

/// Maximum transaction data size (100KB)
pub const MAX_TX_DATA_SIZE: usize = 100_000;

/// Zero address
pub const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

// ============================================================================
// Wallet & Cryptography
// ============================================================================

/// Default BIP39 word count
pub const DEFAULT_MNEMONIC_WORD_COUNT: usize = 12;

/// Maximum BIP39 word count
pub const MAX_MNEMONIC_WORD_COUNT: usize = 24;

/// PBKDF2 iterations for key derivation
pub const PBKDF2_ITERATIONS: u32 = 100_000;

/// AES-GCM nonce size (12 bytes)
pub const AES_GCM_NONCE_SIZE: usize = 12;

/// AES-GCM salt size (16 bytes)
pub const AES_GCM_SALT_SIZE: usize = 16;

/// Standard Ethereum derivation path prefix (BIP44)
pub const ETHEREUM_DERIVATION_PREFIX: &str = "m/44'/60'/0'/0";

// ============================================================================
// Cache TTL (in seconds)
// ============================================================================

/// Default cache TTL
pub const DEFAULT_CACHE_TTL_SECS: u64 = 3600; // 1 hour

/// Contract cache TTL
pub const CONTRACT_CACHE_TTL_SECS: u64 = 3600; // 1 hour

/// Identity cache TTL
pub const IDENTITY_CACHE_TTL_SECS: u64 = 300; // 5 minutes

/// Block cache TTL (no expiration)
pub const BLOCK_CACHE_TTL_SECS: Option<u64> = None;

/// Cache auto-save interval
pub const CACHE_SAVE_INTERVAL_SECS: u64 = 60;

// ============================================================================
// Retry & Circuit Breaker
// ============================================================================

/// Default max retry attempts
pub const DEFAULT_MAX_RETRIES: u32 = 5;

/// Aggressive retry max attempts
pub const AGGRESSIVE_MAX_RETRIES: u32 = 10;

/// Conservative retry max attempts
pub const CONSERVATIVE_MAX_RETRIES: u32 = 3;

/// Default retry initial interval (ms)
pub const DEFAULT_RETRY_INITIAL_MS: u64 = 100;

/// Default retry max interval (seconds)
pub const DEFAULT_RETRY_MAX_INTERVAL_SECS: u64 = 30;

/// Circuit breaker failure threshold
pub const CIRCUIT_BREAKER_FAILURE_THRESHOLD: u32 = 3;

/// Circuit breaker success threshold
pub const CIRCUIT_BREAKER_SUCCESS_THRESHOLD: u32 = 2;

/// Circuit breaker timeout (seconds)
pub const CIRCUIT_BREAKER_TIMEOUT_SECS: u64 = 60;

// ============================================================================
// Rate Limiting
// ============================================================================

/// Default max calls per second
pub const DEFAULT_MAX_CALLS_PER_SEC: u32 = 10;

/// Default rate limit window (seconds)
pub const DEFAULT_RATE_LIMIT_WINDOW_SECS: u64 = 60;

// ============================================================================
// Health Check
// ============================================================================

/// Health check RPC timeout
pub const HEALTH_CHECK_RPC_TIMEOUT: Duration = Duration::from_secs(5);

/// Health check interval (seconds)
pub const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// Max sync lag blocks
pub const MAX_SYNC_LAG_BLOCKS: u64 = 10;

// ============================================================================
// Chain IDs
// ============================================================================

/// Ethereum Mainnet chain ID
pub const CHAIN_ID_ETHEREUM: u64 = 1;

/// BSC Mainnet chain ID
pub const CHAIN_ID_BSC: u64 = 56;

/// BSC Testnet chain ID
pub const CHAIN_ID_BSC_TESTNET: u64 = 97;

/// Beechain chain ID
pub const CHAIN_ID_BEECHAIN: u64 = 3188;

/// Monad Mainnet chain ID
pub const CHAIN_ID_MONAD: u64 = 1_014_301;

/// Monad Testnet chain ID
pub const CHAIN_ID_MONAD_TESTNET: u64 = 10_143;

/// Local devnet chain ID
pub const CHAIN_ID_LOCAL: u64 = 1337;

// ============================================================================
// EIP-1559 Constants
// ============================================================================

/// EIP-1559 base fee max change denominator
pub const EIP1559_BASE_FEE_CHANGE_DENOMINATOR: u64 = 8;

/// EIP-1559 elasticity multiplier
pub const EIP1559_ELASTICITY_MULTIPLIER: u64 = 2;

/// Default max fee per gas (100 gwei)
pub const DEFAULT_MAX_FEE_PER_GAS: u128 = 100_000_000_000;

/// Default max priority fee per gas (2 gwei)
pub const DEFAULT_MAX_PRIORITY_FEE_PER_GAS: u128 = 2_000_000_000;

// ============================================================================
// Proposal Types
// ============================================================================

/// Standard proposal voting period in blocks (~7 days)
pub const PROPOSAL_STANDARD_VOTING_PERIOD: u64 = 40_320;

/// Fast track proposal voting period in blocks (~1 day)
pub const PROPOSAL_FAST_TRACK_VOTING_PERIOD: u64 = 5_760;

/// Emergency proposal voting period in blocks (~15 minutes)
pub const PROPOSAL_EMERGENCY_VOTING_PERIOD: u64 = 60;

// ============================================================================
// HTTP & Network
// ============================================================================

/// Default RPC timeout (seconds)
pub const DEFAULT_RPC_TIMEOUT_SECS: u64 = 30;

/// Default WebSocket timeout (seconds)
pub const DEFAULT_WS_TIMEOUT_SECS: u64 = 60;

/// Maximum RPC batch size
pub const MAX_RPC_BATCH_SIZE: usize = 100;

// ============================================================================
// Numeric Conversions
// ============================================================================

/// One ether in wei as U256-compatible u64
pub const ONE_ETHER_U64: u64 = 1_000_000_000_000_000_000;

/// One gwei in wei as u64
pub const ONE_GWEI_U64: u64 = 1_000_000_000;

/// Default cache capacity
pub const DEFAULT_CACHE_CAPACITY: usize = 100;

/// Max cache history size
pub const MAX_CACHE_HISTORY: usize = 10;
