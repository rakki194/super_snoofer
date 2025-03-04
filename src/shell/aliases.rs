#![warn(clippy::all, clippy::pedantic)]

use fancy_regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::hash::BuildHasher;
use anyhow::{Context, Result};
use std::path::PathBuf;

/// Parse shell aliases from various shell config files
#[must_use]
pub fn parse_shell_aliases() -> Result<HashMap<String, String>> {
    let mut aliases = HashMap::new();

    // Try to parse aliases from different shell config files
    if let Some(bash_aliases) = parse_bash_aliases() {
        aliases.extend(bash_aliases);
    }

    if let Some(zsh_aliases) = parse_zsh_aliases() {
        aliases.extend(zsh_aliases);
    }

    if let Some(fish_aliases) = parse_fish_aliases() {
        aliases.extend(fish_aliases);
    }

    Ok(aliases)
}

/// Parse Bash aliases from .bashrc and .`bash_aliases`
fn parse_bash_aliases() -> Option<HashMap<String, String>> {
    let home = dirs::home_dir()?;
    let mut aliases = HashMap::new();

    // Check .bashrc
    let bashrc_path = home.join(".bashrc");
    if bashrc_path.exists() {
        if let Ok(content) = fs::read_to_string(&bashrc_path) {
            parse_bash_alias_content(&content, &mut aliases);
        }
    }

    // Check .bash_aliases
    let bash_aliases_path = home.join(".bash_aliases");
    if bash_aliases_path.exists() {
        if let Ok(content) = fs::read_to_string(&bash_aliases_path) {
            parse_bash_alias_content(&content, &mut aliases);
        }
    }

    Some(aliases)
}

/// Parse Bash/Zsh style alias definitions from content
pub fn parse_bash_alias_content<S: BuildHasher>(
    content: &str,
    aliases: &mut HashMap<String, String, S>,
) {
    // Regular expression for alias: alias name='command' or alias name="command"
    if let Ok(re) = Regex::new(r#"^\s*alias\s+([a-zA-Z0-9_-]+)=(['"])(.+?)\2"#) {
        for line in content.lines() {
            if let Ok(Some(caps)) = re.captures(line) {
                let name_result = caps.get(1);
                let cmd_result = caps.get(3);

                if let (Some(name_match), Some(cmd_match)) = (name_result, cmd_result) {
                    let name = name_match.as_str();
                    let cmd = cmd_match.as_str();
                    aliases.insert(name.to_string(), cmd.to_string());
                }
            }
        }
    }
}

/// Parse Zsh aliases from .zshrc
fn parse_zsh_aliases() -> Option<HashMap<String, String>> {
    let home = dirs::home_dir()?;
    let mut aliases = HashMap::new();

    // Parse .zshrc and related files
    let zsh_files = vec![
        home.join(".zshrc"),
        home.join("toolkit/zsh/core_shell.zsh"),
        home.join("toolkit/zsh/docker.zsh"),
        home.join("toolkit/zsh/git.zsh"),
        home.join("toolkit/zsh/personal.zsh"),
    ];

    for file_path in zsh_files {
        if file_path.exists() {
            if let Ok(()) = parse_aliases_from_file(&file_path, &mut aliases) {
                // Successfully parsed aliases from this file
            }
        }
    }

    // Check .zsh_aliases if it exists
    let zsh_aliases_path = home.join(".zsh_aliases");
    if zsh_aliases_path.exists() {
        if let Ok(content) = fs::read_to_string(&zsh_aliases_path) {
            parse_bash_alias_content(&content, &mut aliases);
        }
    }

    // Check .oh-my-zsh/custom/aliases.zsh if it exists
    let omz_path = home.join(".oh-my-zsh").join("custom").join("aliases.zsh");
    if omz_path.exists() {
        if let Ok(content) = fs::read_to_string(&omz_path) {
            parse_bash_alias_content(&content, &mut aliases);
        }
    }

    Some(aliases)
}

