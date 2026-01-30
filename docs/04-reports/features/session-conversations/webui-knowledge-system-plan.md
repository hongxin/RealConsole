# WebUI 知识系统改进计划

> **版本**: v2.3.0 规划
> **创建日期**: 2026-01-30
> **状态**: 规划中
> **核心原则**: CLI 保持稳定，WebUI 重点演进

---

## 背景与动机

基于对 RealConsole 与 Claude Code 22 天对话历史（81MB、10,194 条消息）的深度分析，识别出以下关键改进方向：

### 分析发现

1. **对话历史利用率低** - 81MB 知识资产仅做简单备份，未转化为可检索知识
2. **Agent 工作流不可视** - 任务分解和执行过程黑盒化
3. **哲学理念落地不足** - "一分为三"设计未完全融入代码
4. **协作模式可优化** - 对话效率和模式可分析改进

### 改进优先级

| 优先级 | 改进项 | 收益 | 实现位置 |
|--------|--------|------|----------|
| **P0** | 对话知识图谱 | 知识复用 | WebUI 新功能 |
| **P1** | Agent 工作流可视化 | 开发效率 | WebUI 新功能 |
| **P2** | 一分为三渐进落地 | 产品差异化 | 核心架构 |
| **P3** | 对话模式分析 | 长期优化 | 分析工具 |

---

## 一、对话知识图谱系统 (P0)

### 1.1 目标

将 Claude Code 对话历史转化为可检索、可导航的知识库，在 WebUI 中提供智能访问。

### 1.2 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    WebUI 知识图谱界面                        │
│  ┌──────────────┬──────────────┬──────────────────────────┐ │
│  │   时间线视图   │   主题视图    │      搜索/检索视图       │ │
│  └──────────────┴──────────────┴──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            ↑ API
┌─────────────────────────────────────────────────────────────┐
│                    知识服务层 (Rust)                         │
│  ┌──────────────┬──────────────┬──────────────────────────┐ │
│  │  索引构建器   │  语义分析器   │       检索引擎          │ │
│  │ IndexBuilder │ SemanticProc │    RetrievalEngine      │ │
│  └──────────────┴──────────────┴──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                            ↑
┌─────────────────────────────────────────────────────────────┐
│                    数据存储层                                │
│  ┌──────────────┬──────────────┬──────────────────────────┐ │
│  │  原始对话     │   元数据索引   │    知识摘要缓存         │ │
│  │  (JSON/JSONL)│   (SQLite)    │    (JSON)              │ │
│  └──────────────┴──────────────┴──────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 1.3 功能模块

#### 模块 A: 对话导入与索引

```rust
// src/web/knowledge/importer.rs
pub struct ConversationImporter {
    /// 导入 Claude Code 对话历史
    pub async fn import_from_claude_dir(&self, path: &Path) -> Result<ImportStats>;

    /// 导入备份 JSON 文件
    pub async fn import_from_backup(&self, backup_file: &Path) -> Result<ImportStats>;

    /// 增量更新（只导入新内容）
    pub async fn incremental_update(&self) -> Result<UpdateStats>;
}
```

#### 模块 B: 主题提取与分类

```rust
// src/web/knowledge/topic.rs
pub struct TopicExtractor {
    /// 提取对话主题
    pub fn extract_topics(&self, messages: &[Message]) -> Vec<Topic>;

    /// 识别关键决策点
    pub fn identify_decisions(&self, messages: &[Message]) -> Vec<Decision>;

    /// 提取代码变更关联
    pub fn extract_code_changes(&self, messages: &[Message]) -> Vec<CodeChange>;
}

pub struct Topic {
    pub id: String,
    pub name: String,
    pub keywords: Vec<String>,
    pub message_ids: Vec<String>,
    pub time_range: (DateTime, DateTime),
    pub summary: String,
}
```

#### 模块 C: 知识检索

```rust
// src/web/knowledge/retrieval.rs
pub struct KnowledgeRetrieval {
    /// 关键词搜索
    pub fn search_keyword(&self, query: &str) -> Vec<SearchResult>;

    /// 时间范围查询
    pub fn search_by_time(&self, start: DateTime, end: DateTime) -> Vec<Message>;

    /// 主题浏览
    pub fn browse_topics(&self) -> Vec<TopicSummary>;

    /// 相关对话推荐
    pub fn recommend_related(&self, context: &str) -> Vec<Message>;
}
```

