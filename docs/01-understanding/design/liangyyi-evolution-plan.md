# 两仪架构演化计划

**制定日期**: 2025-10-27
**版本**: v1.0
**状态**: 执行中
**原则**: 思想超越，行动务实，渐进演化

---

## 🎯 核心理念

### 易的三层含义

**简易**：大道至简，复杂问题简单化
- 系统只分两类：观测（坎）vs 行动（离）
- 循环只有三步：收 → 炼 → 发
- 抽象层次清晰，代码易于理解

**变易**：万物皆变，系统持续演化
- 功能可以增减，但框架稳定
- 从简单到复杂，逐步演化
- 保持适应性和扩展性

**不易**：本质不变，核心规律恒定
- 观测-决策-行动的循环不变
- 离坎循环的自主学习不变
- 一分为三的方法论不变

### 演化原则

1. **继承优秀**：已有的好特性保留并增强
2. **淘汰混乱**：逻辑不清的功能逐步移除
3. **渐进改进**：每一步都有实际价值
4. **保持可用**：任何时候系统都能正常运行
5. **文档同步**：代码变化，文档同步更新

---

## 📊 现有系统评估

### 一、优秀特性（必须继承）

#### 1.1 Suggest 模块（⭐⭐⭐⭐⭐）

**现状**：
- 三源融合（Context + History + LLM）✅
- 快速执行（数字快捷键）✅
- 拼写纠错（Levenshtein 距离）✅
- 建议缓存（三态生命周期）✅
- 反馈学习（P2.1 已实现基础）✅

**评价**：这是系统的核心价值，设计优秀，实现完整

**演化方向**：
- 保持现有架构不变
- 增强：集成离坎循环，实现真正的自主学习
- 位置：作为 `ActionSystem` 的核心 Actor

#### 1.2 Task 系统（⭐⭐⭐⭐⭐）

**现状**：
- 自然语言任务分解 ✅
- 依赖分析与并行执行 ✅
- 可视化进度展示 ✅

**评价**：体现了"一分为三"的智慧，非常优秀

**演化方向**：
- 保持现有架构
- 位置：作为 `ActionSystem` 的 Planner

#### 1.3 四维追踪（⭐⭐⭐⭐）

**现状**：
- Statistics（统计）✅
- Coordination（协调）✅
- BlackBox（黑盒）✅
- Memory（记忆）✅

**评价**：四维观测非常有价值，但有改进空间

**演化方向**：
- 统一到 `ObservationSystem` 框架下
- 四维作为不同的 Observer 实现
- 增加统一的查询接口

#### 1.4 Intent DSL（⭐⭐⭐⭐）

**现状**：
- 50+ 内置意图
- 正则匹配 + 模板引擎
- LRU 缓存优化

**评价**：非常实用的命令路由系统

**演化方向**：
- 保持现有实现
- 位置：作为 `DecisionSystem` 的核心组件
- 增强：与离坎循环集成，自动发现新意图

#### 1.5 LLM 流式输出（⭐⭐⭐⭐⭐）

**现状**：
- Deepseek/Ollama/OpenAI 支持
- Token-by-token 流式显示
- Tool Calling 集成

**评价**：用户体验极佳，是核心竞争力

**演化方向**：
- 保持现有实现
- 位置：作为 `DecisionSystem` 的智能核心

### 二、需要改进的部分

#### 2.1 Memory 系统（⭐⭐⭐ → ⭐⭐⭐⭐⭐）

**现状问题**：
- 只是简单的对话存储
- 缺乏智能检索
- 没有自主学习能力

**改进方向**：
- 升级为 `ObservationSystem` 的核心存储
- 实现离坎循环的"坎"端
- 增加模式提取和知识沉淀

#### 2.2 Trace 的重复性（⭐⭐⭐）

**现状问题**：
- Statistics、Coordination、BlackBox 之间有重复
- 四个维度的边界不够清晰
- 查询接口不统一

