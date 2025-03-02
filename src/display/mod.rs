use anyhow::Result;
use colored::Colorize;
use crate::HistoryTracker;

/// Default number of history entries to display
pub const HISTORY_DISPLAY_LIMIT: usize = 20;

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
        return Ok(());
    }

    println!("🐺 Your recent command corrections:");
    for (i, entry) in history.iter().enumerate() {
        println!(
            "{}. {} → {} ({})",
            i + 1,
            entry.typo.bright_red(),
            entry.correction.bright_green(),
            humantime::format_rfc3339(entry.timestamp)
        );
    }

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