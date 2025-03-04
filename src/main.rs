#![warn(clippy::all, clippy::pedantic)]

use anyhow::{anyhow, Result};
use std::io::Write;

// Import modules for functionality
use super_snoofer::{
    cache::CommandCache,
    history::HistoryTracker,
    shell::{install_shell_integration, uninstall_shell_integration, add_alias, suggest_aliases},
    tui::run_tui_mode,
};

mod cli;
mod commands;
mod ollama;
mod tui;

use cli::{Cli, Commands};

/// Learn a command correction
fn learn_correction(typo: &str, command: &str) -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.learn_correction(typo, command)?;
    println!("Got it! 🐺 I'll remember that '{}' means '{}'", typo, command);
    cache.save()?;
    Ok(())
}

/// Reset the command cache but keep learned corrections
fn reset_cache() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_cache();
    cache.save()?;
    Ok(())
}

/// Reset both the command cache and learned corrections
fn reset_memory() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_memory();
    cache.save()?;
    Ok(())
}

/// Show command history
fn show_history() -> Result<()> {
    let cache = CommandCache::load()?;
    if !cache.is_history_enabled() {
        println!("Command history tracking is disabled! 🐺");
        return Ok(());
    }
    
    let history = cache.get_command_history(10);
    if history.is_empty() {
        println!("No command history found! 🐺");
        return Ok(());
    }

    println!("🐺 Your recent command corrections:");
    for (i, entry) in history.iter().enumerate() {
        println!("{}. {} → {}", i + 1, entry.typo, entry.correction);
    }
    Ok(())
}

/// Show most frequent typos
fn show_frequent_typos() -> Result<()> {
    let cache = CommandCache::load()?;
    if !cache.is_history_enabled() {
        println!("Command history tracking is disabled! 🐺");
        return Ok(());
    }

    let typos = cache.get_frequent_typos(10);
    if typos.is_empty() {
        println!("No typos found! 🐺");
        return Ok(());
    }

    println!("🐺 Your most common typos:");
    for (i, (typo, count)) in typos.iter().enumerate() {
        println!("{}. {} ({} times)", i + 1, typo, count);
    }
    Ok(())
}

/// Show most frequently used corrections
fn show_frequent_corrections() -> Result<()> {
    let cache = CommandCache::load()?;
    if !cache.is_history_enabled() {
        println!("Command history tracking is disabled! 🐺");
        return Ok(());
    }

    let corrections = cache.get_frequent_corrections(10);
    if corrections.is_empty() {
        println!("No corrections found! 🐺");
        return Ok(());
    }

    println!("🐺 Your most frequently used corrections:");
    for (i, (correction, count)) in corrections.iter().enumerate() {
        println!("{}. {} ({} times)", i + 1, correction, count);
    }
    Ok(())
}

/// Clear command history
fn clear_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_history();
    cache.save()?;
    Ok(())
}

/// Enable command history tracking
fn enable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.enable_history()?;
    cache.save()?;
    Ok(())
}

/// Disable command history tracking
fn disable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.disable_history()?;
    cache.save()?;
    Ok(())
}

/// Check command line for corrections
fn check_command_line(command: &str) -> Result<()> {
    let cache = CommandCache::load()?;
    let correction = match cache.fix_command_line(command) {
        Some(s) => s.to_string(),
        None => return Ok(()),
    };
    println!("Awoo! 🐺 Did you mean `{}`? *wags tail* (Y/n/c)", correction);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    match input.trim().to_lowercase().as_str() {
        "y" | "" => {
            println!("Running suggested command...");
            process_full_command(&correction)?;
        }
        "c" => {
            print!("What's the correct command? ");
            std::io::stdout().flush()?;
            let mut correct = String::new();
            std::io::stdin().read_line(&mut correct)?;
            learn_correction(command, correct.trim())?;
        }
        _ => println!("Command '{}' not found! 🐺", command)
    }
    Ok(())
}

/// Process a full command line
fn process_full_command(command: &str) -> Result<()> {
    // Split the command into program and arguments
    let mut parts = command.split_whitespace();
    let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
    let args: Vec<&str> = parts.collect();
    
    // Execute the command
    let status = std::process::Command::new(program)
        .args(args)
        .status()?;
    
    if !status.success() {
        return Err(anyhow!("Command failed with status: {}", status));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse_args();

    // Handle command not found case
    if !cli.command_to_check.is_empty() {
        let cmd = cli.command_to_check.join(" ");
        return commands::check_command_line(&cmd);
    }

    // Handle prompt mode
    if let Some(prompt) = cli.prompt.as_ref() {
        return run_tui_mode(prompt, cli.codestral).await;
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
            commands::check_command_line(command)?;
        }
        Some(Commands::ResetCache) => {
            commands::reset_cache()?;
            println!("Command cache cleared successfully! 🐺");
        }
        Some(Commands::ResetMemory) => {
            commands::reset_memory()?;
            println!("Command cache and learned corrections cleared successfully! 🐺");
        }
        Some(Commands::History) => {
            commands::show_history()?;
        }
        Some(Commands::FrequentTypos) => {
            commands::show_frequent_typos()?;
        }
        Some(Commands::FrequentCorrections) => {
            commands::show_frequent_corrections()?;
        }
        Some(Commands::ClearHistory) => {
            commands::clear_history()?;
            println!("Command history cleared successfully! 🐺");
        }
        Some(Commands::EnableHistory) => {
            commands::enable_history()?;
            println!("Command history tracking is now enabled! 🐺");
        }
        Some(Commands::DisableHistory) => {
            commands::disable_history()?;
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
            commands::check_command_line(command)?;
        }
        Some(Commands::FullCommand { command }) => {
            commands::process_full_command(command)?;
        }
        Some(Commands::LearnCorrection { typo, command }) => {
            commands::learn_correction(typo, command)?;
            println!("Correction learned successfully! 🐺");
        }
        Some(Commands::Prompt { prompt, codestral }) => {
            run_tui_mode(prompt, *codestral).await?;
        }
        None => {
            // Show help
            println!("Super Snoofer - Your friendly command line companion! 🐺");
            println!("Use --help to see available commands.");
        }
    }

    Ok(())
}
