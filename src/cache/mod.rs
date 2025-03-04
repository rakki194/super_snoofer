#![warn(clippy::all, clippy::pedantic)]

use crate::{
    command::CommandPatterns,
    history::{CommandHistoryEntry, HistoryManager, HistoryTracker},
    shell::aliases::parse_shell_aliases,
    utils::{find_closest_match, get_path_commands},
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};

/// Default file name for the cache
pub const CACHE_FILE: &str = "super_snoofer_cache.json";

/// Threshold for similarity checks
pub const SIMILARITY_THRESHOLD: f64 = 0.6;

/// Cache lifetime in seconds (24 hours)
pub const CACHE_LIFETIME_SECS: u64 = 86400;

/// Cache lifetime for aliases in seconds (24 hours)
pub const ALIAS_CACHE_LIFETIME_SECS: u64 = 86400;

/// Main cache structure for the Super Snoofer application
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCache {
    /// Set of available commands in the PATH
    commands: HashSet<String>,

    /// Map of learned corrections: typo -> correct command
    learned_corrections: HashMap<String, String>,

    /// Timestamp of the last cache update
    #[serde(default = "SystemTime::now")]
    last_update: SystemTime,

    /// Cache file path (not serialized)
    #[serde(skip)]
    cache_path: Option<PathBuf>,

    /// Shell aliases - key is the alias name, value is the command it expands to
    #[serde(default)]
    shell_aliases: HashMap<String, String>,

    /// Last time shell aliases were updated
    #[serde(default = "SystemTime::now")]
    alias_last_update: SystemTime,

    /// History management
    #[serde(default)]
    history_manager: HistoryManager,

    /// Command patterns for well-known commands (not serialized)
    #[serde(skip)]
    command_patterns: CommandPatterns,
}

impl Default for CommandCache {
    fn default() -> Self {
        Self {
            commands: HashSet::new(),
            learned_corrections: HashMap::new(),
            last_update: SystemTime::now(),
            cache_path: None,
            shell_aliases: HashMap::new(),
            alias_last_update: SystemTime::now(),
            history_manager: HistoryManager::default(),
            command_patterns: CommandPatterns::new(),
        }
    }
}

impl CommandCache {
    /// Create a new `CommandCache` instance
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load the command cache from the default location
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded cache or a new one if no cache exists
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache file exists but cannot be opened
    /// - The cache file exists but cannot be parsed as valid JSON
    /// - There is an error updating the cache if needed
    pub fn load() -> Result<Self> {
        // Try to find the cache file in the standard locations
        let cache_dir = dirs::cache_dir().or_else(dirs::home_dir);

        if let Some(dir) = cache_dir {
            let cache_path = if dir.ends_with(".cache") {
                dir.join(CACHE_FILE)
            } else {
                dir.join(format!(".{CACHE_FILE}"))
            };

            return Self::load_from_path(&cache_path);
        }

        Ok(Self::default())
    }

    /// Load the command cache from a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to load the cache from
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded cache or a new one if no cache exists at the path
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache file exists but cannot be opened
    /// - The cache file exists but cannot be parsed as valid JSON
    /// - There is an error updating the cache if needed
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let cache = if path.exists() {
            // Try to load the existing cache
            let file = File::open(path)
                .with_context(|| format!("Failed to open cache file at {}", path.display()))?;

            let mut cache: CommandCache = serde_json::from_reader(file)
                .with_context(|| format!("Failed to parse cache file at {}", path.display()))?;

            // Set the cache path
            cache.cache_path = Some(path.to_path_buf());

            // Initialize command patterns
            cache.command_patterns = CommandPatterns::new();

            // If the cache is too old, clear it
            if cache.should_clear_cache() {
                cache.clear_cache();
            }

            // If alias cache is too old, update it
            if cache.should_update_aliases() {
                cache.update_aliases();
            }

            cache
        } else {
            // Create a new cache
            let mut cache = Self {
                cache_path: Some(path.to_path_buf()),
                ..Default::default()
            };

            // Ensure the cache is up to date
            if cache.commands.is_empty() {
                cache.update()?;
            }

            cache
        };

