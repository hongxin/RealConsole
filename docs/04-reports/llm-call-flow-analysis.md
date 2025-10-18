# RealConsole LLM 调用流程深度分析

**分析日期**: 2025-10-18
**分析对象**: BNB 投资分析案例
**目标**: 理解 LLM 工具调用的完整流程，为套路化复用做准备

---

## 目录

1. [案例回顾](#案例回顾)
2. [完整流程图](#完整流程图)
3. [详细流程分析](#详细流程分析)
4. [核心代码定位](#核心代码定位)
5. [套路化复用方案](#套路化复用方案)
6. [性能优化建议](#性能优化建议)

---

## 案例回顾

### 用户输入
```bash
./target/release/realconsole --once "请帮我访问**非小号网站**的数据，分析目前BNB的走势和投资策略"
```

### 系统输出特点
1. 成功获取了非小号网站的 BNB 数据
2. 进行了全面的投资分析（技术指标、基本面、风险提示）
3. 输出结构化、专业化
4. 整个过程自动完成，无需人工干预

---

## 完整流程图

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 命令行入口                                               │
│    main.rs:387 - if let Some(input) = args.once            │
│    repl.rs:153 - run_once(&agent, &input)                  │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Agent 核心调度                                           │
│    agent.rs:326 - handle(line: &str)                       │
│    ├─ 记录用户输入到 Memory (agent.rs:338-343)             │
│    ├─ 提取实体到 ContextTracker (agent.rs:345-354)         │
│    └─ 使用 CommandRouter 识别命令类型 (agent.rs:357)       │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. 命令类型路由                                             │
│    agent.rs:359-381 - match router_result                  │
│    ├─ CommonShell: 常见 Shell 命令                         │
│    ├─ ForcedShell: ! 前缀强制 Shell                        │
│    ├─ SystemCommand: / 前缀系统命令                        │
│    └─ NaturalLanguage: 自然语言 → handle_text()            │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. 自然语言处理                                             │
│    agent.rs:749 - handle_text(&text)                       │
│    ├─ 检查是否有活跃对话 (agent.rs:754)                    │
│    ├─ 尝试启动新对话 (agent.rs:758)                        │
│    ├─ 检查工具调用是否启用 (agent.rs:763)                  │
│    │  └─ use_tools = config.features.tool_calling_enabled  │
│    ├─ 如果启用 → handle_text_with_tools() [重点！]         │
│    ├─ 否则尝试 Intent DSL 匹配 (agent.rs:771)              │
│    └─ 最后回退到流式输出 (agent.rs:776)                    │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. 工具调用模式 [核心流程]                                  │
│    agent.rs:779-831 - handle_text_with_tools()             │
│    ├─ 启动 Spinner (agent.rs:782)                          │
│    ├─ 获取 LLM 客户端 (agent.rs:787-791)                   │
│    ├─ 获取工具 Schemas (agent.rs:794-795)                  │
│    │  └─ registry.get_function_schemas()                   │
│    └─ 调用迭代工具执行引擎 (agent.rs:808-810)              │
│       └─ tool_executor.execute_iterative(llm, text, tools) │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. 迭代工具链执行 [最关键的部分！]                          │
│    tool_executor.rs:219-288 - execute_iterative()          │
│    ┌───────────────────────────────────────────────┐       │
│    │ 循环流程（最多 max_iterations 轮，默认 5）    │       │
│    ├───────────────────────────────────────────────┤       │
│    │ Round 1:                                      │       │
│    │   ├─ 发送用户消息 + 工具 Schema 到 LLM       │       │
│    │   │  └─ llm.chat_with_tools(messages, tools) │       │
│    │   ├─ LLM 返回工具调用请求                    │       │
│    │   │  └─ response.tool_calls (可能多个)       │       │
│    │   ├─ 执行工具调用 (最多 3 个/轮)             │       │
│    │   │  └─ execute_tool_calls(&tool_requests)   │       │
│    │   ├─ 将工具结果添加到消息历史                │       │
│    │   └─ 继续下一轮                              │       │
│    │                                               │       │
│    │ Round 2+:                                     │       │
│    │   ├─ 再次发送完整对话历史到 LLM              │       │
│    │   ├─ LLM 可能继续调用工具或返回最终答案      │       │
│    │   └─ 直到 is_final=true 或达到最大轮数       │       │
│    └───────────────────────────────────────────────┘       │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. 工具并行执行                                             │
│    tool_executor.rs:179-209 - execute_tool_calls()         │
│    ├─ 限制单轮工具数量 (max_tools_per_round=3)             │
│    ├─ 根据执行模式选择：                                    │
│    │  ├─ Sequential: 串行执行（保持顺序）                   │
│    │  └─ Parallel: 并行执行（性能优化，默认） [✨]          │
│    └─ 返回所有工具结果                                      │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 8. 单个工具执行                                             │
│    tool_executor.rs:128-175 - execute_tool_call()          │
│    ├─ 检查缓存 (如果启用) (agent.rs:135-144)               │
│    ├─ 从工具注册表获取工具                                  │
│    ├─ 执行工具 handler                                      │
│    │  └─ registry.execute(tool_name, arguments)            │
│    ├─ 写入缓存 (成功时) (agent.rs:153-155)                 │
│    └─ 返回 ToolCallResult (包含耗时统计)                    │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 9. HTTP GET 工具执行 [案例中的关键工具]                     │
│    advanced_tools.rs:38-113 - http_get tool                │
│    ├─ 解析参数 (url, timeout)                              │
│    ├─ 安全检查：URL 必须是 http/https                       │
│    ├─ 超时限制：1-60 秒                                     │
│    ├─ 使用 reqwest 发送 GET 请求 (agent.rs:75-110)         │
│    │  ├─ 创建 HTTP 客户端                                   │
│    │  ├─ 发送请求                                           │
│    │  ├─ 检查状态码                                         │
│    │  ├─ 读取响应体 (限制 10MB)                             │
│    │  └─ 转换为字符串                                       │
│    └─ 返回网站内容                                          │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 10. LLM 综合分析                                            │
│     - LLM 接收到网站数据后                                  │
│     - 分析 BNB 价格、市值、涨跌幅等数据                     │
│     - 生成投资策略和风险提示                                │
│     - 返回最终文本响应 (is_final=true)                      │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 11. 记录与输出                                              │
│     agent.rs:388-475 - 记录响应和执行日志                  │
│     ├─ 记录到 ExecutionLogger (agent.rs:397-406)           │
│     ├─ 记录到 HistoryManager (agent.rs:409-412)            │
│     ├─ 记录到 StatsCollector (agent.rs:415-423)            │
│     ├─ 记录到 Memory (带截断，最多200字符) (agent.rs:426)  │
│     ├─ 自动保存到文件 (如果启用) (agent.rs:442-453)        │
│     ├─ 更新 ContextTracker (agent.rs:456-472)              │
│     └─ 返回响应文本 (agent.rs:477)                         │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────────┐
│ 12. 用户看到输出                                            │
│     repl.rs:154-157 - 打印响应                             │
│     └─ println!("{}", response)                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 详细流程分析

### 阶段 1: 命令行参数解析
**位置**: `src/main.rs:387-389`

```rust
if let Some(input) = args.once {
    // 单次执行模式
    repl::run_once(&agent, &input);
}
```

**关键点**:
- `--once` 参数允许单次命令执行，不进入 REPL 循环
- 直接调用 `run_once()` 函数处理输入

---

### 阶段 2: Agent 核心调度
**位置**: `src/agent.rs:326-478`

```rust
pub fn handle(&self, line: &str) -> String {
    // 1. 记录用户输入到 Memory
    let mut memory = self.memory.write().await;
    memory.add(line.to_string(), EntryType::User);

    // 2. 提取实体并更新上下文追踪器
    let mut tracker = self.context_tracker.write().await;
    let entities = tracker.extract_entities(line);

    // 3. 使用智能命令路由器识别命令类型
    let router_result = self.command_router.route(line);

    // 4. 根据类型分发处理
    match router_result {
        RouterCommandType::NaturalLanguage(text) => {
            self.handle_text(&text)  // 关键！
        }
        // ... 其他类型
    }
}
```

**关键决策点**:
- CommandRouter 识别这是自然语言输入
- 不是 Shell 命令（没有 ! 前缀）
- 不是系统命令（没有 / 前缀）
- 进入自然语言处理流程

---

### 阶段 3: 自然语言处理决策
**位置**: `src/agent.rs:749-777`

```rust
fn handle_text(&self, text: &str) -> String {
    // 一分为三的决策逻辑（东方哲学）

    // 1️⃣ 对话态：检查是否有活跃对话
    if has_active_conversation() {
        return self.handle_conversation_input(text);
    }

    // 2️⃣ 检查是否需要启动新对话
    if let Some(response) = self.try_start_conversation(text) {
        return response;
    }

    // 3️⃣ 工具调用态：检查工具调用是否启用
    let use_tools = self.config.features.tool_calling_enabled.unwrap_or(false);

    if use_tools {
        return self.handle_text_with_tools(text);  // ✨ 进入这里！
    }

    // 4️⃣ 意图态：Intent DSL 匹配
    if let Some(plan) = self.try_match_intent(text) {
        return self.execute_intent(&plan);
    }

    // 5️⃣ 最后回退：流式 LLM 输出
    self.handle_text_streaming(text)
}
```

**关键配置**:
```yaml
# realconsole.yaml
features:
  tool_calling_enabled: true  # 启用工具调用！
```

---

### 阶段 4: 工具调用模式 [核心]
**位置**: `src/agent.rs:779-831`

```rust
fn handle_text_with_tools(&self, text: &str) -> String {
    let spinner = Spinner::new();  // 显示 Loading 动画

    // 1. 获取 LLM 客户端
    let manager = self.llm_manager.read().await;
    let llm = manager.primary().or(manager.fallback())
        .ok_or_else(|| "未配置 LLM 客户端")?;

    // 2. 获取所有可用工具的 Schema
    let registry = self.tool_registry.read().await;
    let tool_schemas = registry.get_function_schemas();

    // 3. 执行迭代工具链（最重要！）
    self.tool_executor
        .execute_iterative(llm.as_ref(), text, tool_schemas)
        .await
}
```

**工具 Schemas 示例**:
```json
[
  {
    "type": "function",
    "function": {
      "name": "http_get",
      "description": "发送 HTTP GET 请求获取数据",
      "parameters": {
        "type": "object",
        "properties": {
          "url": {
            "type": "string",
            "description": "目标 URL（http 或 https）"
          },
          "timeout": {
            "type": "number",
            "description": "超时时间（秒），默认 30，最大 60"
          }
        },
        "required": ["url"]
      }
    }
  },
  // ... 其他工具（calculator, read_file, json_parse, 等）
]
```

---

### 阶段 5: 迭代工具链执行 [最关键！]
**位置**: `src/tool_executor.rs:219-288`

```rust
pub async fn execute_iterative(
    &self,
    llm: &dyn LlmClient,
    initial_message: &str,
    tool_schemas: Vec<JsonValue>,
) -> Result<String, String> {
    let mut messages = vec![Message::user(initial_message)];
    let mut iteration = 0;

    loop {
        iteration += 1;

        // 检查迭代次数限制（最多 5 轮）
        if iteration > self.max_iterations {
            return Err("达到最大迭代次数, 工具调用可能陷入循环");
        }

        // 调用 LLM with tools
        let response = llm
            .chat_with_tools(messages.clone(), tool_schemas.clone())
            .await?;

        // 如果是最终响应（没有工具调用），返回结果
        if response.is_final {
            return Ok(response.content.unwrap_or_default());
        }

        // 有工具调用，需要执行
        let tool_requests = convert_to_tool_requests(&response.tool_calls);

        // 执行工具（可能并行）
        let tool_results = self.execute_tool_calls(&tool_requests).await;

        // 将助手的工具调用添加到消息历史
        messages.push(Message::assistant_with_tools(response.tool_calls));

        // 将工具结果添加到消息历史
        for result in tool_results {
            messages.push(Message::tool_result(result.call_id, result.content));
        }

        // 继续下一轮迭代
    }
}
```

**实际案例中的迭代过程**:

```
Round 1:
  User: "请帮我访问**非小号网站**的数据，分析目前BNB的走势和投资策略"

  LLM 决策: "需要先获取网站数据"
  Tool Call: http_get(url="https://www.feixiaohao.co/...")

  Tool Result: "<html>... BNB 数据 ...</html>"

Round 2:
  LLM 分析: 收到网站数据，进行综合分析
  Final Response: "基于我从非小号网站获取的BNB数据，我来为您分析..."
  is_final: true
```

---

### 阶段 6: 工具并行执行
**位置**: `src/tool_executor.rs:179-209`

```rust
pub async fn execute_tool_calls(
    &self,
    calls: &[ToolCallRequest],
) -> Vec<ToolCallResult> {
    // 限制单轮工具数量（最多 3 个）
    let limited_calls = if calls.len() > self.max_tools_per_round {
        &calls[..self.max_tools_per_round]
    } else {
        calls
    };

    match self.execution_mode {
        ExecutionMode::Sequential => {
            // 串行执行
            let mut results = Vec::new();
            for call in limited_calls {
                results.push(self.execute_tool_call(call).await);
            }
            results
        }
        ExecutionMode::Parallel => {
            // ✨ 并行执行（默认，性能更好）
            let futures: Vec<_> = limited_calls
                .iter()
                .map(|call| self.execute_tool_call(call))
                .collect();

            futures::future::join_all(futures).await
        }
    }
}
```

**性能优势**:
- 如果 LLM 同时调用多个工具，并行执行可大幅提升速度
- 例如：同时调用 `http_get`、`calculator`、`json_parse` 三个工具
- 并行模式：总耗时 = max(t1, t2, t3)
- 串行模式：总耗时 = t1 + t2 + t3

---

### 阶段 7: HTTP GET 工具执行
**位置**: `src/advanced_tools.rs:38-113`

```rust
fn create_http_get_tool() -> Tool {
    Tool::new(
        "http_get",
        "发送 HTTP GET 请求获取数据",
        vec![/* 参数定义 */],
        |args: JsonValue| -> Result<String, String> {
            let url = args["url"].as_str().ok_or("缺少参数 'url'")?;

            // 安全检查
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("URL 必须以 http:// 或 https:// 开头");
            }

            // 超时限制
            let timeout = args["timeout"]
                .as_f64()
                .unwrap_or(30.0)
                .clamp(1.0, 60.0);

            // 异步 HTTP 请求
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(timeout as u64))
                        .build()?;

                    let response = client.get(url).send().await?;

                    // 状态码检查
                    if !response.status().is_success() {
                        return Err(format!("HTTP 错误: {}", response.status()));
                    }

                    // 读取响应（限制 10MB）
                    let bytes = response.bytes().await?;
                    if bytes.len() > 10 * 1024 * 1024 {
                        return Err("响应内容超过 10MB 限制");
                    }

                    // 转换为字符串
                    Ok(String::from_utf8_lossy(&bytes).to_string())
                })
            })
        },
    )
}
```

**安全特性**:
1. URL 协议检查（只允许 http/https）
2. 超时限制（1-60 秒）
3. 响应大小限制（10MB）
4. 错误处理和状态码检查

---

### 阶段 8: 记录与输出
**位置**: `src/agent.rs:388-475`

```rust
// 记录响应和执行日志
if !response.is_empty() {
    // 1. 判断是否成功
    let success = !response.contains("错误")
        && !response.contains("失败")
        && !response.contains("error");

    // 2. 记录到执行日志
    let mut logger = self.exec_logger.write().await;
    logger.log(line, command_type, success, duration, &response);

    // 3. 记录到命令历史
    let mut history = self.history.write().await;
    history.add(line, success);

    // 4. 记录到统计收集器
    self.stats_collector.record(StatEvent::CommandExecution {
        command: line.to_string(),
        success,
        duration,
    }).await;

    // 5. 记录到 Memory（截断到 200 字符）
    let mut memory = self.memory.write().await;
    let content = if response.len() > 200 {
        format!("{}...", &response[..200])
    } else {
        response.clone()
    };
    memory.add(content, EntryType::Assistant);

    // 6. 自动保存到文件（如果启用）
    if config.memory.auto_save {
        Memory::append_to_file(&path, entry);
    }

    // 7. 更新上下文追踪器
    let mut tracker = self.context_tracker.write().await;
    tracker.update_working_context(WorkingContextUpdate::LastCommand(line));
}
```

---

## 核心代码定位

### 关键文件清单

| 文件路径 | 核心功能 | 重点方法 |
|---------|---------|---------|
| `src/main.rs` | 程序入口、参数解析 | `main()` (L218-397) |
| `src/repl.rs` | REPL 循环、单次执行 | `run_once()` (L153-158) |
| `src/agent.rs` | Agent 核心调度逻辑 | `handle()` (L326-478)<br>`handle_text()` (L749-777)<br>`handle_text_with_tools()` (L779-831) |
| `src/tool_executor.rs` | 工具执行引擎 | `execute_iterative()` (L219-288)<br>`execute_tool_calls()` (L179-209)<br>`execute_tool_call()` (L128-175) |
| `src/builtin_tools.rs` | 内置工具（计算器、文件操作等） | `register_builtin_tools()` (L14-21) |
| `src/advanced_tools.rs` | 高级工具（HTTP、JSON等） | `create_http_get_tool()` (L38-113)<br>`create_http_post_tool()` (L116-209)<br>`create_json_parse_tool()` (L216-264) |
| `src/tool.rs` | 工具注册表 | `ToolRegistry::get_function_schemas()` |
| `src/llm/mod.rs` | LLM 客户端 trait | `chat_with_tools()` |
| `src/command_router.rs` | 智能命令路由 | `route()` |

---

## 套路化复用方案

### 模式 1: 网站数据分析套路

**适用场景**: 获取网站数据 + AI 分析

**配置要求**:
```yaml
features:
  tool_calling_enabled: true
  max_tool_iterations: 5
  max_tools_per_round: 3
```

**命令模板**:
```bash
./target/release/realconsole --once "请帮我访问 {网站} 的数据，分析 {主题}"
```

**实际案例**:
```bash
# 加密货币分析
./target/release/realconsole --once "请帮我访问非小号网站的数据，分析目前BNB的走势和投资策略"

# 股票分析
./target/release/realconsole --once "请帮我访问东方财富网的数据，分析茅台股票的投资价值"

# 天气分析
./target/release/realconsole --once "请帮我访问中国天气网的数据，分析北京未来一周的天气趋势"
```

**套路流程**:
1. LLM 识别需要获取网站数据
2. 调用 `http_get` 工具获取网页内容
3. LLM 分析网页内容，提取关键信息
4. 可能调用 `json_parse` 或 `text_search` 辅助处理
5. LLM 生成综合分析报告

---

### 模式 2: 数据处理 + 计算套路

**适用场景**: 复杂数据处理 + 数学计算

**命令模板**:
```bash
./target/release/realconsole --once "请帮我 {数据处理任务} 并计算 {指标}"
```

**实际案例**:
```bash
# 文件数据分析
./target/release/realconsole --once "请帮我读取 data.json，解析其中的销售数据，并计算总销售额和平均值"

# 代码统计分析
./target/release/realconsole --once "请帮我统计 src 目录下的 Rust 代码行数，并计算平均每个文件的行数"
```

**套路流程**:
1. LLM 识别需要读取文件或目录
2. 调用 `read_file` 或 `count_code_lines` 工具
3. 调用 `json_parse` 解析数据（如果是 JSON）
4. 调用 `calculator` 进行数学计算
5. LLM 生成分析报告

---

### 模式 3: Shell 命令 + 分析套路

**适用场景**: 执行系统命令 + 结果分析

**命令模板**:
```bash
./target/release/realconsole --once "请帮我 {执行命令} 并分析 {指标}"
```

**实际案例**:
```bash
# 磁盘分析
./target/release/realconsole --once "请帮我检查当前目录的磁盘占用情况，并分析哪些目录占用最大"

# 进程分析
./target/release/realconsole --once "请帮我查看系统进程，并分析哪些进程占用 CPU 最高"

# 网络分析
./target/release/realconsole --once "请帮我测试网络连接，并分析网络延迟情况"
```

**套路流程**:
1. LLM 识别需要执行 Shell 命令
2. 调用 `shell_execute` 工具（安全受限）
3. LLM 分析命令输出
4. 可能调用 `text_search` 或 `text_split` 辅助处理
5. LLM 生成分析报告

---

### 通用套路总结

**核心要素**:
1. **明确的任务描述**: "请帮我访问...分析..."
2. **工具链自动组合**: LLM 根据需求自动选择工具
3. **迭代式执行**: 最多 5 轮，每轮最多 3 个工具
4. **智能分析**: LLM 综合所有工具结果生成报告

**成功关键**:
1. ✅ 启用 `tool_calling_enabled`
2. ✅ 配置合适的 LLM（支持 Function Calling）
3. ✅ 注册足够的工具（内置 + 高级）
4. ✅ 清晰的用户意图描述

---

## 性能优化建议

### 1. 并行工具执行（已实现）
**位置**: `src/tool_executor.rs:199-207`

```rust
// 默认使用并行模式
ExecutionMode::Parallel
```

**效果**:
- 多工具同时执行，总耗时 = max(t1, t2, t3)
- 适合独立工具（http_get + calculator + json_parse）

**建议**:
- 保持默认并行模式
- 仅在工具有依赖关系时使用串行模式

---

### 2. 工具响应缓存（已支持）
**位置**: `src/tool_executor.rs:134-144`

```rust
// 尝试从缓存获取
if let Some(cache) = &self.cache {
    if let Some(cached_content) = cache.get(&call.name, &call.arguments).await {
        return cached_content;  // 缓存命中，秒级响应
    }
}
```

**启用方式**:
```rust
let cache = Arc::new(ToolCache::new(100));  // 缓存 100 条
let executor = ToolExecutor::new(registry, 5, 3)
    .with_cache(cache);
```

**效果**:
- 相同参数的工具调用直接返回缓存
- 适合重复查询场景（如多次访问同一网站）

---

### 3. 迭代次数优化
**当前配置**:
```rust
max_iterations: 5       // 最多 5 轮
max_tools_per_round: 3  // 每轮最多 3 个工具
```

**建议**:
- 简单任务：`max_iterations = 3`
- 复杂任务：`max_iterations = 5-7`
- 避免过多迭代导致 LLM 成本增加

---

### 4. LLM 提示词优化
**优化方向**:
1. **明确工具使用场景**: 在 System Prompt 中说明工具最佳实践
2. **减少无效工具调用**: 引导 LLM 一次性调用所需工具
3. **优化工具描述**: 更清晰的 `description` 字段

**示例**:
```rust
Tool::new(
    "http_get",
    "发送 HTTP GET 请求获取网页内容。适用于：分析网站数据、获取 API 响应、下载页面。不适用于：需要认证的 API、大文件下载。",
    // ...
)
```

---

### 5. 流式输出优化
**当前**:
- 工具调用模式：批量返回（spinner 动画）
- 流式模式：实时输出（逐 token 显示）

**建议**:
- 混合模式：工具执行显示进度，分析结果流式输出
- 实现方式：在 `tool_executor` 中添加进度回调

---

## 扩展建议

### 1. 新增专用工具
**建议工具**:
- `web_scraper`: 专门的网页抓取工具（支持 CSS 选择器）
- `api_client`: RESTful API 调用工具（支持认证）
- `data_analyzer`: 数据分析工具（统计、可视化）
- `markdown_render`: Markdown 渲染工具（美化输出）

---

### 2. 工具链预设
**概念**: 预定义常用工具组合

```rust
pub enum ToolChain {
    WebAnalysis,      // http_get + json_parse + text_search
    DataProcessing,   // read_file + json_parse + calculator
    SystemMonitor,    // shell_execute + text_search + get_system_info
}
```

**使用**:
```bash
./target/release/realconsole --chain web-analysis --once "分析 BNB 走势"
```

---

### 3. LLM 模型适配
**当前**: Deepseek 支持 Function Calling

**建议扩展**:
- OpenAI GPT-4 (Function Calling)
- Claude 3.5 Sonnet (Tool Use)
- Qwen (Function Calling)

**适配要点**:
```rust
// 不同模型的工具调用格式可能不同
trait LlmClient {
    async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolSchema>,
    ) -> Result<ToolResponse>;
}
```

---

## 总结

### 核心流程精髓
1. **三层决策**: 对话态 → 工具态 → 意图态 → 流式态
2. **迭代执行**: LLM + Tools 形成闭环，最多 5 轮
3. **并行优化**: 多工具并行执行，提升性能
4. **安全保障**: 工具权限限制、参数校验、资源限制

### 套路化关键
1. **明确任务**: "访问网站 + 分析数据"
2. **工具自选**: LLM 自动选择合适工具
3. **迭代分析**: 获取数据 → 分析 → 生成报告
4. **记录输出**: 执行日志、历史记录、统计数据

### 复用建议
1. ✅ 保持 `tool_calling_enabled = true`
2. ✅ 使用 `--once` 参数快速执行
3. ✅ 清晰描述任务意图
4. ✅ 添加专用工具扩展能力

---

**生成时间**: 2025-10-18
**分析工具**: Claude Code
**项目**: RealConsole v0.10.5
