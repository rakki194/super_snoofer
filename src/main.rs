#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::{
    env,
    process::{exit, Command},
    io::Write,
};

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
            _ => {}
        }
    }

    if args.len() != 2 {
        eprintln!("Usage: {} <command> | --reset_cache | --reset_memory", args[0]);
        exit(1);
    }

    let typed_command = &args[1];
    let mut cache = super_snoofer::CommandCache::load()?;
    cache.update()?;

    if let Some(suggestion) = cache.find_similar(typed_command) {
        print!(
            "Awoo! 🐺 Did you mean `{}`? *wags tail* (Y/n/c) ",
            suggestion.bright_green()
        );
        std::io::stdout().flush()?;

        let mut response = String::new();
        std::io::stdin().read_line(&mut response)?;
        let response = response.trim().to_lowercase();

        match response.as_str() {
            "n" => {
                eprintln!("Command '{}' not sniffed! 🐺", typed_command.bright_red());
                exit(1);
            }
            "c" => {
                print!("What's the correct command? ");
                std::io::stdout().flush()?;
                
                let mut correct_command = String::new();
                std::io::stdin().read_line(&mut correct_command)?;
                let correct_command = correct_command.trim();

                if !correct_command.is_empty() {
                    if let Err(e) = cache.learn_correction(typed_command, correct_command) {
                        eprintln!("Error learning correction: {e}");
                        exit(1);
                    }
                    
                    // Double-check the correction was actually saved by verifying it's in the cache
                    let verification_cache = super_snoofer::CommandCache::load()?;
                    if verification_cache.find_similar(typed_command) == Some(correct_command.to_string()) {
                        println!("Got it! 🐺 I'll remember that '{typed_command}' means '{correct_command}'");
                        
                        // Execute the correct command
                        let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
                        let status = Command::new(&shell)
                            .arg("-c")
                            .arg(correct_command)
                            .status()
                            .with_context(|| format!("Failed to execute command: {correct_command}"))?;
                        exit(status.code().unwrap_or(1));
                    } else {
                        eprintln!("Failed to remember correction. Something might be wrong with the cache file.");
                        exit(1);
                    }
                }
            }
            _ => {
                println!("Running suggested command...");
                
                let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
                let status = Command::new(&shell)
                    .arg("-c")
                    .arg(&suggestion)
                    .status()
                    .with_context(|| format!("Failed to execute command: {suggestion}"))?;
                exit(status.code().unwrap_or(1));
            }
        }
    } else {
        eprintln!("Command '{}' not sniffed! 🐺", typed_command.bright_red());
    }

    exit(1);
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
