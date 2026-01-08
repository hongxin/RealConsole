# RealConsole Developer Guide

**[中文](developer-guide.md)** | English

**Version**: v1.52.0
**Updated**: 2026-01-08
**Audience**: RealConsole contributors and extension developers

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Project Architecture](#project-architecture)
3. [Code Structure](#code-structure)
4. [Development Environment](#development-environment)
5. [Building and Testing](#building-and-testing)
6. [Code Standards](#code-standards)
7. [Contribution Guide](#contribution-guide)
8. [Core Modules](#core-modules)
9. [Extension Development](#extension-development)

---

## Quick Start

### Clone Project

```bash
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole
```

### Install Dependencies

Ensure Rust toolchain (1.70.0+) is installed:

```bash
# Check Rust version
rustc --version
cargo --version

# If not installed, visit https://rustup.rs/
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build Project

```bash
# Debug mode (for development)
cargo build

# Release mode (for production)
cargo build --release
```

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_intent_matching

# Show test output
cargo test -- --nocapture
```

### Run Program

```bash
# Debug mode
cargo run

# Release mode
./target/release/realconsole
```

---

## Project Architecture

### Architecture Overview

RealConsole uses a modular architecture centered around the `Agent`:

```
┌──────────────────────────────────────────────────────┐
│                     RealConsole                       │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │              Agent (Core Scheduler)              │ │
│  │                                                  │ │
│  │  ┌──────────────┐  ┌────────────┐              │ │
│  │  │ LLM Manager  │  │ Tool       │              │ │
│  │  │ - Primary    │  │ Registry   │              │ │
│  │  │ - Fallback   │  │ (14 tools) │              │ │
│  │  └──────────────┘  └────────────┘              │ │
│  │                                                  │ │
│  │  ┌──────────────┐  ┌────────────┐              │ │
│  │  │ Intent       │  │ Memory     │              │ │
│  │  │ Matcher      │  │ System     │              │ │
│  │  │ (50+ intents)│  │            │              │ │
│  │  └──────────────┘  └────────────┘              │ │
│  │                                                  │ │
│  │  ┌──────────────┐  ┌────────────┐              │ │
│  │  │ Shell        │  │ Execution  │              │ │
│  │  │ Executor     │  │ Logger     │              │ │
│  │  └──────────────┘  └────────────┘              │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │           System Commands (Core Commands)        │ │
│  │  /help  /tools  /memory  /log  /version  ...    │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │              CLI Interface (REPL)                │ │
│  │            - rustyline editor                    │ │
│  │            - History support                     │ │
│  │            - Auto-completion                     │ │
│  └─────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Agent (src/agent.rs)

**Responsibilities**:
- Receive user input and dispatch to different processing paths
- Manage all subsystems (LLM, tools, memory, logs)
- Coordinate tool calls and multi-turn conversations

**Key Methods**:
- `handle(&mut self, input: &str) -> String` - Unified entry point
- `handle_text(&self, text: &str) -> String` - Handle AI conversations
- `handle_shell(&self, cmd: &str) -> String` - Handle Shell commands
- `handle_system_command(&self, cmd: &str) -> String` - Handle system commands

#### 2. LLM Manager (src/llm/)

**Responsibilities**:
- Manage Primary + Fallback LLM clients
- Unified LLM call interface (supports streaming)
- Automatic failover

**Key Modules**:
- `llm_manager.rs` - LLM manager
- `deepseek.rs` - Deepseek API client
- `ollama.rs` - Ollama local client
- `trait LlmClient` - Unified LLM interface

#### 3. Tool System (src/tool*.rs)

**Responsibilities**:
- Tool registration and management
- Tool execution (supports parallel execution)
- OpenAI Function Calling Schema generation

**Key Modules**:
- `tool_registry.rs` - Tool registration center
- `tool_executor.rs` - Tool execution engine
- `builtin_tools.rs` - 14 built-in tools
- `advanced_tools.rs` - Advanced tools (HTTP, encoding, etc.)

#### 4. Intent DSL (src/dsl/intent/)

**Responsibilities**:
- Natural language intent recognition
- Entity extraction and parameter parsing
- LRU cache optimization

#### 5. Memory System (src/memory.rs)

**Responsibilities**:
- Short-term memory (ring buffer)
- Long-term memory (persistence)
- Memory search and management

#### 6. Web Terminal (src/web/)

**Responsibilities**:
- Browser-based terminal access
- WebSocket real-time communication
- Data visualization (ECharts)

---

## Code Structure

```
realconsole/
├── src/
│   ├── main.rs                   # CLI entry
│   ├── lib.rs                    # Library entry
│   ├── agent.rs                  # Core Agent
│   ├── config.rs                 # Configuration management
│   ├── error.rs                  # Error system (30+ codes)
│   │
│   ├── llm/
│   │   ├── mod.rs                # LLM module entry
│   │   ├── llm_manager.rs        # LLM manager
│   │   ├── deepseek.rs           # Deepseek client
│   │   └── ollama.rs             # Ollama client
│   │
│   ├── dsl/
│   │   └── intent/
│   │       ├── mod.rs            # Intent module entry
│   │       ├── matcher.rs        # Intent matching engine
│   │       ├── builtin.rs        # Built-in intents
│   │       └── template.rs       # Template engine
│   │
│   ├── web/
│   │   ├── mod.rs                # Web module entry
│   │   ├── server.rs             # HTTP server
│   │   ├── websocket.rs          # WebSocket handler
│   │   └── frontend.rs           # Frontend assets
│   │
│   ├── visualization/
│   │   ├── mod.rs                # Visualization module
│   │   ├── types.rs              # Chart types
│   │   └── parser.rs             # Command parser
│   │
│   ├── tool_registry.rs          # Tool registration
│   ├── tool_executor.rs          # Tool execution engine
│   ├── builtin_tools.rs          # Built-in tools
│   ├── memory.rs                 # Memory system
│   ├── shell_executor.rs         # Shell executor
│   └── i18n.rs                   # Internationalization
│
├── tests/                        # Integration tests
├── docs/                         # Documentation
├── config/                       # Configuration examples
├── locales/                      # Language files
├── Cargo.toml                    # Project dependencies
└── CLAUDE.md                     # Claude Code project guide
```

---

## Development Environment

### Required Tools

- **Rust**: 1.70.0 or higher (recommend using rustup)
- **Cargo**: Rust package manager (installed with Rust)
- **Git**: Version control

### Recommended Tools

- **IDE**:
  - VSCode + rust-analyzer extension
  - IntelliJ IDEA + Rust plugin
  - Vim/Neovim + rust.vim

- **Code Formatting**:
  - `rustfmt` - Auto formatting
  - `clippy` - Static analysis

- **Debug Tools**:
  - `lldb` (macOS) / `gdb` (Linux)
  - `cargo-watch` - Auto recompile
  - `cargo-tree` - View dependency tree

### VSCode Configuration

`.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer"
}
```

### Environment Variables

Recommended for development:

```bash
# Enable Rust backtrace
export RUST_BACKTRACE=1

# Enable detailed logs
export RUST_LOG=debug

# Speed up compilation (optional)
export CARGO_INCREMENTAL=1
```

---

## Building and Testing

### Build

```bash
# Debug mode (includes debug symbols, unoptimized)
cargo build

# Release mode (fully optimized, for production)
cargo build --release

# Check for compilation errors (fast, no binary output)
cargo check

# Clean build artifacts
cargo clean
```

### Test

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test intent

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Code Quality

```bash
# Format code
cargo fmt

# Static analysis
cargo clippy

# Zero warnings check
cargo clippy -- -D warnings
```

---

## Code Standards

### Naming Conventions

- **Files**: snake_case (e.g., `tool_registry.rs`)
- **Functions/Variables**: snake_case (e.g., `get_user_input`)
- **Types/Structs**: CamelCase (e.g., `ToolRegistry`)
- **Constants**: SCREAMING_SNAKE_CASE (e.g., `MAX_RETRIES`)

### Error Handling

```rust
use anyhow::Result;
use thiserror::Error;

// Define custom errors
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    NotFound(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}

// Use Result type
fn execute_tool(name: &str) -> Result<String> {
    // ...
}
```

### Async Patterns

```rust
use tokio;

// LLM calls must be async
async fn call_llm(prompt: &str) -> Result<String> {
    // ...
}

// Use streaming for real-time output
async fn stream_response(&self) -> impl Stream<Item = String> {
    // ...
}
```

---

## Contribution Guide

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make changes and add tests
4. Run tests: `cargo test`
5. Run formatter: `cargo fmt`
6. Run linter: `cargo clippy`
7. Commit with clear message
8. Submit Pull Request

### Commit Message Format

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(tool): add HTTP request tool`
- `fix(llm): resolve timeout issue`
- `docs(guide): update developer guide`

### Code Review Checklist

- [ ] Tests pass
- [ ] No clippy warnings
- [ ] Code formatted
- [ ] Documentation updated
- [ ] Commit messages clear

---

## Core Modules

### Adding a New Tool

Implement the `Tool` trait in `src/builtin_tools.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;
use anyhow::Result;

pub struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }

    fn description(&self) -> &str {
        "Description of my tool"
    }

    fn parameters_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "First parameter"
                }
            },
            "required": ["param1"]
        })
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let param1 = params["param1"].as_str().unwrap_or("");
        // Tool logic here
        Ok(serde_json::json!({
            "result": format!("Processed: {}", param1)
        }))
    }
}
```

Register the tool in `create_default_registry()`:

```rust
registry.register(Box::new(MyTool));
```

### Adding a New Intent

Add to `src/dsl/intent/builtin.rs`:

```rust
intents.push(Intent {
    name: "my_intent".to_string(),
    patterns: vec![
        r"my pattern (?P<entity>\w+)".to_string(),
    ],
    response_template: "Response with {entity}".to_string(),
    priority: 50,
});
```

### Adding a New LLM Provider

Implement `LlmClient` trait in `src/llm/`:

```rust
#[async_trait]
impl LlmClient for MyProvider {
    async fn chat(&self, messages: Vec<Message>) -> Result<String>;
    async fn chat_stream(&self, messages: Vec<Message>) -> Result<impl Stream<Item = String>>;
}
```

---

## Extension Development

### Custom Tool Development

See [Tool Development Guide](tool-development.md) for detailed instructions.

### Plugin Architecture (Planned)

Future versions will support:
- Dynamic plugin loading
- Custom command registration
- Event hooks

---

## Debugging Tips

### Enable Debug Logs

```bash
RUST_LOG=debug cargo run
```

### Test Configuration

```bash
realconsole --config test-config.yaml --once "/version"
```

### Common Issues

1. **LLM Connection Failed**: Check API key and network
2. **Tool Not Found**: Verify tool registration
3. **Build Errors**: Run `cargo clean` and rebuild

---

## Related Resources

- [Quick Start](../user/quickstart.en.md)
- [LLM Setup Guide](../user/llm-setup.en.md)
- [Tool Development](tool-development.md)
- [API Reference](api-reference.md)

---

**Version**: v1.52.0
**Updated**: 2026-01-08