fn parse_aliases_from_file(file_path: &PathBuf, aliases: &mut HashMap<String, String>) -> Result<()> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    for line in content.lines() {
        let line = line.trim();
        
        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse alias definitions
        if line.starts_with("alias ") {
            parse_alias_line(line, aliases);
        }
    }

    Ok(())
}

fn parse_alias_line(line: &str, aliases: &mut HashMap<String, String>) {
    // Remove 'alias ' prefix
    let line = line.trim_start_matches("alias ").trim();
    
    // Split on first '=' to separate alias name and command
    if let Some((name, command)) = line.split_once('=') {
        let name = name.trim();
        let mut command = command.trim();
        
        // Remove surrounding quotes if present
        if (command.starts_with('\'') && command.ends_with('\'')) || 
           (command.starts_with('"') && command.ends_with('"')) {
            command = &command[1..command.len() - 1];
        }
        
        aliases.insert(name.to_string(), command.to_string());
    }
}

/// Parse Fish aliases from fish config
fn parse_fish_aliases() -> Option<HashMap<String, String>> {
    let home = dirs::home_dir()?;
    let mut aliases = HashMap::new();

    // Check fish config.fish
    let config_path = home.join(".config").join("fish").join("config.fish");
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            parse_fish_alias_content(&content, &mut aliases);
        }
    }

    // Check fish functions directory for alias functions
    let functions_dir = home.join(".config").join("fish").join("functions");
    if functions_dir.exists() && functions_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&functions_dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "fish") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        // Extract alias name from file name
                        if let Some(file_stem) = path.file_stem() {
                            if let Some(name) = file_stem.to_str() {
                                parse_fish_function_alias(&content, name, &mut aliases);
                            }
                        }
                    }
                }
            }
        }
    }

    Some(aliases)
}

/// Parse aliases from fish config content
fn parse_fish_alias_content<S: BuildHasher>(
    content: &str,
    aliases: &mut HashMap<String, String, S>,
) {
    // Fish aliases can be defined as: alias name='command' or using functions
    // First try the alias command format
    if let Ok(re) = Regex::new(r#"^\s*alias\s+([a-zA-Z0-9_-]+)=(['"])(.+?)\2"#) {
        for line in content.lines() {
            if let Ok(Some(caps)) = re.captures(line) {
                let name_result = caps.get(1);
                let cmd_result = caps.get(3);

                if let (Some(name_match), Some(cmd_match)) = (name_result, cmd_result) {
                    let name = name_match.as_str();
                    let cmd = cmd_match.as_str();
                    aliases.insert(name.to_string(), cmd.to_string());
                }
            }
        }
    }

    // Also check for alias using the `alias` command without quotes
    if let Ok(re2) = Regex::new(r#"^\s*alias\s+([a-zA-Z0-9_-]+)\s+(['"])(.*?)\2(;\s*|$)"#) {
        for line in content.lines() {
            if let Ok(Some(caps)) = re2.captures(line) {
                let name_result = caps.get(1);
                let cmd_result = caps.get(3);

                if let (Some(name_match), Some(cmd_match)) = (name_result, cmd_result) {
                    let name = name_match.as_str();
                    let cmd = cmd_match.as_str();
                    aliases.insert(name.to_string(), cmd.to_string());
                }
            }
        }
    }
}

/// Parse fish function files for aliases
fn parse_fish_function_alias<S: BuildHasher>(
    content: &str,
    function_name: &str,
    aliases: &mut HashMap<String, String, S>,
) {
    if let Ok(re) = Regex::new(r"(?:command|exec)\s+([^\s;]+)") {
        // Try to find command references in the function
        for caps in re.captures_iter(content).flatten() {
            if let Some(cmd_match) = caps.get(1) {
                let cmd = cmd_match.as_str();
                aliases.insert(function_name.to_string(), cmd.to_string());
                // We only need the first match
                break;
            }
        }
    }
}
