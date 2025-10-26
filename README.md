# RealConsole

> Intelligent CLI Agent Infused with Eastern Philosophy

[Installation](#installation) | [Quick Start](#quick-start) | [Documentation](#documentation) | [Examples](#examples)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1000%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.8.0-blue.svg)](CHANGELOG.md)

English | **[中文](README.cn.md)**

**RealConsole** is an intelligent command-line agent built with Rust, based on the "One Divides into Three" (一分为三) philosophy. It combines LLM-powered conversation, proactive suggestions, task orchestration, and DevOps tools to provide a seamless CLI experience.

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

**Requirements**: Rust 1.70+, LLM Provider ([Ollama](https://ollama.ai/)/[Deepseek](https://platform.deepseek.com/)/[OpenAI](https://platform.openai.com/))

> 📖 **Detailed instructions**: [Installation Guide](docs/QUICKSTART.md#installation)

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

```bash
realconsole
```

### 3. Try it out

```bash
% hello                           # Chat with AI
% !ls -la                         # Execute shell commands
% /suggest                        # Get proactive suggestions ⭐
% /plan create a Rust project    # Task orchestration
% /trace                          # Unified tracking
```

> 📖 **Full guide**: [Quick Start Guide](docs/QUICKSTART.md)

## Key Features

### 🤖 Intelligent Conversation
- **LLM Integration**: Ollama (local) / Deepseek / OpenAI support
- **Streaming Output**: Real-time token-by-token display
- **Multi-turn Context**: Automatic context management (Auto/Manual/Disabled modes)
- **Tool Calling**: 14+ built-in tools (calculator, file ops, datetime, etc.)

### 💡 Proactive Suggestions ⭐ NEW (v1.8.0)
- **Context-Aware**: Intelligent suggestions based on project type, command history, and errors
- **Quick Execute**: Number shortcuts (e.g., `1`, `2`, `3`) to run suggestions instantly
- **Spell Checking**: Automatic typo correction with Levenshtein distance algorithm
- **Feedback Learning**: Adapts to your preferences through user feedback (acceptance rate + position)
- **Smart Caching**: Recently shown suggestions cached with expiration (2.5-5 min lifecycle)

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
- **Visual Progress**: Tree-structured task display with real-time status

```bash
% /plan create a Rust project with src and tests
🤖 Decomposed into 6 tasks

% /execute
✓ 6/6 · 100% · 10s
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
% git status  # 80+ commands auto-recognized
```

### Error Recovery

```bash
% !cagro build
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
- **[Quick Start](docs/QUICKSTART.md)** - Get up and running in 5 minutes
- **[User Guide](docs/02-practice/user/user-guide.md)** - Complete feature documentation
- **[FAQ](docs/02-practice/user/faq.md)** - Common questions

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
Smart Router ──┬── Shell Execution (! prefix or auto-detected)
               ├── System Commands (/help, /suggest, /trace, etc.)
               └── LLM + Tool Calling (streaming output)
                      ↓
                 Proactive Suggestion System ⭐
                 ├── Context Analyzer (project type, history)
                 ├── Spell Checker (Levenshtein distance)
                 ├── Feedback Learner (user preferences)
                 └── Suggestion Cache (2.5-5 min lifecycle)
```

**Key Components**:
- **Agent**: Unified entry point, command routing
- **LLM Client**: Streaming output, tool calling
- **Task System**: Dependency analysis, parallel execution
- **Suggestion Engine**: Three-source fusion (Context + History + LLM)
- **Tracer**: Four-dimensional observation system

## What's New in v1.8.0

### Proactive Suggestion System

**Phase 4.2 Complete** - Three major features:

1. **P0 - Quick Execute & Enhanced Error Analysis**
   - Number shortcuts for instant suggestion execution
   - 11 error patterns (command not found, permission denied, etc.)

2. **P1 - Spell Checking & Suggestion Cache**
   - Levenshtein distance algorithm with 100+ command dictionary
   - Three-state cache lifecycle (Fresh/Stale/Expired)

3. **P2.1 - Feedback Learning System**
   - Three-state feedback (Accepted/Skipped/Rejected)
   - Quality score = 70% acceptance rate + 30% position
   - Automatic score adjustment (0.5x-1.5x range)

📖 **Details**: [CHANGELOG.md](CHANGELOG.md) | [Completion Reports](docs/04-reports/)

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
- **Community**: Rust Community, LLM Providers (Ollama, Deepseek, OpenAI)
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
