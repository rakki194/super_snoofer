use anyhow::{Context, Result};
use colored::Colorize;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub mod aliases;

/// Detect the user's shell and return appropriate configuration details
pub fn detect_shell_config(
    alias_name: &str,
    correction: &str,
) -> Result<(&'static str, PathBuf, String)> {
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    
    // Get the SHELL environment variable
    let shell_path = std::env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    let shell_executable = Path::new(&shell_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("bash");
    
    // Check for common environment variables that indicate PowerShell
    let is_powershell = std::env::var("PSModulePath").is_ok()
        || std::env::var("POWERSHELL_DISTRIBUTION_CHANNEL").is_ok();
    
    // Check for nushell
    let is_nushell = shell_executable == "nu";
    
    // Determine the shell type, config path, and alias line
    match (shell_executable, is_powershell, is_nushell) {
        (_, true, _) => {
            // PowerShell
            let profile_path = if cfg!(windows) {
                // Windows PowerShell profile paths
                if let Ok(documents) = std::env::var("USERPROFILE") {
                    let documents_path = PathBuf::from(documents.clone())
                        .join("Documents")
                        .join("WindowsPowerShell")
                        .join("Microsoft.PowerShell_profile.ps1");
                    if documents_path.exists() {
                        documents_path
                    } else {
                        PathBuf::from(documents)
                            .join("Documents")
                            .join("PowerShell")
                            .join("Microsoft.PowerShell_profile.ps1")
                    }
                } else {
                    home_dir
                        .join("Documents")
                        .join("PowerShell")
                        .join("Microsoft.PowerShell_profile.ps1")
                }
            } else {
                // PowerShell on Linux/macOS
                home_dir
                    .join(".config")
                    .join("powershell")
                    .join("Microsoft.PowerShell_profile.ps1")
            };
            
            let alias_line = format!("Set-Alias -Name {alias_name} -Value {correction}");
            Ok(("PowerShell", profile_path, alias_line))
        },
        (_, _, true) => {
            // Nushell
            let config_path = home_dir.join(".config").join("nushell").join("config.nu");
            let alias_line = format!("alias {alias_name} = {correction}");
            Ok(("Nushell", config_path, alias_line))
        },
        ("fish", _, _) => {
            // Fish shell
            let config_path = home_dir.join(".config").join("fish").join("config.fish");
            let alias_line = format!("alias {alias_name} '{correction}'");
            Ok(("Fish", config_path, alias_line))
        },
        ("zsh", _, _) => {
            // Zsh shell
            let config_path = home_dir.join(".zshrc");
            let alias_line = format!("alias {alias_name}='{correction}'");
            Ok(("Zsh", config_path, alias_line))
        },
        ("bash", _, _) => {
            // Bash shell
            let config_path = home_dir.join(".bashrc");
            let alias_line = format!("alias {alias_name}='{correction}'");
            Ok(("Bash", config_path, alias_line))
        },
        ("ksh", _, _) => {
            // Korn shell
            let config_path = home_dir.join(".kshrc");
            let alias_line = format!("alias {alias_name}='{correction}'");
            Ok(("Korn shell", config_path, alias_line))
        },
        ("csh" | "tcsh", _, _) => {
            // C shell or TCSH
            let config_path = if shell_executable == "tcsh" {
                home_dir.join(".tcshrc")
            } else {
                home_dir.join(".cshrc")
            };
            let alias_line = format!("alias {alias_name} '{correction}'");
            Ok(("C shell", config_path, alias_line))
        },
        _ if cfg!(windows) && !is_powershell => {
            // Windows Command Prompt
            let config_path = PathBuf::from(
                std::env::var("USERPROFILE").unwrap_or_else(|_| String::from("C:\\Users\\Default")),
            )
            .join("doskey.bat");
            let alias_line = format!("doskey {alias_name}={correction}");
            Ok(("Windows Command Prompt", config_path, alias_line))
        },
        _ => {
            // Default to Bash for unknown shells
            let config_path = home_dir.join(".bashrc");
            let alias_line = format!("alias {alias_name}='{correction}'");
            Ok(("Bash", config_path, alias_line))
        }
    }
}

/// Add an alias to the appropriate shell configuration file
pub fn add_to_shell_config(shell_type: &str, config_path: &Path, alias_line: &str) -> Result<()> {
    println!("Adding alias to {}", config_path.display());
    
    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Append the alias to the config file
    let mut config = if config_path.exists() {
        std::fs::OpenOptions::new()
            
            .append(true)
            .open(config_path)?
    } else {
        std::fs::File::create(config_path)?
    };
    
    // Add a newline before the alias if the file doesn't end with one
    if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        if !content.ends_with('\n') {
            writeln!(config)?;
        }
    }
    
    // Add a comment and the alias
    writeln!(config, "\n# Added by Super Snoofer")?;
    writeln!(config, "{alias_line}")?;
    
    // Generate appropriate reload command based on shell
    let reload_cmd = match shell_type {
        "Bash" => format!("source {}", config_path.display()),
        "Zsh" => format!("source {}", config_path.display()),
        "Fish" => format!("source {}", config_path.display()),
        "PowerShell" => format!(". {}", config_path.display()),
        "Nushell" => format!("source {}", config_path.display()),
        "Korn shell" => format!(". {}", config_path.display()),
        "C shell" => format!("source {}", config_path.display()),
        "Windows Command Prompt" => format!("call {}", config_path.display()),
        _ => format!("source {}", config_path.display()),
    };
    
    println!(
        "{}",
        format!(
            "Added alias to your {shell_type} configuration! 🐺 Please run '{reload_cmd}' to use it."
        )
        .bright_green()
    );
    
    Ok(())
} 