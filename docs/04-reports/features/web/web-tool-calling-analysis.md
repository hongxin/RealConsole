# Web 版本工具调用缺失问题分析和改进方案

> 📅 **日期**: 2025-11-04
> 🎯 **问题**: Web 版本缺少工具调用能力，无法感知现实世界
> 🔍 **分析**: 深度调研命令行版本的工具调用系统

---

## 一、问题现象

### 1.1 实际测试案例

**用户测试**：
```bash
% 请告诉我现在几点了

我无法获取您当前的准确时间，因为我无法访问实时数据。
建议您查看您的设备上的时钟...
```

**问题**：
- ❌ LLM 无法获取实时信息
- ❌ 缺少"感知现实世界"的能力
- ❌ 无法调用工具（datetime、read_file、calculator 等）

### 1.2 预期行为（命令行版本）

**命令行版本**：
```bash
% 请告诉我现在几点了

[LLM 调用 datetime 工具]
当前时间是：2025-11-04 15:30:45 CST
```

**能力**：
- ✅ LLM 可以调用 datetime 工具
- ✅ 工具返回实际时间
- ✅ LLM 整合工具结果给出回复
- ✅ 具备"感知现实世界"能力

---

## 二、根本原因分析

### 2.1 当前 Web 实现

**src/web/websocket.rs - execute_llm_chat()**：
```rust
async fn execute_llm_chat(
    input: &str,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let llm_manager = agent.llm_manager.read().await;

    // ❌ 问题：直接调用简单的 chat()
    match llm_manager.chat(input).await {
        Ok(response) => {
            send_message(response);
        }
        // ...
    }
}
```

**缺失的关键**：
1. ❌ 没有传递工具定义（tools）给 LLM
2. ❌ 没有处理 LLM 的 function_call 响应
3. ❌ 没有执行工具并返回结果给 LLM
4. ❌ 没有支持多轮对话（LLM → 工具 → LLM）

### 2.2 命令行版本实现

**src/agent.rs - handle_text_with_tools()**：
```rust
fn handle_text_with_tools(&self, ctx: &TraceContext, text: &str) -> String {
    // 1. 创建带工具的请求
    let request = if let Some(msgs) = messages {
        LlmRequest::with_tools_and_context(msgs)  // ✅ 关键
    } else {
        LlmRequest::with_tools(text.to_string())  // ✅ 关键
    };

    // 2. 调用 LlmService（内部处理工具调用）
    let result = self.llm_service().process(request).await;

    // 3. 解析工具调用信息
    if let Some(rounds) = ToolExecutor::decode_debug_info(&response) {
        // 显示工具调用详情
        for round in &rounds {
            for tool_call in &round.tool_calls {
                // 记录和显示工具调用
            }
        }
    }
}
```

**关键组件**：
1. ✅ `LlmRequest::with_tools()` - 包含工具定义
2. ✅ `LlmService` - 处理多轮工具调用
3. ✅ `ToolExecutor` - 执行工具并返回结果
4. ✅ 支持多轮对话直到 LLM 获取足够信息

---

## 三、命令行版本工具调用流程

### 3.1 完整工作流程

```
用户输入: "请告诉我现在几点了"
    ↓
1. Agent.handle_text_with_tools()
    ↓
2. LlmService.process(LlmRequest::with_tools)
    ↓
3. 发送请求给 LLM，包含工具定义（datetime, read_file, calculator...）
    ↓
4. LLM 分析：需要调用 datetime 工具
    ↓
5. LLM 返回: function_call { name: "datetime", arguments: {} }
    ↓
6. ToolExecutor.execute("datetime", {})
    ↓
7. datetime 工具返回: "2025-11-04 15:30:45 CST"
    ↓
8. 将工具结果添加到对话历史
    ↓
9. 再次调用 LLM（带工具结果）
    ↓
10. LLM 整合信息：现在是下午3:30...
    ↓
11. 返回最终回复给用户
```

