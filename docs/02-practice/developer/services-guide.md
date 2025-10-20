# 服务层架构使用指南

**版本**: v1.3.0
**作者**: RealConsole Contributors
**更新**: 2025-10-20

## 概述

RealConsole v1.3.0 引入了服务层架构，将 Agent 的职责拆分为独立的服务模块。

## 核心服务

### 1. StateManager - 状态管理

统一管理所有状态组件：

```rust
let agent = Agent::new(config, registry);
let state_manager = agent.state_manager();

// 访问各个状态组件
let memory = state_manager.memory();
let history = state_manager.history();
let context_tracker = state_manager.context_tracker();
let stats_collector = state_manager.stats_collector();
let exec_logger = state_manager.exec_logger();
```

### 2. IntentService - Intent DSL 处理

处理自然语言意图识别：

```rust
use realconsole::services::{IntentRequest, Service};

let agent = Agent::new(config, registry);
let intent_service = agent.intent_service();

// 创建请求
let request = IntentRequest::from_config(
    "统计 Python 代码行数".to_string(),
    &config
);

// 处理请求
let response = intent_service.process(request).await?;

if let Some(plan) = response.plan {
    println!("执行计划: {}", plan.command);
    println!("置信度: {}", response.confidence);
}
```

**功能特性**：
- Workflow Intent 匹配（如果启用）
- LLM 驱动的 Pipeline 生成（如果启用）
- 传统 Intent DSL 匹配
- 参数提取（支持 LLM 增强）

### 3. LlmService - LLM 交互

处理所有 LLM 相关操作：

```rust
use realconsole::services::{LlmRequest, LlmMode, Service};

let agent = Agent::new(config, registry);
let llm_service = agent.llm_service();

// 普通模式 - 一次性返回
let request = LlmRequest::normal("你好，RealConsole".to_string());
let response = llm_service.process(request).await?;
println!("响应: {}", response.text);
println!("耗时: {}ms", response.duration_ms);

// 流式模式 - 实时输出
let request = LlmRequest::streaming("解释什么是服务层架构".to_string());
let response = llm_service.process(request).await?;
// 输出已实时打印到 stdout

// 工具调用模式 - Function Calling
let request = LlmRequest::with_tools("统计当前目录的代码行数".to_string());
let response = llm_service.process(request).await?;
println!("工具调用结果: {}", response.text);
```

**三种处理模式**：
- `Normal` - 普通对话，一次性返回完整响应
- `Streaming` - 流式输出，实时显示 token
- `WithTools` - 工具调用模式，支持 Function Calling

**Primary/Fallback 机制**：
- 自动使用 Primary LLM（如 Deepseek）
- 失败时自动切换到 Fallback（如 Ollama）
- `used_fallback` 字段标识是否使用了 Fallback

### 4. ShellService - Shell 命令执行

处理 Shell 命令执行和错误修复：

```rust
use realconsole::services::{ShellRequest, Service};

let agent = Agent::new(config, registry);
let shell_service = agent.shell_service();

// 普通执行
let request = ShellRequest::new("ls -la".to_string());
let response = shell_service.process(request).await?;

if response.result.success {
    println!("输出: {}", response.result.output);
} else {
    println!("错误: {}", response.result.output);

    // 显示修复建议
    for suggestion in response.fix_suggestions {
        println!("建议: {}", suggestion);
    }
}

// 强制执行（跳过危险命令检测）
let request = ShellRequest::forced("rm dangerous.txt".to_string());
let response = shell_service.process(request).await?;
```

**功能特性**：
- 危险命令检测（如 `rm -rf /`）
- 自动错误分析
- 智能修复建议
- 反馈学习系统

## Service Trait

所有服务都实现了统一的 `Service` trait：

```rust
#[async_trait]
pub trait Service: Send + Sync {
    type Request;
    type Response;
    type Error;

    async fn process(&self, request: Self::Request)
        -> Result<Self::Response, Self::Error>;

    fn name(&self) -> &str;

    async fn health_check(&self) -> bool {
        true
    }
}
```

