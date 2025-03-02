#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use colored::Colorize;
use std::{
    env,
    process::{exit, Command},
    io::Write,
};
use rand::Rng;

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
                // Check if history is enabled
                if !cache.is_history_enabled() {
                    println!("🐺 Command history tracking is currently disabled.");
                    println!("To enable it, run: super_snoofer --enable-history");
                    exit(0);
                }
                cache.clear_history();
                println!("Command history cleared successfully! 🐺");
                exit(0);
            }
            "--enable-history" => {
                let mut cache = super_snoofer::CommandCache::load()?;
                cache.enable_history()?;
                println!("Command history tracking is now enabled! 🐺");
                exit(0);
            }
            "--disable-history" => {
                let mut cache = super_snoofer::CommandCache::load()?;
                cache.disable_history()?;
                println!("Command history tracking is now disabled! 🐺");
                exit(0);
            }
            "--suggest" => {
                generate_alias_suggestion()?;
                exit(0);
            }
            _ => {}
        }
    }

    if args.len() != 2 {
        eprintln!("Usage: {} <command> | --reset_cache | --reset_memory | --history | --frequent-typos | --frequent-corrections | --clear-history | --enable-history | --disable-history | --suggest", args[0]);
        exit(1);
    }

    let typed_command = &args[1];
    let mut cache = super_snoofer::CommandCache::load()?;
    cache.update()?;
    
    // Get the full command line (including any arguments)
    let command_line = env::args().skip(1).collect::<Vec<String>>().join(" ");
    
    // First, try to correct the full command line for well-known commands
    if let Some(suggestion) = cache.fix_command_line(&command_line) {
        // Get frequency info if available for the base command
        let base_suggestion = suggestion.split_whitespace().next().unwrap_or(&suggestion);
        let frequency_info = if let Some(count) = cache.correction_frequency.get(base_suggestion) {
            if *count > 0 {
                format!(" (used {} times)", count)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Check if the suggestion is a shell alias
        if let Some(alias_target) = cache.get_alias_target(base_suggestion) {
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
                cache.record_correction(&command_line, &suggestion);

                println!("Running suggested command...");
                
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(&suggestion)
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
                
                let base_correction = correction.split_whitespace().next().unwrap_or(correction);
                if !cache.contains(base_correction) {
                    println!("Warning: '{}' is not a known command.", base_correction);
                    
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
                cache.record_correction(&command_line, correction);
                
                // Also learn the base command correction
                let base_typed = typed_command.split_whitespace().next().unwrap_or(typed_command);
                cache.learn_correction(base_typed, base_correction)?;
                
                println!("Got it! 🐺 I'll remember that '{}' means '{}'", base_typed, base_correction);
                
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(correction)
                    .status()?;
                
                exit(status.code().unwrap_or(1));
            }
            _ => {
                println!("Command '{}' not found! 🐺", typed_command);
                exit(127); // Standard "command not found" exit code
            }
        }
    } else if let Some(suggestion) = cache.find_similar_with_frequency(typed_command) {
        // Fallback to the original behavior for single command matching
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
                
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} {}", suggestion, command_line.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>().join(" ")))
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
                
                let status = Command::new("sh")
                    .arg("-c")
                    .arg(format!("{} {}", correction, command_line.split_whitespace().skip(1).map(String::from).collect::<Vec<String>>().join(" ")))
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
fn display_frequent_corrections() -> Result<()> {
    let cache = super_snoofer::CommandCache::load()?;
    
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

/// Generates a personalized alias suggestion based on command history
fn generate_alias_suggestion() -> Result<()> {
    // Load the cache
    let cache = super_snoofer::CommandCache::load()?;
    
    // Check if history is enabled
    if !cache.is_history_enabled() {
        println!("🐺 Command history tracking is currently disabled.");
        println!("To enable it, run: super_snoofer --enable-history");
        return Ok(());
    }
    
    // Get the most frequent typos
    let typos = cache.get_frequent_typos(100);
    
    if typos.is_empty() {
        println!("🐺 No typo history found yet. Try using Super Snoofer more to get personalized suggestions!");
        return Ok(());
    }
    
    // Pick a random typo from the top 5 (or all if less than 5)
    let mut rng = rand::thread_rng();
    let top_n = std::cmp::min(5, typos.len());
    let top_typos = &typos[0..top_n];
    let idx = rng.gen_range(0..top_n);
    let (selected_typo, count) = &top_typos[idx];
    
    // Get the correction for this typo
    let correction = if let Some(correction_for_typo) = cache.find_similar_with_frequency(selected_typo) {
        correction_for_typo
    } else {
        println!("🐺 Couldn't find a correction for '{}'. This is unexpected!", selected_typo);
        return Ok(());
    };
    
    // Generate the alias name
    let alias_name = if selected_typo.len() <= 3 {
        // For very short typos, use as is
        selected_typo.clone()
    } else {
        // For longer typos, use the first letter or first two letters
        if rng.gen_bool(0.5) {
            selected_typo[0..1].to_string()
        } else if selected_typo.len() >= 2 {
            selected_typo[0..2].to_string()
        } else {
            selected_typo[0..1].to_string()
        }
    };
    
    // Generate a personalized tip
    let tips = [
        format!("You've mistyped '{}' {} times! Let's create an alias for that.", selected_typo, count),
        format!("Awoo! 🐺 I noticed you typed '{}' when you meant '{}' {} times!", selected_typo, correction, count),
        format!("Good boy tip: Create an alias for '{}' to avoid typing '{}' again! You've done it {} times!", correction, selected_typo, count),
        format!("Woof! 🐺 '{}' is one of your most common typos. Let me help with that!", selected_typo),
        format!("Super Snoofer suggests: You might benefit from an alias for '{}' since you've typed '{}' {} times!", correction, selected_typo, count),
    ];
    
    let tip_idx = rng.gen_range(0..tips.len());
    println!("{}", tips[tip_idx].bright_cyan());
    
    // Make the alias suggestion
    println!("\nSuggested alias: {} → {}", alias_name.bright_green(), correction.bright_blue());
    
    // Create the alias command for different shells
    let bash_alias = format!("alias {}='{}'", alias_name, correction);
    let zsh_alias = bash_alias.clone();
    let fish_alias = format!("alias {} '{}'", alias_name, correction);
    
    // Print the shell-specific commands
    println!("\nTo add this alias to your shell configuration:");
    
    println!("\nBash (add to ~/.bashrc):");
    println!("{}", bash_alias.bright_yellow());
    
    println!("\nZsh (add to ~/.zshrc):");
    println!("{}", zsh_alias.bright_yellow());
    
    println!("\nFish (add to ~/.config/fish/config.fish):");
    println!("{}", fish_alias.bright_yellow());
    
    // Ask if user wants to automatically add the alias
    print!("\nWould you like me to add this alias to your shell configuration? (y/N) ");
    std::io::stdout().flush()?;
    
    let mut response = String::new();
    std::io::stdin().read_line(&mut response)?;
    let response = response.trim().to_lowercase();
    
    if response == "y" || response == "yes" {
        // Detect the current shell
        let shell = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
        
        if shell.contains("bash") {
            add_to_shell_config("bash", &bash_alias)?;
        } else if shell.contains("zsh") {
            add_to_shell_config("zsh", &zsh_alias)?;
        } else if shell.contains("fish") {
            add_to_shell_config("fish", &fish_alias)?;
        } else {
            println!("Unsupported shell: {}. Please add the alias manually.", shell);
        }
    } else {
        println!("No problem! You can add the alias manually whenever you're ready.");
    }
    
    Ok(())
}

/// Add an alias to the appropriate shell configuration file
fn add_to_shell_config(shell_type: &str, alias_line: &str) -> Result<()> {
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    
    let (config_path, success_message) = match shell_type {
        "bash" => {
            let path = home_dir.join(".bashrc");
            (path, "Added alias to ~/.bashrc! 🐺 Please run 'source ~/.bashrc' to use it.")
        }
        "zsh" => {
            let path = home_dir.join(".zshrc");
            (path, "Added alias to ~/.zshrc! 🐺 Please run 'source ~/.zshrc' to use it.")
        }
        "fish" => {
            let path = home_dir.join(".config").join("fish").join("config.fish");
            if !path.exists() {
                // Create fish config directory if it doesn't exist
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            (path, "Added alias to ~/.config/fish/config.fish! 🐺 Please restart your shell or run 'source ~/.config/fish/config.fish' to use it.")
        }
        _ => {
            return Err(anyhow::anyhow!("Unsupported shell type: {}", shell_type));
        }
    };
    
    // Append the alias to the config file
    let mut config = if config_path.exists() {
        std::fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&config_path)?
    } else {
        std::fs::File::create(&config_path)?
    };
    
    // Add a newline before the alias if the file doesn't end with one
    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        if !content.ends_with('\n') {
            writeln!(config)?;
        }
    }
    
    // Add a comment and the alias
    writeln!(config, "\n# Added by Super Snoofer")?;
    writeln!(config, "{}", alias_line)?;
    
    println!("{}", success_message.bright_green());
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

    #[test]
    fn test_suggest_functionality() -> Result<()> {
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
        
        // Set up environment variable to use our test cache
        {
            std::env::var_os("SUPER_SNOOFER_CACHE_PATH").map(|_| ());  // Just to check if it exists
            
            let cache_dir = temp_dir.path().join("cache");
            fs::create_dir_all(&cache_dir)?;
            let cache_file = cache_dir.join("super_snoofer_cache.json");
            
            // Use a safer approach with temporary directories instead of env vars
            let mut cache = super_snoofer::CommandCache::load_from_path(&cache_file)?;
            cache.clear_memory();
            cache.insert("git");
            cache.insert("docker");
            cache.insert("ls");
            
            // Add some history data
            for _ in 0..10 {
                cache.record_correction("gti", "git");
            }
            
            for _ in 0..5 {
                cache.record_correction("dcoker", "docker");
            }
            
            cache.save()?;
            
            // Load the cache and verify we have the expected data
            let cache = super_snoofer::CommandCache::load_from_path(&cache_file)?;
            
            // Verify we have typo frequency data
            let typos = cache.get_frequent_typos(10);
            assert!(!typos.is_empty(), "Should have typo frequency data");
            
            // Check for specific entries
            let has_gti = typos.iter().any(|(typo, _)| typo == "gti");
            assert!(has_gti, "Should have 'gti' in frequent typos");
            
            let has_docker = typos.iter().any(|(typo, _)| typo == "dcoker");
            assert!(has_docker, "Should have 'dcoker' in frequent typos");
        }
        
        Ok(())
    }
}
