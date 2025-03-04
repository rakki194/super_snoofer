use anyhow::Result;
use std::{
    fs,
    io::Write,
};

pub mod aliases {
    use anyhow::Result;
    use std::{collections::HashMap, fs};

    pub fn parse_shell_aliases() -> Result<HashMap<String, String>> {
        let mut aliases = HashMap::new();
        
        // Get home directory
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        // Parse .zshrc and related files
        let zsh_files = vec![
            home_dir.join(".zshrc"),
            home_dir.join("toolkit/zsh/core_shell.zsh"),
            home_dir.join("toolkit/zsh/docker.zsh"),
            home_dir.join("toolkit/zsh/git.zsh"),
            home_dir.join("toolkit/zsh/personal.zsh"),
        ];

        for file_path in zsh_files {
            if file_path.exists() {
                let content = fs::read_to_string(&file_path)?;
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("alias ") {
                        if let Some((name, command)) = parse_alias_line(line) {
                            aliases.insert(name, command);
                        }
                    }
                }
            }
        }

        Ok(aliases)
    }

    fn parse_alias_line(line: &str) -> Option<(String, String)> {
        let line = line.trim_start_matches("alias ").trim();
        if let Some((name, command)) = line.split_once('=') {
            let name = name.trim();
            let mut command = command.trim();
            
            // Remove surrounding quotes if present
            if (command.starts_with('\'') && command.ends_with('\'')) || 
               (command.starts_with('"') && command.ends_with('"')) {
                command = &command[1..command.len() - 1];
            }
            
            Some((name.to_string(), command.to_string()))
        } else {
            None
        }
    }
}

pub fn detect_shell_config(alias_name: &str, command: &str) -> Result<(String, String, String)> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let zshrc_path = home_dir.join(".zshrc");
    if zshrc_path.exists() {
        let alias_line = format!("alias {}='{}'", alias_name, command);
        return Ok(("zsh".to_string(), zshrc_path.to_string_lossy().into(), alias_line));
    }
    Err(anyhow::anyhow!("No supported shell config found"))
}

pub fn add_to_shell_config(_shell_type: &str, config_path: &std::path::Path, config: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(config_path)?;
    writeln!(file, "\n{}", config)?;
    Ok(())
}

pub fn install_shell_integration() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let zshrc_path = home_dir.join(".zshrc");

    // Create the integration script
    let script = r#"
# Super Snoofer Integration
__super_snoofer_check_command_line() {
    local cmd="$1"
    
    # Skip empty commands and super_snoofer itself
    [[ -z "$cmd" || "$cmd" =~ ^super_snoofer ]] && return 0
    
    # Handle both > and >> commands
    if [[ "$cmd" =~ ^[>]{1,2}[[:space:]]*$ || "$cmd" =~ ^[>]{1,2}[[:space:]]+ ]]; then
        # Extract the prompt, removing the > or >> prefix and leading whitespace
        local prompt
        if [[ "$cmd" =~ ^">>" ]]; then
            prompt="${cmd#>>}"
            prompt="${prompt## }"
            # Launch TUI even if empty for >>
            command super_snoofer --prompt "${prompt:-}" --codestral
            return $?
        else
            prompt="${cmd#>}"
            prompt="${prompt## }"
            # Launch TUI even if empty for >
            command super_snoofer --prompt "${prompt:-}"
            return $?
        fi
    fi
    
    return 0
}

# Register the hook
autoload -Uz add-zsh-hook
add-zsh-hook preexec __super_snoofer_check_command_line

# Prevent shell redirection for > and >>
function _super_snoofer_gt() {
    if [[ "$BUFFER" =~ ^[>]{1,2}([[:space:]]+.*)?$ ]]; then
        zle accept-line
    else
        zle self-insert
    fi
}

# Register the widget for both > and >>
zle -N _super_snoofer_gt
bindkey ">" _super_snoofer_gt

# Add alias to prevent accidental redirection
alias \>='_super_snoofer_gt'
alias \>>='_super_snoofer_gt'
"#;

    // Append the integration script to .zshrc if it doesn't already exist
    let zshrc_content = fs::read_to_string(&zshrc_path)?;
    if !zshrc_content.contains("# Super Snoofer Integration") {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&zshrc_path)?;
        writeln!(file, "{}", script)?;
    }

    Ok(())
}

pub fn uninstall_shell_integration() -> Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let zshrc_path = home_dir.join(".zshrc");

    // Read the current content
    let content = fs::read_to_string(&zshrc_path)?;

    // Remove our integration block
    let new_content = content
        .lines()
        .take_while(|line| !line.contains("# Super Snoofer Integration"))
        .chain(
            content
                .lines()
                .skip_while(|line| !line.contains("# Super Snoofer Integration"))
                .skip_while(|line| !line.contains("add-zsh-hook preexec __super_snoofer_check_command_line"))
                .skip(1)
        )
        .collect::<Vec<_>>()
        .join("\n");

    // Write the updated content back
    fs::write(&zshrc_path, new_content)?;

    Ok(())
} 