#![warn(clippy::all, clippy::pedantic)]

#[cfg(test)]
mod shell_tests {
    use crate::shell::aliases::parse_bash_alias_content;
    use crate::shell::{add_to_shell_config, detect_shell_config};
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_shell_config_detection() -> Result<(), Box<dyn std::error::Error>> {
        let (shell_type, config_path, alias_line) = detect_shell_config("g", "git")?;
        assert_eq!(shell_type, "zsh");
        assert!(Path::new(&config_path).exists() || Path::new(&config_path).parent().map_or(false, |p| p.exists()));
        assert_eq!(alias_line, "alias g='git'");
        Ok(())
    }

    #[test]
    fn test_add_to_shell_config() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for testing
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("test_config");

        // Create an empty config file
        std::fs::write(&config_path, "")?;

        // Add a test alias
        add_to_shell_config("Test Shell", &config_path, "alias g='git'")?;

        // Read the file content
        let content = std::fs::read_to_string(&config_path)?;
        assert!(content.contains("alias g='git'"));

        Ok(())
    }

    #[test]
    fn test_parse_bash_alias_content() {
        let content = r#"
        # Some comment
        alias ll='ls -la'
        alias g="git"
        alias gst='git status'
        "#;

        let mut aliases = HashMap::new();
        parse_bash_alias_content(content, &mut aliases);

        assert_eq!(aliases.get("ll"), Some(&"ls -la".to_string()));
        assert_eq!(aliases.get("g"), Some(&"git".to_string()));
        assert_eq!(aliases.get("gst"), Some(&"git status".to_string()));
    }
}