### 3.2 核心组件

**1. Tool 系统** (`src/tool.rs`):
```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub handler: Arc<dyn Fn(JsonValue) -> Result<String, String>>,
}

// 转换为 OpenAI Function Schema
pub fn to_function_schema(&self) -> JsonValue {
    json!({
        "type": "function",
        "function": {
            "name": self.name,
            "description": self.description,
            "parameters": { /* ... */ }
        }
    })
}
```

**2. ToolRegistry** (`src/tool.rs`):
```rust
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

// 获取所有工具的 Function Schema
pub fn get_function_schemas(&self) -> Vec<JsonValue> {
    self.tools.values()
        .map(|tool| tool.to_function_schema())
        .collect()
}
```

**3. ToolExecutor** (`src/tool_executor.rs`):
```rust
impl ToolExecutor {
    // 执行工具调用
    pub fn execute(&self, name: &str, args: JsonValue) -> Result<String, String> {
        let registry = self.registry.read().unwrap();
        let tool = registry.get(name)?;
        tool.execute(args)
    }
}
```

**4. LlmService** (`src/services/llm_service.rs`):
```rust
impl LlmService {
    async fn process(&self, request: LlmRequest) -> Result<LlmResponse> {
        if request.use_tools {
            // 多轮工具调用循环
            loop {
                let response = llm.chat_with_tools(&messages, &tools).await?;

                if let Some(function_call) = response.function_call {
                    // 执行工具
                    let result = tool_executor.execute(&function_call.name, &function_call.args)?;

                    // 添加工具结果到历史
                    messages.push(tool_result_message);

                    // 继续循环
                } else {
                    // LLM 完成推理，返回最终结果
                    return Ok(response);
                }
            }
        } else {
            // 简单对话
            llm.chat(&messages).await
        }
    }
}
```

### 3.3 内置工具列表

**当前可用工具** (`src/builtin_tools.rs`):
1. **calculator** - 数学计算
   ```
   用途：执行数学表达式
   示例：2+2, sqrt(16), sin(pi/2)
   ```

2. **read_file** - 读取文件
   ```
   用途：读取文件内容
   参数：path (文件路径)
   限制：禁止敏感文件，最多1000字符
   ```

3. **write_file** - 写入文件
   ```
   用途：写入内容到文件
   参数：path, content
   限制：禁止系统目录
   ```

4. **list_dir** - 列出目录
   ```
   用途：列出目录内容
   参数：path (目录路径)
   ```

5. **datetime** - 日期时间
   ```
   用途：获取当前日期时间
   返回：2025-11-04 15:30:45 CST
   ```

6. **lunar** - 农历查询
   ```
   用途：公历/农历转换、节气、干支
   参数：year, month, day
   ```

7. **shell_execute** - Shell 执行
   ```
   用途：执行 Shell 命令
   参数：command
   安全：需要用户确认
   ```

---

## 四、Web 版本改进方案

### 4.1 方案对比

#### 方案 A：简单集成（推荐）⭐

**实现**：
```rust
async fn execute_llm_chat_with_tools(
    input: &str,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 1. 调用 Agent 的现有方法
    let response = tokio::task::spawn_blocking({
        let input = input.to_string();
        let agent_clone = /* clone agent */;
        move || {
            // 使用 Agent 的 handle_text_with_tools
            agent_clone.handle_text_with_tools(&ctx, &input)
        }
    }).await?;

    // 2. 发送响应
    send_output(response, sender).await?;

    Ok(())
}
```

**优势**：
- ✅ 100% 复用现有逻辑
- ✅ 最小化修改（~20 行）
- ✅ 自动支持所有工具
- ✅ 自动支持多轮调用
- ✅ 符合"充分复用"原则

**劣势**：
- ⚠️ 需要处理同步/异步转换
- ⚠️ Agent 的 clone 可能需要 Arc

