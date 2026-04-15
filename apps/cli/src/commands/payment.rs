use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct PaymentArgs {
    #[command(subcommand)]
    pub command: PaymentCommand,
}

#[derive(Subcommand)]
pub enum PaymentCommand {
    /// Show balance
    Balance {
        /// Token symbol (default: native)
        #[arg(short, long)]
        token: Option<String>,
    },

    /// Send payment
    Send {
        /// Recipient address
        to: String,

        /// Amount
        amount: String,

        /// Token symbol
        #[arg(short, long)]
        token: Option<String>,

        /// Payment description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Create payment mandate
    Mandate {
        #[command(subcommand)]
        action: MandateAction,
    },

    /// List transactions
    History {
        /// Number of transactions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by token
        #[arg(short, long)]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MandateAction {
    /// Create a new mandate
    Create {
        /// Grantee address
        grantee: String,

        /// Allowance amount
        allowance: String,

        /// Token symbol
        #[arg(short, long)]
        token: Option<String>,

        /// Validity duration in days
        #[arg(short, long, default_value = "30")]
        duration: u32,
    },

    /// List mandates
    List {
        /// Show as grantor
        #[arg(long)]
        grantor: bool,

        /// Show as grantee
        #[arg(long)]
        grantee: bool,
    },

    /// Revoke a mandate
    Revoke {
        /// Mandate ID
        id: String,
    },
}

pub async fn execute(args: PaymentArgs) -> Result<()> {
    let client = crate::client::ChainClient::new()?;

    match args.command {
        PaymentCommand::Balance { token } => {
            let balance = client
                .get_balance(&client.default_address(), token.as_deref())
                .await?;
            if let Some(symbol) = token {
                println!("Balance: {} {}", balance, symbol);
            } else {
                println!("Balance: {} BEE", balance);
            }
        }

        PaymentCommand::Send {
            to,
            amount,
            token,
            description,
        } => {
            println!("Sending {} to {}...", amount, to);
            let tx_hash = client.transfer(&to, &amount, token.as_deref()).await?;
            println!("Transaction: {}", tx_hash);

            if let Some(desc) = description {
                // Store payment metadata
                client.store_payment_metadata(&tx_hash, &desc).await?;
            }
        }

        PaymentCommand::Mandate { action } => match action {
            MandateAction::Create {
                grantee,
                allowance,
                token,
                duration,
            } => {
                println!(
                    "Creating mandate for {} (allowance: {})...",
                    grantee, allowance
                );
                let mandate = client
                    .create_mandate(&grantee, &allowance, token.as_deref(), duration)
                    .await?;
                println!("Mandate created: {}", mandate.id);
                println!("Expires in {} days", duration);
            }
            MandateAction::List { grantor, grantee } => {
                let mandates = if grantor {
                    client.list_mandates_as_grantor().await?
                } else if grantee {
                    client.list_mandates_as_grantee().await?
                } else {
                    client.list_all_mandates().await?
                };

                println!(
                    "{:<36} {:<42} {:<20} {:<10}",
                    "ID", "Counterparty", "Allowance", "Status"
                );
                println!("{}", "-".repeat(108));
                for m in mandates {
                    let counterparty = if grantor { &m.grantee } else { &m.grantor };
                    println!(
                        "{:<36} {:<42} {:<20} {:<10}",
                        m.id,
                        counterparty,
                        m.remaining,
                        if m.active { "Active" } else { "Inactive" }
                    );
                }
            }
            MandateAction::Revoke { id } => {
                println!("Revoking mandate '{}'...", id);
                client.revoke_mandate(&id).await?;
                println!("Mandate revoked.");
            }
        },

        PaymentCommand::History { limit, token } => {
            let txs = client.get_transactions(token.as_deref(), limit).await?;
            println!(
                "{:<66} {:<20} {:<15} {:<10}",
                "Tx Hash", "Amount", "Token", "Status"
            );
            println!("{}", "-".repeat(111));
            for tx in txs {
                println!(
                    "{:<66} {:<20} {:<15} {:<10}",
                    tx.hash, tx.amount, tx.token, tx.status
                );
            }
        }
    }

    Ok(())
}
