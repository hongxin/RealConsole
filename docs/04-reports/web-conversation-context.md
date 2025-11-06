# Web 终端多轮对话上下文支持

**版本**: v1.27.0
**日期**: 2025-11-06
**功能**: Web 版本支持多轮对话上下文，重用 CLI 版本配置

---

## 功能概述

Web 终端现在支持多轮对话上下文，能够在对话中记住之前的交流内容，提供更智能的连续对话体验。

**核心特性**：
- ✅ 自动识别需要上下文的场景
- ✅ 保持对话历史（最近 9 轮）
- ✅ 智能上下文管理（5 分钟空闲自动清除）
- ✅ 复用 CLI 版本的所有配置参数

## 配置说明

### 启用方式

对话上下文功能通过 `realconsole.yaml` 配置，Web 和 CLI 版本**完全共享配置**：

```yaml
# 对话上下文：自动模式
conversation:
  mode: auto  # 智能识别需要上下文的场景
  # 保留最近 9 轮对话（适合短对话）
  max_turns: 9
  # 最大上下文长度 64K 字符（约 16K tokens）
  max_context_length: 64000
  # 自动清除策略
  auto_clear:
    enabled: true
    idle_timeout: 300      # 5 分钟空闲后清除
    on_task_complete: true # 任务完成后清除
  # 包含工具调用和错误信息
  include:
    tool_calls: true
    shell_output: false
    errors: true
```

### 配置参数说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `mode` | String | `auto` | 模式：`auto`（智能）/ `always`（始终）/ `never`（从不） |
| `max_turns` | Integer | `9` | 保留的最大对话轮数 |
| `max_context_length` | Integer | `64000` | 最大上下文字符数 |
| `auto_clear.enabled` | Boolean | `true` | 是否启用自动清除 |
| `auto_clear.idle_timeout` | Integer | `300` | 空闲超时时间（秒） |
| `auto_clear.on_task_complete` | Boolean | `true` | 任务完成后是否清除 |
| `include.tool_calls` | Boolean | `true` | 是否包含工具调用 |
| `include.shell_output` | Boolean | `false` | 是否包含 Shell 输出 |
| `include.errors` | Boolean | `true` | 是否包含错误信息 |

## 实现原理

### 架构设计

```
┌─────────────────────────────────────────┐
│  WebSocket Session                      │
│  ├─ session.id: WebSocket 会话 ID       │
│  └─ session.conversation_id: 对话 ID    │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  Agent (独立实例)                       │
│  └─ state_manager.conversation_context  │
│      └─ ContextManager                  │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│  LLM 请求处理                           │
│  1. should_use_context() 判断是否启用  │
│  2. build_messages() 构建历史消息      │
│  3. with_tools_and_context() 创建请求  │
│  4. add_turn() 记录本轮对话            │
└─────────────────────────────────────────┘
```

### 核心逻辑

#### 1. 判断是否使用上下文

```rust
let ctx_arc = agent.state_manager().conversation_context();
let mut ctx_manager = ctx_arc.write().await;

// 智能判断是否需要上下文
let should_use_context = ctx_manager.should_use_context(input);
```

**判断规则**（由 `ContextManager` 实现）：
- 检查 `conversation.mode` 配置
- `auto` 模式下，根据用户输入智能判断
- 例如："继续"、"刚才"、"上次" 等关键词会触发上下文

#### 2. 构建消息列表

```rust
let messages = if should_use_context {
    ctx_manager.build_messages(input)  // 包含历史 + 当前
} else {
    vec![Message::user(input)]  // 仅当前输入
};
```

**消息结构**（带上下文时）：
```
[
  { role: "user", content: "第1轮用户输入" },
  { role: "assistant", content: "第1轮助手回复" },
  { role: "user", content: "第2轮用户输入" },
  { role: "assistant", content: "第2轮助手回复" },
  ...
  { role: "user", content: "当前用户输入" }
]
```

#### 3. 创建 LLM 请求

```rust
let request = if should_use_context {
    LlmRequest::with_tools_and_context(messages.clone())
} else {
    LlmRequest::with_tools(input.to_string())
};
```

