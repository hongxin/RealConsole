# RealConsole 快速开始

欢迎使用 RealConsole！这份指南将帮助你在 **5 分钟内**完成安装、配置并开始使用。

## 目录
- [快速安装](#快速安装)
- [首次运行](#首次运行)
- [核心功能](#核心功能)
- [常用命令](#常用命令)
- [故障排查](#故障排查)
- [下一步](#下一步)

---

## 快速安装

### 1. 构建项目

```bash
cd realconsole
cargo build --release
```

构建时间约 2-3 分钟，生成的二进制文件位于 `target/release/realconsole`。

### 2. 首次运行：配置向导

RealConsole 提供了交互式配置向导，让你快速完成初始化：

```bash
./target/release/realconsole wizard
```

或使用快速模式（推荐新用户）：

```bash
./target/release/realconsole wizard --quick
```

**向导会引导你完成**：
- ✅ LLM 提供商选择（Deepseek API 或 Ollama 本地）
- ✅ API Key 配置（如使用 Deepseek）
- ✅ 基础功能设置（Shell 执行、记忆系统等）
- ✅ 自动生成 `realconsole.yaml` 配置文件

**快速模式示例**：
```
🧙 RealConsole 配置向导（快速模式）

LLM 提供商:
1. Deepseek API（推荐，云端服务）
2. Ollama（本地运行）

选择 (1-2): 1

请输入 Deepseek API Key: sk-xxxxxxxx

✅ 配置文件已生成: realconsole.yaml
✅ 环境变量文件已创建: .env

现在可以运行: realconsole
```

### 3. 启动 RealConsole

```bash
./target/release/realconsole
```

启动后你会看到欢迎界面：

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

---

## 核心功能

### 1. 智能对话（AI 助手）

直接输入问题，无需命令前缀：

```bash
» 计算 2 的 10 次方
🤖 AI: 2^10 = 1024
ⓘ 0.8s

» 用 Rust 写一个 hello world
🤖 AI: 这是一个简单的 Rust Hello World 程序：

fn main() {
    println!("Hello, World!");
}

你可以用 `cargo run` 运行它。
ⓘ 1.2s
```

**特性**：
- ✨ LLM 实时流式输出
- ⏱️ 自动显示响应时间
- 🔄 支持多轮对话（带记忆）

### 2. Shell 命令执行

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
```

**安全保护**：
- ⚠️ 危险命令自动拦截（如 `rm -rf /`）
- ⏱️ 超时保护（默认 10 秒）
- 🔐 黑名单机制

**示例**：
```bash
» !rm -rf /
[E302] 命令包含危险操作，已被安全策略阻止

💡 修复建议:
1. 此命令可能造成系统损坏，建议使用更安全的替代方案
2. 查看允许的命令列表和安全策略
   📖 https://docs.realconsole.com/shell-safety
```

### 3. 工具调用（14 个内置工具）

RealConsole 内置了 14 个实用工具，AI 可以自动调用它们完成任务：

```bash
» /tools list
📦 已注册工具 (14):
  1. calculator - 数学计算器
  2. datetime - 时间日期工具
  3. file_read - 读取文件内容
  4. file_write - 写入文件
  5. weather - 天气查询
  6. search - 网络搜索
  ...
```

**手动调用工具**：
```bash
» /tools call calculator {"expression": "sqrt(144)"}
✓ 工具调用成功
结果: 12

» /tools call datetime {"format": "RFC3339"}
✓ 工具调用成功
当前时间: 2025-10-15T10:30:00+08:00
```

**AI 自动调用**：
```bash
» 帮我计算 125 的立方根
🤖 AI: [调用工具: calculator]
参数: {"expression": "125^(1/3)"}
结果: 5

125 的立方根是 5。
ⓘ 1.1s
```

### 4. 多层次帮助系统

RealConsole 提供了丰富的帮助信息：

```bash
» /help           # 快速帮助（一屏内容）
» /help all       # 完整帮助（所有命令）
» /help tools     # 工具管理帮助
» /help memory    # 记忆系统帮助
» /help shell     # Shell 执行帮助
» /examples       # 使用示例库
» /quickref       # 快速参考卡片
```

**示例**：
```bash
» /examples
💡 RealConsole 使用示例

━━━ 智能对话 ━━━
  计算 2 的 10 次方
  用 Rust 写一个 hello world
  ...

━━━ 工具调用 ━━━
  /tools call calculator {"expression": "sqrt(144)"}
  /tools call datetime {"format": "RFC3339"}
  ...

━━━ Shell 执行 ━━━
  !ls -la
  !git status
  ...
```

### 5. 单次执行模式

不启动 REPL，直接执行单个命令（适合脚本调用）：

```bash
# 显示帮助
./target/release/realconsole --once "/help"

# 调用工具
./target/release/realconsole --once "/tools call calculator {\"expression\": \"2+2\"}"

# AI 对话
./target/release/realconsole --once "什么是 Rust"
```

---

## 常用命令

### 基础命令

| 命令 | 说明 | 别名 |
|------|------|------|
| `/help` | 快速帮助 | `/h`, `/?` |
| `/help all` | 完整帮助 | - |
| `/examples` | 使用示例 | - |
| `/quickref` | 快速参考 | - |
| `/version` | 显示版本 | `/v` |
| `/quit` | 退出程序 | `/q`, `Ctrl-D` |

### 工具管理

| 命令 | 说明 |
|------|------|
| `/tools list` | 列出所有可用工具 |
| `/tools call <name> <json>` | 手动调用指定工具 |
| `/tools schema <name>` | 查看工具的 JSON Schema |

### 记忆系统

| 命令 | 说明 |
|------|------|
| `/memory list` | 列出所有记忆 |
| `/memory search <query>` | 搜索记忆 |
| `/memory clear` | 清空记忆 |
| `/memory export` | 导出记忆到文件 |

### 执行日志

| 命令 | 说明 |
|------|------|
| `/log show` | 显示最近执行记录 |
| `/log export` | 导出日志到文件 |
| `/log clear` | 清空日志 |

### 配置管理

| 命令 | 说明 |
|------|------|
| `realconsole wizard` | 启动配置向导（完整模式） |
| `realconsole wizard --quick` | 启动配置向导（快速模式） |
| `realconsole --config <path>` | 使用指定配置文件 |

---

## 键盘快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl-D` | 退出程序 |
| `Ctrl-C` | 中断当前输入 |
| `Ctrl-L` | 清屏 |
| `↑` / `↓` | 历史命令导航 |
| `Ctrl-A` | 光标移到行首 |
| `Ctrl-E` | 光标移到行尾 |
| `Ctrl-U` | 清空当前行 |

---

## 故障排查

### 常见问题

#### 1. 配置文件未找到

**错误信息**：
```
[E001] 配置文件不存在: realconsole.yaml

💡 修复建议:
1. 运行配置向导创建配置文件
   💻 realconsole wizard
```

**解决方案**：
运行配置向导自动生成配置文件：
```bash
./target/release/realconsole wizard --quick
```

#### 2. LLM API Key 错误

**错误信息**：
```
[E102] LLM 身份验证失败

💡 修复建议:
1. 检查 API Key 是否正确配置
2. 验证 .env 文件中的 API Key 格式
   💻 cat .env
```

**解决方案**：
1. 检查 `.env` 文件中的 `DEEPSEEK_API_KEY` 是否正确
2. 重新运行向导：`realconsole wizard --quick`

#### 3. Shell 命令超时

**错误信息**：
```
[E303] 命令执行超时（超过 10 秒）

💡 修复建议:
1. 命令执行时间过长，请检查命令或增加超时时间
2. 在配置文件中调整 features.shell_timeout
   💻 vi realconsole.yaml
```

**解决方案**：
编辑 `realconsole.yaml`，增加超时时间：
```yaml
features:
  shell_timeout: 30  # 增加到 30 秒
```

#### 4. 工具调用失败

**错误信息**：
```
[E401] 工具未注册: unknown_tool
```

**解决方案**：
查看可用工具列表：
```bash
» /tools list
```

#### 5. 记忆系统错误

**错误信息**：
```
[E501] 记忆系统未初始化
```

**解决方案**：
确保配置文件中启用了记忆系统：
```yaml
features:
  memory_enabled: true
  memory_max_size: 100
```

### 调试技巧

**查看详细日志**：
```bash
RUST_LOG=debug ./target/release/realconsole
```

**测试配置文件**：
```bash
./target/release/realconsole --config realconsole.yaml --once "/version"
```

**检查依赖版本**：
```bash
cargo tree
```

---

## 常见问题 (FAQ)

### Q1: RealConsole 需要联网吗？

**A**: 取决于你的 LLM 配置：
- 使用 **Deepseek API**：需要联网
- 使用 **Ollama 本地模型**：不需要联网（推荐离线使用）

### Q2: 如何切换 LLM 提供商？

**A**: 重新运行配置向导：
```bash
./target/release/realconsole wizard
```
或手动编辑 `realconsole.yaml` 文件。

### Q3: 支持哪些操作系统？

**A**:
- ✅ macOS (Intel & Apple Silicon)
- ✅ Linux (x86_64 & ARM64)
- ✅ Windows (需要 WSL 或原生构建)

### Q4: RealConsole 与 Python 版本的区别？

**A**:
| 特性 | Python 版本 | Rust 版本 (RealConsole) |
|------|-------------|-------------------------|
| 启动速度 | ~300ms | ~10ms (30倍提升) |
| 内存占用 | ~80MB | ~8MB (10倍优化) |
| 二进制大小 | N/A | ~15MB |
| 工具调用 | 基础 | 14 个内置工具 + 并行执行 |
| 错误系统 | 简单 | 30+ 错误码 + 修复建议 |

### Q5: 如何添加自定义工具？

**A**: 参考 [工具调用开发指南](TOOL_CALLING_DEVELOPER_GUIDE.md)，实现 `Tool` trait：
```rust
use realconsole::Tool;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    async fn execute(&self, params: Value) -> Result<Value> { ... }
}
```

### Q6: 支持多轮对话吗？

**A**: 是的！RealConsole 内置记忆系统，自动记录对话上下文：
```bash
» 我的名字是小明
🤖 AI: 你好，小明！

» 你还记得我叫什么吗？
🤖 AI: 当然记得，你叫小明。
```

---

## 下一步

恭喜完成快速入门！接下来你可以：

### 📚 深入学习

- **[用户完整手册](../USER_GUIDE.md)** - 所有功能的详细说明
- **[工具调用用户指南](TOOL_CALLING_USER_GUIDE.md)** - 14 个工具的完整文档
- **[Intent DSL 指南](INTENT_DSL_GUIDE.md)** - 自定义意图识别
- **[LLM 配置指南](LLM_SETUP_GUIDE.md)** - 高级 LLM 配置

### 🛠️ 开发扩展

- **[开发者指南](../DEVELOPER_GUIDE.md)** - 架构与开发环境
- **[工具调用开发指南](TOOL_CALLING_DEVELOPER_GUIDE.md)** - 创建自定义工具
- **[API 文档](../API.md)** - 核心模块 API

### 🚀 参与贡献

- **[GitHub 仓库](https://github.com/your-repo/realconsole)** - 查看源码
- **[Issue 跟踪](https://github.com/your-repo/realconsole/issues)** - 报告问题
- **[贡献指南](../DEVELOPER_GUIDE.md#贡献指南)** - 如何贡献代码

### 💡 实战案例

尝试这些实际应用场景：

**数据分析**：
```bash
» 读取 data.csv 文件，计算平均值
» 使用工具分析文件内容并生成报告
```

**系统管理**：
```bash
» !df -h
» !ps aux | grep python
» 查看系统磁盘使用情况
```

**编程助手**：
```bash
» 用 Rust 写一个 HTTP 服务器
» 解释这段代码的作用: [粘贴代码]
» 如何优化这个算法？
```

---

**版本**: v0.5.0
**更新日期**: 2025-10-15
**文档状态**: ✅ Week 3 更新完成

有问题？查看 [完整文档](../README.md) 或 [提交 Issue](https://github.com/your-repo/realconsole/issues)！
