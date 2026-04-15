//! DAO Voting Module

use crate::compat::{Address, U256};
use crate::dao::VoteType;

/// Vote record
#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: Address,
    pub proposal_id: u64,
    pub vote_type: VoteType,
    pub weight: U256,
}

/// Voting power snapshot
#[derive(Debug, Clone)]
pub struct VotingSnapshot {
    pub block_number: u64,
    pub voting_power: U256,
}

/// Vote counter
pub struct VoteCounter {
    pub for_votes: U256,
    pub against_votes: U256,
    pub abstain_votes: U256,
}

impl VoteCounter {
    pub fn new() -> Self {
        Self {
            for_votes: U256::ZERO,
            against_votes: U256::ZERO,
            abstain_votes: U256::ZERO,
        }
    }

    pub fn add_vote(&mut self, vote_type: VoteType, weight: U256) {
        match vote_type {
            VoteType::For => self.for_votes += weight,
            VoteType::Against => self.against_votes += weight,
            VoteType::Abstain => self.abstain_votes += weight,
        }
    }

    pub fn has_quorum(&self, total_supply: U256, quorum_basis_points: u16) -> bool {
        let total_votes = self.for_votes + self.against_votes + self.abstain_votes;
        let quorum_needed = (total_supply * U256::from(quorum_basis_points)) / U256::from(10000);
        total_votes >= quorum_needed
    }

    pub fn is_passed(&self) -> bool {
        self.for_votes > self.against_votes
    }
}

impl Default for VoteCounter {
    fn default() -> Self {
        Self::new()
    }
}
