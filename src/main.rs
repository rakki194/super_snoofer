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
                    cache.learn_correction(typed_command, correct_command)?;
                    println!("Got it! 🐺 I'll remember that '{typed_command}' means '{correct_command}'");
                    
                    // Execute the correct command
                    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
                    let status = Command::new(&shell)
                        .arg("-c")
                        .arg(correct_command)
                        .status()
                        .with_context(|| format!("Failed to execute command: {correct_command}"))?;
                    exit(status.code().unwrap_or(1));
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

    #[test]
    fn test_command_execution() -> Result<()> {
        // This test remains in main.rs as it tests the command execution functionality
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

        Ok(())
    }
}
