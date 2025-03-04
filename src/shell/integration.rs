#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use std::{
    fs,
    io::Write,
};

pub fn install_shell_integration() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let config_dir = home_dir.join(".config").join("super_snoofer");
    let integration_path = config_dir.join("shell_integration.zsh");
    let zshrc_path = home_dir.join(".zshrc");

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir)?;

    // Create the integration script
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
    
    # Handle command not found (but ignore ] and ]] commands)
    if ! command -v "$cmd" >/dev/null 2>&1 && [[ ! "$cmd" =~ '^(\]|\]\])' ]]; then
        __super_snoofer_executing=1
        command super_snoofer -- "$cmd" "${args[@]}"
        return $?
    fi
    
    return 0
}

# Register Super Snoofer hook
autoload -Uz add-zsh-hook
add-zsh-hook preexec __super_snoofer_check_command_line

# Handle ] and ]] input
function _super_snoofer_bracket() {
    # If we have ]] with content
    if [[ "$BUFFER" =~ '^]][[:space:]]+' ]]; then
        zle accept-line
        return
    fi
    
    # If we have ] with content
    if [[ "$BUFFER" =~ '^][[:space:]]+' ]]; then
        zle accept-line
        return
    fi
    
    # If we have an empty ]
    if [[ "$BUFFER" == "]" ]]; then
        # Check if the last character typed was also ]
        if [[ "$LBUFFER" == "]" ]]; then
            BUFFER="]]"
            CURSOR=2  # Set cursor after the ]]
            zle redisplay
            return
        fi
        zle accept-line
        return
    fi
    
    # Default: insert ]
    zle self-insert
}

# Register the widget
zle -N _super_snoofer_bracket
bindkey "]" _super_snoofer_bracket

# Register command not found handler
function command_not_found_handler() {
    local cmd="$1"
    shift
    local args=("$@")
    
    if [[ ! "$cmd" =~ '^(\]|\]\])' ]]; then
        __super_snoofer_executing=1
        # Pass the command directly to super_snoofer without a subcommand
        if [ ${#args[@]} -eq 0 ]; then
            command super_snoofer -- "$cmd"
        else
            command super_snoofer -- "$cmd" "${args[@]}"
        fi
        return $?
    fi
    
    return 127
}"#;

    // Write the integration script to the dedicated file
    fs::write(&integration_path, script)?;

    // Add source line to .zshrc if it doesn't exist
    let source_line = format!("\n# Source Super Snoofer integration\n[ -f {} ] && source {}", 
        integration_path.display(), integration_path.display());
    
    let zshrc_content = fs::read_to_string(&zshrc_path)?;
    if !zshrc_content.contains(&source_line) {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&zshrc_path)?;
        writeln!(file, "{}", source_line)?;
    }

    Ok(())
}

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