### 1.4 WebUI 界面设计

#### 界面 1: 知识浏览器

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 对话知识库                              [🔍 搜索...]     │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ 📅 时间线                                                    │
│ ├─ 2026-01-30 (今天)                                        │
│ │   └─ 对话历史备份功能实现                                  │
│ ├─ 2026-01-19                                               │
│ │   └─ Memory 2.0 智能增强                                  │
│ ├─ 2026-01-16 ~ 01-19                                       │
│ │   └─ Unified Notebook Mode                                │
│ └─ 2026-01-08 ~ 01-11                                       │
│     └─ Storage Layer 2.0 大重构                             │
│                                                              │
│ 🏷️ 主题标签                                                 │
│ [架构设计] [性能优化] [测试] [文档] [Bug修复] [新功能]       │
│                                                              │
│ 💡 关键决策                                                  │
│ • Storage Layer 采用 25 组件分层架构                        │
│ • Memory 2.0 三层架构：感知/理解/编排                       │
│ • Web Terminal 采用 Jupyter-like 体验                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

#### 界面 2: 搜索结果

```
┌─────────────────────────────────────────────────────────────┐
│ 🔍 搜索: "Storage Layer"                        [清除] [×]  │
├─────────────────────────────────────────────────────────────┤
│ 找到 42 条相关对话                                          │
│                                                              │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 📄 Storage Layer 2.0 设计讨论                           │ │
│ │ 🕐 2026-01-08 14:30                                     │ │
│ │ 💬 "让我们设计一个分层的存储架构，包含缓存层..."         │ │
│ │ 🏷️ [架构设计] [Storage]                                │ │
│ │                                     [查看完整对话 →]     │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                              │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ 📄 CachedStorage 实现                                   │ │
│ │ 🕐 2026-01-08 16:45                                     │ │
│ │ 💬 "CachedStorage 使用 LRU 缓存策略..."                 │ │
│ │ 🏷️ [实现] [缓存]                                       │ │
│ │                                     [查看完整对话 →]     │ │
│ └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### 1.5 实施计划

| 阶段 | 任务 | 工作量 | 输出 |
|------|------|--------|------|
| **Phase 1** | 数据模型与存储 | 3 天 | `src/web/knowledge/mod.rs` |
| **Phase 2** | 导入器实现 | 2 天 | 支持 Claude Code 目录导入 |
| **Phase 3** | 主题提取算法 | 3 天 | 关键词 + 时间聚类 |
| **Phase 4** | 检索 API | 2 天 | WebSocket 消息协议 |
| **Phase 5** | 前端界面 | 4 天 | 知识浏览器 UI |
| **Phase 6** | 测试与优化 | 2 天 | 性能测试、边界情况 |

**总计**: 约 16 天

---

## 二、Agent 工作流可视化 (P1)

### 2.1 目标

可视化展示 Claude 的任务分解、Agent 调度和执行过程，增强开发透明度。

### 2.2 数据模型

```rust
// src/web/workflow/types.rs

/// 工作流执行记录
pub struct WorkflowExecution {
    pub id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub status: WorkflowStatus,
    pub root_task: TaskNode,
    pub agents: Vec<AgentExecution>,
}

/// 任务节点（树形结构）
pub struct TaskNode {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
    pub agent_id: Option<String>,
    pub children: Vec<TaskNode>,
    pub output: Option<String>,
    pub duration_ms: Option<u64>,
}

/// Agent 执行记录
pub struct AgentExecution {
    pub agent_id: String,
    pub agent_type: String,  // "Explore", "Plan", "general-purpose"
    pub parent_session: Option<String>,
    pub task_description: String,
    pub message_count: usize,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}
