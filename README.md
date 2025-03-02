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
