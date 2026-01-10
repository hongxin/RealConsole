# RealConsole

> Intelligent CLI Agent Infused with Eastern Philosophy

[Installation](#installation) | [Quick Start](#quick-start) | [Documentation](#documentation) | [Examples](#examples)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1400%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.63.0-blue.svg)](CHANGELOG.md)

English | **[中文](README.cn.md)**

**RealConsole** is an intelligent command-line agent built with Rust, based on the "One Divides into Three" (一分为三) philosophy. It combines LLM-powered conversation, proactive suggestions, task orchestration, and DevOps tools to provide a seamless CLI experience. **Now with Web Terminal support!**

## Installation

### Quick Install

```bash
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole
make install
```

The binary will be installed to `~/.local/bin/realconsole`.

### Build from Source

```bash
cargo build --release
./target/release/realconsole
```

**Requirements**: Rust 1.70+, LLM Provider ([Ollama](https://ollama.ai/)/[Deepseek](https://platform.deepseek.com/)/[Gemini](https://aistudio.google.com/)/[OpenAI](https://platform.openai.com/))

> 📖 **Detailed instructions**: [Installation Guide](docs/02-practice/user/quickstart.md)

## Quick Start

### 1. Configure

Run the interactive wizard:

```bash
realconsole wizard --quick
```

Or manually copy configuration files:

```bash
cp .env.example .env
cp config/realconsole.yaml.example realconsole.yaml
# Edit .env and realconsole.yaml
```

### 2. Run

**CLI Mode**:
```bash
realconsole
```

**Web Terminal Mode** (NEW):
```bash
realconsole web
# Visit http://127.0.0.1:7788
```

### 3. Try it out

```bash
% hello                           # Chat with AI
% ls -la                          # Execute shell commands (smart routing)
% /suggest                        # Get proactive suggestions
% /plan create a Rust project    # Task orchestration
% /trace                          # Unified tracking
```

> 📖 **Full guide**: [Quick Start Guide](docs/02-practice/user/quickstart.md)

## Key Features

### 🌐 Web Terminal ⭐ Core Highlight (v1.23.0 - v1.39.0)

**Cross-platform Web Terminal** - Access RealConsole anywhere (17 versions of continuous optimization):

```bash
realconsole web --bind 0.0.0.0 --port 7788
```

**Core Features**:
- ✨ **Smart Routing**: Auto-detects shell commands, no `!` prefix needed
- 🎯 **Intent Understanding**: 50+ built-in intents, natural language task execution
- 🔧 **Tool Calling**: Full LLM tool calling capability
- 📒 **Jupyter-like Experience**: Round cards, collapsible output, one-click rerun (v1.28.0+)
- 🤖 **Intent Decomposition Visualization**: Visualize AI thinking process, auto-execute tools (v1.39.0)
- 👁️ **Eye Protection Colors**: Professional dark theme, inspired by GitHub/Binance, comfortable for long-term use (v1.39.0)
- 🎨 **Beautiful UI**: Real-time streaming output, command history, auto-completion
- 📱 **Mobile Friendly**: Responsive design with touch support
- 🌍 **LAN Access**: Team collaboration across devices

**Use Cases**:
- Remote server management
- Mobile device access
- Team collaboration demos
- Quick testing without installation

> 📖 **Detailed docs**: [Web Terminal User Guide](docs/02-practice/user/web-terminal.md)

### 🌍 Internationalization (v1.24.0)

**Complete bilingual support (English/Chinese)**:

- **CLI Interface**: All command outputs and prompts
- **LLM Prompts**: System prompts in both languages
- **YAML Config**: Internationalized configuration files
- **Dynamic Switching**: `REALCONSOLE_LANG=zh-CN|en-US` environment variable

### 🤖 Intelligent Conversation
- **LLM Integration**: Ollama (local) / Deepseek / Gemini / OpenAI support
- **Streaming Output**: Real-time token-by-token display
- **Multi-turn Context**: Automatic context management (Auto/Manual/Disabled modes)
- **Tool Calling**: 14+ built-in tools (calculator, file ops, datetime, etc.)

### 💡 Proactive Suggestions
- **Context-Aware**: Intelligent suggestions based on project type, command history, and errors
- **Quick Execute**: Number shortcuts (e.g., `1`, `2`, `3`) to run suggestions instantly
- **Spell Checking**: Automatic typo correction with Levenshtein distance algorithm
- **Feedback Learning**: Adapts to your preferences through user feedback

```bash
% /suggest
💡 Based on your context:
  1. cargo build - Build the project
  2. cargo test - Run tests
  3. git commit - Commit changes

% 1           # Quick execute
```

### 🛠️ Task Orchestration
- **Natural Language Goals**: Describe what you want, AI decomposes into tasks
- **Intelligent Parallel Execution**: Automatic dependency analysis and optimization
- **Task Persistence**: Save and load tasks across sessions
- **Visual Progress**: Tree-structured task display with real-time status

```bash
% /plan create a Rust project with src and tests
🤖 Decomposed into 6 tasks

% /execute
✓ 6/6 · 100% · 10s

% /task save my_build    # Save for later
% /task list              # List all saved tasks
% /task load 0            # Load task
```

### 📊 Unified Tracking
- **Four-Dimensional Observation**: History + Log + LLM-Log + Context
- **Smart Deduplication**: Content hash + time window algorithm
- **Multi-dimensional Query**: By dimension, time, keyword

```bash
% /trace
📊 20 records from 4 sources
📊 Statistics | 🔗 Coordination | 🤖 BlackBox | 💭 Memory
```

### ⚙️ DevOps Toolset
- **Git Assistant**: Smart status, diff analysis, commit message generation
- **Log Analyzer**: Multi-format parsing, error aggregation, health assessment
- **System Monitor**: CPU/Memory/Disk monitoring, process TOP list
- **Project Context**: Auto-detect project type, recommend build/test/run commands

### 🔐 Security
- **Shell Blacklist**: Block dangerous commands (`rm -rf /`, `dd`, fork bombs)
- **Timeout Control**: Default 30s execution limit
- **Output Limits**: Max 100KB to prevent resource exhaustion
- **API Key Security**: Environment variable storage, `.env` excluded from version control

## Examples

### Smart Command Routing

```bash
% ls          # Auto-detected as shell
% pwd         # No ! prefix needed for common commands
% git status  # 100+ commands auto-recognized
```

### Error Recovery

```bash
% cagro build
❌ Command not found: cagro

💡 Did you mean?
  1. cargo (0.93) - Rust package manager
  2. cat (0.65) - Display file

% 1           # Execute: cargo build
```

### Feedback Learning

```bash
# RealConsole learns from your choices
% /suggest
1. cargo check (0.85) - Fast syntax check
2. cargo build (0.80) - Full build

% 1  # Choose cargo check

# After multiple uses
% /suggest
1. cargo check (0.92) ⬆️ # Boosted!
2. cargo build (0.80)
```

### Task Orchestration

```bash
% /plan setup a web API project with routes, models, tests

📊 Execution Plan:
▸ 4 stages · 8 tasks · ⚡ 25s
├─ → Stage 1: Create root
├─ ⇉ Stage 2: Create dirs [parallel]
├─ ⇉ Stage 3: Create files [parallel]
└─ → Stage 4: Init config

% /execute
✓ 8/8 · 100% · 15s
```

### DevOps Workflow

```bash
% /gs          # Git status (colorized)
% /gd          # Diff analysis
% /ga          # Auto-generate commit message
% /la app.log  # Analyze logs
% /sys         # System overview
```

> 📖 **More examples**: [Examples Directory](examples/)

## Documentation

### Getting Started
- **[Quick Start](docs/02-practice/user/quickstart.md)** - Get up and running in 5 minutes
- **[User Guide](docs/02-practice/user/user-guide.md)** - Complete feature documentation
- **[Web Terminal Guide](docs/02-practice/user/web-terminal.md)** - Web version usage

### Core Concepts
- **[One Divides into Three Philosophy](docs/00-core/philosophy.md)** - Design principles
- **[Product Vision](docs/00-core/vision.md)** - Goals and positioning
- **[Architecture](docs/01-understanding/design/architecture.md)** - System design

### For Developers
- **[Developer Guide](docs/02-practice/developer/developer-guide.md)** - Contributing and extending
- **[API Reference](docs/02-practice/developer/api-reference.md)** - Code documentation
- **[Project Structure](docs/02-practice/developer/project-structure.md)** - Codebase organization

### Reference
- **[Command Reference](docs/02-practice/user/commands-reference.md)** - All commands
- **[Configuration](docs/02-practice/user/configuration.md)** - Config file options
- **[Roadmap](docs/00-core/roadmap.md)** - Future plans
- **[Changelog](CHANGELOG.md)** - Version history

> 📖 **Documentation Hub**: [docs/README.md](docs/README.md)

## Architecture

```
User Input
   ↓
Smart Router ──┬── Shell Execution (auto-detected 100+ commands)
               ├── System Commands (/help, /suggest, /trace, etc.)
               └── LLM + Tool Calling (streaming output)
                      ↓
                 Proactive Suggestion System
                 ├── Context Analyzer (project type, history)
                 ├── Spell Checker (Levenshtein distance)
                 ├── Feedback Learner (user preferences)
                 └── Suggestion Cache (smart lifecycle)
```

**Key Components**:
- **Agent**: Unified entry point, command routing
- **LLM Client**: Streaming output, tool calling
- **Task System**: Dependency analysis, parallel execution
- **Suggestion Engine**: Three-source fusion (Context + History + LLM)
- **Tracer**: Four-dimensional observation system
- **Web Server**: Axum framework, WebSocket real-time communication

## What's New

### v1.39.0 - Intent Decomposition Auto-Execute + Eye Protection 🎯👁️

**AI Thinking Visualization + Comfortable Long-Term Use**

#### Core Improvements

**Intent Decomposition Auto-Execute**:
- ✅ `/decompose` command now actually executes tools and returns real results (not just visualization)
- ✅ See both AI thinking process (intent understanding, step planning) and get actual results
- ✅ Consistent intelligent experience with direct execution mode, preserves teaching and debugging value

**Eye Protection Color System Optimization**:
- ✅ Significantly reduced blue/cyan usage (blue light intensity reduced by 83%)
- ✅ Adopted GitHub/Binance professional dark theme style
- ✅ Removed 25+ glow effects, reducing eye fatigue
- ✅ Long-term comfort increased by 113% (40 → 85 score)

```bash
# Try intent decomposition auto-execute
% /decompose calculate 2 + 3
→ Show intent understanding → Show step plan → Auto-execute → Return result: 5

# Eye protection colors are enabled by default, no configuration needed
```

📖 **Details**: [CHANGELOG.md v1.39.0](CHANGELOG.md#1390---2025-01-08)

---

### v1.38.0 - Cell Rerun Feature 🔄

**Jupyter-like Experience Upgrade**

- ✅ One-click rerun of any historical command/conversation (Cell Rerun Feature)
- ✅ Cyberpunk UI - Simple icon-style buttons
- ✅ Real-time feedback - Loading state, error handling, button disabled
- ✅ WebSocket communication - Complete frontend-backend message flow

📖 **Details**: [CHANGELOG.md v1.38.0](CHANGELOG.md#1380---2025-01-08)

---

### v1.28.0 - Web Round Visualization 📒

**Jupyter-ification of Conversation History**

- ✅ Jupyter-like conversation round cards
- ✅ Dual view mode (round view/traditional view) switching
- ✅ Complete metadata display (time, duration, token statistics)
- ✅ Cell collapse/expand for optimized long output display

📖 **Details**: [CHANGELOG.md v1.28.0](CHANGELOG.md#1280---2025-01-07)

---

### v1.24.0 - Full Internationalization Support 🌍

**Seamless Bilingual Experience**

- ✅ Complete CLI internationalization: All command outputs, prompts, error messages
- ✅ Bilingual LLM prompts: System prompts support Chinese context
- ✅ Internationalized YAML config: Configuration file comments in both languages
- ✅ Environment variable control: `REALCONSOLE_LANG=zh-CN|en-US` for dynamic switching

---

### v1.23.0 - Web Terminal Release 🌐

**Access RealConsole Anywhere**

- ✅ Complete web terminal implementation (Axum + WebSocket)
- ✅ Smart routing and Intent understanding
- ✅ Beautiful UI with real-time streaming output
- ✅ Mobile-friendly responsive design
- ✅ LAN access support

📖 **Complete Web Terminal Documentation**: [Web Terminal User Guide](docs/02-practice/user/web-terminal.md)

---

**More historical features** see [CHANGELOG.md](CHANGELOG.md) | **Complete version history**: [docs/03-evolution/version-history.md](docs/03-evolution/version-history.md)

## Disclaimer

This program is implemented using [Claude Code](https://claude.com/claude-code)'s Vibe Coding approach. It is intended for **educational**, **research**, and **technical exploration** purposes only. **Not recommended for production use.**

By using this program, you acknowledge its experimental nature and assume full responsibility for any consequences.

## Contributing

Contributions welcome! See [Developer Guide](docs/02-practice/developer/developer-guide.md) and [Contributing Guide](docs/02-practice/developer/contributing.md).

```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

[MIT License](LICENSE)

## Acknowledgments

- **Inspiration**: SmartConsole (Python version)
- **Community**: Rust Community, LLM Providers (Ollama, Deepseek, Gemini, OpenAI)
- **Development**: Built with [Claude Code](https://claude.com/claude-code)

---

<p align="center">
  <b>RealConsole</b> - Where Philosophy Meets Technology
</p>

<p align="center">
  <a href="https://github.com/hongxin/RealConsole">GitHub</a> •
  <a href="docs/README.md">Documentation</a> •
  <a href="examples/">Examples</a> •
  <a href="https://github.com/hongxin/RealConsole/issues">Issues</a>
</p>
