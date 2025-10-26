# RealConsole

> 🌟 Intelligent CLI Agent Infused with Eastern Philosophy

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-1000%2B-green.svg)](tests/)
[![Version](https://img.shields.io/badge/version-1.8.0-blue.svg)](CHANGELOG.md)

English | **[中文](README.cn.md)**

RealConsole is an intelligent command-line agent built with Rust, based on the "One Divides into Three" (一分为三) philosophy, providing a seamless CLI experience for developers and DevOps engineers.

## ⚡ Key Features

- **🤖 Intelligent Conversation** - LLM-powered natural language interaction (Ollama/Deepseek/OpenAI), multi-turn context, automatic tool calling
- **🛠️ Task Orchestration** - Describe goals in natural language, AI automatically decomposes into executable tasks with intelligent parallel optimization
- **📊 Unified Tracking** - Four-dimensional observation system (History/Log/LLM-Log/Context) reducing cognitive load
- **💡 Proactive Suggestions** - Intelligent suggestion system based on project type, command history, and error analysis, with user feedback learning
- **⚙️ DevOps Toolset** - Git assistant, log analyzer, system monitor, project context awareness
- **🔐 Security Protection** - Shell blacklist, timeout control, output limits, secure API key storage

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole

# Install (recommended)
make install

# Or build manually
cargo build --release
```

### Configuration

Use the interactive wizard for quick setup (recommended):

```bash
realconsole wizard --quick
```

Or configure manually:

```bash
# 1. Copy configuration template
cp .env.example .env

# 2. Add API Key (if using Deepseek)
# .env: DEEPSEEK_API_KEY=sk-your-key-here

# 3. Edit configuration file (optional)
vi realconsole.yaml
```

### Run

```bash
realconsole
```

## 💡 Usage Examples

### Intelligent Conversation

```bash
% hello
Hello! I'm an AI assistant, how can I help you?

% write a hello world in Rust
Sure, here's a simple Rust Hello World program:
...
```

### Shell Commands

```bash
% !ls -la
% !git status
% !cargo build
```

### Task Orchestration

```bash
% /plan create a Rust project with src and tests directories
🤖 AI task decomposition...
✓ Decomposed into 6 subtasks

% /execute
✓ 6/6 · 100% · 10s
```

### Proactive Suggestions

```bash
% /suggest
💡 Suggestions based on current context:
  1. cargo build - Build the project
  2. cargo test - Run tests
  3. git commit -m "..." - Commit changes

% 1           # Quick execute the first suggestion
```

### Unified Tracking

```bash
% /trace
📊 Unified Tracking - 20 Records
📊 ✓ [15:30:42] Statistics: ls -la
🔗 ✓ [15:30:45] Coordination: Execute Shell → Success
🤖 ✓ [15:30:50] BlackBox: Model: deepseek-chat
💭 ✓ [15:31:00] Memory: user: show top 3 .rs files
```

## 📚 Documentation

- **[Quick Start](docs/02-practice/user/quickstart.md)** - 5-minute onboarding guide
- **[User Manual](docs/02-practice/user/user-guide.md)** - Complete feature documentation
- **[Developer Guide](docs/02-practice/developer/developer-guide.md)** - Architecture and extensions
- **[Documentation Hub](docs/README.md)** - Complete documentation navigation

### Core Philosophy

- **[One Divides into Three Philosophy](docs/00-core/philosophy.md)** - Design principles
- **[Product Vision](docs/00-core/vision.md)** - Positioning and goals
- **[Roadmap](docs/00-core/roadmap.md)** - Development plan

## 🏗️ Architecture

Modular design based on "One Divides into Three" philosophy:

```
User Input
   ↓
Smart Router ──┬── Shell Execution
               ├── System Commands (/help, /tools, /trace...)
               └── LLM + Tool Calling (streaming output)
```

Details: [System Architecture](docs/01-understanding/design/architecture.md)

## ⚠️ Disclaimer

This program is primarily implemented using [Claude Code](https://claude.com/claude-code)'s Vibe Coding approach. It is intended for **educational**, **research**, and **technical exploration** purposes only. Not recommended for production use.

By using, compiling, or running this program, you acknowledge its experimental nature and potential risks. You assume full responsibility for any issues, losses, or damages resulting from its use.

## 🤝 Contributing

Contributions are welcome! See [Developer Guide](docs/02-practice/developer/developer-guide.md)

## 📄 License

[MIT License](LICENSE)

## 🙏 Acknowledgments

- **Python Version**: SmartConsole - Design inspiration
- **Rust Community**: Excellent tools and libraries
- **LLM Providers**: Ollama, Deepseek, OpenAI

---

**Project Homepage**: https://github.com/hongxin/RealConsole