```

### 2.3 WebUI 界面

```
┌─────────────────────────────────────────────────────────────┐
│ 🔄 工作流执行视图                           [刷新] [导出]   │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ 📋 任务: "实现对话历史备份功能"                              │
│ ⏱️ 耗时: 5m 32s | 状态: ✅ 完成                             │
│                                                              │
│ 任务分解树:                                                  │
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ ● 主任务                                                 │ │
│ │   ├─● 创建目录结构 ✅ (0.5s)                            │ │
│ │   ├─● 编写 Python 脚本 ✅ (45s)                         │ │
│ │   │   └─🤖 Agent: general-purpose                       │ │
│ │   ├─● 创建说明文档 ✅ (30s)                             │ │
│ │   ├─● 执行备份 ✅ (2m)                                  │ │
│ │   └─● 验证结果 ✅ (15s)                                 │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                              │
│ Agent 执行详情:                                              │
│ ┌──────────────────┬────────────┬──────────┬──────────────┐ │
│ │ Agent ID         │ 类型       │ 消息数   │ 耗时         │ │
│ ├──────────────────┼────────────┼──────────┼──────────────┤ │
│ │ a04bc06          │ general    │ 47       │ 1m 23s       │ │
│ │ ae359de          │ Explore    │ 35       │ 45s          │ │
│ └──────────────────┴────────────┴──────────┴──────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.4 实施计划

| 阶段 | 任务 | 工作量 |
|------|------|--------|
| Phase 1 | 数据模型定义 | 2 天 |
| Phase 2 | 执行追踪钩子 | 3 天 |
| Phase 3 | WebSocket 协议扩展 | 2 天 |
| Phase 4 | 前端可视化组件 | 4 天 |
| Phase 5 | 测试与优化 | 2 天 |

**总计**: 约 13 天

---

## 三、一分为三渐进落地 (P2)

### 3.1 落地路径

```
当前状态 (已完成)
├─ 三态枚举
│   └─ Safe/NeedsConfirmation/Dangerous
│
Phase 1: 多维置信度 (待实现)
├─ IntentMatchState 向量
│   └─ {confidence, risk, user_level, ...}
│
Phase 2: 状态演化 (规划中)
├─ StateVector::evolve_towards()
│   └─ 渐进式状态变化
│
Phase 3: 规则引擎 (远期)
└─ CompositeRule + AdaptiveRule
    └─ 可组合、自学习的规则系统
```

### 3.2 Phase 1 实现：多维 Intent 匹配

```rust
// src/dsl/intent/vector_match.rs

/// Intent 匹配状态向量
pub struct IntentMatchState {
    pub confidence: f64,           // 匹配置信度 [0, 1]
    pub risk_level: f64,           // 命令风险 [0, 1]
    pub user_experience: f64,      // 用户经验 [0, 1]
    pub historical_success: f64,   // 历史成功率 [0, 1]
    pub context_relevance: f64,    // 上下文相关性 [0, 1]
}

impl IntentMatchState {
    /// 计算综合决策分数
    pub fn decision_score(&self) -> f64 {
        self.confidence * 0.4
            + (1.0 - self.risk_level) * 0.3
            + self.user_experience * 0.15
            + self.historical_success * 0.1
            + self.context_relevance * 0.05
    }

    /// 三态决策
    pub fn to_action(&self) -> IntentAction {
        let score = self.decision_score();
        match score {
            s if s > 0.7 => IntentAction::Execute,
            s if s > 0.4 => IntentAction::Confirm,
            _ => IntentAction::FallbackToLLM,
        }
    }
}
```

### 3.3 实施计划

| 阶段 | 任务 | 工作量 |
|------|------|--------|
| Phase 1 | IntentMatchState 实现 | 3 天 |
| Phase 2 | 集成到 Intent 匹配流程 | 2 天 |
| Phase 3 | WebUI 可视化展示 | 2 天 |
| Phase 4 | 测试与调优 | 2 天 |

**总计**: 约 9 天

---

## 四、对话模式分析 (P3)

### 4.1 分析维度

```rust
// src/web/analytics/conversation.rs

pub struct ConversationAnalytics {
    /// 任务完成效率
    pub avg_turns_per_task: f64,
    pub clarification_rate: f64,      // 需要澄清的比例
    pub rollback_rate: f64,           // 回退/重做比例

    /// 交互模式
    pub peak_hours: Vec<u8>,          // 活跃时段
    pub avg_session_duration: Duration,
    pub common_task_types: Vec<(String, usize)>,

    /// 质量指标
    pub first_attempt_success: f64,   // 首次尝试成功率
    pub tool_usage_distribution: HashMap<String, usize>,
}
```

### 4.2 WebUI 仪表板

