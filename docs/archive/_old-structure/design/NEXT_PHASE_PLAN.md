# Rust 版本下一阶段开发计划

## 📋 规划概览

基于 [功能差距分析](./PYTHON_RUST_GAP_ANALYSIS.md)，本文档制定 Rust 版本的具体开发计划。

**规划原则**:
1. **稳扎稳打** - 保持代码质量，不求快速
2. **极简设计** - 继承 Rust 版本的简洁特性
3. **类型安全** - 充分利用 Rust 类型系统
4. **渐进增强** - 每个阶段都是可用的完整系统

---

## 🎯 Phase 1: 记忆与日志系统（1-2 周）

**目标**: 实现持续对话和命令追踪能力

### 1.1 短期记忆系统 (2-3 天)

#### 功能需求
- Ring Buffer 实现（固定大小，FIFO）
- 对话历史记录
- 记忆查询和检索
- 与 LLM 上下文集成

#### 技术设计

**数据结构**:
```rust
// src/memory.rs

use std::collections::VecDeque;
use chrono::{DateTime, Utc};

/// 记忆条目
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub entry_type: EntryType,
}

#[derive(Debug, Clone, Copy)]
pub enum EntryType {
    User,
    Assistant,
    System,
    Shell,
}

/// 记忆系统
pub struct Memory {
    entries: VecDeque<MemoryEntry>,
    capacity: usize,
}

impl Memory {
    pub fn new(capacity: usize) -> Self;
    pub fn add(&mut self, content: String, entry_type: EntryType);
    pub fn recent(&self, n: usize) -> Vec<&MemoryEntry>;
    pub fn search(&self, keyword: &str) -> Vec<&MemoryEntry>;
    pub fn dump(&self) -> Vec<&MemoryEntry>;
    pub fn clear(&mut self);
}
```

**集成方案**:
```rust
// src/agent.rs
pub struct Agent {
    // ... 现有字段
    pub memory: Memory,  // 新增
}

// 在处理用户输入时记录
impl Agent {
    pub fn handle(&mut self, line: &str) -> String {
        self.memory.add(format!("USER: {}", line), EntryType::User);
        let response = self.handle_internal(line);
        self.memory.add(format!("ASSISTANT: {}", response), EntryType::Assistant);
        response
    }
}
```

#### 新增命令

```bash
# 查看最近记忆
» /memory recent 10

# 搜索记忆
» /memory search "rust"

# 清空记忆
» /memory clear

# 导出记忆
» /memory dump
```

**实现文件**:
- `src/memory.rs` (新建, ~200 lines)
- `src/agent.rs` (修改, +20 lines)
- `src/commands/memory.rs` (新建, ~150 lines)
- `tests/test_memory.rs` (新建, ~100 lines)

---

### 1.2 长期记忆持久化 (1-2 天)

#### 功能需求
- JSONL 格式持久化
- 启动时加载历史
- 增量追加写入
- 文件轮转（大小限制）

#### 技术设计

```rust
// src/memory.rs (扩展)

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentEntry {
    pub timestamp: String,
    pub content: String,
    pub entry_type: String,
}

impl Memory {
    /// 从文件加载历史记忆
    pub fn load_from_file(path: &str, capacity: usize) -> Result<Self, std::io::Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut memory = Memory::new(capacity);

        for line in reader.lines() {
            let line = line?;
            if let Ok(entry) = serde_json::from_str::<PersistentEntry>(&line) {
                memory.add_raw(entry);
            }
        }
        Ok(memory)
    }

    /// 追加写入到文件
    pub fn append_to_file(&self, path: &str, entry: &MemoryEntry) -> Result<(), std::io::Error> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let persistent = PersistentEntry::from(entry);
        let line = serde_json::to_string(&persistent)?;
        writeln!(file, "{}", line)?;
        Ok(())
    }
}
```

**配置集成**:
```yaml
# realconsole.yaml
memory:
  short_term_capacity: 100
  persistent_file: "memory/long_memory.jsonl"
  auto_save: true
  max_file_size_mb: 10
```

