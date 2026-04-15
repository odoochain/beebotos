//! Ethereum Bridge Adapter

use crate::compat::Address;

/// Ethereum bridge adapter
pub struct EthereumBridgeAdapter {
    #[allow(dead_code)]
    bridge_contract: Address,
}

impl EthereumBridgeAdapter {
    pub fn new(bridge_contract: Address) -> Self {
        Self { bridge_contract }
    }
}