**改进方向**：
- 统一到 `ObservationSystem`
- 每个维度成为独立的 Observer
- 提供统一的多维查询

#### 2.3 Context 的碎片化（⭐⭐）

**现状问题**：
- 上下文散落在各处
- 没有统一的 Context 管理
- 难以形成完整的认知

**改进方向**：
- 在 `DecisionSystem` 中统一管理
- 从各个 Observer 聚合上下文
- 形成结构化的 Context

### 三、需要淘汰的部分

#### 3.1 过度复杂的配置

**问题**：
- 配置项过多，用户困惑
- 有些配置项从未使用

**方案**：
- 识别并移除未使用的配置
- 合并重复的配置项
- 提供更智能的默认值

#### 3.2 冗余的日志

**问题**：
- 日志过于详细，淹没关键信息
- BlackBox 和其他日志重复

**方案**：
- 统一日志级别管理
- 减少冗余日志
- 重要事件使用 Trace，调试信息使用 log

---

## 🗺️ 演化路线图

### Phase 1: 两仪框架（2 周）

**目标**：建立统一的观测-决策-行动框架，不破坏现有功能

#### Week 1: ObservationSystem 基础

**任务**：
1. ✅ 创建 `Observer` trait
2. ✅ 实现 `ObservationSystem` 核心
3. ✅ 将 Memory 迁移为第一个 Observer
4. ✅ 测试：Memory 功能不受影响

**代码结构**：
```rust
// src/observation/mod.rs
pub trait Observer {
    fn observe(&mut self, event: &SystemEvent) -> Result<Observation>;
    fn dimension(&self) -> ObservationDimension;
    fn query(&self, query: &ObservationQuery) -> Result<Vec<Observation>>;
}

pub struct ObservationSystem {
    observers: HashMap<ObservationDimension, Box<dyn Observer>>,
    storage: ObservationStorage,
}

// src/observation/observers/memory.rs
pub struct MemoryObserver {
    // 复用现有 memory 实现
    inner: ConversationMemory,
}

impl Observer for MemoryObserver {
    fn observe(&mut self, event: &SystemEvent) -> Result<Observation> {
        // 包装现有逻辑
        if let SystemEvent::LlmInteraction { role, content, .. } = event {
            self.inner.add(role, content)?;
            Ok(Observation::Memory { ... })
        } else {
            Ok(Observation::Skip)
        }
    }
}
```

**验证**：
- 现有 Memory 功能完全正常
- 可以通过 ObservationSystem 访问
- 单元测试全部通过

#### Week 2: ActionSystem 基础

**任务**：
1. ✅ 创建 `Actor` trait
2. ✅ 实现 `ActionSystem` 核心
3. ✅ 将 Suggest 迁移为第一个 Actor
4. ✅ 测试：Suggest 功能不受影响

**代码结构**：
```rust
// src/action/mod.rs
pub trait Actor {
    fn act(&mut self, intent: &Intent, context: &Context) -> Result<Action>;
    fn action_type(&self) -> ActionType;
    fn can_handle(&self, intent: &Intent) -> bool;
}

pub struct ActionSystem {
    actors: HashMap<ActionType, Box<dyn Actor>>,
}

// src/action/actors/suggester.rs
pub struct SuggesterActor {
    // 复用现有 suggest 实现
    engine: SuggestionEngine,
}

impl Actor for SuggesterActor {
    fn act(&mut self, intent: &Intent, context: &Context) -> Result<Action> {
        // 包装现有逻辑
        if intent.is_suggest_command() {
            let suggestions = self.engine.generate(context)?;
            Ok(Action::Suggestions(suggestions))
        } else {
            Ok(Action::None)
        }
    }
}
```

**验证**：
- 现有 Suggest 功能完全正常
- 可以通过 ActionSystem 调用
- 单元测试全部通过

