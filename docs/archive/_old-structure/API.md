# RealConsole API 文档

**版本**: v0.5.0
**更新日期**: 2025-10-15
**适用对象**: RealConsole 扩展开发者

---

## 目录

1. [概述](#概述)
2. [核心模块 API](#核心模块-api)
3. [扩展点说明](#扩展点说明)
4. [使用示例](#使用示例)
5. [类型定义](#类型定义)
6. [错误处理](#错误处理)

---

## 概述

本文档是 [Rust 自动生成 API 文档](../target/doc/realconsole/index.html) 的补充，重点介绍：

- **核心模块的公共 API**
- **扩展接口（Tool、LlmClient 等）**
- **实际使用示例**

**查看完整 API 文档**：

```bash
# 生成并打开 Rust 文档
cargo doc --no-deps --open
```

---

## 核心模块 API

### 1. Agent（核心调度器）

**模块路径**: `realconsole::Agent`

#### Agent::new

创建新的 Agent 实例。

```rust
pub fn new(config: Config) -> Self
```

**参数**：
- `config: Config` - 配置对象

**返回**：
- `Agent` - Agent 实例

**示例**：

```rust
use realconsole::{Config, Agent};

let config = Config::from_file("realconsole.yaml")?;
let agent = Agent::new(config);
```

#### Agent::handle

处理用户输入（异步方法）。

```rust
pub async fn handle(&mut self, input: &str) -> String
```

**参数**：
- `input: &str` - 用户输入文本

**返回**：
- `String` - 处理结果

**示例**：

```rust
let response = agent.handle("计算 2+2").await;
println!("{}", response);
```

---

### 2. ToolRegistry（工具注册中心）

**模块路径**: `realconsole::ToolRegistry`

#### ToolRegistry::new

创建空的工具注册中心。

```rust
pub fn new() -> Self
```

#### ToolRegistry::register

注册一个工具。

```rust
pub fn register(&mut self, tool: Box<dyn Tool>)
```

**参数**：
- `tool: Box<dyn Tool>` - 实现了 Tool trait 的工具对象

**示例**：

```rust
use realconsole::{ToolRegistry, Tool};

let mut registry = ToolRegistry::new();
registry.register(Box::new(CalculatorTool));
registry.register(Box::new(DatetimeTool));
```

#### ToolRegistry::get

根据名称获取工具。

```rust
pub fn get(&self, name: &str) -> Option<&Box<dyn Tool>>
```

**参数**：
- `name: &str` - 工具名称

**返回**：
- `Option<&Box<dyn Tool>>` - 工具引用（如果存在）

**示例**：

```rust
if let Some(tool) = registry.get("calculator") {
    println!("Found tool: {}", tool.name());
}
```

#### ToolRegistry::list

列出所有已注册的工具。

```rust
pub fn list(&self) -> Vec<String>
```

**返回**：
- `Vec<String>` - 工具名称列表

**示例**：

```rust
let tools = registry.list();
println!("Available tools: {:?}", tools);
```

---

### 3. ToolExecutor（工具执行引擎）

**模块路径**: `realconsole::ToolExecutor`

#### ToolExecutor::new

创建工具执行引擎。

```rust
pub fn new(registry: Arc<ToolRegistry>, max_parallel: usize, max_iterations: usize) -> Self
```

**参数**：
- `registry: Arc<ToolRegistry>` - 工具注册中心（共享引用）
- `max_parallel: usize` - 最大并行工具数（默认 3）
- `max_iterations: usize` - 最大迭代轮数（默认 5）

#### ToolExecutor::execute_tools

并行执行多个工具。

```rust
pub async fn execute_tools(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult>
```

**参数**：
- `tool_calls: Vec<ToolCall>` - 工具调用列表

**返回**：
- `Vec<ToolResult>` - 工具执行结果列表

**示例**：

```rust
use realconsole::{ToolCall, ToolExecutor};
use serde_json::json;

let tool_calls = vec![
    ToolCall {
        name: "calculator".to_string(),
        params: json!({"expression": "2+2"}),
    },
    ToolCall {
        name: "datetime".to_string(),
        params: json!({"format": "RFC3339"}),
    },
];

let results = executor.execute_tools(tool_calls).await;
for result in results {
    println!("Tool: {}, Result: {:?}", result.name, result.output);
}
```

---

### 4. Memory（记忆系统）

**模块路径**: `realconsole::Memory`

#### Memory::new

创建记忆系统。

```rust
pub fn new(max_size: usize) -> Self
```

**参数**：
- `max_size: usize` - 最大记忆条数

#### Memory::add

添加一条记忆。

```rust
pub fn add(&mut self, role: &str, content: &str)
```

**参数**：
- `role: &str` - 角色（"user" 或 "assistant"）
- `content: &str` - 对话内容

**示例**：

```rust
use realconsole::Memory;

let mut memory = Memory::new(100);
memory.add("user", "你好");
memory.add("assistant", "你好！有什么可以帮助你的吗？");
```

#### Memory::get_recent

获取最近 N 条记忆。

```rust
pub fn get_recent(&self, n: usize) -> Vec<MemoryEntry>
```

**参数**：
- `n: usize` - 获取数量

**返回**：
- `Vec<MemoryEntry>` - 记忆条目列表

#### Memory::search

搜索包含关键词的记忆。

```rust
pub fn search(&self, keyword: &str) -> Vec<MemoryEntry>
```

**参数**：
- `keyword: &str` - 搜索关键词

**返回**：
- `Vec<MemoryEntry>` - 匹配的记忆列表

**示例**：

```rust
let results = memory.search("Rust");
for entry in results {
    println!("[{}] {}", entry.role, entry.content);
}
```

#### Memory::clear

清空所有记忆。

```rust
pub fn clear(&mut self)
```

---

### 5. Config（配置管理）

**模块路径**: `realconsole::Config`

#### Config::from_file

从 YAML 文件加载配置。

```rust
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, RealError>
```

**参数**：
- `path: P` - 配置文件路径

**返回**：
- `Result<Config, RealError>` - 配置对象或错误

**示例**：

```rust
use realconsole::Config;

let config = Config::from_file("realconsole.yaml")?;
println!("Prefix: {}", config.prefix);
```

#### Config::default

创建默认配置。

```rust
pub fn default() -> Self
```

**返回**：
- `Config` - 默认配置对象

---

### 6. Shell Executor（Shell 执行器）

**模块路径**: `realconsole::shell_executor`

#### execute_shell

安全执行 Shell 命令。

```rust
pub async fn execute_shell(command: &str) -> Result<String, RealError>
```

**参数**：
- `command: &str` - Shell 命令

**返回**：
- `Result<String, RealError>` - 命令输出或错误

**错误**：
- `ErrorCode::ShellCommandEmpty` - 命令为空
- `ErrorCode::ShellDangerousCommand` - 危险命令（黑名单）
- `ErrorCode::ShellTimeoutError` - 命令超时
- `ErrorCode::ShellExecutionError` - 执行失败

**示例**：

```rust
use realconsole::shell_executor;

match shell_executor::execute_shell("ls -la").await {
    Ok(output) => println!("Output:\n{}", output),
    Err(e) => eprintln!("Error: {}", e.format_user_friendly()),
}
```

---

## 扩展点说明

### 1. Tool Trait（工具接口）

**定义**：

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名称（唯一标识符）
    fn name(&self) -> &str;

    /// 工具描述
    fn description(&self) -> &str;

    /// 执行工具（异步方法）
    async fn execute(&self, params: Value) -> Result<Value>;

    /// 生成 OpenAI Function Calling Schema
    fn to_openai_schema(&self) -> Value;
}
```

**实现示例**：

参考 [工具调用开发指南](guides/TOOL_CALLING_DEVELOPER_GUIDE.md)

```rust
use realconsole::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};
use anyhow::Result;

pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "数学计算器（支持 +, -, *, /, ^, sqrt 等）"
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        let expression = params["expression"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'expression' parameter"))?;

        // 使用 evalexpr 计算表达式
        let result = evalexpr::eval(expression)?;

        Ok(json!({"result": result.to_string()}))
    }

    fn to_openai_schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": self.name(),
                "description": self.description(),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "数学表达式"
                        }
                    },
                    "required": ["expression"]
                }
            }
        })
    }
}
```

**注册工具**：

```rust
let mut registry = ToolRegistry::new();
registry.register(Box::new(CalculatorTool));
```

---

### 2. LlmClient Trait（LLM 客户端接口）

**定义**：

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 普通对话（非流式）
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Value>>,
    ) -> Result<ChatResponse>;

    /// 流式对话
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Value>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
```

