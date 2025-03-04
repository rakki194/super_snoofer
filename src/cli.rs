#![warn(clippy::all, clippy::pedantic)]

use clap::{Parser, Subcommand};

use crate::ollama::{DEFAULT_DOLPHIN_MODEL, DEFAULT_CODESTRAL_MODEL};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Prompt to process (launches TUI mode)
    #[arg(short, long)]
    pub prompt: Option<String>,

    /// Use Codestral model instead of Dolphin
    #[arg(long)]
    pub codestral: bool,
    
    /// Specify the standard model to use (overrides default)
    #[arg(long, default_value_t = DEFAULT_DOLPHIN_MODEL.to_string())]
    pub standard_model: String,
    
    /// Specify the code model to use (overrides default)
    #[arg(long, default_value_t = DEFAULT_CODESTRAL_MODEL.to_string())]
    pub code_model: String,

    /// Command line to check (for command not found handler)
    #[arg(name = "command", last = true, allow_hyphen_values = true)]
    pub command_to_check: Vec<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install shell integration
    Install,
    /// Uninstall shell integration
    Uninstall,
    /// Normal operation: suggest similar commands
    Command {
        command: String,
    },
    /// Clear the command cache but keep learned corrections
    ResetCache,
    /// Clear both the command cache and learned corrections
    ResetMemory,
    /// Display your recent command corrections
    History,
    /// Display your most common typos
    FrequentTypos,
    /// Display your most frequently used corrections
    FrequentCorrections,
    /// Clear your command history
    ClearHistory,
    /// Enable command history tracking
    EnableHistory,
    /// Disable command history tracking
    DisableHistory,
    /// Add shell alias (default: super_snoofer)
    AddAlias {
        /// Alias name
        name: String,
        /// Command to alias (defaults to super_snoofer)
        #[arg(default_value = "super_snoofer")]
        command: Option<String>,
    },
    /// Suggest personalized shell aliases
    Suggest,
    /// Check command line for corrections
    CheckCommandLine {
        /// Command line to check
        command: String,
    },
    /// Process a full command line (for shell integration)
    FullCommand {
        /// Command line to process
        command: String,
    },
    /// Manually teach a command correction
    LearnCorrection {
        /// The typo to correct
        typo: String,
        /// The correct command
        command: String,
    },
    /// Chat with AI about super snoofer
    Prompt {
        /// Question to ask
        prompt: String,
        /// Use Codestral model instead of Dolphin
        #[arg(long)]
        codestral: bool,
        /// Specify the standard model to use (overrides default)
        #[arg(long, default_value_t = DEFAULT_DOLPHIN_MODEL.to_string())]
        standard_model: String,
        /// Specify the code model to use (overrides default)
        #[arg(long, default_value_t = DEFAULT_CODESTRAL_MODEL.to_string())]
        code_model: String,
    },
}

impl Cli {
    /// Parse command line arguments, with special handling for command not found cases
    pub fn parse_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        
        // If we have a -- separator, everything after it is a command to check
        if let Some(sep_pos) = args.iter().position(|x| x == "--") {
            if sep_pos + 1 < args.len() {
                return Self {
                    command: None,
                    prompt: None,
                    codestral: false,
                    standard_model: DEFAULT_DOLPHIN_MODEL.to_string(),
                    code_model: DEFAULT_CODESTRAL_MODEL.to_string(),
                    command_to_check: args[sep_pos + 1..].to_vec(),
                };
            }
        }
        
        // Otherwise, use normal clap parsing
        Self::parse()
    }
} 