use anyhow::Result;
use clap::{Parser, Subcommand};
use futures::StreamExt;

#[derive(Parser)]
pub struct ChainArgs {
    #[command(subcommand)]
    pub command: ChainCommand,
}

#[derive(Subcommand)]
pub enum ChainCommand {
    /// Show chain status
    Status,

    /// Query balance
    Balance {
        /// Address (defaults to current account)
        #[arg(long)]
        address: Option<String>,

        /// Token symbol (defaults to native token)
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Transfer tokens
    Transfer {
        /// Recipient address
        to: String,

        /// Amount
        amount: String,

        /// Token symbol
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Deploy contract
    Deploy {
        /// Contract artifact path
        artifact: String,

        /// Constructor arguments
        #[arg(long)]
        args: Vec<String>,

        /// Skip verification
        #[arg(long)]
        no_verify: bool,
    },

    /// Call contract
    Call {
        /// Contract address
        contract: String,

        /// Function signature
        function: String,

        /// Function arguments
        #[arg(long)]
        args: Vec<String>,

        /// Send as transaction (default: read-only call)
        #[arg(short, long)]
        send: bool,
    },

    /// Watch events
    Watch {
        /// Contract address (optional)
        #[arg(long)]
        contract: Option<String>,

        /// Event signature
        #[arg(long)]
        event: Option<String>,

        /// From block
        #[arg(long)]
        from_block: Option<u64>,
    },
}

pub async fn execute(args: ChainArgs) -> Result<()> {
    let client = crate::client::ChainClient::new()?;

    match args.command {
        ChainCommand::Status => {
            let status = client.get_status().await?;
            println!("Chain Status:");
            println!("  Network: {}", status.network);
            println!("  Chain ID: {}", status.chain_id);
            println!("  Block Number: {}", status.block_number);
            println!("  Sync Status: {}", status.sync_status);
        }

        ChainCommand::Balance { address, token } => {
            let addr = address.unwrap_or_else(|| client.default_address());
            let balance = client.get_balance(&addr, token.as_deref()).await?;
            println!("Balance for {}: {}", addr, balance);
        }

        ChainCommand::Transfer { to, amount, token } => {
            println!("Transferring {} to {}...", amount, to);
            let tx_hash = client.transfer(&to, &amount, token.as_deref()).await?;
            println!("Transaction sent: {}", tx_hash);
            println!("Waiting for confirmation...");
            let receipt = client.wait_for_confirmation(&tx_hash).await?;
            println!("Confirmed in block {}", receipt.block_number);
        }

        ChainCommand::Deploy {
            artifact,
            args,
            no_verify,
        } => {
            println!("Deploying contract from {}...", artifact);
            let address = client.deploy_contract(&artifact, &args).await?;
            println!("Contract deployed at: {}", address);

            if !no_verify {
                println!("Verifying contract...");
                client.verify_contract(&address, &artifact).await?;
                println!("Contract verified.");
            }
        }

        ChainCommand::Call {
            contract,
            function,
            args,
            send,
        } => {
            if send {
                println!("Calling {} on {}...", function, contract);
                let tx_hash = client.send_transaction(&contract, &function, &args).await?;
                println!("Transaction: {}", tx_hash);
            } else {
                let result = client.call(&contract, &function, &args).await?;
                println!("Result: {}", result);
            }
        }

        ChainCommand::Watch {
            contract,
            event,
            from_block,
        } => {
            println!("Watching events... (Ctrl+C to exit)");
            let mut stream = client
                .watch_events(contract.as_deref(), event.as_deref(), from_block)
                .await?;
            while let Some(evt) = stream.next().await {
                println!("New event: {:?}", evt);
            }
        }
    }

    Ok(())
}
