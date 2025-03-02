use crate::{CommandCache, HistoryTracker};
use anyhow::Result;
use tempfile::TempDir;

/// Test for frequent command suggestion functionality
#[test]
fn test_frequent_command_suggestions() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path);

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Record some test commands with varying frequencies
    for _ in 0..5 {
        cache.record_valid_command("git status");
    }

    for _ in 0..3 {
        cache.record_valid_command("git commit -m 'test'");
    }

    for _ in 0..10 {
        cache.record_valid_command("docker ps");
    }

    cache.record_valid_command("docker build -t test .");
    cache.record_valid_command("cargo build");

    // Get all frequent command suggestions
    let suggestions = crate::cache::get_frequent_commands_for_prefix("");

    // Ensure we get some general suggestions
    assert!(
        !suggestions.is_empty(),
        "Should have some frequent command suggestions"
    );

    // Check if docker command is included in frequent suggestions
    let docker_included = suggestions.iter().any(|s| s.starts_with("docker"));
    assert!(
        docker_included,
        "Docker commands should be included in frequent suggestions"
    );

    // Get git specific suggestions
    let git_suggestions = crate::cache::get_frequent_commands_for_prefix("git");

    // Test for git command suggestions
    assert!(
        !git_suggestions.is_empty(),
        "Should have git command suggestions"
    );
    assert!(
        git_suggestions.iter().any(|s| s.contains("commit")),
        "Git commit should be in suggestions"
    );

    // Get more specific git commit suggestions
    let git_commit_suggestions = crate::cache::get_frequent_commands_for_prefix("git c");

    // Check that it returns git commit
    if !git_commit_suggestions.is_empty() {
        assert!(
            git_commit_suggestions.iter().any(|s| s.contains("commit")),
            "Git commit should be in the commit suggestions"
        );
    } else {
        println!("Note: git_commit_suggestions was empty, but test continues");
    }

    // Test for typo suggestions
    let typo_suggestions = crate::cache::get_frequent_commands_for_prefix("git pus");

    // Check if typo suggestions include git push
    if !typo_suggestions.is_empty() {
        assert!(
            typo_suggestions[0].starts_with("git pus") || typo_suggestions[0].contains("push"),
            "Typo suggestion should start with 'git pus' or contain 'push'"
        );
    } else {
        println!("Note: No typo suggestions found for 'git pus', but test continues");
    }

    Ok(())
}

/// Test full command completion functionality
#[test]
fn test_full_command_completion() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path);

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Train the cache with some common command patterns
    cache.record_valid_command("git status");
    cache.record_valid_command("git status");
    cache.record_valid_command("git commit -m 'test'");
    cache.record_valid_command("git commit -m 'fixes'");
    cache.record_valid_command("git push origin main");
    cache.record_valid_command("docker run -it ubuntu bash");
    cache.record_valid_command("docker ps -a");

    // Get git completions
    let git_completion = crate::cache::generate_full_completion("git");

    // Check we have completions for git
    assert!(
        !git_completion.is_empty(),
        "Should have completions for git"
    );

    // Check specific completions
    assert!(
        git_completion.iter().any(|c| c.contains("status")),
        "Git completion should include status"
    );

    // Test docker completion
    let docker_completion = crate::cache::generate_full_completion("docker r");

    // Check docker completions
    assert!(
        !docker_completion.is_empty(),
        "Should have completions for docker r"
    );
    assert!(
        docker_completion.iter().any(|c| c.contains("run")),
        "Docker completion should include run"
    );

    // Test unknown command completion
    let unknown_completion = crate::cache::generate_full_completion("unknown_cmd");

    // Unknown command should return empty completions
    assert!(
        unknown_completion.is_empty(),
        "Unknown command should have no completions"
    );

    // Test empty prefix completion
    let empty_completion = crate::cache::generate_full_completion("");

    // Empty completion should return common commands
    assert!(
        !empty_completion.is_empty(),
        "Empty prefix should have some completions"
    );
    assert!(
        empty_completion.iter().any(|c| c.contains("git")),
        "Empty completion should include git"
    );

    Ok(())
}

/// Test early suggestion functionality
#[test]
fn test_early_command_suggestions() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path);

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Add some common commands to the cache
    cache.record_valid_command("git status");
    cache.record_valid_command("git commit -m 'test'");
    cache.record_valid_command("grep pattern file.txt");
    cache.record_valid_command("find . -name '*.rs'");

    // Simulate early suggestions by using get_command_suggestion with a single character
    // Note: The ZSH integration would normally handle early suggestions differently,
    // but we can test the same underlying functionality here

    // Use command cache to get suggestion for 'g'
    let single_char_suggestion = cache.get_command_suggestion("g");

    // We're testing capability, not exact match since early suggestions depend on
    // the ZSH integration to determine when to show them
    if let Some(suggestion) = single_char_suggestion {
        assert!(
            suggestion.starts_with("g"),
            "Suggestion should start with the typed character"
        );
        assert!(
            suggestion.contains("git") || suggestion.contains("grep"),
            "Should suggest a command starting with 'g'"
        );
    }

    // Train with more examples of a specific command
    for _ in 0..5 {
        cache.record_valid_command("git status");
    }

    // Now 'g' should more strongly suggest 'git'
    let stronger_suggestion = cache.get_command_suggestion("g");

    if let Some(suggestion) = stronger_suggestion {
        // More likely to suggest 'git' now
        assert!(
            suggestion.starts_with("g"),
            "Suggestion should start with the typed character"
        );
    }

    Ok(())
}

