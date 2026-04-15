use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct InteractiveArgs {
    /// Start with specific agent
    #[arg(long)]
    pub agent: Option<String>,

    /// Load history from file
    #[arg(long)]
    pub history: Option<String>,
}

pub async fn execute(args: InteractiveArgs) -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║     BeeBotOS Interactive Shell         ║");
    println!("║     Type 'help' for commands           ║");
    println!("║     Type 'exit' to quit                ║");
    println!("╚════════════════════════════════════════╝");
    println!();

    let client = crate::client::ApiClient::new()?;
    let mut context = InteractiveContext {
        current_agent: args.agent,
        variables: std::collections::HashMap::new(),
    };

    let mut rl = rustyline::DefaultEditor::new()?;

    // Load history if provided
    if let Some(history_file) = args.history {
        if let Err(e) = rl.load_history(&history_file) {
            eprintln!("Warning: Could not load history: {}", e);
        }
    }

    loop {
        let prompt = context
            .current_agent
            .as_ref()
            .map(|a| format!("{}> ", a))
            .unwrap_or_else(|| "beebotos> ".to_string());

        let input = match rl.readline(&prompt) {
            Ok(line) => line,
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        };

        rl.add_history_entry(&input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }

            "help" => {
                print_help();
            }

            "agent" => {
                if args.is_empty() {
                    if let Some(ref agent) = context.current_agent {
                        println!("Current agent: {}", agent);
                    } else {
                        println!("No agent selected. Use 'agent <id>' to select one.");
                    }
                } else {
                    context.current_agent = Some(args[0].to_string());
                    println!("Switched to agent: {}", args[0]);
                }
            }

            "agents" => match client.list_agents(None, false).await {
                Ok(agents) => {
                    for agent in agents {
                        let marker = context
                            .current_agent
                            .as_ref()
                            .filter(|a| *a == &agent.id)
                            .map(|_| "* ")
                            .unwrap_or("  ");
                        println!("{}{} - {} ({})", marker, agent.id, agent.name, agent.status);
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            },

            "say" | "ask" => {
                let message = args.join(" ");
                if let Some(ref agent) = context.current_agent {
                    match client.send_message(agent, &message, 60).await {
                        Ok(response) => println!("{}", response),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    println!("No agent selected. Use 'agent <id>' to select one.");
                }
            }

            "status" => {
                if let Some(ref agent) = context.current_agent {
                    match client.get_agent(agent).await {
                        Ok(info) => {
                            println!("Agent: {}", info.name);
                            println!("  Status: {}", info.status);
                            println!("  Last Active: {}", info.last_active);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                } else {
                    println!("No agent selected.");
                }
            }

            "set" => {
                if args.len() >= 2 {
                    let key = args[0];
                    let value = args[1..].join(" ");
                    context.variables.insert(key.to_string(), value);
                    println!("Set {} = {}", key, context.variables.get(key).unwrap());
                } else {
                    println!("Usage: set <key> <value>");
                }
            }

            "vars" => {
                for (key, value) in &context.variables {
                    println!("{} = {}", key, value);
                }
            }

            _ => {
                // Try to execute as a system command
                println!(
                    "Unknown command: {}. Type 'help' for available commands.",
                    cmd
                );
            }
        }
    }

    Ok(())
}

struct InteractiveContext {
    current_agent: Option<String>,
    variables: std::collections::HashMap<String, String>,
}

fn print_help() {
    println!("Available commands:");
    println!();
    println!("  agent [id]        - Show or select current agent");
    println!("  agents            - List all agents");
    println!("  ask <message>     - Send message to current agent");
    println!("  status            - Show current agent status");
    println!("  set <key> <val>   - Set a variable");
    println!("  vars              - List variables");
    println!("  help              - Show this help");
    println!("  exit              - Exit interactive shell");
    println!();
}
