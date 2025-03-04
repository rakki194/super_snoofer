#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use std::{
    fs,
    io::Write,
};

/// Installs shell integration for Super Snoofer
/// 
/// # Errors
/// Returns an error if the shell integration installation fails due to file system operations or permission issues
pub fn install_shell_integration() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home_dir.join(".config").join("super_snoofer");
    let integration_path = config_dir.join("shell_integration.zsh");
    let zshrc_path = home_dir.join(".zshrc");

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;

    // Create the integration script
    write_integration_script(&integration_path)?;
    
    // Add source directive to shell config files if not already present
    add_source_directive(&zshrc_path, &integration_path)?;

    println!("Super Snoofer shell integration installed successfully.");
    println!("Please restart your shell or run 'source ~/.zshrc' to activate it.");
    
    Ok(())
}

/// Writes the shell integration script to the specified path
/// 
/// # Errors
/// Returns an error if writing to the file fails
fn write_integration_script(integration_path: &std::path::Path) -> Result<()> {
    let script = r#"# Super Snoofer Integration
# Flag to prevent double execution
typeset -g __super_snoofer_executing=0

function __super_snoofer_check_command_line() {
    local cmd="$1"
    shift
    local args=("$@")
    
    # Skip if we're already executing a super_snoofer command
    if (( __super_snoofer_executing )); then
        __super_snoofer_executing=0
        return 0
    fi
    
    # Skip empty commands, super_snoofer itself, and commands starting with space
    [[ -z "$cmd" || "$cmd" =~ ^[[:space:]]+ || "$cmd" =~ ^super_snoofer ]] && return 0
    
    # Handle ] and ]] commands
    if [[ "$cmd" == "]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt ""
        return $?
    elif [[ "$cmd" == "]]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt "" --codestral
        return $?
    elif [[ "$cmd" =~ ^"][[:space:]]+" ]]; then
        local prompt="${cmd#]}"
        prompt="${prompt## }"
        __super_snoofer_executing=1
        command super_snoofer --prompt "$prompt"
        return $?
    elif [[ "$cmd" =~ ^"]][[:space:]]+" ]]; then
        local prompt="${cmd#]]}"
        prompt="${prompt## }"
        __super_snoofer_executing=1
        command super_snoofer --prompt "$prompt" --codestral
        return $?
    fi
    
    # Record successful commands (exclude failures) with history toggled on
    if [[ $? -eq 0 ]]; then
        __super_snoofer_executing=1
        command super_snoofer --check "$cmd ${args[*]}"
        local ret=$?
        __super_snoofer_executing=0
        return $ret
    fi
    
    return 0
}

# Hook into the pre-exec function in ZSH
autoload -Uz add-zsh-hook
add-zsh-hook preexec __super_snoofer_check_command_line
"#;

    fs::write(integration_path, script)?;
    
    Ok(())
}

/// Adds a source directive to the shell configuration file if not already present
/// 
/// # Errors
/// Returns an error if reading from or writing to the shell configuration file fails
fn add_source_directive(zshrc_path: &std::path::Path, integration_path: &std::path::Path) -> Result<()> {
    let integration_path_str = integration_path.to_string_lossy();
    let source_line = format!("source {integration_path_str}");
    
    let mut add_to_zshrc = true;
    
    // Check if the source directive already exists in .zshrc
    if zshrc_path.exists() {
        let zshrc_content = fs::read_to_string(zshrc_path)?;
        if zshrc_content.contains(&source_line) || zshrc_content.contains(&*integration_path_str) {
            add_to_zshrc = false;
        }
    }
    
    // Add the source directive to .zshrc if needed
    if add_to_zshrc {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(zshrc_path)?;
            
        writeln!(file, "\n# Super Snoofer shell integration")?;
        writeln!(file, "{source_line}")?;
    }
    
    Ok(())
}

/// Uninstalls Super Snoofer shell integration
/// 
/// # Errors
/// Returns an error if the uninstallation fails due to file system operations or permission issues
pub fn uninstall_shell_integration() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home_dir.join(".config").join("super_snoofer");
    let integration_path = config_dir.join("shell_integration.zsh");
    let zshrc_path = home_dir.join(".zshrc");

    // Remove the integration file if it exists
    if integration_path.exists() {
        fs::remove_file(&integration_path)?;
    }

    // Remove the source line from .zshrc
    let content = fs::read_to_string(&zshrc_path)?;
    let integration_path_str = integration_path.to_string_lossy();
    let new_content = content
        .lines()
        .filter(|line| !line.contains("Source Super Snoofer integration") && 
                       !line.contains(&*integration_path_str))
        .collect::<Vec<_>>()
        .join("\n");

    // Write the updated content back
    fs::write(&zshrc_path, new_content)?;

    // Try to remove config directory if empty
    if config_dir.exists() {
        if let Ok(entries) = fs::read_dir(&config_dir) {
            if entries.count() == 0 {
                fs::remove_dir(&config_dir)?;
            }
        }
    }

    Ok(())
} 