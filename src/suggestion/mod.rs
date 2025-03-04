#![warn(clippy::all, clippy::pedantic)]

use crate::{
    shell::{add_to_shell_config, detect_shell_config},
    HistoryTracker,
};
use anyhow::Result;
use colored::Colorize;
use std::io::Write;
use std::path::Path;

/// Generate a personalized alias suggestion based on command history
///
/// # Returns
///
/// A `Result` indicating success or failure
///
/// # Errors
///
/// This function will return an error if:
/// - The command cache cannot be loaded
/// - Shell configuration files cannot be detected
/// - There is an error when adding aliases to shell configuration
/// - There is an error reading user input
pub fn suggest_alias_command() -> Result<()> {
    // Load the cache
    let cache = crate::CommandCache::load()?;

    // Get the most frequent corrections
    let corrections = cache.get_frequent_corrections(10);
    if corrections.is_empty() {
        println!(
            "🐺 No command history found yet. Try using Super Snoofer more to get personalized suggestions!"
        );
        return Ok(());
    }

    // Pick the most frequent correction
    let (command, frequency) = &corrections[0];

    // Generate an alias suggestion
    let alias_name = if command.len() <= 3 {
        command.to_string()
    } else {
        command[0..2].to_string()
    };

    // Generate a personalized tip
    println!("🐺 *friendly growl* I noticed you use '{}' frequently! ({}x)", command.bright_cyan(), frequency);
    println!(
        "\nSuggested alias: {} → {}",
        alias_name.bright_green(),
        command.bright_blue()
    );

    // Ask if user wants to add the alias
    print!("\nWould you like to add this alias? (y/N) ");
    std::io::stdout().flush()?;

    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();

    if response == "y" || response == "yes" {
        // First detect the shell config
        let (shell_type, config_path, alias_line) = detect_shell_config(&alias_name, command)?;
        // Then add the alias to the config
        add_to_shell_config(&shell_type, Path::new(&config_path), &alias_line)?;
        println!("✨ Alias added successfully!");
    } else {
        println!("No problem! You can add the alias manually whenever you're ready.");
    }

    Ok(())
}

/// Get command suggestions for a possibly misspelled command
///
/// # Arguments
///
/// * `command` - The potentially misspelled command
/// * `cache` - The command cache to search through
///
/// # Returns
///
/// A vector of suggested commands that are similar to the input command
#[must_use]
pub fn get_command_suggestions(command: &str, cache: &crate::CommandCache) -> Vec<String> {
    let mut suggestions = Vec::new();

    // First check if we have a learned correction
    if let Some(correction) = cache.find_similar_with_frequency(command) {
        suggestions.push(correction);
    }

    // Then look for aliases and similar commands
    if let Some(correction) = cache.get_closest_match(command, crate::cache::SIMILARITY_THRESHOLD) {
        if !suggestions.contains(&correction) {
            suggestions.push(correction);
        }
    }

    suggestions
}