**实现文件**:
- `src/memory.rs` (扩展, +100 lines)
- `src/config.rs` (修改, +15 lines)
- `tests/test_memory_persistence.rs` (新建, ~80 lines)

---

### 1.3 执行日志系统 (1-2 天)

#### 功能需求
- 命令执行记录（时间、命令、结果、耗时）
- 日志查询和过滤
- 统计分析（成功率、平均耗时）

#### 技术设计

```rust
// src/execution_logger.rs (新建)

use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub timestamp: DateTime<Utc>,
    pub command: String,
    pub success: bool,
    pub duration_ms: u64,
    pub result_preview: String,  // 前100字符
}

pub struct ExecutionLogger {
    logs: Vec<ExecutionLog>,
    max_logs: usize,
}

impl ExecutionLogger {
    pub fn new(max_logs: usize) -> Self;

    pub fn log(&mut self, command: String, success: bool, duration: Duration, result: &str);

    pub fn recent(&self, n: usize) -> Vec<&ExecutionLog>;

    pub fn stats(&self) -> ExecutionStats;

    pub fn search(&self, keyword: &str) -> Vec<&ExecutionLog>;
}

#[derive(Debug)]
pub struct ExecutionStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub avg_duration_ms: f64,
}
```

**集成方案**:
```rust
// src/agent.rs
pub struct Agent {
    // ... 现有字段
    pub exec_logger: ExecutionLogger,  // 新增
}

impl Agent {
    pub fn handle(&mut self, line: &str) -> String {
        let start = Instant::now();
        let response = self.handle_internal(line);
        let duration = start.elapsed();

        let success = !response.contains("错误") && !response.contains("失败");
        self.exec_logger.log(line.to_string(), success, duration, &response);

        response
    }
}
```

#### 新增命令

```bash
# 查看执行历史
» /log recent 20

# 搜索日志
» /log search "llm"

# 查看统计
» /log stats

# 清空日志
» /log clear
```

**实现文件**:
- `src/execution_logger.rs` (新建, ~250 lines)
- `src/agent.rs` (修改, +15 lines)
- `src/commands/log.rs` (新建, ~180 lines)
- `tests/test_execution_logger.rs` (新建, ~120 lines)

---

### Phase 1 交付物

**代码文件**:
- ✅ `src/memory.rs` (~300 lines)
- ✅ `src/execution_logger.rs` (~250 lines)
- ✅ `src/commands/memory.rs` (~150 lines)
- ✅ `src/commands/log.rs` (~180 lines)
- ✅ 测试文件 (~300 lines)

**新增命令**:
- `/memory recent <n>` - 查看最近记忆
- `/memory search <keyword>` - 搜索记忆
- `/memory clear` - 清空记忆
- `/memory dump` - 导出全部记忆
- `/log recent <n>` - 查看执行历史
- `/log search <keyword>` - 搜索日志
- `/log stats` - 查看统计
- `/log clear` - 清空日志

**功能完成度**: 30% → 35%

**发布版本**: v0.2.0

---

## 🚀 Phase 2: 工具调用系统（2-3 周）

**目标**: 实现 Agent 的核心智能能力

### 2.1 工具注册框架 (2 天)

#### 功能需求
- 工具定义（名称、描述、参数模式）
- 工具注册表
- 工具查询和调用

#### 技术设计

```rust
// src/tool.rs (新建)

use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    pub required: bool,
}

/// 工具定义
#[derive(Debug, Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub handler: ToolHandler,
}

pub type ToolHandler = fn(args: &HashMap<String, Value>) -> Result<String, ToolError>;

#[derive(Debug, Clone)]
pub enum ToolError {
    InvalidArgs(String),
    ExecutionFailed(String),
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self;

    pub fn register(&mut self, tool: Tool);

    pub fn get(&self, name: &str) -> Option<&Tool>;

    pub fn list(&self) -> Vec<&Tool>;

    pub fn to_schemas(&self) -> Vec<ToolSchema>;
}

/// 工具 Schema（用于 LLM）
#[derive(Debug, Serialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}
```

**实现文件**:
- `src/tool.rs` (新建, ~300 lines)
- `tests/test_tool.rs` (新建, ~150 lines)

---