```
┌─────────────────────────────────────────────────────────────┐
│ 📊 对话分析仪表板                                            │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ 效率指标                               交互分布              │
│ ┌───────────────────────┐            ┌─────────────────────┐│
│ │ 平均完成轮次: 4.2     │            │   [饼图: 任务类型]  ││
│ │ 首次成功率: 78%       │            │                     ││
│ │ 澄清需求率: 12%       │            │ ● 功能开发 45%      ││
│ │ 回退重做率: 8%        │            │ ● Bug修复 25%       ││
│ └───────────────────────┘            │ ● 文档 15%          ││
│                                       │ ● 重构 15%          ││
│ 工具使用统计                          └─────────────────────┘│
│ ┌─────────────────────────────────────────────────────────┐ │
│ │ Read    ████████████████████████ 1,234                  │ │
│ │ Edit    ██████████████████ 987                          │ │
│ │ Bash    ████████████████ 876                            │ │
│ │ Write   ██████████ 543                                  │ │
│ │ Grep    ████████ 432                                    │ │
│ └─────────────────────────────────────────────────────────┘ │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 实施计划

| 阶段 | 任务 | 工作量 |
|------|------|--------|
| Phase 1 | 分析算法实现 | 3 天 |
| Phase 2 | 数据聚合管道 | 2 天 |
| Phase 3 | 前端仪表板 | 3 天 |
| Phase 4 | 测试与优化 | 2 天 |

**总计**: 约 10 天

---

## 五、综合实施时间表

### 整体规划

```
2026-01-30                    2026-02                    2026-03
    |                             |                          |
    ├─ P0: 知识图谱 (16天) ──────┤
    |                             |
    |       ├─ P1: 工作流可视化 (13天) ──────┤
    |       |                                 |
    |       |         ├─ P2: 一分为三 (9天) ──┤
    |       |         |                       |
    |       |         |   ├─ P3: 模式分析 (10天) ──┤
    |       |         |   |                        |
    v       v         v   v                        v
    ================================================
                  v2.3.0 发布目标
```

### 里程碑

| 日期 | 版本 | 主要交付 |
|------|------|----------|
| 2026-02-15 | v2.3.0-alpha.1 | 知识图谱基础功能 |
| 2026-02-28 | v2.3.0-alpha.2 | 工作流可视化 |
| 2026-03-10 | v2.3.0-beta.1 | 一分为三落地 |
| 2026-03-20 | v2.3.0-beta.2 | 对话分析仪表板 |
| 2026-03-30 | v2.3.0 | 正式发布 |

### 资源分配

- **总工作量**: 约 48 天
- **并行度**: 部分任务可并行
- **预计历时**: 约 2 个月（考虑并行和缓冲）

---

## 六、CLI 稳定性保证

### 不动范围

以下模块在 v2.3.0 开发期间保持稳定，不做改动：

- `src/agent.rs` - 核心调度（仅添加钩子，不改逻辑）
- `src/llm/` - LLM 客户端
- `src/dsl/intent/` - Intent DSL（P2 例外，但保持兼容）
- `src/command/` - 命令系统
- `src/tool/` - 工具系统
- `src/task/` - 任务编排

### 变动范围

- `src/web/` - 主要改动区域
- `src/web/knowledge/` - 新增模块
- `src/web/workflow/` - 新增模块
- `src/web/analytics/` - 新增模块

### 兼容性承诺

- WebSocket 协议向后兼容
- 现有 API 保持不变
- 新功能为可选扩展

---

## 七、验收标准

### P0: 知识图谱

- [ ] 支持导入 Claude Code 对话历史
- [ ] 支持关键词搜索
- [ ] 支持时间线浏览
- [ ] 支持主题分类
- [ ] WebUI 界面完整可用

### P1: 工作流可视化

- [ ] 任务分解树可视化
- [ ] Agent 执行追踪
- [ ] 实时状态更新
- [ ] 历史执行记录

### P2: 一分为三

- [ ] IntentMatchState 向量实现
- [ ] 多维决策逻辑
- [ ] WebUI 状态展示
- [ ] 与现有系统兼容

### P3: 模式分析

- [ ] 效率指标计算
- [ ] 工具使用统计
- [ ] 分析仪表板
- [ ] 数据导出功能

---

**文档版本**: 1.0
**创建者**: Claude Opus 4.5
**审核者**: RealConsole Team
