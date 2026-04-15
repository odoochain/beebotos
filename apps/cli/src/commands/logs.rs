//! View logs command

use crate::config::Config;

/// Logs subcommand
#[derive(clap::Args)]
pub struct LogsArgs {
    /// Component to view logs for
    #[arg(value_enum)]
    pub component: Option<Component>,

    /// Number of lines to show
    #[arg(short, long, default_value = "100")]
    pub lines: usize,

    /// Follow logs (tail -f)
    #[arg(long)]
    pub follow: bool,

    /// Show all logs
    #[arg(long)]
    pub all: bool,

    /// Filter by level
    #[arg(long, value_enum)]
    pub level: Option<LogLevel>,

    /// Filter by pattern
    #[arg(short, long)]
    pub grep: Option<String>,
}

/// Log component
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum Component {
    Kernel,
    Agents,
    Chain,
    Gateway,
    Dao,
}

/// Log level
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Run logs command
pub async fn run(args: LogsArgs, _config: Config) -> anyhow::Result<()> {
    let log_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find data directory"))?
        .join("beebotos")
        .join("logs");

    if !log_dir.exists() {
        anyhow::bail!("No logs found. Is BeeBotOS running?");
    }

    // Determine which logs to show
    let log_files: Vec<_> = if args.all {
        std::fs::read_dir(&log_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .map(|e| e.path())
            .collect()
    } else if let Some(component) = args.component {
        let filename = format!("{:?}.log", component).to_lowercase();
        vec![log_dir.join(filename)]
    } else {
        // Default to kernel logs
        vec![log_dir.join("kernel.log")]
    };

    for log_file in log_files {
        if !log_file.exists() {
            continue;
        }

        println!("=== {} ===", log_file.display());

        if args.follow {
            // Tail -f equivalent
            use std::io::{BufRead, Seek};

            let file = std::fs::File::open(&log_file)?;
            let mut reader = std::io::BufReader::new(file);
            reader.seek(std::io::SeekFrom::End(0))?;

            loop {
                let mut line = String::new();
                if reader.read_line(&mut line)? > 0 {
                    if should_print(&line, &args) {
                        print!("{}", line);
                    }
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        } else {
            // Read last N lines
            let content = std::fs::read_to_string(&log_file)?;
            let lines: Vec<_> = content.lines().collect();

            let start = lines.len().saturating_sub(args.lines);
            for line in &lines[start..] {
                if should_print(line, &args) {
                    println!("{}", line);
                }
            }
        }

        println!();
    }

    Ok(())
}

fn should_print(line: &str, args: &LogsArgs) -> bool {
    // Filter by level
    if let Some(level) = args.level {
        let level_str = format!("{:?}", level).to_uppercase();
        if !line.contains(&level_str) {
            return false;
        }
    }

    // Filter by pattern
    if let Some(pattern) = &args.grep {
        if !line.contains(pattern) {
            return false;
        }
    }

    true
}
