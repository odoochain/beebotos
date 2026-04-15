//! Query DAO state example

use alloy_primitives::{Address, U256};
use beebotos_chain::dao::{Proposal, VoteType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏛️ DAO Query Example\n");

    // Example DAO address (would be from env in production)
    let dao_address: Address = "0x1234567890123456789012345678901234567890".parse()?;

    println!("📋 DAO Information:");
    println!("  Address: {}", dao_address);

    // Example proposal data
    let proposal = Proposal {
        id: 1,
        proposer: "0x1234567890123456789012345678901234567890"
            .parse()
            .unwrap(),
        description: "Treasury Allocation Q2".to_string(),
        for_votes: U256::from(1000000u64),
        against_votes: U256::from(200000u64),
        abstain_votes: U256::from(50000u64),
        executed: false,
        eta: 1700000000,
    };

    println!("\n📜 Active Proposal:");
    println!("  #{}: {}", proposal.id, proposal.description);
    println!("  Proposer: {}", proposal.proposer);
    println!("  For: {} BEE", proposal.for_votes);
    println!("  Against: {} BEE", proposal.against_votes);
    println!("  Abstain: {} BEE", proposal.abstain_votes);

    // Calculate quorum
    let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
    let for_percentage = if total_votes > U256::ZERO {
        let for_votes_u128: u128 = proposal.for_votes.try_into().unwrap_or(u128::MAX);
        let total_u128: u128 = total_votes.try_into().unwrap_or(u128::MAX);
        for_votes_u128 as f64 / total_u128 as f64 * 100.0
    } else {
        0.0
    };

    println!("  Support: {:.1}%", for_percentage);

    // Example vote types
    let vote = VoteType::For;
    println!("\n🗳️  Example Vote: {:?}", vote);

    println!("\n✅ Query complete!");
    Ok(())
}
