# RealConsole 用户手册

**版本**: v0.5.0
**更新日期**: 2025-10-15
**适用对象**: 所有 RealConsole 用户

---

## 目录

1. [概述](#概述)
2. [安装与配置](#安装与配置)
3. [基础使用](#基础使用)
4. [高级功能](#高级功能)
5. [配置详解](#配置详解)
6. [故障排除](#故障排除)
7. [最佳实践](#最佳实践)
8. [附录](#附录)

---

## 概述

### 什么是 RealConsole？

**RealConsole** 是一个融合东方哲学智慧的智能命令行助手（CLI Agent），使用 Rust 构建，具备以下核心能力：

- 🤖 **智能对话**: 基于 LLM 的自然语言交互
- ⚡ **工具调用**: 14 个内置工具，支持并行执行
- 🐚 **Shell 执行**: 安全的系统命令执行
- 🧠 **记忆系统**: 短期+长期记忆，支持上下文对话
- 📝 **执行日志**: 完整的操作记录与审计
- 🎯 **Intent DSL**: 智能意图识别与匹配
- ⚠️ **错误系统**: 30+ 错误代码，友好的修复建议

### 核心特性

| 特性 | 说明 |
|------|------|
| **高性能** | 启动速度 ~10ms（相比 Python 版本提升 30 倍） |
| **低内存** | 运行时占用 ~8MB（优化 10 倍） |
| **安全性** | Shell 命令黑名单、超时保护、危险操作拦截 |
| **可扩展** | 易于添加自定义工具和意图 |
| **跨平台** | macOS、Linux、Windows (WSL) |

### 设计哲学

RealConsole 遵循"一分为三"的设计理念，将易经变化智慧融入系统架构。不是简单的二元对立（Safe/Dangerous），而是多维度的状态演化（Safe, NeedsConfirmation, Dangerous + confidence, risk, user_level 等向量）。

详见：[PHILOSOPHY.md](design/PHILOSOPHY.md)

---

## 安装与配置

### 系统要求

- **操作系统**: macOS 10.15+, Linux (Kernel 4.4+), Windows 10+ (WSL2)
- **Rust 工具链**: 1.70.0 或更高版本
- **内存**: 最低 256MB，推荐 512MB+
- **磁盘**: 约 100MB（包含构建产物）

### 安装步骤

#### 1. 克隆仓库

```bash
git clone https://github.com/your-repo/realconsole.git
cd realconsole
```

#### 2. 构建项目

```bash
cargo build --release
```

构建时间约 2-3 分钟，生成的二进制文件位于 `target/release/realconsole`。

#### 3. 可选：安装到系统路径

```bash
# macOS/Linux
sudo cp target/release/realconsole /usr/local/bin/

# 验证安装
realconsole --version
```

### 配置向导

RealConsole 提供了交互式配置向导，帮助你快速完成初始化。

#### 完整模式（推荐高级用户）

```bash
realconsole wizard
```

完整模式会询问所有配置选项：
- LLM 提供商（Deepseek API / Ollama）
- API Key 或本地模型配置
- Shell 执行设置（超时、黑名单）
- 记忆系统配置（大小、持久化路径）
- 执行日志配置
- 高级功能开关

#### 快速模式（推荐新用户）

```bash
realconsole wizard --quick
```

快速模式只询问必要配置：
- LLM 提供商选择
- API Key（如果需要）
- 基础功能开关

**示例流程**：

```
🧙 RealConsole 配置向导（快速模式）

━━━ 步骤 1/3: LLM 提供商 ━━━

选择 LLM 提供商:
1. Deepseek API（推荐，云端服务，需要 API Key）
2. Ollama（本地运行，无需 API Key）

选择 (1-2): 1

━━━ 步骤 2/3: API 配置 ━━━

请输入 Deepseek API Key: sk-xxxxxxxxxxxxxxxx
✓ API Key 已保存

━━━ 步骤 3/3: 功能设置 ━━━

启用 Shell 执行？(y/n) [默认: y]: y
启用记忆系统？(y/n) [默认: y]: y
启用执行日志？(y/n) [默认: y]: y

━━━ 配置完成 ━━━

✅ 配置文件已生成: /Users/user/realconsole.yaml
✅ 环境变量文件已创建: /Users/user/.env

现在可以运行:
  realconsole
```

### 手动配置

如果你喜欢手动配置，可以创建 `realconsole.yaml` 文件：

**最小配置示例**：

```yaml
# realconsole.yaml
prefix: "/"

llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

features:
  shell_enabled: true
  shell_timeout: 10
  memory_enabled: true
  memory_max_size: 100
  log_enabled: true
```

**环境变量配置（.env）**：

```bash
# .env
DEEPSEEK_API_KEY=sk-your-api-key-here
```

### LLM 提供商配置

#### 选项 A: Deepseek API（云端）

**优点**：
- ✅ 无需本地部署
- ✅ 模型质量高
- ✅ 响应速度快

**配置**：

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

`.env` 文件：
```bash
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxx
```

**获取 API Key**：访问 [Deepseek 官网](https://platform.deepseek.com/)

#### 选项 B: Ollama（本地）

**优点**：
- ✅ 完全离线运行
- ✅ 无需 API Key
- ✅ 数据隐私保护

**安装 Ollama**：

```bash
# macOS/Linux
curl https://ollama.ai/install.sh | sh

# 启动服务
ollama serve

# 拉取模型（推荐 qwen2.5）
ollama pull qwen2.5:7b
```

**配置**：

```yaml
llm:
  primary:
    provider: ollama
    model: qwen2.5:7b
    endpoint: http://localhost:11434
```

#### 双 LLM 模式（高可用）

你可以配置 primary + fallback 两个 LLM，当主 LLM 失败时自动切换到备用 LLM：

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}

  fallback:
    provider: ollama
    model: qwen2.5:7b
    endpoint: http://localhost:11434
```

### 验证配置

测试 LLM 连接：

```bash
realconsole --once "你好"
```

如果看到 AI 的回复，说明配置成功！

---

## 基础使用

### 启动 RealConsole

```bash
# 使用默认配置
realconsole

# 使用指定配置文件
realconsole --config /path/to/config.yaml

# 单次执行模式
realconsole --once "你好"
```

### 交互界面

启动后你会看到 REPL 界面：

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
     RealConsole v0.5.0
     智能 CLI Agent
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

💡 快速入门:
  /help       查看快速帮助
  /examples   查看使用示例
  Ctrl-D      退出程序

»
```

### 三种输入模式

RealConsole 支持三种输入模式，根据前缀自动识别：

| 前缀 | 模式 | 示例 |
|------|------|------|
| 无前缀 | 智能对话 | `计算 2 的 10 次方` |
| `/` | 系统命令 | `/help`, `/tools list` |
| `!` | Shell 执行 | `!ls -la`, `!pwd` |

#### 1. 智能对话模式

直接输入问题，无需前缀：

```bash
» 你好
🤖 AI: 你好！我是 RealConsole AI 助手，有什么可以帮助你的吗？
ⓘ 0.5s

» 计算 2 的 10 次方
🤖 AI: [调用工具: calculator]
参数: {"expression": "2^10"}
结果: 1024

2 的 10 次方等于 1024。
ⓘ 0.8s

» 用 Rust 写一个 hello world
🤖 AI: 这是一个简单的 Rust Hello World 程序：

```rust
fn main() {
    println!("Hello, World!");
}
```

你可以通过以下步骤运行：
1. 创建新项目: `cargo new hello_world`
2. 编辑 `src/main.rs`，粘贴上面的代码
3. 运行: `cargo run`

ⓘ 1.2s
```

**特性**：
- ✨ **流式输出**: 实时显示 LLM 生成的文本
- ⏱️ **响应时间**: 自动显示每次对话的耗时
- 🧠 **上下文记忆**: 自动记录对话历史（可配置）
- 🔧 **自动工具调用**: AI 可自动调用工具完成任务

#### 2. 系统命令模式

使用 `/` 前缀执行 RealConsole 内置命令：

```bash
» /help
💬 RealConsole v0.5.0

智能对话:
  直接输入问题即可，无需命令前缀
  示例: 计算 2 的 10 次方

快速命令:
  /help       显示此帮助
  /help all   显示所有命令（详细）
  /examples   查看使用示例
  /quickref   快速参考卡片
  /quit       退出程序
...

» /version
RealConsole v0.5.0
```

**常用系统命令**：

| 命令 | 说明 | 别名 |
|------|------|------|
| `/help` | 快速帮助 | `/h`, `/?` |
| `/help all` | 完整帮助 | - |
| `/help tools` | 工具管理帮助 | - |
| `/help memory` | 记忆系统帮助 | - |
| `/help shell` | Shell 执行帮助 | - |
| `/examples` | 使用示例库 | - |
| `/quickref` | 快速参考卡片 | - |
| `/version` | 显示版本 | `/v` |
| `/quit` | 退出程序 | `/q`, `Ctrl-D` |

#### 3. Shell 执行模式

使用 `!` 前缀执行系统命令：

```bash
» !ls -la
total 128
drwxr-xr-x  15 user  staff   480 Oct 15 10:00 .
drwxr-xr-x   8 user  staff   256 Oct 14 18:30 ..
-rw-r--r--   1 user  staff  1234 Oct 15 09:45 README.md
...

» !pwd
/Users/user/realconsole

» !echo "Hello from shell"
Hello from shell
```

**安全机制**：

RealConsole 内置多重安全保护：

1. **黑名单机制**: 自动拦截危险命令
2. **超时保护**: 默认 10 秒超时（可配置）
3. **空命令检测**: 拒绝执行空命令

**危险命令示例**：

```bash
» !rm -rf /
[E302] 命令包含危险操作，已被安全策略阻止

💡 修复建议:
1. 此命令可能造成系统损坏，建议使用更安全的替代方案
2. 查看允许的命令列表和安全策略
   📖 https://docs.realconsole.com/shell-safety

» !dd if=/dev/zero of=/dev/sda
[E302] 命令包含危险操作，已被安全策略阻止
...
```

**超时示例**：

```bash
» !sleep 20
[E303] 命令执行超时（超过 10 秒）

💡 修复建议:
1. 命令执行时间过长，请检查命令或增加超时时间
2. 在配置文件中调整 features.shell_timeout
   💻 vi realconsole.yaml
```

---

## 高级功能

### 工具调用系统

RealConsole 内置了 14 个实用工具，AI 可以自动调用它们完成任务。

#### 查看可用工具

```bash
» /tools list
📦 已注册工具 (14):
  1. calculator - 数学计算器（支持四则运算、幂、根号等）
  2. datetime - 时间日期工具（当前时间、格式化、时区转换）
  3. file_read - 读取文件内容
  4. file_write - 写入文件
  5. file_list - 列出目录内容
  6. weather - 天气查询（城市天气信息）
  7. search - 网络搜索（模拟搜索引擎）
  8. http_get - HTTP GET 请求
  9. http_post - HTTP POST 请求
  10. json_parse - JSON 解析与格式化
  11. base64_encode - Base64 编码
  12. base64_decode - Base64 解码
  13. hash - 哈希计算（MD5、SHA256）
  14. uuid - UUID 生成
```

#### 手动调用工具

格式：`/tools call <工具名> <JSON参数>`

**示例 1: 计算器**

```bash
» /tools call calculator {"expression": "sqrt(144) + 2^3"}
✓ 工具调用成功
结果: 20

» /tools call calculator {"expression": "sin(3.14159/2)"}
✓ 工具调用成功
结果: 0.9999999999964793
```

**示例 2: 时间工具**

```bash
» /tools call datetime {"format": "RFC3339"}
✓ 工具调用成功
当前时间: 2025-10-15T10:30:00+08:00

» /tools call datetime {"format": "timestamp"}
✓ 工具调用成功
当前时间: 1729045800
```

**示例 3: 文件操作**

```bash
» /tools call file_write {"path": "test.txt", "content": "Hello RealConsole"}
✓ 工具调用成功
已写入 test.txt

» /tools call file_read {"path": "test.txt"}
✓ 工具调用成功
内容: Hello RealConsole

» /tools call file_list {"path": "."}
✓ 工具调用成功
文件列表:
  test.txt
  realconsole.yaml
  ...
```

**示例 4: 编解码工具**

```bash
» /tools call base64_encode {"text": "Hello World"}
✓ 工具调用成功
结果: SGVsbG8gV29ybGQ=

» /tools call base64_decode {"encoded": "SGVsbG8gV29ybGQ="}
✓ 工具调用成功
结果: Hello World

» /tools call hash {"text": "password123", "algorithm": "SHA256"}
✓ 工具调用成功
结果: ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f
```

#### AI 自动调用工具

AI 会根据你的问题自动选择并调用合适的工具：

```bash
» 帮我计算 125 的立方根
🤖 AI: [调用工具: calculator]
参数: {"expression": "125^(1/3)"}
结果: 5

125 的立方根是 5。
ⓘ 1.1s

» 现在几点了？
🤖 AI: [调用工具: datetime]
参数: {"format": "RFC3339"}
结果: 2025-10-15T10:30:00+08:00

现在是 2025 年 10 月 15 日，上午 10:30。
ⓘ 0.7s

» 帮我生成一个 UUID
🤖 AI: [调用工具: uuid]
参数: {}
结果: 550e8400-e29b-41d4-a716-446655440000

已生成 UUID: 550e8400-e29b-41d4-a716-446655440000
ⓘ 0.6s
```

#### 工具并行执行

RealConsole 支持同时调用多个工具（最多 3 个并行）：

```bash
» 帮我计算 2+2，并告诉我现在几点
🤖 AI: [并行调用工具]
  1. calculator: {"expression": "2+2"}
  2. datetime: {"format": "RFC3339"}

结果:
  1. 计算结果: 4
  2. 当前时间: 2025-10-15T10:30:00+08:00

2+2 等于 4，现在是 2025 年 10 月 15 日上午 10:30。
ⓘ 0.9s
```

#### 查看工具 Schema

查看工具的完整参数定义：

```bash
» /tools schema calculator
{
  "type": "object",
  "properties": {
    "expression": {
      "type": "string",
      "description": "数学表达式（支持 +, -, *, /, ^, sqrt, sin, cos 等）"
    }
  },
  "required": ["expression"]
}
```

### Intent DSL 系统

Intent DSL 是 RealConsole 的智能意图识别引擎，能够理解用户的自然语言输入并匹配到合适的处理逻辑。

#### 什么是 Intent？

Intent（意图）代表用户想要完成的任务或获取的信息。RealConsole 内置了 50+ 意图模板，涵盖：

- 📊 **数学计算**: `计算 2+2`，`求平方根 144`
- ⏰ **时间查询**: `现在几点`，`今天星期几`
- 🌤️ **天气查询**: `北京天气`，`上海明天天气`
- 📁 **文件操作**: `读取 file.txt`，`列出当前目录`
- 🔍 **搜索**: `搜索 Rust 教程`，`查询量子计算`
- 💻 **编程助手**: `用 Python 写 hello world`，`解释这段代码`

#### Intent 匹配示例

```bash
» 计算 2 的 10 次方
[Intent匹配] calculate_power
  关键词: ["计算", "次方"]
  实体提取: {base: 2, exponent: 10}
🤖 AI: 2^10 = 1024

» 现在几点了
[Intent匹配] query_time
  关键词: ["现在", "几点"]
🤖 AI: 现在是 2025-10-15 10:30:00

» 北京天气怎么样
[Intent匹配] query_weather
  关键词: ["天气"]
  实体提取: {city: "北京"}
🤖 AI: [调用工具: weather]
...
```

#### Intent 缓存

RealConsole 使用 LRU 缓存优化 Intent 匹配性能：

- **缓存容量**: 100 条
- **命中率**: 通常 > 60%
- **性能提升**: 匹配速度提升 10-50 倍

查看缓存统计（开发模式）：

```bash
RUST_LOG=debug realconsole
```

### 记忆系统

RealConsole 内置短期+长期记忆系统，让 AI 能够记住对话上下文。

#### 工作原理

- **短期记忆**: 当前会话的对话历史（环形缓冲区，默认保留最近 100 条）
- **长期记忆**: 持久化到文件，跨会话保留（可选）

#### 基础用法

```bash
» 我的名字是小明
🤖 AI: 你好，小明！很高兴认识你。
ⓘ 0.5s

» 你还记得我叫什么吗？
🤖 AI: 当然记得，你叫小明。
ⓘ 0.6s

» 我喜欢编程语言 Rust
🤖 AI: 很棒！Rust 是一门优秀的系统编程语言，具有内存安全和高性能的特点。
ⓘ 0.7s

» 我喜欢什么编程语言？
🤖 AI: 你喜欢 Rust 编程语言。
ⓘ 0.5s
```

#### 记忆管理命令

**查看所有记忆**：

```bash
» /memory list
📝 记忆列表 (5 条):
  1. [2025-10-15 10:00] 用户: 我的名字是小明
  2. [2025-10-15 10:00] AI: 你好，小明！很高兴认识你。
  3. [2025-10-15 10:01] 用户: 你还记得我叫什么吗？
  4. [2025-10-15 10:01] AI: 当然记得，你叫小明。
  5. [2025-10-15 10:02] 用户: 我喜欢编程语言 Rust
```

**搜索记忆**：

```bash
» /memory search "Rust"
🔍 搜索结果 (2 条):
  1. [2025-10-15 10:02] 用户: 我喜欢编程语言 Rust
  2. [2025-10-15 10:02] AI: 很棒！Rust 是一门优秀的系统编程语言...
```

**导出记忆**：

```bash
» /memory export
✓ 记忆已导出到: memory_export_20251015_103000.json
```

**清空记忆**：

```bash
» /memory clear
⚠️  确认清空所有记忆吗？此操作不可恢复 (y/n): y
✓ 已清空所有记忆
```

#### 记忆配置

在 `realconsole.yaml` 中配置记忆系统：

```yaml
features:
  memory_enabled: true           # 启用记忆系统
  memory_max_size: 100           # 短期记忆最大条数
  memory_persist: true           # 持久化到文件
  memory_persist_path: ~/.realconsole/memory.json  # 持久化路径
```

### 执行日志

RealConsole 会记录所有执行的命令和操作，用于审计和回溯。

#### 查看日志

```bash
» /log show
📋 执行日志（最近 10 条）:
  1. [2025-10-15 10:00:15] CHAT   "计算 2+2"
  2. [2025-10-15 10:00:16] TOOL   calculator {"expression": "2+2"}
  3. [2025-10-15 10:01:20] SHELL  !ls -la
  4. [2025-10-15 10:02:30] CMD    /help
  5. [2025-10-15 10:03:45] CHAT   "现在几点"
  6. [2025-10-15 10:03:46] TOOL   datetime {"format": "RFC3339"}
  ...
```

#### 导出日志

```bash
» /log export
✓ 日志已导出到: execution_log_20251015_103000.json
```

#### 清空日志

```bash
» /log clear
⚠️  确认清空所有日志吗？此操作不可恢复 (y/n): y
✓ 已清空所有日志
```

#### 日志配置

```yaml
features:
  log_enabled: true              # 启用执行日志
  log_max_size: 1000             # 最大日志条数
  log_persist: true              # 持久化到文件
  log_persist_path: ~/.realconsole/execution.log  # 日志文件路径
```

---

## 配置详解

### 配置文件结构

完整的 `realconsole.yaml` 配置文件包含以下部分：

```yaml
# 命令前缀
prefix: "/"

# LLM 配置
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
    max_tokens: 2000
    temperature: 0.7

  fallback:
    provider: ollama
    model: qwen2.5:7b
    endpoint: http://localhost:11434

# 功能开关
features:
  # Shell 执行
  shell_enabled: true
  shell_timeout: 10
  shell_blacklist:
    - "rm -rf /"
    - "dd if=/dev/zero"
    - "mkfs"

  # 记忆系统
  memory_enabled: true
  memory_max_size: 100
  memory_persist: true
  memory_persist_path: ~/.realconsole/memory.json

  # 执行日志
  log_enabled: true
  log_max_size: 1000
  log_persist: true
  log_persist_path: ~/.realconsole/execution.log

  # 工具调用
  tool_calling_enabled: true
  tool_max_parallel: 3
  tool_max_iterations: 5

  # Intent DSL
  intent_enabled: true
  intent_cache_size: 100

# 工具配置
tools:
  calculator:
    enabled: true
  datetime:
    enabled: true
    timezone: "Asia/Shanghai"
  weather:
    enabled: true
    api_key: ${WEATHER_API_KEY}  # 可选
  file_read:
    enabled: true
    max_size: 1048576  # 1MB
  file_write:
    enabled: true
    max_size: 1048576  # 1MB
```

### 配置项说明

#### 1. 命令前缀

```yaml
prefix: "/"
```

自定义系统命令前缀，默认为 `/`。可以改为其他字符，如 `!`、`#` 等。

#### 2. LLM 配置

**provider**: LLM 提供商（`deepseek` 或 `ollama`）

**model**: 模型名称
- Deepseek: `deepseek-chat`, `deepseek-coder`
- Ollama: 已拉取的模型名（如 `qwen2.5:7b`）

**endpoint**: API 端点
- Deepseek: `https://api.deepseek.com/v1`
- Ollama: `http://localhost:11434`

**api_key**: API 密钥（仅 Deepseek 需要）

**max_tokens**: 最大生成 token 数（默认 2000）

**temperature**: 生成随机性（0-2，默认 0.7）
- 0: 确定性输出（适合代码生成）
- 1: 平衡
- 2: 高随机性（适合创意写作）

#### 3. Shell 执行配置

**shell_enabled**: 是否启用 Shell 执行（默认 true）

**shell_timeout**: 命令超时时间（秒，默认 10）

**shell_blacklist**: 危险命令黑名单（数组）

#### 4. 记忆系统配置

**memory_enabled**: 是否启用记忆系统（默认 true）

**memory_max_size**: 短期记忆最大条数（默认 100）

**memory_persist**: 是否持久化（默认 false）

**memory_persist_path**: 持久化文件路径

#### 5. 工具调用配置

**tool_calling_enabled**: 是否启用工具调用（默认 true）

**tool_max_parallel**: 最大并行工具数（默认 3）

**tool_max_iterations**: 最大迭代轮数（默认 5）

### 环境变量

配置文件支持环境变量替换：

```yaml
api_key: ${DEEPSEEK_API_KEY}
endpoint: ${API_ENDPOINT:-https://api.deepseek.com/v1}  # 带默认值
```

`.env` 文件示例：

```bash
# .env
DEEPSEEK_API_KEY=sk-xxxxxxxxxxxxxxxx
WEATHER_API_KEY=your-weather-api-key
API_ENDPOINT=https://custom-endpoint.com
```

---

## 故障排除

### 常见错误及解决方案

#### [E001] 配置文件不存在

**问题**：
```
[E001] 配置文件不存在: realconsole.yaml
```

**解决方案**：
1. 运行配置向导：
   ```bash
   realconsole wizard --quick
   ```
2. 或手动创建配置文件（参考 [配置详解](#配置详解)）

#### [E002] 配置文件解析失败

**问题**：
```
[E002] 配置文件解析失败: realconsole.yaml
YAML 语法错误: mapping values are not allowed here
```

**解决方案**：
1. 检查 YAML 语法（缩进、冒号、引号等）
2. 使用在线 YAML 验证器检查
3. 参考示例配置文件：`config/minimal.yaml`

#### [E102] LLM 身份验证失败

**问题**：
```
[E102] LLM 身份验证失败
HTTP 401: Unauthorized
```

**解决方案**：
1. 检查 `.env` 文件中的 `DEEPSEEK_API_KEY` 是否正确
2. 验证 API Key 格式（应以 `sk-` 开头）
3. 确认 API Key 未过期

#### [E302] 命令包含危险操作

**问题**：
```
[E302] 命令包含危险操作，已被安全策略阻止
```

**原因**：
命令匹配了黑名单规则（如 `rm -rf /`、`dd`、`mkfs` 等）

**解决方案**：
1. 检查命令是否真的需要执行
2. 如确实需要，考虑在虚拟环境中执行
3. 或修改配置文件的 `shell_blacklist`（谨慎！）

#### [E303] 命令执行超时

**问题**：
```
[E303] 命令执行超时（超过 10 秒）
```

**解决方案**：
1. 检查命令是否卡死
2. 增加超时时间（编辑 `realconsole.yaml`）：
   ```yaml
   features:
     shell_timeout: 30  # 增加到 30 秒
   ```

#### [E401] 工具未注册

**问题**：
```
[E401] 工具未注册: unknown_tool
```

**解决方案**：
1. 查看可用工具：
   ```bash
   /tools list
   ```
2. 检查工具名拼写是否正确
3. 确认工具已在配置文件中启用

### 调试技巧

#### 启用详细日志

```bash
RUST_LOG=debug realconsole
```

日志级别：
- `error`: 仅错误
- `warn`: 警告+错误
- `info`: 一般信息
- `debug`: 调试信息（包含 Intent 匹配、工具调用等）
- `trace`: 最详细日志

#### 测试单次命令

```bash
realconsole --once "/version"
realconsole --once "你好"
realconsole --once "/tools list"
```

#### 查看 Backtrace

```bash
RUST_BACKTRACE=1 realconsole
```

#### 验证配置文件

```bash
# 使用 yq 工具（如果已安装）
yq . realconsole.yaml

# 或使用 Python
python3 -c "import yaml; print(yaml.safe_load(open('realconsole.yaml')))"
```

---

## 最佳实践

### 1. LLM 选择建议

| 场景 | 推荐配置 |
|------|----------|
| **在线使用，追求质量** | Deepseek API (primary) |
| **离线使用，数据隐私** | Ollama (primary) |
| **高可用性** | Deepseek (primary) + Ollama (fallback) |
| **开发测试** | Ollama (本地调试更方便) |

### 2. 安全建议

1. **永远不要在 git 中提交 `.env` 文件**
2. **定期更新 API Key**
3. **谨慎修改 `shell_blacklist`**
4. **生产环境建议禁用 Shell 执行**：
   ```yaml
   features:
     shell_enabled: false
   ```
5. **使用只读权限运行 RealConsole**（如果可能）

### 3. 性能优化

1. **启用 Intent 缓存**（默认已启用）：
   ```yaml
   features:
     intent_cache_size: 200  # 增大缓存
   ```

2. **调整工具并行度**：
   ```yaml
   features:
     tool_max_parallel: 5  # 增加并行数（根据 CPU 核心数）
   ```

3. **减少记忆大小**（节省内存）：
   ```yaml
   features:
     memory_max_size: 50  # 减少到 50 条
   ```

4. **禁用不需要的功能**：
   ```yaml
   features:
     log_enabled: false  # 禁用日志（如不需要）
   ```

### 4. 使用场景推荐

#### 场景 A: 数据分析助手

```yaml
tools:
  calculator: { enabled: true }
  file_read: { enabled: true }
  file_write: { enabled: true }
  json_parse: { enabled: true }

features:
  shell_enabled: true  # 允许运行数据处理脚本
  memory_enabled: true  # 记住分析上下文
```

#### 场景 B: 编程助手

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-coder  # 使用代码专用模型
    temperature: 0.3  # 降低随机性

features:
  shell_enabled: true  # 允许运行代码
  memory_enabled: true  # 记住项目上下文
```

#### 场景 C: 系统管理

```yaml
features:
  shell_enabled: true
  shell_timeout: 30  # 增加超时（某些命令可能较慢）
  log_enabled: true  # 启用日志审计
  log_persist: true
```

### 5. 多轮对话技巧

1. **明确上下文**：
   ```bash
   » 我正在开发一个 Rust Web 服务器项目
   » 使用 Actix-Web 框架
   » 现在遇到了路由配置问题
   » 如何配置嵌套路由？
   ```

2. **引用之前的对话**：
   ```bash
   » 你刚才提到的 `HttpServer::new`，能详细解释一下吗？
   ```

3. **使用记忆搜索**：
   ```bash
   » /memory search "Actix"  # 查找之前关于 Actix 的讨论
   ```

---

## 附录

### A. 完整命令参考

#### 基础命令

| 命令 | 说明 | 别名 |
|------|------|------|
| `/help` | 快速帮助 | `/h`, `/?` |
| `/help all` | 完整帮助 | - |
| `/help tools` | 工具管理帮助 | - |
| `/help memory` | 记忆系统帮助 | - |
| `/help shell` | Shell 执行帮助 | - |
| `/examples` | 使用示例库 | - |
| `/quickref` | 快速参考卡片 | - |
| `/version` | 显示版本 | `/v` |
| `/quit` | 退出程序 | `/q`, `Ctrl-D` |

#### 工具管理

| 命令 | 说明 |
|------|------|
| `/tools list` | 列出所有工具 |
| `/tools call <name> <json>` | 调用工具 |
| `/tools schema <name>` | 查看工具 Schema |

#### 记忆系统

| 命令 | 说明 |
|------|------|
| `/memory list` | 列出所有记忆 |
| `/memory search <query>` | 搜索记忆 |
| `/memory clear` | 清空记忆 |
| `/memory export` | 导出记忆 |

#### 执行日志

| 命令 | 说明 |
|------|------|
| `/log show` | 显示日志 |
| `/log export` | 导出日志 |
| `/log clear` | 清空日志 |

### B. 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl-D` | 退出程序 |
| `Ctrl-C` | 中断当前输入 |
| `Ctrl-L` | 清屏 |
| `↑` / `↓` | 历史命令导航 |
| `Ctrl-A` | 光标移到行首 |
| `Ctrl-E` | 光标移到行尾 |
| `Ctrl-U` | 清空当前行 |
| `Ctrl-K` | 删除光标到行尾 |
| `Ctrl-W` | 删除前一个单词 |
| `Alt-B` / `Alt-F` | 光标移动一个单词 |

### C. 错误代码速查

| 代码 | 分类 | 说明 |
|------|------|------|
| E001 | Config | 配置文件不存在 |
| E002 | Config | 配置文件解析失败 |
| E100-E199 | LLM | LLM 相关错误 |
| E102 | LLM | 身份验证失败 |
| E300-E399 | Shell | Shell 执行错误 |
| E302 | Shell | 危险命令拦截 |
| E303 | Shell | 命令超时 |
| E400-E499 | Tool | 工具调用错误 |
| E401 | Tool | 工具未注册 |
| E500-E599 | Memory | 记忆系统错误 |

完整错误码列表：[docs/design/ERROR_SYSTEM_DESIGN.md](design/ERROR_SYSTEM_DESIGN.md)

### D. 相关文档

- **[快速入门](guides/QUICKSTART.md)** - 5 分钟上手
- **[开发者指南](DEVELOPER_GUIDE.md)** - 架构与开发
- **[工具调用用户指南](guides/TOOL_CALLING_USER_GUIDE.md)** - 工具详解
- **[工具调用开发指南](guides/TOOL_CALLING_DEVELOPER_GUIDE.md)** - 创建自定义工具
- **[Intent DSL 指南](guides/INTENT_DSL_GUIDE.md)** - 意图识别详解
- **[LLM 配置指南](guides/LLM_SETUP_GUIDE.md)** - LLM 高级配置
- **[设计哲学](design/PHILOSOPHY.md)** - "一分为三"设计理念
- **[API 文档](API.md)** - 核心模块 API
- **[CHANGELOG](CHANGELOG.md)** - 版本更新历史

### E. 资源链接

- **GitHub**: https://github.com/your-repo/realconsole
- **文档中心**: https://docs.realconsole.com
- **Issue 跟踪**: https://github.com/your-repo/realconsole/issues
- **Deepseek API**: https://platform.deepseek.com/
- **Ollama**: https://ollama.ai/

---

**版本**: v0.5.0
**更新日期**: 2025-10-15
**文档状态**: ✅ Week 3 更新完成

**有问题？** 查看 [故障排除](#故障排除) 或 [提交 Issue](https://github.com/your-repo/realconsole/issues)！
