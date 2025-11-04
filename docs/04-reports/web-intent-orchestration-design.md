# Web 版本意图理解和任务编排设计

> 📅 **日期**: 2025-11-04
> 🎯 **目标**: 实现智能意图理解和任务编排，让 Web 版本"跟手"
> 🏮 **理念**: 简化够用且好用 · 充分复用命令行版本经验

---

## 一、需求背景

### 1.1 用户核心诉求

> "对于文字的理解和用户意图的拆解执行非常好，比如可以告诉它们'帮我查看最近的git提交并总结'，那么web端会调度LLM和本地的git还有其他工具，来汇总最终结果呈现给我"

**示例场景**：

```bash
# 用户输入
% 帮我查看最近的git提交并总结

# 系统智能分解
1. 执行: git log -10 --oneline
2. 获取输出
3. 调用 LLM 总结
4. 返回: "最近10次提交主要涉及..."
```

### 1.2 设计原则

**遵循极简哲学**：
> "努力的过程可能会比较复杂具有挑战性，但是交付的结果用户使用起来会很简洁"

**充分复用经验**：
> "千万不要完全重启炉灶，而是充分学习原有命令行版本的中经验，将之提炼抽取巧妙运用进来"

---

## 二、命令行版本的现有能力

### 2.1 Intent DSL 系统

**位置**: `src/dsl/intent/`

**核心组件**：

1. **IntentMatcher** - 意图匹配器
   - 50+ 内置意图
   - 基于关键词和正则模式匹配
   - 返回匹配度和提取的实体

2. **TemplateEngine** - 模板引擎
   - 将 Intent 转换为可执行计划
   - 支持变量替换
   - 生成 Shell 命令

3. **EntityExtractor** - 实体提取器
   - 提取参数（文件名、数量、时间等）
   - 支持 LLM 辅助提取

**使用流程**：

```rust
// 1. 匹配意图
let matches = intent_matcher.match_intent("统计 Python 代码行数");

// 2. 生成执行计划
let plan = template_engine.generate_plan(&intent_match)?;

// 3. 执行命令
shell_executor.execute(&plan.command).await?;
```

### 2.2 Task 系统

**位置**: `src/task/`

**核心组件**：

1. **TaskDecomposer** - 任务分解器
   - 使用 LLM 智能分解复杂任务
   - 输入：目标描述
   - 输出：SubTask 列表

2. **TaskPlanner** - 任务规划器
   - 分析任务依赖
   - 生成执行计划（串行/并行）
   - 优化执行顺序

3. **TaskExecutor** - 任务执行器
   - 执行计划
   - 进度反馈
   - 错误处理

**使用流程**：

```rust
// 1. 分解任务
let decomposer = TaskDecomposer::new(llm);
let subtasks = decomposer.decompose("部署应用", &context).await?;

// 2. 规划执行
let planner = TaskPlanner::new();
let plan = planner.plan("部署应用", subtasks)?;

// 3. 执行任务
let executor = TaskExecutor::new(shell_executor);
let result = executor.execute(plan).await?;
```

### 2.3 两个系统的区别

| 特性 | Intent DSL | Task 系统 |
|------|-----------|----------|
| **适用场景** | 预定义的常见任务 | 复杂、动态任务 |
| **匹配方式** | 关键词 + 正则 | LLM 理解 |
| **执行模式** | 单一命令或简单流程 | 多步骤编排 |
| **性能** | 快速（本地匹配） | 较慢（需要 LLM） |
| **灵活性** | 固定模板 | 动态分解 |

---

## 三、Web 版本实现方案

### 3.1 整体架构

**三层结构**（遵循"道生三"）：

```
┌─────────────────────────────────────┐
│  Router 层 (天 - 意图识别)           │
│  - CommandRouter: 命令路由           │
│  - IntentMatcher: 意图匹配 (新增)    │
├─────────────────────────────────────┤
│  Orchestrator 层 (人 - 编排协调)    │
│  - SimpleOrchestrator: 简单编排      │
│  - TaskOrchestrator: 复杂编排 (可选) │
├─────────────────────────────────────┤
│  Executor 层 (地 - 执行)            │
│  - ShellExecutor: Shell 执行        │
│  - LlmExecutor: LLM 调用            │
│  - SystemExecutor: 系统命令         │
└─────────────────────────────────────┘
```

### 3.2 方案对比

#### 方案 A：仅集成 Intent DSL（推荐）✅

**特点**：
- 极简实现，复用现有 IntentMatcher
- 覆盖 80% 常见场景
- 性能好（本地匹配）
- 实现成本低