**实现示例**：

```rust
use realconsole::LlmClient;
use async_trait::async_trait;

pub struct MyLlmClient {
    api_key: String,
    endpoint: String,
}

#[async_trait]
impl LlmClient for MyLlmClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Value>>,
    ) -> Result<ChatResponse> {
        // 实现 LLM 调用逻辑
        todo!()
    }

    async fn chat_stream(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Value>>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // 实现流式 LLM 调用逻辑
        todo!()
    }
}
```

---

## 使用示例

### 示例 1：创建自定义 Agent

```rust
use realconsole::{Config, Agent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 加载配置
    let config = Config::from_file("realconsole.yaml")?;

    // 创建 Agent
    let mut agent = Agent::new(config);

    // 处理用户输入
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let response = agent.handle(input.trim()).await;
        println!("{}", response);
    }
}
```

### 示例 2：注册自定义工具

```rust
use realconsole::{Config, Agent, ToolRegistry, Tool};
use async_trait::async_trait;
use serde_json::{Value, json};
use anyhow::Result;

// 自定义工具
pub struct WeatherTool;

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str { "weather" }
    fn description(&self) -> &str { "查询天气" }

    async fn execute(&self, params: Value) -> Result<Value> {
        let city = params["city"].as_str().unwrap_or("北京");
        // 实际应该调用天气 API
        Ok(json!({"city": city, "weather": "晴", "temp": 25}))
    }

    fn to_openai_schema(&self) -> Value {
        json!({
            "type": "function",
            "function": {
                "name": "weather",
                "description": "查询天气",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "city": {"type": "string", "description": "城市名"}
                    },
                    "required": ["city"]
                }
            }
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_file("realconsole.yaml")?;
    let mut agent = Agent::new(config);

    // 注册自定义工具（需要修改 Agent::new 以支持外部注册）
    // agent.register_tool(Box::new(WeatherTool));

    let response = agent.handle("查询北京天气").await;
    println!("{}", response);

    Ok(())
}
```