#### 方案 B：完全重写（不推荐）

**实现**：
```rust
// 在 Web 中重新实现完整的工具调用逻辑
// - 获取工具列表
// - 构建 Function Schema
// - 处理 function_call
// - 执行工具
// - 多轮对话循环
```

**优势**：
- ✅ 完全控制流程
- ✅ 可自定义 Web 特定行为

**劣势**：
- ❌ 大量重复代码（200+ 行）
- ❌ 违背"不重复造轮子"原则
- ❌ 维护成本高
- ❌ 容易不同步

### 4.2 推荐方案详细设计

**核心思路**：复用 Agent 的服务层

**修改点 1**: 创建辅助方法（src/web/websocket.rs）:
```rust
/// 执行 LLM 对话（带工具调用）
async fn execute_llm_chat_with_tools(
    input: &str,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 发送 Thinking 消息
    send_thinking_message(sender).await?;

    // 使用 Agent 的服务层
    let request = crate::services::LlmRequest::with_tools(input.to_string());

    let response = {
        let llm_service = agent.llm_service();
        llm_service.process(request).await?
    };

    // 发送响应
    send_output_message(&response.text, sender).await?;

    Ok(())
}
```

**修改点 2**: 修改路由逻辑：
```rust
CommandType::NaturalLanguage(text) => {
    // 先尝试 Intent 匹配
    if let Some(intent_match) = try_match_intent(&text, &agent) {
        execute_intent(&intent_match, &text, &agent, sender).await
    } else {
        // ✅ 使用带工具的 LLM 调用
        execute_llm_chat_with_tools(&text, &agent, sender).await
    }
}
```

**修改点 3**: 确保配置启用工具调用（src/web/session.rs）:
```rust
impl Session {
    pub async fn new(config: Config, registry: CommandRegistry) -> Self {
        // ...

        // ✅ 启用工具调用
        let mut config = config;
        config.features.tool_calling_enabled = Some(true);

        let mut agent = Agent::new(config.clone(), registry);
        // ...
    }
}
```

### 4.3 实现步骤

**Step 1**: 研究 Agent 服务层 API ✅ (已完成)

**Step 2**: 修改 Session 配置
- 启用 tool_calling_enabled
- 确保 LLM 配置正确

**Step 3**: 修改 websocket.rs
- 添加 execute_llm_chat_with_tools()
- 修改路由逻辑调用新函数
- 处理工具调用的显示（可选）

**Step 4**: 测试验证
- 测试："请告诉我现在几点了"
- 测试："计算 123 + 456"
- 测试："读取 README.md 文件"

---

## 五、预期效果

### 5.1 改进前后对比

**改进前**：
```bash
% 请告诉我现在几点了
❌ 我无法获取实时数据

% 计算 123 + 456
❌ LLM 直接计算（可能不准确）

% 读取 package.json
❌ 我无法访问文件系统
```

**改进后**：
```bash
% 请告诉我现在几点了
✅ [调用 datetime 工具]
✅ 当前时间是：2025-11-04 15:30:45 CST

% 计算 123 + 456
✅ [调用 calculator 工具]
✅ 123 + 456 = 579

% 读取 package.json
✅ [调用 read_file 工具]
✅ {
✅   "name": "my-project",
✅   "version": "1.0.0",
✅   ...
✅ }
```

### 5.2 新增能力

**实时信息获取**：
- ✅ 当前日期时间
- ✅ 文件系统访问
- ✅ 目录列表

**数学计算**：
- ✅ 复杂表达式
- ✅ 精确计算
- ✅ 三角函数、开方等

**文件操作**：
- ✅ 读取文件内容
- ✅ 写入文件
- ✅ 列出目录

**Shell 执行**：
- ✅ 执行系统命令
- ✅ 获取命令输出

**农历查询**：
- ✅ 公历转农历
- ✅ 节气查询
- ✅ 干支生肖

---