## 使用场景

### 场景 1: 构建自定义 CLI 工具

```rust
use realconsole::{Agent, Config, services::Service};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.yaml")?;
    let agent = Agent::new(config, Default::default());

    // 使用 Intent 服务处理用户输入
    let request = IntentRequest::from_config(
        "查找最大的文件".to_string(),
        &agent.config
    );

    let response = agent.intent_service().process(request).await?;

    if let Some(plan) = response.plan {
        // 使用 Shell 服务执行命令
        let shell_req = ShellRequest::new(plan.command);
        let shell_resp = agent.shell_service().process(shell_req).await?;
        println!("{}", shell_resp.result.output);
    }

    Ok(())
}
```

### 场景 2: 集成到现有项目

```rust
// 只使用 LlmService 进行对话
use realconsole::services::{LlmService, LlmRequest, Service};

let llm_service = /* 初始化 LlmService */;

let request = LlmRequest::normal("帮我解释这段代码".to_string());
let response = llm_service.process(request).await?;
println!("AI 回复: {}", response.text);
```

### 场景 3: 测试驱动开发

```rust
#[tokio::test]
async fn test_intent_service() {
    let intent_service = /* 初始化 IntentService */;

    let request = IntentRequest {
        text: "统计文件数量".to_string(),
        llm_generation_enabled: false,
        llm_extraction_enabled: false,
        llm_validation_enabled: false,
        workflow_enabled: false,
    };

    let response = intent_service.process(request).await.unwrap();

    assert!(response.plan.is_some());
    assert!(response.confidence > 0.8);
}
```

## 设计原则

### 1. 单一职责

每个服务只负责一个明确的领域：
- IntentService - 意图识别
- LlmService - LLM 交互
- ShellService - Shell 执行
- StateManager - 状态管理

### 2. 依赖注入

服务通过构造函数接收依赖，易于测试和替换：

```rust
let llm_service = LlmService::new(
    llm_manager,
    tool_registry,
    tool_executor,
);
```

### 3. 接口隔离

通过 Service trait 定义清晰的接口，降低耦合度。

### 4. 状态分离

状态管理独立于业务逻辑，通过 StateManager 统一访问。

## 最佳实践

### 1. 错误处理

```rust
match intent_service.process(request).await {
    Ok(response) => {
        if let Some(plan) = response.plan {
            // 处理成功
        } else {
            // 没有匹配的意图
        }
    }
    Err(IntentError::NoMatch) => {
        // 回退到 LLM 处理
    }
    Err(e) => {
        // 其他错误
        eprintln!("错误: {}", e);
    }
}
```

### 2. 健康检查

```rust
// 检查服务是否可用
if !llm_service.health_check().await {
    eprintln!("LLM 服务不可用");
    return;
}
```

### 3. 性能优化

```rust
// 使用异步并发处理多个请求
let (intent_result, llm_result) = tokio::join!(
    intent_service.process(intent_req),
    llm_service.process(llm_req)
);
```

## 向后兼容

Agent 保留了所有原有字段和方法，确保 100% 向后兼容：

```rust
// 旧代码仍然可以工作
let memory = agent.memory();
let llm_manager = agent.llm_manager();

// 新代码使用服务层
let intent_service = agent.intent_service();
let llm_service = agent.llm_service();
```

## 未来扩展

服务层架构为未来功能扩展奠定了基础：

- **ToolService** - 工具管理服务
- **CommandService** - 系统命令服务
- **MemoryService** - 记忆管理服务
- **AnalyticsService** - 分析统计服务

## 参考资料

- [服务层重构设计](../../03-evolution/agent-refactoring-v1.3.md)
- [Service trait 源码](../../../src/services/mod.rs)
- [IntentService 源码](../../../src/services/intent_service.rs)
- [LlmService 源码](../../../src/services/llm_service.rs)
- [ShellService 源码](../../../src/services/shell_service.rs)

---

**维护者**: RealConsole Contributors
**许可**: MIT
