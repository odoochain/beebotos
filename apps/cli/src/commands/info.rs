use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
pub struct InfoArgs {
    /// Show detailed information
    #[arg(short, long)]
    pub detailed: bool,

    /// Output format (table, json, yaml)
    #[arg(long, value_enum, default_value = "table")]
    pub output_format: InfoOutputFormat,
}

#[derive(Clone, Copy, Debug, Default, clap::ValueEnum)]
pub enum InfoOutputFormat {
    #[default]
    Table,
    Json,
    Yaml,
}

pub async fn execute(args: InfoArgs) -> Result<()> {
    let info = SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        commit_hash: option_env!("VERGEN_GIT_SHA")
            .unwrap_or("unknown")
            .to_string(),
        build_date: option_env!("VERGEN_BUILD_DATE")
            .unwrap_or("unknown")
            .to_string(),
        rust_version: option_env!("VERGEN_RUSTC_SEMVER")
            .unwrap_or("unknown")
            .to_string(),
        platform: format!(
            "{}-{}",
            option_env!("VERGEN_CARGO_TARGET_OS").unwrap_or("unknown"),
            option_env!("VERGEN_CARGO_TARGET_ARCH").unwrap_or("unknown")
        ),
    };

    match args.output_format {
        InfoOutputFormat::Json => println!("{}", serde_json::to_string_pretty(&info)?),
        InfoOutputFormat::Yaml => println!("{}", serde_yaml::to_string(&info)?),
        InfoOutputFormat::Table => {
            println!("BeeBotOS CLI Information");
            println!("========================");
            println!("Version:      {}", info.version);
            println!("Commit:       {}", info.commit_hash);
            println!("Build Date:   {}", info.build_date);
            println!("Rust Version: {}", info.rust_version);
            println!("Platform:     {}", info.platform);
        }
    }

    Ok(())
}

#[derive(serde::Serialize)]
struct SystemInfo {
    version: String,
    commit_hash: String,
    build_date: String,
    rust_version: String,
    platform: String,
}
