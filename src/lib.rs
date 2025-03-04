#![warn(clippy::all, clippy::pedantic)]

pub mod cache;
pub mod command;
pub mod commands;
pub mod display;
pub mod history;
pub mod shell;
pub mod suggestion;
pub mod utils;
pub mod ollama;
pub mod tui;

// Re-export key structs and traits for easier access
pub use cache::CommandCache;
pub use command::CommandPatterns;
pub use history::{CommandHistoryEntry, HistoryManager, HistoryTracker};
pub use shell::{
    add_alias,
    suggest_aliases,
    install_shell_integration,
    uninstall_shell_integration,
};
pub use ollama::OllamaClient;
pub use tui::TuiApp;

// Constants re-exported for backward compatibility
pub use cache::{CACHE_FILE, SIMILARITY_THRESHOLD};

pub use tui::run_tui_mode;
