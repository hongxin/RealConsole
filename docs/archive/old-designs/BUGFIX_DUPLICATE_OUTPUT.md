# Bug 修复：Function Calling 输出重复

## 问题描述

在 Function Calling 模式下，LLM 的响应被打印了两次：

```
realconsole> 列出 src 目录下的所有 Rust 文件
在 src 目录下，我找到了以下 Rust 文件：
1. `llm_manager.rs` (文件)
2. `repl.rs` (文件)
...
在 src 目录下，我找到了以下 Rust 文件：  <-- 重复了
1. `llm_manager.rs` (文件)
2. `repl.rs` (文件)
...
```

## 根本原因

在 `src/agent.rs` 的 `handle_text_with_tools` 方法中：

```rust
fn handle_text_with_tools(&self, text: &str) -> String {
    match /* ... */ {
        Ok(response) => {
            println!("{}", response);  // 第一次打印
            response  // 返回给 handle()
        }
        // ...
    }
}
```

然后在 `src/repl.rs` 中：

```rust
pub fn run(agent: &Agent) -> RustyResult<()> {
    loop {
        let response = agent.handle(&line);
        if !response.is_empty() {
            println!("{}", response);  // 第二次打印
        }
    }
}
```

响应被打印了两次：
1. 在 `handle_text_with_tools` 中打印
2. 在 REPL 中又打印了返回值

## 解决方案

移除 `handle_text_with_tools` 中的打印，让 REPL 统一处理所有输出：

### 修复前 (src/agent.rs:269-272)

```rust
Ok(response) => {
    // 显示响应
    println!("{}", response);  // ❌ 多余的打印
    response
}
```

### 修复后 (src/agent.rs:269-271)

```rust
Ok(response) => {
    // 返回响应，让 REPL 统一处理打印
    response  // ✅ 只返回，不打印
}
```

## 设计原则

为了保持一致性和可维护性，采用以下设计原则：

### 1. 统一输出策略

所有用户可见的输出应该在同一层级处理：

```
用户输入 → Agent.handle() → [处理] → 返回结果 → REPL 打印
```

**好处**:
- 单一职责：Agent 负责处理，REPL 负责显示
- 易于测试：可以测试 Agent 的返回值而不依赖 stdout
- 易于扩展：未来可以轻松添加日志、格式化等功能

### 2. 流式输出的特例

唯一的例外是流式输出模式 (`handle_text_streaming`)：

```rust
fn handle_text_streaming(&self, text: &str) -> String {
    manager.chat_stream(text, |token| {
        print!("{}", token);  // 实时打印每个 token
        let _ = io::stdout().flush();
    }).await;

    println!();
    String::new()  // 返回空字符串，内容已实时显示
}
```

**为什么可以这样**:
- 流式输出的目的就是实时显示
- 不需要等待完整响应
- 牺牲完整记忆以换取实时性

### 3. 对比表

| 模式 | 打印位置 | 返回值 | 记录到记忆 | 原因 |
|------|---------|--------|-----------|------|
| **Shell 执行** | REPL | 输出内容 | ✓ | 标准输出流 |
| **命令执行** | REPL | 命令结果 | ✓ | 标准输出流 |
| **Function Calling** | REPL | 完整响应 | ✓ | 标准输出流 |
| **流式输出** | Agent | 空字符串 | ✗ | 实时性优先 |

## 测试验证

### 1. 单元测试

```bash
cargo test test_e2e_simple_tool_call -- --nocapture
```

**预期**:
- 测试通过
- 没有重复输出

### 2. 集成测试

```bash
# 启用 Function Calling
# 修改 realconsole.yaml:
#   features:
#     tool_calling_enabled: true

./target/release/realconsole

realconsole> 现在几点了？
```

**预期**:
- 响应只显示一次
- 响应被记录到记忆系统

### 3. 回归测试

```bash
# 测试其他模式没有被影响
./target/release/realconsole --once "!date"
./target/release/realconsole --once "/help"
```

**预期**:
- 所有命令正常工作
- 输出格式正确

## 相关文件

修改的文件：
- `src/agent.rs` - 移除 `handle_text_with_tools` 中的 `println!`

相关文件（未修改）：
- `src/repl.rs` - 统一的输出处理逻辑
- `src/tool_executor.rs` - 工具执行引擎
- `tests/test_function_calling_e2e.rs` - E2E 测试

## 影响范围

- ✅ 修复了 Function Calling 模式的重复输出
- ✅ 保持了记忆系统的正常工作
- ✅ 保持了执行日志的正常记录
- ✅ 其他模式（Shell、命令、流式输出）不受影响

## 提交信息

```
fix(agent): remove duplicate output in Function Calling mode

Function calling responses were being printed twice:
1. In handle_text_with_tools (println!)
2. In REPL (println! on returned value)

Solution: Remove printing from handle_text_with_tools, let REPL
handle all output consistently.

This maintains memory recording and execution logging while fixing
the duplicate output issue.

Fixes: #XX (如果有 issue 编号)
```

## 经验教训

1. **保持一致性**: 所有输出应该在同一层级处理
2. **单一职责**: Agent 处理逻辑，REPL 处理显示
3. **测试覆盖**: E2E 测试应该验证输出行为
4. **文档记录**: 重要的设计决策应该有文档支持

---

修复日期: 2025-10-14
修复版本: v0.1.0
修复人员: Claude Code
