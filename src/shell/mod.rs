#![warn(clippy::all, clippy::pedantic)]

pub mod aliases;
pub mod integration;

// Re-export the public interface
pub use integration::{install_shell_integration, uninstall_shell_integration};
pub use aliases::{add_alias, suggest_aliases};
