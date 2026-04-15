//! DAO Governance Module

use crate::compat::U256;

/// Governance parameters
#[derive(Debug, Clone)]
pub struct GovernanceParams {
    pub voting_delay: u64,
    pub voting_period: u64,
    pub proposal_threshold: U256,
    pub quorum_numerator: u16,
    pub quorum_denominator: u16,
}

impl Default for GovernanceParams {
    fn default() -> Self {
        Self {
            voting_delay: 1,
            voting_period: 40320,                    // ~1 week at 15s/block
            proposal_threshold: U256::from(1000e18), // 1000 tokens
            quorum_numerator: 4,
            quorum_denominator: 100, // 4%
        }
    }
}

/// Governance config
pub struct GovernanceConfig {
    params: GovernanceParams,
}

impl GovernanceConfig {
    pub fn new(params: GovernanceParams) -> Self {
        Self { params }
    }

    pub fn params(&self) -> &GovernanceParams {
        &self.params
    }
}
