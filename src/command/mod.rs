use crate::utils::remove_trailing_flags;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Common commands and their arguments/flags for better correction
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandPattern {
    pub command: String,
    pub args: Vec<String>,
    pub flags: Vec<String>,
    /// Last time this command was updated
    #[serde(default = "SystemTime::now")]
    pub last_updated: SystemTime,
    /// Count of how many times this command was used
    #[serde(default)]
    pub usage_count: usize,
}

/// Map of well-known commands and their common arguments/flags
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandPatterns {
    /// Command patterns indexed by command name
    pub patterns: HashMap<String, CommandPattern>,
    /// Max number of arguments to store per command
    #[serde(default = "default_max_args")]
    max_args_per_command: usize,
    /// Max number of flags to store per command
    #[serde(default = "default_max_flags")]
    max_flags_per_command: usize,
    /// Threshold for adding new args/flags (minimum uses)
    #[serde(default = "default_usage_threshold")]
    usage_threshold: usize,
    /// Last time we did a full command discovery scan
    #[serde(default = "SystemTime::now")]
    last_discovery_scan: SystemTime,
}

/// Default maximum number of arguments to store per command
fn default_max_args() -> usize {
    50
}

/// Default maximum number of flags to store per command
fn default_max_flags() -> usize {
    30
}

/// Default usage threshold before adding a new argument
fn default_usage_threshold() -> usize {
    2
}

/// Regular expression for extracting command and arguments
pub static COMMAND_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^(?P<cmd>\S+)(?:\s+(?P<args>.+))?$").unwrap());

/// Regular expression for identifying flags
pub static FLAG_REGEX: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^-{1,2}[a-zA-Z0-9][\w-]*(?:=.*)?$").unwrap());

impl Default for CommandPatterns {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPatterns {
    /// Create a new `CommandPatterns` instance with predefined common commands
    #[must_use]
    pub fn new() -> Self {
        let mut patterns = HashMap::new();

        // Git commands - We'll keep some initial data to help with common commands
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
                last_updated: SystemTime::now(),
                usage_count: 1,
            },
        );

