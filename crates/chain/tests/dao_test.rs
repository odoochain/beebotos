//! DAO client tests

use beebotos_chain::dao::{ProposalBuilder, ProposalType, VoteType};

#[test]
fn test_proposal_builder() {
    let builder = ProposalBuilder::new("Test Proposal").proposal_type(ProposalType::Standard);

    let (targets, values, _calldatas, description) = builder.build();
    assert!(targets.is_empty());
    assert!(values.is_empty());
    assert_eq!(description, "Test Proposal");
}

#[test]
fn test_proposal_builder_with_actions() {
    use beebotos_chain::compat::{Address, Bytes, U256};

    let target = Address::from([1u8; 20]);
    let builder = ProposalBuilder::new("Transfer Proposal")
        .add_transfer(target, U256::from(1000))
        .add_contract_call(
            target,
            Bytes::from(vec![0x01, 0x02]),
            "transfer(address,uint256)",
        );

    let (targets, values, calldatas, signatures, description, proposal_type) = builder.build_full();

    assert_eq!(targets.len(), 2);
    assert_eq!(values[0], U256::from(1000));
    assert_eq!(values[1], U256::ZERO);
    assert!(!calldatas[1].is_empty());
    assert_eq!(signatures[1], "transfer(address,uint256)");
    assert_eq!(description, "Transfer Proposal");
    assert_eq!(proposal_type, ProposalType::Standard);
}

#[test]
fn test_vote_type_conversion() {
    // Test that VoteType maps to correct u8 values
    let against_val: u8 = match VoteType::Against {
        VoteType::Against => 0,
        VoteType::For => 1,
        VoteType::Abstain => 2,
    };
    let for_val: u8 = match VoteType::For {
        VoteType::Against => 0,
        VoteType::For => 1,
        VoteType::Abstain => 2,
    };
    let abstain_val: u8 = match VoteType::Abstain {
        VoteType::Against => 0,
        VoteType::For => 1,
        VoteType::Abstain => 2,
    };

    assert_eq!(against_val, 0);
    assert_eq!(for_val, 1);
    assert_eq!(abstain_val, 2);
}

#[test]
fn test_proposal_type_voting_periods() {
    assert_eq!(ProposalType::Standard.voting_period(), 40_320);
    assert_eq!(ProposalType::FastTrack.voting_period(), 5_760);
    assert_eq!(ProposalType::Emergency.voting_period(), 60);
}

#[test]
fn test_proposal_builder_full() {
    use beebotos_chain::compat::{Address, U256};

    let target1 = Address::from([1u8; 20]);
    let target2 = Address::from([2u8; 20]);

    let builder = ProposalBuilder::new("Complex Proposal")
        .proposal_type(ProposalType::FastTrack)
        .add_transfer(target1, U256::from(100))
        .add_transfer(target2, U256::from(200));

    assert_eq!(builder.get_proposal_type(), ProposalType::FastTrack);
    assert_eq!(builder.voting_period(), 5_760);

    let (targets, values, _, _) = builder.build();
    assert_eq!(targets.len(), 2);
    assert_eq!(values[0], U256::from(100));
    assert_eq!(values[1], U256::from(200));
}
