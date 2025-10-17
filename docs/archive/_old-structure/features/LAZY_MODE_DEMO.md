# RealConsole 懒人模式演示

## ✨ 新特性：智能对话模式

现在您可以直接输入问题，无需输入 `/ask` 命令前缀！

## 使用示例

### 旧方式（仍然支持）
```bash
» /ask 你好
你好！请问有什么可以帮你的？

» /ask 用 Rust 写一个 hello world
[AI 回复...]
```

### 新方式（推荐）
```bash
» 你好
你好！请问有什么可以帮你的？

» 用 Rust 写一个 hello world
[AI 回复...]
```

## 设计理念

**极简主义 · 懒人化**
- ✅ 默认智能：直接输入问题即可对话
- ✅ 保留命令：需要特定功能时使用 `/` 前缀
- ✅ 安静优雅：无需额外的提示和噪音

## 启动体验

```
$ ./realconsole --config realconsole.yaml

RealConsole v0.1.0
极简版智能 CLI Agent

💬 直接输入问题即可对话
   使用 /help 查看命令 | Ctrl-D 退出

» 你好
你好！请问有什么可以帮你的？

» 什么是 Rust？
Rust 是一种系统编程语言...

» /help
[显示帮助信息]

» /quit
Bye 👋
```

## 命令优先级

1. **命令模式** (`/` 开头)：执行特定命令
   - `/help` - 显示帮助
   - `/llm` - LLM 状态管理
   - `/quit` - 退出程序

2. **Shell 模式** (`!` 开头)：执行 shell 命令
   - `!ls` - 列出文件
   - `!pwd` - 显示当前目录

3. **智能对话模式** (其他输入)：与 AI 对话
   - `你好`
   - `用 Python 写一个快速排序`
   - `解释一下 Rust 的所有权系统`

## 哲学

> "大道至简" - 《道德经》

最好的工具是让你感觉不到它的存在。RealConsole 遵循这一理念：
- 启动即用，无需学习复杂命令
- 智能理解意图，减少用户负担
- 保持安静简洁，专注核心价值

## 配置

确保配置文件中有可用的 LLM：

```yaml
llm:
  primary:
    provider: deepseek
    model: deepseek-chat
    endpoint: https://api.deepseek.com/v1
    api_key: ${DEEPSEEK_API_KEY}
```

## 错误处理

如果 LLM 不可用，系统会友好地提示：

```
» 你好
LLM 调用失败: No LLM configured
提示: 使用 /help
```

---

**享受简洁高效的 AI 交互体验！** 🚀