### Phase 2: 决策系统整合（1 周）

**目标**：建立 DecisionSystem，整合 Agent、LLM、Intent

#### Week 3: DecisionSystem 核心

**任务**：
1. ✅ 创建 `DecisionSystem` 结构
2. ✅ 整合 ObservationSystem 和 ActionSystem
3. ✅ 保持 Agent 原有调度逻辑
4. ✅ 测试：系统整体功能不受影响

**代码结构**：
```rust
// src/decision/mod.rs
pub struct DecisionSystem {
    observation: Arc<RwLock<ObservationSystem>>,
    action: Arc<RwLock<ActionSystem>>,
    llm: Arc<dyn LlmClient>,
    intent: IntentMatcher,
    router: CommandRouter,
}

impl DecisionSystem {
    /// 核心决策流程
    pub async fn decide(&mut self, input: UserInput) -> Result<Decision> {
        // 1. 识别意图
        let intent = self.intent.match_intent(&input)?;

        // 2. 收集上下文（从观测系统）
        let context = self.observation.read().await
            .gather_context(&intent).await?;

        // 3. 决策（使用 LLM 或规则）
        let decision = match intent {
            Intent::Shell(cmd) => self.decide_shell(cmd, context).await?,
            Intent::Suggest => self.decide_suggest(context).await?,
            Intent::LlmQuery(q) => self.decide_llm(q, context).await?,
            _ => self.router.route(intent, context).await?,
        };

        // 4. 执行行动
        let result = self.action.write().await
            .execute(&decision.action).await?;

        // 5. 记录观测
        self.observation.write().await
            .observe(SystemEvent::Decision { decision, result }).await?;

        Ok(decision)
    }
}
```

**验证**：
- Agent 的所有功能正常
- 用户体验无变化
- 性能无明显下降

### Phase 3: 离坎循环（2 周）

**目标**：实现自主学习的核心循环

#### Week 4: 简单循环

**任务**：
1. ✅ 实现简单的统计分析（无 LLM）
2. ✅ 从 ObservationSystem 提取模式
3. ✅ 向 ActionSystem 注入规则
4. ✅ 测试：循环能自主运行

**代码结构**：
```rust
// src/decision/li_kan_cycle.rs
pub struct LiKanCycle {
    config: CycleConfig,
}

impl LiKanCycle {
    /// 简单循环（统计分析）
    pub async fn simple_cycle(
        &self,
        observation: &ObservationSystem,
        action: &mut ActionSystem,
    ) -> Result<CycleReport> {

        // 收：从观测系统获取数据
        let observations = observation.query(
            ObservationQuery::recent(Duration::days(7))
        ).await?;

        // 炼：简单统计分析
        let patterns = self.extract_simple_patterns(&observations)?;

        // 发：注入到行动系统
        for pattern in patterns {
            if pattern.confidence > 0.8 {
                action.add_rule(pattern.to_rule())?;
            }
        }

        Ok(CycleReport {
            patterns_found: patterns.len(),
            observations_analyzed: observations.len(),
        })
    }

    /// 提取简单模式（频率、序列）
    fn extract_simple_patterns(&self, obs: &[Observation]) -> Result<Vec<Pattern>> {
        let mut patterns = Vec::new();

        // 1. 频率模式：统计命令频率
        let freq = self.analyze_frequency(obs)?;
        for (cmd, count) in freq {
            if count >= 5 {
                patterns.push(Pattern::Frequency {
                    command: cmd,
                    count,
                    confidence: (count as f64 / obs.len() as f64).min(1.0),
                });
            }
        }

        // 2. 序列模式：发现命令序列
        let sequences = self.analyze_sequence(obs)?;
        for seq in sequences {
            if seq.occurrences >= 3 {
                patterns.push(Pattern::Sequence {
                    commands: seq.commands,
                    confidence: seq.confidence,
                });
            }
        }

        Ok(patterns)
    }
}
```

