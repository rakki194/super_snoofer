#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use std::{fs, io::Write};

/// Installs shell integration for Super Snoofer
///
/// # Errors
/// Returns an error if the shell integration installation fails due to file system operations or permission issues
pub fn install_shell_integration() -> Result<()> {
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
    let script = r###"# Super Snoofer Integration - Fixed Version v2
# Flag to prevent double execution
typeset -g __super_snoofer_executing=0

function __super_snoofer_check_command_line() {
    # Get the raw command line as passed to preexec
    local raw_cmd="$1"
    # Skip if we're already executing a super_snoofer command
    if (( __super_snoofer_executing )); then
        __super_snoofer_executing=0
        return 0
    fi
    
    # Check for pipes, redirects, and other special shell syntax that should NOT be intercepted
    if [[ "$raw_cmd" == *"|"* || "$raw_cmd" == *">"* || "$raw_cmd" == *"<"* || 
          "$raw_cmd" == *"&"* || "$raw_cmd" == *";"* || "$raw_cmd" == *"&&"* || 
          "$raw_cmd" == *"||"* ]]; then
        # Let the shell handle these complex commands normally
        return 0
    fi
    
    # Get just the first word (command name) for basic checks
    local cmd=$(echo "$raw_cmd" | awk '{print $1}')
    
    # Handle the special Super Snoofer AI commands
    if [[ "$cmd" == "]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt ""
        return 1  # Prevent original command execution
    elif [[ "$cmd" == "]]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt "" --codestral
        return 1  # Prevent original command execution
    elif [[ "$raw_cmd" =~ ^"][[:space:]]+" ]]; then
        local prompt="${raw_cmd#]}"
        prompt="${prompt## }"
        __super_snoofer_executing=1
        command super_snoofer --prompt "$prompt"
        return 1  # Prevent original command execution
    elif [[ "$raw_cmd" =~ ^"]][[:space:]]+" ]]; then
        local prompt="${raw_cmd#]]}"
        prompt="${prompt## }"
        __super_snoofer_executing=1
        command super_snoofer --prompt "$prompt" --codestral
        return 1  # Prevent original command execution
    fi
    
    # Skip checking for typos in these common commands
    if [[ "$cmd" =~ ^(ls|cd|pwd|man|echo|cat|grep|find|git|vim|nvim|code|python|python3|cargo|rm|cp|mv|mkdir|touch|chmod|npm|yarn|go|make|docker|kubectl|ssh|curl|wget)$ ]]; then
        return 0
    fi
    
    # Only process commands that don't exist
    if type "$cmd" > /dev/null 2>&1; then
        return 0
    fi
    
    # At this point, we have a simple command that doesn't exist
    # Let the command_not_found_handler take care of it
    return 0
}

# Function to get suggestions from super_snoofer for any command
function __super_snoofer_get_suggestion() {
    local cmd="$1"
    local result
    
    # Skip certain command patterns that might cause issues
    if [[ "$cmd" =~ "failed with status" || "$cmd" =~ "exit status" || "$cmd" =~ "Command failed" ]]; then
        echo "$cmd"
        return
    fi
    
    # Skip system commands and common utilities that don't need suggestions
    if [[ "$cmd" =~ ^alias || "$cmd" =~ ^which || "$cmd" =~ ^echo || "$cmd" =~ ^compgen || \
          "$cmd" =~ ^nvim || "$cmd" =~ ^vim || "$cmd" =~ ^cd || "$cmd" =~ ^ls || "$cmd" =~ ^git || \
          "$cmd" =~ ^python || "$cmd" =~ ^python3 || "$cmd" =~ ^pip || "$cmd" =~ ^pip3 ]]; then
        echo "$cmd"
        return
    fi
    
    # Special handling for cargo subcommands
    if [[ "$cmd" =~ ^cargo[[:space:]]+ ]]; then
        local subcmd=$(echo "$cmd" | cut -d' ' -f2)
        local cargo_args=$(echo "$cmd" | cut -d' ' -f3-)
        
        # Common cargo command corrections
        case "$subcmd" in
            "urn") 
                echo "cargo run $cargo_args"
                return
                ;;
            "biuld") 
                echo "cargo build $cargo_args"
                return
                ;;
            "cehck") 
                echo "cargo check $cargo_args"
                return
                ;;
            "tset") 
                echo "cargo test $cargo_args"
                return
                ;;
            "isntall") 
                echo "cargo install $cargo_args"
                return
                ;;
            # No correction for valid cargo commands
            "run"|"build"|"test"|"check"|"update"|"clean"|"doc"|"publish"|"install")
                echo "$cmd"
                return
                ;;
        esac
    fi
    
    # Call super_snoofer in quiet mode to check if this is a known typo
    result=$(super_snoofer full-command "$cmd" 2>/dev/null)
    
    # If super_snoofer returned a valid suggestion, use it
    if [[ $? -eq 0 && -n "$result" && "$result" != "$cmd" && "$result" != *"failed with status"* && "$result" != *"exit status"* ]]; then
        echo "$result"
    else
        # No suggestion found or invalid suggestion
        echo "$cmd"
    fi
}

