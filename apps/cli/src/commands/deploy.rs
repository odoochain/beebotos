//! Deploy command for BeeBotOS CLI

use crate::config::Config;

/// Deploy subcommand
#[derive(clap::Args)]
pub struct DeployArgs {
    /// Contract to deploy
    #[arg(value_enum)]
    pub contract: ContractType,

    /// Network to deploy to
    #[arg(short, long, default_value = "monad")]
    pub network: String,

    /// Constructor arguments (JSON array)
    #[arg(long)]
    pub args: Option<String>,

    /// Verify on explorer
    #[arg(long)]
    pub verify: bool,

    /// Gas price override (in gwei)
    #[arg(long)]
    pub gas_price: Option<u64>,

    /// Skip confirmation
    #[arg(short, long)]
    pub yes: bool,
}

/// Contract types
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum ContractType {
    /// Deploy full DAO suite
    Dao,
    /// Deploy BeeToken only
    Token,
    /// Deploy TreasuryManager
    Treasury,
    /// Deploy AgentRegistry
    Registry,
}

/// Run deploy command
pub async fn run(args: DeployArgs, _config: Config) -> anyhow::Result<()> {
    println!("🚀 Deploying to {}...", args.network);
    println!("Contract type: {:?}", args.contract);

    if let Some(constructor_args) = args.args {
        println!("Constructor args: {}", constructor_args);
    }

    if args.verify {
        println!("Will verify on explorer");
    }

    if let Some(gas) = args.gas_price {
        println!("Gas price: {} gwei", gas);
    }

    println!("\n⚠️  Chain operations are not fully implemented yet.");
    println!("   Enable the 'chain' feature for full blockchain support.");

    Ok(())
}

fn _load_abi(_name: &str) -> anyhow::Result<serde_json::Value> {
    let path = format!("contracts/artifacts/{}.json", _name);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to load {}: {}", path, e))?;
    let artifact: serde_json::Value = serde_json::from_str(&content)?;
    Ok(artifact["abi"].clone())
}

fn _load_bytecode(_name: &str) -> anyhow::Result<String> {
    let path = format!("contracts/artifacts/{}.json", _name);
    let content = std::fs::read_to_string(&path)?;
    let artifact: serde_json::Value = serde_json::from_str(&content)?;
    artifact["bytecode"]["object"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("Bytecode not found"))
}
