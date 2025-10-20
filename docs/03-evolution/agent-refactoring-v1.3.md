# Agent 重构设计 - v1.3.0

## 背景

Agent 当前是一个 2,694 行的"God Object"，包含 21 个字段，职责混杂。需要通过服务层架构进行重构。

## 设计目标

1. **简化 Agent** - 从 2,694 行减少到 ~800 行
2. **服务化** - 将职责分离到独立的服务
3. **向后兼容** - 保持现有 API 不变
4. **渐进迁移** - 分阶段实施，避免破坏性变更

## 当前字段分析（21 个字段）

### 核心配置（2 个）
- `config: Config` - 应用配置
- `registry: CommandRegistry` - 命令注册表

### LLM 相关（2 个）
- `llm_manager: Arc<RwLock<LlmManager>>` - LLM 管理器
- `llm_bridge: Option<Arc<LlmToPipeline>>` - LLM-Pipeline 桥接

### 工具相关（2 个）
- `tool_registry: Arc<RwLock<ToolRegistry>>` - 工具注册表
- `tool_executor: Arc<ToolExecutor>` - 工具执行器

### Intent DSL 相关（5 个）
- `intent_matcher: IntentMatcher` - Intent 匹配器
- `template_engine: TemplateEngine` - 模板引擎
- `pipeline_converter: IntentToPipeline` - Pipeline 转换器
- `workflow_intents: Vec<WorkflowIntent>` - Workflow Intent 列表
- `workflow_executor: Option<Arc<WorkflowExecutor>>` - Workflow 执行器

### 状态管理相关（5 个）
- `memory: Arc<RwLock<Memory>>` - 记忆系统
- `exec_logger: Arc<RwLock<ExecutionLogger>>` - 执行日志
- `history: Arc<RwLock<HistoryManager>>` - 历史管理
- `stats_collector: Arc<StatsCollector>` - 统计收集器
- `context_tracker: Arc<RwLock<ContextTracker>>` - 上下文追踪

### Shell 执行相关（2 个）
- `shell_executor_with_fixer: Arc<ShellExecutorWithFixer>` - Shell 执行器
- `last_failed_command: Arc<RwLock<Option<String>>>` - 最后失败的命令

### 其他（3 个）
- `conversation_manager: Arc<RwLock<ConversationManager>>` - 对话管理器
- `command_router: CommandRouter` - 命令路由器

## 重构策略：渐进式迁移

### Phase 1: 添加服务层（v1.3.0）✅

**保留所有现有字段，添加服务层字段**：

```rust
pub struct Agent {
    // === 核心配置 ===
    pub config: Config,
    pub registry: CommandRegistry,

    // === 服务层（新增）===
    state_manager: Arc<StateManager>,
    intent_service: Arc<IntentService>,
    llm_service: Arc<LlmService>,
    shell_service: Arc<ShellService>,

    // === 原有字段（保留，向后兼容）===
    pub llm_manager: Arc<RwLock<LlmManager>>,
    pub memory: Arc<RwLock<Memory>>,
    pub exec_logger: Arc<RwLock<ExecutionLogger>>,
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    pub tool_executor: Arc<ToolExecutor>,
    pub intent_matcher: IntentMatcher,
    pub template_engine: TemplateEngine,
    pub pipeline_converter: IntentToPipeline,
    pub llm_bridge: Option<Arc<LlmToPipeline>>,
    pub history: Arc<RwLock<HistoryManager>>,
    pub conversation_manager: Arc<RwLock<ConversationManager>>,
    pub stats_collector: Arc<StatsCollector>,
    pub context_tracker: Arc<RwLock<ContextTracker>>,
    pub shell_executor_with_fixer: Arc<ShellExecutorWithFixer>,
    pub last_failed_command: Arc<RwLock<Option<String>>>,
    pub command_router: CommandRouter,
    pub workflow_intents: Vec<WorkflowIntent>,
    pub workflow_executor: Option<Arc<WorkflowExecutor>>,
}
```

### Phase 2: 迁移方法实现（v1.3.0）

**逐步迁移方法到服务调用**：

1. **Intent 处理** → IntentService
   - `try_match_intent()` → `intent_service.process()`
   - `handle_text_with_intent()` → 使用 IntentService

2. **LLM 处理** → LlmService
   - `handle_text()` → `llm_service.process(LlmRequest::normal())`
   - `handle_text_with_tools()` → `llm_service.process(LlmRequest::with_tools())`

3. **Shell 执行** → ShellService
   - `handle_shell()` → `shell_service.process()`

### Phase 3: 弃用旧字段（v1.4.0）

**添加 #[deprecated] 标记**：

```rust
#[deprecated(since = "1.4.0", note = "Use state_manager.memory() instead")]
pub memory: Arc<RwLock<Memory>>,
```

### Phase 4: 移除旧字段（v2.0.0）

**最终移除所有被服务替代的字段**。

## 实施计划

### Step 1: 添加服务字段（当前）✅

- [x] 创建 StateManager
- [x] 创建 IntentService
- [x] 创建 LlmService
- [x] 创建 ShellService
- [ ] 在 Agent 中添加服务字段
- [ ] 更新 Agent::new() 初始化逻辑

### Step 2: 迁移 Intent 处理

- [ ] 重构 `try_match_intent()` 使用 IntentService
- [ ] 重构 `handle_text_with_intent()` 使用 IntentService
- [ ] 验证测试通过

### Step 3: 迁移 LLM 处理

- [ ] 重构 `handle_text()` 使用 LlmService
- [ ] 重构 `handle_text_with_tools()` 使用 LlmService
- [ ] 验证测试通过

### Step 4: 迁移 Shell 处理

- [ ] 重构 `handle_shell()` 使用 ShellService
- [ ] 验证测试通过

### Step 5: 全量测试

- [ ] 运行所有单元测试
- [ ] 运行集成测试
- [ ] 性能测试

## 预期效果

- **代码行数**: 2,694 → ~800 行（减少 70%）
- **字段数量**: 21 → 8 个（减少 62%）
- **可测试性**: 提升（服务可独立测试）
- **可维护性**: 提升（职责清晰）
- **向后兼容**: 100%（Phase 1-2）

## 风险评估

- **低风险**: Phase 1-2（添加字段，迁移实现）
- **中风险**: Phase 3（添加 deprecated 标记）
- **高风险**: Phase 4（移除字段，破坏性变更）

## 时间估算

- Phase 1: 1-2 天 ✅
- Phase 2: 2-3 天（当前）
- Phase 3: 1 天（v1.4.0）
- Phase 4: 1 天（v2.0.0）

---

**创建日期**: 2025-10-20
**作者**: RealConsole Contributors
**版本**: v1.3.0-dev
