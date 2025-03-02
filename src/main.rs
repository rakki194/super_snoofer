#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    io::Write,
    process::{Command, exit},
    fs::File,
    path::PathBuf,
};

// Import modules for functionality
use super_snoofer::{
    CommandCache, 
    HistoryTracker,
    display,
    suggestion,
};

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

/// Handle auto-completion commands
fn handle_completion_commands(command: &str, args: &[String]) -> Result<()> {
    match command {
        "--enable-completion" => {
            let mut cache = CommandCache::load()?;
            cache.enable_completion()?;
            println!("Auto-completion enabled! 🐺");
            println!("To use auto-completion, add this to your ~/.zshrc:");
            println!("source ~/.zsh_super_snoofer_completions");
            exit(0);
        }
        "--disable-completion" => {
            let mut cache = CommandCache::load()?;
            cache.disable_completion()?;
            println!("Auto-completion disabled! 🐺");
            exit(0);
        }
        "--export-completions" => {
            // Check if a path was provided
            let completion_path = if args.len() >= 3 {
                PathBuf::from(&args[2])
            } else {
                // Default path is current directory with a standard name
                PathBuf::from("super_snoofer_completions.zsh")
            };
            
            let cache = CommandCache::load()?;
            let completions = cache.command_patterns.generate_all_completions();
            
            // Write completions to the specified file
            let mut file = File::create(&completion_path)?;
            file.write_all(completions.as_bytes())?;
            
            println!("Completions exported to {}! 🐺", completion_path.display());
            exit(0);
        }
        _ => Ok(()),
    }
}

