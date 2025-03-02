#[cfg(test)]
mod shell_tests {
    use crate::shell::{detect_shell_config, add_to_shell_config};
    use crate::shell::aliases::parse_bash_alias_content;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;
    
    #[test]
    fn test_detect_shell_config() {
        // Test basic shell detection
        let (shell_type, config_path, alias_line) = detect_shell_config("g", "git").unwrap();
        
        // We can't assert exactly what shell should be detected since it depends on the environment,
        // but we can make sure the function returned something reasonable
        assert!(!shell_type.is_empty(), "Shell type should not be empty");
        assert!(config_path.exists() || config_path.parent().map_or(false, |p| p.exists()), 
                "Config path or its parent directory should exist");
        assert!(alias_line.contains("g") && alias_line.contains("git"), 
                "Alias line should contain both 'g' and 'git'");
    }
    
    #[test]
    fn test_add_to_shell_config() -> anyhow::Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("test_shell_config");
        
        // Create a test shell config
        add_to_shell_config("Test Shell", &config_path, "alias g='git'")?;
        
        // Verify the file was created
        assert!(config_path.exists(), "Config file should have been created");
        
        // Check content
        let content = fs::read_to_string(&config_path)?;
        assert!(content.contains("alias g='git'"), "Config should contain the alias");
        assert!(content.contains("Added by Super Snoofer"), "Config should have the comment");
        
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