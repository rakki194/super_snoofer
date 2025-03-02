#![warn(clippy::all, clippy::pedantic)]

// Public API
pub mod cache;
pub mod command;
pub mod display;
pub mod history;
pub mod shell;
pub mod suggestion;
pub mod utils;

// Re-export commonly used types
pub use cache::CommandCache;
pub use history::HistoryTracker;

// Re-export constants for backward compatibility
pub use cache::{CACHE_FILE, SIMILARITY_THRESHOLD};

// Tests
#[cfg(test)]
pub mod tests;
