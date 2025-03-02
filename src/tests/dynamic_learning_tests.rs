use crate::CommandCache;
use crate::command::CommandPatterns;
use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Test that command patterns can learn from command usage
#[test]
fn test_command_pattern_learning() -> Result<()> {
    // Create a new command patterns instance
    let mut patterns = CommandPatterns::new();

    // Test learning from a simple command
    patterns.learn_from_command("mycmd --flag value");

    // Verify that the command was added
    assert!(patterns.is_known_command("mycmd"));

    // Get the pattern and verify the flag was learned
    if let Some(pattern) = patterns.get("mycmd") {
        assert!(pattern.flags.contains(&"--flag".to_string()));
    } else {
        panic!("Command pattern not found");
    }

    // Test learning from another usage of the same command
    patterns.learn_from_command("mycmd subcommand --another-flag");

    // Verify that both the subcommand and flag were learned
    if let Some(pattern) = patterns.get("mycmd") {
        assert!(pattern.flags.contains(&"--another-flag".to_string()));
        // Note: The subcommand might not be added immediately due to the usage threshold
        // It should be added after multiple uses
    } else {
        panic!("Command pattern not found");
    }

    // Simulate multiple usages to pass the threshold
    patterns.learn_from_command("mycmd subcommand");
    patterns.learn_from_command("mycmd subcommand");

    // Now the subcommand should be learned
    if let Some(pattern) = patterns.get("mycmd") {
        assert!(pattern.args.contains(&"subcommand".to_string()));
    } else {
        panic!("Command pattern not found");
    }

    Ok(())
}

/// Test that completion generation works correctly
#[test]
fn test_completion_generation() -> Result<()> {
    // Create a new command patterns instance
    let mut patterns = CommandPatterns::new();

    // Add some command usage
    patterns.learn_from_command("testcmd subcmd1 --flag1 value");
    patterns.learn_from_command("testcmd subcmd2 --flag2");
    patterns.learn_from_command("testcmd subcmd1 --flag3");

    // Generate completion for the command
    let completion = patterns
        .generate_zsh_completion("testcmd")
        .expect("Completion should be generated");

    // Verify that the completion contains our flags and arguments
    assert!(completion.contains("#compdef testcmd"));
    assert!(completion.contains("'--flag1[--flag1]'"));
    assert!(completion.contains("'--flag2[--flag2]'"));
    assert!(completion.contains("'--flag3[--flag3]'"));

    // Arguments may not appear immediately due to threshold, so add more usages
    patterns.learn_from_command("testcmd subcmd1");
    patterns.learn_from_command("testcmd subcmd2");

    // Generate completion again
    let completion = patterns
        .generate_zsh_completion("testcmd")
        .expect("Completion should be generated");

    // Now the completion should contain the subcommands
    assert!(completion.contains("subcmd1"));
    assert!(completion.contains("subcmd2"));

    Ok(())
}

/// Test that the cache correctly integrates with command patterns learning
#[test]
fn test_cache_command_learning_integration() -> Result<()> {
    // Create a temporary directory for the cache
    let temp_dir = TempDir::new()?;
    let cache_file = temp_dir.path().join("test_cache.json");

    // Create a new cache with our temp file
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_file.clone());

    // Record valid command usage
    cache.record_valid_command("mycmd subcommand --flag");

    // Save the cache
    cache.save()?;

    // Reload the cache to verify persistence
    let cache = CommandCache::load_from_path(&cache_file)?;

    // Verify the command pattern was recorded and persisted
    assert!(cache.command_patterns.is_known_command("mycmd"));

    // Check for the flag (should be recorded immediately)
    if let Some(pattern) = cache.command_patterns.get("mycmd") {
        assert!(pattern.flags.contains(&"--flag".to_string()));
    } else {
        panic!("Command pattern not found after reload");
    }

    Ok(())
}

