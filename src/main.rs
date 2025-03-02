#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::{
    env,
    process::{exit, Command},
    io::Write,
};

const HISTORY_DISPLAY_LIMIT: usize = 20; // Default number of history entries to display

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Handle command line flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--reset_cache" => {
                let mut cache = super_snoofer::CommandCache::load()?;
                cache.clear_cache();
                cache.save()?;
                println!("Cache cleared successfully! 🐺");
                exit(0);
            }
            "--reset_memory" => {
                let mut cache = super_snoofer::CommandCache::load()?;
                cache.clear_memory();
                cache.save()?;
                println!("Cache and learned corrections cleared successfully! 🐺");
                exit(0);
            }
            "--history" => {
                display_command_history()?;
                exit(0);
            }
            "--frequent-typos" => {
                display_frequent_typos()?;
                exit(0);
            }
            "--frequent-corrections" => {
                display_frequent_corrections()?;
                exit(0);
            }
            "--clear-history" => {
                let mut cache = super_snoofer::CommandCache::load()?;
                cache.clear_history();
                println!("Command history cleared successfully! 🐺");
                exit(0);
            }
            _ => {}
        }
    }

    if args.len() != 2 {
        eprintln!("Usage: {} <command> | --reset_cache | --reset_memory | --history | --frequent-typos | --frequent-corrections | --clear-history", args[0]);
        exit(1);
    }

    let typed_command = &args[1];
    let mut cache = super_snoofer::CommandCache::load()?;
    cache.update()?;

    // Use frequency-aware suggestion function
    if let Some(suggestion) = cache.find_similar_with_frequency(typed_command) {
        // Get frequency info if available
        let frequency_info = if let Some(count) = cache.correction_frequency.get(&suggestion) {
            if *count > 0 {
                format!(" (used {} times)", count)
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
                
                // Get any arguments after the first one from the original command
                let args_from_command: Vec<String> = env::args().skip(2).collect();
                
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} {}", suggestion, args_from_command.join(" ")))
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
                    println!("Warning: '{}' is not a known command.", correction);
                    
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
                println!("Got it! 🐺 I'll remember that '{}' means '{}'", typed_command, correction);
                
                let args_from_command: Vec<String> = env::args().skip(2).collect();
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} {}", correction, args_from_command.join(" ")))
                    .status()?;
                
                exit(status.code().unwrap_or(1));
            }
            _ => {
                println!("Command '{}' not found! 🐺", typed_command);
                exit(127); // Standard "command not found" exit code
            }
        }
    } else {
        println!("Command '{}' not found! 🐺", typed_command);
        exit(127); // Standard "command not found" exit code
    }
}

/// Display command correction history
fn display_command_history() -> Result<()> {
    let cache = super_snoofer::CommandCache::load()?;
    let history = cache.get_command_history(HISTORY_DISPLAY_LIMIT);
    
    if history.is_empty() {
        println!("🐺 No command history found yet.");
        return Ok(());
    }
    
    println!("🐺 Your recent command corrections:");
    for (i, entry) in history.iter().enumerate() {
        println!("{}. {} → {} ({})", 
            i + 1, 
            entry.typo.bright_red(), 
            entry.correction.bright_green(),
            humantime::format_rfc3339(entry.timestamp)
        );
    }
    
    Ok(())
}