**验证**：
- 能从历史数据中发现频繁命令
- 能自动生成建议规则
- Suggest 质量有所提升

#### Week 5: 增强循环

**任务**：
1. ✅ 增加 LLM 辅助的深度分析
2. ✅ 实现自主触发机制
3. ✅ 后台循环线程
4. ✅ 测试：长期运行，质量持续提升

**代码结构**：
```rust
impl LiKanCycle {
    /// 增强循环（包含 LLM 分析）
    pub async fn enhanced_cycle(
        &self,
        observation: &ObservationSystem,
        action: &mut ActionSystem,
        llm: &dyn LlmClient,
    ) -> Result<CycleReport> {

        // 收
        let observations = observation.query(...).await?;

        // 炼（三种方法）
        let stat_patterns = self.extract_simple_patterns(&observations)?;
        let llm_patterns = self.extract_llm_patterns(&observations, llm).await?;
        let combined = self.merge_patterns(stat_patterns, llm_patterns)?;

        // 发
        for pattern in combined {
            action.add_rule(pattern.to_rule())?;
        }

        Ok(CycleReport { ... })
    }

    /// LLM 辅助深度分析
    async fn extract_llm_patterns(
        &self,
        obs: &[Observation],
        llm: &dyn LlmClient,
    ) -> Result<Vec<Pattern>> {
        // 构造 prompt
        let prompt = format!(
            "分析以下用户行为，发现深层规律：\n{}",
            self.format_observations(obs)
        );

        // 调用 LLM
        let response = llm.chat(&prompt).await?;

        // 解析 LLM 返回的模式
        self.parse_llm_patterns(&response)
    }
}
```

**验证**：
- LLM 能发现更深层的规律
- 建议质量明显优于 Week 4
- 系统能自主学习用户习惯

### Phase 4: 完整迁移（2 周）

**目标**：将所有功能迁移到两仪架构

#### Week 6: 观测系统完整化

**任务**：
1. ✅ 迁移 Statistics → StatisticsObserver
2. ✅ 迁移 Trace → TraceObserver
3. ✅ 迁移 BlackBox → BlackBoxObserver
4. ✅ 迁移 History → HistoryObserver
5. ✅ 统一查询接口

**代码结构**：
```rust
// src/observation/observers/statistics.rs
pub struct StatisticsObserver {
    // 复用现有实现
    inner: Statistics,
}

// src/observation/observers/trace.rs
pub struct TraceObserver {
    inner: Tracer,
}

// ... 其他 Observer
```

**验证**：
- 所有观测功能正常
- 可以跨维度查询
- 性能无明显下降

#### Week 7: 行动系统完整化

**任务**：
1. ✅ 迁移 Task → PlannerActor
2. ✅ 迁移 Shell → ExecutorActor
3. ✅ 迁移 ToolCalling → ToolCallerActor
4. ✅ 迁移 LLM Response → ResponderActor
5. ✅ 统一执行接口

**代码结构**：
```rust
// src/action/actors/planner.rs
pub struct PlannerActor {
    inner: TaskSystem,
}

// src/action/actors/executor.rs
pub struct ExecutorActor {
    inner: ShellExecutor,
}

// ... 其他 Actor
```

**验证**：
- 所有行动功能正常
- 可以统一调度
- 用户体验无变化

### Phase 5: 优化与完善（1 周）

**目标**：清理冗余，优化性能，完善文档

#### Week 8: 清理与优化

**任务**：
1. ✅ 移除冗余代码
2. ✅ 优化性能瓶颈
3. ✅ 更新文档
4. ✅ 增加测试覆盖率
5. ✅ 用户指南更新

**清理清单**：
- 移除未使用的配置项
- 合并重复的日志
- 简化复杂的接口
- 统一命名规范

**文档更新**：
- 两仪架构设计文档
- 开发者迁移指南
- 用户使用指南
- API 参考文档

