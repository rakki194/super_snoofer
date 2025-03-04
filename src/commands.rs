#![warn(clippy::all, clippy::pedantic)]

use anyhow::{anyhow, Result};
use std::{io::Write, process::Command};
use crate::{CommandCache, HistoryTracker};

/// Learns a correction for a typo
/// 
/// # Errors
/// Returns an error if saving the correction to the database fails
pub fn learn_correction(typo: &str, command: &str) -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.learn_correction(typo, command)?;
    println!("Got it! 🐺 I'll remember that '{typo}' means '{command}'");
    cache.save()?;
    Ok(())
}

/// Resets the command cache
/// 
/// # Errors
/// Returns an error if the cache file cannot be deleted
pub fn reset_cache() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_cache();
    cache.save()?;
    Ok(())
}

/// Resets the memory database
/// 
/// # Errors
/// Returns an error if the memory database cannot be reset
pub fn reset_memory() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_memory();
    cache.save()?;
    Ok(())
}

/// Shows the command history
/// 
/// # Errors
/// Returns an error if the history file cannot be read or parsed
pub fn show_history() -> Result<()> {
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

/// Shows the most frequent typos
/// 
/// # Errors
/// Returns an error if the typo data cannot be retrieved or processed
pub fn show_frequent_typos() -> Result<()> {
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

/// Shows the most frequent corrections
/// 
/// # Errors
/// Returns an error if the correction data cannot be retrieved or processed
pub fn show_frequent_corrections() -> Result<()> {
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

/// Clears the command history
/// 
/// # Errors
/// Returns an error if the history file cannot be cleared
pub fn clear_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_history();
    cache.save()?;
    Ok(())
}

/// Enables tracking of command history
/// 
/// # Errors
/// Returns an error if the history settings cannot be updated
pub fn enable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.enable_history()?;
    cache.save()?;
    Ok(())
}

/// Disables tracking of command history
/// 
/// # Errors
/// Returns an error if the history settings cannot be updated
pub fn disable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.disable_history()?;
    cache.save()?;
    Ok(())
}

/// Checks a command line for potential corrections
/// 
/// # Errors
/// Returns an error if the command line cannot be processed or suggestions cannot be generated
pub fn check_command_line(command: &str) -> Result<()> {
    let cache = CommandCache::load()?;
    if let Some(correction) = cache.fix_command_line(command).map(|s| s.to_string()) {
        println!("Awoo! 🐺 Did you mean `{correction}`? *wags tail* (Y/n/c)");
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
            _ => println!("Command '{command}' not found! 🐺")
        }
    }
    Ok(())
}

/// Processes a full command line
/// 
/// # Errors
/// Returns an error if the command cannot be processed or if there are issues with the command execution
pub fn process_full_command(command: &str) -> Result<()> {
    // Execute the command through the shell to ensure PATH is used
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", command])
            .status()?
    } else {
        Command::new("sh")
            .args(["-c", command])
            .status()?
    };
    
    if !status.success() {
        return Err(anyhow!("Command failed with status: {}", status));
    }
    Ok(())
} 