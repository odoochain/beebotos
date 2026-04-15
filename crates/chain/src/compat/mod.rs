//! Alloy Types Module
//!
//! Re-exports Alloy primitives as the primary types for the chain layer.
//! 
//! 🟡 P1 FIX: Now uses unified types from beebotos_core for consistency.

pub mod alloy_adapter;
pub mod client_trait;
pub mod provider;
pub mod retry;

// Re-export client trait types for dynamic dispatch
pub use client_trait::{
    AgentIdentityInfo, ChainClientTrait, ContractCall, HealthStatus, LogEntry, SyncStatus, 
    TransactionReceipt, TransactionRequest, ProposalInfo, ProposalState, ChainClientError
};
pub use alloy_adapter::{AlloyChainClient, create_chain_client};

pub use provider::{AlloyClient, Provider};
pub use retry::{with_retry, with_retry_and_handler, CircuitBreaker, RateLimiter, RetryConfig};

// Re-export Alloy provider trait and types for convenience
pub use alloy_provider::Provider as AlloyProvider;
pub use alloy_provider::ReqwestProvider;

// 🟡 P1 FIX: Re-export unified types from core for consistency
pub use beebotos_core::{Address, Bytes, B256, U256, TxHash, BlockNumber, Gas, Wei};
// ChainId is defined as enum in core, re-export it separately
pub use beebotos_core::ChainId;
/// Numeric chain ID type for low-level operations
pub type ChainIdNum = u64;

// Re-export FixedBytes for compatibility
pub use alloy_primitives::FixedBytes;
