# 🐺 Super Snoofer

A fuzzy command finder for shells that suggests and executes similar commands when a typo is made. When you mistype a command, Super Snoofer will suggest the closest matching command and offer to execute it for you.

```plaintext
$ gti status
Awoo! 🐺 Did you mean `git`? *wags tail* (Y/n/c) y
Running suggested command...
```

## ✨ Features

- 🔍 Fuzzy command matching using Levenshtein distance
- 🚀 Fast command lookup with cached command list
- 🌟 Colorful and friendly interface
- 🔄 Automatic command execution on confirmation
- 🧠 Command learning for frequently used corrections
- 🐍 Support for Python scripts (both .py and without extension)
- 💾 Persistent command cache
- 🔒 Safe command execution through user's shell
- ⚡ Parallel command matching using Rayon

## 📦 Installation

### From crates.io

```bash
cargo install super_snoofer
```

### From Source

```bash
git clone https://github.com/rakki194/super_snoofer.git
cd super_snoofer
cargo install --path .
```

## 🔧 Setup

### ZSH Integration

Add this to your `~/.zshrc`:

```bash
command_not_found_handler() {
    super_snoofer "$1"
    return $?
}
```

### Bash Integration

Add this to your `~/.bashrc`:

```bash
command_not_found_handle() {
    super_snoofer "$1"
    return $?
}
```

### Fish Integration

Create a function in `~/.config/fish/functions/fish_command_not_found.fish`:

```fish
function fish_command_not_found
    super_snoofer "$argv[1]"
    return $status
end
```

## 🎯 Usage

Super Snoofer works automatically once integrated with your shell. When you type a command that doesn't exist, it will:

1. Search for similar commands in your PATH
2. If a match is found, suggest it with a friendly prompt
3. You can:
   - Press Enter or 'y' to accept and execute the suggestion
   - Press 'n' to reject the suggestion
   - Press 'c' to teach Super Snoofer the correct command
4. Exit with the appropriate status code

### Command Line Options

```bash
super_snoofer <command>          # Normal operation: suggest similar commands
super_snoofer --reset_cache      # Clear the command cache but keep learned corrections
super_snoofer --reset_memory     # Clear both the command cache and learned corrections
```

### Example Interactions

```bash
# Basic suggestion and execution
$ carg build
Awoo! 🐺 Did you mean `cargo`? *wags tail* (Y/n/c) y
Running suggested command...
   Compiling super_snoofer v0.1.0

# Teaching Super Snoofer a correction
$ gti status
Awoo! 🐺 Did you mean `got`? *wags tail* (Y/n/c) c
What's the correct command? git
Got it! 🐺 I'll remember that 'gti' means 'git'
[git output follows...]

# Using a learned correction
$ gti status
Awoo! 🐺 Did you mean `git`? *wags tail* (Y/n/c) y
Running suggested command...
[git output follows...]

# Rejecting a suggestion
$ pythn
Awoo! 🐺 Did you mean `python`? *wags tail* (Y/n/c) n
Command 'pythn' not found! 🐺
```

## ⚙️ Configuration

Super Snoofer stores its data in:

- `~/.cache/super_snoofer_cache.json` (if ~/.cache exists)
- `~/.super_snoofer_cache.json` (fallback)

The cache contains:

- List of available commands in your PATH (refreshed daily)
- Learned command corrections (persistent unless cleared)
- Command execution history

Cache Management:

- The command cache is automatically cleared and rebuilt every 24 hours
- Learned corrections persist across cache resets
- Use `--reset_cache` to manually clear the command cache
- Use `--reset_memory` to clear both cache and learned corrections

## 🔬 Technical Details

### Command Learning

Super Snoofer can learn from your corrections:

- When a suggestion is wrong, press 'c' to teach it the right command
- Learned corrections take precedence over fuzzy matching
- Corrections are persisted in the cache file
- Only valid commands can be learned as corrections
- Learned corrections are ~500x faster than fuzzy matching

### Performance

Command matching performance (on typical systems):

- Learned corrections: ~30 nanoseconds
- Fuzzy matching (exact or typo): ~16 microseconds
- Cache updates: performed asynchronously to minimize latency

This means:

- Learned corrections are nearly instant
- Fuzzy matching is fast enough for interactive use
- The more you teach Super Snoofer, the faster it gets

### Similarity Matching

Super Snoofer uses the Levenshtein distance algorithm to find similar commands, with a configurable similarity threshold (currently set to 0.6). This means:

- Commands must be at least 60% similar to be suggested
- Matches are found based on character-level differences
- The most similar command is always suggested first

### Command Discovery

Super Snoofer finds commands by:

1. Scanning all directories in your PATH:
   - Finds executable files
   - Follows symbolic links (including multi-level and circular links)
   - Adds both symlink names and their targets to the command list
   - Handles relative and absolute symlink paths

2. Special handling for Python:
   - Discovers Python executables (python, python3, etc.)
   - Finds executable Python scripts in Python directories
   - Adds both .py and non-.py versions of script names

3. Command caching:
   - Caches discovered commands for better performance
   - Updates cache daily or on manual reset
   - Preserves learned corrections across cache updates

### Symlink Resolution

Super Snoofer handles symbolic links intelligently:

- Follows multi-level symlink chains (e.g., `python -> python3 -> python3.13`)
- Adds all names in the symlink chain to the command list
- Handles both relative and absolute symlink paths
- Detects and safely handles circular symlinks
- Preserves symlink-based command aliases

For example, if you have:

```bash
/usr/bin/python3.13          # Actual executable
/usr/bin/python3 -> python3.13
/usr/bin/python -> python3
```

Super Snoofer will suggest any of these names when you make a typo:

```bash
$ pythn
Awoo! 🐺 Did you mean `python`? *wags tail* (Y/n/c)

$ pythn3
Awoo! 🐺 Did you mean `python3`? *wags tail* (Y/n/c)

$ python313
Awoo! 🐺 Did you mean `python3.13`? *wags tail* (Y/n/c)
```

### Security

- Commands are always executed through the user's shell
- No commands are executed without user confirmation
- The cache file uses standard file permissions
- Command execution preserves arguments and exit codes

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add or update tests
5. Submit a pull request

Please make sure to:

- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

The test suite includes:

- Unit tests for command matching
- Integration tests for command execution
- Command learning and persistence tests
- Cache handling tests
- Path resolution tests

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [strsim](https://crates.io/crates/strsim) for Levenshtein distance calculation
- [colored](https://crates.io/crates/colored) for terminal colors
- [rayon](https://crates.io/crates/rayon) for parallel processing
- [walkdir](https://crates.io/crates/walkdir) for filesystem traversal

## 🐛 Troubleshooting

### Common Issues

1. **Command not found after installation**
   - Ensure `~/.cargo/bin` is in your PATH
   - Try running `source ~/.bashrc` or equivalent

2. **Cache not updating**
   - Check file permissions in ~/.cache
   - Try removing the cache file manually

3. **No suggestions appearing**
   - Verify shell integration is properly set up
   - Check if the command exists in your PATH

### Getting Help

If you encounter issues:

1. Check the [Issues](https://github.com/yourusername/super_snoofer/issues) page
2. Include relevant error messages and your environment details
3. Describe the steps to reproduce the problem