#### 4. 记录对话轮次

```rust
if should_use_context {
    let turn = Turn::new(
        input.to_string(),
        llm_response.text.clone(),
    );
    ctx_manager.add_turn(turn);
}
```

### 会话管理

#### Session 结构

```rust
pub struct Session {
    pub id: SessionId,                    // WebSocket 会话 ID
    pub agent: Arc<RwLock<Agent>>,        // 独立 Agent 实例
    pub conversation_id: String,          // 对话 ID (web-{uuid})
    pub created_at: DateTime<Utc>,
    pub llm_init_error: Option<String>,
}
```

**关键点**：
- 每个 WebSocket 连接有独立的 `Session`
- 每个 `Session` 有独立的 `Agent` 实例
- 每个 `Agent` 有独立的 `ContextManager`
- 因此每个浏览器标签页的对话上下文是**独立**的

## 使用场景

### 场景 1：连续提问

**用户**：
```
% 什么是 Rust？

% 它有什么特点？

% 给我举个例子
```

**效果**：
- 第1个问题：不使用上下文
- 第2个问题：自动识别"它"指代 Rust，使用上下文
- 第3个问题：知道是要 Rust 的例子，使用上下文

### 场景 2：代码审查

**用户**：
```
% 帮我写一个快速排序

% 这个算法的时间复杂度是多少？

% 能优化吗？
```

**效果**：
- LLM 记住了之前写的快速排序代码
- 可以针对性回答复杂度问题
- 可以基于之前的代码进行优化

### 场景 3：多步任务

**用户**：
```
% 创建一个 Node.js 项目

% 添加 Express 依赖

% 创建一个 Hello World 路由
```

**效果**：
- LLM 知道是在同一个项目上工作
- 理解"添加依赖"是指前面创建的项目
- "创建路由"也是针对同一个 Express 应用

## 技术细节

### 上下文长度管理

**限制机制**：
1. **轮数限制**：`max_turns: 9`（保留最近 9 轮对话）
2. **字符限制**：`max_context_length: 64000`（约 16K tokens）
3. **自动裁剪**：超过限制时，自动删除最旧的对话轮次

**Token 估算**：
- 英文：1 token ≈ 4 字符
- 中文：1 token ≈ 1.5 字符
- 64000 字符 ≈ 16K tokens（混合中英文）

### 自动清除策略

#### 空闲超时（Idle Timeout）

```yaml
auto_clear:
  enabled: true
  idle_timeout: 300  # 5 分钟
```

**行为**：
- 用户 5 分钟没有新输入
- 自动清除对话上下文
- 下次对话从新的上下文开始

#### 任务完成清除

```yaml
auto_clear:
  on_task_complete: true
```

**行为**：
- 检测到任务完成的信号
- 自动清除对话上下文
- 例如：用户说"好的，谢谢"、"完成了"等

### 性能优化

#### 锁管理

```rust
// 读取上下文（短时间持有锁）
let ctx_arc = agent.state_manager().conversation_context();
let mut ctx_manager = ctx_arc.write().await;
let messages = ctx_manager.build_messages(input);
drop(ctx_manager);  // ← 立即释放锁

// ... LLM 处理（不持有锁）...

// 写入上下文（短时间持有锁）
let mut ctx_manager = ctx_arc.write().await;
ctx_manager.add_turn(turn);
drop(ctx_manager);  // ← 立即释放锁
```

**优势**：
- 避免长时间持有锁
- LLM 调用期间不阻塞其他请求
- 支持并发 WebSocket 连接

## 与 CLI 版本对比

| 特性 | CLI 版本 | Web 版本 | 说明 |
|------|---------|---------|------|
| **配置文件** | ✅ 支持 | ✅ 支持 | 完全共享 `realconsole.yaml` |
| **上下文管理** | ✅ 全局 | ✅ 独立 | Web 每个 Session 独立 |
| **自动识别** | ✅ 支持 | ✅ 支持 | 使用相同的智能判断逻辑 |
| **轮数限制** | ✅ 支持 | ✅ 支持 | 参数完全相同 |
| **自动清除** | ✅ 支持 | ✅ 支持 | 参数完全相同 |
| **会话隔离** | ❌ 单进程 | ✅ 多会话 | Web 支持多标签页独立上下文 |

