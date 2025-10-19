# RealConsole (Rust)

> **[中文](README.md) | English**

- An intelligent CLI Agent beloved by programmers and DevOps engineers
- Works seamlessly in Linux/Mac/Windows WSL command-line environments with a smooth user experience
- Built with Rust for high-performance terminal execution

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-654%2B%20passed-green.svg)](tests/)
[![Coverage](https://img.shields.io/badge/coverage-78%2B%25-yellow.svg)](docs/test_reports/)
[![Version](https://img.shields.io/badge/version-1.1.0-blue.svg)](docs/CHANGELOG.md)

## ⚠️ Disclaimer

**Important Notice**: This program is primarily developed using [Claude Code](https://claude.com/claude-code)'s Vibe Coding methodology, an exploratory development approach. As such, we cannot guarantee the program's security and stability.

**Intended Use**:
- For **educational**, **research**, and **technical exploration** purposes only
- Not recommended for production environments

**Liability**:
By using, compiling, or running this program, you acknowledge its experimental nature and potential risks. Users assume full responsibility for any issues, losses, or damages resulting from the use of this program. The developers, contributors, and maintainers of this program shall not be held liable.

**Recommendations**:
- Use cautiously in testing environments
- Regularly backup important data
- Understand each command before execution

---

## ✨ Core Features

### 🧠 AI Capabilities
- **LLM-Powered Conversation** - Ollama/Deepseek support, real-time streaming output, natural language interaction
- **Task Orchestration System** ⭐ NEW - LLM intelligently decomposes complex goals, automatic dependency analysis, and parallel execution optimization (`/plan`, `/execute`)
- **Smart Pipeline Generation** - Automatically understands user intent, converts natural language to file operation commands
- **Automatic Tool Calling** - 14+ built-in tools (calculator, file operations, time queries, etc.), intelligent parallel execution
- **Intent Recognition** - 50+ built-in intent templates, automatically understand user needs and execute
- **Multi-Layer Fallback** - 4-layer guarantee (LLM generation → rule matching → template matching → conversation), ensuring system never fails

**Usage Examples**:
```bash
% Show the 3 largest rs files
🤖 LLM Generated
→ Executing: find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 3

% Calculate 2 to the power of 10
[LLM automatically calls calculator tool]
Based on calculation, 2^10 = 1024

% /plan Create a Rust project with src and tests directories, then create main.rs
🤖 Smart Task Decomposition
▸ 3 Stages · 4 Tasks · ⚡ 15s (saved 5s)
├─ → Stage 1 (5s)
│  └─ Create project root $ mkdir -p myproject
├─ ⇉ Stage 2 (5s)  [Parallel Execution]
│  ├─ Create src directory $ mkdir -p myproject/src
│  └─ Create tests directory $ mkdir -p myproject/tests
└─ → Stage 3 (5s)
   └─ Create main.rs $ touch myproject/src/main.rs

% /execute
✓ 4/4 · 100% · 12s
```

### 🛠️ DevOps Toolkit
- **Project Context Awareness** - Automatically detects project type (Rust/Python/Node/Go/Java), intelligently recommends build/test/run commands (`/project`)
- **Git Smart Assistant** - Status viewing, change analysis, automatic commit message generation (follows Conventional Commits) (`/gs`, `/gd`, `/ga`, `/gb`)
- **Log Analysis Tool** - Multi-format parsing, error aggregation, health assessment (`/la`, `/le`, `/lt`)
- **Safe Shell Execution** - Execute commands with `!` prefix, blacklist protection, timeout control

### 💻 System Monitoring
- **System Resource Monitoring** - Real-time CPU/memory/disk monitoring, process TOP list (`/sys`, `/cpu`, `/disk`, `/top`)
- **Cross-Platform Support** - Full macOS + Linux support, zero additional dependencies
- **Execution Logging** - Complete operation records and auditing

### 🎨 User-Friendly Experience
- **Configuration Wizard** - 5-minute quick initialization (`realconsole wizard --quick`)
- **Multi-Level Help** - Quick/All/Topic help system, example library, quick reference cards (`/help`, `/examples`, `/quickref`)
- **Intelligent Error Messages** - 30+ error codes, detailed fix suggestions, source error tracking
- **Memory System** - Short-term + long-term memory, supports search and export
- **Lazy Mode** - Direct input for conversation, no command prefix needed

## 🚀 Quick Start

### 1. Install Rust

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Build Project

```bash
# Clone repository
git clone https://github.com/your-repo/realconsole.git
cd realconsole

# Build release version
cargo build --release
```

### 3. Configuration Wizard (Recommended for New Users) 🧙

**Quick Mode** (5 minutes):

```bash
./target/release/realconsole wizard --quick
```

The wizard will guide you through:
- ✅ LLM provider selection (Deepseek API / Ollama local)
- ✅ API Key configuration (if using Deepseek)
- ✅ Basic feature settings (Shell execution, memory system, etc.)
- ✅ Automatically generate `realconsole.yaml` and `.env` files

**Full Mode** (more options):

```bash
./target/release/realconsole wizard
```

### 4. Run RealConsole

```bash
# Use default configuration
./target/release/realconsole

# Use specified configuration file
./target/release/realconsole --config realconsole.yaml

# One-shot execution mode
./target/release/realconsole --once "Hello"
```

### Manual Configuration (Advanced Users)

If you don't want to use the configuration wizard:

1. **Copy environment variable example**:
```bash
cp .env.example .env
```

2. **Edit `.env` and fill in API Key**:
```bash
DEEPSEEK_API_KEY=sk-your-key-here
```

3. **Edit `realconsole.yaml` to configure LLM**:
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

features:
  shell_enabled: true
  memory_enabled: true
  tool_calling_enabled: true
```

**Detailed Configuration Guide**:
- [Configuration Wizard Design](docs/01-understanding/design/config-wizard.md)
- [LLM Configuration Guide](docs/02-practice/user/llm-setup.md)
- [Complete User Manual](docs/02-practice/user/user-guide.md)

## 💬 Usage Examples

### 1. Smart Conversation (Lazy Mode)

Direct input, no command prefix needed:

```bash
% Hello
Hi! I'm an AI assistant. How can I help you?

% Write a hello world in Rust
Sure, here's a simple Rust Hello World program:

fn main() {
    println!("Hello, World!");
}

To run it:
1. Save as main.rs
2. Run: rustc main.rs && ./main
```

### 2. Shell Command Execution

Use `!` prefix to execute system commands:

```bash
% !pwd
/Users/user/project/realconsole

% !ls -la
total 96
drwxr-xr-x  10 user  staff   320 Oct 14 10:30 .
...

% !echo "Hello from shell"
Hello from shell

% !date
Mon Oct 14 00:41:12 CST 2025
```

### 3. Tool Calling (Automatic Execution) ✨

After enabling tool calling, LLM will automatically call tools:

```bash
# Enable tool calling (edit realconsole.yaml)
features:
  tool_calling_enabled: true

# Example 1: Automatic calculation
% Calculate 2 to the power of 10

[LLM automatically calls calculator tool]
Based on calculation, 2^10 = 1024

# Example 2: File operations
% Read the first 5 lines of README.md

[LLM automatically calls read_file tool]
File content:
# RealConsole (Rust)
Minimalist intelligent CLI Agent...

# Example 3: Get time
% What time is it now?

[LLM automatically calls get_datetime tool]
Current time is 2025-01-15 14:30:45
```

**Built-in Tools**:
- `calculator` - Mathematical calculations (supports +, -, *, /, ^, sin, cos, sqrt, etc.)
- `read_file` - Read file contents
- `write_file` - Write file contents
- `list_dir` - List directory contents
- `get_datetime` - Get current date and time

**View All Tools**:
```bash
% /tools
Available tools (5 tools):
  • calculator - Execute mathematical expressions
  • read_file - Read file contents
  • write_file - Write file contents
  • list_dir - List directory contents
  • get_datetime - Get current date and time
```

Detailed Documentation:
- **User Guide**: [docs/02-practice/user/tool-calling-guide.md](docs/02-practice/user/tool-calling-guide.md)
- **Developer Guide**: [docs/02-practice/developer/tool-development.md](docs/02-practice/developer/tool-development.md)

---

### 4. Multi-Level Help System 📚

Use `/` prefix to access system commands:

```bash
% /help
RealConsole v1.1.0

💬 Smart Conversation:
  Direct input, no command prefix needed
  Example: Calculate 2 to the power of 10
  Example: Write a hello world in Rust

🚀 Smart Command Routing (Phase 10.1):
  Common commands can be entered directly (automatic recognition)
  ls         List files (auto-detected)
  pwd        Show current directory
  git status View Git status
  !ls -la    Force Shell execution

⚡ Quick Commands:
  /help      Show this help
  /help all  Show all commands (detailed)
  /examples  View usage examples
  /quickref  Quick reference card
  /quit      Exit program

🛠️ Tool Calling:
  /tools        List all tools
  /tools call <name> <args>   Call a tool

💾 Memory & Logs:
  /memory recent    View recent conversations
  /log stats        View execution statistics

Tips:
  Use /help <command> for command details
  System automatically recognizes command types, use /help shell for routing info

% /examples           # Example library
% /quickref           # Quick reference card
% /quit
Bye 👋
```

---

### 5. Task Orchestration System ⭐ NEW

New task orchestration feature in v1.0.0 that lets AI automatically decompose complex goals into executable tasks:

```bash
# Step 1: Describe goal in natural language
% /plan Create a Rust project with src and tests directories, then create main.rs and lib.rs

🤖 LLM Smart Task Decomposition...
✓ Decomposed into 6 subtasks

📊 Execution Plan
▸ 4 Stages · 6 Tasks · ⚡ 20s (saved 10s)
├─ → Stage 1 (5s)
│  └─ Create project root $ mkdir -p myproject
├─ ⇉ Stage 2 (5s)  [Parallel Execution]
│  ├─ Create src directory $ mkdir -p myproject/src
│  └─ Create tests directory $ mkdir -p myproject/tests
├─ ⇉ Stage 3 (5s)  [Parallel Execution]
│  ├─ Create main.rs $ touch myproject/src/main.rs
│  └─ Create lib.rs $ touch myproject/src/lib.rs
└─ → Stage 4 (5s)
   └─ Create test file $ touch myproject/tests/integration_test.rs

Use /execute to run

# Step 2: Execute plan
% /execute
⚡ Starting execution: Create Rust project...

→ Stage 1: Create project root ✓ (2s)
⇉ Stage 2: Parallel execution src and tests directory creation ✓ (3s)
⇉ Stage 3: Parallel execution main.rs and lib.rs creation ✓ (2s)
→ Stage 4: Create test file ✓ (3s)

✓ 6/6 · 100% · 10s
```

**Core Features**:
- ✅ **LLM Smart Decomposition** - Describe goals in natural language, AI automatically breaks down into executable steps
- ✅ **Dependency Analysis** - Kahn topological sort automatically detects task dependencies, ensures execution order
- ✅ **Parallel Optimization** - Automatically identifies parallelizable tasks, significantly improves execution efficiency (max 4 concurrent)
- ✅ **Minimalist Visualization** - Tree structure clearly shows task hierarchy, 75%+ reduction in output lines
- ✅ **Security Protection** - Inherits Shell blacklist and timeout control mechanisms

**Typical Scenarios**:
- Project scaffolding creation (directories, files, configuration initialization)
- Batch file operations (rename, convert, cleanup)
- Data processing pipelines (extract, transform, load)
- Development workflows (build, test, deploy)

Detailed Documentation:
- **Usage Guide**: [examples/task_system_usage.md](examples/task_system_usage.md)
- **Visualization Design**: [examples/task_visualization.md](examples/task_visualization.md)

---

### 6. DevOps Workflows ✨

#### Project Context Awareness

Quickly understand project information and recommended commands:

```bash
% /project
📦 Project Context

  Project Name: realconsole
  Project Type: Rust Project
  Root Directory: /Users/user/realconsole

🔨 Recommended Commands:
  Build: cargo build
  Test: cargo test
  Run: cargo run

📊 Project Info:
  ✓ Found Cargo.toml
  ✓ Found src/ directory
  ✓ Found test directory

🔄 Git Info:
  Branch: main
  Status: 2 files modified
```

#### Git Smart Assistant

Accelerate Git workflow:

```bash
# 1. View Git status (colored categorized display)
% /gs
📊 Git Repository Status

📁 Modified Files (2):
  • src/main.rs
  • Cargo.toml

# 2. View diff analysis
% /gd
📊 Code Change Analysis

📈 Statistics:
  • Added: 120 lines
  • Deleted: 45 lines
  • Modified Files: 2

🔍 Change Patterns:
  ✓ New function definitions detected
  ✓ New test cases detected

# 3. Auto-generate commit message (follows Conventional Commits)
% /ga
📝 Change Analysis & Commit Suggestion

🎯 Change Type: feat (new feature)
📁 Scope: core

💬 Suggested Commit Message:
feat(core): add DevOps features

- Add project context detection
- Add Git smart assistant
- Add log analyzer
- Add system monitor
```

#### Log Analysis Tool

Quickly diagnose log issues:

```bash
# Analyze log file
% /la /var/log/app.log
📊 Log Analysis Report

📈 Statistics:
  • Total Lines: 10,234
  • Time Range: 2025-01-15 10:00:00 - 14:30:45

📊 Log Level Distribution:
  • ERROR: 23 (0.2%)
  • WARN: 156 (1.5%)
  • INFO: 8,945 (87.4%)
  • DEBUG: 1,110 (10.9%)

⚠️ Top 5 Error Patterns:
  1. "Connection timeout after Nms" - 12 occurrences
  2. "Failed to load config from /PATH" - 5 occurrences
  3. "Database query timeout" - 3 occurrences

🏥 Health: Good (ERROR < 1%)
```

#### System Monitoring Tool

Quickly view system resources:

```bash
# System overview (one-click view all resources)
% /sys
💻 System Monitoring

━━━ CPU ━━━
  Usage: 15.3%
  • User: 8.2%
  • System: 7.1%
  • Idle: 84.7%

━━━ Memory ━━━
  Total: 16.0 GB
  Used: 8.5 GB (53%)
  Available: 7.5 GB
  Cache: 2.3 GB

━━━ Disk ━━━
  / (root partition):
    Total: 500 GB
    Used: 320 GB (64%)
    Available: 180 GB
```

---

### 7. Friendly Error Messages ⚠️

RealConsole provides 30+ error codes and detailed fix suggestions:

```bash
% !rm -rf /
[E302] Command contains dangerous operation, blocked by security policy

💡 Fix Suggestions:
1. This command may cause system damage, recommend safer alternatives
2. View allowed command list and security policy
   📖 https://docs.realconsole.com/shell-safety

% !sleep 20
[E303] Command execution timeout (exceeded 10 seconds)

💡 Fix Suggestions:
1. Command execution took too long, check command or increase timeout
2. Adjust features.shell_timeout in configuration file
   💻 vi realconsole.yaml
```

## 📁 Project Structure

```
realconsole/
├── src/                    # 🦀 Core Code
│   ├── agent.rs            # Agent core dispatcher
│   ├── command/            # Command system (DevOps toolkit)
│   ├── task/               # ⭐ Task orchestration system (LLM+Kahn+Parallel)
│   ├── dsl/intent/         # Intent DSL (50+ intent templates)
│   ├── llm/                # LLM integration (Ollama/Deepseek/OpenAI)
│   └── tool/               # Tool calling system (14+ built-in tools)
│
├── tests/                  # 🧪 Tests (654+ passed, 78%+ coverage)
├── docs/                   # 📚 Documentation (Five-state architecture, 226+ archived docs)
├── config/                 # ⚙️ Configuration examples
└── examples/               # 💡 Usage examples
```

**Detailed Structure**: [Complete Project Structure Documentation](docs/02-practice/developer/project-structure.md)

## 🏗️ Architecture Design

### Core Philosophy: Trinity

```
         User Input
            │
      ┌─────▼──────┐
      │ Command    │  ← Smart recognition (Force Shell/System/Common Shell/Natural Language)
      │ Router     │
      └─────┬──────┘
            │
    ┌───────┼───────┐
    │       │       │
  Shell   System   LLM+Tools
  Exec    Cmds    (Streaming)
```

**Core Features**:
- **Smart Routing** - Automatic command type recognition, seamless transition
- **Streaming Output** - SSE real-time response, token-level display
- **Security Protection** - Blacklist + timeout + output limits
- **Multi-Layer Fallback** - LLM generation → rule matching → template → conversation

**Detailed Documentation**: [Architecture Overview](docs/01-understanding/overview.md) | [Design Documents](docs/01-understanding/design/)

## 🔐 Security Features

- **Blacklist Protection** - Blocks dangerous commands like `rm -rf /`, `sudo`, `dd`, `mkfs`, Fork bombs
- **Timeout Control** - Default 30 seconds auto-termination
- **Output Limits** - Maximum 100KB to prevent resource exhaustion
- **API Key Security** - Environment variable storage, `.env` not committed to version control

**Detailed Documentation**: [Shell Safe Execution](docs/03-evolution/features/shell-execution.md)

## 📊 Project Highlights

| Dimension | Data |
|-----------|------|
| 🧪 **Test Quality** | 654+ tests passed (98.9% pass rate) · 78%+ coverage |
| ⚡ **Performance** | Startup < 50ms · Memory ~5MB · LLM first token < 500ms |
| 📝 **Code Scale** | 13,000+ lines of Rust code · 226+ archived documents |
| 🛠️ **Feature Rich** | 50+ Intent templates · 14+ built-in tools · 30+ system commands |

### Latest Updates (v1.1.0 - 2025-10-19)

- 🐛 Fixed OllamaClient parameter order error
- ✨ Enhanced Ollama health check and diagnostics
- 🎨 Added task orchestration UI display functions
- 🧪 Optimized test environment performance

**Complete History**: [CHANGELOG.md](docs/CHANGELOG.md)

## 📚 Documentation Navigation

> **Documentation System**: Five-state architecture based on Trinity philosophy (Philosophy·Understanding·Practice·Evolution·Reports)

### Quick Access

- 📚 **[Documentation Center](docs/README.md)** - Complete navigation and recommended reading paths
- 🚀 **[Quick Start](docs/02-practice/user/quickstart.md)** - 5-minute getting started guide
- 📖 **[User Manual](docs/02-practice/user/user-guide.md)** - Complete feature description
- 👨‍💻 **[Developer Guide](docs/02-practice/developer/developer-guide.md)** - Architecture and extension development

### Core Philosophy

- 💭 **[Trinity Philosophy](docs/00-core/philosophy.md)** - Design philosophy
- 🎯 **[Product Vision](docs/00-core/vision.md)** - Positioning and goals
- 🗺️ **[Technical Roadmap](docs/00-core/roadmap.md)** - Development plan

### More Documentation

- **User Guides**: [LLM Configuration](docs/02-practice/user/llm-setup.md) · [Tool Calling](docs/02-practice/user/tool-calling-guide.md) · [Intent DSL](docs/02-practice/user/intent-dsl-guide.md)
- **Developer Docs**: [API Reference](docs/02-practice/developer/api-reference.md) · [Tool Development](docs/02-practice/developer/tool-development.md) · [Project Structure](docs/02-practice/developer/project-structure.md)
- **Features**: [Task Orchestration](examples/task_system_usage.md) · [Git Assistant](docs/03-evolution/features/git-assistant.md) · [Log Analysis](docs/03-evolution/features/log-analyzer.md)

## 🚧 Planned Features

See [Technical Roadmap](docs/00-core/roadmap.md)

## 🔧 Development

```bash
# Run tests
cargo test

# Code formatting
cargo fmt

# Linting
cargo clippy

# Run demo
./scripts/demo/demo_shell.sh
```

**Detailed Guide**: [Developer Guide](docs/02-practice/developer/developer-guide.md)

## 🤝 Contributing

Contributions welcome! Please ensure passing `cargo test`, `cargo fmt`, and `cargo clippy`

## 📄 License

MIT License - See [LICENSE](LICENSE) file

## 🙏 Acknowledgments

- **Python Version**: [SmartConsole](https://github.com/example/smartconsole) - Design inspiration
- **Rust Community**: Excellent tools and libraries
- **LLM Providers**: Ollama, Deepseek, OpenAI

---

**RealConsole** - Minimalist yet powerful intelligent CLI Agent 🚀