### 示例 3：使用记忆系统

```rust
use realconsole::Memory;

fn main() {
    let mut memory = Memory::new(100);

    // 添加记忆
    memory.add("user", "我的名字是小明");
    memory.add("assistant", "你好，小明！");
    memory.add("user", "我喜欢 Rust");
    memory.add("assistant", "Rust 是一门很棒的语言！");

    // 获取最近对话
    let recent = memory.get_recent(4);
    for entry in recent {
        println!("[{}] {}", entry.role, entry.content);
    }

    // 搜索记忆
    let results = memory.search("Rust");
    println!("\n搜索 'Rust' 的结果:");
    for entry in results {
        println!("[{}] {}", entry.role, entry.content);
    }

    // 清空记忆
    memory.clear();
    println!("\n记忆已清空");
}
```

### 示例 4：工具并行执行

```rust
use realconsole::{ToolExecutor, ToolRegistry, ToolCall};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 创建工具注册中心
    let mut registry = ToolRegistry::new();
    // 注册内置工具
    realconsole::register_builtin_tools(&mut registry);

    // 创建工具执行引擎
    let executor = ToolExecutor::new(Arc::new(registry), 3, 5);

    // 并行调用多个工具
    let tool_calls = vec![
        ToolCall {
            name: "calculator".to_string(),
            params: json!({"expression": "2+2"}),
        },
        ToolCall {
            name: "datetime".to_string(),
            params: json!({"format": "RFC3339"}),
        },
        ToolCall {
            name: "uuid".to_string(),
            params: json!({}),
        },
    ];

    let results = executor.execute_tools(tool_calls).await;

    for result in results {
        println!("Tool: {}", result.name);
        println!("Output: {:?}", result.output);
        println!("---");
    }

    Ok(())
}
```

### 示例 5：Shell 安全执行

```rust
use realconsole::shell_executor;

#[tokio::main]
async fn main() {
    // 安全命令
    match shell_executor::execute_shell("ls -la").await {
        Ok(output) => println!("输出:\n{}", output),
        Err(e) => eprintln!("错误: {}", e.format_user_friendly()),
    }

    // 危险命令（会被拦截）
    match shell_executor::execute_shell("rm -rf /").await {
        Ok(_) => println!("不应该到达这里"),
        Err(e) => {
            eprintln!("命令被拦截:");
            eprintln!("{}", e.format_user_friendly());
        }
    }

    // 超时命令
    match shell_executor::execute_shell("sleep 20").await {
        Ok(_) => println!("不应该到达这里"),
        Err(e) => {
            eprintln!("命令超时:");
            eprintln!("{}", e.format_user_friendly());
        }
    }
}
```

---

## 类型定义

### Message

LLM 对话消息。

