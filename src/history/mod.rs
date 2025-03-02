#![warn(clippy::all, clippy::pedantic)]

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    time::SystemTime,
};

/// Maximum number of entries in history
pub const MAX_HISTORY_SIZE: usize = 100_000;

/// Entry in the command history
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandHistoryEntry {
    pub typo: String,
    pub correction: String,
    pub timestamp: SystemTime,
}

/// Gets whether history tracking is enabled by default
#[must_use]
pub fn default_history_enabled() -> bool {
    true
}

/// Functions for tracking and analyzing command history
pub trait HistoryTracker {
    /// Record a correction in the history
    fn record_correction(&mut self, typo: &str, correction: &str);

    /// Get frequent typos with their counts, limited to a specified number
    fn get_frequent_typos(&self, limit: usize) -> Vec<(String, usize)>;

    /// Get frequent corrections with their counts, limited to a specified number
    fn get_frequent_corrections(&self, limit: usize) -> Vec<(String, usize)>;

    /// Get recent command history entries, limited to a specified number
    fn get_command_history(&self, limit: usize) -> Vec<CommandHistoryEntry>;

    /// Get the total number of history entries
    fn get_history_size(&self) -> usize;

    /// Clear all history data
    fn clear_history(&mut self);

    /// Check if history tracking is enabled
    fn is_history_enabled(&self) -> bool;

    /// Enable history tracking
    ///
    /// # Errors
    /// This method will return an error if the history state cannot be persisted
    fn enable_history(&mut self) -> anyhow::Result<()>;

    /// Disable history tracking
    ///
    /// # Errors
    /// This method will return an error if the history state cannot be persisted
    fn disable_history(&mut self) -> anyhow::Result<()>;
}

/// Manages a history of command corrections and frequency data
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryManager {
    /// Command history for frequency analysis
    pub command_history: VecDeque<CommandHistoryEntry>,
    /// Frequency counter for typos
    pub typo_frequency: HashMap<String, usize>,
    /// Frequency counter for corrections
    pub correction_frequency: HashMap<String, usize>,
    /// Whether history tracking is enabled
    #[serde(default = "default_history_enabled")]
    pub history_enabled: bool,
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self {
            command_history: VecDeque::new(),
            typo_frequency: HashMap::new(),
            correction_frequency: HashMap::new(),
            history_enabled: default_history_enabled(),
        }
    }
}

impl HistoryManager {
    /// Create a new history manager
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Find a similar command with frequency bias
    pub fn find_similar_with_frequency(
        &self,
        command: &str,
        find_similar_fn: impl Fn(&str) -> Option<String>,
    ) -> Option<String> {
        // First, check if we have a learned correction
        if let Some(correction) = find_similar_fn(command) {
            // If we have a correction and we have frequency data for it,
            // return it along with the frequency data
            return Some(correction);
        }

        None
    }
}

impl HistoryTracker for HistoryManager {
    fn record_correction(&mut self, typo: &str, correction: &str) {
        // Skip recording if history is disabled
        if !self.history_enabled {
            return;
        }

        // Update frequency counters
        *self.typo_frequency.entry(typo.to_string()).or_insert(0) += 1;
        *self
            .correction_frequency
            .entry(correction.to_string())
            .or_insert(0) += 1;

        // Add to history
        self.command_history.push_front(CommandHistoryEntry {
            typo: typo.to_string(),
            correction: correction.to_string(),
            timestamp: SystemTime::now(),
        });

        // Ensure we don't exceed the maximum history size
        if self.command_history.len() > MAX_HISTORY_SIZE {
            self.command_history.pop_back();
        }
    }

    fn get_frequent_typos(&self, limit: usize) -> Vec<(String, usize)> {
        let mut typos: Vec<(String, usize)> = self
            .typo_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        typos.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency in descending order
        typos.truncate(limit); // Limit to the requested number

        typos
    }

    fn get_frequent_corrections(&self, limit: usize) -> Vec<(String, usize)> {
        let mut corrections: Vec<(String, usize)> = self
            .correction_frequency
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        corrections.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by frequency in descending order
        corrections.truncate(limit); // Limit to the requested number

        corrections
    }

    fn get_command_history(&self, limit: usize) -> Vec<CommandHistoryEntry> {
        self.command_history.iter().take(limit).cloned().collect()
    }

    fn get_history_size(&self) -> usize {
        self.command_history.len()
    }

    fn clear_history(&mut self) {
        self.command_history.clear();
        self.typo_frequency.clear();
        self.correction_frequency.clear();
    }

    fn is_history_enabled(&self) -> bool {
        self.history_enabled
    }

    fn enable_history(&mut self) -> anyhow::Result<()> {
        self.history_enabled = true;
        Ok(())
    }

    fn disable_history(&mut self) -> anyhow::Result<()> {
        self.history_enabled = false;
        Ok(())
    }
}
