# 🐺 Super Snoofer

Super Snoofer is an intelligent command correction utility that:

1. Intercepts typos before they cause errors
2. Learns from your command usage patterns
3. Provides auto-completion for commands and arguments
4. Builds knowledge of commands and their arguments over time

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
- 🧩 Smart shell configuration for creating and managing aliases
- 🔍 Enhanced typo correction for common commands like Git
- ✨ Proactive typo detection for all commands, not just missing ones
- 📊 Command history analytics to suggest better corrections
- 🔄 Background learning from successfully executed commands
- 🔄 Auto-completion for commands and arguments
- 🔄 Low system impact by incrementally building knowledge
- 🔄 Command history analytics to suggest better corrections
- 🔄 Customizable command exclusion preferences
- 💡 Real-time command suggestions as you type

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

Super Snoofer offers two levels of integration with your shell:

1. **Basic Integration** (command-not-found handler only): Suggests corrections when a command doesn't exist
2. **Advanced Integration** (proactive correction): Intercepts and corrects typos in all commands before execution

### ZSH Integration

#### Recommended Setup (Clean .zshrc)

For the cleanest and most maintainable setup, source the `zsh_integration.zsh` file directly from your `.zshrc`:

1. First, ensure the zsh_integration.zsh file is in your Super Snoofer directory
2. Add just this single line to your `.zshrc`:

```bash
# Super Snoofer integration
if [[ -f /path/to/super_snoofer/zsh_integration.zsh ]]; then
  source /path/to/super_snoofer/zsh_integration.zsh
fi
```

This approach:

- Keeps your `.zshrc` clean and organized
- Makes it easy to update Super Snoofer independently
- Maintains all functionality in a single external file
- Allows toggling Super Snoofer by commenting out just one line

#### Basic Integration (Command-not-found handler only)

Add this to your `~/.zshrc`:

```bash
command_not_found_handler() {
    super_snoofer "$@"
    return $?
}
```

### Bash Integration

#### Basic Integration (Command-not-found handler only)

Add this to your `~/.bashrc`:

```bash
command_not_found_handle() {
    super_snoofer "$@"
    return $?
}
```

### Fish Integration

#### Basic Integration (Command-not-found handler only)

Create a function in `~/.config/fish/functions/fish_command_not_found.fish`:

```fish
function fish_command_not_found
    super_snoofer $argv[1]
    return $status
end
```

## 🎯 Usage

Super Snoofer works in two ways depending on your integration level:

### Basic Usage (command-not-found handler)

When you type a command that doesn't exist, Super Snoofer will:

1. Search for similar commands in your PATH
2. If a match is found, suggest it with a friendly prompt
3. You can:
   - Press Enter or 'y' to accept and execute the suggestion
   - Press 'n' to reject the suggestion
   - Press 'c' to teach Super Snoofer the correct command
4. Exit with the appropriate status code

### Advanced Usage (proactive correction)

With advanced integration, Super Snoofer will:

1. Intercept all commands before they execute
2. Check for typos in the command, arguments, and flags
3. If typos are found, suggest a corrected version
4. You can:
   - Press Enter to use the corrected command
   - Press Ctrl+C to cancel and use the original command
5. Learn from your successfully executed commands

### Command Line Options

```bash
super_snoofer <command>              # Normal operation: suggest similar commands
super_snoofer --reset_cache          # Clear the command cache but keep learned corrections
super_snoofer --reset_memory         # Clear both the command cache and learned corrections
super_snoofer --history              # Display your recent command corrections
super_snoofer --frequent-typos       # Display your most common typos
super_snoofer --frequent-corrections # Display your most frequently used corrections
super_snoofer --clear-history        # Clear your command history
super_snoofer --enable-history       # Enable command history tracking
super_snoofer --disable-history      # Disable command history tracking
super_snoofer --suggest              # Suggest personalized shell aliases
super_snoofer --check-command <cmd>  # Check if a command has typos (for shell integration)
super_snoofer --record-correction <typo> <correction>  # Record a correction for history
super_snoofer --record-valid-command <cmd>  # Record a valid command usage
super_snoofer --suggest-completion <cmd>  # Get real-time command suggestions (for shell integration)
super_snoofer --enable-completion    # Enable ZSH auto-completion
super_snoofer --disable-completion   # Disable ZSH auto-completion
super_snoofer --export-completions [PATH]  # Export completion script to a file
```

### Example Interactions

#### Basic Usage (command-not-found handler)

```bash
# Basic suggestion and execution
$ carg build
Command 'carg' not found! Did you mean:
1. cargo
2. Enter custom command
3. Add permanent shell alias
4. Exit without running
Enter your choice (1-4): 1
   Compiling super_snoofer v0.1.0

# Teaching Super Snoofer a correction
$ gti status
Command 'gti' not found! Did you mean:
1. git
2. Enter custom command
3. Add permanent shell alias
4. Exit without running
Enter your choice (1-4): 2
Enter the correct command: git
Got it! 🐺 I'll remember that 'gti' means 'git'
[git output follows...]
```

