# 🐺 Super Snoofer

A fuzzy command finder for shells that suggests and executes similar commands when a typo is made. When you mistype a command, Super Snoofer will suggest the closest matching command and offer to execute it for you.

## ✨ Features

- 🔍 Fuzzy command matching using Levenshtein distance
- 🚀 Fast command lookup with cached command list
- 🌟 Colorful and friendly interface
- 🔄 Automatic command execution on confirmation
- 🧠 Command learning for frequently used corrections
- 🤖 AI-powered chat interface with multiple models
- 🎯 Quick model access with `>` and `>>` shortcuts
- 💬 Interactive TUI for comfortable AI conversations
- 🔒 Safe command execution through user's shell
- ⚡ Parallel command matching using Rayon
- 🔗 Shell alias detection and fuzzy matching
- 🔮 Full command line correction
- 🕵️ History tracking that can be enabled or disabled
- 🧩 Smart shell configuration
- 🙃 Built-in AI assistant

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