/// Handle shell integration commands
fn handle_shell_integration(command: &str, args: &[String]) -> Result<()> {
    if command == "--add-alias" && args.len() >= 3 {
        let alias_name = &args[2];
        let alias_command = if args.len() >= 4 {
            &args[3]
        } else {
            "super_snoofer"
        };

        let (shell_type, config_path, alias_line) = 
            super_snoofer::shell::detect_shell_config(alias_name, alias_command)?;
        
        super_snoofer::shell::add_to_shell_config(shell_type, &config_path, &alias_line)?;
        exit(0);
    }
    
    Ok(())
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

/// Handle command recording for history tracking
fn handle_record_correction(args: &[String]) -> Result<()> {
    if args.len() >= 4 && args[1] == "--record-correction" {
        let typo = &args[2];
        let correction = &args[3];
        
        let mut cache = CommandCache::load()?;
        
        // Check if history is enabled before recording
        if cache.is_history_enabled() {
            // Record the correction
            cache.record_correction(typo, correction);
            cache.save()?;
        }
        
        // Always exit quietly
        exit(0);
    }
    
    Ok(())
}

/// Handle command suggestions for real-time completion
fn handle_suggest_completion(args: &[String]) -> Result<()> {
    if args.len() >= 3 && args[1] == "--suggest-completion" {
        let partial_cmd = args[2..].join(" ");
        
        // Load the cache
        let cache = CommandCache::load()?;
        
        // Extract the base command (first word)
        let base_cmd = if let Some(cmd) = partial_cmd.split_whitespace().next() {
            cmd
        } else {
            // No command found, just return the original
            println!("{}", partial_cmd);
            exit(0);
        };
        
        // Check if this is a known command
        if cache.contains(base_cmd) {
            // See if we have command patterns with arguments/flags that can be suggested
            if let Some(suggestion) = cache.get_command_suggestion(&partial_cmd) {
                println!("{}", suggestion);
                exit(0);
            }
        }
        
        // Fallback to basic command correction if no specific suggestion
        if let Some(corrected) = cache.fix_command_line(&partial_cmd) {
            println!("{}", corrected);
        } else {
            // No suggestion found, return the original
            println!("{}", partial_cmd);
        }
        
        exit(0);
    }
    
    Ok(())
}

/// Handle recording valid commands for learning
fn handle_record_valid_command(args: &[String]) -> Result<()> {
    if args.len() >= 3 && args[1] == "--record-valid-command" {
        let command = &args[2];
        
        let mut cache = CommandCache::load()?;
        
        // Check if history is enabled before recording
        if cache.is_history_enabled() {
            // Record the valid command (this may need to be implemented in the cache)
            cache.record_valid_command(command);
            cache.save()?;
        }
        
        // Always exit quietly
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
        println!("  --help, -h                           Show this help message");
        println!("  --reset_cache                        Clear the command cache");
        println!("  --reset_memory                       Clear the cache and learned corrections");
        println!("  --history                            Show command history");
        println!("  --frequent-typos                     Show most common typos");
        println!("  --frequent-corrections               Show most used corrections");
        println!("  --clear-history                      Clear command history");
        println!("  --enable-history                     Enable command history tracking");
        println!("  --disable-history                    Disable command history tracking");
        println!("  --add-alias NAME [CMD]               Add shell alias (default: super_snoofer)");
        println!("  --suggest                            Suggest personalized shell aliases");
        println!("  --check-command CMD                  Check if a command has typos and output the corrected version");
        println!("  --record-correction TYPO CORRECTION  Record a correction for history (quiet)");
        println!("  --record-valid-command CMD           Record a valid command usage (quiet)");
        println!("  --suggest-completion CMD             Get real-time command suggestions (for shell integration)");
        println!("  --enable-completion                  Enable ZSH auto-completion for commands");
        println!("  --disable-completion                 Disable ZSH auto-completion");
        println!("  --export-completions [PATH]          Export completion script to a file");
        exit(0);
    }
}

/// Handle check command functionality
fn handle_check_command(args: &[String]) -> Result<()> {
    if args.len() >= 3 && args[1] == "--check-command" {
        let command_line = args[2..].join(" ");
        let cache = CommandCache::load()?;
        
        // Try to correct the command line
        if let Some(corrected) = cache.fix_command_line(&command_line) {
            // Just output the corrected command - no interactive prompts
            println!("{}", corrected);
            exit(0);
        } else {
            // If no correction is available, just echo back the original command
            println!("{}", command_line);
            exit(0);
        }
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
        let status = Command::new("sh")
            .arg("-c")
            .arg(command_line)
            .status()?;
        
        exit(status.code().unwrap_or(1));
    }
    
    // Command not found, suggest corrections
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
    println!("{}. {}", corrections.len() + 1, "Enter custom command".bright_yellow());
    
    // Add option to add permanent alias
    println!("{}. {}", corrections.len() + 2, "Add permanent shell alias".bright_blue());
    
    // Add option to exit without running anything
    println!("{}. {}", corrections.len() + 3, "Exit without running".bright_red());
    
    print!("Enter your choice (1-{}): ", corrections.len() + 3);
    std::io::stdout().flush()?;
    
    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;
    
    let choice = choice.trim();
    
    // Handle numeric choice
    if let Ok(num) = choice.parse::<usize>() {
        if num >= 1 && num <= corrections.len() {
            // User selected a suggested correction
            let correction = &corrections[num - 1];
            
            // Record the correction in history
            cache.record_correction(typed_command, correction);
            cache.save()?;
            
            // Try to correct the entire command line, not just the first part
            let full_command_line = if let Some(fixed_cmd_line) = cache.fix_command_line(command_line) {
                fixed_cmd_line
            } else {
                // If we can't correct the entire command line, just use the corrected command
                // with the original arguments
                format!(
                    "{} {}",
                    correction,
                    command_line
                        .split_whitespace()
                        .skip(1)
                        .map(String::from)
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            };
            
            let status = Command::new("sh")
                .arg("-c")
                .arg(full_command_line)
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
            println!(
                "Got it! 🐺 I'll remember that '{typed_command}' means '{correction}'"
            );
            
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
            process_add_permanent_alias(typed_command, cache)?;
        } else if num == corrections.len() + 3 {
            // User wants to exit without running anything
            println!("Exiting without running any command.");
            exit(1);
        } else {
            println!("Invalid choice. Exiting.");
            exit(1);
        }
    } else if corrections.len() == 1 {
        // If there's only one suggestion and user pressed enter, use it
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
fn process_add_permanent_alias(typed_command: &str, cache: &mut CommandCache) -> Result<()> {
    print!("Enter the correct command for the alias: ");
    std::io::stdout().flush()?;
    
    let mut correction = String::new();
    std::io::stdin().read_line(&mut correction)?;
    let correction = correction.trim();
    
    if correction.is_empty() {
        println!("No command entered. Exiting.");
        exit(1);
    }
    
    // Add alias to shell config
    let (shell_type, config_path, alias_line) = 
        super_snoofer::shell::detect_shell_config(typed_command, correction)?;
    
    super_snoofer::shell::add_to_shell_config(shell_type, &config_path, &alias_line)?;
    
    // Record the manual correction in history
    cache.record_correction(typed_command, correction);
    
    cache.learn_correction(typed_command, correction)?;
    println!(
        "Got it! 🐺 I'll remember that '{typed_command}' means '{correction}'"
    );
    
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} {}",
            correction,
            typed_command
                .split_whitespace()
                .skip(1)
                .map(String::from)
                .collect::<Vec<String>>()
                .join(" ")
        ))
        .status()?;
    
    exit(status.code().unwrap_or(1));
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle command line flags
    if args.len() > 1 {
        let command = &args[1];
        
        // Check for non-interactive commands early
        if command == "--check-command" {
            handle_check_command(&args)?;
        } else if command == "--record-correction" {
            handle_record_correction(&args)?;
        } else if command == "--record-valid-command" {
            handle_record_valid_command(&args)?;
        } else if command == "--suggest-completion" {
            handle_suggest_completion(&args)?;
        }
        
        // Try handling different types of commands
        handle_cache_commands(command)?;
        handle_history_commands(command)?;
        handle_history_tracking_commands(command)?;
        handle_completion_commands(command, &args)?;
        handle_shell_integration(command, &args)?;
        handle_suggestion_command(command)?;
        handle_help_command(command);
        
        // If we get here, it's an unrecognized command
        let typed_command = command;
        let command_line = env::args().skip(1).collect::<Vec<_>>().join(" ");
        
        process_command(typed_command, &command_line)?;
    } else {
        println!("Super Snoofer - Command correction utility 🐺");
        println!("Run 'super_snoofer --help' for usage information.");
        exit(0);
    }
    
    Ok(())
}
