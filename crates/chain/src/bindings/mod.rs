//! Contract Bindings
//!
//! Alloy-based contract bindings using sol! macro.
//!
//! Note: All BeeBotOS contract bindings are now in `contracts::bindings`.
//! This module re-exports from the new location for backward compatibility.

pub use crate::contracts::bindings::{
    A2ACommerce, AgentDAO, AgentIdentity, AgentMetadata, AgentPayment, AgentRegistry, BeeToken,
    CrossChainBridge, DealEscrow, DisputeResolution, DisputeStatus, PaymentMandate,
    ReputationSystem, Resolution, SkillNFT, Stream, TreasuryManager,
};

// Re-export AgentIdentityInfo from AgentIdentity contract
pub use crate::contracts::bindings::AgentIdentity::AgentIdentityInfo;
