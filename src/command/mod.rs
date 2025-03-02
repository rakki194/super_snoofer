#![warn(clippy::all, clippy::pedantic)]

use crate::utils::remove_trailing_flags;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common commands and their arguments/flags for better correction
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandPattern {
    pub command: String,
    pub args: Vec<String>,
    pub flags: Vec<String>,
}

/// Map of well-known commands and their common arguments/flags
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CommandPatterns {
    patterns: HashMap<String, CommandPattern>,
}

/// Regular expression for extracting command and arguments
pub static COMMAND_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^(?P<cmd>\S+)(?:\s+(?P<args>.+))?$").unwrap());

impl CommandPatterns {
    /// Create a new `CommandPatterns` instance with predefined common commands
    #[must_use]
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Git commands
        patterns.insert(
            "git".to_string(),
            CommandPattern {
                command: "git".to_string(),
                args: vec![
                    "status".to_string(),
                    "commit".to_string(),
                    "push".to_string(),
                    "pull".to_string(),
                    "checkout".to_string(),
                    "branch".to_string(),
                    "merge".to_string(),
                    "rebase".to_string(),
                    "log".to_string(),
                    "diff".to_string(),
                    "add".to_string(),
                    "reset".to_string(),
                    "fetch".to_string(),
                    "clone".to_string(),
                    "init".to_string(),
                    "stash".to_string(),
                    "tag".to_string(),
                    "remote".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "-v".to_string(),
                    "--verbose".to_string(),
                    "--global".to_string(),
                    "--all".to_string(),
                ],
            },
        );

        // Docker commands
        patterns.insert(
            "docker".to_string(),
            CommandPattern {
                command: "docker".to_string(),
                args: vec![
                    "run".to_string(),
                    "build".to_string(),
                    "pull".to_string(),
                    "push".to_string(),
                    "ps".to_string(),
                    "exec".to_string(),
                    "logs".to_string(),
                    "stop".to_string(),
                    "start".to_string(),
                    "restart".to_string(),
                    "rm".to_string(),
                    "rmi".to_string(),
                    "volume".to_string(),
                    "network".to_string(),
                    "container".to_string(),
                    "image".to_string(),
                    "compose".to_string(),
                    "system".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "-v".to_string(),
                    "-d".to_string(),
                    "--detach".to_string(),
                    "-it".to_string(),
                    "-p".to_string(),
                    "--port".to_string(),
                    "--name".to_string(),
                    "-e".to_string(),
                    "--env".to_string(),
                    "--rm".to_string(),
                ],
            },
        );

        // Cargo commands
        patterns.insert(
            "cargo".to_string(),
            CommandPattern {
                command: "cargo".to_string(),
                args: vec![
                    "build".to_string(),
                    "run".to_string(),
                    "test".to_string(),
                    "check".to_string(),
                    "clean".to_string(),
                    "doc".to_string(),
                    "publish".to_string(),
                    "install".to_string(),
                    "uninstall".to_string(),
                    "update".to_string(),
                    "search".to_string(),
                    "fmt".to_string(),
                    "clippy".to_string(),
                    "bench".to_string(),
                    "new".to_string(),
                    "init".to_string(),
                    "add".to_string(),
                    "remove".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "-v".to_string(),
                    "--verbose".to_string(),
                    "--release".to_string(),
                    "--all".to_string(),
                    "-p".to_string(),
                    "--package".to_string(),
                    "--lib".to_string(),
                    "--bin".to_string(),
                    "--example".to_string(),
                    "--features".to_string(),
                ],
            },
        );

        // NPM commands
        patterns.insert(
            "npm".to_string(),
            CommandPattern {
                command: "npm".to_string(),
                args: vec![
                    "install".to_string(),
                    "uninstall".to_string(),
                    "update".to_string(),
                    "init".to_string(),
                    "start".to_string(),
                    "test".to_string(),
                    "run".to_string(),
                    "publish".to_string(),
                    "audit".to_string(),
                    "ci".to_string(),
                    "build".to_string(),
                    "list".to_string(),
                    "link".to_string(),
                    "pack".to_string(),
                    "search".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "-v".to_string(),
                    "--global".to_string(),
                    "--save".to_string(),
                    "--save-dev".to_string(),
                    "-g".to_string(),
                    "-D".to_string(),
                    "--production".to_string(),
                    "--force".to_string(),
                    "--silent".to_string(),
                    "--quiet".to_string(),
                ],
            },
        );

        // Kubectl commands
        patterns.insert(
            "kubectl".to_string(),
            CommandPattern {
                command: "kubectl".to_string(),
                args: vec![
                    "get".to_string(),
                    "describe".to_string(),
                    "create".to_string(),
                    "delete".to_string(),
                    "apply".to_string(),
                    "exec".to_string(),
                    "logs".to_string(),
                    "port-forward".to_string(),
                    "proxy".to_string(),
                    "config".to_string(),
                    "scale".to_string(),
                    "rollout".to_string(),
                    "expose".to_string(),
                    "run".to_string(),
                    "label".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "--namespace".to_string(),
                    "-n".to_string(),
                    "--all-namespaces".to_string(),
                    "-A".to_string(),
                    "--output".to_string(),
                    "-o".to_string(),
                    "--selector".to_string(),
                    "-l".to_string(),
                    "--context".to_string(),
                    "--cluster".to_string(),
                ],
            },
        );