**流程**：

```rust
// 1. 路由
let router = CommandRouter::new(prefix);
match router.route(input) {
    CommandType::NaturalLanguage(text) => {
        // 2. 尝试 Intent 匹配
        if let Some(intent_match) = intent_matcher.match_intent(&text) {
            // 3. 生成执行计划
            let plan = template_engine.generate_plan(&intent_match)?;

            // 4. 执行命令
            execute_shell_command(&plan.command, ...).await?;
        } else {
            // 5. 回退到普通 LLM 对话
            execute_llm_chat(&text, ...).await?;
        }
    }
    // ... 其他路由
}
```

**优势**：
- ✅ 简单清晰
- ✅ 性能好
- ✅ 100% 复用现有代码
- ✅ 符合"简化够用且好用"理念

**局限**：
- ⚠️ 不支持动态任务分解
- ⚠️ 需要预定义意图模板

#### 方案 B：集成完整 Task 系统

**特点**：
- 完整的任务分解能力
- 动态理解复杂任务
- 需要 LLM 支持
- 实现复杂度高

**流程**：

```rust
// 1. LLM 判断是否需要任务分解
let needs_decomposition = llm.analyze(&input).await?;

if needs_decomposition {
    // 2. 分解任务
    let decomposer = TaskDecomposer::new(llm);
    let subtasks = decomposer.decompose(&input, &context).await?;

    // 3. 规划执行
    let planner = TaskPlanner::new();
    let plan = planner.plan(&input, subtasks)?;

    // 4. 执行
    let executor = TaskExecutor::new(shell_executor);
    let result = executor.execute(plan).await?;
} else {
    // 回退到普通 LLM 对话
    execute_llm_chat(&input, ...).await?;
}
```

**优势**：
- ✅ 支持复杂动态任务
- ✅ 更智能的理解

**局限**：
- ❌ 实现复杂
- ❌ 需要多次 LLM 调用
- ❌ 性能开销大
- ❌ 违背"简化"原则

#### 方案 C：混合方案（渐进式）🎯

**特点**：
- 先实现方案 A（Intent DSL）
- 保留扩展接口
- 未来可集成 Task 系统

**阶段规划**：

**Phase 1: Intent 集成** ✅ 推荐优先实现
- 集成 IntentMatcher 到 WebSocket
- 支持常见意图模板
- 覆盖 80% 场景

**Phase 2: 增强 Intent** (可选)
- 添加 LLM 辅助实体提取
- 扩展内置意图库

**Phase 3: Task 集成** (按需)
- 添加 TaskDecomposer
- 用于真正复杂的场景

---

## 四、推荐实现：Intent DSL 集成

### 4.1 核心修改

**文件**: `src/web/websocket.rs`

**修改 handle_input() 函数**：

```rust
async fn handle_input(
    session: &Arc<Session>,
    input: &str,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(());
    }

    let agent = session.agent.read().await;

    // 1. 使用 CommandRouter 进行智能路由
    let router = CommandRouter::new(agent.config.prefix.clone());
    let result = match router.route(input) {
        CommandType::SystemCommand(cmd, args) => {
            let cmd_input = if args.is_empty() {
                cmd
            } else {
                format!("{} {}", cmd, args)
            };
            execute_system_command(&cmd_input, &agent, sender).await
        }
        CommandType::CommonShell(cmd) | CommandType::ForcedShell(cmd) => {
            execute_shell_command(&format!("!{}", cmd), &agent, sender).await
        }
        CommandType::NaturalLanguage(text) => {
            // 2. 🆕 尝试 Intent 匹配
            if let Some(intent_match) = agent.intent_matcher.match_intent(&text) {
                execute_intent(&intent_match, &agent, sender).await
            } else {
                // 3. 回退到 LLM 对话
                execute_llm_chat(&text, &agent, sender).await
            }
        }
    };

    if let Err(e) = result {
        let error_msg = ServerMessage::Error {
            content: format!("执行失败: {}", e),
        };
        sender.send(Message::Text(serde_json::to_string(&error_msg)?)).await?;
    }

    Ok(())
}
```

**新增函数**: `execute_intent()`