## 六、实施计划

### 6.1 Phase 1: 核心功能（优先）

**任务**：
1. [ ] 修改 Session 启用 tool_calling
2. [ ] 添加 execute_llm_chat_with_tools()
3. [ ] 修改路由逻辑
4. [ ] 基础测试

**预计时间**: 1-2 小时

### 6.2 Phase 2: 优化显示（可选）

**任务**：
1. [ ] 显示工具调用过程
2. [ ] 添加工具调用消息类型
3. [ ] 前端显示优化

**预计时间**: 1-2 小时

### 6.3 Phase 3: 完善文档（必须）

**任务**：
1. [ ] 更新用户指南
2. [ ] 添加工具调用示例
3. [ ] 更新测试报告

**预计时间**: 0.5-1 小时

---

## 七、关键代码参考

### 7.1 LlmRequest 构造

**src/services/llm_service.rs**:
```rust
pub struct LlmRequest {
    pub text: Option<String>,
    pub messages: Option<Vec<Message>>,
    pub use_tools: bool,  // ✅ 关键标志
    pub stream: bool,
}

impl LlmRequest {
    pub fn with_tools(text: String) -> Self {
        Self {
            text: Some(text),
            messages: None,
            use_tools: true,  // ✅ 启用工具
            stream: false,
        }
    }
}
```

### 7.2 Agent 服务访问

**src/agent.rs**:
```rust
impl Agent {
    // 获取服务
    pub fn llm_service(&self) -> Arc<LlmService> {
        Arc::clone(&self.llm_service)
    }

    pub fn state_manager(&self) -> Arc<StateManager> {
        Arc::clone(&self.state_manager)
    }
}
```

### 7.3 配置启用

**realconsole.yaml**:
```yaml
features:
  tool_calling_enabled: true  # ✅ 启用工具调用
```

---

## 八、风险和限制

### 8.1 潜在风险

**性能**：
- ⚠️ 工具调用需要多轮 LLM 请求
- ⚠️ 响应时间可能较长（2-5秒）
- 缓解：显示进度、Thinking 动画

**安全**：
- ⚠️ shell_execute 工具有安全风险
- ⚠️ 文件操作需要权限控制
- 缓解：保留现有安全检查

**复杂度**：
- ⚠️ 调试难度增加
- ⚠️ 错误处理更复杂
- 缓解：详细日志、错误提示

### 8.2 已有保护

**命令行版本的安全措施**：
```rust
// 文件读取限制
let dangerous_patterns = ["/etc/shadow", "/etc/passwd", ".ssh/id_rsa"];

// 文件写入限制
if path.starts_with("/etc/") || path.starts_with("/sys/") {
    return Err("禁止写入系统目录");
}

// Shell 执行（需要用户确认）
shell_execute 工具标记为需要确认
```

---

## 九、总结

### 9.1 核心问题

**当前状态**：
- ❌ Web 版本缺少工具调用能力
- ❌ LLM 无法感知现实世界
- ❌ 无法获取实时信息

**根本原因**：
- 使用简单的 `llm_manager.chat()`
- 没有传递工具定义
- 没有处理 function_call

### 9.2 解决方案

**推荐方案**：
- ✅ 复用 Agent 的 LlmService
- ✅ 启用 tool_calling_enabled
- ✅ 最小化修改（~30 行）
- ✅ 100% 复用现有逻辑

**预期效果**：
- ✅ 支持所有 7+ 个内置工具
- ✅ 自动多轮工具调用
- ✅ "感知现实世界"能力
- ✅ 与命令行版本一致的体验

### 9.3 下一步

1. **立即实施** Phase 1 核心功能
2. **验证测试** 关键场景
3. **优化显示** 工具调用过程（可选）
4. **更新文档** 完善说明

---

**最后更新**: 2025-11-04
**状态**: 📋 分析完成，待实施
**优先级**: 🔥 高（核心能力缺失）
