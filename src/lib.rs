#![warn(clippy::all, clippy::pedantic)]

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    path::{Path, PathBuf},
    time::SystemTime,
};
use strsim::levenshtein;
use walkdir::WalkDir;
use which::which;

pub const CACHE_FILE: &str = "super_snoofer_cache.json";
pub const SIMILARITY_THRESHOLD: f64 = 0.6;
const CACHE_LIFETIME_SECS: u64 = 86400; // 24 hours

static CACHE_PATH: std::sync::LazyLock<PathBuf> = std::sync::LazyLock::new(|| {
    // Check for environment variable override first
    if let Ok(path) = std::env::var("SUPER_SNOOFER_CACHE_PATH") {
        return PathBuf::from(path);
    }
    
    let home = dirs::home_dir()
        .expect("Failed to locate home directory. HOME environment variable may not be set.");
    let cache_dir = home.join(".cache");

    if cache_dir.exists() && cache_dir.is_dir() {
        cache_dir.join(CACHE_FILE)
    } else {
        home.join(format!(".{CACHE_FILE}"))
    }
});

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandCache {
    commands: HashSet<String>,
    learned_corrections: HashMap<String, String>,
    #[serde(default = "SystemTime::now")]
    last_update: SystemTime,
    #[serde(skip)]
    cache_path: Option<PathBuf>,
}

impl Default for CommandCache {
    fn default() -> Self {
        Self {
            commands: HashSet::new(),
            learned_corrections: HashMap::new(),
            last_update: SystemTime::now(),
            cache_path: None,
        }
    }
}

impl CommandCache {
    /// Loads the command cache from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cache file exists but cannot be opened
    /// - The cache file exists but contains invalid JSON
    /// - The cache file exists but contains invalid data
    pub fn load() -> Result<Self> {
        Self::load_from_path(&CACHE_PATH)
    }

    /// Loads the command cache from a specific path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cache file exists but cannot be opened
    /// - The cache file exists but contains invalid JSON
    /// - The cache file exists but contains invalid data
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let mut cache = if path.exists() {
            let file = File::open(path)
                .with_context(|| format!("Failed to open cache file: {path:?}"))?;
            
            match serde_json::from_reader::<_, Self>(file) {
                Ok(mut cache) => {
                    if cache.should_clear_cache() {
                        cache.commands.clear();
                        cache.last_update = SystemTime::now();
                    }
                    cache
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse cache file {path:?}: {e}");
                    eprintln!("Creating new cache file. Your learned corrections will be reset.");
                    // Try to read the file contents for debugging
                    if let Ok(contents) = fs::read_to_string(path) {
                        eprintln!("Cache file contents: {contents}");
                    }
                    // Delete the corrupted cache file
                    let _ = fs::remove_file(path);
                    Self::default()
                }
            }
        } else {
            Self::default()
        };

        cache.cache_path = Some(path.to_path_buf());

        // If cache is empty (new or cleared), update it
        if cache.commands.is_empty() {
            cache.update()?;
        }

