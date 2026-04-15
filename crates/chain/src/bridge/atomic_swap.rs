//! Atomic Swap Implementation
//!
//! DEPRECATED: This module is kept for backward compatibility.
//! Use `crate::bridge::client::{HTLC, AtomicSwapClient}` instead.

// Re-export the canonical implementation from client module
pub use super::client::{AtomicSwapClient, HTLC};