#### Advanced Usage (proactive correction)

```bash
# Proactive command correction
$ gti stauts
🐺 Did you mean: git status
Press Enter to use the corrected command, or Ctrl+C to cancel
[Press Enter]
[git status output follows...]

# Command with typo'd flags
$ git commt --al
🐺 Did you mean: git commit --all
Press Enter to use the corrected command, or Ctrl+C to cancel
[Press Enter]
[git commit output follows...]

# Docker command with multiple typo'd flags
$ dokcer run --detetch --naem container
🐺 Did you mean: docker run --detach --name container
Press Enter to use the corrected command, or Ctrl+C to cancel
[Press Enter]
[docker run output follows...]
```

## ⚙️ Configuration

Super Snoofer stores its data in:

- `~/.cache/super_snoofer_cache.json` (if ~/.cache exists)
- `~/.super_snoofer_cache.json` (fallback)

The cache contains:

- List of available commands in your PATH (refreshed daily)
- Learned command corrections (persistent unless cleared)
- Command execution history and frequency data
- Shell aliases for suggestions

Cache Management:

- The command cache is automatically cleared and rebuilt every 24 hours
- Learned corrections persist across cache resets
- Use `--reset_cache` to manually clear the command cache
- Use `--reset_memory` to clear both cache and learned corrections

### Customizing Advanced Integration

You can customize the advanced shell integration in several ways:

#### Exempted Commands

Edit the `exempted_commands` array in your shell integration to skip correction for certain commands:

```bash
# Add commands you want to exempt from correction:
local exempted_commands=("cd" "ls" "pwd" "echo" "clear" "exit" "ssh" "vim" "gl")
```

This is useful for:

- Short or frequently used commands where typos are rare
- Commands where interrupting with a correction would be disruptive
- Commands that are part of automated scripts

## 🔬 Technical Details

### Command Learning

Super Snoofer can learn from your corrections in two ways:

1. **Explicit learning**: When you choose to teach it a correction
   - Occurs when you select "Enter custom command" when a command isn't found
   - Learned corrections take precedence over fuzzy matching
   - Corrections are persisted in the cache file

2. **Passive learning** (with advanced integration):
   - Records corrections you accept via the preexec hook
   - Records successful commands you run via the postexec hook
   - Builds a frequency map of commands to improve suggestions

### Proactive Correction

With advanced integration, Super Snoofer can:

1. Correct typos in the command itself (`gti` → `git`)
2. Correct typos in arguments (`git stauts` → `git status`)
3. Correct typos in flags (`git commit --al` → `git commit --all`)
4. Handle multiple typos in a single command line

The correction uses a knowledge base of common commands and their arguments/flags, with special handling for:

- Git commands
- Docker commands
- Cargo (Rust) commands
- npm commands
- kubectl commands

### Performance

All Super Snoofer operations are designed to be non-blocking and lightweight:

- Proactive correction takes < 20ms for most commands
- History recording happens in the background
- Exempt commands bypass correction entirely
- Cached corrections are retrieved in microseconds

### Security

- Commands are always executed through the user's shell
- No commands are executed without user confirmation
- The cache file uses standard file permissions
- Hooks run in the user's security context
- All background operations run with reduced priority

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
super_snoofer --add-alias NAME [CMD] # Add shell alias (default: super_snoofer)
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
# Generate a personalized alias suggestion
$ super_snoofer --suggest
You've mistyped 'gti' 17 times! Let's create an alias for that.

Suggested alias: g → git

To add this alias to your shell configuration:

alias g='git'

Would you like me to add this alias to your shell configuration? (y/N)
```

The `--suggest` command analyzes your command history to:

1. Identify your most common typos
2. Recommend useful aliases based on your usage patterns
3. Offer to automatically add the aliases to your shell configuration
4. Create shortcuts for your most frequently used commands

This feature helps you create a more efficient workflow by automating the creation of aliases tailored to your specific typing patterns and command usage.

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

### Enhanced Typo Correction

Super Snoofer v0.3.0 includes special handling for common Git command typos, such as:

```bash
# Common "status" typos
$ gti statuus
Awoo! 🐺 Did you mean `git status`? *wags tail* (Y/n/c) y
Running suggested command...

$ git satatus
Awoo! 🐺 Did you mean `git status`? *wags tail* (Y/n/c) y
Running suggested command...

$ git statsu
Awoo! 🐺 Did you mean `git status`? *wags tail* (Y/n/c) y
Running suggested command...
```

The correction engine is especially tuned for:

- Common Git operations (status, commit, push, pull)
- Docker commands (run, build, ps)
- Cargo commands (build, run, test)
- npm commands (install, run)
- kubectl commands

Super Snoofer uses a combination of pattern matching, Levenshtein distance, and special case handling to provide highly accurate corrections for these common command patterns.

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

## 🧩 Smart Shell Configuration

Super Snoofer includes intelligent shell detection and configuration management features that make it easy to add aliases and integrate with your system:

### Automatic Shell Detection

Super Snoofer can detect your current shell environment and provide appropriate configuration instructions:

```bash
# When suggesting an alias
$ super_snoofer --suggest
You've mistyped 'gti' 17 times! Let's create an alias for that.