---

## 📋 具体实施细节

### 一、代码迁移策略

#### 原则

1. **包装而非重写**：复用现有实现，用 trait 包装
2. **渐进式**：一次迁移一个模块
3. **向后兼容**：保留旧接口作为过渡
4. **测试先行**：迁移前后测试对比

#### 示例：Memory 迁移

**Before**：
```rust
// src/memory.rs
pub struct ConversationMemory {
    entries: Vec<MemoryEntry>,
}

impl ConversationMemory {
    pub fn add(&mut self, role: &str, content: &str) { ... }
    pub fn search(&self, query: &str) -> Vec<&MemoryEntry> { ... }
}
```

**After（保留原实现）**：
```rust
// src/observation/observers/memory.rs
use crate::memory::ConversationMemory; // 复用

pub struct MemoryObserver {
    inner: ConversationMemory, // 包装
}

impl Observer for MemoryObserver {
    fn observe(&mut self, event: &SystemEvent) -> Result<Observation> {
        // 适配层
        match event {
            SystemEvent::LlmInteraction { role, content, .. } => {
                self.inner.add(role, content); // 调用原实现
                Ok(Observation::Memory { ... })
            }
            _ => Ok(Observation::Skip),
        }
    }

    fn query(&self, query: &ObservationQuery) -> Result<Vec<Observation>> {
        // 适配层
        let results = self.inner.search(&query.keyword); // 调用原实现
        Ok(results.into_iter().map(|e| Observation::from(e)).collect())
    }
}
```

**优点**：
- 原有代码完全保留
- 只增加薄薄的适配层
- 迁移风险极低
- 可以逐步优化内部实现

### 二、测试策略

#### 迁移前测试

```rust
#[cfg(test)]
mod before_migration {
    use super::*;

    #[tokio::test]
    async fn test_memory_basic() {
        let mut memory = ConversationMemory::new();
        memory.add("user", "hello");
        memory.add("assistant", "hi");

        let results = memory.search("hello");
        assert_eq!(results.len(), 1);
    }
}
```

#### 迁移后测试（确保行为一致）

```rust
#[cfg(test)]
mod after_migration {
    use super::*;

    #[tokio::test]
    async fn test_memory_observer() {
        let mut observer = MemoryObserver::new();

        observer.observe(&SystemEvent::LlmInteraction {
            role: "user",
            content: "hello",
        }).await.unwrap();

        observer.observe(&SystemEvent::LlmInteraction {
            role: "assistant",
            content: "hi",
        }).await.unwrap();

        let results = observer.query(&ObservationQuery {
            keyword: "hello",
            ..Default::default()
        }).await.unwrap();

        assert_eq!(results.len(), 1); // 行为一致！
    }
}
```

### 三、性能监控

#### 关键指标

```rust
// src/metrics.rs
pub struct PerformanceMetrics {
    // 决策延迟
    pub decision_latency: Duration,

    // 观测系统延迟
    pub observation_latency: HashMap<ObservationDimension, Duration>,

    // 行动系统延迟
    pub action_latency: HashMap<ActionType, Duration>,

    // 循环耗时
    pub cycle_duration: Duration,
}
```

#### 监控点

```rust
impl DecisionSystem {
    pub async fn decide(&mut self, input: UserInput) -> Result<Decision> {
        let start = Instant::now();

        // ... 决策逻辑

        let latency = start.elapsed();
        self.metrics.record_decision_latency(latency);

        if latency > Duration::from_millis(100) {
            tracing::warn!("Slow decision: {:?}", latency);
        }

        Ok(decision)
    }
}
```

---

## 🎯 验收标准

### Phase 1: 两仪框架

**功能**：
- ✅ ObservationSystem 和 ActionSystem 基础建立
- ✅ Memory 和 Suggest 成功迁移
- ✅ 所有现有功能正常工作