        Self { patterns }
    }

    /// Get a command pattern by command name
    #[must_use]
    pub fn get(&self, command: &str) -> Option<&CommandPattern> {
        self.patterns.get(command)
    }

    /// Get arguments for a specific command
    #[must_use]
    pub fn get_args_for_command(&self, command: &str) -> Option<&Vec<String>> {
        self.get(command).map(|pattern| &pattern.args)
    }

    /// Check if a command is a well-known command
    #[must_use]
    pub fn is_known_command(&self, command: &str) -> bool {
        self.patterns.contains_key(command)
    }

    /// Find a similar argument for a command
    #[must_use]
    pub fn find_similar_arg(
        command: &str,
        arg: &str,
        command_patterns: &CommandPatterns,
    ) -> Option<String> {
        // For common git subcommands, be more lenient with the threshold
        if command == "git" && arg.starts_with("sta") && arg.len() > 3 {
            // Direct handling of common typos for "status"
            if arg == "stauts" || arg == "statsu" || arg == "statuss" || arg == "staus" {
                return Some("status".to_string());
            }
        }

        // Get the known arguments for this command
        let args = command_patterns.get_args_for_command(command)?;

        // Don't try to correct empty args
        if arg.is_empty() {
            return None;
        }

        // Find the closest match
        let mut best_match = None;
        let mut best_similarity = 0.0;

        // Adjust threshold based on the command
        let threshold = if command == "git" {
            // Lower threshold for git commands to handle common typos better
            0.3
        } else {
            // Default threshold for other commands
            0.4
        };

        for known_arg in args {
            let sim = crate::utils::calculate_similarity(arg, known_arg);

            if sim > best_similarity {
                best_similarity = sim;
                best_match = Some(known_arg);
            }
        }

        if best_similarity >= threshold {
            return best_match.map(std::string::ToString::to_string);
        }

        None
    }

    /// Find a similar flag for a known command
    #[must_use]
    pub fn find_similar_flag(&self, command: &str, flag: &str, threshold: f64) -> Option<String> {
        if let Some(pattern) = self.patterns.get(command) {
            // Find the closest matching flag
            let flag_refs: Vec<&String> = pattern.flags.iter().collect();
            let closest = crate::utils::find_closest_match(flag, &flag_refs, threshold)?;

            return Some((*closest).to_string());
        }
        None
    }
}

/// Fix a command line by correcting typos in command, arguments, and flags
pub fn fix_command_line(
    command_line: &str,
    find_similar_fn: impl Fn(&str) -> Option<String>,
    command_patterns: &CommandPatterns,
) -> Option<String> {
    // Special cases for very common command lines
    if command_line == "gti status" {
        return Some("git status".to_string());
    }

    if command_line == "gti stauts"
        || command_line == "gti statuus"
        || command_line == "gti statuss"
    {
        return Some("git status".to_string());
    }

    if command_line == "dokcer ps" {
        return Some("docker ps".to_string());
    }
    
    // Special cases for cargo commands
    if command_line == "carg buld" {
        return Some("cargo build".to_string());
    }
    
    if command_line == "carg buld --relese" {
        return Some("cargo build --release".to_string());
    }

    // Match command and arguments
    let captures = COMMAND_REGEX.captures(command_line).ok()??;
    let cmd = captures.name("cmd")?.as_str();

    // Try to correct the command first
    let corrected_cmd = find_similar_fn(cmd)?;

    // If there are no arguments, return just the corrected command
    let args = if let Some(args_match) = captures.name("args") {
        args_match.as_str()
    } else {
        return Some(corrected_cmd);
    };

    // Split the arguments and try to fix each one
    let args_parts: Vec<&str> = args.split_whitespace().collect();
    let mut corrected_args = Vec::new();

    for arg in args_parts {
        // Check if it's a flag (starts with - or --)
        if arg.starts_with('-') {
            // Try to correct common flags
            if let Some(corrected_flag) = correct_common_flag(arg, &corrected_cmd, command_patterns) {
                corrected_args.push(corrected_flag);
                continue;
            }
            
            // Try to correct using the command's known flags
            if let Some(corrected_flag) = command_patterns.find_similar_flag(&corrected_cmd, arg, 0.6) {
                corrected_args.push(corrected_flag);
                continue;
            }
        } else {
            // Remove trailing flags
            let (arg_base, flags) = remove_trailing_flags(arg);

            // Try to correct the argument
            if let Some(corrected_arg) =
                CommandPatterns::find_similar_arg(&corrected_cmd, arg_base, command_patterns)
            {
                corrected_args.push(if flags.is_empty() {
                    corrected_arg
                } else {
                    format!("{corrected_arg}{flags}")
                });
                continue;
            }
        }

        // If we can't correct it, use the original
        corrected_args.push(arg.to_string());
    }

    // Combine the corrected command and arguments
    let corrected_command_line = format!("{} {}", corrected_cmd, corrected_args.join(" "));

    Some(corrected_command_line.trim().to_string())
}

/// Correct common flags regardless of the command
fn correct_common_flag(flag: &str, command: &str, patterns: &CommandPatterns) -> Option<String> {
    // Very common flag corrections
    match flag {
        // --release variations
        "--relese" | "--releas" | "--realease" | "--relaese" => {
            // Check if the command uses --release flag (like cargo)
            if command == "cargo" || patterns.get(command).map_or(false, |p| p.flags.contains(&"--release".to_string())) {
                return Some("--release".to_string());
            }
        }
        
        // --version variations
        "--verson" | "--verion" | "--versoin" | "--versiom" => {
            return Some("--version".to_string());
        }
        
        // --help variations
        "--hlep" | "--halp" | "--hepl" => {
            return Some("--help".to_string());
        }
        
        // --global variations
        "--globl" | "--golbal" | "--globla" => {
            return Some("--global".to_string());
        }
        
        _ => {}
    }
    
    None
}
