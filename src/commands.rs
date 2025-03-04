#![warn(clippy::all, clippy::pedantic)]

use anyhow::{anyhow, Result};
use std::{io::Write, process::Command};
use crate::{CommandCache, HistoryTracker};

/// Learn a command correction
pub fn learn_correction(typo: &str, command: &str) -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.learn_correction(typo, command)?;
    println!("Got it! 🐺 I'll remember that '{typo}' means '{command}'");
    cache.save()?;
    Ok(())
}

/// Reset the command cache but keep learned corrections
pub fn reset_cache() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_cache();
    cache.save()?;
    Ok(())
}

/// Reset both the command cache and learned corrections
pub fn reset_memory() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_memory();
    cache.save()?;
    Ok(())
}

/// Show command history
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

/// Show most frequent typos
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

/// Show most frequently used corrections
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

/// Clear command history
pub fn clear_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.clear_history();
    cache.save()?;
    Ok(())
}

/// Enable command history tracking
pub fn enable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.enable_history()?;
    cache.save()?;
    Ok(())
}

/// Disable command history tracking
pub fn disable_history() -> Result<()> {
    let mut cache = CommandCache::load()?;
    cache.disable_history()?;
    cache.save()?;
    Ok(())
}

/// Check command line for corrections
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

/// Process a full command line
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