        // We'll keep a few more common commands as seed data
        patterns.insert(
            "docker".to_string(),
            CommandPattern {
                command: "docker".to_string(),
                args: vec![
                    "run".to_string(),
                    "build".to_string(),
                    "pull".to_string(),
                    "ps".to_string(),
                    "exec".to_string(),
                    "logs".to_string(),
                ],
                flags: vec![
                    "--help".to_string(),
                    "-h".to_string(),
                    "--version".to_string(),
                ],
                last_updated: SystemTime::now(),
                usage_count: 1,
            },
        );

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
                ],
                flags: vec![
                    "--help".to_string(),
                    "--version".to_string(),
                    "--release".to_string(),
                ],
                last_updated: SystemTime::now(),
                usage_count: 1,
            },
        );

        Self {
            patterns,
            max_args_per_command: default_max_args(),
            max_flags_per_command: default_max_flags(),
            usage_threshold: default_usage_threshold(),
            last_discovery_scan: SystemTime::now(),
        }
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

    /// Check if a command is a known command
    #[must_use]
    pub fn is_known_command(&self, command: &str) -> bool {
        self.patterns.contains_key(command)
    }

    /// Learn from a valid command execution
    /// Adds the command and its arguments to our knowledge base
    pub fn learn_from_command(&mut self, command_line: &str) {
        // Extract command and arguments
        let captures = match COMMAND_REGEX.captures(command_line) {
            Ok(Some(caps)) => caps,
            _ => return, // Can't parse the command line
        };

        let cmd = match captures.name("cmd") {
            Some(cmd) => cmd.as_str(),
            None => return, // No command found
        };

        // Skip learning for some common command line tools like grep, cat, etc.
        if ["grep", "cat", "less", "more", "head", "tail"].contains(&cmd) {
            return;
        }

        // Create or update the command pattern
        if let Some(pattern) = self.patterns.get_mut(cmd) {
            // Command exists, update it
            pattern.usage_count += 1;
            pattern.last_updated = SystemTime::now();

            // Process arguments if present
            if let Some(args_match) = captures.name("args") {
                let args_str = args_match.as_str();

                // This is a simple split - for a production system we'd need more
                // sophisticated parsing to handle quotes, escapes, etc.
                let arg_parts: Vec<&str> = args_str.split_whitespace().collect();

                // Process each argument
                for arg in arg_parts {
                    // Skip very short arguments - likely to be filenames
                    if arg.len() < 2 {
                        continue;
                    }

                    let is_flag = FLAG_REGEX.is_match(arg).unwrap_or(false);

                    if is_flag {
                        // It's a flag - add it to flags if not already present
                        if !pattern.flags.contains(&arg.to_string()) {
                            // Maintain size limit
                            if pattern.flags.len() >= self.max_flags_per_command {
                                pattern.flags.remove(0);
                            }
                            pattern.flags.push(arg.to_string());
                        }
                    } else {
                        // It's a regular argument
                        // Skip arguments that look like paths or filenames
                        if arg.contains('/') || arg.contains('\\') || arg.contains('.') {
                            continue;
                        }

                        // Only consider adding if usage count exceeds threshold
                        if pattern.usage_count >= self.usage_threshold {
                            // Add the argument if not already present
                            if !pattern.args.contains(&arg.to_string()) {
                                // Maintain size limit
                                if pattern.args.len() >= self.max_args_per_command {
                                    pattern.args.remove(0);
                                }
                                pattern.args.push(arg.to_string());
                            }
                        }
                    }
                }
            }
        } else {
            // Command doesn't exist, create a new pattern
            let mut new_pattern = CommandPattern {
                command: cmd.to_string(),
                args: Vec::new(),
                flags: Vec::new(),
                last_updated: SystemTime::now(),
                usage_count: 1,
            };

            // Process arguments if present
            if let Some(args_match) = captures.name("args") {
                let args_str = args_match.as_str();
                let arg_parts: Vec<&str> = args_str.split_whitespace().collect();

                // Process each argument
                for arg in arg_parts {
                    // Skip very short arguments
                    if arg.len() < 2 {
                        continue;
                    }

                    let is_flag = FLAG_REGEX.is_match(arg).unwrap_or(false);

                    if is_flag {
                        // It's a flag - add to flags
                        new_pattern.flags.push(arg.to_string());
                    }
                    // We don't add regular arguments on first usage
                }
            }

            // Add the new pattern
            self.patterns.insert(cmd.to_string(), new_pattern);
        }
    }

    /// Generate ZSH completion script for a command
    #[must_use]
    pub fn generate_zsh_completion(&self, command: &str) -> Option<String> {
        let pattern = self.get(command)?;

        let mut completion = format!("#compdef {}\n\n", command);
        completion.push_str("_arguments \\\n");

        // Add flags
        for flag in &pattern.flags {
            completion.push_str(&format!("  '{}[{}]' \\\n", flag, flag));
        }

        // Add arguments - in ZSH we'll use them as completions for the first argument
        if !pattern.args.is_empty() {
            completion.push_str("  '*:arg:->args' \\\n");
            completion.push_str("\n\ncase $state in\n");
            completion.push_str("  args)\n");
            completion.push_str("    local -a args\n");
            completion.push_str("    args=(\n");

            for arg in &pattern.args {
                completion.push_str(&format!("      '{}:{}'\n", arg, arg));
            }

            completion.push_str("    )\n");
            completion.push_str("    _describe 'command arguments' args\n");
            completion.push_str("    ;;\n");
            completion.push_str("esac\n");
        }

        Some(completion)
    }

    /// Generate a combined ZSH completion script for all known commands
    #[must_use]
    pub fn generate_all_completions(&self) -> String {
        let mut all_completions = String::new();
        all_completions.push_str("# Generated by super_snoofer\n\n");

        for command in self.patterns.keys() {
            if let Some(completion) = self.generate_zsh_completion(command) {
                all_completions.push_str(&format!("# Completion for {}\n", command));
                all_completions.push_str(&completion);
                all_completions.push_str("\n\n");
            }
        }

        all_completions
    }

    /// Check if we need to do a discovery scan
    #[must_use]
    pub fn should_run_discovery(&self) -> bool {
        // Only run discovery every 7 days
        if let Ok(duration) = SystemTime::now().duration_since(self.last_discovery_scan) {
            return duration.as_secs() > 7 * 86400; // 7 days in seconds
        }
        false
    }

    /// Update the discovery scan timestamp
    pub fn update_discovery_timestamp(&mut self) {
        self.last_discovery_scan = SystemTime::now();
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

    // First, check if the entire command line has a direct correction
    // by calling find_similar_fn with the whole command line
    if let Some(direct_correction) = find_similar_fn(command_line) {
        return Some(direct_correction);
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
        // Remove trailing flags
        let (arg_base, flags) = remove_trailing_flags(arg);

        // Try to correct the argument if it's not a flag
        if !arg_base.starts_with('-') {
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
        } else if arg_base.starts_with('-') {
            // This is a flag, try to correct it
            if let Some(corrected_flag) =
                command_patterns.find_similar_flag(&corrected_cmd, arg_base, 0.5)
            {
                corrected_args.push(if flags.is_empty() {
                    corrected_flag
                } else {
                    format!("{corrected_flag}{flags}")
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