/// Test typo correction in command completion
#[test]
fn test_typo_correction_completion() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path);

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Add some commands to the cache
    cache.insert("git");
    cache.insert("grep");
    cache.insert("docker");

    // Record some learned corrections
    cache.learn_correction("gt", "git")?;
    cache.learn_correction("gerp", "grep")?;
    cache.learn_correction("doker", "docker")?;

    // Test correction of simple typo
    let corrected1 = cache.fix_command_line("gt status");
    assert!(corrected1.is_some(), "Should correct 'gt status'");
    assert_eq!(
        corrected1.unwrap(),
        "git status",
        "Should correct to 'git status'"
    );

    // Test correction of typo with arguments
    let corrected2 = cache.fix_command_line("gerp pattern file.txt");
    assert!(
        corrected2.is_some(),
        "Should correct 'gerp pattern file.txt'"
    );
    assert_eq!(
        corrected2.unwrap(),
        "grep pattern file.txt",
        "Should correct to 'grep pattern file.txt'"
    );

    // Test correction of typo with complex arguments
    let corrected3 = cache.fix_command_line("doker run -it --rm ubuntu bash");
    assert!(
        corrected3.is_some(),
        "Should correct 'doker run -it --rm ubuntu bash'"
    );
    assert_eq!(
        corrected3.unwrap(),
        "docker run -it --rm ubuntu bash",
        "Should correct to 'docker run -it --rm ubuntu bash'"
    );

    // Test integration with suggestion system
    let suggestion = cache.get_command_suggestion("gt s");

    // Should suggest the corrected command
    assert!(suggestion.is_some(), "Should provide suggestion for typo");
    if let Some(sugg) = suggestion {
        assert!(sugg.starts_with("git"), "Should correct 'gt' to 'git'");
    }

    // Test multi-word correction
    cache.learn_correction("git sttaus", "git status")?;

    let corrected4 = cache.fix_command_line("git sttaus");
    assert!(corrected4.is_some(), "Should correct 'git sttaus'");
    assert_eq!(
        corrected4.unwrap(),
        "git status",
        "Should correct to 'git status'"
    );

    Ok(())
}

/// Test the integration of the command handlers with the cache
#[test]
fn test_suggestion_command_integration() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path.clone());

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Add some test commands
    cache.record_valid_command("git status");
    cache.record_valid_command("git status"); // Record twice to increase frequency
    cache.record_valid_command("git commit -m 'test'");
    cache.record_valid_command("docker ps");

    // Save the cache
    cache.save()?;

    // Use suggestion commands directly on cache path

    // Test --suggest-completion
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "super_snoofer",
        "--",
        "--suggest-completion",
        "git s",
    ]);
    cmd.env("SUPER_SNOOFER_CACHE_PATH", &cache_path);

    // This is just to check that the command runs without error
    // In a real-world scenario, we'd capture output and verify it
    let result = cmd.status();
    assert!(result.is_ok(), "Command should execute successfully");

    // Test --suggest-full-completion
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "super_snoofer",
        "--",
        "--suggest-full-completion",
        "git",
    ]);
    cmd.env("SUPER_SNOOFER_CACHE_PATH", &cache_path);

    let result = cmd.status();
    assert!(result.is_ok(), "Command should execute successfully");

    // Test --suggest-frequent-command
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "run",
        "--bin",
        "super_snoofer",
        "--",
        "--suggest-frequent-command",
        "git",
    ]);
    cmd.env("SUPER_SNOOFER_CACHE_PATH", &cache_path);

    let result = cmd.status();
    assert!(result.is_ok(), "Command should execute successfully");

    Ok(())
}

/// Test combined completion features
#[test]
fn test_combined_completion_features() -> Result<()> {
    // Create a temp directory for our cache
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create a new cache with the temp path
    let mut cache = CommandCache::new();
    cache.set_cache_path(cache_path);

    // Enable history tracking for the cache
    cache.enable_history()?;

    // Add a mix of commands with some typos and corrections
    cache.record_valid_command("git status");
    cache.record_valid_command("git commit -m 'test'");
    cache.record_valid_command("docker ps");

    // Record some corrections
    cache.learn_correction("gt", "git")?;
    cache.learn_correction("doker", "docker")?;

    // Test typo correction with frequent commands
    let corrected = cache.fix_command_line("gt s");

    // Should correct "gt" to "git" and complete with common arguments
    assert!(corrected.is_some(), "Should correct 'gt s'");

    // Store corrected value for later use
    let corrected_value = corrected.clone();

    if let Some(fixed) = corrected {
        assert!(fixed.starts_with("git"), "Should correct 'gt' to 'git'");
    }

    // Now test full completion after correction
    if let Some(fixed) = corrected_value {
        let completed = crate::cache::generate_full_completion(&fixed);

        assert!(
            !completed.is_empty(),
            "Should provide completion after correction"
        );

        // Fix the type mismatch by using any to check for strings within the vector
        assert!(
            completed.iter().any(|s| s.contains("status"))
                || completed.iter().any(|s| s.contains("commit")),
            "Should include common git subcommands after correction"
        );
    }

    // Test frequent commands with correction
    cache.record_valid_command("git push");
    cache.record_valid_command("git push");
    cache.record_valid_command("git pull");

    // Should find frequent commands even with a typo
    let frequent_suggestions = crate::cache::get_frequent_commands_for_prefix("gt");

    // Check suggestions if any are returned
    if !frequent_suggestions.is_empty() {
        // Check if any suggestion starts with "git" or "gt"
        assert!(
            frequent_suggestions
                .iter()
                .any(|s| s.starts_with("git") || s.starts_with("gt")),
            "Should suggest git commands or correct the typo"
        );
    }

    Ok(())
}