Suggested alias: g → git

To add this alias to your shell configuration:

alias g='git'

Would you like me to add this alias to your Zsh shell configuration? (y/N) y
Adding alias to /home/user/.zshrc

Added alias to your Zsh configuration! 🐺 Please run 'source /home/user/.zshrc' to use it.
```

### Supported Shells

Super Snoofer automatically detects and supports:

- 🐚 **Bash** - Configures `.bashrc`
- 🔮 **Zsh** - Configures `.zshrc`
- 🐟 **Fish** - Configures `config.fish`
- 💻 **PowerShell** - Configures profile script
- 🔵 **Nushell** - Configures `config.nu`
- 🐚 **Korn Shell** - Configures `.kshrc`
- 🔄 **C Shell/TCSH** - Configures `.cshrc` or `.tcshrc`
- 🪟 **Windows Command Prompt** - Configures doskey batch file

### Configuration Files

Super Snoofer creates appropriate shell-specific alias syntax based on your shell:

| Shell | Configuration File | Alias Format |
|-------|-------------------|--------------|
| Bash | ~/.bashrc | `alias g='git'` |
| Zsh | ~/.zshrc | `alias g='git'` |
| Fish | ~/.config/fish/config.fish | `alias g 'git'` |
| PowerShell | ~/Documents/PowerShell/Microsoft.PowerShell_profile.ps1 | `Set-Alias -Name g -Value git` |
| Nushell | ~/.config/nushell/config.nu | `alias g = git` |
| Korn Shell | ~/.kshrc | `alias g='git'` |
| C Shell/TCSH | ~/.cshrc or ~/.tcshrc | `alias g 'git'` |
| Windows CMD | %USERPROFILE%\doskey.bat | `doskey g=git` |

### Shell Integration With Auto-Setup

Super Snoofer can not only suggest aliases but also handle the entire configuration process for you:

1. Detects your current shell environment automatically
2. Creates appropriate configuration files if they don't exist
3. Adds shell-specific syntax for aliases and functions
4. Preserves existing content in configuration files
5. Suggests reload commands after configuration changes

This makes it simple to integrate Super Snoofer with your workflow without needing to remember shell-specific configuration details.

## 💡 Real-Time Command Suggestions

Super Snoofer v0.5.0 introduces a revolutionary new feature: real-time command suggestions as you type, similar to GitHub Copilot's assistance when writing code. This feature helps you:

1. **Complete commands automatically** - Suggestions appear as you type
2. **Learn arguments and flags** - Automatic suggestions for command arguments
3. **Correct typos instantly** - Fix mistakes before pressing Enter
4. **Discover new options** - Learn about command flags you didn't know existed

### How It Works

As you type in your shell:

1. Super Snoofer analyzes each keystroke in real-time
2. It looks for possible completions based on command patterns and your usage history
3. Suggestions appear in a faded color
4. Press Tab to accept a suggestion or continue typing

### Examples

```bash
# Completing a command
$ git s[TAB]
# Becomes:
$ git status

# Suggesting flags
$ git commit --a[TAB]
# Becomes:
$ git commit --all

# Correcting typos in real-time
$ gti pus[TAB]
# Becomes:
$ git push

# Command with multiple arguments
$ docker run --deta[TAB]
# Becomes:
$ docker run --detach
```

### Technical Details

The real-time suggestion system works by:

1. Intercepting keyboard input using ZSH's widget system
2. Triggering the `--suggest-completion` command to get suggestions
3. Displaying suggestions using ZSH's region highlighting
4. Accepting suggestions with the Tab key

Super Snoofer learns from your command usage to make increasingly relevant and personalized suggestions. Commands, arguments, and flags used frequently will be suggested first, creating a highly personalized command-line experience.

### Customization

You can enable/disable real-time suggestions with:

```bash
# Toggle suggestions on/off
$ super_snoofer_toggle_suggestions

# Disable suggestions for specific commands by adding them to the exclude list
# Edit in your .zshrc:
SUPER_SNOOFER_EXCLUDE_COMMANDS="vim vi nano emacs cd ls cat"
```

### ZSH Integration

The real-time suggestion feature is included in the advanced ZSH integration. If using our integration script, this feature is enabled by default and can be toggled on/off with the `super_snoofer_toggle_suggestions` command.

```bash
# To see if suggestions are enabled
$ echo $SUPER_SNOOFER_SUGGESTIONS_ENABLED

# To temporarily disable suggestions for the current session
$ SUPER_SNOOFER_SUGGESTIONS_ENABLED=false

# To permanently disable suggestions, edit your .zshrc:
SUPER_SNOOFER_SUGGESTIONS_ENABLED=false
```
