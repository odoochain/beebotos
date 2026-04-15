//! DAO Interaction Example
//!
//! Demonstrates interacting with the BeeBotOS DAO using Alloy.

use alloy_primitives::{Address, U256};
use beebotos_chain::dao::*;
use beebotos_chain::contracts::ContractClient;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration
    let dao_address: Address = "0x1234567890123456789012345678901234567890".parse()?;
    let rpc_url = "https://rpc.testnet.monad.xyz";
    
    // Create contract client
    let client = ContractClient::new(rpc_url)?;
    println!("✅ Connected to network at {}", rpc_url);
    
    // Get contract instances
    let dao_contract = client.agent_dao(dao_address);
    println!("✅ DAO contract instance created at {}", dao_address);
    
    // Example: Query proposal count (read-only)
    // Note: This is a placeholder showing the pattern
    // In a real implementation, you would call the contract method
    println!("\n📋 DAO Information:");
    println!("  Address: {}", dao_address);
    
    // Create a proposal builder
    let proposal = client::ProposalBuilder::new("Increase Staking Rewards")
        .proposal_type(client::ProposalType::Standard)
        .add_transfer(
            "0x1234567890123456789012345678901234567890".parse()?,
            U256::from(1000u64),
        );
    
    let (targets, values, calldatas, description) = proposal.build();
    
    println!("\n📝 Proposal Structure:");
    println!("  Description: {}", description);
    println!("  Targets: {:?}", targets.len());
    println!("  Values: {:?}", values.len());
    println!("  Calldatas: {:?}", calldatas.len());
    
    // Example vote
    let vote = VoteType::For;
    println!("\n🗳️  Example Vote: {:?}", vote);
    
    // Example proposal data
    let proposal_data = Proposal {
        id: 1,
        proposer: dao_address,
        description: "Test Proposal".to_string(),
        for_votes: U256::from(10000u64),
        against_votes: U256::from(2000u64),
        abstain_votes: U256::from(500u64),
        executed: false,
        eta: 1700000000,
    };
    
    println!("\n📊 Proposal Status:");
    println!("  For: {} votes", proposal_data.for_votes);
    println!("  Against: {} votes", proposal_data.against_votes);
    println!("  Abstain: {} votes", proposal_data.abstain_votes);
    
    // Calculate support percentage
    let total_votes = proposal_data.for_votes + proposal_data.against_votes + proposal_data.abstain_votes;
    if total_votes > U256::ZERO {
        let for_u128: u128 = proposal_data.for_votes.try_into().unwrap_or(u128::MAX);
        let total_u128: u128 = total_votes.try_into().unwrap_or(u128::MAX);
        let support_pct = (for_u128 as f64 / total_u128 as f64) * 100.0;
        println!("  Support: {:.2}%", support_pct);
    }
    
    println!("\n✅ DAO interaction example complete!");
    
    Ok(())
}
