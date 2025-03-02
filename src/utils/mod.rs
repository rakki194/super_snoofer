use log::debug;
use std::{collections::HashSet, env, fs, path::Path};
use walkdir::WalkDir;
use strsim::normalized_levenshtein;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Find the closest matching string in the given list
pub fn find_closest_match<'a, S>(
    query: &str,
    options: &'a [S],
    threshold: f64,
) -> Option<&'a S>
where
    S: AsRef<str>,
{
    if options.is_empty() {
        return None;
    }

    // Special case for common typos
    if query == "gti" {
        for option in options {
            if option.as_ref() == "git" {
                return Some(option);
            }
        }
    }

    let mut best_match = None;
    let mut best_score = 0.0;

    // Convert query to lowercase for case-insensitive matching
    let query_lower = query.to_lowercase();

    for option in options {
        // Calculate similarity using our specialized function
        let option_str = option.as_ref();
        let score = calculate_similarity(&query_lower, option_str);

        if score > best_score && score >= threshold {
            best_score = score;
            best_match = Some(option);
        }
    }

    best_match
}

/// Calculate Levenshtein distance between two strings
#[must_use] pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_len = s1.chars().count();
    let s2_len = s2.chars().count();

    // Convert to character vectors for random access
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // Early return for empty strings
    if s1_len == 0 {
        return s2_len;
    }
    if s2_len == 0 {
        return s1_len;
    }

    // Create a matrix to store distances
    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

    // Initialize the first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(s1_len + 1) {
        row[0] = i;
    }
    for j in 0..=s2_len {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=s1_len {
        for j in 1..=s2_len {
            let cost = usize::from(s1_chars[i - 1] != s2_chars[j - 1]);

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // Deletion
                    matrix[i][j - 1] + 1, // Insertion
                ),
                matrix[i - 1][j - 1] + cost, // Substitution
            );
        }
    }

    // Return the bottom-right value, which is the total distance
    matrix[s1_len][s2_len]
}

/// Calculate similarity between two strings
#[must_use] pub fn calculate_similarity(a: &str, b: &str) -> f64 {
    // Handle case insensitivity by converting to lowercase
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    // Use the lowercase strings for comparison
    let a = a_lower.as_str();
    let b = b_lower.as_str();

    // Handle special cases for very short strings
    if a.len() <= 3 && b.len() <= 3 {
        // For very short strings, exact match is best
        if a == b {
            return 1.0;
        }
        
        // For common typos like "gti" vs "git", be more lenient
        if (a == "gti" && b == "git") || (a == "git" && b == "gti") {
            return 0.9;  // Very high similarity for this common typo
        }
        
        // For other short strings, use a specialized similarity measure
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        
        // Count matching characters in any position
        let mut matches = 0;
        for c1 in &a_chars {
            if b_chars.contains(c1) {
                matches += 1;
            }
        }
        
        // Calculate similarity based on matches and length
        let total = a.len().max(b.len());
        if total > 0 {
            // Use u32 as an intermediate type to avoid precision loss
            let matches_f64 = f64::from(u32::try_from(matches).unwrap_or(u32::MAX));
            let total_f64 = f64::from(u32::try_from(total).unwrap_or(u32::MAX));
            matches_f64 / total_f64
        } else {
            0.0
        }
    } else {
        // For longer strings, use normalized Levenshtein distance
        normalized_levenshtein(a, b)
    }
}

/// Checks if a file is executable on the current platform
///
/// # Arguments
///
/// * `path` - The path to the file to check
///
/// # Returns
///
/// `true` if the file is executable by the current user, `false` otherwise
#[must_use] pub fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        // Follow symlinks when checking permissions
        fs::metadata(path)
            .or_else(|_| {
                // If we can't get metadata, try following the symlink manually
                if path.is_symlink() {
                    fs::read_link(path).and_then(fs::metadata)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "Not a symlink",
                    ))
                }
            })
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        let extension = path.extension().and_then(|ext| ext.to_str());
        matches!(extension, Some("exe") | Some("bat") | Some("cmd"))
    }
    #[cfg(not(unix))]
    {
        // On non-Unix platforms, check for common executable extensions
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();
            ["exe", "bat", "cmd", "com", "ps1"].contains(&ext_str.as_str())
        } else {
            false
        }
    }
}

/// Get all commands from the PATH environment variable
pub fn get_path_commands() -> HashSet<String> {
    let mut commands = HashSet::new();

    // Get all directories in PATH
    if let Some(path) = env::var_os("PATH") {
        for dir in env::split_paths(&path) {
            if dir.exists() {
                for entry in WalkDir::new(dir)
                    .max_depth(1)
                    .into_iter()
                    .filter_map(Result::ok)
                {
                    if (entry.file_type().is_file() || entry.file_type().is_symlink())
                        && is_executable(entry.path())
                    {
                        if let Some(name) = entry.file_name().to_str() {
                            commands.insert(name.to_string());

                            // If this is a symlink, follow it and add target name
                            #[cfg(unix)]
                            if entry.file_type().is_symlink() {
                                let mut current_path = entry.path().to_path_buf();
                                let mut seen_paths = HashSet::new();

                                // Follow symlink chain to handle multiple levels
                                while current_path.is_symlink() {
                                    // Add the current path to our seen paths set to detect cycles
                                    if !seen_paths.insert(current_path.clone()) {
                                        // Circular symlink detected, stop here
                                        debug!("Circular symlink detected: {:?}", current_path);
                                        break;
                                    }

                                    match fs::read_link(&current_path) {
                                        Ok(target) => {
                                            // Resolve the target path, making it absolute if needed
                                            current_path = if target.is_absolute() {
                                                target
                                            } else {
                                                // Relative paths are relative to the directory containing the symlink
                                                if let Some(parent) = current_path.parent() {
                                                    parent.join(&target)
                                                } else {
                                                    target
                                                }
                                            };

                                            // Extract the command name from the resolved path
                                            if let Some(target_name) = current_path.file_name() {
                                                if let Some(name) = target_name.to_str() {
                                                    commands.insert(name.to_string());
                                                    debug!("Added symlink target: {}", name);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            // Log errors but continue processing
                                            debug!(
                                                "Error following symlink {}: {}",
                                                current_path.display(),
                                                e
                                            );
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
    }

    // Add Python scripts from Python directories
    for python_cmd in ["python", "python3"] {
        if let Ok(python_path) = which::which(python_cmd) {
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
                                commands.insert(name.to_string());
                                // Also add the name without .py extension
                                if let Some(stem) = std::path::Path::new(name).file_stem() {
                                    if let Some(stem_str) = stem.to_str() {
                                        commands.insert(stem_str.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    commands
}

/// Remove trailing flags from an argument
/// e.g. "file.txt:10" -> ("file.txt", ":10")
#[must_use] pub fn remove_trailing_flags(arg: &str) -> (&str, String) {
    // Handle flags that start after the argument
    if let Some(pos) = arg.find([':', '=', '@']) {
        let (base, flag) = arg.split_at(pos);
        return (base, flag.to_string());
    }
    
    (arg, String::new())
}
