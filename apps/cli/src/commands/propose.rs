//! Create proposal command

use crate::config::Config;

/// Propose subcommand
#[derive(clap::Args)]
pub struct ProposeArgs {
    /// Proposal type
    #[arg(value_enum)]
    pub proposal_type: ProposalTypeArg,

    /// Proposal title
    #[arg(short, long)]
    pub title: String,

    /// Proposal description
    #[arg(short, long)]
    pub description: String,

    /// Target address (for execution)
    #[arg(long)]
    pub target: Option<String>,

    /// Call data (hex encoded)
    #[arg(long)]
    pub calldata: Option<String>,

    /// Amount (for treasury proposals)
    #[arg(long)]
    pub amount: Option<f64>,

    /// Skip confirmation
    #[arg(short, long)]
    pub yes: bool,
}

/// Proposal type argument
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum ProposalTypeArg {
    Standard,
    Emergency,
    ParameterChange,
    Treasury,
    Upgrade,
}

/// Run propose command
pub async fn run(args: ProposeArgs, _config: Config) -> anyhow::Result<()> {
    println!("📝 Creating proposal...");
    println!("Title: {}", args.title);
    println!("Type: {:?}", args.proposal_type);
    println!("Description: {}", args.description);

    if let Some(target) = args.target {
        println!("Target: {}", target);
    }

    if let Some(amount) = args.amount {
        println!("Amount: {}", amount);
    }

    println!("\n⚠️  Chain operations are not fully implemented yet.");
    println!("   Enable the 'chain' feature for full blockchain support.");

    Ok(())
}
