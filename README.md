# 🐺 Super Snoofer

A fuzzy command finder for shells that suggests and executes similar commands when a typo is made. When you mistype a command, Super Snoofer will suggest the closest matching command and offer to execute it for you.  

```plaintext
$ gti status
Awoo! 🐺 Did you mean `git status`? *wags tail* (Y/n/c) y
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
- 🔗 Shell alias detection and fuzzy matching for Bash, Zsh, and Fish
- 🔮 Full command line correction including arguments and flags for well-known commands
- 🕵️ History tracking that can be enabled or disabled for privacy

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
super_snoofer <command>            # Normal operation: suggest similar commands
super_snoofer --reset_cache        # Clear the command cache but keep learned corrections
super_snoofer --reset_memory       # Clear both the command cache and learned corrections
super_snoofer --history            # Display your recent command corrections
super_snoofer --frequent-typos     # Display your most common typos
super_snoofer --frequent-corrections # Display your most frequently used corrections
super_snoofer --clear-history      # Clear your command history
super_snoofer --enable-history     # Enable command history tracking
super_snoofer --disable-history    # Disable command history tracking
super_snoofer --suggest            # Generate personalized alias suggestions based on your typo history
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

### Shell Aliases

Super Snoofer now detects and includes shell aliases in its suggestions:

- Automatically finds and loads aliases from:
  - **Bash**: `.bashrc` and `.bash_aliases`
  - **Zsh**: `.zshrc`, `.zsh_aliases`, and Oh-My-Zsh custom aliases
  - **Fish**: `config.fish` and function-based aliases in `~/.config/fish/functions/`
- Updates the alias cache every 24 hours
- Shows both the alias name and the command it represents
- Provides fuzzy matching for aliases just like regular commands

#### Alias Matching Examples

Here are some useful examples of how Super Snoofer matches and suggests aliases:

```bash
# Example 1: Mistyped alias corrected to original alias
$ kk stutus                  # Where kk is an alias for kubectl
Awoo! 🐺 Did you mean `kk` (alias for `kubectl`)? *wags tail* (Y/n/c) y
Running suggested command: kubectl status
[command output follows...]

# Example 2: Mistyped alias suggesting multiple possibilities
$ dc ps                      # When you have both 'dc' and 'dco' as aliases
Awoo! 🐺 Did you mean `dc` (alias for `docker-compose`)? *wags tail* (Y/n/c) y
Running suggested command: docker-compose ps
[command output follows...]

# Example 3: Related aliases for common commands
$ giff                       # When you have git-related aliases
Awoo! 🐺 Did you mean `giff` (alias for `git diff --color`)? *wags tail* (Y/n/c) y
[git diff output follows...]

# Example 4: Learning a correction for a complex alias
$ dkr-rmall
Awoo! 🐺 Did you mean `dkr` (alias for `docker`)? *wags tail* (Y/n/c) c
What's the correct command? drm-all
Got it! 🐺 I'll remember that 'dkr-rmall' means 'drm-all'
[docker remove all containers output follows...]

# Example 5: Nested alias resolution
$ gs                         # Where gs is an alias for 'git status'
Awoo! 🐺 Did you mean `gs` (alias for `git status`)? *wags tail* (Y/n/c) y
[git status output follows...]
```

#### Benefits of Alias Matching

Alias matching in Super Snoofer provides several advantages:

1. **Consistency** - Get suggestions for both commands and their aliases
2. **Discoverability** - Learn about available aliases in your system
3. **Convenience** - See what command an alias actually runs
4. **Context awareness** - Suggestions are tailored to your shell setup

Aliases are treated as first-class commands in Super Snoofer, meaning:

- You get suggestions for typos of aliases
- The underlying command is shown in the suggestion
- Aliases can be learned as corrections just like regular commands
- Aliases are included in fuzzy matching searches

## 📊 Command History & Frequency Analysis

Super Snoofer is a good boy and will try to learn from your mistakes, by tracking the history of your command corrections and typos to provide smarter suggestions over time:

- **Command history tracking** - Records all command corrections and queries
- **Frequency analysis** - Suggests more commonly used commands first
- **Pattern recognition** - Learns your specific typing patterns and common mistakes
- **Personalized suggestions** - Tailors suggestions based on your command usage history

### History Features

```bash
# View your command correction history
$ super_snoofer --history
🐺 Your recent command corrections:
1. gti → git (17 times)
2. pythno → python (8 times)
3. docekr → docker (5 times)
...

# View most frequent typos
$ super_snoofer --frequent-typos
🐺 Your most common typos:
1. gti (17 times)
2. pythno (8 times)
3. docekr (5 times)
...

# View most frequent corrections
$ super_snoofer --frequent-corrections
🐺 Your most frequently used corrections:
1. git (22 times)
2. python (15 times)
3. docker (11 times)
...

