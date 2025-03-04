#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::{
    io::{Write},
    process::{Command, exit},
};
use clap::{Parser, Subcommand};

// Import modules for functionality
use super_snoofer::{CommandCache, HistoryTracker, display, suggestion, tui::TerminalUI};

mod ollama;
mod shell;
mod tui;

use ollama::OllamaClient;
use shell::{install_shell_integration, uninstall_shell_integration};
use tui::{TuiApp, draw_ui};

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
}

#[derive(Debug)]
struct App {
    input: String,
    cursor_position: usize,
    scroll: u16,
    thinking_visible: bool,
    thinking_text: String,
    response_text: String,
    ollama: OllamaClient,
    last_response: Option<String>,
}

impl App {
    async fn new() -> Result<App> {
        Ok(App {
            input: String::new(),
            cursor_position: 0,
            scroll: 0,
            thinking_visible: true,
            thinking_text: String::new(),
            response_text: String::new(),
            ollama: OllamaClient::new(),
            last_response: None,
        })
    }

    fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    async fn submit_prompt(&mut self) -> Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }

        self.thinking_text = "🤔 Thinking...".to_string();
        self.thinking_visible = true;

        let prompt = self.input.clone();
        self.response_text.clear();
        
        // Get response from Ollama
        match self.ollama.generate_response(&prompt, false).await {
            Ok(response) => {
                self.response_text = response;
            }
            Err(e) => {
                self.response_text = format!("Error: {}", e);
            }
        }

        self.last_response = Some(self.response_text.clone());
        self.thinking_visible = false;
        Ok(())
    }

    fn toggle_thinking(&mut self) {
        self.thinking_visible = !self.thinking_visible;
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Input box
    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .scroll((0, app.scroll));
    f.render_widget(input, chunks[0]);

    // Set cursor position
    f.set_cursor_position((
        chunks[0].x + app.cursor_position as u16 + 1,
        chunks[0].y + 1,
    ));

    // Thinking area (collapsible)
    if app.thinking_visible {
        let thinking = Paragraph::new(app.thinking_text.as_str())
            .block(Block::default().borders(Borders::ALL).title("Thinking 🤔"));
        f.render_widget(thinking, chunks[1]);
    }

    // Response area
    let response = Paragraph::new(app.response_text.as_str())
        .block(Block::default().borders(Borders::ALL).title("Response 🐬"));
    f.render_widget(response, chunks[2]);
}