```rust
pub struct Message {
    pub role: String,      // "user", "assistant", "system"
    pub content: String,   // 消息内容
}
```

### ToolCall

工具调用请求。

```rust
pub struct ToolCall {
    pub name: String,      // 工具名称
    pub params: Value,     // JSON 格式参数
}
```

### ToolResult

工具执行结果。

```rust
pub struct ToolResult {
    pub name: String,      // 工具名称
    pub output: Value,     // JSON 格式输出
    pub error: Option<String>,  // 错误信息（如果有）
}
```

### MemoryEntry

记忆条目。

```rust
pub struct MemoryEntry {
    pub role: String,      // "user" or "assistant"
    pub content: String,   // 对话内容
    pub timestamp: i64,    // Unix 时间戳
}
```

### ChatResponse

LLM 响应。

```rust
pub struct ChatResponse {
    pub content: String,           // 响应文本
    pub tool_calls: Vec<ToolCall>, // 工具调用请求（如果有）
}
```

---

## 错误处理

### RealError

统一错误类型。

```rust
pub struct RealError {
    pub code: ErrorCode,                  // 错误代码
    pub message: String,                  // 错误消息
    pub suggestions: Vec<FixSuggestion>,  // 修复建议
    pub source: Option<Box<dyn Error>>,   // 源错误
}
```

**方法**：

```rust
impl RealError {
    /// 创建新错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self;

    /// 添加修复建议
    pub fn with_suggestion(mut self, suggestion: FixSuggestion) -> Self;

    /// 添加源错误
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self;

    /// 格式化为用户友好的错误信息
    pub fn format_user_friendly(&self) -> String;
}
```

### ErrorCode

错误代码枚举。

```rust
pub enum ErrorCode {
    // Config errors (E001-E099)
    ConfigNotFound,      // E001
    ConfigParseError,    // E002

    // LLM errors (E100-E199)
    LlmAuthError,        // E102

    // Shell errors (E300-E399)
    ShellDangerousCommand,  // E302
    ShellTimeoutError,      // E303

    // Tool errors (E400-E499)
    ToolNotFound,        // E401

    // Memory errors (E500-E599)
    MemoryNotInitialized,  // E501

    // ... 30+ 错误码
}
```

### FixSuggestion

修复建议。

```rust
pub struct FixSuggestion {
    pub description: String,       // 建议描述
    pub command: Option<String>,   // 建议执行的命令
    pub doc_link: Option<String>,  // 文档链接
}
```

**Builder 模式**：

```rust
FixSuggestion::new("运行配置向导")
    .with_command("realconsole wizard")
    .with_doc("https://docs.realconsole.com/wizard")
```

**使用示例**：

```rust
use realconsole::error::{ErrorCode, FixSuggestion, RealError};

fn my_function() -> Result<(), RealError> {
    Err(RealError::new(
        ErrorCode::ConfigNotFound,
        "配置文件不存在: realconsole.yaml",
    )
    .with_suggestion(
        FixSuggestion::new("运行配置向导创建配置文件")
            .with_command("realconsole wizard --quick"),
    )
    .with_suggestion(
        FixSuggestion::new("查看配置指南")
            .with_doc("https://docs.realconsole.com/config"),
    ))
}
```

---

## 附录

### A. 完整 API 文档

生成完整的 Rust API 文档：

```bash
cargo doc --no-deps --open
```

文档位置：`target/doc/realconsole/index.html`

### B. 相关文档

- **[用户手册](USER_GUIDE.md)** - 所有功能的详细说明
- **[开发者指南](DEVELOPER_GUIDE.md)** - 架构与开发环境
- **[工具调用开发指南](guides/TOOL_CALLING_DEVELOPER_GUIDE.md)** - 创建自定义工具
- **[Intent DSL 指南](guides/INTENT_DSL_GUIDE.md)** - 自定义意图识别
- **[错误系统设计](design/ERROR_SYSTEM_DESIGN.md)** - 错误代码详解

### C. 示例代码

完整示例代码请参考：

- `tests/integration_test.rs` - 集成测试示例
- `src/builtin_tools.rs` - 内置工具实现
- `src/commands/core.rs` - 系统命令实现

---

**版本**: v0.5.0
**更新日期**: 2025-10-15
**文档状态**: ✅ Week 3 更新完成

**有问题？** 查看 [开发者指南](DEVELOPER_GUIDE.md) 或 [提交 Issue](https://github.com/your-repo/realconsole/issues)！
