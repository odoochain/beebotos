//! DAO integration tests

use alloy_primitives::{Address, U256};
use beebotos_chain::dao::*;

#[test]
fn test_proposal_creation() {
    let proposal = Proposal {
        id: 1,
        proposer: Address::ZERO,
        description: "Test Proposal".to_string(),
        for_votes: U256::ZERO,
        against_votes: U256::ZERO,
        abstain_votes: U256::ZERO,
        executed: false,
        eta: 1700000000,
    };
    
    assert_eq!(proposal.id, 1);
    assert_eq!(proposal.description, "Test Proposal");
    assert!(!proposal.executed);
}

#[test]
fn test_vote_type_values() {
    let for_vote = VoteType::For;
    let against_vote = VoteType::Against;
    let abstain_vote = VoteType::Abstain;
    
    // Verify vote types can be created and matched
    assert!(matches!(for_vote, VoteType::For));
    assert!(matches!(against_vote, VoteType::Against));
    assert!(matches!(abstain_vote, VoteType::Abstain));
}

#[test]
fn test_governance_params_default() {
    let params = GovernanceParams::default();
    
    assert_eq!(params.voting_delay, 1);
    assert_eq!(params.voting_period, 40320);
    assert_eq!(params.quorum_numerator, 400);
    assert_eq!(params.reputation_weight_bps, 3000);
}

#[test]
fn test_vote_counter() {
    use beebotos_chain::dao::voting::VoteCounter;
    
    let mut counter = VoteCounter::new();
    
    // Add votes
    counter.add_vote(VoteType::For, U256::from(1000u64));
    counter.add_vote(VoteType::Against, U256::from(500u64));
    counter.add_vote(VoteType::Abstain, U256::from(200u64));
    
    assert_eq!(counter.for_votes, U256::from(1000u64));
    assert_eq!(counter.against_votes, U256::from(500u64));
    assert_eq!(counter.abstain_votes, U256::from(200u64));
    
    // Check quorum
    let total_supply = U256::from(10000u64);
    assert!(counter.has_quorum(total_supply, 1000)); // 10% quorum
}

#[test]
fn test_treasury_budget() {
    use beebotos_chain::dao::treasury::{Budget, TreasuryManager};
    
    let budget = Budget {
        id: 1,
        beneficiary: Address::ZERO,
        amount: U256::from(10000u64),
        token: Address::ZERO,
        start_time: 1000,
        end_time: 2000,
        released: false,
    };
    
    assert_eq!(budget.id, 1);
    assert_eq!(budget.amount, U256::from(10000u64));
    assert!(!budget.released);
}

#[test]
fn test_voting_power_calculation() {
    // Simple voting power calculation
    let token_balance = U256::from(1000u64) * U256::from(10u64.pow(18));
    let reputation_score = 8000u64;
    let reputation_weight_bps = 3000u16;
    
    // Calculate weighted power: tokens + (tokens * reputation / 10000 * weight / 10000)
    let reputation_bonus = (token_balance * U256::from(reputation_score) * U256::from(reputation_weight_bps)) 
        / U256::from(100_000_000u64);
    let power = token_balance + reputation_bonus;
    
    assert!(power > U256::ZERO);
    assert!(power >= token_balance); // Reputation adds to power
}