/// Handle cache management commands
fn handle_cache_commands(command: &str) -> Result<()> {
    match command {
        "--reset_cache" => {
            let mut cache = CommandCache::load()?;
            cache.clear_cache();
            cache.save()?;
            println!("Cache cleared successfully! 🐺");
            exit(0);
        }
        "--reset_memory" => {
            let mut cache = CommandCache::load()?;
            cache.clear_memory();
            cache.save()?;
            println!("Cache and learned corrections cleared successfully! 🐺");
            exit(0);
        }
        "--clear-history" => {
            let mut cache = CommandCache::load()?;
            // Check if history is enabled
            if !cache.is_history_enabled() {
                println!("🐺 Command history tracking is currently disabled.");
                println!("To enable it, run: super_snoofer --enable-history");
                exit(0);
            }
            cache.clear_history();
            println!("Command history cleared successfully! 🐺");
            cache.save()?;
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle history display commands
fn handle_history_commands(command: &str) -> Result<()> {
    match command {
        "--history" => {
            display::display_command_history()?;
            exit(0);
        }
        "--frequent-typos" => {
            display::display_frequent_typos()?;
            exit(0);
        }
        "--frequent-corrections" => {
            display::display_frequent_corrections()?;
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle history tracking enable/disable commands
fn handle_history_tracking_commands(command: &str) -> Result<()> {
    match command {
        "--enable-history" => {
            let mut cache = CommandCache::load()?;
            cache.enable_history()?;
            cache.save()?;
            println!("Command history tracking enabled! 🐺");
            exit(0);
        }
        "--disable-history" => {
            let mut cache = CommandCache::load()?;
            cache.disable_history()?;
            cache.save()?;
            println!("Command history tracking disabled! 🐺");
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle shell integration commands
fn handle_shell_integration(command: &str, _args: &[String]) -> Result<()> {
    match command {
        "--install" => {
            install_shell_integration()?;
            println!("🐺 Shell integration installed successfully!");
            println!("Please restart your shell or run 'source ~/.zshrc' to apply changes.");
            exit(0);
        }
        "--uninstall" => {
            uninstall_shell_integration()?;
            println!("🐺 Shell integration uninstalled successfully!");
            println!("Please restart your shell or run 'source ~/.zshrc' to apply changes.");
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle suggestion commands
fn handle_suggestion_command(command: &str) -> Result<()> {
    match command {
        "--suggest" => {
            suggestion::suggest_alias_command()?;
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle command line check for ZSH integration
fn handle_check_command_line(command: &str, args: &[String]) -> Result<()> {
    if command == "--check-command-line" && args.len() >= 3 {
        // Load cache
        let cache = CommandCache::load()?;
        
        // Reconstruct the command line to check
        let command_line = args[2..].join(" ");
        
        // Try to fix the command line
        if let Some(fixed_command_line) = cache.fix_command_line(&command_line) {
            if fixed_command_line != command_line {
                // Only print the correction if it's different from the input
                println!("{}", fixed_command_line);
            }
        }
        
        exit(0);
    }
    
    Ok(())
}

/// Handle full command line processing for shell integration
fn handle_full_command(command: &str, args: &[String]) -> Result<()> {
    if command == "--full-command" && args.len() >= 3 {
        // Load cache
        let mut cache = CommandCache::load()?;
        
        // Extract the main command and the full command line
        let full_cmd = args[2..].join(" ");
        let cmd_parts: Vec<&str> = full_cmd.splitn(2, ' ').collect();
        let typed_command = cmd_parts[0];
        
        // Check if the command exists in PATH or as an alias
        if cache.command_exists(typed_command)? {
            // Command exists, just pass through
            let status = Command::new("sh")
                .arg("-c")
                .arg(&full_cmd)
                .status()?;
            
            exit(status.code().unwrap_or(1));
        }
        
        // Try to fix the entire command line first
        if full_cmd.contains(' ') {
            if let Some(fixed_command_line) = cache.fix_command_line(&full_cmd) {
                // We found a correction for the entire command line
                let corrections = vec![fixed_command_line];
                
                // Display correction options
                return process_correction_options(
                    typed_command,
                    &full_cmd,
                    &corrections,
                    &mut cache,
                );
            }
        }
        
        // If full command line correction failed, fall back to command-only correction
        let corrections = suggestion::get_command_suggestions(typed_command, &cache);
        
        if corrections.is_empty() {
            println!("Command '{typed_command}' not found! 🐺");
            exit(127); // Standard "command not found" exit code
        }
        
        // Display correction options
        process_correction_options(typed_command, &full_cmd, &corrections, &mut cache)?;
        
        exit(0);
    }
    
    Ok(())
}

/// Handle help display
fn handle_help_command(command: &str) {
    if command == "--help" || command == "-h" {
        println!("Super Snoofer - Command correction utility 🐺");
        println!("Usage:");
        println!("  super_snoofer [OPTION]");
        println!("  super_snoofer [COMMAND] [OPTIONS]");
        println!("\nOptions:");
        println!("  --help, -h                   Show this help message");
        println!("  --reset_cache                Clear the command cache");
        println!("  --reset_memory               Clear the cache and learned corrections");
        println!("  --history                    Show command history");
        println!("  --frequent-typos             Show most common typos");
        println!("  --frequent-corrections       Show most used corrections");
        println!("  --clear-history              Clear command history");
        println!("  --enable-history             Enable command history tracking");
        println!("  --disable-history            Disable command history tracking");
        println!("  --add-alias NAME [CMD]       Add shell alias (default: super_snoofer)");
        println!("  --suggest                    Suggest personalized shell aliases");
        println!("  --check-command-line         Check command line for corrections");
        println!("  --full-command CMD           Process a full command line (for shell integration)");
        println!("  --learn-correction TYPO CMD  Manually teach a command correction");
        exit(0);
    }
}

/// Handle manually learning a command correction
fn handle_learn_correction(command: &str, args: &[String]) -> Result<()> {
    if command == "--learn-correction" && args.len() >= 4 {
        // Load the cache
        let mut cache = CommandCache::load()?;
        
        // Extract the typo and correction from args
        let typo = &args[2];
        let correction = &args[3];
        
        // Learn the correction
        cache.learn_correction(typo, correction)?;
        
        println!("Got it! 🐺 I'll remember that '{}' means '{}'", typo, correction);
        exit(0);
    }
    
    Ok(())
}

/// Process an unrecognized command and suggest corrections
fn process_command(typed_command: &str, command_line: &str) -> Result<()> {
    // Skip our executable name
    let mut cache = CommandCache::load()?;

    // Check if the command exists in PATH or as an alias
    if cache.command_exists(typed_command)? {
        // Command exists, just pass through
        let status = Command::new("sh").arg("-c").arg(command_line).status()?;

        exit(status.code().unwrap_or(1));
    }

    // Try to fix the entire command line first
    if command_line.contains(' ') {
        // Command includes arguments, try to fix the entire line
        if let Some(fixed_command_line) = cache.fix_command_line(command_line) {
            // We found a correction for the entire command line
            let corrections = vec![fixed_command_line];

            // Display correction options
            return process_correction_options(
                typed_command,
                command_line,
                &corrections,
                &mut cache,
            );
        }
    }

    // If full command line correction failed or there were no arguments,
    // fall back to command-only correction
    let corrections = suggestion::get_command_suggestions(typed_command, &cache);

    if corrections.is_empty() {
        println!("Command '{typed_command}' not found! 🐺");
        exit(127); // Standard "command not found" exit code
    }

    // Display correction options
    process_correction_options(typed_command, command_line, &corrections, &mut cache)
}

/// Process and display correction options to the user
fn process_correction_options(
    typed_command: &str,
    command_line: &str,
    corrections: &[String],
    cache: &mut CommandCache,
) -> Result<()> {
    println!("Command '{typed_command}' not found! Did you mean:");

    for (i, correction) in corrections.iter().enumerate() {
        println!("{}. {}", i + 1, correction.bright_green());
    }

    // Add option to enter custom correction
    println!(
        "{}. {}",
        corrections.len() + 1,
        "Enter custom command".bright_yellow()
    );

    // Add option to add permanent alias
    println!(
        "{}. {}",
        corrections.len() + 2,
        "Add permanent shell alias".bright_blue()
    );

    // Add option to exit without running anything
    println!(
        "{}. {}",
        corrections.len() + 3,
        "Exit without running".bright_red()
    );

    print!("Enter your choice (1-{}): ", corrections.len() + 3);
    std::io::stdout().flush()?;

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;

    let choice = choice.trim();

    // If user pressed Enter (empty choice), default to option 1
    if choice.is_empty() && !corrections.is_empty() {
        // Use the first suggestion
        let correction = &corrections[0];
        cache.record_correction(typed_command, correction);
        cache.save()?;

        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "{} {}",
                correction,
                command_line
                    .split_whitespace()
                    .skip(1)
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .join(" ")
            ))
            .status()?;

        exit(status.code().unwrap_or(1));
    }

    // Handle numeric choice
    if let Ok(num) = choice.parse::<usize>() {
        if num >= 1 && num <= corrections.len() {
            // User selected a suggested correction
            let correction = &corrections[num - 1];

            // Record the correction in history
            cache.record_correction(typed_command, correction);
            cache.save()?;

            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "{} {}",
                    correction,
                    command_line
                        .split_whitespace()
                        .skip(1)
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(" ")
                ))
                .status()?;

            exit(status.code().unwrap_or(1));
        } else if num == corrections.len() + 1 {
            // User wants to enter custom command
            print!("Enter the correct command: ");
            std::io::stdout().flush()?;

            let mut correction = String::new();
            std::io::stdin().read_line(&mut correction)?;
            let correction = correction.trim();

            if correction.is_empty() {
                println!("No command entered. Exiting.");
                exit(1);
            }

            // Record the manual correction in history
            cache.record_correction(typed_command, correction);
            cache.learn_correction(typed_command, correction)?;
            println!("Got it! 🐺 I'll remember that '{typed_command}' means '{correction}'");

            let status = Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "{} {}",
                    correction,
                    command_line
                        .split_whitespace()
                        .skip(1)
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(" ")
                ))
                .status()?;

            exit(status.code().unwrap_or(1));
        } else if num == corrections.len() + 2 {
            // User wants to add a permanent alias
            print!("Enter the command for the alias: ");
            std::io::stdout().flush()?;

            let mut correction = String::new();
            std::io::stdin().read_line(&mut correction)?;
            let correction = correction.trim();

            if correction.is_empty() {
                println!("No command entered. Exiting.");
                exit(1);
            }

            process_add_permanent_alias(typed_command, correction)?;
            println!("Alias added successfully! 🐺");
            exit(0);
        } else if num == corrections.len() + 3 {
            // User wants to exit without running anything
            println!("Exiting without running any command.");
            exit(1);
        } else {
            println!("Invalid choice. Exiting.");
            exit(1);
        }
    } else if corrections.len() == 1 {
        // This block is no longer needed as it's redundant with our new empty choice handler
        // Keeping it for backward compatibility in case there are other conditions
        let correction = &corrections[0];
        cache.record_correction(typed_command, correction);
        cache.save()?;

        let status = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "{} {}",
                correction,
                command_line
                    .split_whitespace()
                    .skip(1)
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .join(" ")
            ))
            .status()?;

        exit(status.code().unwrap_or(1));
    } else {
        println!("Invalid choice. Exiting.");
        exit(1);
    }

    Ok(())
}

/// Process adding a permanent alias
fn process_add_permanent_alias(typed_command: &str, correction: &str) -> Result<()> {
    let (shell_type, config_path, alias_line) = super_snoofer::shell::detect_shell_config(typed_command, correction)?;
    super_snoofer::shell::add_to_shell_config(&shell_type, std::path::Path::new(&config_path), &alias_line)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => {
            install_shell_integration()?;
            println!("Shell integration installed successfully! 🐺");
            println!("Please restart your shell or source your shell configuration file.");
            exit(0);
        }
        Some(Commands::Uninstall) => {
            uninstall_shell_integration()?;
            println!("Shell integration uninstalled successfully! 🐺");
            println!("Please restart your shell or source your shell configuration file.");
            exit(0);
        }
        None => {
            // Initialize terminal
            let mut terminal = TerminalUI::new()?;
            
            // Initialize app state
            let ollama = OllamaClient::new();
            let mut app = TuiApp::new(ollama, cli.codestral);

            // If a prompt was provided, set it as the initial input
            if let Some(prompt) = cli.prompt {
                app.input = prompt;
                app.cursor_position = app.input.len();
            }

            // Event loop
            loop {
                // Draw the UI
                terminal.draw(|f| draw_ui(f, &app))?;

                // Handle input
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                app.submit_prompt().await?;
                            }
                            KeyCode::Char(c) => {
                                app.enter_char(c);
                            }
                            KeyCode::Backspace => {
                                app.delete_char();
                            }
                            KeyCode::Delete => {
                                app.delete_char_forward();
                            }
                            KeyCode::Left => {
                                app.move_cursor_left();
                            }
                            KeyCode::Right => {
                                app.move_cursor_right();
                            }
                            KeyCode::Esc => {
                                // Clean up and exit
                                drop(terminal);
                                std::process::exit(0);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
