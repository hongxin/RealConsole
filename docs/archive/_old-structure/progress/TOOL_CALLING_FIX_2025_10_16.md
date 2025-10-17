# Tool Calling Bug Fix - 2025-10-16

## 问题描述

用户报告使用工具调用功能时出现 HTTP 400 错误：

```bash
./target/release/realconsole
这个项目有多少行rs代码？

处理失败: LLM 调用失败: HTTP 400: {"error":{"message":"An assistant message with 'tool_calls' must be followed by tool messages responding to each 'tool_call_id'. (insufficient tool messages following tool_calls message)"...
```

## 根本原因

在 `src/tool_executor.rs` 的 `execute_iterative()` 方法中（第143行附近），存在工具调用数量不匹配的问题：

### 问题代码

```rust
// 执行工具调用（会被限制为最多 3 个）
let tool_results = self.execute_tool_calls(&tool_requests).await;

// 将助手的工具调用添加到消息历史（包含所有工具调用，可能 > 3）
messages.push(Message::assistant_with_tools(response.tool_calls));

// 将工具结果添加到消息历史（只有 <= 3 个结果）
for result in tool_results {
    messages.push(Message::tool_result(result.call_id, result.content));
}
```

### 问题分析

1. `execute_tool_calls()` 方法内部会限制每轮最多执行 `max_tools_per_round` 个工具（默认3个）
2. 但是添加到消息历史的 `Message::assistant_with_tools()` 包含了**所有**的 tool_calls（可能是4个或更多）
3. 这导致 Deepseek API 收到的消息中：
   - assistant 消息有 4 个 tool_calls
   - 但只有 3 个 tool 结果消息
4. 违反了 Deepseek API 的要求："每个 tool_call_id 必须有对应的 tool 结果消息"

## 解决方案

在 `src/tool_executor.rs` 第268-279行添加了工具调用限制逻辑：

```rust
// ⚠️ 限制工具调用数量，确保 assistant 消息中的 tool_calls 与实际执行的一致
let limited_tool_calls = if response.tool_calls.len() > self.max_tools_per_round {
    response.tool_calls[..self.max_tools_per_round].to_vec()
} else {
    response.tool_calls.clone()
};

// 执行工具调用（execute_tool_calls 内部也会限制，这里保持一致）
let tool_results = self.execute_tool_calls(&tool_requests).await;

// 将助手的工具调用添加到消息历史（只包含实际执行的工具调用）
messages.push(Message::assistant_with_tools(limited_tool_calls));

// 将工具结果添加到消息历史
for result in tool_results {
    messages.push(Message::tool_result(result.call_id, result.content));
}
```

## 验证

### 单元测试
```bash
$ cargo test tool_executor --lib

running 7 tests
test tool_executor::tests::test_execute_tool_call ... ok
test tool_executor::tests::test_execution_statistics ... ok
test tool_executor::tests::test_execution_mode_switch ... ok
test tool_executor::tests::test_execute_tool_call_error ... ok
test tool_executor::tests::test_execute_tool_calls_limit ... ok
test tool_executor::tests::test_sequential_execution ... ok
test tool_executor::tests::test_parallel_execution ... ok

test result: ok. 7 passed
```

### 集成测试
HTTP 400 错误已不再出现，但发现了新的超时问题（需要进一步调查）。

## 影响范围

- **修改文件**: 1 个 (`src/tool_executor.rs`)
- **新增代码**: 13 行
- **受影响功能**: 所有涉及工具调用的 LLM 交互
- **向后兼容**: 是（不影响现有功能）

## 遗留问题

1. **工具调用超时**: 修复后发现某些工具调用场景会超时（60秒），需要进一步调查：
   - 可能是 LLM 响应时间过长
   - 可能是工具执行时间过长
   - 可能是迭代次数过多

2. **建议后续优化**:
   - 添加工具调用的详细日志
   - 增加可配置的超时时间
   - 对超过限制的工具调用向用户提示

## 技术细节

### Deepseek API Tool Calling 要求

根据 Deepseek API 文档，当使用 Function Calling 时：

1. 如果 assistant 消息包含 `tool_calls` 字段，必须紧接着为**每一个** `tool_call_id` 提供一个 tool 角色的消息
2. tool 消息必须包含：
   - `role: "tool"`
   - `tool_call_id`: 对应的调用 ID
   - `content`: 工具执行结果
3. 消息顺序必须是：user → assistant(with tool_calls) → tool → tool → ... → 下一轮

### 消息序列化示例

**修复前**（错误）:
```json
[
  {"role": "user", "content": "帮我计算1+2+3+4"},
  {
    "role": "assistant",
    "tool_calls": [
      {"id": "call_1", "function": {...}},
      {"id": "call_2", "function": {...}},
      {"id": "call_3", "function": {...}},
      {"id": "call_4", "function": {...}}  // 第4个
    ]
  },
  {"role": "tool", "tool_call_id": "call_1", "content": "3"},
  {"role": "tool", "tool_call_id": "call_2", "content": "7"},
  {"role": "tool", "tool_call_id": "call_3", "content": "..."}
  // ❌ 缺少 call_4 的结果！
]
```

**修复后**（正确）:
```json
[
  {"role": "user", "content": "帮我计算1+2+3+4"},
  {
    "role": "assistant",
    "tool_calls": [
      {"id": "call_1", "function": {...}},
      {"id": "call_2", "function": {...}},
      {"id": "call_3", "function": {...}}
      // ✅ 只包含前3个（max_tools_per_round=3）
    ]
  },
  {"role": "tool", "tool_call_id": "call_1", "content": "3"},
  {"role": "tool", "tool_call_id": "call_2", "content": "7"},
  {"role": "tool", "tool_call_id": "call_3", "content": "..."}
  // ✅ 每个 tool_call_id 都有对应的结果
]
```

## 经验教训

1. **API 契约验证**: 当集成第三方 API 时，必须严格遵守其消息格式要求
2. **限流一致性**: 如果在多处进行限流，必须确保所有地方的限制逻辑一致
3. **测试覆盖**: 需要添加更多边界情况的测试（例如：LLM 返回超过限制数量的工具调用）

---

**修复时间**: 2025-10-16
**修复人**: Claude Code
**测试状态**: ✅ 单元测试通过，集成测试部分通过（超时问题待解决）
**合并状态**: ✅ 已合并到主分支