        Ok(cache)
    }

    /// Check if the cache should be cleared due to age
    fn should_clear_cache(&self) -> bool {
        if let Ok(duration) = SystemTime::now().duration_since(self.last_update) {
            return duration.as_secs() > CACHE_LIFETIME_SECS;
        }

        false
    }

    /// Check if shell aliases should be updated due to age
    fn should_update_aliases(&self) -> bool {
        if let Ok(duration) = SystemTime::now().duration_since(self.alias_last_update) {
            return duration.as_secs() > ALIAS_CACHE_LIFETIME_SECS;
        }

        false
    }

    /// Clear the command cache (retains learned corrections)
    pub fn clear_cache(&mut self) {
        self.commands.clear();
        self.last_update = SystemTime::now();
    }

    /// Clear both the command cache and learned corrections
    pub fn clear_memory(&mut self) {
        self.clear_cache();
        self.learned_corrections.clear();
        self.history_manager.clear_history();
    }

    /// Check if the cache has a correction for the given typo
    #[must_use]
    pub fn has_correction(&self, typo: &str) -> bool {
        self.learned_corrections.contains_key(typo)
    }

    /// Save the command cache to disk
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The parent directory for the cache file cannot be created
    /// - The cache file cannot be created
    /// - The cache cannot be serialized to JSON
    pub fn save(&self) -> Result<()> {
        if let Some(cache_path) = &self.cache_path {
            // Ensure the parent directory exists
            if let Some(parent) = cache_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }

            let file = File::create(cache_path).with_context(|| {
                format!("Failed to create cache file at {}", cache_path.display())
            })?;

            serde_json::to_writer(file, self)
                .with_context(|| format!("Failed to write cache to {}", cache_path.display()))?;
        }

        Ok(())
    }

    /// Learn a correction for a typo
    ///
    /// # Arguments
    ///
    /// * `typo` - The mistyped command
    /// * `correct_command` - The correct command
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The cache cannot be saved to disk
    pub fn learn_correction(&mut self, typo: &str, correct_command: &str) -> Result<()> {
        // If the correction contains spaces, it likely contains arguments
        // In this case, we'll store the full correction for the typo
        let correction = if correct_command.contains(' ') {
            correct_command.to_string()
        } else {
            // Otherwise, store just the command name
            correct_command.to_string()
        };

        self.learned_corrections
            .insert(typo.to_string(), correction);
        self.save()
    }

    /// Find a similar command for a given command
    #[must_use]
    pub fn find_similar(&self, command: &str) -> Option<String> {
        // First, check if we have this exact command
        if self.commands.contains(command) || self.shell_aliases.contains_key(command) {
            return Some(command.to_string());
        }

        // Second, check learned corrections - this should return the actual correction
        if let Some(correction) = self.learned_corrections.get(command) {
            return Some(correction.clone());
        }

        // Last resort: find the closest match using fuzzy matching
        self.get_closest_match(command, SIMILARITY_THRESHOLD)
    }

    /// Insert a command into the cache
    pub fn insert(&mut self, command: &str) {
        self.commands.insert(command.to_string());
    }

    /// Update the command cache with current PATH commands
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - There is an error retrieving commands from PATH
    /// - There is an error reading shell configuration files
    /// - There is an error saving the updated cache to disk
    pub fn update(&mut self) -> Result<()> {
        self.update_path_commands();

        if self.shell_aliases.is_empty() || self.should_update_aliases() {
            self.update_aliases();
        }

        self.last_update = SystemTime::now();
        self.save()
    }

    /// Update commands from PATH
    fn update_path_commands(&mut self) {
        // Get commands from PATH
        let path_commands = get_path_commands();

        // Update the command set
        self.commands = path_commands;
    }

    /// Update shell aliases
    fn update_aliases(&mut self) {
        if let Ok(aliases) = parse_shell_aliases() {
            self.shell_aliases = aliases;
            self.alias_last_update = SystemTime::now();
        }
    }

    /// Check if the cache contains a command
    #[must_use]
    pub fn contains(&self, command: &str) -> bool {
        self.commands.contains(command) || self.shell_aliases.contains_key(command)
    }

    /// Get the closest matching command within a threshold
    #[must_use]
    pub fn get_closest_match(&self, command: &str, threshold: f64) -> Option<String> {
        // Combine commands and alias names for matching
        let mut all_commands: Vec<String> = self.commands.iter().cloned().collect();
        all_commands.extend(self.shell_aliases.keys().cloned());

        // Create a vector of references to use with find_closest_match
        let command_refs: Vec<&String> = all_commands.iter().collect();

        // Find the closest match
        find_closest_match(command, &command_refs, threshold).map(|s| (*s).to_string())
    }

    /// Get the target command for an alias
    #[must_use]
    pub fn get_alias_target(&self, alias: &str) -> Option<&String> {
        self.shell_aliases.get(alias)
    }

    /// Find a similar command with frequency bias
    #[must_use]
    pub fn find_similar_with_frequency(&self, command: &str) -> Option<String> {
        // First, check for exact match
        if self.commands.contains(command) || self.shell_aliases.contains_key(command) {
            return Some(command.to_string());
        }

        // Then, check learned corrections
        if let Some(correction) = self.learned_corrections.get(command) {
            return Some(correction.clone());
        }

        // Finally, use the history manager to find a similar command with frequency bias
        self.history_manager
            .find_similar_with_frequency(command, |cmd| {
                self.get_closest_match(cmd, SIMILARITY_THRESHOLD)
            })
    }

    /// Fix a command line by correcting typos in command, arguments, and flags
    #[must_use]
    pub fn fix_command_line(&self, command_line: &str) -> Option<String> {
        crate::command::fix_command_line(
            command_line,
            |cmd| self.find_similar(cmd),
            &self.command_patterns,
        )
    }

    /// Set the cache path (useful for testing)
    pub fn set_cache_path(&mut self, path: PathBuf) {
        self.cache_path = Some(path);
    }

    /// Get a reference to the history manager
    #[must_use]
    pub fn history_manager(&self) -> &HistoryManager {
        &self.history_manager
    }

    /// Get the direct correction for a typo without fuzzy matching
    #[must_use]
    pub fn get_direct_correction(&self, typo: &str) -> Option<&String> {
        self.learned_corrections.get(typo)
    }

    /// Check if shell aliases are empty (helpful for testing)
    #[must_use]
    #[cfg(test)]
    pub fn is_aliases_empty(&self) -> bool {
        self.shell_aliases.is_empty()
    }

    /// Update shell aliases (exposed for testing)
    #[cfg(test)]
    pub fn update_aliases_for_test(&mut self) {
        self.update_aliases();
    }

    /// Get the alias last update timestamp (helpful for testing)
    #[must_use]
    #[cfg(test)]
    pub fn get_alias_last_update(&self) -> std::time::SystemTime {
        self.alias_last_update
    }

    /// Add a test alias (helpful for testing)
    #[cfg(test)]
    pub fn add_test_alias(&mut self, alias: &str, command: &str) {
        self.shell_aliases
            .insert(alias.to_string(), command.to_string());
    }

    /// Check if a command exists in PATH or shell aliases
    ///
    /// # Returns
    ///
    /// Returns a Result<bool> indicating if the command exists
    ///
    /// # Errors
    ///
    /// This function will return an error if there's an issue updating the cache
    pub fn command_exists(&self, command: &str) -> Result<bool> {
        // Check if it's in our commands set
        if self.contains(command) {
            return Ok(true);
        }

        // Check if it's an alias
        if self.get_alias_target(command).is_some() {
            return Ok(true);
        }

        Ok(false)
    }
}

// Implement HistoryTracker to delegate to the history manager
impl HistoryTracker for CommandCache {
    fn record_correction(&mut self, typo: &str, correction: &str) {
        self.history_manager.record_correction(typo, correction);
    }

    fn get_frequent_typos(&self, limit: usize) -> Vec<(String, usize)> {
        self.history_manager.get_frequent_typos(limit)
    }

    fn get_frequent_corrections(&self, limit: usize) -> Vec<(String, usize)> {
        self.history_manager.get_frequent_corrections(limit)
    }

    fn get_command_history(&self, limit: usize) -> Vec<CommandHistoryEntry> {
        self.history_manager.get_command_history(limit)
    }

    fn get_history_size(&self) -> usize {
        self.history_manager.get_history_size()
    }

    fn clear_history(&mut self) {
        self.history_manager.clear_history();
    }

    fn is_history_enabled(&self) -> bool {
        self.history_manager.is_history_enabled()
    }

    fn enable_history(&mut self) -> Result<()> {
        self.history_manager.enable_history()?;
        self.save()
    }

    fn disable_history(&mut self) -> Result<()> {
        self.history_manager.disable_history()?;
        self.save()
    }
}
