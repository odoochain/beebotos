//! Cross-chain Router Implementation

use crate::compat::Address;

/// Bridge route
#[derive(Debug, Clone)]
pub struct BridgeRoute {
    pub from_chain: u64,
    pub to_chain: u64,
    pub bridge_address: Address,
    pub estimated_time: u64,
    pub fee_basis_points: u16,
}

/// Route finder
pub struct RouteFinder {
    routes: Vec<BridgeRoute>,
}

impl RouteFinder {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route(&mut self, route: BridgeRoute) {
        self.routes.push(route);
    }

    pub fn find_route(&self, from_chain: u64, to_chain: u64) -> Option<&BridgeRoute> {
        self.routes
            .iter()
            .find(|r| r.from_chain == from_chain && r.to_chain == to_chain)
    }
}

impl Default for RouteFinder {
    fn default() -> Self {
        Self::new()
    }
}