## 调试与诊断

### 查看上下文使用情况

**日志输出**（服务器端）：
```
📝 使用对话上下文 (3 轮)
📝 添加对话轮次 (总计 4 轮)
```

**未来改进**：
- 在 Web UI 显示上下文状态
- 提供 `/context` 命令查看当前上下文
- 提供 `/clear` 命令手动清除上下文

### 常见问题

**Q: 为什么有时候不使用上下文？**

A: `auto` 模式下，系统会智能判断。如果用户输入是全新的话题，不会使用上下文。可以设置 `mode: always` 强制始终使用。

**Q: 上下文什么时候会被清除？**

A:
1. 5 分钟空闲（可配置）
2. 任务完成后（可配置）
3. 浏览器刷新页面（WebSocket 重连，创建新 Session）

**Q: 多个浏览器标签页的上下文是否共享？**

A: 不共享。每个标签页有独立的 WebSocket 连接和独立的上下文。

**Q: 关闭标签页后上下文会保留吗？**

A: 不会。WebSocket 断开后，Session 被销毁，上下文丢失。

## 未来改进方向

### 1. 上下文持久化

**目标**：浏览器刷新后恢复上下文

**实现方案**：
- 在 Session 销毁前保存上下文到数据库/文件
- 使用 Cookie/LocalStorage 记录会话 ID
- 重连时恢复之前的上下文

### 2. 上下文可视化

**目标**：在 Web UI 显示上下文状态

**功能**：
```
┌─────────────────────────────┐
│ 对话上下文: 3/9 轮          │
│ 最后更新: 1 分钟前          │
│ [查看详情] [清除上下文]     │
└─────────────────────────────┘
```

### 3. 手动控制命令

**新增系统命令**：
- `/context` - 查看当前上下文
- `/context clear` - 清除上下文
- `/context mode <auto|always|never>` - 切换模式

### 4. 跨 Session 上下文

**目标**：多个标签页共享上下文

**实现方案**：
- 使用共享存储（Redis/数据库）
- 基于用户 ID 而非 Session ID 管理上下文
- 需要添加用户认证系统

## 修改文件

| 文件 | 修改内容 | 行数变化 |
|------|---------|---------|
| `src/web/session.rs` | 添加 `conversation_id` 字段 | +3 行 |
| `src/web/websocket.rs` | 实现上下文管理逻辑 | +30 行 |

**总计**：新增代码 ~33 行

## 测试建议

### 基础功能测试

1. **首次对话**（不使用上下文）
   ```
   % Rust 是什么？
   ```
   预期：正常回答

2. **连续对话**（自动使用上下文）
   ```
   % Rust 是什么？
   % 它有什么特点？
   ```
   预期：第二个问题知道"它"指 Rust

3. **全新话题**（自动切换上下文）
   ```
   % Rust 是什么？
   % Python 的特点是什么？
   ```
   预期：第二个问题不依赖第一个问题的上下文

### 配置测试

4. **强制使用上下文**
   ```yaml
   conversation:
     mode: always
   ```
   预期：每个问题都带上之前的对话历史

5. **禁用上下文**
   ```yaml
   conversation:
     mode: never
   ```
   预期：每个问题都是独立的，不使用历史

### 清除测试

6. **空闲超时清除**
   - 连续对话 2-3 轮
   - 等待 5 分钟
   - 再次提问
   - 预期：不记得之前的对话

7. **刷新页面清除**
   - 连续对话 2-3 轮
   - 刷新浏览器
   - 再次提问
   - 预期：不记得之前的对话

---

**功能完成** ✅
**配置复用**: 100%（CLI 配置完全适用）
**代码增量**: 最小化（仅 33 行）
**用户体验**: 🌟🌟🌟🌟🌟
