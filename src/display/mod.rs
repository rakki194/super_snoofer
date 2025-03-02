#![warn(clippy::all, clippy::pedantic)]

use crate::HistoryTracker;
use anyhow::Result;
use chrono::{DateTime, Local};
use colored::Colorize;
use std::time::SystemTime;

/// Default number of history entries to display
pub const HISTORY_DISPLAY_LIMIT: usize = 20;

/// Format a system time as a human-readable local datetime
fn format_time(system_time: SystemTime) -> String {
    let datetime: DateTime<Local> = system_time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Display command correction history
///
/// # Errors
/// Returns an error if the command cache cannot be loaded
pub fn display_command_history() -> Result<()> {
    let cache = crate::CommandCache::load()?;

    // Check if history is enabled
    if !cache.is_history_enabled() {
        println!("🐺 Command history tracking is currently disabled.");
        println!("To enable it, run: super_snoofer --enable-history");
        return Ok(());
    }

    let history = cache.get_command_history(HISTORY_DISPLAY_LIMIT);

    if history.is_empty() {
        println!("🐺 No command history found yet.");
        println!("History will be recorded when you use Super Snoofer to correct commands.");
        return Ok(());
    }

    println!("{}", "🐺 Your recent command corrections:".bold());
    println!("{}", "─".repeat(80));

    // Print a formatted header
    println!(
        "{:<5} {:<20} {:<20} {:<30}",
        "#".bold(),
        "Typed".bold(),
        "Corrected To".bold(),
        "When".bold()
    );

    println!("{}", "─".repeat(80));

    for (i, entry) in history.iter().enumerate() {
        println!(
            "{:<5} {:<20} {:<20} {:<30}",
            (i + 1).to_string().bold(),
            entry.typo.bright_red(),
            entry.correction.bright_green(),
            format_time(entry.timestamp).dimmed()
        );
    }

    println!("{}", "─".repeat(80));
    println!(
        "{} commands shown. Total history: {} entries.",
        history.len(),
        cache.get_history_size()
    );

    println!("\nTo view more history information:");
    println!(
        "  {} - Show frequent typos",
        "super_snoofer --frequent-typos".bright_yellow()
    );
    println!(
        "  {} - Show frequently used corrections",
        "super_snoofer --frequent-corrections".bright_yellow()
    );
    println!(
        "  {} - Clear history",
        "super_snoofer --clear-history".bright_yellow()
    );

    Ok(())
}

/// Display most frequent typos
///
/// # Errors
/// Returns an error if the command cache cannot be loaded
pub fn display_frequent_typos() -> Result<()> {
    let cache = crate::CommandCache::load()?;

    // Check if history is enabled
    if !cache.is_history_enabled() {
        println!("🐺 Command history tracking is currently disabled.");
        println!("To enable it, run: super_snoofer --enable-history");
        return Ok(());
    }

    let typos = cache.get_frequent_typos(HISTORY_DISPLAY_LIMIT);

    if typos.is_empty() {
        println!("🐺 No typo history found yet.");
        return Ok(());
    }

    println!("🐺 Your most common typos:");
    for (i, (typo, count)) in typos.iter().enumerate() {
        println!("{}. {} ({} times)", i + 1, typo.bright_red(), count);
    }

    Ok(())
}

/// Display most frequent corrections
///
/// # Errors
/// Returns an error if the command cache cannot be loaded
pub fn display_frequent_corrections() -> Result<()> {
    let cache = crate::CommandCache::load()?;

    // Check if history is enabled
    if !cache.is_history_enabled() {
        println!("🐺 Command history tracking is currently disabled.");
        println!("To enable it, run: super_snoofer --enable-history");
        return Ok(());
    }

    let corrections = cache.get_frequent_corrections(HISTORY_DISPLAY_LIMIT);

    if corrections.is_empty() {
        println!("🐺 No correction history found yet.");
        return Ok(());
    }

    println!("🐺 Your most frequently used corrections:");
    for (i, (correction, count)) in corrections.iter().enumerate() {
        println!("{}. {} ({} times)", i + 1, correction.bright_green(), count);
    }

    Ok(())
}
