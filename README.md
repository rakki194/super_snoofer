# 🐺 Super Snoofer

A fuzzy command finder for ZSH that suggests and executes similar commands when a typo is made. When you mistype a command, Super Snoofer will suggest the closest matching command and offer to execute it for you. It also provides a TUI for comfortable AI conversations.

## ✨ Features

- 🔍 Fuzzy command matching using Levenshtein distance
- 🚀 Fast command lookup with cached command list
- 🌟 Colorful and friendly interface
- 🔄 Automatic command execution on confirmation
- 🧠 Command learning for frequently used corrections
- 🙃 AI-powered chat interface with multiple models
- 🎯 Quick model access with `]` and `]]` shortcuts
- 💬 Interactive TUI for comfortable AI conversations
- 🎭 Real-time loading animation during model loading
- 📝 Live streaming text display for AI responses
- 🔄 Clear visual state indicators for all AI processing phases
- 📜 Dynamic scrollbar with mouse support and keyboard navigation
- 🤔 Collapsible "thinking" sections in AI responses
- 🔒 Safe command execution through user's shell
- ⚡ Parallel command matching using Rayon
- 🔗 Shell alias detection and fuzzy matching
- 🔮 Full command line correction
- 🕵️ History tracking that can be enabled or disabled
- 🧩 Smart shell configuration

## 📦 Installation

## From crates.io

```bash
cargo install super_snoofer
```

### From Source

```bash
git clone https://github.com/rakki194/super_snoofer.git
cd super_snoofer
cargo install --path .
```

## 🧠 AI Conversation

Super Snoofer includes a Terminal User Interface (TUI) for having comfortable conversations with AI models through Ollama:

- Real-time loading animations during model loading
- Live streaming of AI responses as they're generated
- Responsive UI that never freezes, even during heavy model operations
- Auto-timeout protection to prevent hanging on unresponsive models
- Support for multiple LLM models with easy switching
- Code-optimized models for programming questions
- Visual state indicators for each phase:
  - ⏳ Model loading - Initializing the AI model
  - 🔄 Generating - Model is working but no text has been received yet
  - 💬 Streaming - Text is being received and displayed in real-time
  - ✨ Complete - The response has been fully generated
- Advanced navigation features:
  - Dynamic scrollbar that only appears when content is scrollable
  - Mouse wheel scrolling support
  - Click on the scrollbar to jump to a position
  - Keyboard navigation with arrow keys, Page Up/Down, Home/End
  - Collapsible "thinking" sections (toggle with T key)

To start an AI conversation:

```bash
super_snoofer --prompt "Your question here"
```

Or use the shortcuts in your terminal:

```bash
] Your question here        # Uses the standard model
]] Your code question here  # Uses the code-optimized model
```

### Keyboard Controls

- **Enter**: Submit prompt
- **Shift+Enter**: Add a new line in input
- **Escape**: Cancel response (during streaming) or exit application
- **Ctrl+C**: Exit application
- **T**: Toggle thinking sections
- **Ctrl+S**: Toggle text selection mode
- **↑/↓**: Move cursor in input field or scroll response
- **Home/End**: Move to start/end of current line
- **Page Up/Page Down**: Scroll response by page
- **Ctrl+Home/End**: Scroll to top/bottom of response

### Mouse Controls

- **Scroll wheel**: Scroll through response
- **Click on scrollbar**: Jump to position
- **Click and drag**: Select text (in selection mode)

## Development

The project is structured as follows:

- `src/main.rs`: Main entry point and application setup
- `src/tui/`: UI components and event handling
- `src/ollama.rs`: API client for Ollama interaction