# Clear your command history
$ super_snoofer --clear-history
Command history cleared successfully! 🐺

# Enable command history tracking
$ super_snoofer --enable-history
Command history tracking is now enabled! 🐺

# Disable command history tracking
$ super_snoofer --disable-history
Command history tracking is now disabled! 🐺
```

When making suggestions, Super Snoofer now prioritizes commands based on your usage history:

```bash
# When typing a command with multiple possible corrections
$ gti
Awoo! 🐺 Did you mean `git` (used 17 times)? *wags tail* (Y/n/c) y
Running suggested command...
```

The history data is stored in your cache file and is used to:

1. Prioritize frequently used commands in suggestions
2. Identify patterns in your typing mistakes
3. Improve suggestion accuracy over time
4. Provide insights into your command usage habits

This feature helps Super Snoofer become more personalized to your workflow the more you use it.

### History Control

Super Snoofer allows you to control whether command history is tracked:

- **History Tracking Enabled** (default): Super Snoofer records all typos and corrections to provide increasingly personalized suggestions over time
- **History Tracking Disabled**: No command history is recorded, providing more privacy but without the benefits of personalized suggestions

You can toggle this setting using the following commands:

```bash
# Disable history tracking
$ super_snoofer --disable-history
Command history tracking is now disabled! 🐺

# Enable history tracking
$ super_snoofer --enable-history
Command history tracking is now enabled! 🐺
```

When history tracking is disabled:

- No new command corrections will be recorded
- Frequency analysis will not be updated
- Existing learned corrections will still be used
- The `--history`, `--frequent-typos`, and `--frequent-corrections` commands will inform you that history tracking is disabled

This feature is useful if you:

- Are concerned about privacy
- Share your computer with others
- Want to prevent recording sensitive commands
- Prefer not to have personalized suggestions

The setting persists between Super Snoofer sessions.

## 🧠 Personalized Alias Suggestions

Super Snoofer can analyze your command history and suggest helpful aliases to save you time:

```bash
# Generate personalized alias suggestions
$ super_snoofer --suggest
You've mistyped 'gti' 17 times! Let's create an alias for that.

Suggested alias: g → git

To add this alias to your shell configuration:

Bash (add to ~/.bashrc):
alias g='git'

Zsh (add to ~/.zshrc):
alias g='git'

Fish (add to ~/.config/fish/config.fish):
alias g 'git'

Would you like me to add this alias to your shell configuration? (y/N)
```

This feature:

1. **Analyzes your typo history** to identify the commands you most frequently mistype
2. **Generates useful aliases** based on your personal command usage patterns
3. **Provides personalized tips** with a touch of Super Snoofer's friendly personality
4. **Offers one-click installation** to your shell configuration file
5. **Supports multiple shells** including Bash, Zsh, and Fish

Each time you run `--suggest`, Super Snoofer will randomly select one of your frequent typos and suggest an appropriate alias. This helps you gradually build a set of personalized shortcuts that match your specific typing patterns and workflow.

The suggested aliases are designed to be:

- **Memorable**: Usually based on the first letter(s) of the command
- **Intuitive**: Directly related to your common typos
- **Time-saving**: Focused on your most frequently used commands

Over time, these suggestions can significantly reduce the need for command corrections by giving you shorter alternatives for your most-used commands.

## 🔮 Full Command Line Correction

Super Snoofer v0.3.0 now corrects typos in the entire command line, not just the command name. For well-known commands, it can intelligently fix typos in subcommands, arguments, and flags:

```bash
# Correcting typos in git commands
$ gti sttaus
Awoo! 🐺 Did you mean `git status`? *wags tail* (Y/n/c) y
Running suggested command...

# Correcting typos in docker commands
$ dockr ps -a
Awoo! 🐺 Did you mean `docker ps -a`? *wags tail* (Y/n/c) y
Running suggested command...

# Correcting typos in cargo commands and flags
$ carg buld --relese
Awoo! 🐺 Did you mean `cargo build --release`? *wags tail* (Y/n/c) y
Running suggested command...
```

### Supported Commands

Super Snoofer includes built-in knowledge about these common commands and their arguments:

- **Git**: status, commit, push, pull, branch, merge, etc.
- **Docker**: run, build, ps, exec, logs, etc.
- **Cargo**: build, run, test, check, publish, etc.
- **npm**: install, uninstall, update, run, etc.
- **kubectl**: get, describe, apply, delete, logs, etc.

### How It Works

When you enter a command with typos:

1. Super Snoofer first corrects the base command (e.g., "gti" → "git")
2. For well-known commands, it then attempts to correct each argument and flag
3. For arguments, it checks against known subcommands (e.g., "sttaus" → "status")
4. For flags, it checks against known options (e.g., "--hlp" → "--help")
5. It presents the fully corrected command line for your approval

This works best with common CLI tools, but will fall back to simple command correction for other commands.