```rust
/// 执行 Intent 意图
async fn execute_intent(
    intent_match: &IntentMatch,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    // 1. 生成执行计划
    let plan = agent.template_engine.generate_plan(intent_match)?;

    // 2. 发送提示信息
    let info_msg = ServerMessage::Output {
        content: format!("🎯 意图识别: {}\n", intent_match.intent.name),
    };
    sender.send(Message::Text(serde_json::to_string(&info_msg)?)).await?;

    // 3. 执行计划中的命令
    match plan.execution_type {
        ExecutionType::ShellCommand => {
            execute_shell_command(&format!("!{}", plan.command), agent, sender).await
        }
        ExecutionType::LlmQuery => {
            execute_llm_chat(&plan.command, agent, sender).await
        }
        ExecutionType::Workflow => {
            // 工作流：多步骤执行
            execute_workflow(&plan, agent, sender).await
        }
    }
}
```

**新增函数**: `execute_workflow()` (可选)

```rust
/// 执行工作流（多步骤）
async fn execute_workflow(
    plan: &ExecutionPlan,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let mut workflow_output = String::new();

    for (idx, step) in plan.steps.iter().enumerate() {
        // 1. 发送步骤提示
        let step_msg = ServerMessage::Output {
            content: format!("📍 步骤 {}/{}: {}\n", idx + 1, plan.steps.len(), step.description),
        };
        sender.send(Message::Text(serde_json::to_string(&step_msg)?)).await?;

        // 2. 执行步骤
        let step_output = match &step.action {
            StepAction::Shell(cmd) => {
                let output = execute_shell_sync(cmd).await?;
                workflow_output.push_str(&output);
                output
            }
            StepAction::Llm(prompt) => {
                let output = execute_llm_sync(prompt, agent).await?;
                workflow_output.push_str(&output);
                output
            }
        };

        // 3. 发送步骤结果
        let result_msg = ServerMessage::Output {
            content: step_output,
        };
        sender.send(Message::Text(serde_json::to_string(&result_msg)?)).await?;
    }

    Ok(())
}
```

### 4.2 示例意图

**场景 1: 查看最近的 Git 提交并总结**

```rust
// Intent 定义（在 builtin.rs 中）
Intent {
    name: "git_log_summary".to_string(),
    domain: IntentDomain::Git,
    keywords: vec!["查看".to_string(), "git".to_string(), "提交".to_string(), "总结".to_string()],
    patterns: vec![r"(查看|看一下).*(git|提交).*(总结|summarize)".to_string()],
    // ...
}

// Template 定义
Template {
    intent_name: "git_log_summary".to_string(),
    execution_type: ExecutionType::Workflow,
    steps: vec![
        Step {
            description: "获取最近的 Git 提交".to_string(),
            action: StepAction::Shell("git log -10 --oneline".to_string()),
        },
        Step {
            description: "使用 LLM 总结提交信息".to_string(),
            action: StepAction::Llm("请总结以下 Git 提交记录:\n{previous_output}".to_string()),
        },
    ],
}
```

**用户交互**：

```bash
% 帮我查看最近的git提交并总结

🎯 意图识别: git_log_summary

📍 步骤 1/2: 获取最近的 Git 提交
32b8009 task cmds refactoring
8ba9397 readme update and v1.22.1 release notes
1734e33 feat: v1.22.1 - 任务命令统一重构
...

📍 步骤 2/2: 使用 LLM 总结提交信息
最近10次提交主要涉及：
1. 任务命令系统重构（v1.22.1）
2. README 文档更新
3. 版本发布相关工作
...
```

### 4.3 需要添加的文件

**无需新增文件** ✅

完全复用现有：
- `src/dsl/intent/mod.rs`
- `src/dsl/intent/matcher.rs`
- `src/dsl/intent/template.rs`
- `src/dsl/intent/builtin.rs`

**只需修改**：
- `src/web/websocket.rs` - 集成 Intent 处理

---

## 五、实现细节

### 5.1 Session 初始化

**修改**: `src/web/session.rs`

```rust
impl Session {
    pub async fn new(config: Config, registry: CommandRegistry) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut agent = Agent::new(config.clone(), registry);

        // 配置 LLM
        Self::configure_llm(&mut agent, &config).await;

        // 🆕 初始化 Intent 系统（已经在 Agent::new() 中完成）
        // agent.intent_matcher 已经包含内置意图

        Self {
            id: id.clone(),
            agent: Arc::new(RwLock::new(agent)),
            created_at: chrono::Utc::now(),
        }
    }
}
```

**无需修改** ✅ - Agent 已经包含 `intent_matcher` 和 `template_engine`

### 5.2 消息扩展