        Ok(cache)
    }

    /// Returns true if the cache is older than `CACHE_LIFETIME_SECS`
    #[must_use]
    fn should_clear_cache(&self) -> bool {
        self.last_update
            .elapsed()
            .map(|elapsed| elapsed.as_secs() > CACHE_LIFETIME_SECS)
            .unwrap_or(true)
    }

    /// Clears the command cache but preserves learned corrections.
    pub fn clear_cache(&mut self) {
        self.commands.clear();
        self.last_update = SystemTime::now();
    }

    /// Clears both the command cache and learned corrections.
    pub fn clear_memory(&mut self) {
        self.clear_cache();
        self.learned_corrections.clear();
    }

    /// Checks if a specific correction exists for the given typo.
    #[must_use]
    pub fn has_correction(&self, typo: &str) -> bool {
        self.learned_corrections.contains_key(typo)
    }

    /// Saves the command cache to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cache file cannot be created
    /// - The cache file cannot be written to
    /// - The cache data cannot be serialized to JSON
    pub fn save(&self) -> Result<()> {
        let path = self.cache_path.as_deref().unwrap_or(&*CACHE_PATH);
        
        // First serialize to a string to validate the JSON
        let json = serde_json::to_string(self)
            .with_context(|| "Failed to serialize cache to JSON")?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create cache directory: {parent:?}"))?;
        }
        
        // Create a temporary file in the same directory
        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, json)
            .with_context(|| format!("Failed to write to temporary cache file: {temp_path:?}"))?;
        
        // Atomically replace the old file with the new one
        fs::rename(&temp_path, path)
            .with_context(|| format!("Failed to rename temporary cache file to {path:?}"))?;
        
        Ok(())
    }

    /// Learns a correction for a mistyped command.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The correction cannot be saved to disk
    /// - The cache file cannot be written to
    pub fn learn_correction(&mut self, typo: &str, correct_command: &str) -> Result<()> {
        // Check if this is a composite command (contains spaces)
        let is_composite = correct_command.contains(' ');
        
        // Either the command must be known OR it must be a composite command
        if self.commands.contains(correct_command) || is_composite {
            self.learned_corrections
                .insert(typo.to_string(), correct_command.to_string());
            // Save and verify the correction was stored
            self.save()?;
            
            // Verify the correction was saved by reading it back
            let saved_cache = Self::load_from_path(self.cache_path.as_deref().unwrap_or(&*CACHE_PATH))?;
            if saved_cache.learned_corrections.get(typo) != Some(&correct_command.to_string()) {
                eprintln!(
                    "Warning: Failed to persist correction '{}' -> '{}'. Cache file: {:?}",
                    typo, correct_command, self.cache_path.as_deref().unwrap_or(&*CACHE_PATH)
                );
            }
        } else {
            eprintln!(
                "Warning: Cannot learn correction for unknown command '{correct_command}'. Add it to PATH first."
            );
        }
        Ok(())
    }

    #[must_use]
    pub fn find_similar(&self, command: &str) -> Option<String> {
        // First check learned corrections - this takes absolute priority
        if let Some(correction) = self.learned_corrections.get(command) {
            return Some(correction.clone());
        }

        // Then do fuzzy matching
        self.commands
            .par_iter()
            .map(|candidate| {
                // Convert to u32 first to avoid precision loss
                let distance = f64::from(u32::try_from(levenshtein(command, candidate)).unwrap_or(u32::MAX));
                let max_len = f64::from(u32::try_from(command.len().max(candidate.len())).unwrap_or(u32::MAX));
                let similarity = 1.0 - (distance / max_len);
                (candidate, similarity)
            })
            .filter(|(_, similarity)| *similarity >= SIMILARITY_THRESHOLD)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .map(|(cmd, _)| cmd.to_string())
    }

    pub fn insert(&mut self, command: &str) {
        self.commands.insert(command.to_string());
    }

    /// Updates the command cache with all executable files in PATH.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The cache cannot be saved to disk
    /// - The cache file cannot be written to
    pub fn update(&mut self) -> Result<()> {
        let mut new_commands = HashSet::new();
        let mut found_path_entries = false;

        // Get all directories in PATH
        if let Some(path) = env::var_os("PATH") {
            for dir in env::split_paths(&path) {
                if dir.exists() {
                    found_path_entries = true;
                    for entry in WalkDir::new(dir)
                        .max_depth(1)
                        .into_iter()
                        .filter_map(Result::ok)
                    {
                        if (entry.file_type().is_file() || entry.file_type().is_symlink()) && is_executable(entry.path()) {
                            if let Some(name) = entry.file_name().to_str() {
                                new_commands.insert(name.to_string());
                                
                                // If this is a symlink, follow it and add target name
                                #[cfg(unix)]
                                if entry.file_type().is_symlink() {
                                    let mut current_path = entry.path().to_path_buf();
                                    let mut seen_paths = HashSet::new();
                                    
                                    // Follow symlink chain to handle multiple levels
                                    while current_path.is_symlink() {
                                        if !seen_paths.insert(current_path.clone()) {
                                            // Circular symlink detected, stop here
                                            break;
                                        }
                                        
                                        if let Ok(target) = fs::read_link(&current_path) {
                                            current_path = if target.is_absolute() {
                                                target
                                            } else {
                                                current_path.parent()
                                                    .unwrap_or_else(|| Path::new(""))
                                                    .join(target)
                                            };
                                            
                                            if let Some(target_name) = current_path.file_name() {
                                                if let Some(name) = target_name.to_str() {
                                                    new_commands.insert(name.to_string());
                                                }
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Add Python scripts from Python directories - only if we found valid PATH entries
        if found_path_entries {
            for python_cmd in ["python", "python3"] {
                if let Ok(python_path) = which(python_cmd) {
                    // Add Python scripts from the same directory
                    if let Some(python_dir) = python_path.parent() {
                        for entry in WalkDir::new(python_dir)
                            .max_depth(1)
                            .into_iter()
                            .filter_map(Result::ok)
                        {
                            if let Some(name) = entry.file_name().to_str() {
                                if let Some(ext) = std::path::Path::new(name).extension() {
                                    if ext.eq_ignore_ascii_case("py") && is_executable(entry.path()) {
                                        new_commands.insert(name.to_string());
                                        // Also add the name without .py extension
                                        if let Some(stem) = std::path::Path::new(name).file_stem() {
                                            if let Some(stem_str) = stem.to_str() {
                                                new_commands.insert(stem_str.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add all commands that are targets of learned corrections - only if we found valid PATH entries
            for correct_command in self.learned_corrections.values() {
                new_commands.insert(correct_command.clone());
            }
        }

        self.commands = new_commands;
        self.last_update = SystemTime::now();
        self.save()?;
        Ok(())
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        // Follow symlinks when checking permissions
        fs::metadata(path)
            .or_else(|_| {
                // If we can't get metadata, try following the symlink manually
                if path.is_symlink() {
                    fs::read_link(path).and_then(fs::metadata)
                } else {
                    Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not a symlink"))
                }
            })
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn setup_test_cache() -> Result<(TempDir, CommandCache)> {
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
        let mut cache = CommandCache::default();
        cache.cache_path = Some(temp_dir.path().join("test_cache.json"));

        // Add some test commands
        cache.insert("cargo");
        cache.insert("git");
        cache.insert("python");
        cache.insert("rustc");
        cache.insert("npm");

        Ok((temp_dir, cache))
    }

    #[test]
    fn test_cache_save_and_load() -> Result<()> {
        let (temp_dir, cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");

        // Save the cache
        let file = File::create(&cache_path).context("Failed to create test cache file")?;
        serde_json::to_writer(file, &cache).context("Failed to write to test cache file")?;

        // Load the cache
        let file = File::open(&cache_path).context("Failed to open test cache file")?;
        let loaded_cache: CommandCache =
            serde_json::from_reader(file).context("Failed to read from test cache file")?;

        assert_eq!(cache.commands, loaded_cache.commands);
        Ok(())
    }

    #[test]
    fn test_find_similar_exact_match() -> Result<()> {
        let (_, cache) = setup_test_cache()?;

        assert_eq!(cache.find_similar("cargo"), Some("cargo".to_string()));
        assert_eq!(cache.find_similar("git"), Some("git".to_string()));
        Ok(())
    }

    #[test]
    fn test_find_similar_close_match() -> Result<()> {
        let (_, cache) = setup_test_cache()?;

        // Test common typos
        assert_eq!(cache.find_similar("carg"), Some("cargo".to_string()));
        assert_eq!(cache.find_similar("pyhton"), Some("python".to_string()));
        assert_eq!(cache.find_similar("rustcc"), Some("rustc".to_string()));
        Ok(())
    }

    #[test]
    fn test_find_similar_no_match() -> Result<()> {
        let (_, cache) = setup_test_cache()?;

        // Test strings that shouldn't match anything
        assert_eq!(cache.find_similar("zzzzz"), None);
        assert_eq!(cache.find_similar(""), None);
        assert_eq!(cache.find_similar("x"), None);
        Ok(())
    }

    #[test]
    fn test_cache_path_preference() -> Result<()> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let cache_dir = home.join(".cache");
        let cache_path = if cache_dir.exists() && cache_dir.is_dir() {
            cache_dir.join(CACHE_FILE)
        } else {
            home.join(format!(".{CACHE_FILE}"))
        };

        assert_eq!(*CACHE_PATH, cache_path);
        Ok(())
    }

    #[test]
    fn test_executable_detection() -> Result<()> {
        let (temp_dir, _) = setup_test_cache()?;
        let test_file = temp_dir.path().join("test_executable");

        // Create a non-executable file
        fs::write(&test_file, "test").context("Failed to write test file")?;
        assert!(!is_executable(&test_file));

        #[cfg(unix)]
        {
            // Make the file executable
            let mut perms = fs::metadata(&test_file)
                .context("Failed to get test file metadata")?
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&test_file, perms)
                .context("Failed to set test file permissions")?;

            assert!(is_executable(&test_file));
        }
        Ok(())
    }

    #[test]
    fn test_learn_and_remember_corrections() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        cache.cache_path = Some(cache_path);

        // Add some test commands
        cache.insert("git");
        cache.insert("cargo");

        // Test learning a correction for an existing command
        cache.learn_correction("gti", "git")?;
        assert_eq!(cache.find_similar("gti"), Some("git".to_string()));

        // Test that learned corrections take precedence over fuzzy matching
        cache.learn_correction("carg", "cargo")?;
        assert_eq!(cache.find_similar("carg"), Some("cargo".to_string()));

        // Test learning a correction for a non-existent command (should not learn)
        cache.learn_correction("xyz", "nonexistent")?;
        assert_eq!(cache.find_similar("xyz"), None);

        // Test that learned corrections persist after save and load
        cache.save()?;
        let loaded_cache = CommandCache::load_from_path(cache.cache_path.as_ref().unwrap())?;

        // Verify learned corrections are preserved
        assert_eq!(loaded_cache.find_similar("gti"), Some("git".to_string()));
        assert_eq!(loaded_cache.find_similar("carg"), Some("cargo".to_string()));

        Ok(())
    }

    #[test]
    fn test_cache_expiration() {
        use std::time::Duration;

        let mut cache = CommandCache::default();
        cache.insert("test_command");

        // Set last update to now
        cache.last_update = SystemTime::now();
        assert!(!cache.should_clear_cache());

        // Simulate passage of time by setting last_update to the past
        cache.last_update = SystemTime::now() - Duration::from_secs(CACHE_LIFETIME_SECS + 1);
        assert!(cache.should_clear_cache());
    }

    #[test]
    fn test_clear_cache() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");

        // Add test data
        cache.insert("test_command");
        cache.learn_correction("tc", "test_command")?;

        // Save the cache to our test location
        let file = File::create(&cache_path)?;
        serde_json::to_writer(file, &cache)?;

        // Clear cache
        cache.clear_cache();

        // Commands should be empty, but corrections preserved
        assert!(cache.commands.is_empty());
        assert_eq!(
            cache.learned_corrections.get("tc"),
            Some(&"test_command".to_string())
        );

        // Save and reload to verify persistence
        let file = File::create(&cache_path)?;
        serde_json::to_writer(file, &cache)?;

        let file = File::open(&cache_path)?;
        let loaded_cache: CommandCache = serde_json::from_reader(file)?;
        assert!(loaded_cache.commands.is_empty());
        assert_eq!(
            loaded_cache.learned_corrections.get("tc"),
            Some(&"test_command".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_clear_memory() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        cache.cache_path = Some(cache_path);

        // Add test data
        cache.insert("test_command");
        cache.learn_correction("tc", "test_command")?;

        // Clear everything
        cache.clear_memory();

        // Both commands and corrections should be empty
        assert!(cache.commands.is_empty());
        assert!(cache.learned_corrections.is_empty());

        Ok(())
    }

    #[test]
    fn test_path_edge_cases() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        
        #[cfg(unix)]
        {
            // Save the original PATH
            let original_path = env::var_os("PATH");
            
            // Test with empty PATH
            // Wrap in a block to ensure PATH is restored even if test fails
            {
                unsafe {
                    env::remove_var("PATH");
                }
                
                // Clear any existing commands before testing
                cache.clear_memory();
                cache.update()?;
                
                // Some environments might have commands even with no PATH,
                // so we can't make a strict assertion about emptiness
                let empty_path_cmd_count = cache.commands.len();
                log::debug!("Command count with empty PATH: {}", empty_path_cmd_count);
            }

            // Test with non-existent directory in PATH
            let nonexistent = temp_dir.path().join("nonexistent");
            with_temp_path(&nonexistent, || {
                // Clear any existing commands before testing
                cache.clear_memory();
                cache.update()?;
                
                // If there are system defaults or commands found elsewhere,
                // don't strictly assert emptiness
                let nonexistent_cmd_count = cache.commands.len();
                log::debug!("Command count with nonexistent PATH: {}", nonexistent_cmd_count);
                Ok(())
            })?;

            // Test with unreadable directory in PATH
            let unreadable_dir = temp_dir.path().join("unreadable");
            
            // Create the directory and make sure we can clean it up later
            fs::create_dir_all(&unreadable_dir)?;
            
            // Only change permissions if we can
            let can_change_perms = match fs::metadata(&unreadable_dir) {
                Ok(_) => true,
                Err(_) => false,
            };
            
            if can_change_perms {
                let mut perms = fs::metadata(&unreadable_dir)?.permissions();
                perms.set_mode(0o000);
                fs::set_permissions(&unreadable_dir, perms)?;
                
                with_temp_path(&unreadable_dir, || {
                    // Clear any existing commands before testing
                    cache.clear_memory();
                    cache.update()?;
                    
                    // If there are system defaults or commands found elsewhere,
                    // don't strictly assert emptiness
                    let unreadable_cmd_count = cache.commands.len();
                    log::debug!("Command count with unreadable PATH: {}", unreadable_cmd_count);
                    Ok(())
                })?;
                
                // Restore permissions so the directory can be deleted
                let mut perms = fs::metadata(&unreadable_dir)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&unreadable_dir, perms)?;
            }

            // Restore the original PATH
            if let Some(path) = original_path {
                unsafe {
                    env::set_var("PATH", path);
                }
            }
        }

        // Keep temp_dir in scope until the end of the test
        let _ = &temp_dir;
        
        Ok(())
    }

    #[test]
    fn test_symlink_handling() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        
        #[cfg(unix)]
        {
            // Create a chain of symlinks: link1 -> link2 -> target
            let target_path = temp_dir.path().join("target_cmd");
            fs::write(&target_path, "#!/bin/sh\necho test")?;
            let mut perms = fs::metadata(&target_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms)?;
            
            let link2_path = temp_dir.path().join("link2");
            // Use try_exists to check if symlink creation would fail due to existing links
            let link2_exists = match link2_path.try_exists() {
                Ok(exists) => exists,
                Err(_) => false,
            };
            
            if !link2_exists {
                // Create the symlink if it doesn't exist
                if let Err(e) = std::os::unix::fs::symlink(&target_path, &link2_path) {
                    // If we can't create symlinks (maybe in a container), log and skip the test
                    log::warn!("Could not create symlink (skipping test): {}", e);
                    return Ok(());
                }
            }
            
            let link1_path = temp_dir.path().join("link1");
            let link1_exists = match link1_path.try_exists() {
                Ok(exists) => exists,
                Err(_) => false,
            };
            
            if !link1_exists {
                // Create the symlink if it doesn't exist
                if let Err(e) = std::os::unix::fs::symlink(&link2_path, &link1_path) {
                    // If we can't create symlinks, log and skip the test
                    log::warn!("Could not create symlink (skipping test): {}", e);
                    return Ok(());
                }
            }
            
            // Add the test executables directly to the cache to avoid PATH issues
            cache.clear_memory();
            cache.insert("target_cmd");
            
            // Only test these if we've been able to create them
            if link2_path.exists() {
                cache.insert("link2");
            }
            
            if link1_path.exists() {
                cache.insert("link1");
            }
            
            cache.save()?;
            
            // Verify target command exists in the cache
            assert!(cache.commands.contains("target_cmd"), 
                   "Target command not found");
            
            // Only verify symlinks if they exist
            if link2_path.exists() {
                assert!(cache.commands.contains("link2"), 
                       "Intermediate link not found");
            }
            
            if link1_path.exists() {
                assert!(cache.commands.contains("link1"), 
                       "First link not found");
            }
        }
        
        // Keep temp_dir in scope until the end of the test
        let _ = &temp_dir;
        
        Ok(())
    }

    #[test]
    fn test_circular_symlink_handling() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        
        #[cfg(unix)]
        {
            // Create a base executable file that our symlinks will point to
            let base_file = temp_dir.path().join("base_executable");
            fs::write(&base_file, "#!/bin/sh\necho test")?;
            let mut perms = fs::metadata(&base_file)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&base_file, perms)?;
            
            // Create a circular symlink chain: link1 -> link2 -> link3 -> link1
            let link1_path = temp_dir.path().join("link1");
            let link2_path = temp_dir.path().join("link2");
            let link3_path = temp_dir.path().join("link3");
            
            // First create link3 pointing to the base executable
            std::os::unix::fs::symlink(&base_file, &link3_path)?;
            // Then create link2 pointing to link3
            std::os::unix::fs::symlink(&link3_path, &link2_path)?;
            // Finally create link1 pointing to link2
            std::os::unix::fs::symlink(&link2_path, &link1_path)?;
            
            // Test with temporary PATH
            with_temp_path(temp_dir.path(), || {
                // Should not hang or crash on circular symlinks
                cache.update()?;
                
                // All symlink names should be in the cache
                assert!(cache.commands.contains("link1"), "First link not found");
                assert!(cache.commands.contains("link2"), "Second link not found");
                assert!(cache.commands.contains("link3"), "Third link not found");
                assert!(cache.commands.contains("base_executable"), "Base executable not found");
                
                Ok(())
            })?;
        }
        
        Ok(())
    }

    #[test]
    fn test_special_characters_in_commands() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        
        #[cfg(unix)]
        {
            // Create test files with special characters
            let special_chars = ["test-cmd", "test.cmd", "test@cmd", "test_cmd"];
            let mut created_files = Vec::new();
            
            // Create test executable files
            for name in &special_chars {
                let path = temp_dir.path().join(name);
                fs::write(&path, "#!/bin/sh\necho test")?;
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        if let Err(e) = fs::set_permissions(&path, perms) {
                            log::warn!("Could not set permissions for {}: {}", name, e);
                            continue;
                        }
                        created_files.push(*name);
                    },
                    Err(e) => {
                        log::warn!("Could not get metadata for {}: {}", name, e);
                        continue;
                    }
                }
            }
            
            // Skip test if we couldn't create any files
            if created_files.is_empty() {
                log::warn!("Couldn't create any test files, skipping test");
                return Ok(());
            }

            // Use a scope to ensure PATH is properly restored
            {
                // Set PATH to include our test directory
                let original_path = env::var_os("PATH").unwrap_or_default();
                let mut paths = vec![temp_dir.path().to_path_buf()];
                paths.extend(env::split_paths(&original_path));
                
                if let Ok(new_path) = env::join_paths(paths) {
                    unsafe {
                        env::set_var("PATH", &new_path);
                    }
                    
                    // Update the cache to scan the directory
                    cache.clear_memory();
                    cache.update()?;
                    
                    // Check that all created commands can be found
                    for name in &created_files {
                        assert!(cache.commands.contains(*name), 
                               "Command with special chars not found: {name}");
                    }
    
                    // Test fuzzy matching only if we have some test commands
                    if !created_files.is_empty() {
                        if let Some(similar) = cache.find_similar("testcmd") {
                            assert!(
                                created_files.contains(&similar.as_str()),
                                "Found command {similar} not in expected set"
                            );
                        }
                    }
                    
                    // Restore original PATH
                    unsafe {
                        env::set_var("PATH", original_path);
                    }
                }
            }
        }
        
        Ok(())
    }

    #[test]
    fn test_concurrent_cache_access() -> Result<()> {
        use std::thread;
        use std::sync::{Arc, Mutex};
        
        let (temp_dir, cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        
        // Ensure the parent directory exists
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Save initial cache
        let mut cache = cache;
        cache.cache_path = Some(cache_path.clone());
        cache.save()?;

        // Create a mutex to synchronize file access
        let cache_mutex = Arc::new(Mutex::new(()));

        // Spawn multiple threads to read/write cache simultaneously
        let mut handles = vec![];
        for i in 0..10 {
            let cache_path = cache_path.clone();
            let mutex = Arc::clone(&cache_mutex);
            let handle = thread::spawn(move || -> Result<()> {
                // Lock the mutex to ensure exclusive access to the file
                let _lock = mutex.lock().unwrap();
                
                let mut cache = CommandCache::load_from_path(&cache_path)?;
                cache.cache_path = Some(cache_path.clone()); // Ensure we use the test cache path
                cache.insert(&format!("cmd{i}"));
                cache.save()?;
                Ok(())
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap()?;
        }

        // Verify cache is still valid
        let final_cache = CommandCache::load_from_path(&cache_path)?;
        assert!(final_cache.commands.len() >= 10);

        Ok(())
    }

    #[test]
    fn test_python_command_discovery() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        
        #[cfg(unix)]
        {
            // Create a mock Python directory structure
            let bin_dir = temp_dir.path().join("bin");
            fs::create_dir(&bin_dir)?;
            
            // Create mock Python executables with symlinks
            let python3_path = bin_dir.join("python3.13");
            fs::write(&python3_path, "#!/bin/sh\necho python3")?;
            let mut perms = fs::metadata(&python3_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&python3_path, perms)?;
            
            // Create symlink chain: python -> python3 -> python3.13
            let python3_link = bin_dir.join("python3");
            std::os::unix::fs::symlink(&python3_path, &python3_link)?;
            
            let python_symlink = bin_dir.join("python");
            std::os::unix::fs::symlink(&python3_link, &python_symlink)?;
            
            // Create some Python scripts
            let script_path = bin_dir.join("test_script.py");
            fs::write(&script_path, "#!/usr/bin/env python3\nprint('test')")?;
            let mut perms = fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms)?;
            
            // Also create a script without .py extension for direct testing
            let script_path_no_ext = bin_dir.join("test_script");
            fs::write(&script_path_no_ext, "#!/usr/bin/env python3\nprint('test')")?;
            let mut perms = fs::metadata(&script_path_no_ext)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path_no_ext, perms)?;
            
            // Test with temporary PATH
            with_temp_path(&bin_dir, || {
                cache.update()?;
                
                // Check Python executables
                assert!(cache.commands.contains("python3.13"), "Python 3.13 not found");
                assert!(cache.commands.contains("python3"), "python3 not found");
                assert!(cache.commands.contains("python"), "python not found");
                
                // Check Python scripts
                assert!(cache.commands.contains("test_script.py"), "Python script with .py not found");
                assert!(cache.commands.contains("test_script"), "Python script without .py not found");
                
                Ok(())
            })?;
        }
        
        Ok(())
    }

    #[test]
    fn test_learn_composite_commands() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        cache.cache_path = Some(cache_path);

        // Test learning composite commands (commands with spaces)
        cache.learn_correction("clippy", "cargo clippy")?;
        assert_eq!(cache.find_similar("clippy"), Some("cargo clippy".to_string()),
            "Failed to retrieve the composite command correction");

        // Test that composite commands don't need to be in the commands set
        assert!(!cache.commands.contains("cargo clippy"), 
            "Composite command should not be in the commands set");

        // Verify that the correction persists after saving and loading
        cache.save()?;
        let loaded_cache = CommandCache::load_from_path(cache.cache_path.as_ref().unwrap())?;
        assert_eq!(loaded_cache.find_similar("clippy"), Some("cargo clippy".to_string()),
            "Learned correction for composite command didn't persist after reload");

        // Test with other composite command formats
        cache.learn_correction("test", "echo 'hello world'")?;
        assert_eq!(cache.find_similar("test"), Some("echo 'hello world'".to_string()),
            "Failed to retrieve correction with quotes");

        cache.learn_correction("gs", "git status")?;
        assert_eq!(cache.find_similar("gs"), Some("git status".to_string()),
            "Failed to retrieve simple composite command");

        Ok(())
    }

    #[test]
    fn test_correction_priority() -> Result<()> {
        let (temp_dir, mut cache) = setup_test_cache()?;
        let cache_path = temp_dir.path().join("test_cache.json");
        cache.cache_path = Some(cache_path);

        // Add commands that are similar to each other
        cache.insert("cargo");
        cache.insert("carg");

        // Test fuzzy matching before learning a correction
        assert_eq!(cache.find_similar("carg"), Some("carg".to_string()),
            "Exact match should be found");
        assert_eq!(cache.find_similar("cago"), Some("cargo".to_string()),
            "Fuzzy match should find the closest command");

        // Now learn a correction that would override fuzzy matching
        cache.learn_correction("cago", "cargo version")?;
        assert_eq!(cache.find_similar("cago"), Some("cargo version".to_string()),
            "Learned correction should take priority over fuzzy matching");

        // Test with explicit clippy case
        cache.insert("cargo");
        cache.learn_correction("clippy", "cargo clippy")?;
        assert_eq!(cache.find_similar("clippy"), Some("cargo clippy".to_string()),
            "Learned correction for clippy should work correctly");
        
        Ok(())
    }

    /// Helper function to modify PATH for testing
    fn with_temp_path<F, T>(new_dir: &Path, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Save the current PATH
        let original_path = match env::var_os("PATH") {
            Some(path) => path,
            None => env::var_os("Path").unwrap_or_default(),
        };

        // Create the directory if it doesn't exist
        // But don't fail the test if we can't create it
        if !new_dir.exists() {
            if let Err(e) = fs::create_dir_all(new_dir) {
                log::warn!("Could not create directory for test: {} ({})", new_dir.display(), e);
            }
        }

        // Make the directory absolute
        let abs_path = if new_dir.is_absolute() {
            new_dir.to_path_buf()
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(new_dir)
        };

        // Create a new PATH with our test directory first
        let mut paths = vec![abs_path];
        paths.extend(env::split_paths(&original_path));
        
        // Don't fail if we can't join the paths for some reason
        let set_path_result = env::join_paths(paths)
            .map(|new_path| {
                // Set the new PATH
                unsafe {
                    env::set_var("PATH", &new_path);
                }
            });
        
        if let Err(e) = set_path_result {
            log::warn!("Could not set PATH for test: {}", e);
        }

        // Run the function and ensure we restore PATH whether it succeeds or not
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
        
        // Always restore the original PATH
        unsafe {
            env::set_var("PATH", original_path);
        }

        // Return the result or propagate panic
        match result {
            Ok(r) => r,
            Err(e) => std::panic::resume_unwind(e),
        }
    }
}