### 2.2 自动工具调用 (3-4 天)

#### 功能需求
- 解析 LLM 返回的工具调用请求
- 自动执行工具
- 将结果反馈给 LLM
- 多轮迭代（最多5轮）

#### 技术设计

```rust
// src/tool_call.rs (新建)

use crate::llm::LlmClient;
use crate::tool::{Tool, ToolRegistry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 工具调用请求（LLM 返回）
#[derive(Debug, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// 工具调用结果
#[derive(Debug, Serialize)]
pub struct ToolCallResult {
    pub tool: String,
    pub success: bool,
    pub output: String,
}

/// 自动工具调用引擎
pub struct ToolCallEngine {
    llm: Arc<dyn LlmClient>,
    registry: Arc<ToolRegistry>,
    max_rounds: usize,
}

impl ToolCallEngine {
    pub fn new(llm: Arc<dyn LlmClient>, registry: Arc<ToolRegistry>) -> Self {
        Self {
            llm,
            registry,
            max_rounds: 5,
        }
    }

    /// 自动工具调用主循环
    pub async fn run(&self, user_query: &str) -> Result<String, ToolCallError> {
        let mut conversation = vec![
            Message::system("你可以调用工具来完成任务。"),
            Message::user(user_query),
        ];

        for round in 1..=self.max_rounds {
            // 1. 发送给 LLM（附带工具列表）
            let response = self.llm.chat(conversation.clone()).await?;

            // 2. 解析响应
            if let Some(tool_call) = self.parse_tool_call(&response) {
                // 3. 执行工具
                let result = self.execute_tool(&tool_call)?;

                // 4. 将结果加入对话
                conversation.push(Message::assistant(&response));
                conversation.push(Message::user(&format!("工具执行结果: {}", result.output)));
            } else {
                // 没有工具调用，返回最终答案
                return Ok(response);
            }
        }

        Err(ToolCallError::MaxRoundsExceeded)
    }

    fn parse_tool_call(&self, response: &str) -> Option<ToolCallRequest>;

    fn execute_tool(&self, request: &ToolCallRequest) -> Result<ToolCallResult, ToolCallError>;
}

#[derive(Debug)]
pub enum ToolCallError {
    LlmError(String),
    ToolNotFound(String),
    ToolExecutionFailed(String),
    MaxRoundsExceeded,
}
```

**实现文件**:
- `src/tool_call.rs` (新建, ~500 lines)
- `tests/test_tool_call.rs` (新建, ~200 lines)

---

### 2.3 内置工具实现 (2-3 天)

#### 基础工具集

