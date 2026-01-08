# RealConsole Quick Start

Welcome to RealConsole! This guide will help you complete installation, configuration, and get started in **5 minutes**.

**[中文](quickstart.md)** | English

## Table of Contents
- [Quick Installation](#quick-installation)
- [First Run](#first-run)
- [Core Features](#core-features)
- [Common Commands](#common-commands)
- [Troubleshooting](#troubleshooting)
- [Next Steps](#next-steps)

---

## Quick Installation

### 1. Build the Project

```bash
cd realconsole
cargo build --release
```

Build time is approximately 2-3 minutes. The binary will be located at `target/release/realconsole`.

### 2. First Run: Configuration Wizard

RealConsole provides an interactive configuration wizard for quick initialization:

```bash
./target/release/realconsole wizard
```

Or use quick mode (recommended for new users):

```bash
./target/release/realconsole wizard --quick
```

**The wizard will guide you through**:
- LLM provider selection (Deepseek API or Ollama local)
- API Key configuration (if using Deepseek)
- Basic feature settings (Shell execution, memory system, etc.)
- Auto-generation of `realconsole.yaml` configuration file

**Quick mode example**:
```
RealConsole Configuration Wizard (Quick Mode)

LLM Provider:
1. Deepseek API (Recommended, cloud service)
2. Ollama (Local)

Select (1-2): 1

Enter Deepseek API Key: sk-xxxxxxxx

Configuration file generated: realconsole.yaml
Environment file created: .env

Now you can run: realconsole
```

### 3. Configuration File Location (Optional)

RealConsole supports flexible configuration file locations, automatically searching:

```
1. Current directory:    ./realconsole.yaml
2. User config directory: ~/.realconsole/realconsole.yaml
```

**Recommended practice** (global usage):

```bash
# Move config to user directory
mkdir -p ~/.realconsole
mv realconsole.yaml ~/.realconsole/
mv .env ~/.realconsole/  # If using .env file

# Also copy language files
cp -r locales ~/.realconsole/
```

This allows you to run `realconsole` from **any directory** without being in the project folder.

### 4. Start RealConsole

```bash
./target/release/realconsole
```

You'll see the welcome screen:

```
RealConsole v1.52.0
Intelligent CLI Agent

Quick Start:
  /help       View quick help
  /examples   View usage examples
  Ctrl-D      Exit program

%
```

---

## Core Features

### 1. Intelligent Conversation (AI Assistant)

Enter questions directly without command prefix:

```bash
% Calculate 2 to the power of 10
AI: 2^10 = 1024
0.8s

% Write a hello world in Rust
AI: Here's a simple Rust Hello World program:

fn main() {
    println!("Hello, World!");
}

You can run it with `cargo run`.
1.2s
```

**Features**:
- Real-time LLM streaming output
- Automatic response time display
- Multi-turn conversation support (with memory)

### 2. Shell Command Execution

Use `!` prefix to execute system commands:

```bash
% !ls -la
total 128
drwxr-xr-x  15 user  staff   480 Oct 15 10:00 .
drwxr-xr-x   8 user  staff   256 Oct 14 18:30 ..
-rw-r--r--   1 user  staff  1234 Oct 15 09:45 README.md
...

% !pwd
/Users/user/realconsole
```

**Safety Protection**:
- Dangerous commands automatically blocked (e.g., `rm -rf /`)
- Timeout protection (default 10 seconds)
- Blacklist mechanism

### 3. Tool Calling (14 Built-in Tools)

RealConsole has 14 built-in utility tools that AI can automatically invoke:

```bash
% /tools list
Registered Tools (14):
  1. calculator - Math calculator
  2. datetime - Date and time tool
  3. file_read - Read file contents
  4. file_write - Write to file
  5. weather - Weather query
  6. search - Web search
  ...
```

**Manual tool invocation**:
```bash
% /tools call calculator {"expression": "sqrt(144)"}
Tool call successful
Result: 12

% /tools call datetime {"format": "RFC3339"}
Tool call successful
Current time: 2026-01-08T10:30:00+08:00
```

**AI automatic invocation**:
```bash
% Help me calculate the cube root of 125
AI: [Calling tool: calculator]
Parameters: {"expression": "125^(1/3)"}
Result: 5

The cube root of 125 is 5.
1.1s
```

### 4. Multi-level Help System

RealConsole provides comprehensive help information:

```bash
% /help           # Quick help (one screen)
% /help all       # Complete help (all commands)
% /help tools     # Tool management help
% /help memory    # Memory system help
% /help shell     # Shell execution help
% /examples       # Usage examples library
% /quickref       # Quick reference card
```

### 5. Conversation Context (Optional)

RealConsole supports **optional conversation context modes** for different scenarios:

#### Three Modes

**Disabled (Off, default)**:
- Single command execution, no context
- Fastest speed, lowest token consumption
- Suitable for: quick queries, script calls

**Manual**:
```bash
% /context start      # Start recording context
Context started

% Analyze errors in error.log
AI: Found 3 error types...

% Count each error type    # Automatically uses context
AI: Based on the analyzed error.log...
- TypeError: 15 times
- ValueError: 8 times
- RuntimeError: 2 times

% /context stop       # Stop and clear
Context stopped
Stats: Cleared 2 rounds of conversation (847 characters)
```

**Auto**:
- Intelligently detects when context is needed
- Automatically enables when references are detected (e.g., "they", "it", "those")
- Recommended for multi-turn conversation scenarios

#### Configuration Example

Edit `realconsole.yaml`:

```yaml
conversation:
  mode: manual              # disabled (default), manual, auto
  max_turns: 20            # Keep up to 20 rounds
  max_context_length: 8000 # Maximum 8000 characters
  auto_clear:
    enabled: true          # Enable auto-clear
    idle_timeout: 300      # Clear after 5 minutes of inactivity
```

### 6. Single Execution Mode

Execute a single command without starting REPL (suitable for scripts):

```bash
# Display help
./target/release/realconsole --once "/help"

# Call tool
./target/release/realconsole --once "/tools call calculator {\"expression\": \"2+2\"}"

# AI conversation
./target/release/realconsole --once "What is Rust"
```

---

## Common Commands

### Basic Commands

| Command | Description | Alias |
|---------|-------------|-------|
| `/help` | Quick help | `/h`, `/?` |
| `/help all` | Complete help | - |
| `/examples` | Usage examples | - |
| `/quickref` | Quick reference | - |
| `/version` | Show version | `/v` |
| `/quit` | Exit program | `/q`, `Ctrl-D` |

### Tool Management

| Command | Description |
|---------|-------------|
| `/tools list` | List all available tools |
| `/tools call <name> <json>` | Manually call a tool |
| `/tools schema <name>` | View tool's JSON Schema |

### Memory System

| Command | Description |
|---------|-------------|
| `/memory list` | List all memories |
| `/memory search <query>` | Search memories |
| `/memory clear` | Clear all memories |
| `/memory export` | Export memories to file |

### Execution Log

| Command | Description |
|---------|-------------|
| `/log show` | Show recent execution records |
| `/log export` | Export log to file |
| `/log clear` | Clear log |

### Context Management

| Command | Description |
|---------|-------------|
| `/context start` | Start recording context (Manual mode) |
| `/context stop` | Stop and clear context |
| `/context clear` | Clear context without stopping |
| `/context show` | View current context state |
| `/context status` | View configuration and statistics |

---

## Keyboard Shortcuts

| Shortcut | Function |
|----------|----------|
| `Ctrl-D` | Exit program |
| `Ctrl-C` | Interrupt current input |
| `Ctrl-L` | Clear screen |
| `Up/Down` | Navigate command history |
| `Ctrl-A` | Move cursor to line start |
| `Ctrl-E` | Move cursor to line end |
| `Ctrl-U` | Clear current line |

---

## Troubleshooting

### Common Issues

#### 1. Configuration File Not Found

**Error message**:
```
[E001] Configuration file not found: realconsole.yaml
```

**Solution**:
Run the configuration wizard to auto-generate:
```bash
./target/release/realconsole wizard --quick
```

#### 2. LLM API Key Error

**Error message**:
```
[E102] LLM authentication failed
```

**Solution**:
1. Check if `DEEPSEEK_API_KEY` in `.env` file is correct
2. Re-run wizard: `realconsole wizard --quick`

#### 3. Shell Command Timeout

**Error message**:
```
[E303] Command execution timed out (exceeded 10 seconds)
```

**Solution**:
Edit `realconsole.yaml`, increase timeout:
```yaml
features:
  shell_timeout: 30  # Increase to 30 seconds
```

### Debugging Tips

**View detailed logs**:
```bash
RUST_LOG=debug ./target/release/realconsole
```

**Test configuration file**:
```bash
./target/release/realconsole --config realconsole.yaml --once "/version"
```

---

## FAQ

### Q1: Does RealConsole require internet?

**A**: Depends on your LLM configuration:
- Using **Deepseek API**: Requires internet
- Using **Ollama local model**: No internet required (recommended for offline use)

### Q2: How to switch LLM providers?

**A**: Re-run the configuration wizard:
```bash
./target/release/realconsole wizard
```
Or manually edit `realconsole.yaml`.

### Q3: Which operating systems are supported?

**A**:
- macOS (Intel & Apple Silicon)
- Linux (x86_64 & ARM64)
- Windows (requires WSL or native build)

### Q4: RealConsole vs Python version?

**A**:
| Feature | Python Version | Rust Version (RealConsole) |
|---------|----------------|---------------------------|
| Startup time | ~300ms | ~10ms (30x faster) |
| Memory usage | ~80MB | ~8MB (10x optimized) |
| Binary size | N/A | ~15MB |
| Tool calling | Basic | 14 built-in tools + parallel execution |
| Error system | Simple | 30+ error codes + fix suggestions |

### Q5: How to add custom tools?

**A**: Refer to [Tool Development Guide](tool-development.md), implement the `Tool` trait:
```rust
use realconsole::Tool;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    async fn execute(&self, params: Value) -> Result<Value> { ... }
}
```

### Q6: Does it support multi-turn conversations?

**A**: Yes! RealConsole has a built-in memory system that automatically records conversation context:
```bash
% My name is John
AI: Hello, John!

% Do you remember my name?
AI: Of course, your name is John.
```

---

## Next Steps

Congratulations on completing the quick start! Next you can:

### Learn More

- **[User Guide](user-guide.md)** - Detailed documentation of all features
- **[Tool Calling Guide](tool-calling-guide.md)** - Complete documentation for 14 tools
- **[Intent DSL Guide](intent-dsl-guide.md)** - Custom intent recognition
- **[LLM Setup Guide](llm-setup.md)** - Advanced LLM configuration

### Development

- **[Developer Guide](../developer/developer-guide.md)** - Architecture and development environment
- **[Tool Development](../developer/tool-development.md)** - Create custom tools
- **[API Reference](../developer/api-reference.md)** - Core module APIs

### Contribute

- **[GitHub Repository](https://github.com/hongxin/RealConsole)** - View source code
- **[Issue Tracker](https://github.com/hongxin/RealConsole/issues)** - Report issues

---

**Version**: v1.52.0
**Updated**: 2026-01-08