/// Display most frequent typos
fn display_frequent_typos() -> Result<()> {
    let cache = super_snoofer::CommandCache::load()?;
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
fn display_frequent_corrections() -> Result<()> {
    let cache = super_snoofer::CommandCache::load()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::{fs, sync::Once};

    // Setup logging for tests
    static INIT: Once = Once::new();
    fn setup_logging() {
        INIT.call_once(|| {
            env_logger::builder().is_test(true).init();
        });
    }

    #[test]
    fn test_command_execution() -> Result<()> {
        // Create a temporary directory for our test script
        let temp_dir = TempDir::new()?;
        let script_path = temp_dir.path().join("test_script.sh");
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // Write a simple shell script that echoes a test message
            std::fs::write(&script_path, "#!/bin/sh\necho 'test command executed'")?;
            
            // Make the script executable
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }

        // Test actual command execution
        let status = Command::new(&script_path)
            .status()
            .with_context(|| format!("Failed to execute test command: {}", script_path.display()))?;
        
        assert!(status.success());

        // Keep temp_dir in scope until the end of the test
        let _ = &temp_dir;
        
        Ok(())
    }

    // Helper function that correctly sets up the test cache environment
    #[test]
    fn test_reset_cache_flag() -> Result<()> {
        setup_logging();
        
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        
        // Ensure parent directory exists with proper error handling
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {parent:?}"))?;
        }
        
        // Keep a strong reference to temp_dir to prevent premature cleanup
        let _temp_dir_guard = &temp_dir;
        
        // Initialize a fresh cache - use load_from_path directly instead of relying on env vars
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_memory();
            cache.insert("cargo");
            cache.insert("git");
            cache.insert("python");
            
            // Add a learned correction
            let typo = "clippy";
            let correction = "cargo clippy";
            cache.learn_correction(typo, correction)?;
            cache.save()?;
            
            // Verify it was saved correctly
            assert_eq!(cache.find_similar(typo), Some(correction.to_string()), 
                       "Correction not properly saved before testing reset");
        }
        
        // Load a fresh instance to verify the correction exists
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert_eq!(cache.find_similar("clippy"), Some("cargo clippy".to_string()),
                       "Correction not found before resetting cache");
        }
        
        // Emulate the --reset_cache command line flag
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_cache();
            cache.save()?;
        }
        
        // Load a fresh instance to verify cache is cleared but corrections remain
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            // We can't check commands directly, but we can check that clippy still works
            assert_eq!(cache.find_similar("clippy"), Some("cargo clippy".to_string()),
                       "Correction was lost after resetting cache");
        }
        
        Ok(())
    }

    #[test]
    fn test_reset_memory_flag() -> Result<()> {
        setup_logging();
        
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        
        // Explicitly ensure parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {parent:?}"))?;
        }
        
        // Keep a strong reference to temp_dir to prevent premature cleanup
        let _temp_dir_guard = &temp_dir;
        
        // Initialize a fresh cache - use load_from_path directly
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_memory();
            cache.insert("cargo");
            cache.insert("git");
            cache.insert("python");
            
            // Add a learned correction
            let typo = "clippy";
            let correction = "cargo clippy";
            cache.learn_correction(typo, correction)?;
            cache.save()?;
            
            // Verify it was saved correctly
            assert_eq!(cache.find_similar(typo), Some(correction.to_string()),
                       "Correction not properly saved before testing reset");
        }
        
        // Verify the correction exists
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert!(cache.has_correction("clippy"), "Correction for 'clippy' should exist before reset");
        }
        
        // Emulate the --reset_memory command line flag
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_memory();
            cache.save()?;
        }
        
        // Verify both cache and corrections are cleared
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert!(!cache.has_correction("clippy"), "Correction for 'clippy' should be cleared after reset");
        }
        
        Ok(())
    }

    #[test]
    fn test_composite_command_correction_integration() -> Result<()> {
        setup_logging();
        
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        
        // Explicitly ensure parent directory exists with proper error handling
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {parent:?}"))?;
        }
        
        // Keep a strong reference to temp_dir to prevent premature cleanup
        let _temp_dir_guard = &temp_dir;
        
        // Define typed_command at the function level so it's in scope for all blocks
        let typed_command = "clippy";
        let correct_command = "cargo clippy";
        
        // Initialize a fresh cache
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_memory();
            cache.insert("cargo");
            cache.insert("git");
            cache.insert("python");
            
            // Simulate learning a correction
            cache.learn_correction(typed_command, correct_command)?;
            cache.save()?;
            
            // Verify it was saved correctly 
            assert_eq!(cache.find_similar(typed_command), Some(correct_command.to_string()),
                      "Correction not properly saved initially");
        }
        
        // Verify the correction was saved properly
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert_eq!(
                cache.find_similar(typed_command),
                Some("cargo clippy".to_string()),
                "Correction was not properly saved for composite command"
            );
        }
        
        // Test that the correction persists across cache updates
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.save()?;
        }
        
        // Verify correction persisted after update
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert_eq!(
                cache.find_similar(typed_command),
                Some("cargo clippy".to_string()),
                "Correction did not persist after cache update"
            );
        }
        
        Ok(())
    }
    
    #[test]
    fn test_correction_verification() -> Result<()> {
        setup_logging();
        
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        
        // Explicitly ensure parent directory exists and handle potential errors
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {parent:?}"))?;
        }
        
        // Keep a strong reference to temp_dir to prevent premature cleanup
        let _temp_dir_guard = &temp_dir;
        
        // Initialize a fresh cache
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            cache.clear_memory();
            cache.insert("cargo");
            cache.insert("git");
            cache.insert("python");
            
            // Learn a correction
            let typed_command = "gs";
            let correct_command = "git status";
            
            cache.learn_correction(typed_command, correct_command)?;
            cache.save()?;
            
            // Verify it was saved correctly
            assert_eq!(cache.find_similar(typed_command), Some(correct_command.to_string()),
                       "Correction not properly saved initially");
        }
        
        // Verify correction was learned and can be found
        {
            let cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            assert_eq!(
                cache.find_similar("gs"),
                Some("git status".to_string()),
                "Verification failed to find the learned correction"
            );
        }
        
        // Test invalid command handling
        {
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_path)?;
            let result = cache.learn_correction("test", "nonexistent_command");
            assert!(result.is_ok(), "Learning invalid command should not fail");
        }
        
        Ok(())
    }
}
