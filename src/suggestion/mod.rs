use anyhow::Result;
use colored::Colorize;
use rand::Rng;
use std::io::Write;
use crate::{HistoryTracker, shell::{detect_shell_config, add_to_shell_config}};

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

    // Check if history is enabled
    if !cache.is_history_enabled() {
        println!("🐺 Command history tracking is currently disabled.");
        println!("To enable it, run: super_snoofer --enable-history");
        return Ok(());
    }

    // Get the most frequent typos
    let typos = cache.get_frequent_typos(100);

    if typos.is_empty() {
        println!(
            "🐺 No typo history found yet. Try using Super Snoofer more to get personalized suggestions!"
        );
        return Ok(());
    }

    // Pick a random typo from the top 5 (or all if less than 5)
    let mut rng = rand::rng();
    let top_n = std::cmp::min(5, typos.len());
    let top_typos = &typos[0..top_n];
    let idx = rng.random_range(0..top_n);
    let (selected_typo, count) = &top_typos[idx];

    // Get the correction for this typo
    let Some(correction) = cache.find_similar_with_frequency(selected_typo) else {
        println!(
            "🐺 Couldn't find a correction for '{selected_typo}'. This is unexpected!"
        );
        return Ok(());
    };

    // Generate the alias name
    let alias_name = if selected_typo.len() <= 3 {
        // For very short typos, use as is
        selected_typo.clone()
    } else {
        // For longer typos, use the first letter or first two letters
        if rng.random_bool(0.5) {
            selected_typo[0..1].to_string()
        } else if selected_typo.len() >= 2 {
            selected_typo[0..2].to_string()
        } else {
            selected_typo[0..1].to_string()
        }
    };

    // Generate a personalized tip
    let tips = [
        format!(
            "You've mistyped '{selected_typo}' {count} times! Let's create an alias for that."
        ),
        format!(
            "Awoo! 🐺 I noticed you typed '{selected_typo}' when you meant '{correction}' {count} times!"
        ),
        format!(
            "Good Boy Tip: Create an alias for '{correction}' to avoid typing '{selected_typo}' again! You've done it {count} times!"
        ),
        format!(
            "*friendly growl* 🐺 '{selected_typo}' is one of your most common typos. Let me help with that!"
        ),
        format!(
            "You might benefit from an alias for '{correction}' since you've typed '{selected_typo}' {count} times!"
        ),
    ];

    let tip_idx = rng.random_range(0..tips.len());
    println!("{}", tips[tip_idx].bright_cyan());

    // Make the alias suggestion
    println!(
        "\nSuggested alias: {} → {}",
        alias_name.bright_green(),
        correction.bright_blue()
    );

    // Detect the current shell
    let (shell_type, config_path, alias_line) = detect_shell_config(&alias_name, &correction)?;

    // Print the shell-specific command for the detected shell only
    println!("\nTo add this alias to your shell configuration:");
    println!("\n{}", alias_line.bright_yellow());

    // Ask if user wants to automatically add the alias
    print!(
        "\nWould you like me to add this alias to your {shell_type} shell configuration? (y/N) "
    );
    std::io::stdout().flush()?;

    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();

    if response == "y" || response == "yes" {
        add_to_shell_config(shell_type, &config_path, &alias_line)?;
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
#[must_use] pub fn get_command_suggestions(command: &str, cache: &crate::CommandCache) -> Vec<String> {
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