**可选**: 添加新的 ServerMessage 类型

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    Thinking { model: String },
    Output { content: String },
    Stream { chunk: String },
    Error { content: String },
    Clear,

    // 🆕 Intent 相关消息
    IntentMatched {
        intent_name: String,
        confidence: f64,
    },
    WorkflowStep {
        current: usize,
        total: usize,
        description: String,
    },
}
```

**前端处理**：

```javascript
case 'intent_matched':
    term.write(`\r\n🎯 ${msg.intent_name}\r\n`);
    break;

case 'workflow_step':
    term.write(`\r\n📍 步骤 ${msg.current}/${msg.total}: ${msg.description}\r\n`);
    break;
```

---

## 六、测试场景

### 6.1 Intent 匹配场景

| 用户输入 | 匹配意图 | 执行结果 |
|---------|---------|---------|
| `统计 Python 代码行数` | count_lines | 执行: `find . -name "*.py" \| xargs wc -l` |
| `查看最近的 git 提交` | git_log | 执行: `git log -10 --oneline` |
| `帮我查看最近的git提交并总结` | git_log_summary | 工作流: git log → LLM 总结 |
| `查找包含 TODO 的文件` | find_todos | 执行: `grep -r "TODO" .` |

### 6.2 回退场景

| 用户输入 | Intent 匹配 | 实际处理 |
|---------|------------|---------|
| `你好` | 无匹配 | LLM 对话 |
| `介绍一下 Rust` | 无匹配 | LLM 对话 |
| `翻译这段文字` | 无匹配 | LLM 对话 |

---

## 七、优势分析

### 7.1 与方案 B 对比

| 维度 | 方案 A (Intent) | 方案 B (Task) |
|------|----------------|--------------|
| **实现复杂度** | 低（~100行） | 高（~500行） |
| **代码复用** | 100% | ~60% |
| **性能** | 快（本地匹配） | 慢（多次 LLM） |
| **响应时间** | <50ms | >2s |
| **覆盖场景** | 80% 常见场景 | 100% |
| **用户体验** | 简洁清晰 | 复杂但强大 |
| **符合极简理念** | ✅ 是 | ❌ 否 |

### 7.2 符合设计原则

**极简主义** ✅
- 最小化修改（~100行代码）
- 复用现有 Intent 系统
- 清晰的三层架构

**够用** ✅
- 覆盖 80% 常见场景
- 预定义 50+ 意图
- 支持扩展

**好用** ✅
- 快速响应（本地匹配）
- 清晰的反馈（步骤提示）
- 智能回退（LLM 对话）

**充分复用** ✅
- 100% 复用 IntentMatcher
- 100% 复用 TemplateEngine
- 无需重复造轮子

---

## 八、实施计划

### 8.1 Phase 1: 核心集成（推荐优先）

**任务**：
1. ✅ 探索命令行版本的 Intent 和 Task 系统
2. ⏳ 修改 `websocket.rs` 集成 Intent 处理
3. ⏳ 实现 `execute_intent()` 函数
4. ⏳ 测试常见意图场景
5. ⏳ 编写实现文档

**预计时间**: 2-3小时

### 8.2 Phase 2: 工作流支持（可选）

**任务**：
1. 实现 `execute_workflow()` 函数
2. 添加步骤消息类型
3. 前端显示步骤进度
4. 测试多步骤场景

**预计时间**: 1-2小时

### 8.3 Phase 3: Task 集成（按需）

**任务**：
1. 集成 TaskDecomposer
2. 实现动态任务分解
3. 添加进度反馈
4. 复杂场景测试

**预计时间**: 4-6小时

**建议**: 暂不实施，保持极简

---

## 九、总结

### 9.1 推荐方案

**✅ 方案 A: Intent DSL 集成**

**理由**：
1. 符合"简化够用且好用"理念
2. 100% 复用现有代码
3. 覆盖 80% 常见场景
4. 实现成本低，风险小
5. 性能好，用户体验佳

### 9.2 关键指标

| 指标 | 目标值 |
|------|--------|
| 代码复用率 | 100% |
| 新增代码 | ~100 行 |
| 场景覆盖 | 80% |
| 响应时间 | <50ms |
| 编译时间 | +10s |

### 9.3 下一步行动

**立即实施**：
1. 修改 `websocket.rs` 集成 Intent
2. 实现 `execute_intent()`
3. 测试常见场景
4. 更新文档

**后续优化**（可选）：
- 添加工作流支持
- 扩展意图库
- 优化错误提示

---

**最后更新**: 2025-11-04
**版本**: v1.23.0+
**状态**: 🎯 设计完成，待实施