**性能**：
- 决策延迟 < 100ms
- 内存增长 < 10%

**代码质量**：
- 测试覆盖率 > 80%
- Clippy 零警告
- 文档完整

### Phase 2: 决策系统

**功能**：
- ✅ DecisionSystem 整合完成
- ✅ Agent 逻辑保持不变
- ✅ 用户体验无变化

**性能**：
- 整体延迟无明显增加
- 内存占用稳定

### Phase 3: 离坎循环

**功能**：
- ✅ 简单循环运行正常
- ✅ 能自动提取模式
- ✅ 建议质量有提升

**效果**：
- 发现至少 3 种频繁模式
- 建议准确率提升 > 10%

**性能**：
- 循环耗时 < 1 秒
- 不影响主流程

### Phase 4: 完整迁移

**功能**：
- ✅ 所有模块迁移完成
- ✅ 所有测试通过
- ✅ 无功能退化

**代码**：
- 冗余代码移除
- 命名规范统一
- 文档完整更新

### Phase 5: 最终验收

**用户体验**：
- 系统响应流畅
- 建议质量优秀
- 无明显 bug

**开发体验**：
- 代码结构清晰
- 易于理解和扩展
- 文档齐全

**维护性**：
- 模块职责明确
- 接口简洁统一
- 测试覆盖充分

---

## 📚 文档计划

### 设计文档

1. ✅ `liangyyi-evolution-plan.md`（本文档）
2. ✅ `bagua-memory-palace-design.md`（哲学基础）
3. 🆕 `two-yi-architecture.md`（详细架构）
4. 🆕 `migration-guide.md`（迁移指南）

### 开发文档

1. 🆕 `observer-trait-guide.md`（如何实现 Observer）
2. 🆕 `actor-trait-guide.md`（如何实现 Actor）
3. 🆕 `li-kan-cycle-internals.md`（循环内部机制）

### 用户文档

1. ✅ 更新 `user-guide.md`（用户手册）
2. ✅ 更新 `QUICKSTART.md`（快速开始）
3. 🆕 `advanced-features.md`（高级特性）

---

## 🌟 哲学总结

### 思想层面

**易的体悟**：
- **简易**：两仪架构，简单明了
- **变易**：渐进演化，持续改进
- **不易**：核心循环，永恒规律

**一分为三的应用**：
- 系统：观测 - 决策 - 行动
- 循环：收 - 炼 - 发
- 触发：量 - 时 - 质

### 实践层面

**继承优秀**：
- Suggest（三源融合）
- Task（智能编排）
- LLM（流式体验）

**淘汰混乱**：
- 重复的观测逻辑
- 碎片化的上下文
- 过度的配置

**渐进演化**：
- 每周交付价值
- 保持系统可用
- 持续优化改进

### 目标愿景

**令人舒服的系统**：
- 设计：清晰的两仪架构，易于理解
- 实现：简洁的代码，易于维护
- 应用：流畅的体验，易于使用
- 维护：完善的文档，易于扩展

---

## 📅 时间表

| 阶段 | 时间 | 目标 | 交付物 |
|------|------|------|--------|
| Phase 1 | Week 1-2 | 两仪框架 | ObservationSystem + ActionSystem |
| Phase 2 | Week 3 | 决策整合 | DecisionSystem |
| Phase 3 | Week 4-5 | 离坎循环 | LiKanCycle（简单+增强）|
| Phase 4 | Week 6-7 | 完整迁移 | 所有模块迁移 |
| Phase 5 | Week 8 | 优化完善 | 清理、文档、测试 |

**总计**：8 周（2 个月）

---

**制定者**: RealConsole Team
**审核者**: 待定
**状态**: ✅ 计划完成，等待执行

---

> "思想要超越，行动要务实"
> "继承优秀，淘汰混乱"
> "渐进演化，持续改进"
>
> 让系统在演化中走向完美！
> 🌊🔥☯️
