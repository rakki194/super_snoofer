use anyhow::Result;
use colored::Colorize;
use rand::Rng;
use std::io::Write;
use crate::{HistoryTracker, shell::{detect_shell_config, add_to_shell_config}};

/// Generate a personalized alias suggestion based on command history
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
    let correction =
        if let Some(correction_for_typo) = cache.find_similar_with_frequency(selected_typo) {
            correction_for_typo
        } else {
            println!(
                "🐺 Couldn't find a correction for '{}'. This is unexpected!",
                selected_typo
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
            "You've mistyped '{}' {} times! Let's create an alias for that.",
            selected_typo, count
        ),
        format!(
            "Awoo! 🐺 I noticed you typed '{}' when you meant '{}' {} times!",
            selected_typo, correction, count
        ),
        format!(
            "Good Boy Tip: Create an alias for '{}' to avoid typing '{}' again! You've done it {} times!",
            correction, selected_typo, count
        ),
        format!(
            "*friendly growl* 🐺 '{}' is one of your most common typos. Let me help with that!",
            selected_typo
        ),
        format!(
            "You might benefit from an alias for '{}' since you've typed '{}' {} times!",
            correction, selected_typo, count
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
        "\nWould you like me to add this alias to your {} shell configuration? (y/N) ",
        shell_type
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