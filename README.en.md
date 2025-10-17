# RealConsole (Rust)

> **[中文](README.md) | English**

An intelligent CLI agent beloved by programmers and DevOps engineers - High-performance Rust implementation

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-645%2B%20passed-green.svg)](tests/)
[![Coverage](https://img.shields.io/badge/coverage-78%2B%25-yellow.svg)](docs/test_reports/)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](docs/CHANGELOG.md)

## ⚠️ Disclaimer

**Important Notice**: This program is primarily developed using the VIBE CODING methodology with [Claude Code](https://claude.com/claude-code), an exploratory development approach. Therefore, we cannot guarantee the security and stability of the program.

**Intended Use**:
- This program is intended for **educational**, **research**, and **technical exploration** purposes only
- Not recommended for production environments

**Liability Statement**:
By using, compiling, or running this program, you acknowledge that you are fully aware of its experimental nature and potential risks. You assume all responsibility for any issues, losses, or damages arising from the use of this program. The developers, contributors, and maintainers of this program bear no responsibility whatsoever.

**Recommendations**:
- Use cautiously in test environments
- Regularly back up important data
- Understand what each command does before executing it

---

## ✨ Core Features

### 🧠 Intelligent AI Capabilities
- **LLM-Powered Conversations** - Supports Ollama/Deepseek with real-time streaming output and natural language interaction
- **Task Orchestration System** ⭐ NEW - LLM intelligently decomposes complex goals, auto-analyzes dependencies, and optimizes parallel execution (`/plan`, `/execute`)
- **Intelligent Pipeline Generation** - Automatically understands user intent and converts natural language to file operation commands
- **Automatic Tool Calling** - 14+ built-in tools (calculator, file operations, time queries, etc.) with smart parallel execution
- **Intent Recognition** - 50+ built-in intent templates that automatically understand and execute user needs
- **Multi-layer Fallback Mechanism** - 4-layer guarantee (LLM generation → rule matching → template matching → conversation) ensures the system never fails

**Usage Examples**:
```bash
» Show the 3 largest .rs files
🤖 LLM Generated
→ Execute: find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 3

» Calculate 2 to the power of 10
[LLM automatically calls calculator tool]
Based on the calculation, 2^10 = 1024

» /plan Create a Rust project with src and tests directories, then create main.rs
🤖 Intelligent Task Decomposition
▸ 3 Stages · 4 Tasks · ⚡ 15s (saved 5s)
├─ → Stage 1 (5s)
│  └─ Create project root $ mkdir -p myproject
├─ ⇉ Stage 2 (5s)  [Parallel Execution]
│  ├─ Create src directory $ mkdir -p myproject/src
│  └─ Create tests directory $ mkdir -p myproject/tests
└─ → Stage 3 (5s)
   └─ Create main.rs file $ touch myproject/src/main.rs

» /execute
✓ 4/4 · 100% · 12s
```

### 🛠️ DevOps Toolkit
- **Project Context Awareness** - Auto-detects project types (Rust/Python/Node/Go/Java), intelligently recommends build/test/run commands (`/project`)
- **Git Smart Assistant** - Status viewing, change analysis, auto-generates commit messages (follows Conventional Commits) (`/gs`, `/gd`, `/ga`, `/gb`)
- **Log Analysis Tool** - Multi-format parsing, error aggregation, health assessment (`/la`, `/le`, `/lt`)
- **Safe Shell Execution** - Execute commands with `!` prefix, blacklist protection, timeout control

### 💻 System Monitoring
- **System Resource Monitoring** - Real-time CPU/memory/disk monitoring, process TOP list (`/sys`, `/cpu`, `/disk`, `/top`)
- **Cross-platform Support** - Full macOS + Linux support with zero external dependencies
- **Execution Logs** - Complete operation records and auditing

### 🎨 User-Friendly Experience
- **Configuration Wizard** - Complete initialization in 5 minutes (`realconsole wizard --quick`)
- **Multi-level Help System** - Quick/All/Topic help system, example library, quick reference cards (`/help`, `/examples`, `/quickref`)
- **Intelligent Error Messages** - 30+ error codes with detailed fix suggestions and source error tracking
- **Memory System** - Short-term + long-term memory with search and export support
- **Lazy Mode** - Direct input for conversation, no command prefix needed

## 🚀 Quick Start

### 1. Install Rust

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Build the Project

```bash
# Clone the repository
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole

# Build release version
cargo build --release
```

### 3. Configuration Wizard (Recommended for New Users) 🧙

**Quick Mode** (5 minutes):

```bash
./target/release/realconsole wizard --quick
```

The wizard will guide you through:
- ✅ LLM provider selection (Deepseek API / Local Ollama)
- ✅ API Key configuration (if using Deepseek)
- ✅ Basic feature setup (Shell execution, memory system, etc.)
- ✅ Automatic generation of `realconsole.yaml` and `.env` files

**Complete Mode** (more options):

```bash
./target/release/realconsole wizard
```

### 4. Run RealConsole

```bash
# Use default configuration
./target/release/realconsole

# Use specified config file
./target/release/realconsole --config realconsole.yaml

# Single execution mode
./target/release/realconsole --once "hello"
```

### Manual Configuration (Advanced Users)

If you don't want to use the configuration wizard, you can manually create the configuration:

1. **Copy environment variable example**:
```bash
cp .env.example .env
```

2. **Edit `.env` to add API Key**:
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

**Detailed Configuration Guides**:
- [Configuration Wizard Design](docs/01-understanding/design/config-wizard.md)
- [LLM Setup Guide](docs/02-practice/user/llm-setup.md)
- [Complete User Manual](docs/02-practice/user/user-guide.md)

## 💬 Usage Examples

### 1. Intelligent Conversations (Lazy Mode)

Directly input questions without command prefix:

```bash
» hello
Hello! I'm an AI assistant. How can I help you today?

» Write a hello world in Rust
Sure! Here's a simple Rust Hello World program:

fn main() {
    println!("Hello, World!");
}

To run it:
1. Save as main.rs
2. Run: rustc main.rs && ./main
```

### 2. Shell Command Execution

Use the `!` prefix to execute system commands:

```bash
» !pwd
/Users/user/project/realconsole

» !ls -la
total 96
drwxr-xr-x  10 user  staff   320 Oct 14 10:30 .
...

» !echo "Hello from shell"
Hello from shell

» !date
Mon Oct 14 00:41:12 CST 2025
```

### 3. Tool Calling (Automatic Execution) ✨

After enabling tool calling, LLM will automatically invoke tools to complete tasks:

```bash
# Enable tool calling (edit realconsole.yaml)
features:
  tool_calling_enabled: true

# Example 1: Automatic calculation
» Calculate 2 to the power of 10

[LLM automatically calls calculator tool]
Based on the calculation, 2^10 = 1024

# Example 2: File operations
» Read the first 5 lines of README.md

[LLM automatically calls read_file tool]
File content:
# RealConsole (Rust)
Minimalist intelligent CLI Agent...

# Example 3: Get time
» What time is it now?

[LLM automatically calls get_datetime tool]
Current time is 2025-01-15 14:30:45
```

**Built-in Tools**:
- `calculator` - Math calculations (supports +, -, *, /, ^, sin, cos, sqrt, etc.)
- `read_file` - Read file content
- `write_file` - Write file content
- `list_dir` - List directory contents
- `get_datetime` - Get current date and time

**View All Tools**:
```bash
» /tools
5 available tools:
  • calculator - Execute mathematical expressions
  • read_file - Read file content
  • write_file - Write file content
  • list_dir - List directory contents
  • get_datetime - Get current date and time
```

Detailed Documentation:
- **User Guide**: [docs/02-practice/user/tool-calling-guide.md](docs/02-practice/user/tool-calling-guide.md)
- **Developer Guide**: [docs/02-practice/developer/tool-development.md](docs/02-practice/developer/tool-development.md)

---

### 4. Multi-level Help System 📚

Use the `/` prefix to access system commands:

```bash
» /help
💬 RealConsole v0.7.0

Intelligent Conversation:
  Directly input questions, no command prefix needed
  Example: calculate 2 to the power of 10

⚡ Quick Commands:
  /help       Show this help
  /help all   Show all commands (detailed)
  /examples   View usage examples
  /quickref   Quick reference card
  /quit       Exit program
...

» /help all           # Complete help
» /help tools         # Tool management help
» /help memory        # Memory system help
» /help shell         # Shell execution help

» /examples           # Example library
💡 RealConsole Usage Examples

━━━ Intelligent Conversation ━━━
  Calculate 2 to the power of 10
  Write a hello world in Rust
  ...

» /quickref           # Quick reference card
╭─────────────── RealConsole Quick Reference ───────────────╮
│                                                           │
│  Conversation        Direct input                         │
│  Execute Shell       !<command>                           │
│  System Command      /<command>                           │
│  ...                                                      │
╰───────────────────────────────────────────────────────────╯

» /quit
Bye 👋
```

---

### 5. Task Orchestration System ⭐ NEW

New task orchestration feature in v1.0.0 - AI automatically decomposes complex goals into executable tasks:

```bash
# Step 1: Describe the goal in natural language
» /plan Create a Rust project with src and tests directories, then create main.rs and lib.rs

🤖 LLM intelligently decomposing tasks...
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

# Step 2: Execute the plan
» /execute
⚡ Starting execution: Creating Rust project...

→ Stage 1: Create project root ✓ (2s)
⇉ Stage 2: Parallel create src and tests directories ✓ (3s)
⇉ Stage 3: Parallel create main.rs and lib.rs ✓ (2s)
→ Stage 4: Create test file ✓ (3s)

✓ 6/6 · 100% · 10s

# View task list
» /tasks
Create Rust project 6 tasks · 4 stages · 20s
├─ → Stage 1
│  └─ Create project root
├─ ⇉ Stage 2
│  ├─ Create src directory
│  └─ Create tests directory
└─ ...

# View execution status
» /task_status
✓ 6/6 · 10s · 100%
✓ Create project root (2s)
✓ Create src directory (3s)
✓ Create tests directory (3s)
✓ Create main.rs (2s)
✓ Create lib.rs (2s)
✓ Create test file (3s)
```

**Core Features**:
- ✅ **LLM Smart Decomposition** - Describe goals in natural language, AI auto-breaks into executable steps
- ✅ **Dependency Analysis** - Kahn topological sort auto-detects task dependencies, ensures execution order
- ✅ **Parallel Optimization** - Auto-identifies parallel tasks, significantly improves efficiency (max 4 concurrent)
- ✅ **Minimalist Visualization** - Tree structure clearly shows task hierarchy, 75%+ fewer output lines
- ✅ **Security Protection** - Inherits Shell blacklist and timeout control mechanisms

**Typical Scenarios**:
- Project scaffolding creation (directories, files, config initialization)
- Batch file operations (rename, convert, cleanup)
- Data processing pipelines (extract, transform, load)
- Development workflows (build, test, deploy)

Detailed Documentation:
- **Usage Guide**: [examples/task_system_usage.md](examples/task_system_usage.md)
- **Visualization Design**: [examples/task_visualization.md](examples/task_visualization.md)

---

### 6. DevOps Workflow ✨

#### Project Context Awareness

Quickly understand project information and recommended commands:

```bash
» /project
📦 Project Context

  Project Name: realconsole
  Project Type: Rust Project
  Root Directory: /Users/user/realconsole

🔨 Recommended Commands:
  Build: cargo build
  Test: cargo test
  Run: cargo run

📊 Project Information:
  ✓ Found Cargo.toml
  ✓ Found src/ directory
  ✓ Found test directory

🔄 Git Information:
  Branch: main
  Status: 2 files modified
```

#### Git Smart Assistant

Accelerate your Git workflow:

```bash
# 1. View Git status (color-coded categorized display)
» /gs
📊 Git Repository Status

📁 Modified Files (2):
  • src/main.rs
  • Cargo.toml

# 2. View diff analysis
» /gd
📊 Code Change Analysis

📈 Statistics:
  • Added: 120 lines
  • Deleted: 45 lines
  • Modified files: 2

🔍 Change Patterns:
  ✓ New function definitions detected
  ✓ New test cases detected

# 3. Auto-generate commit message (follows Conventional Commits)
» /ga
📝 Change Analysis & Commit Suggestion

🎯 Change Type: feat (new feature)
📁 Scope: core

💬 Suggested Commit Message:
feat(core): add DevOps features

- Add project context detection
- Add Git smart assistant
- Add log analyzer
- Add system monitor

Detailed changes:
- Added 3,431 lines of code
- Added 37+ tests
- 4 new modules
```

#### Log Analysis Tool

Quickly diagnose log issues:

```bash
# Analyze log file
» /la /var/log/app.log
📊 Log Analysis Report

📈 Statistics:
  • Total lines: 10,234
  • Time range: 2025-01-15 10:00:00 - 14:30:45

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

# View errors only
» /le /var/log/app.log
[Shows all ERROR level logs...]

# Monitor log tail in real-time (similar to tail -f)
» /lt /var/log/app.log
[Real-time display of new log entries...]
```

#### System Monitoring Tool

Quickly view system resources:

```bash
# System overview (view all resources at once)
» /sys
💻 System Monitor

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

# CPU details
» /cpu
[Shows detailed CPU information...]

# Memory details
» /memory-info
[Shows detailed memory information...]

# Disk usage
» /disk
[Shows all disk partitions...]

# Process TOP list
» /top
🔝 Process Resource Usage TOP 5

Sorted by CPU:
  1. chrome - 45.2% CPU, 1.2 GB memory
  2. node - 12.3% CPU, 850 MB memory
  ...
```

---

### 7. Friendly Error Messages ⚠️

RealConsole provides 30+ error codes with detailed fix suggestions:

```bash
» !rm -rf /
[E302] Command contains dangerous operation, blocked by security policy

💡 Fix Suggestions:
1. This command may cause system damage, safer alternatives recommended
2. View allowed command list and security policy
   📖 https://docs.realconsole.com/shell-safety

» !sleep 20
[E303] Command execution timeout (exceeded 10 seconds)

💡 Fix Suggestions:
1. Command took too long, please check command or increase timeout
2. Adjust features.shell_timeout in config file
   💻 vi realconsole.yaml

» realconsole --config nonexistent.yaml
[E001] Configuration file not found: nonexistent.yaml

💡 Fix Suggestions:
1. Run configuration wizard to create config file
   💻 realconsole wizard --quick
2. View configuration guide
   📖 https://docs.realconsole.com/config
```

## 📁 Project Structure

```
realconsole/
├── README.md                 # Project documentation
├── Cargo.toml                # Rust project config
├── realconsole.yaml        # Main config file
├── .env                      # Environment variables (not committed)
│
├── src/                      # 🦀 Source code
│   ├── main.rs               # Program entry point
│   ├── agent.rs              # Agent core (Intent DSL integration)
│   ├── repl.rs               # REPL interaction loop
│   ├── config.rs             # Configuration system
│   ├── command/              # Command system
│   │   ├── command.rs        # Command registry & dispatch
│   │   ├── commands_core.rs  # Core commands
│   │   ├── task_cmd.rs       # Task orchestration commands ⭐ NEW
│   │   └── commands_*.rs     # Other command modules
│   ├── shell_executor.rs     # Shell command execution
│   ├── llm_manager.rs        # LLM manager
│   ├── llm/                  # LLM clients
│   │   ├── ollama.rs
│   │   ├── deepseek.rs
│   │   └── openai.rs
│   ├── task/                 # ⭐ NEW - Task orchestration system
│   │   ├── mod.rs            # Module definition
│   │   ├── types.rs          # Task data structures
│   │   ├── decomposer.rs     # LLM task decomposer
│   │   ├── planner.rs        # Dependency analysis & planning
│   │   └── executor.rs       # Parallel execution engine
│   ├── dsl/                  # DSL system
│   │   ├── intent/           # Intent DSL
│   │   │   ├── types.rs      # Core data structures
│   │   │   ├── matcher.rs    # Intent matcher
│   │   │   ├── template.rs   # Template engine
│   │   │   ├── builtin.rs    # 50+ built-in intents
│   │   │   └── extractor.rs  # Entity extraction engine
│   │   └── type_system/      # Type system
│   ├── memory.rs             # Memory system
│   ├── tool.rs               # Tool registry
│   ├── tool_executor.rs      # Tool execution engine
│   └── builtin_tools.rs      # 14+ built-in tools
│
├── tests/                    # 🧪 Tests
│   ├── test_*.rs             # Unit tests
│   └── test_intent_integration.rs  # Intent DSL integration test
│
├── docs/                     # 📚 Documentation (Five-tier architecture)
│   ├── README.md             # Documentation center index
│   ├── CHANGELOG.md          # Complete development history
│   ├── 00-core/              # Core philosophy (philosophy, vision, roadmap)
│   ├── 01-understanding/     # Understanding tier (design, analysis, thinking)
│   │   ├── design/           # Design documents
│   │   ├── analysis/         # Analysis documents
│   │   └── thinking/         # Thinking notes
│   ├── 02-practice/          # Practice tier (guides, use cases, examples)
│   │   ├── user/             # User guides
│   │   ├── developer/        # Developer guides
│   │   └── use-cases/        # Use cases
│   ├── 03-evolution/         # Evolution tier (progress, features)
│   │   ├── phases/           # Phase summaries
│   │   ├── features/         # Feature documentation
│   │   └── milestones/       # Milestones
│   ├── 04-reports/           # Collaboration reports (decision records)
│   └── archive/              # Archive (226 historical documents)
│
├── config/                   # ⚙️ Config samples
│   ├── minimal.yaml          # Minimal config example
│   └── test-memory.yaml      # Test memory config
│
├── scripts/                  # 🔧 Script tools
│   ├── demo/                 # Demo scripts
│   └── test/                 # Test scripts
│
└── memory/                   # 💾 Memory storage
    └── long_memory.jsonl     # Long-term memory
```

## 🏗️ Architecture Design

### Core Components

```
┌─────────────────────────────────────────┐
│              User Input                  │
└────────────────┬────────────────────────┘
                 │
         ┌───────▼────────┐
         │  Agent::handle │
         └───────┬────────┘
                 │
      ┏━━━━━━━━━┻━━━━━━━━━┓
      ┃                    ┃
┌─────▼─────┐      ┌──────▼───────┐
│ Text Input │      │ Shell (!prefix)│
│           │      │               │
│ LLM Stream│      │ Shell Executor│
│  Output   │      │               │
└───────────┘      └──────────────┘
```

### LLM Streaming Output

- **SSE Parsing**: Server-Sent Events format
- **Real-time Callback**: Each token displayed immediately
- **Graceful Degradation**: Non-streaming clients auto-degrade

Detailed Design: [docs/03-evolution/features/streaming.md](docs/03-evolution/features/streaming.md)

### Shell Execution

- **Blacklist Check**: Prohibits dangerous commands (rm -rf /, sudo, etc.)
- **Timeout Control**: Auto-terminates after 30 seconds
- **Output Limit**: Maximum 100KB
- **Cross-platform**: Unix/Windows support

Detailed Design: [docs/03-evolution/features/shell-execution.md](docs/03-evolution/features/shell-execution.md)

## 🔐 Security Features

### Shell Command Blacklist

The following dangerous commands are prohibited:
- `rm -rf /` - Delete root directory
- `sudo` - Privilege escalation
- `dd if=/dev/zero` - Disk write
- `mkfs` - Format
- Fork bombs, shutdown, reboot, etc.

Example:
```bash
» !rm -rf /
Shell execution failed: Dangerous command prohibited: Matched pattern 'rm\s+-rf\s+/'

» !sudo apt-get update
Shell execution failed: Dangerous command prohibited: Matched pattern 'sudo\s+'
```

## 📊 Performance Metrics

| Metric | Value |
|--------|-------|
| Startup Time | < 50ms |
| Memory Usage | ~5MB |
| LLM First Token Latency | < 500ms |
| Shell Command Timeout | 30s (configurable) |
| Maximum Output | 100KB (configurable) |

## 🎯 Project Statistics

### Code Quality
- **Test Coverage**: 645+ tests passed (100% pass rate)
- **Lines of Code**: 13,000+ lines of Rust code
- **Test Coverage Rate**: 78%+
- **Clippy Warnings**: 0

### Main Commands
- **System Commands**: `/help`, `/quit`, `/clear`, `/examples`, `/quickref`
- **Task Orchestration** ⭐ NEW: `/plan`, `/execute`, `/tasks`, `/task_status`
- **Project Tools**: `/project`, `/proj`
- **Git Assistant**: `/gs`, `/gd`, `/ga`, `/gb`
- **Log Analysis**: `/la`, `/le`, `/lt`
- **System Monitoring**: `/sys`, `/cpu`, `/disk`, `/top`, `/memory-info`
- **Tool Management**: `/tools`, `/mem`, `/log`, `/shell`

### Architecture Features
- **REPL Interaction Loop** - CLI based on rustyline
- **LLM Integration** - Supports Ollama/Deepseek/OpenAI with streaming output (SSE)
- **Task Orchestration System** ⭐ NEW - LLM smart decomposition + Kahn topological sort + parallel optimization
- **Tool Calling System** - 14+ built-in tools with parallel execution support
- **Intent DSL** - 50+ built-in intent templates with smart matching
- **Memory System** - Short-term + long-term memory with search and export
- **Security Protection** - Blacklist check, timeout control, output limit
- **Cross-platform Support** - macOS + Linux with zero external dependencies

For detailed development history and technical documentation, see [CHANGELOG](docs/CHANGELOG.md)

## 🚧 Planned Features

- [ ] Command history search
- [ ] Tab auto-completion
- [ ] Vector retrieval optimization
- [ ] Web interface
- [ ] More built-in intents (currently 10)
- [ ] Intent DSL performance optimization (LRU cache, fuzzy matching)

## 📚 Documentation

> **Documentation system upgraded**: Five-tier architecture based on "one divides into three" philosophy, clear and easy to navigate ✨

### Documentation Center
- **📚 Main Documentation Index**: [docs/README.md](docs/README.md) - Complete documentation navigation and recommended reading paths

### Core Documents (00-core)
- **💭 One Divides into Three Philosophy**: [docs/00-core/philosophy.md](docs/00-core/philosophy.md) - Design philosophy
- **🎯 Product Vision**: [docs/00-core/vision.md](docs/00-core/vision.md) - Product positioning
- **🗺️ Technical Roadmap**: [docs/00-core/roadmap.md](docs/00-core/roadmap.md) - Development plan

### User Guides (02-practice/user)
- **🚀 Quick Start**: [docs/02-practice/user/quickstart.md](docs/02-practice/user/quickstart.md) - Get started in 5 minutes
- **📖 User Manual**: [docs/02-practice/user/user-guide.md](docs/02-practice/user/user-guide.md) - Complete feature description
- **🛠️ Tool Calling Guide**: [docs/02-practice/user/tool-calling-guide.md](docs/02-practice/user/tool-calling-guide.md)
- **🧠 Intent DSL**: [docs/02-practice/user/intent-dsl-guide.md](docs/02-practice/user/intent-dsl-guide.md)
- **🔧 LLM Setup**: [docs/02-practice/user/llm-setup.md](docs/02-practice/user/llm-setup.md)
- **🌐 Environment Variables**: [docs/02-practice/user/env-config.md](docs/02-practice/user/env-config.md)

### Developer Documentation (01-understanding & 02-practice/developer)
- **🏗️ Architecture Overview**: [docs/01-understanding/overview.md](docs/01-understanding/overview.md)
- **👨‍💻 Developer Guide**: [docs/02-practice/developer/developer-guide.md](docs/02-practice/developer/developer-guide.md)
- **🔨 Tool Development**: [docs/02-practice/developer/tool-development.md](docs/02-practice/developer/tool-development.md)
- **📘 API Reference**: [docs/02-practice/developer/api-reference.md](docs/02-practice/developer/api-reference.md)

### Feature Documentation (03-evolution/features)
- **🔄 Git Smart Assistant**: [docs/03-evolution/features/git-assistant.md](docs/03-evolution/features/git-assistant.md)
- **📊 Log Analyzer**: [docs/03-evolution/features/log-analyzer.md](docs/03-evolution/features/log-analyzer.md)
- **💻 System Monitor**: [docs/03-evolution/features/system-monitor.md](docs/03-evolution/features/system-monitor.md)
- **🧙 Configuration Wizard**: [docs/03-evolution/features/config-wizard.md](docs/03-evolution/features/config-wizard.md)
- **⚡ Lazy Mode**: [docs/03-evolution/features/lazy-mode.md](docs/03-evolution/features/lazy-mode.md)
- **🌊 Streaming Output**: [docs/03-evolution/features/streaming.md](docs/03-evolution/features/streaming.md)
- **🔧 Shell Execution**: [docs/03-evolution/features/shell-execution.md](docs/03-evolution/features/shell-execution.md)
- **📚 Feature Summary**: [docs/03-evolution/features/summary.md](docs/03-evolution/features/summary.md)

### Progress & Reports
- **📊 Development History**: [docs/CHANGELOG.md](docs/CHANGELOG.md) - Complete changelog
- **📈 Phase Summaries**: [docs/03-evolution/phases/](docs/03-evolution/phases/) - Development progress records
- **📝 Collaboration Reports**: [docs/04-reports/](docs/04-reports/) - Important decisions and analysis reports

## 🔧 Development

### Run Tests

```bash
cargo test
```

### Run Demos

```bash
# Shell execution demo
./scripts/demo/demo_shell.sh

# Lazy mode demo
./scripts/demo/demo_lazy_mode.sh

# Deepseek demo
./scripts/demo/demo-deepseek.sh
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## 🤝 Contributing

Contributions welcome! Please ensure:
1. Code passes `cargo test`
2. Code is formatted with `cargo fmt`
3. No clippy warnings `cargo clippy`

## 📄 License

MIT License - See [LICENSE](LICENSE) file for details

## 🙏 Acknowledgments

- **Python Version**: [SmartConsole](https://github.com/example/smartconsole) - Design inspiration
- **Rust Community**: Excellent tools and libraries
- **LLM Providers**: Ollama, Deepseek, OpenAI

---

**RealConsole** - Minimalist yet powerful intelligent CLI Agent 🚀

> **[中文](README.md) | English**
