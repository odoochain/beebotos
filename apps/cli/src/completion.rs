use clap::Command;

pub fn generate(shell: &str, cmd: &mut Command) -> anyhow::Result<()> {
    match shell {
        "bash" => {
            clap_complete::generate(
                clap_complete::shells::Bash,
                cmd,
                "beebotos",
                &mut std::io::stdout(),
            );
        }
        "zsh" => {
            clap_complete::generate(
                clap_complete::shells::Zsh,
                cmd,
                "beebotos",
                &mut std::io::stdout(),
            );
        }
        "fish" => {
            clap_complete::generate(
                clap_complete::shells::Fish,
                cmd,
                "beebotos",
                &mut std::io::stdout(),
            );
        }
        "powershell" => {
            clap_complete::generate(
                clap_complete::shells::PowerShell,
                cmd,
                "beebotos",
                &mut std::io::stdout(),
            );
        }
        "elvish" => {
            clap_complete::generate(
                clap_complete::shells::Elvish,
                cmd,
                "beebotos",
                &mut std::io::stdout(),
            );
        }
        _ => {
            eprintln!("Unknown shell: {}", shell);
            eprintln!("Supported shells: bash, zsh, fish, powershell, elvish");
        }
    };
    Ok(())
}

#[allow(dead_code)]
pub fn print_installation_instructions(shell: &str) {
    println!(
        "# To enable completions for {}, add the following to your shell configuration:",
        shell
    );
    println!();

    match shell {
        "bash" => {
            println!("# Add to ~/.bashrc or ~/.bash_profile:");
            println!("source <(beebotos completion bash)");
            println!();
            println!("# Or save to a file and source it:");
            println!("beebotos completion bash > /etc/bash_completion.d/beebotos");
        }
        "zsh" => {
            println!("# Add to ~/.zshrc:");
            println!("source <(beebotos completion zsh)");
            println!();
            println!("# Or save to a file:");
            println!("beebotos completion zsh > ${{fpath[1]}}/_beebotos");
        }
        "fish" => {
            println!("# Save to completions directory:");
            println!("beebotos completion fish > ~/.config/fish/completions/beebotos.fish");
        }
        "powershell" => {
            println!("# Add to your PowerShell profile:");
            println!("beebotos completion powershell | Out-String | Invoke-Expression");
            println!();
            println!("# Or save to a file:");
            println!("beebotos completion powershell > $PROFILE.Completions/beebotos.ps1");
        }
        _ => {}
    }
}
