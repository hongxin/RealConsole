# Quick Start Guide

Get RealConsole up and running in 5 minutes.

## Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **LLM Provider** (choose one):
  - [Ollama](https://ollama.ai/) (local, recommended for privacy)
  - [Deepseek API](https://platform.deepseek.com/) (cloud, requires API key)
  - [OpenAI API](https://platform.openai.com/) (cloud, requires API key)

## Installation

### Option 1: Quick Install (Recommended)

```bash
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole
make install
```

This will:
- Build the release version
- Install to `~/.local/bin/realconsole`
- Make it available in your PATH

### Option 2: Build Only

```bash
git clone https://github.com/hongxin/RealConsole.git
cd RealConsole
cargo build --release
./target/release/realconsole
```

## Configuration

### Interactive Setup (Easiest)

Run the configuration wizard:

```bash
realconsole wizard --quick
```

The wizard will guide you through:
1. Choosing LLM provider (Ollama/Deepseek/OpenAI)
2. Setting up API keys (if needed)
3. Configuring basic features

### Manual Setup

<details>
<summary>Click to expand manual configuration steps</summary>

1. **Copy configuration files**:
   ```bash
   cp .env.example .env
   cp config/realconsole.yaml.example realconsole.yaml
   ```

2. **Edit `.env`** (if using cloud LLM):
   ```bash
   # For Deepseek
   DEEPSEEK_API_KEY=sk-your-key-here

   # For OpenAI
   OPENAI_API_KEY=sk-your-key-here
   ```

3. **Edit `realconsole.yaml`**:
   ```yaml
   llm:
     primary:
       provider: deepseek  # or ollama, openai
       model: deepseek-chat
       endpoint: https://api.deepseek.com/v1
       api_key: ${DEEPSEEK_API_KEY}

   features:
     shell_enabled: true
     memory_enabled: true
     tool_calling_enabled: true
     suggestion_enabled: true  # NEW: Proactive suggestions
   ```

</details>

## First Run

Start RealConsole:

```bash
realconsole
```

You should see:

```
RealConsole v1.8.0 | Type /help or Ctrl-D to exit
(RealConsole v1) you@hostname RealConsole %
```

## Try It Out

### 1. Chat with AI

```bash
% hello
Hello! I'm your AI assistant. How can I help you today?

% explain what is Rust
Rust is a systems programming language...
```

### 2. Execute Shell Commands

```bash
% !ls -la
% !git status
% !cargo build
```

### 3. Get Proactive Suggestions ✨ NEW

```bash
% /suggest
💡 Based on your context, here are some suggestions:
  1. cargo build - Build the project
  2. cargo test - Run tests
  3. git commit -m "..." - Commit changes

% 1           # Quick execute suggestion #1
```

### 4. Use Task Orchestration

```bash
% /plan create a Rust project with src and tests directories
🤖 AI is analyzing your goal...
✓ Decomposed into 6 subtasks

% /execute
✓ 6/6 tasks completed in 10s
```

### 5. View Unified Tracking

```bash
% /trace
📊 Unified Tracking - Last 20 records
[Shows commands, LLM calls, and system events]
```

## Common Commands

| Command | Description |
|---------|-------------|
| `/help` | Show help |
| `/suggest` | Get proactive suggestions |
| `/plan <goal>` | Create task plan |
| `/execute` | Execute planned tasks |
| `/trace` | View unified tracking |
| `/tools` | List available tools |
| `/quit` | Exit RealConsole |

## Troubleshooting

### LLM Connection Issues

**Ollama not responding?**
```bash
# Check Ollama is running
ollama serve

# Test connection
curl http://localhost:11434/api/tags
```

**Deepseek API errors?**
```bash
# Verify API key
echo $DEEPSEEK_API_KEY

# Check configuration
cat realconsole.yaml | grep api_key
```

### Permission Issues

```bash
# Make sure binary is executable
chmod +x ~/.local/bin/realconsole

# Or run from build directory
./target/release/realconsole
```

### Configuration Not Found

```bash
# Run wizard to create config
realconsole wizard --quick

# Or copy example files
cp .env.example .env
cp config/realconsole.yaml.example realconsole.yaml
```

## Next Steps

- **[User Guide](02-practice/user/user-guide.md)** - Complete feature documentation
- **[Examples](../examples/)** - More usage examples
- **[Configuration Guide](02-practice/user/configuration.md)** - Advanced configuration
- **[FAQ](02-practice/user/faq.md)** - Common questions

## Get Help

- **Issues**: [GitHub Issues](https://github.com/hongxin/RealConsole/issues)
- **Documentation**: [docs/README.md](README.md)
- **In-app help**: Type `/help` in RealConsole

---

**Next**: [Explore Examples](../examples/) | [Read User Guide](02-practice/user/user-guide.md)
