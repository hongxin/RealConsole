# LLM Configuration Guide

**[中文](llm-setup.md)** | English

## Quick Start

RealConsole supports multiple LLM providers, including local Ollama and remote APIs (Deepseek, OpenAI, Gemini).

### 1. Configuration File Location

Default configuration file: `realconsole.yaml`

Specify configuration file:
```bash
realconsole --config my-config.yaml
```

### 2. Configuration Structure

```yaml
llm:
  primary:     # Primary LLM (usually remote API)
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  fallback:    # Fallback LLM (usually local Ollama)
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

### 3. Environment Variables

Configuration supports environment variable substitution:

```bash
# Set environment variable
export DEEPSEEK_API_KEY="sk-your-api-key"

# Use in configuration
api_key: ${DEEPSEEK_API_KEY}

# With default value
endpoint: ${OLLAMA_ENDPOINT:-http://localhost:11434}
```

## Supported LLM Providers

### 1. Ollama (Local)

**Advantages:**
- Runs completely locally, privacy-safe
- No API key required
- Fast response times
- Supports various open-source models

**Install Ollama:**
```bash
# macOS/Linux
curl https://ollama.ai/install.sh | sh

# Or visit https://ollama.com to download
```

**Start Ollama service:**
```bash
ollama serve
```

**Pull models:**
```bash
ollama pull qwen3:4b      # Recommended: lightweight, fast
ollama pull qwen3:8b      # Balance performance and speed
ollama pull gemma3:27b    # High performance
ollama pull deepseek-r1:8b  # Reasoning optimized
```

**Configuration example:**
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

### 2. Deepseek (Remote)

**Advantages:**
- High-performance models
- Cost-effective
- Supports long context

**Get API Key:**
1. Visit https://platform.deepseek.com
2. Register an account
3. Get API key

**Configuration example:**
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

**Environment variable:**
```bash
export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxx"
```

### 3. OpenAI

```yaml
llm:
  primary:
    provider: openai
    model: gpt-4
    endpoint: https://api.openai.com/v1
    api_key: ${OPENAI_API_KEY}
```

### 4. Google Gemini

```yaml
llm:
  primary:
    provider: gemini
    model: gemini-pro
    api_key: ${GEMINI_API_KEY}
```

## Configuration Examples

### Scenario 1: Local Ollama Only

```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

**Usage:**
```bash
realconsole
> Hello, introduce yourself
```

### Scenario 2: Primary + Fallback (Recommended)

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

**Usage logic:**
- Conversation uses primary when available
- Falls back to local Ollama if primary fails

### Scenario 3: Remote API Only

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

## Command Usage

### Check LLM Status

```bash
$ realconsole --once "/llm"
Fallback LLM: qwen3:4b (ollama)
LLM Status:
  Primary: (not configured)
  Fallback: qwen3:4b

Tip: /llm diag <primary|fallback> to diagnose connection
```

### Diagnose Connection

```bash
$ realconsole --once "/llm diag fallback"
Fallback LLM Diagnostics:
Endpoint: http://localhost:11434
Model: qwen3:4b
Connection successful
Available models: 4
Models: qwen3:4b, qwen3:8b, gemma3:27b, deepseek-r1:8b
```

## FAQ

### Q1: Ollama Connection Failed (502 Error)

**Cause:** Ollama service not started

**Solution:**
```bash
# Start Ollama
ollama serve

# Or check if running
ps aux | grep ollama
```

### Q2: Model Not Found

**Cause:** Model not downloaded

**Solution:**
```bash
# View installed models
ollama list

# Download model
ollama pull qwen3:4b
```

### Q3: Deepseek API Key Invalid

**Solution:**
1. Check if API key is correct
2. Confirm environment variable is set: `echo $DEEPSEEK_API_KEY`
3. Restart RealConsole

### Q4: How to Switch Models?

Modify the `model` field in configuration:
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:8b  # Switch to larger model
```

### Q5: How to Use Proxy?

Set environment variables:
```bash
export HTTPS_PROXY=http://127.0.0.1:7890
export HTTP_PROXY=http://127.0.0.1:7890
```

## Performance Recommendations

### Model Selection

| Model | Size | Speed | Quality | Use Case |
|-------|------|-------|---------|----------|
| qwen3:4b | Small | Fast | Good | Quick responses, simple tasks |
| qwen3:8b | Medium | Medium | Better | Balance performance and speed |
| gemma3:27b | Large | Slow | Excellent | Complex reasoning, high-quality output |
| deepseek-r1:8b | Medium | Medium | Excellent | Code generation, logical reasoning |

### Configuration Recommendations

**Development environment:**
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b  # Fast iteration
```

**Production environment:**
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat  # High-quality output
  fallback:
    provider: ollama
    model: qwen3:8b  # Backup option
```

## Complete Configuration Example

```yaml
# RealConsole Configuration File
prefix: "/"

llm:
  # Primary LLM - Deepseek API
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  # Fallback LLM - Local Ollama
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: ${OLLAMA_ENDPOINT:-http://localhost:11434}

features:
  shell_enabled: true
  shell_timeout: 10
```

## Security Recommendations

1. **Never write API keys directly in configuration files**
   - Use environment variables: `api_key: ${DEEPSEEK_API_KEY}`
   - Avoid: `api_key: sk-xxxxx`

2. **Use .env file**
   ```bash
   # .env
   DEEPSEEK_API_KEY=sk-xxxxx
   ```

3. **Add to .gitignore**
   ```
   .env
   realconsole.yaml  # If contains sensitive info
   ```

## Troubleshooting

### Enable Debug Logs

```bash
# View detailed error information
RUST_LOG=debug realconsole --config realconsole.yaml
```

### Test Connection

```bash
# Test Ollama
curl http://localhost:11434/api/tags

# Test Deepseek
curl https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY"
```

## Summary

RealConsole's LLM configuration is highly flexible:

- Supports multiple providers (Ollama, Deepseek, OpenAI, Gemini)
- Environment variable substitution (secure)
- Primary/Fallback architecture (reliable)
- Real-time diagnostics (/llm diag)
- Plug-and-play (zero-code configuration)

Get started:
```bash
# 1. Create configuration file
cp realconsole.yaml my-config.yaml

# 2. Edit configuration
vim my-config.yaml

# 3. Start
realconsole --config my-config.yaml
```

---

**Version**: v1.52.0
**Updated**: 2026-01-08
