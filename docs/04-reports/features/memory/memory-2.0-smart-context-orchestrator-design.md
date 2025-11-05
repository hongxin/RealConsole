# Memory 2.0: 智能上下文编排器设计方案

**创建时间**: 2025-11-05
**状态**: 设计完成 - 待评审
**版本**: v2.0
**设计哲学**: 一分为三（感知 → 理解 → 编排）

---

## 目录

- [一、背景与动机](#一背景与动机)
- [二、四维观测体系分析](#二四维观测体系分析)
- [三、Memory 2.0 定位](#三memory-20-定位)
- [四、三态架构设计](#四三态架构设计)
- [五、核心数据结构](#五核心数据结构)
- [六、实施路线图](#六实施路线图)
- [七、风险控制](#七风险控制)
- [八、与现有系统的关系](#八与现有系统的关系)
- [九、一分为三哲学体现](#九一分为三哲学体现)
- [十、决策记录](#十决策记录)

---

## 一、背景与动机

### 1.1 Memory 1.0 的问题

根据 `memory-system-redesign.md` 的分析，Memory 1.0 存在根本性错位：

**初心**：
- 学习 Claude Code，应对 LLM 128k 上下文限制
- 智能上下文管理，从历史中提取相关信息
- 为 LLM 提供精选上下文

**现实**：
- 变成"第五个全量记录器"
- 与 History、ExecutionLogger、Context、LlmLogger 产生 250-300% 数据冗余
- 简单的关键词搜索，没有"智能性"
- 用户困惑：`/memory` vs `/history` vs `/log` 有什么区别？

**结论**：Memory 想做的（智能上下文选择）和实际做的（全量记录）完全不一致！

### 1.2 设计动机

基于"一分为三"哲学，寻找第三条演化路径：

```
❌ 路径一：保留 Memory 1.0（全量记录，冗余）
❌ 路径二：完全废弃 Memory（失去长期记忆能力）
✅ 路径三：转型为智能上下文编排器（主动选择，零冗余）
```

**核心洞察**（用户原话）：
> "其本质都是最后将什么样的合适且有限长度的内容灌输给聪明大模型，从而推动整个处理分析计算任务往前走。"

**Memory 2.0 的使命**：**Smart Context Orchestrator**（智能上下文编排器）

---

## 二、四维观测体系分析

### 2.1 现状架构（v1.16.5）

```
┌─────────────────────────────────────────────────────────────┐
│                   四维观测体系（v1.16.5）                      │
├───────────────┬──────────────┬──────────────┬───────────────┤
│ 统计维度       │ 协同维度      │ 黑盒维度      │ 记忆维度       │
│ History       │ExecutionLogger│ LlmLogger    │ Context       │
├───────────────┼──────────────┼──────────────┼───────────────┤
│ 观察"习惯"     │ 观察"流程"    │ 观察"细节"    │ 观察"状态"     │
│ 去重统计       │ 全量追踪      │ API 详情      │ 工作记忆       │
│ 1000 条        │ 1000 条       │ 无限制        │ 9 轮对话       │
│ JSON 持久化    │ VecDeque     │ JSONL 持久化  │ 非持久化       │
└───────────────┴──────────────┴──────────────┴───────────────┘
                              ↓
              UnifiedTracer - 统一查询接口（v1.16.5）
```

### 2.2 四维边界与留白空间

| 维度 | 核心能力 | 无法做到的事 |
|------|---------|-------------|
| **History** | 去重统计、频率分析、综合得分排序 | ❌ 不理解"哪些历史对当前任务相关" |
| **ExecutionLogger** | 时序追踪、耗时分析、结果预览 | ❌ 不能"从历史中提取相关经验" |
| **LlmLogger** | Token 统计、API 参数、完整 messages | ❌ 不做"基于任务的上下文优化" |
| **Context** | 9 轮对话、参数收集、状态管理 | ❌ 超过 9 轮就遗忘，无长期记忆检索 |

**关键发现**：**四维系统都是"被动记录器"，没有一个能做"主动智能选择"**

这正是 Memory 2.0 的价值空间！

---

## 三、Memory 2.0 定位

### 3.1 核心定位

**不是记录器（Recorder）**，而是**编排器（Orchestrator）**：

```
输入：当前任务意图 + 四维系统的全量数据
  ↓
智能处理：语义理解、相关性排序、智能压缩
  ↓
输出：最相关的上下文片段（控制在 token 预算内）
```

### 3.2 与四维系统的关系

```
┌──────────────────────────────────────────────────────────┐
│              Memory 2.0: 智能上下文编排器                  │
│        （第五维度 - 从"被动记录"到"主动选择"）              │
├──────────────────────────────────────────────────────────┤
│  🎯 核心能力：从历史中提取最相关的上下文                    │
│                                                            │
│  输入：当前任务 "调试 Rust 的 trait 问题"                  │
│    ↓                                                       │
│  查询四维：                                                 │
│    • History: 最近使用的 Rust 命令                         │
│    • ExecutionLogger: 最近的 Rust 相关执行记录             │
│    • LlmLogger: 过去讨论 trait 的对话                      │
│    • Context: 当前对话状态（9 轮）                         │
│    ↓                                                       │
│  语义匹配 + 时间衰减 + 重要性评分                           │
│    ↓                                                       │
│  输出：【相关上下文片段，控制在 2000 tokens】               │
│    1. 上周解决过类似 trait 问题的对话（高相关）             │
│    2. 最近 3 次 cargo build 的错误信息（中相关）            │
│    3. 常用的 Rust debug 命令（低相关）                     │
└──────────────────────────────────────────────────────────┘
```

**关键设计原则**：
1. **不重复记录**：从四维系统读取数据，不自己存储
2. **按需查询**：只在需要时（用户请求、LLM 上下文不足）才触发
3. **智能排序**：语义相关性 > 时间新鲜度 > 访问频率

---

## 四、三态架构设计

基于"一分为三"哲学，Memory 2.0 分为三个层次：

```
┌─────────────────────────────────────────────────────────────┐
│                  Memory 2.0 三态架构                         │
│            （感知 → 理解 → 编排）                            │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  【态一】感知层（Perception Layer）                           │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • 从四维系统采集原始数据                                      │
│  • 统一数据模型（TraceEntry → ContextChunk）                  │
│  • 基础过滤（时间范围、类型筛选）                              │
│                                                               │
│  【态二】理解层（Understanding Layer）                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • 语义分析（关键词提取、实体识别）                            │
│  • 相关性评分（任务匹配度计算）                                │
│  • 重要性推断（频率、成功率、用户反馈）                        │
│                                                               │
│  【态三】编排层（Orchestration Layer）                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • 上下文优化（去重、压缩、摘要）                              │
│  • Token 预算管理（控制在限制内）                             │
│  • 结构化输出（为 LLM 友好的格式）                            │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 4.1 态一：感知层（Perception Layer）

**职责**：数据采集与统一

```rust
/// 感知层：从四维系统采集数据
pub struct PerceptionLayer {
    unified_tracer: Arc<UnifiedTracer>,
    context_tracker: Arc<RwLock<ContextTracker>>, // ✅ 复用现有！
}

impl PerceptionLayer {
    /// 采集原始数据
    async fn collect_raw_data(
        &self,
        time_range: Option<TimeRange>,
        types: Vec<Dimension>,
    ) -> Result<Vec<ContextChunk>>;

    /// 提取当前上下文实体（利用 ContextTracker）
    async fn extract_current_entities(&self, task: &str) -> Vec<Entity>;
}
```

**设计要点**：
- ✅ 不自己存储：直接从 UnifiedTracer 读取
- ✅ 复用 ContextTracker：实体提取不重复造轮子
- ✅ 统一数据模型：ContextChunk 作为后续处理的基础

### 4.2 态二：理解层（Understanding Layer）

**职责**：语义分析与相关性评分

```rust
/// 理解层：语义分析与评分
pub struct UnderstandingLayer {
    // 方案 A：轻量级实现（无外部依赖）
    keyword_matcher: KeywordMatcher,

    // 方案 B：增强实现（可选 embedding）
    embedding_engine: Option<Arc<EmbeddingEngine>>,
}

impl UnderstandingLayer {
    /// 计算相关性得分
    ///
    /// 综合得分 = (关键词匹配 70% + 语义相似度 30%) × 时间衰减
    async fn score_relevance(
        &self,
        task: &str,
        chunks: Vec<ContextChunk>,
    ) -> Vec<ContextChunk>;

    /// 推断重要性
    ///
    /// 因子：执行成功率、频率、Token 消耗、用户标记
    fn infer_importance(&self, chunk: &mut ContextChunk);
}
```

**评分算法**：

1. **相关性得分**：
   - 关键词匹配（TF-IDF 或 Jaccard）：70% 权重
   - 语义相似度（embedding，可选）：30% 权重

2. **时间衰减**：
   - 指数衰减，半衰期 7 天
   - `decay = exp(-age_days / 7.0)`

3. **重要性得分**：
   - 基准分：0.5
   - 执行成功：+0.1，失败：-0.2
   - 高频命令：+0.3（对数缩放）
   - 长对话（> 2000 tokens）：+0.2
   - 用户标记：+0.5

**设计要点**：
- ✅ 双轨实现：轻量级（关键词）+ 可选增强（embedding）
- ✅ 多因子评分：相关性 + 时间衰减 + 重要性
- ✅ 渐进增强：先实现方案 A，未来可升级到方案 B

### 4.3 态三：编排层（Orchestration Layer）

**职责**：上下文优化与 Token 管理

```rust
/// 编排层：上下文优化与组装
pub struct OrchestrationLayer {
    token_counter: TokenCounter,
    compressor: ContextCompressor,
}

impl OrchestrationLayer {
    /// 构建优化的上下文
    ///
    /// 贪心算法：按 relevance × importance 排序
    /// 超出预算时尝试压缩
    async fn build_optimized_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: usize,  // Token 预算（如 2000）
    ) -> Result<OptimizedContext>;

    /// 去重相似内容
    fn deduplicate(&self, chunks: Vec<ContextChunk>) -> Result<Vec<ContextChunk>>;

    /// 生成上下文摘要
    fn generate_summary(&self, chunks: &[ContextChunk]) -> Result<String>;
}
```

**Token 管理策略**：

1. **贪心选择**：
   - 按 `relevance × importance` 排序
   - 优先选择高分片段
   - 低于阈值（0.3）停止选择

2. **智能压缩**：
   - 超出预算时，对低重要性内容压缩
   - 保留关键句，去除冗余格式
   - 可选：调用 LLM 生成摘要

3. **去重优化**：
   - 编辑距离或 Jaccard 相似度检测
   - 相似内容合并

**设计要点**：
- ✅ Token 预算管理：严格控制不超出限制
- ✅ 智能压缩：对低重要性但高相关性的内容进行摘要
- ✅ 去重优化：避免重复内容浪费 token

---

## 五、核心数据结构

### 5.1 ContextChunk（统一上下文片段）

```rust
/// 统一的上下文片段
#[derive(Debug, Clone)]
pub struct ContextChunk {
    id: Uuid,
    timestamp: DateTime<Utc>,
    dimension: Dimension,        // 来自哪个维度
    content: String,             // 原始内容
    metadata: HashMap<String, Value>, // 元数据

    // 后续层级填充
    relevance_score: Option<f64>,    // 理解层填充
    importance_score: Option<f64>,   // 理解层填充
    compressed: Option<String>,      // 编排层填充
}

impl ContextChunk {
    fn from_trace_entry(entry: TraceEntry) -> Self;
}
```

### 5.2 OptimizedContext（优化后的上下文）

```rust
/// 优化后的上下文
pub struct OptimizedContext {
    pub chunks: Vec<ContextChunk>,
    pub total_tokens: usize,
    pub summary: String, // 元信息，便于调试
}
```

### 5.3 SmartContextOrchestrator（主接口）

```rust
/// Memory 2.0: 智能上下文编排器
pub struct SmartContextOrchestrator {
    perception: PerceptionLayer,
    understanding: UnderstandingLayer,
    orchestration: OrchestrationLayer,
}

impl SmartContextOrchestrator {
    /// 核心方法：为当前任务提取相关上下文
    pub async fn extract_relevant_context(
        &self,
        task: &str,
        token_budget: usize,
    ) -> Result<OptimizedContext>;

    /// 辅助方法：为 LLM 构建系统提示词
    pub fn build_system_prompt(&self, context: &OptimizedContext) -> String;
}
```

---

## 六、实施路线图

### 6.1 四阶段渐进式实施（2-3 个月）

```
Phase 1: 基础设施（2 周）
  ↓
Phase 2: 轻量级实现（3 周）
  ↓
Phase 3: 增强优化（3 周）
  ↓
Phase 4: 生产就绪（2 周）
```

#### Phase 1：基础设施搭建（2 周）

**Week 1：感知层**

任务清单：
- [ ] 定义 ContextChunk 数据结构
- [ ] 实现 PerceptionLayer::collect_raw_data()
- [ ] 集成 ContextTracker（已有，直接复用）
- [ ] 单元测试：从 UnifiedTracer 读取数据

验收标准：
- ✅ 能从四维系统采集 500 条数据
- ✅ ContextChunk 转换无损失
- ✅ ContextTracker 实体提取工作正常

**Week 2：理解层（轻量级版本）**

任务清单：
- [ ] 实现 KeywordMatcher（TF-IDF 或 Jaccard）
- [ ] 实现时间衰减算法
- [ ] 实现重要性推断（基于元数据）
- [ ] 单元测试：相关性评分准确性

验收标准：
- ✅ 关键词匹配准确率 > 70%
- ✅ 时间衰减曲线符合预期
- ✅ 重要性评分合理

#### Phase 2：轻量级实现（3 周）

**Week 3：编排层（基础版本）**

任务清单：
- [ ] 实现 TokenCounter（使用 tiktoken-rs 或简化版）
- [ ] 实现贪心选择算法
- [ ] 实现基础去重（编辑距离）
- [ ] 集成测试：端到端流程

验收标准：
- ✅ Token 计数误差 < 5%
- ✅ 贪心算法不超出预算
- ✅ 去重率 > 30%

**Week 4-5：命令接口与集成**

任务清单：
- [ ] 实现 `/memory extract <任务描述> [--budget 2000]`
- [ ] 实现 `/memory preview` - 预览当前上下文选择
- [ ] 集成到 Agent 主循环（可选触发）
- [ ] 编写用户文档

验收标准：
- ✅ 命令响应时间 < 500ms（500 条数据）
- ✅ 输出格式清晰易读
- ✅ 与 LLM 集成无冲突

#### Phase 3：增强优化（3 周）

**Week 6-7：语义理解增强（可选）**

任务清单：
- [ ] 集成 embedding 引擎（rust-bert 或 API）
- [ ] 实现向量相似度计算
- [ ] A/B 测试：关键词 vs 语义

验收标准：
- ✅ 语义匹配准确率 > 85%
- ✅ 响应时间 < 1s

**Week 8：压缩与摘要**

任务清单：
- [ ] 实现智能截断（保留关键句）
- [ ] 集成 LLM 摘要（可选）
- [ ] 实现结构化压缩

验收标准：
- ✅ 压缩率 > 40%
- ✅ 摘要准确性 > 80%

#### Phase 4：生产就绪（2 周）

**Week 9：性能优化**

任务清单：
- [ ] 并发处理（tokio::spawn 并行查询四维）
- [ ] 缓存热点数据（LRU 缓存）
- [ ] 批量操作优化

性能基准：
- ✅ 1000 条数据 < 200ms
- ✅ 5000 条数据 < 1s
- ✅ 并发 10 次请求 < 300ms

**Week 10：文档与测试**

任务清单：
- [ ] 完整单元测试（覆盖率 > 80%）
- [ ] 集成测试（端到端场景）
- [ ] 压力测试（10,000 条数据）
- [ ] 用户文档 + 开发者文档

验收标准：
- ✅ 所有测试通过
- ✅ 文档完整清晰
- ✅ 准备好发布

### 6.2 成功标准（Definition of Done）

**功能完整性**：
- ✅ 三态架构全部实现
- ✅ 轻量级方案可用（不依赖 embedding）
- ✅ 命令接口完善

**性能指标**：
- ✅ 500 条数据处理 < 500ms
- ✅ 1000 条数据处理 < 1s
- ✅ Token 预算控制准确率 > 95%

**质量指标**：
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试通过率 100%
- ✅ 相关性匹配准确率 > 70%（关键词）或 > 85%（语义）

**用户体验**：
- ✅ 文档完整
- ✅ 错误提示清晰
- ✅ 响应格式友好

---

## 七、风险控制

### 7.1 风险矩阵

| 风险类型 | 概率 | 影响 | 应对策略 |
|---------|------|------|---------|
| **技术风险** | | | |
| 性能不达标（> 1s） | 中 | 高 | ✅ 分层缓存 + 异步处理 + 降级方案 |
| embedding 依赖复杂 | 高 | 中 | ✅ 双轨实现，方案 A 不依赖 embedding |
| Token 计数不准确 | 中 | 中 | ✅ 使用 tiktoken-rs，留 10% 余量 |
| **产品风险** | | | |
| 用户不理解如何使用 | 高 | 高 | ✅ 丰富文档 + 示例 + 智能默认值 |
| 与现有功能冲突 | 中 | 高 | ✅ 可选开关，不强制启用 |
| 输出质量不稳定 | 中 | 中 | ✅ A/B 测试 + 用户反馈机制 |
| **架构风险** | | | |
| UnifiedTracer API 变动 | 低 | 高 | ✅ 定义适配器层，隔离变化 |
| 四维数据不一致 | 低 | 中 | ✅ 容错处理，部分失败不影响整体 |
| 内存占用过高 | 中 | 中 | ✅ 流式处理，不一次性加载全量 |

### 7.2 关键风险应对

**过度设计风险**：

投入：2-3 个月开发时间
收益：解决 Context 9 轮限制、应对 128k 上下文管理

应对策略：
- 先完成 Phase 1-2（3-4 周）
- 评估用户使用率（目标 > 10%）
- 如果 3 个月内使用频率 < 10%，考虑简化或废弃

---

## 八、与现有系统的关系

### 8.1 五维体系最终架构

```
┌────────────────────────────────────────────────────────────────┐
│                  RealConsole 五维观测体系                        │
│               （四维被动记录 + 一维主动编排）                     │
├────────────────────────────────────────────────────────────────┤
│  【第一维】Statistics - 统计维度                                 │
│    History: 命令使用习惯，去重统计，频率分析                      │
│    命令: /history                                                │
│                                                                  │
│  【第二维】Coordination - 协同维度                               │
│    ExecutionLogger: 端到端执行追踪，性能分析                     │
│    命令: /log                                                    │
│                                                                  │
│  【第三维】BlackBox - 黑盒维度                                   │
│    LlmLogger: API 调用详情，Token 统计                           │
│    命令: /llm-log                                                │
│                                                                  │
│  【第四维】Memory - 记忆维度                                     │
│    Context: 对话连贯性，9 轮工作记忆                             │
│    命令: /context                                                │
│                                                                  │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━          │
│                                                                  │
│  【第五维】Intelligence - 智能维度 ⭐ NEW                        │
│    Memory 2.0: 智能上下文编排，从历史中提取相关信息              │
│    命令: /memory extract <任务> [--budget <tokens>]              │
│         /memory preview                                          │
│                                                                  │
└────────────────────────────────────────────────────────────────┘
```

### 8.2 命令对比

| 命令 | 维度 | 用途 | Memory 2.0 的关系 |
|------|------|------|------------------|
| `/history` | 统计 | 查看命令使用习惯 | 📥 **数据源** |
| `/log` | 协同 | 查看执行追踪 | 📥 **数据源** |
| `/llm-log` | 黑盒 | 查看 LLM 调用详情 | 📥 **数据源** |
| `/context` | 记忆 | 管理当前对话上下文 | 🔄 **互补**（短期 vs 长期） |
| `/trace` | 统一 | 跨维度查询 | 🔄 **互补**（全量 vs 精选） |
| `/memory extract` | 智能 | 提取相关上下文 | ⭐ **新能力** |

**关键区别**：
- **被动 vs 主动**：前四维是"被动记录"，Memory 2.0 是"主动选择"
- **全量 vs 精选**：前四维提供"全部数据"，Memory 2.0 提供"相关数据"
- **过去 vs 未来**：前四维回顾"发生了什么"，Memory 2.0 预测"什么有用"

---

## 九、一分为三哲学体现

### 9.1 太极（一）- 统一的本质

```
Memory 的本质：为 LLM 提供最优上下文
```

### 9.2 两仪（二）- 两种极端路径

```
阳：全量记录（Memory 1.0 的错误）
  → 250% 冗余，用户困惑

阴：完全废弃（简单粗暴）
  → 失去长期记忆能力
```

### 9.3 三生万物 - Memory 2.0 的第三条路

```
         感知（采集数据）
              ↓
         理解（分析相关性）
              ↓
         编排（优化组装）

不是"记录"，也不是"废弃"
而是"智能选择" —— 第三态
```

### 9.4 易变体现

- **可进可退**：轻量级 MVP 快速验证，失败成本低
- **顺应变化**：双轨实现（关键词 + 语义），渐进增强
- **动态平衡**：从四维读取（不重复存储），与现有系统和谐共存

---

## 十、决策记录

### 决策 #1：三态架构而非单层设计

**日期**: 2025-11-05
**决策**: 采用感知-理解-编排三层架构
**理由**:
- 符合"一分为三"哲学
- 职责分离，易于测试和扩展
- 每层可独立优化

**替代方案**: 单层 Pipeline 设计
**为何不选**: 耦合度高，难以替换算法

---

### 决策 #2：双轨实现（关键词 + 语义）

**日期**: 2025-11-05
**决策**: 轻量级关键词匹配 + 可选 embedding 增强
**理由**:
- 降低初始复杂度
- 避免依赖重型库
- 渐进式增强

**替代方案**: 直接实现语义匹配
**为何不选**: 风险高，依赖复杂，延长上线时间

---

### 决策 #3：不自己存储数据

**日期**: 2025-11-05
**决策**: 从 UnifiedTracer 读取，不建立独立存储
**理由**:
- 避免数据冗余（Memory 1.0 的教训）
- 保持架构简洁
- 降低维护成本

**替代方案**: 建立独立的向量数据库
**为何不选**: 过度设计，增加复杂度

---

### 决策 #4：可选开关，默认关闭

**日期**: 2025-11-05
**决策**: Memory 2.0 作为可选功能，不强制启用
**理由**:
- 降低用户迁移成本
- 观察实际使用效果
- 避免影响现有工作流

**替代方案**: 默认启用，自动增强 LLM 上下文
**为何不选**: 可能影响性能，用户感知不明确

---

## 十一、示例用法

### 11.1 命令行示例

```bash
> /memory extract "调试 Rust trait 问题" --budget 2000

✅ 已提取相关上下文（共 1850 tokens）

[1] Statistics | 2025-10-15 | 相关度 0.89
  cargo build --verbose  (执行 23 次)

[2] BlackBox | 2025-10-12 | 相关度 0.85
  对话摘要：讨论了 Trait 生命周期标注问题...
  Token: 1200 | 耗时: 3.2s

[3] Coordination | 2025-10-10 | 相关度 0.76
  执行 cargo clippy，发现 trait bound 警告...

📊 来源分布：History(2) | ExecutionLogger(3) | LlmLogger(2)
⏱️ 时间跨度：最近 15 天
```

### 11.2 API 示例

```rust
let orchestrator = SmartContextOrchestrator::new(...);

// 提取相关上下文
let context = orchestrator
    .extract_relevant_context("调试 Rust trait 问题", 2000)
    .await?;

// 构建 LLM 提示词
let system_prompt = orchestrator.build_system_prompt(&context);

// 发送给 LLM
let messages = vec![
    Message {
        role: MessageRole::System,
        content: system_prompt,
    },
    Message {
        role: MessageRole::User,
        content: "如何解决 trait bound 不满足的错误？",
    },
];

let response = llm.chat(messages).await?;
```

---

## 十二、参考资料

### 内部文档

- `docs/04-reports/features/memory/memory-system-redesign.md` - Memory 1.0 问题分析
- `docs/00-core/philosophy.md` - 一分为三哲学
- `docs/00-core/vision.md` - 产品愿景

### 外部参考

- Claude Code 的上下文管理机制
- LangChain 的 Memory 模块设计
- 向量数据库（Qdrant, Milvus）的相似度检索

---

## 十三、变更历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|---------|
| v2.0 | 2025-11-05 | Claude + hongxin | 完整设计方案，记录深度思考 |

---

**文档状态**: 设计完成 - 待评审
**下一步**: 团队评审，决定是否启动 Phase 1 实施
**负责人**: hongxin
