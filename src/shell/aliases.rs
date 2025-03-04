#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use crate::{CommandCache, HistoryTracker};

/// Add a shell alias
pub fn add_alias(name: &str, command: Option<&str>) -> Result<()> {
    let command = command.unwrap_or("super_snoofer");
    let (shell_type, config_path, alias_line) = detect_shell_config(name, command)?;
    add_to_shell_config(&shell_type, std::path::Path::new(&config_path), &alias_line)?;
    Ok(())
}

/// Suggest personalized shell aliases
pub fn suggest_aliases() -> Result<()> {
    let cache = CommandCache::load()?;
    if !cache.is_history_enabled() {
        println!("Command history tracking is disabled! Cannot generate suggestions. 🐺");
        return Ok(());
    }

    let corrections = cache.get_frequent_corrections(5);
    if corrections.is_empty() {
        println!("No alias suggestions available yet! Keep using Super Snoofer to generate personalized suggestions. 🐺");
        return Ok(());
    }

    for (_i, (command, count)) in corrections.iter().enumerate() {
        let alias = if command.len() <= 3 {
            command.to_string()
        } else {
            command[0..2].to_string()
        };

        println!("\nYou've used '{}' {} times! Let's create an alias for that.", command, count);
        println!("\nSuggested alias: {} → {}", alias, command);
        println!("\nTo add this alias to your shell configuration:");
        println!("\nalias {}='{}'", alias, command);
        
        print!("\nWould you like me to add this alias to your shell configuration? (y/N) ");
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if input.trim().eq_ignore_ascii_case("y") {
            add_alias(&alias, Some(command))?;
            println!("Added alias to your shell configuration! 🐺");
        }
    }
    Ok(())
}

/// Parse shell aliases from various shell config files
pub fn parse_shell_aliases() -> Result<HashMap<String, String>> {
    let mut aliases = HashMap::new();
    
    // Get home directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    // Parse .zshrc and related files
    let zsh_files = vec![
        home_dir.join(".zshrc"),
        home_dir.join("toolkit/zsh/core_shell.zsh"),
        home_dir.join("toolkit/zsh/docker.zsh"),
        home_dir.join("toolkit/zsh/git.zsh"),
        home_dir.join("toolkit/zsh/personal.zsh"),
    ];

    for file_path in zsh_files {
        if file_path.exists() {
            if let Ok(()) = parse_aliases_from_file(&file_path, &mut aliases) {
                // Successfully parsed aliases from this file
            }
        }
    }

    Ok(aliases)
}

/// Detect shell config file and generate alias line
pub fn detect_shell_config(alias_name: &str, command: &str) -> Result<(String, String, String)> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let zshrc_path = home_dir.join(".zshrc");
    if zshrc_path.exists() {
        let alias_line = format!("alias {}='{}'", alias_name, command);
        return Ok(("zsh".to_string(), zshrc_path.to_string_lossy().into(), alias_line));
    }
    Err(anyhow::anyhow!("No supported shell config found"))
}

/// Add configuration to shell config file
pub fn add_to_shell_config(_shell_type: &str, config_path: &Path, config: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(config_path)?;
    writeln!(file, "\n{}", config)?;
    Ok(())
}

fn parse_aliases_from_file(file_path: &PathBuf, aliases: &mut HashMap<String, String>) -> Result<()> {
    let content = fs::read_to_string(file_path)?;

    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse alias definitions
        if line.starts_with("alias ") {
            if let Some((name, command)) = parse_alias_line(line) {
                aliases.insert(name, command);
            }
        }
    }

    Ok(())
}

fn parse_alias_line(line: &str) -> Option<(String, String)> {
    let line = line.trim_start_matches("alias ").trim();
    if let Some((name, command)) = line.split_once('=') {
        let name = name.trim();
        let mut command = command.trim();
        
        // Remove surrounding quotes if present
        if (command.starts_with('\'') && command.ends_with('\'')) || 
           (command.starts_with('"') && command.ends_with('"')) {
            command = &command[1..command.len() - 1];
        }
        
        Some((name.to_string(), command.to_string()))
    } else {
        None
    }
}
