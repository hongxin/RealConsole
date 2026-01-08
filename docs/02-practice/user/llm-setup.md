# LLM 配置指南

中文 | **[English](llm-setup.en.md)**

## 快速开始

RealConsole 支持多种 LLM 提供商，包括本地 Ollama 和远程 API（Deepseek、OpenAI）。

### 1. 配置文件位置

默认配置文件：`realconsole.yaml`

指定配置文件：
```bash
realconsole --config my-config.yaml
```

### 2. 配置结构

```yaml
llm:
  primary:     # 主 LLM（通常是远程 API）
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  fallback:    # 备用 LLM（通常是本地 Ollama）
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

### 3. 环境变量

配置支持环境变量替换：

```bash
# 设置环境变量
export DEEPSEEK_API_KEY="sk-your-api-key"

# 在配置中使用
api_key: ${DEEPSEEK_API_KEY}

# 带默认值
endpoint: ${OLLAMA_ENDPOINT:-http://localhost:11434}
```

## 支持的 LLM 提供商

### 1. Ollama（本地）

**优势：**
- 完全本地运行，隐私安全
- 无需 API key
- 响应速度快
- 支持多种开源模型

**安装 Ollama：**
```bash
# macOS/Linux
curl https://ollama.ai/install.sh | sh

# 或访问 https://ollama.com 下载
```

**启动 Ollama 服务：**
```bash
ollama serve
```

**拉取模型：**
```bash
ollama pull qwen3:4b      # 推荐：轻量级，速度快
ollama pull qwen3:8b      # 平衡性能和速度
ollama pull gemma3:27b    # 高性能
ollama pull deepseek-r1:8b  # 推理优化
```

**配置示例：**
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

### 2. Deepseek（远程）

**优势：**
- 高性能模型
- 成本效益好
- 支持长上下文

**获取 API Key：**
1. 访问 https://platform.deepseek.com
2. 注册账号
3. 获取 API key

**配置示例：**
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

**环境变量：**
```bash
export DEEPSEEK_API_KEY="sk-xxxxxxxxxxxx"
```

### 3. OpenAI（即将支持）

```yaml
llm:
  primary:
    provider: openai
    model: gpt-4
    endpoint: https://api.openai.com/v1
    api_key: ${OPENAI_API_KEY}
```

## 配置示例

### 场景 1：仅使用本地 Ollama

```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: http://localhost:11434
```

**使用：**
```bash
# /ask 会使用 fallback LLM
realconsole
> /ask 你好
```

### 场景 2：Primary + Fallback（推荐）

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

**使用逻辑：**
- `/ask` 优先使用 fallback（快速响应）
- Primary 用于未来的高级功能

### 场景 3：仅使用远程 API

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

## 命令使用

### 查看 LLM 状态

```bash
$ realconsole --config realconsole.yaml --once "/llm"
✓ Fallback LLM: qwen3:4b (ollama)
LLM 状态:
  Primary: (未配置)
  Fallback: qwen3:4b

提示: /llm diag <primary|fallback> 诊断连接
```

### 诊断连接

```bash
$ realconsole --config realconsole.yaml --once "/llm diag fallback"
Fallback LLM 诊断:
端点: http://localhost:11434
模型: qwen3:4b
✓ 连接成功
可用模型数: 4
模型: qwen3:4b, qwen3:8b, gemma3:27b, deepseek-r1:8b
```

### 向 LLM 提问

```bash
$ realconsole --config realconsole.yaml
> /ask 你好，请介绍一下自己
你好！我是一个AI助手，基于大语言模型...

> /ask 什么是 Rust 语言？
Rust 是一种系统编程语言...
```

## 常见问题

### Q1: Ollama 连接失败（502 错误）

**原因：** Ollama 服务未启动

**解决：**
```bash
# 启动 Ollama
ollama serve

# 或检查是否已运行
ps aux | grep ollama
```

### Q2: 找不到模型

**原因：** 模型未下载

**解决：**
```bash
# 查看已安装模型
ollama list

# 下载模型
ollama pull qwen3:4b
```

### Q3: Deepseek API key 无效

**解决：**
1. 检查 API key 是否正确
2. 确认环境变量已设置：`echo $DEEPSEEK_API_KEY`
3. 重新启动 RealConsole

### Q4: 如何切换模型？

修改配置文件中的 `model` 字段：
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:8b  # 改为更大的模型
```

### Q5: 如何使用代理？

设置环境变量：
```bash
export HTTPS_PROXY=http://127.0.0.1:7890
export HTTP_PROXY=http://127.0.0.1:7890
```

## 性能建议

### 模型选择

| 模型 | 大小 | 速度 | 质量 | 适用场景 |
|------|------|------|------|----------|
| qwen3:4b | 小 | ⭐⭐⭐ | ⭐⭐ | 快速响应、简单任务 |
| qwen3:8b | 中 | ⭐⭐ | ⭐⭐⭐ | 平衡性能和速度 |
| gemma3:27b | 大 | ⭐ | ⭐⭐⭐⭐ | 复杂推理、高质量输出 |
| deepseek-r1:8b | 中 | ⭐⭐ | ⭐⭐⭐⭐ | 代码生成、逻辑推理 |

### 配置建议

**开发环境：**
```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b  # 快速迭代
```

**生产环境：**
```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat  # 高质量输出
  fallback:
    provider: ollama
    model: qwen3:8b  # 备用方案
```

## 完整配置示例

```yaml
# RealConsole 配置文件
prefix: "/"

llm:
  # Primary LLM - Deepseek API
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  # Fallback LLM - 本地 Ollama
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: ${OLLAMA_ENDPOINT:-http://localhost:11434}

features:
  shell_enabled: true
  shell_timeout: 10
```

## 安全建议

1. **不要将 API key 直接写在配置文件中**
   - ✅ 使用环境变量：`api_key: ${DEEPSEEK_API_KEY}`
   - ❌ 避免：`api_key: sk-xxxxx`

2. **使用 .env 文件**
   ```bash
   # .env
   DEEPSEEK_API_KEY=sk-xxxxx
   ```

3. **添加到 .gitignore**
   ```
   .env
   realconsole.yaml  # 如果包含敏感信息
   ```

## 进阶配置

### 自定义端点

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://my-proxy.example.com/v1  # 自定义代理
    api_key: ${DEEPSEEK_API_KEY}
```

### 多个 Ollama 实例

```yaml
llm:
  fallback:
    provider: ollama
    model: qwen3:4b
    endpoint: ${OLLAMA_ENDPOINT:-http://192.168.1.100:11434}  # 远程 Ollama
```

## 故障排除

### 启用调试日志

```bash
# 查看详细错误信息
RUST_LOG=debug realconsole --config realconsole.yaml
```

### 测试连接

```bash
# 测试 Ollama
curl http://localhost:11434/api/tags

# 测试 Deepseek
curl https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY"
```

## 总结

RealConsole 的 LLM 配置非常灵活：

- ✅ 支持多种提供商（Ollama、Deepseek）
- ✅ 环境变量替换（安全）
- ✅ Primary/Fallback 架构（可靠）
- ✅ 实时诊断（/llm diag）
- ✅ 即插即用（零代码配置）

开始使用：
```bash
# 1. 创建配置文件
cp realconsole.yaml my-config.yaml

# 2. 编辑配置
vim my-config.yaml

# 3. 启动
realconsole --config my-config.yaml
```

祝使用愉快！🚀
