//! Vote command for BeeBotOS CLI

use crate::config::Config;

/// Vote subcommand
#[derive(clap::Args)]
pub struct VoteArgs {
    /// Proposal ID
    #[arg(value_parser)]
    pub proposal_id: u64,

    /// Vote choice
    #[arg(value_enum)]
    pub choice: VoteChoice,

    /// Reason for vote
    #[arg(short, long)]
    pub reason: Option<String>,

    /// Delegate voting power first
    #[arg(short, long)]
    pub delegate: Option<String>,
}

/// Vote choice
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum VoteChoice {
    /// Vote for proposal
    For,
    /// Vote against proposal
    Against,
    /// Abstain from voting
    Abstain,
}

/// Run vote command
pub async fn run(args: VoteArgs, _config: Config) -> anyhow::Result<()> {
    println!("🗳️  Casting vote...");
    println!("Proposal ID: {}", args.proposal_id);
    println!("Choice: {:?}", args.choice);

    if let Some(reason) = args.reason {
        println!("Reason: {}", reason);
    }

    if let Some(delegate) = args.delegate {
        println!("Delegating to: {}", delegate);
    }

    println!("\n⚠️  Chain operations are not fully implemented yet.");
    println!("   Enable the 'chain' feature for full blockchain support.");

    Ok(())
}