# Define shell functions for ] and ]] to avoid "command not found" errors
function ]() {
    __super_snoofer_executing=1
    command super_snoofer --prompt ""
}

# Need to use aliases instead of functions for ]] due to syntax limitations
alias ']]'='__super_snoofer_executing=1; command super_snoofer --prompt "" --codestral'

# Hook into the pre-exec function in ZSH
autoload -Uz add-zsh-hook
add-zsh-hook preexec __super_snoofer_check_command_line

# Save the original command_not_found_handler if it exists
if (( ${+functions[command_not_found_handler]} )); then
    functions[__original_command_not_found_handler]=$functions[command_not_found_handler]
fi

# Super Snoofer command-not-found handler
# This only runs when a command truly does not exist
command_not_found_handler() {
    local cmd="$1"
    shift
    
    # Skip handling for empty commands
    if [[ -z "$cmd" ]]; then
        return 127
    fi
    
    # Special case for ] and ]] if they somehow made it here
    if [[ "$cmd" == "]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt ""
        return 0
    elif [[ "$cmd" == "]]" ]]; then
        __super_snoofer_executing=1
        command super_snoofer --prompt "" --codestral
        return 0
    fi
    
    # For all other commands, use super_snoofer to help
    __super_snoofer_executing=1
    if [ $# -eq 0 ]; then
        command super_snoofer -- "$cmd"
    else
        command super_snoofer -- "$cmd" "$@"
    fi
    return $?
}
"###;

    fs::write(integration_path, script)?;

    Ok(())
}

/// Adds a source directive to the shell configuration file if not already present
///
/// # Errors
/// Returns an error if reading from or writing to the shell configuration file fails
fn add_source_directive(
    zshrc_path: &std::path::Path,
    integration_path: &std::path::Path,
) -> Result<()> {
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
    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
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
        .filter(|line| {
            !line.contains("Source Super Snoofer integration")
                && !line.contains(&*integration_path_str)
        })
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

pub fn get_shell_integration(shell: &str) -> Result<String> {
    match shell {
        "zsh" => {
            let script = format!(
                r#"
# Super Snoofer command-not-found handler
command_not_found_handler() {{
    local cmd="$1"
    shift
    if [ -n "$cmd" ]; then
        if [ $# -eq 0 ]; then
            command super_snoofer -- "$cmd"
        else
            command super_snoofer -- "$cmd" "$@"
        fi
        return $?
    fi
    return 127
}}
"#
            );
            Ok(script)
        }
        "bash" => {
            let script = format!(
                r#"
# Super Snoofer command-not-found handler
command_not_found_handle() {{
    local cmd="$1"
    shift
    if [ -n "$cmd" ]; then
        if [ $# -eq 0 ]; then
            command super_snoofer -- "$cmd"
        else
            command super_snoofer -- "$cmd" "$@"
        fi
        return $?
    fi
    return 127
}}
"#
            );
            Ok(script)
        }
        _ => Err(anyhow::anyhow!("Unsupported shell: {}", shell)),
    }
}