**1. Shell 执行工具**
```rust
pub fn tool_shell(args: &HashMap<String, Value>) -> Result<String, ToolError> {
    let command = args.get("command")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("缺少 command 参数".into()))?;

    // 执行 shell 命令
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

**2. 文件读取工具**
```rust
pub fn tool_read_file(args: &HashMap<String, Value>) -> Result<String, ToolError> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("缺少 path 参数".into()))?;

    std::fs::read_to_string(path)
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))
}
```

**3. 文件写入工具**
```rust
pub fn tool_write_file(args: &HashMap<String, Value>) -> Result<String, ToolError> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("缺少 path 参数".into()))?;

    let content = args.get("content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("缺少 content 参数".into()))?;

    std::fs::write(path, content)
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    Ok(format!("已写入 {} 字节到 {}", content.len(), path))
}
```

**4. 文件列表工具**
```rust
pub fn tool_list_files(args: &HashMap<String, Value>) -> Result<String, ToolError> {
    let path = args.get("path")
        .and_then(|v| v.as_str())
        .unwrap_or(".");

    let entries = std::fs::read_dir(path)
        .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

    let mut result = Vec::new();
    for entry in entries {
        if let Ok(entry) = entry {
            result.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    Ok(result.join("\n"))
}
```

**5. 记忆搜索工具**
```rust
pub fn tool_search_memory(args: &HashMap<String, Value>) -> Result<String, ToolError> {
    let keyword = args.get("keyword")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidArgs("缺少 keyword 参数".into()))?;

    // 从 Agent 获取 memory（需要通过上下文传递）
    // 这里简化处理
    Ok(format!("搜索结果: {}", keyword))
}
```

**工具注册**:
```rust
// src/tools/builtin.rs (新建)

pub fn register_builtin_tools(registry: &mut ToolRegistry) {
    // Shell 工具
    registry.register(Tool {
        name: "shell".into(),
        description: "执行 shell 命令".into(),
        parameters: vec![
            ToolParameter {
                name: "command".into(),
                param_type: "string".into(),
                description: "要执行的命令".into(),
                required: true,
            },
        ],
        handler: tool_shell,
    });

    // 文件读取工具
    registry.register(Tool {
        name: "read_file".into(),
        description: "读取文件内容".into(),
        parameters: vec![
            ToolParameter {
                name: "path".into(),
                param_type: "string".into(),
                description: "文件路径".into(),
                required: true,
            },
        ],
        handler: tool_read_file,
    });

    // ... 其他工具
}
```

**实现文件**:
- `src/tools/builtin.rs` (新建, ~400 lines)
- `src/tools/mod.rs` (新建, ~50 lines)
- `tests/test_builtin_tools.rs` (新建, ~150 lines)

---

### 2.4 集成到 Agent (1 天)

```rust
// src/agent.rs (大幅修改)

pub struct Agent {
    // ... 现有字段
    pub tool_registry: Arc<ToolRegistry>,  // 新增
    pub tool_engine: Option<ToolCallEngine>,  // 新增
}

impl Agent {
    pub fn new_with_tools(
        config: Config,
        registry: CommandRegistry,
        llm: Arc<dyn LlmClient>,
    ) -> Self {
        let mut tool_registry = ToolRegistry::new();
        tools::builtin::register_builtin_tools(&mut tool_registry);

        let tool_registry = Arc::new(tool_registry);
        let tool_engine = ToolCallEngine::new(llm.clone(), tool_registry.clone());

        Self {
            // ...
            tool_registry,
            tool_engine: Some(tool_engine),
            // ...
        }
    }

    pub fn handle(&mut self, line: &str) -> String {
        // 如果不是命令，尝试工具调用
        if !line.starts_with('/') && !line.starts_with('!') {
            if let Some(engine) = &self.tool_engine {
                match tokio::runtime::Runtime::new().unwrap().block_on(
                    engine.run(line)
                ) {
                    Ok(response) => return response,
                    Err(e) => return format!("工具调用失败: {:?}", e),
                }
            }
        }

        // 原有逻辑...
        self.handle_command_or_shell(line)
    }
}
```

---

### Phase 2 交付物

**代码文件**:
- ✅ `src/tool.rs` (~300 lines)
- ✅ `src/tool_call.rs` (~500 lines)
- ✅ `src/tools/builtin.rs` (~400 lines)
- ✅ `src/agent.rs` (重构, +150 lines)
- ✅ 测试文件 (~500 lines)

**新增工具**:
- `shell` - 执行 shell 命令
- `read_file` - 读取文件
- `write_file` - 写入文件
- `list_files` - 列出文件
- `search_memory` - 搜索记忆

**功能完成度**: 35% → 60%

**发布版本**: v0.3.0

---

## 📊 时间线规划

```
Week 1-2: Phase 1 - 记忆与日志
├─ Day 1-3: 短期记忆系统
├─ Day 4-5: 长期记忆持久化
└─ Day 6-7: 执行日志系统
    └─ 发布 v0.2.0

Week 3-5: Phase 2 - 工具调用系统
├─ Day 8-9: 工具注册框架
├─ Day 10-13: 自动工具调用
├─ Day 14-16: 内置工具实现
└─ Day 17: 集成测试
    └─ 发布 v0.3.0

Week 6-8: Phase 3 - Shell 增强与可观测性
├─ Day 18-24: 完整 Shell 命令系统
├─ Day 25-27: 沙箱安全系统
└─ Day 28-30: 可观测性命令
    └─ 发布 v0.4.0

Week 9-12: Phase 4 - 完善与优化
├─ 配置验证
├─ Web 访问
├─ UI 增强
├─ 性能优化
└─ 文档完善
    └─ 发布 v1.0.0
```

---

## ✅ 下一步行动

### 立即开始（本周）

1. **创建 memory 模块** (Day 1)
   ```bash
   touch src/memory.rs
   touch src/commands/memory.rs
   touch tests/test_memory.rs
   ```

2. **实现 Ring Buffer** (Day 1-2)
   - `Memory` struct
   - `add()`, `recent()`, `search()` 方法
   - 基础测试

3. **集成到 Agent** (Day 2)
   - 修改 `Agent` struct
   - 在 `handle()` 中记录对话
   - 添加 `/memory` 命令

4. **持久化实现** (Day 3)
   - JSONL 读写
   - 启动时加载
   - 运行时追加

5. **执行日志** (Day 4-5)
   - `ExecutionLogger` struct
   - 统计功能
   - `/log` 命令

### 测试策略

**单元测试**:
```rust
#[test]
fn test_memory_ring_buffer() {
    let mut mem = Memory::new(5);
    for i in 0..10 {
        mem.add(format!("entry-{}", i), EntryType::User);
    }
    assert_eq!(mem.len(), 5);
    assert!(mem.recent(1)[0].content.contains("entry-9"));
}
```

**集成测试**:
```rust
#[test]
fn test_memory_persistence() {
    let path = "test_memory.jsonl";
    let mut mem = Memory::new(10);
    mem.add("test entry".into(), EntryType::User);
    mem.append_to_file(path, &mem.entries[0]).unwrap();

    let loaded = Memory::load_from_file(path, 10).unwrap();
    assert_eq!(loaded.len(), 1);

    std::fs::remove_file(path).unwrap();
}
```

---

## 📝 开发规范

### 代码风格

1. **类型优先**
   ```rust
   // Good
   pub fn add(&mut self, entry: MemoryEntry) -> Result<(), MemoryError>

   // Bad
   pub fn add(&mut self, entry: MemoryEntry)
   ```

2. **错误处理**
   ```rust
   // Good
   match result {
       Ok(value) => process(value),
       Err(e) => handle_error(e),
   }

   // Bad
   let value = result.unwrap();
   ```

3. **文档注释**
   ```rust
   /// 添加记忆条目
   ///
   /// # 参数
   /// - `content`: 记忆内容
   /// - `entry_type`: 条目类型
   ///
   /// # 示例
   /// ```
   /// let mut mem = Memory::new(100);
   /// mem.add("Hello".into(), EntryType::User);
   /// ```
   pub fn add(&mut self, content: String, entry_type: EntryType)
   ```

### 测试要求

- 单元测试覆盖率 > 80%
- 每个公共函数必须有测试
- 关键路径必须有集成测试
- 错误场景必须有测试

### 提交规范

```bash
# 格式: <type>(<scope>): <subject>

# 示例
feat(memory): add ring buffer implementation
fix(memory): handle empty search results
test(memory): add persistence tests
docs(memory): update API documentation
```

---

## 🎯 成功标准

### Phase 1 完成标志

- ✅ 所有单元测试通过
- ✅ 集成测试通过
- ✅ 代码覆盖率 > 80%
- ✅ 编译无警告
- ✅ 文档完整
- ✅ 手动测试通过
  - 记忆能正确记录和检索
  - 持久化能正确保存和加载
  - 执行日志能正确追踪

### Phase 2 完成标志

- ✅ 工具注册系统工作正常
- ✅ 自动工具调用能正确执行
- ✅ 多轮迭代逻辑正确
- ✅ 内置工具全部可用
- ✅ 错误处理完善
- ✅ 性能满足要求（<100ms 延迟）

---

## 📚 参考资料

### Rust 异步编程
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)

### 数据持久化
- [serde_json](https://docs.rs/serde_json/)
- [JSONL Format](http://jsonlines.org/)

### 工具调用
- [OpenAI Function Calling](https://platform.openai.com/docs/guides/function-calling)
- [Anthropic Tool Use](https://docs.anthropic.com/claude/docs/tool-use)

---

**制定日期**: 2025-10-14
**制定者**: Claude Code
**目标版本**: v0.2.0 (Phase 1)
**预计完成**: 2025-10-28 (2 周后)
