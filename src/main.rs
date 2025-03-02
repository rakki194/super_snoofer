#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use colored::Colorize;
use std::{
    env,
    io::Write,
    process::{Command, exit},
};

// Import modules for functionality
use super_snoofer::{
    CommandCache, 
    HistoryTracker,
    display,
    suggestion,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Handle command line flags
    if args.len() > 1 {
        match args[1].as_str() {
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
                exit(0);
            }
            "--enable-history" => {
                let mut cache = CommandCache::load()?;
                cache.enable_history()?;
                println!("Command history tracking is now enabled! 🐺");
                exit(0);
            }
            "--disable-history" => {
                let mut cache = CommandCache::load()?;
                cache.disable_history()?;
                println!("Command history tracking is now disabled! 🐺");
                exit(0);
            }
            "--suggest" => {
                suggestion::suggest_alias_command()?;
                exit(0);
            }
            _ => {}
        }
    }

    if args.len() != 2 {
        eprintln!(
            "Usage: {} <command> | --reset_cache | --reset_memory | --history | --frequent-typos | --frequent-corrections | --clear-history | --enable-history | --disable-history | --suggest",
            args[0]
        );
        exit(1);
    }

    let typed_command = &args[1];
    let mut cache = CommandCache::load()?;
    cache.update()?;

    // Get the full command line (including any arguments)
    let command_line = env::args().skip(1).collect::<Vec<String>>().join(" ");

    // First, try to correct the full command line for well-known commands
    if let Some(suggestion) = cache.fix_command_line(&command_line) {
        // Get frequency info if available for the base command
        let base_suggestion = suggestion.split_whitespace().next().unwrap_or(&suggestion);
        let frequency_info = if let Some(count) = cache
            .get_frequent_corrections(100)
            .into_iter()
            .find(|(cmd, _)| cmd == &base_suggestion)
            .map(|(_, count)| count)
        {
            if count > 0 {
                format!(" (used {count} times)")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Check if the suggestion is a shell alias
        if let Some(alias_target) = cache.get_alias_target(base_suggestion) {
            print!(
                "Awoo! 🐺 Did you mean `{}`{} (alias for `{}`)? *wags tail* (Y/n/c) ",
                suggestion.bright_green(),
                frequency_info,
                alias_target.bright_blue()
            );
        } else {
            print!(
                "Awoo! 🐺 Did you mean `{}`{}? *wags tail* (Y/n/c) ",
                suggestion.bright_green(),
                frequency_info
            );
        }
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        match response.as_str() {
            "" | "y" | "yes" => {
                // Record the correction in history
                cache.record_correction(&command_line, &suggestion);

                println!("Running suggested command...");

                let status = Command::new("sh").arg("-c").arg(&suggestion).status()?;

                exit(status.code().unwrap_or(1));
            }
            "c" | "correct" => {
                print!("What's the correct command? ");
                std::io::stdout().flush()?;

                let mut correction = String::new();
                std::io::stdin().read_line(&mut correction)?;
                let correction = correction.trim();

                if correction.is_empty() {
                    println!("No correction provided. Exiting.");
                    exit(1);
                }

                let base_correction = correction.split_whitespace().next().unwrap_or(correction);
                if !cache.contains(base_correction) {
                    println!("Warning: '{base_correction}' is not a known command.");

                    print!("Continue anyway? (y/N) ");
                    std::io::stdout().flush()?;

                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm)?;

                    if !["y", "yes"].contains(&confirm.trim().to_lowercase().as_str()) {
                        println!("Aborting correction.");
                        exit(1);
                    }
                }

                // Record the manual correction in history
                cache.record_correction(&command_line, correction);

                // Also learn the base command correction
                let base_typed = typed_command
                    .split_whitespace()
                    .next()
                    .unwrap_or(typed_command);
                cache.learn_correction(base_typed, base_correction)?;

                println!(
                    "Got it! 🐺 I'll remember that '{base_typed}' means '{base_correction}'"
                );

                let status = Command::new("sh").arg("-c").arg(correction).status()?;

                exit(status.code().unwrap_or(1));
            }
            _ => {
                println!("Command '{typed_command}' not found! 🐺");
                exit(127); // Standard "command not found" exit code
            }
        }
    } else if let Some(suggestion) = cache.find_similar_with_frequency(typed_command) {
        // Fallback to the original behavior for single command matching
        // Get frequency info if available
        let frequency_info = if let Some(count) = cache
            .get_frequent_corrections(100)
            .into_iter()
            .find(|(cmd, _)| cmd == &suggestion)
            .map(|(_, count)| count)
        {
            if count > 0 {
                format!(" (used {count} times)")
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Check if the suggestion is a shell alias
        if let Some(alias_target) = cache.get_alias_target(&suggestion) {
            print!(
                "Awoo! 🐺 Did you mean `{}`{} (alias for `{}`)? *wags tail* (Y/n/c) ",
                suggestion.bright_green(),
                frequency_info,
                alias_target.bright_blue()
            );
        } else {
            print!(
                "Awoo! 🐺 Did you mean `{}`{}? *wags tail* (Y/n/c) ",
                suggestion.bright_green(),
                frequency_info
            );
        }
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        match response.as_str() {
            "" | "y" | "yes" => {
                // Record the correction in history
                cache.record_correction(typed_command, &suggestion);

                println!("Running suggested command...");

                let status = Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "{} {}",
                        suggestion,
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
            "c" | "correct" => {
                print!("What's the correct command? ");
                std::io::stdout().flush()?;

                let mut correction = String::new();
                std::io::stdin().read_line(&mut correction)?;
                let correction = correction.trim();

                if correction.is_empty() {
                    println!("No correction provided. Exiting.");
                    exit(1);
                }

                if !cache.contains(correction) {
                    println!("Warning: '{correction}' is not a known command.");

                    print!("Continue anyway? (y/N) ");
                    std::io::stdout().flush()?;

                    let mut confirm = String::new();
                    std::io::stdin().read_line(&mut confirm)?;

                    if !["y", "yes"].contains(&confirm.trim().to_lowercase().as_str()) {
                        println!("Aborting correction.");
                        exit(1);
                    }
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
            }
            _ => {
                println!("Command '{typed_command}' not found! 🐺");
                exit(127); // Standard "command not found" exit code
            }
        }
    } else {
        println!("Command '{typed_command}' not found! 🐺");
        exit(127); // Standard "command not found" exit code
    }
}
