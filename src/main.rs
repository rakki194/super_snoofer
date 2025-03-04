#![warn(clippy::all, clippy::pedantic)]

use anyhow::{anyhow, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io::Write;
use clap::{CommandFactory, Parser, Subcommand};

// Import modules for functionality
use super_snoofer::{
    cache::CommandCache,
    history::HistoryTracker,
    ollama::OllamaClient,
    shell::{install_shell_integration, uninstall_shell_integration},
    tui::{draw_ui, TuiApp},
};

mod ollama;
mod shell;
mod tui;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Prompt to process (launches TUI mode)
    #[arg(short, long)]
    prompt: Option<String>,

    /// Use Codestral model instead of Dolphin
    #[arg(long)]
    codestral: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install shell integration
    Install,
    /// Uninstall shell integration
    Uninstall,
    /// Normal operation: suggest similar commands
    Command {
        command: String,
    },
    /// Clear the command cache but keep learned corrections
    ResetCache,
    /// Clear both the command cache and learned corrections
    ResetMemory,
    /// Display your recent command corrections
    History,
    /// Display your most common typos
    FrequentTypos,
    /// Display your most frequently used corrections
    FrequentCorrections,
    /// Clear your command history
    ClearHistory,
    /// Enable command history tracking
    EnableHistory,
    /// Disable command history tracking
    DisableHistory,
    /// Add shell alias (default: super_snoofer)
    AddAlias {
        /// Alias name
        name: String,
        /// Command to alias (defaults to super_snoofer)
        #[arg(default_value = "super_snoofer")]
        command: Option<String>,
    },
    /// Suggest personalized shell aliases
    Suggest,
    /// Check command line for corrections
    CheckCommandLine {
        /// Command line to check
        command: String,
    },
    /// Process a full command line (for shell integration)
    FullCommand {
        /// Command line to process
        command: String,
    },
    /// Manually teach a command correction
    LearnCorrection {
        /// The typo to correct
        typo: String,
        /// The correct command
        command: String,
    },
    /// Chat with AI about super snoofer
    Prompt {
        /// Question to ask
        prompt: String,
        /// Use Codestral model instead of Dolphin
        #[arg(long)]
        codestral: bool,
    },
}

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

/// Add a shell alias
fn add_alias(name: &str, command: Option<&str>) -> Result<()> {
    let command = command.unwrap_or("super_snoofer");
    let (shell_type, config_path, alias_line) = super_snoofer::shell::detect_shell_config(name, command)?;
    super_snoofer::shell::add_to_shell_config(&shell_type, std::path::Path::new(&config_path), &alias_line)?;
    Ok(())
}

/// Suggest personalized shell aliases
fn suggest_aliases() -> Result<()> {
    let cache = CommandCache::load()?;
    if !cache.is_history_enabled() {
        println!("Command history tracking is disabled! Cannot generate suggestions. 🐺");
        return Ok(());
    }

    let corrections = cache.get_frequent_corrections(5);
    if corrections.is_empty() {
        println!("No alias suggestions available yet! Keep using Super Snoofer to generate personalized suggestions. 🐺");
        return Ok(());
    }

    for (_i, (command, count)) in corrections.iter().enumerate() {
        let alias = if command.len() <= 3 {
            command.to_string()
        } else {
            command[0..2].to_string()
        };

        println!("\nYou've used '{}' {} times! Let's create an alias for that.", command, count);
        println!("\nSuggested alias: {} → {}", alias, command);
        println!("\nTo add this alias to your shell configuration:");
        println!("\nalias {}='{}'", alias, command);
        
        print!("\nWould you like me to add this alias to your shell configuration? (y/N) ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().eq_ignore_ascii_case("y") {
            add_alias(&alias, Some(command))?;
            println!("Added alias to your shell configuration! 🐺");
        }
    }
    Ok(())
}

/// Check command line for corrections
fn check_command_line(command: &str) -> Result<()> {
    let cache = CommandCache::load()?;
    if let Some(correction) = cache.fix_command_line(command) {
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

/// Run the TUI mode
async fn run_tui_mode(prompt: &str, use_codestral: bool) -> Result<()> {
    // Initialize terminal
    let ollama = OllamaClient::new().await?;
    let mut terminal = TuiApp::new(ollama, use_codestral)?;
    
    // Set the initial prompt
    terminal.input = prompt.to_string();
    terminal.cursor_position = terminal.input.len();

    // Event loop
    loop {
        // Create local copies of the values we need
        let input = terminal.input.clone();
        let cursor_position = terminal.cursor_position;
        let thinking_visible = terminal.thinking_visible;
        let thinking_text = terminal.thinking_text.clone();
        let response_text = terminal.response_text.clone();

        // Draw the UI using local copies
        terminal.draw(|f| {
            draw_ui(
                f,
                &terminal
            )
        })?;

        // Handle input
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        terminal.submit_prompt().await?;
                    }
                    KeyCode::Char(c) => {
                        terminal.enter_char(c);
                    }
                    KeyCode::Backspace => {
                        terminal.delete_char();
                    }
                    KeyCode::Delete => {
                        terminal.delete_char_forward();
                    }
                    KeyCode::Left => {
                        terminal.move_cursor_left();
                    }
                    KeyCode::Right => {
                        terminal.move_cursor_right();
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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
            check_command_line(command)?;
        }
        Some(Commands::ResetCache) => {
            reset_cache()?;
            println!("Command cache cleared successfully! 🐺");
        }
        Some(Commands::ResetMemory) => {
            reset_memory()?;
            println!("Command cache and learned corrections cleared successfully! 🐺");
        }
        Some(Commands::History) => {
            show_history()?;
        }
        Some(Commands::FrequentTypos) => {
            show_frequent_typos()?;
        }
        Some(Commands::FrequentCorrections) => {
            show_frequent_corrections()?;
        }
        Some(Commands::ClearHistory) => {
            clear_history()?;
            println!("Command history cleared successfully! 🐺");
        }
        Some(Commands::EnableHistory) => {
            enable_history()?;
            println!("Command history tracking is now enabled! 🐺");
        }
        Some(Commands::DisableHistory) => {
            disable_history()?;
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
            check_command_line(command)?;
        }
        Some(Commands::FullCommand { command }) => {
            process_full_command(command)?;
        }
        Some(Commands::LearnCorrection { typo, command }) => {
            learn_correction(typo, command)?;
            println!("Correction learned successfully! 🐺");
        }
        Some(Commands::Prompt { prompt, codestral }) => {
            run_tui_mode(&prompt, *codestral).await?;
        }
        None => {
            // No command provided, show help
            let mut cmd = Cli::command();
            cmd.print_help()?;
        }
    }

    Ok(())
}
