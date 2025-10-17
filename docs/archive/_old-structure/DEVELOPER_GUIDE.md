# RealConsole 开发者指南

**版本**: v0.5.0
**更新日期**: 2025-10-15
**适用对象**: RealConsole 贡献者和扩展开发者

---

## 目录

1. [快速开始](#快速开始)
2. [项目架构](#项目架构)
3. [代码结构](#代码结构)
4. [开发环境](#开发环境)
5. [编译与测试](#编译与测试)
6. [代码规范](#代码规范)
7. [贡献指南](#贡献指南)
8. [核心模块详解](#核心模块详解)
9. [扩展开发](#扩展开发)
10. [调试技巧](#调试技巧)

---

## 快速开始

### 克隆项目

```bash
git clone https://github.com/your-repo/realconsole.git
cd realconsole
```

### 安装依赖

确保已安装 Rust 工具链（1.70.0+）：

```bash
# 检查 Rust 版本
rustc --version
cargo --version

# 如未安装，访问 https://rustup.rs/
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 编译项目

```bash
# Debug 模式（开发时使用）
cargo build

# Release 模式（发布时使用）
cargo build --release
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_intent_matching

# 显示测试输出
cargo test -- --nocapture
```

### 运行程序

```bash
# Debug 模式
cargo run

# Release 模式
./target/release/realconsole
```

---

## 项目架构

### 架构概览

RealConsole 采用模块化架构，核心组件围绕 `Agent` 展开：

```
┌─────────────────────────────────────────────────────────┐
│                      RealConsole                        │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Agent (核心调度器)                  │   │
│  │                                                  │   │
│  │  ┌──────────────┐  ┌────────────┐             │   │
│  │  │ LLM Manager  │  │ Tool       │             │   │
│  │  │ - Primary    │  │ Registry   │             │   │
│  │  │ - Fallback   │  │ (14 tools) │             │   │
│  │  └──────────────┘  └────────────┘             │   │
│  │                                                  │   │
│  │  ┌──────────────┐  ┌────────────┐             │   │
│  │  │ Intent       │  │ Memory     │             │   │
│  │  │ Matcher      │  │ System     │             │   │
│  │  │ (50+ intents)│  │ (短期+长期) │             │   │
│  │  └──────────────┘  └────────────┘             │   │
│  │                                                  │   │
│  │  ┌──────────────┐  ┌────────────┐             │   │
│  │  │ Shell        │  │ Execution  │             │   │
│  │  │ Executor     │  │ Logger     │             │   │
│  │  │ (安全执行)    │  │ (审计日志)  │             │   │
│  │  └──────────────┘  └────────────┘             │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │            System Commands (核心命令)            │   │
│  │  /help  /tools  /memory  /log  /version  ...   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │               CLI Interface (REPL)              │   │
│  │              - rustyline 编辑器                  │   │
│  │              - 历史记录                          │   │
│  │              - 自动补全                          │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### 核心组件

#### 1. Agent（src/agent.rs）

**职责**：
- 接收用户输入并分发到不同处理路径
- 管理所有子系统（LLM、工具、记忆、日志）
- 协调工具调用和多轮对话

**关键方法**：
- `handle(&mut self, input: &str) -> String` - 统一入口
- `handle_text(&self, text: &str) -> String` - 处理智能对话
- `handle_shell(&self, cmd: &str) -> String` - 处理 Shell 命令
- `handle_system_command(&self, cmd: &str) -> String` - 处理系统命令

#### 2. LLM Manager（src/llm/）

**职责**：
- 管理 Primary + Fallback LLM 客户端
- 统一 LLM 调用接口（支持流式输出）
- 自动故障转移

**关键模块**：
- `llm_manager.rs` - LLM 管理器
- `deepseek.rs` - Deepseek API 客户端
- `ollama.rs` - Ollama 本地客户端
- `trait LlmClient` - LLM 统一接口

#### 3. Tool System（src/tool*.rs）

**职责**：
- 工具注册与管理
- 工具调用执行（支持并行）
- OpenAI Function Calling Schema 生成

**关键模块**：
- `tool_registry.rs` - 工具注册中心
- `tool_executor.rs` - 工具执行引擎（并行执行）
- `builtin_tools.rs` - 14 个内置工具
- `advanced_tools.rs` - 高级工具（HTTP、编码等）

#### 4. Intent DSL（src/dsl/intent/）

**职责**：
- 自然语言意图识别
- 实体提取与参数解析
- LRU 缓存优化

**关键模块**：
- `matcher.rs` - Intent 匹配引擎
- `builtin.rs` - 50+ 内置意图
- `template.rs` - 响应模板引擎
- `entity.rs` - 实体提取器

#### 5. Memory System（src/memory.rs）

**职责**：
- 短期记忆（环形缓冲区）
- 长期记忆（持久化）
- 记忆搜索与管理

**核心数据结构**：
- `VecDeque<MemoryEntry>` - 环形缓冲区
- `MemoryEntry` - 单条记忆（role, content, timestamp）

#### 6. Shell Executor（src/shell_executor.rs）

**职责**：
- 安全执行 Shell 命令
- 危险命令黑名单检测
- 超时保护

**安全机制**：
- 黑名单正则匹配
- `tokio::time::timeout` 超时控制
- 错误码系统集成

---

## 代码结构

```
realconsole/
├── src/
│   ├── main.rs                   # CLI 入口
│   ├── lib.rs                    # 库入口
│   ├── agent.rs                  # 核心 Agent
│   ├── config.rs                 # 配置管理
│   ├── error.rs                  # 错误系统（30+ 错误码）
│   │
│   ├── llm/
│   │   ├── mod.rs                # LLM 模块入口
│   │   ├── llm_manager.rs        # LLM 管理器
│   │   ├── deepseek.rs           # Deepseek 客户端
│   │   └── ollama.rs             # Ollama 客户端
│   │
│   ├── dsl/
│   │   ├── intent/
│   │   │   ├── mod.rs            # Intent 模块入口
│   │   │   ├── matcher.rs        # Intent 匹配引擎
│   │   │   ├── builtin.rs        # 内置意图
│   │   │   ├── template.rs       # 模板引擎
│   │   │   └── entity.rs         # 实体提取
│   │   └── type_system/          # 类型系统（预留）
│   │
│   ├── tool_registry.rs          # 工具注册
│   ├── tool_executor.rs          # 工具执行引擎
│   ├── builtin_tools.rs          # 内置工具
│   ├── advanced_tools.rs         # 高级工具
│   │
│   ├── memory.rs                 # 记忆系统
│   ├── execution_logger.rs       # 执行日志
│   ├── shell_executor.rs         # Shell 执行器
│   │
│   ├── commands/
│   │   ├── mod.rs                # 命令模块入口
│   │   └── core.rs               # 核心系统命令
│   │
│   └── wizard/                   # 配置向导（Week 2 新增）
│       ├── mod.rs                # 向导模块入口
│       ├── wizard.rs             # 向导主逻辑
│       ├── validator.rs          # 配置验证
│       └── generator.rs          # 配置生成
│
├── tests/                        # 集成测试
│   ├── test_intent_*.rs          # Intent DSL 测试
│   ├── integration_test.rs       # 端到端测试
│   └── ...
│
├── docs/                         # 文档
│   ├── README.md                 # 文档中心
│   ├── USER_GUIDE.md             # 用户手册
│   ├── DEVELOPER_GUIDE.md        # 开发者指南（本文档）
│   ├── API.md                    # API 文档
│   ├── CHANGELOG.md              # 更新日志
│   │
│   ├── guides/                   # 用户指南
│   │   ├── QUICKSTART.md         # 快速入门
│   │   ├── TOOL_CALLING_*.md     # 工具调用指南
│   │   ├── INTENT_DSL_GUIDE.md   # Intent DSL 指南
│   │   └── LLM_SETUP_GUIDE.md    # LLM 配置指南
│   │
│   ├── design/                   # 设计文档
│   │   ├── OVERVIEW.md           # 架构概览
│   │   ├── PHILOSOPHY.md         # 设计哲学
│   │   ├── ERROR_SYSTEM_DESIGN.md # 错误系统设计
│   │   ├── WIZARD_DESIGN.md      # 配置向导设计
│   │   └── ...
│   │
│   └── progress/                 # 开发进度
│       ├── WEEK2_*.md            # Week 2 记录
│       ├── WEEK3_PLAN.md         # Week 3 计划
│       └── ...
│
├── config/                       # 配置示例
│   ├── minimal.yaml              # 最小配置
│   ├── full.yaml                 # 完整配置
│   └── .env.example              # 环境变量示例
│
├── sandbox/                      # 测试环境（Week 2 新增）
│   └── wizard-test/              # 向导测试
│
├── Cargo.toml                    # 项目依赖
├── Cargo.lock                    # 依赖锁定
├── README.md                     # 项目 README
└── CLAUDE.md                     # Claude Code 项目指南
```

### 代码统计

| 模块 | 文件数 | 代码行数 | 测试数 |
|------|--------|----------|--------|
| Agent | 1 | 184 | 8 |
| LLM | 4 | 450+ | 12 |
| Tool System | 4 | 600+ | 25 |
| Intent DSL | 6 | 1400+ | 50+ |
| Memory | 1 | 150 | 10 |
| Shell Executor | 1 | 120 | 8 |
| Commands | 2 | 400+ | 15 |
| Wizard | 4 | 500+ | 20 |
| Error System | 1 | 392 | 7 |
| **总计** | **30+** | **4500+** | **226** |

**测试覆盖率**: 73.30% （Week 1 测试增强完成）

---

## 开发环境

### 必需工具

- **Rust**: 1.70.0 或更高（推荐使用 rustup）
- **Cargo**: Rust 包管理器（随 Rust 安装）
- **Git**: 版本控制

### 推荐工具

- **IDE**:
  - VSCode + rust-analyzer 插件
  - IntelliJ IDEA + Rust 插件
  - Vim/Neovim + rust.vim

- **代码格式化**:
  - `rustfmt` - 自动格式化
  - `clippy` - 静态分析

- **调试工具**:
  - `lldb` (macOS) / `gdb` (Linux)
  - `cargo-watch` - 自动重新编译
  - `cargo-expand` - 展开宏
  - `cargo-tree` - 查看依赖树

- **性能分析**:
  - `cargo-flamegraph` - 火焰图
  - `cargo-bench` - 基准测试
  - `valgrind` - 内存分析

- **测试工具**:
  - `cargo-tarpaulin` - 测试覆盖率
  - `cargo-llvm-cov` - LLVM 覆盖率工具

### VSCode 配置推荐

`.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.features": "all",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "rust-lang.rust-analyzer",
  "rust-analyzer.inlayHints.enable": true
}
```

### 环境变量

开发时建议设置：

```bash
# 启用 Rust backtrace
export RUST_BACKTRACE=1

# 启用详细日志
export RUST_LOG=debug

# 加速编译（可选）
export CARGO_INCREMENTAL=1
export RUSTC_WRAPPER=sccache  # 需先安装 sccache
```

---

## 编译与测试

### 编译

```bash
# Debug 模式（包含调试符号，未优化）
cargo build

# Release 模式（完全优化，生产环境使用）
cargo build --release

# 检查编译错误（不生成二进制文件，速度快）
cargo check

# 清理构建产物
cargo clean
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test intent_matcher
cargo test tool_executor

# 运行单个测试
cargo test test_intent_matching_with_entities

# 显示测试输出（包括 println!）
cargo test -- --nocapture

# 显示测试统计
cargo test -- --test-threads=1

# 只运行文档测试
cargo test --doc

# 只运行集成测试
cargo test --test integration_test
```

### 测试覆盖率

```bash
# 使用 cargo-llvm-cov（推荐）
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# 使用 cargo-tarpaulin（Linux）
cargo tarpaulin --out Html
open tarpaulin-report.html
```

**当前覆盖率**: 73.30%

**目标**: Week 3 Day 4 提升至 75%+

### 基准测试

```bash
# 运行基准测试
cargo bench

# 运行特定基准
cargo bench intent_matching
```

### 代码质量检查

```bash
# Clippy 静态分析
cargo clippy

# 严格模式（禁止任何警告）
cargo clippy -- -D warnings

# 格式化检查
cargo fmt -- --check

# 自动格式化
cargo fmt
```

---

## 代码规范

### 命名规范

- **文件名**: snake_case（如 `tool_executor.rs`）
- **类型名**: CamelCase（如 `ToolRegistry`, `LlmManager`）
- **函数名**: snake_case（如 `execute_tool`, `handle_input`）
- **常量**: SCREAMING_SNAKE_CASE（如 `MAX_PARALLEL_TOOLS`）
- **模块名**: snake_case（如 `mod intent_matcher;`）

### 代码风格

使用 `rustfmt` 自动格式化，遵循以下原则：

1. **缩进**: 4 空格（不使用 Tab）
2. **行宽**: 最大 100 字符
3. **函数长度**: 建议不超过 50 行
4. **文件长度**: 建议不超过 500 行

### 文档注释

所有公共 API 必须包含文档注释：

```rust
/// 执行指定工具并返回结果
///
/// # 参数
/// - `tool_name`: 工具名称
/// - `params`: JSON 格式的参数
///
/// # 返回
/// - `Ok(Value)`: 工具执行成功，返回结果
/// - `Err(RealError)`: 工具执行失败
///
/// # 示例
///
/// ```
/// let result = executor.execute_tool("calculator", json!({"expression": "2+2"})).await?;
/// assert_eq!(result["result"], 4);
/// ```
pub async fn execute_tool(&self, tool_name: &str, params: Value) -> Result<Value, RealError> {
    // ...
}
```

### 错误处理

1. **统一使用 RealError**：

```rust
use crate::error::{ErrorCode, RealError, FixSuggestion};

fn my_function() -> Result<(), RealError> {
    Err(RealError::new(
        ErrorCode::ConfigNotFound,
        "配置文件不存在",
    )
    .with_suggestion(
        FixSuggestion::new("运行配置向导创建配置文件")
            .with_command("realconsole wizard"),
    ))
}
```

2. **传播源错误**：

```rust
fs::read_to_string(path).map_err(|e| {
    RealError::new(ErrorCode::FileReadError, "无法读取文件")
        .with_source(e)  // 保留原始错误
})?;
```

### 异步规范

1. **统一使用 tokio**：

```rust
#[tokio::main]
async fn main() {
    // ...
}

async fn my_async_function() -> Result<String, RealError> {
    // ...
}
```

2. **避免阻塞操作**：

```rust
// ❌ 错误：阻塞当前线程
let content = std::fs::read_to_string("file.txt")?;

// ✅ 正确：使用异步 I/O
let content = tokio::fs::read_to_string("file.txt").await?;
```

### 测试规范

1. **单元测试**: 放在模块末尾

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }

    #[tokio::test]
    async fn test_async_function() {
        // ...
    }
}
```

2. **集成测试**: 放在 `tests/` 目录

```rust
// tests/integration_test.rs
use realconsole::Agent;

#[tokio::test]
async fn test_end_to_end() {
    // ...
}
```

---

## 贡献指南

### 开发流程

1. **Fork 仓库** 到你的 GitHub 账号

2. **克隆你的 Fork**：
   ```bash
   git clone https://github.com/your-username/realconsole.git
   cd realconsole
   ```

3. **创建功能分支**：
   ```bash
   git checkout -b feature/my-new-feature
   ```

4. **开发并测试**：
   ```bash
   cargo build
   cargo test
   cargo clippy
   cargo fmt
   ```

5. **提交更改**：
   ```bash
   git add .
   git commit -m "Add: 添加新功能 XXX"
   ```

6. **推送到 Fork**：
   ```bash
   git push origin feature/my-new-feature
   ```

7. **创建 Pull Request** 到主仓库的 `main` 分支

### 提交信息规范

使用语义化提交信息：

```
<类型>: <简短描述>

<详细描述>（可选）

<关联 Issue>（可选）
```

**类型**：
- `Add`: 新增功能
- `Fix`: 修复 Bug
- `Update`: 更新功能
- `Refactor`: 重构代码
- `Doc`: 文档更新
- `Test`: 测试相关
- `Chore`: 构建/工具配置

**示例**：

```
Add: 实现 HTTP POST 工具

- 添加 HttpPostTool 结构体
- 实现异步 POST 请求
- 添加 JSON body 支持
- 包含单元测试

Closes #123
```

### PR 检查清单

提交 PR 前请确认：

- [ ] 代码通过 `cargo build`
- [ ] 所有测试通过 `cargo test`
- [ ] 代码通过 `cargo clippy` 无警告
- [ ] 代码已格式化 `cargo fmt`
- [ ] 添加了必要的测试
- [ ] 更新了相关文档
- [ ] 提交信息符合规范
- [ ] PR 描述清晰说明了改动内容

### Code Review 流程

1. **自动检查**: CI 会自动运行测试和 Clippy
2. **人工审查**: 至少 1 位维护者审查代码
3. **修改建议**: 根据反馈修改代码
4. **合并**: 审查通过后由维护者合并

---

## 核心模块详解

### 1. Agent 模块（src/agent.rs）

**职责**: 核心调度器，协调所有子系统

**关键数据结构**：

```rust
pub struct Agent {
    config: Arc<Config>,
    llm_manager: Arc<LlmManager>,
    tool_registry: Arc<ToolRegistry>,
    tool_executor: Arc<ToolExecutor>,
    intent_matcher: Arc<IntentMatcher>,
    memory: Arc<RwLock<Memory>>,
    execution_logger: Arc<RwLock<ExecutionLogger>>,
}
```

**核心方法**：

```rust
impl Agent {
    /// 统一入口：分发用户输入
    pub async fn handle(&mut self, input: &str) -> String {
        // 1. Shell 命令（! 前缀）
        if input.starts_with("!") {
            return self.handle_shell(&input[1..]);
        }

        // 2. 系统命令（/ 前缀）
        if input.starts_with(&self.config.prefix) {
            return self.handle_system_command(&input[1..]);
        }

        // 3. 智能对话（无前缀）
        self.handle_text(input).await
    }
}
```

### 2. Tool Executor 模块（src/tool_executor.rs）

**职责**: 工具并行执行引擎

**关键方法**：

```rust
impl ToolExecutor {
    /// 执行多个工具（并行）
    pub async fn execute_tools(&self, tool_calls: Vec<ToolCall>) -> Vec<ToolResult> {
        let futures = tool_calls.into_iter()
            .map(|call| self.execute_single_tool(call));

        // 使用 join_all 并行执行
        join_all(futures).await
    }
}
```

**性能优化**（Week 3 Day 2 计划）：
- LRU 缓存工具结果
- 减少不必要的重复调用

### 3. Intent Matcher 模块（src/dsl/intent/matcher.rs）

**职责**: 自然语言意图识别

**核心算法**：

```rust
impl IntentMatcher {
    /// 匹配意图（带缓存）
    pub fn match_intent(&self, text: &str) -> Option<MatchedIntent> {
        // 1. 检查缓存
        if let Some(cached) = self.cache.read().unwrap().get(text) {
            return Some(cached.clone());
        }

        // 2. 遍历所有意图
        for intent in &self.intents {
            if self.is_match(intent, text) {
                let matched = self.extract_entities(intent, text);
                self.cache.write().unwrap().put(text.to_string(), matched.clone());
                return Some(matched);
            }
        }

        None
    }
}
```

**缓存策略**：
- LRU 缓存（容量 100）
- 缓存键：原始输入文本
- 缓存命中率：通常 > 60%

### 4. Memory 模块（src/memory.rs）

**职责**: 对话历史管理

**数据结构**：

```rust
pub struct Memory {
    entries: VecDeque<MemoryEntry>,  // 环形缓冲区
    max_size: usize,                  // 最大容量
    persist_path: Option<PathBuf>,    // 持久化路径
}

pub struct MemoryEntry {
    role: String,        // "user" or "assistant"
    content: String,     // 对话内容
    timestamp: i64,      // 时间戳
}
```

**性能优化**（Week 3 Day 2 计划）：
- 添加关键词索引（HashMap）
- 添加时间戳索引（BTreeMap）
- 批量持久化（异步写入）

---

## 扩展开发

### 添加自定义工具

参考 [工具调用开发指南](guides/TOOL_CALLING_DEVELOPER_GUIDE.md)

**步骤**：

1. **创建工具结构体**：

```rust
use realconsole::Tool;
use async_trait::async_trait;
use serde_json::{Value, json};
use anyhow::Result;

pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn name(&self) -> &str {
        "my_custom_tool"
    }

    fn description(&self) -> &str {
        "这是我的自定义工具"
    }

    async fn execute(&self, params: Value) -> Result<Value> {
        // 解析参数
        let input = params["input"].as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing 'input' parameter"))?;

        // 执行逻辑
        let result = format!("Processed: {}", input);

        // 返回结果
        Ok(json!({"result": result}))
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
                        "input": {
                            "type": "string",
                            "description": "输入文本"
                        }
                    },
                    "required": ["input"]
                }
            }
        })
    }
}
```

2. **注册工具**（src/agent.rs）：

```rust
impl Agent {
    pub fn new(config: Config) -> Self {
        let mut tool_registry = ToolRegistry::new();

        // 注册内置工具
        register_builtin_tools(&mut tool_registry);

        // 注册自定义工具
        tool_registry.register(Box::new(MyCustomTool));

        // ...
    }
}
```

3. **添加测试**：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_custom_tool() {
        let tool = MyCustomTool;
        let params = json!({"input": "test"});
        let result = tool.execute(params).await.unwrap();
        assert_eq!(result["result"], "Processed: test");
    }
}
```

### 添加自定义 Intent

参考 [Intent DSL 指南](guides/INTENT_DSL_GUIDE.md)

**步骤**：

1. **定义 Intent**（src/dsl/intent/builtin.rs）：

```rust
Intent {
    id: "my_custom_intent".to_string(),
    keywords: vec!["关键词1", "关键词2"],
    patterns: vec![r"正则表达式模式"],
    templates: vec!["响应模板: {entity}"],
    domain: "custom".to_string(),
    priority: 50,
    examples: vec!["示例输入"],
}
```

2. **注册 Intent**：

```rust
pub fn builtin_intents() -> Vec<Intent> {
    vec![
        // ... 现有意图
        Intent { /* 自定义意图 */ },
    ]
}
```

---

## 调试技巧

### 1. 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=debug
cargo run

# 只显示特定模块的日志
export RUST_LOG=realconsole::intent_matcher=debug
cargo run
```

### 2. 使用调试器

```bash
# macOS (lldb)
lldb target/debug/realconsole
(lldb) b main
(lldb) run

# Linux (gdb)
gdb target/debug/realconsole
(gdb) break main
(gdb) run
```

### 3. 打印调试信息

```rust
// 开发时临时打印
dbg!(variable);
eprintln!("Debug: {:?}", variable);

// 生产环境使用日志
log::debug!("Processing intent: {:?}", intent);
log::info!("Tool executed successfully: {}", tool_name);
log::warn!("Cache miss for key: {}", key);
log::error!("Failed to execute tool: {}", e);
```

### 4. 性能分析

```bash
# 火焰图（需要先安装 cargo-flamegraph）
cargo flamegraph

# 基准测试
cargo bench

# 内存分析（Linux）
valgrind --tool=massif target/debug/realconsole
```

### 5. 查看宏展开

```bash
# 展开宏（需要先安装 cargo-expand）
cargo expand intent_matcher
```

### 6. 依赖分析

```bash
# 查看依赖树
cargo tree

# 查找重复依赖
cargo tree --duplicates

# 查看过时依赖
cargo outdated
```

---

## 附录

### A. 依赖清单

主要依赖（Cargo.toml）：

```toml
[dependencies]
# 异步运行时
tokio = { version = "1.35", features = ["full"] }

# HTTP 客户端
reqwest = { version = "0.11", features = ["json", "stream"] }

# JSON 处理
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# CLI 交互
rustyline = "13.0"
clap = { version = "4.4", features = ["derive"] }

# 颜色输出
colored = "2.1"

# 日志
log = "0.4"
env_logger = "0.11"

# 错误处理
anyhow = "1.0"
thiserror = "1.0"

# 工具
async-trait = "0.1"
once_cell = "1.19"
regex = "1.10"
chrono = "0.4"
uuid = "1.6"

# 环境变量
dotenvy = "0.15"

# 数学计算
evalexpr = "11.3"
```

### B. 性能指标

| 指标 | 目标 | 当前 |
|------|------|------|
| 启动时间 | < 20ms | ~10ms |
| 内存占用 | < 10MB | ~8MB |
| Intent 匹配 | < 1ms | ~0.5ms (缓存命中) |
| 工具调用 | < 100ms | 取决于工具 |
| 测试覆盖率 | > 75% | 73.30% |

### C. 相关资源

- **Rust 官方文档**: https://doc.rust-lang.org/
- **Tokio 文档**: https://docs.rs/tokio/
- **async-trait**: https://docs.rs/async-trait/
- **Rust 异步编程**: https://rust-lang.github.io/async-book/

---

**版本**: v0.5.0
**更新日期**: 2025-10-15
**文档状态**: ✅ Week 3 更新完成

**有问题？** 查看 [Issue 跟踪](https://github.com/your-repo/realconsole/issues) 或加入我们的开发者社区！
