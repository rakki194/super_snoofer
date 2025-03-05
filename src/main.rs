#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;

// Import modules for functionality
use super_snoofer::{
    commands::{self as cmd},
    shell::{add_alias, install_shell_integration, suggest_aliases, uninstall_shell_integration},
};

use crate::ollama::ModelConfig;
use crate::tui::run_tui_mode;

mod cli;
use cli::{Cli, Commands};
mod ollama;
mod tui;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();
    
    // Create model configuration from CLI parameters
    let model_config = ModelConfig::new(cli.standard_model, cli.code_model);

    // Check if we're coming from a failed ] command
    // The command_to_check will contain "]" if it wasn't intercepted properly
    if cli.command_to_check.len() == 1 && cli.command_to_check[0] == "]" {
        println!("Detected issue with ']' command integration. Fixing shell integration...");
        install_shell_integration()?;
        println!("Shell integration fixed. Please restart your shell or run 'source ~/.zshrc'");
        println!("Launching AI prompt interface now...");
        return run_tui_mode("", false, model_config).await;
    }

    // Handle command not found case
    if !cli.command_to_check.is_empty() {
        let cmd = cli.command_to_check.join(" ");
        return cmd::check_command_line(&cmd);
    }

    // Handle prompt mode
    if let Some(prompt) = cli.prompt.as_ref() {
        return run_tui_mode(prompt, cli.codestral, model_config).await;
    }

    match &cli.command {
        Some(Commands::Install) => {
            install_shell_integration()?;
            println!("Shell integration installed successfully! 🐺");
            println!("Please restart your shell or run 'source ~/.zshrc' to apply changes.");
        }
        Some(Commands::Uninstall) => {
            uninstall_shell_integration()?;
            println!("Shell integration uninstalled successfully! 🐺");
            println!("Please restart your shell or run 'source ~/.zshrc' to apply changes.");
        }
        Some(Commands::Command { command }) => {
            cmd::check_command_line(command)?;
        }
        Some(Commands::ResetCache) => {
            cmd::reset_cache()?;
            println!("Command cache cleared successfully! 🐺");
        }
        Some(Commands::ResetMemory) => {
            cmd::reset_memory()?;
            println!("Command cache and learned corrections cleared successfully! 🐺");
        }
        Some(Commands::History) => {
            cmd::show_history()?;
        }
        Some(Commands::FrequentTypos) => {
            cmd::show_frequent_typos()?;
        }
        Some(Commands::FrequentCorrections) => {
            cmd::show_frequent_corrections()?;
        }
        Some(Commands::ClearHistory) => {
            cmd::clear_history()?;
            println!("Command history cleared successfully! 🐺");
        }
        Some(Commands::EnableHistory) => {
            cmd::enable_history()?;
            println!("Command history tracking is now enabled! 🐺");
        }
        Some(Commands::DisableHistory) => {
            cmd::disable_history()?;
            println!("Command history tracking is now disabled! 🐺");
        }
        Some(Commands::AddAlias { name, command }) => {
            add_alias(name, command.as_deref())?;
            println!("Alias added successfully! 🐺");
            println!("Please restart your shell or run 'source ~/.zshrc' to apply changes.");
        }
        Some(Commands::Suggest) => {
            suggest_aliases()?;
        }
        Some(Commands::CheckCommandLine { command }) => {
            cmd::check_command_line(command)?;
        }
        Some(Commands::FullCommand { command }) => {
            cmd::process_full_command(command)?;
        }
        Some(Commands::LearnCorrection { typo, command }) => {
            cmd::learn_correction(typo, command)?;
            println!("Correction learned successfully! 🐺");
        }
        Some(Commands::Prompt { prompt, codestral, standard_model, code_model }) => {
            // Create a command-specific model config that overrides the global one
            let cmd_model_config = ModelConfig::new(standard_model.clone(), code_model.clone());
            run_tui_mode(prompt, *codestral, cmd_model_config).await?;
        }
        None => {
            // Show help
            println!("Super Snoofer - Your friendly command line companion! 🐺");
            println!("Use --help to see available commands.");
        }
    }

    Ok(())
}