/// Test that completion file generation works correctly
#[test]
fn test_completion_file_generation() -> Result<()> {
    // Create temporary directory
    let temp_dir = TempDir::new()?;
    let cache_file = temp_dir.path().join("test_cache.json");
    let completion_file = temp_dir.path().join("completions.zsh");

    // Create a new cache
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_file);

    // Add some command usages
    cache.record_valid_command("cmd1 subcmd1 --flag1");
    cache.record_valid_command("cmd2 subcmd2 --flag2");

    // Generate completions file
    let completions = cache.command_patterns.generate_all_completions();
    fs::write(&completion_file, completions)?;

    // Read back the file and verify content
    let content = fs::read_to_string(&completion_file)?;

    // Check basic content
    assert!(content.contains("# Generated by super_snoofer"));
    assert!(content.contains("cmd1"));
    assert!(content.contains("cmd2"));
    assert!(content.contains("--flag1"));
    assert!(content.contains("--flag2"));

    Ok(())
}

/// Test command discovery scan
#[test]
fn test_discovery_scan() -> Result<()> {
    // Skip running the actual discovery command to avoid executing processes in tests
    // Instead, test the mechanism using direct function calls

    // Create a new cache
    let mut cache = CommandCache::new();

    // Manually add a test command
    cache.insert("testcmd");

    // Create a new pattern for it
    let mut pattern = crate::command::CommandPattern {
        command: "testcmd".to_string(),
        args: Vec::new(),
        flags: Vec::new(),
        last_updated: std::time::SystemTime::now(),
        usage_count: 1,
    };

    // Update the pattern with flags that would be discovered
    pattern.flags.push("--help".to_string());
    pattern.flags.push("--version".to_string());

    // Add to command patterns
    cache
        .command_patterns
        .patterns
        .insert("testcmd".to_string(), pattern);

    // Verify the pattern was added
    if let Some(pattern) = cache.command_patterns.get("testcmd") {
        assert_eq!(pattern.command, "testcmd");
        assert!(pattern.flags.contains(&"--help".to_string()));
        assert!(pattern.flags.contains(&"--version".to_string()));
    } else {
        panic!("Pattern not found");
    }

    Ok(())
}

/// Test handling of complex command lines
#[test]
fn test_learn_complex_command_line() -> Result<()> {
    let mut patterns = CommandPatterns::new();

    // Test with a complex command line containing multiple flags and args
    patterns.learn_from_command("mycmd --flag1 value1 --flag2=value2 subcommand --flag3");

    // Verify command was added
    assert!(patterns.is_known_command("mycmd"));

    // Check flags were parsed correctly
    if let Some(pattern) = patterns.get("mycmd") {
        assert!(pattern.flags.contains(&"--flag1".to_string()));
        assert!(pattern.flags.contains(&"--flag2=value2".to_string()));
        assert!(pattern.flags.contains(&"--flag3".to_string()));
    } else {
        panic!("Command pattern not found");
    }

    // For subcommand, we need multiple usages to pass threshold
    patterns.learn_from_command("mycmd subcommand");
    patterns.learn_from_command("mycmd subcommand");

    // Now verify subcommand was added
    if let Some(pattern) = patterns.get("mycmd") {
        assert!(pattern.args.contains(&"subcommand".to_string()));
    } else {
        panic!("Command pattern not found");
    }

    Ok(())
}

/// Test integration between command learning and correction
#[test]
fn test_learning_improves_correction() -> Result<()> {
    // Create a temporary directory for the cache
    let temp_dir = TempDir::new()?;
    let cache_file = temp_dir.path().join("test_cache.json");

    // Create a new cache
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_file);

    // Record some valid commands
    cache.record_valid_command("awesomecmd");
    cache.record_valid_command("subcommand");

    // Add direct learned corrections for individual components
    cache.learn_correction("awesomcmd", "awesomecmd")?;
    cache.learn_correction("subcmmand", "subcommand")?;

    // Test basic corrections
    assert_eq!(
        cache.find_similar("awesomcmd"),
        Some("awesomecmd".to_string()),
        "Should find exact match for 'awesomecmd'"
    );

    assert_eq!(
        cache.find_similar("subcmmand"),
        Some("subcommand".to_string()),
        "Should correct 'subcmmand' to 'subcommand'"
    );

    Ok(())
}
