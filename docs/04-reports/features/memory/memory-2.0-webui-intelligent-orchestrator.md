# Memory 2.0 WebUI 智能上下文编排器 - 深度设计

**创建时间**: 2025-11-24
**状态**: 深度设计 - Phase B
**版本**: v2.0-webui
**设计哲学**: 一分为三的高级应用 - 状态向量、演化路径、规律组合

---

## 目录

- [零、设计初心与哲学基础](#零设计初心与哲学基础)
- [一、WebUI vs CLI：本质差异分析](#一webui-vs-cli本质差异分析)
- [二、一分为三的深度应用](#二一分为三的深度应用)
- [三、三维向量空间架构](#三三维向量空间架构)
- [四、状态演化与智能编排](#四状态演化与智能编排)
- [五、核心数据结构](#五核心数据结构)
- [六、实施路线图](#六实施路线图)
- [七、哲学思考的体现](#七哲学思考的体现)
- [八、风险与机遇](#八风险与机遇)

---

## 零、设计初心与哲学基础

### 0.1 用户的关键要求

> **"不能再是简单的修复了，而是用于适应webui场景下的memory管理优化，到时要秉持先进的哲学思考，深度思考后加以实现"**

这不是一个增量改进，而是**架构创新**：
- ❌ 不是 CLI Memory 的简单移植
- ❌ 不是功能修复或补丁
- ✅ 是 WebUI 场景的**重新思考**
- ✅ 是哲学高度的**深度设计**

### 0.2 核心哲学洞察

基于 `docs/00-core/philosophy.md` 的深化理解：

**状态不是离散的点，而是向量空间中的位置**：
```
CLI Memory: 二元状态（记录/不记录）
         ↓
Memory 1.0: 三态（User/AI/Shell）
         ↓
Memory 2.0 CLI: 三层架构（感知/理解/编排）
         ↓
Memory 2.0 WebUI: 多维向量空间（文本/可视化/会话/交互/...）
```

**变化不是跳转，而是演化路径**：
```
静态记录 → 智能选择 → 主动编排 → 预测推荐 → 自适应学习
    ↑                                              ↓
    └──────────────── 持续演化 ──────────────────┘
```

**规律可以组合，形成复杂行为**（易经64卦思想）：
```
8种基础模式 × 8种情境特征 = 64种智能决策
  ↓
每种决策有6个演化阶段 = 384个细节控制点
  ↓
无穷变化的自适应系统
```

### 0.3 Memory 2.0 WebUI 的使命

**不是**：
- ❌ CLI Memory 的浏览器版本
- ❌ 简单的会话记录工具
- ❌ 数据的被动存储器

**而是**：
- ✅ **富媒体智能编排器** - 文本 + 可视化 + 数据的最优组合
- ✅ **跨会话智能推荐器** - 从历史中预测未来需求
- ✅ **自适应学习系统** - 理解用户习惯，持续优化

---

## 一、WebUI vs CLI：本质差异分析

### 1.1 五维对比矩阵

| 维度 | CLI 场景 | WebUI 场景 | 设计影响 |
|------|---------|-----------|----------|
| **交互模式** | 线性、一问一答 | 非线性、多任务并行 | 需要**会话树结构**而非单链表 |
| **状态持久化** | 短期（单次使用） | 长期（跨天/周/月） | 需要**时间衰减算法** + **重要性评分** |
| **输出形式** | 纯文本 | 富媒体（图表/图像/表格） | 需要**可视化记忆**专用处理 |
| **用户群体** | 专家用户 | 广泛用户（含非技术） | 需要**智能推荐**降低门槛 |
| **数据源** | 4维（History/ExecutionLogger/LlmLogger/Context） | **7维**（+ SessionManager/ChartHistory/ImageHistory/UploadedFiles） | 需要**更广泛的感知层** |

### 1.2 WebUI 特有的记忆需求

#### 需求 1：可视化记忆（Visualization Memory）

**问题**：
- 用户生成了20个图表，如何快速找到"上次的销售趋势图"？
- 如何知道哪种图表类型对当前数据最有效？
- 图表参数（颜色、坐标轴范围）如何智能复用？

**传统方案（不足）**：
```rust
// ❌ 简单列表：无智能性
chart_history: Vec<ChartHistoryEntry>
```

**Memory 2.0 方案（智能）**：
```rust
// ✅ 向量化记忆：语义搜索 + 模式识别
struct VisualizationMemory {
    // 图表向量索引（类型、数据特征、参数）
    chart_embeddings: VectorIndex<ChartVector>,

    // 成功模式库（哪些图表用户满意？）
    success_patterns: PatternLibrary,

    // 参数推荐引擎（智能预填充）
    param_recommender: ParameterRecommender,
}
```

#### 需求 2：跨会话智能（Cross-Session Intelligence）

**场景**：
用户上周分析了"Q1销售数据"，生成了5个图表。
今天打开新会话，上传"Q2销售数据"。

**传统方案（不足）**：
```rust
// ❌ 孤立会话：每次从零开始
new_session() // 不知道用户上周做了什么
```

**Memory 2.0 方案（智能）**：
```rust
// ✅ 跨会话关联：智能推荐
fn on_new_data_uploaded(file: &UploadedFile) -> Recommendations {
    // 1. 分析数据特征
    let data_profile = profile_data(file);

    // 2. 从历史会话中查找相似分析
    let similar_sessions = memory.find_similar_sessions(&data_profile);

    // 3. 提取成功的分析流程
    let success_workflows = extract_workflows(similar_sessions);

    // 4. 生成智能推荐
    Recommendations {
        suggested_charts: vec![
            "上次您用折线图展示趋势，是否继续？",
            "Q1数据中，柱状图效果最好，推荐尝试",
        ],
        pre_filled_commands: vec![
            "/chart line sales_trend --x-axis month",
        ],
        related_sessions: similar_sessions,
    }
}
```

#### 需求 3：交互式智能（Interactive Intelligence）

**场景**：
用户在图表上放大了某个区域，点击了某个数据点。

**传统方案（不足）**：
```rust
// ❌ 无记忆：交互丢失
on_chart_click(point) {
    // 仅显示数据，不记录意图
}
```

**Memory 2.0 方案（智能）**：
```rust
// ✅ 交互记忆：理解意图
struct InteractionMemory {
    // 记录用户关注的焦点
    focus_areas: Vec<FocusArea>,

    // 推断用户意图
    inferred_intents: Vec<Intent>,

    // 生成后续建议
    fn on_user_focus(&mut self, area: FocusArea) {
        self.focus_areas.push(area);

        // 推断：用户对这个区域感兴趣
        let intent = infer_intent(&area);

        // 建议：深入分析这个区域
        suggest_next_action(intent); // "是否需要查看这个月的详细数据？"
    }
}
```

### 1.3 架构演化的必然性

```
CLI Memory 1.0（被动记录器）
  ↓
CLI Memory 2.0（智能编排器）
  ↓
WebUI Memory 2.0（富媒体智能编排器 + 跨会话推荐器 + 自适应学习系统）
  ↓
未来：预测式智能助手
```

**关键差异**：
- CLI Memory：为 LLM 提供文本上下文
- WebUI Memory：为用户 + LLM 提供富媒体智能推荐

---

## 二、一分为三的深度应用

### 2.1 从"三态"到"三维向量空间"

基于哲学文档的高级理解，Memory 2.0 WebUI 不是"三个状态"，而是**三个维度的向量空间**：

```
维度一：内容维度（Content Dimension）
  ├─ 文本内容（TextVector）
  ├─ 可视化内容（VisualVector）
  └─ 数据内容（DataVector）

维度二：时间维度（Temporal Dimension）
  ├─ 短期记忆（WorkingMemory: 9轮对话）
  ├─ 中期记忆（SessionMemory: 单次会话）
  └─ 长期记忆（CrossSessionMemory: 跨会话）

维度三：智能维度（Intelligence Dimension）
  ├─ 被动记录（Perception: 采集数据）
  ├─ 主动理解（Understanding: 分析模式）
  └─ 预测编排（Orchestration: 推荐行动）
```

**数学表示**：
```rust
struct MemoryStateVector {
    // 内容维度（3维）
    text_weight: f64,        // 文本内容的权重
    visual_weight: f64,      // 可视化内容的权重
    data_weight: f64,        // 原始数据的权重

    // 时间维度（3维）
    working_relevance: f64,  // 与当前对话的相关性
    session_relevance: f64,  // 与当前会话的相关性
    long_term_value: f64,    // 长期价值（跨会话）

    // 智能维度（3维）
    perception_score: f64,   // 数据采集质量
    understanding_score: f64,// 模式识别准确性
    orchestration_score: f64,// 推荐效果评分
}

// 总计：3×3 = 9 维向量空间
```

### 2.2 易经64卦的映射：智能决策矩阵

**8种内容特征** × **8种时间情境** = **64种智能决策**

#### 8种内容特征（内卦）

1. **纯文本**（乾☰）：对话、命令、文档
2. **纯可视化**（坤☷）：图表、图像、视频
3. **纯数据**（震☳）：CSV、JSON、数据库
4. **文本+可视化**（巽☴）：带图表的报告
5. **文本+数据**（坎☵）：数据分析脚本
6. **可视化+数据**（离☲）：交互式图表
7. **三者混合**（艮☶）：完整的分析报告
8. **交互操作**（兑☱）：用户点击、缩放、选择

#### 8种时间情境（外卦）

1. **当前对话**（乾☰）：9轮以内
2. **当前会话**（坤☷）：本次会话
3. **最近会话**（震☳）：3天内
4. **中期会话**（巽☴）：1周-1月
5. **长期会话**（坎☵）：1月+
6. **重复模式**（离☲）：跨会话的重复任务
7. **演化趋势**（艮☶）：用户习惯的变化
8. **稀有事件**（兑☱）：偶尔出现的特殊需求

#### 64种智能决策示例

| 内卦 | 外卦 | 卦象 | 智能决策 |
|------|------|------|----------|
| 纯文本 | 当前对话 | 乾为天 | **高优先级**：直接加入 LLM 上下文 |
| 纯可视化 | 当前会话 | 坤为地 | **中优先级**：生成可视化摘要 |
| 文本+可视化 | 最近会话 | 雷天大壮 | **推荐复用**：建议使用上次的图表模板 |
| 数据+可视化 | 重复模式 | 水火既济 | **自动预填充**：检测到重复任务，自动建议 |
| 三者混合 | 演化趋势 | 山地剥 | **学习优化**：用户偏好变化，调整推荐策略 |
| ... | ... | ... | ... |

**代码实现思路**：
```rust
fn intelligent_decision(
    content_type: ContentType,  // 内卦（8种）
    time_context: TimeContext,  // 外卦（8种）
) -> MemoryAction {
    // 64种组合 → 智能决策
    match (content_type, time_context) {
        (ContentType::TextOnly, TimeContext::CurrentDialog) => {
            MemoryAction::HighPriority {
                action: "直接加入 LLM 上下文",
                token_budget: 2000,
            }
        }
        (ContentType::VisualOnly, TimeContext::CurrentSession) => {
            MemoryAction::MediumPriority {
                action: "生成图表缩略图 + 文本摘要",
                token_budget: 500,
            }
        }
        (ContentType::TextAndVisual, TimeContext::RecentSession) => {
            MemoryAction::RecommendReuse {
                suggestion: "上次的分析报告可以复用，是否继续？",
                template_id: find_similar_template(),
            }
        }
        (ContentType::DataAndVisual, TimeContext::RepeatedPattern) => {
            MemoryAction::AutoSuggest {
                pre_fill: "检测到周报生成任务，自动预填充参数",
                workflow: extract_repeated_workflow(),
            }
        }
        // ... 其他60种组合
        _ => MemoryAction::FallbackToLLM,
    }
}
```

### 2.3 状态演化：从规则到学习

**初级阶段**（规则引擎）：
```rust
// 硬编码64种规则
match (content, time) {
    (A, B) => action_1,
    (C, D) => action_2,
    ...
}
```

**中级阶段**（自适应规则）：
```rust
// 规则 + 权重调整
struct AdaptiveRule {
    base_rule: Rule,
    success_rate: f64,  // 根据用户反馈调整
    weight: f64,        // 动态调整权重
}
```

**高级阶段**（机器学习）：
```rust
// 从用户行为中学习
struct LearningEngine {
    // 用户点击了推荐 → 正样本
    // 用户忽略了推荐 → 负样本
    fn update_from_feedback(&mut self, action: &MemoryAction, accepted: bool) {
        // 强化学习：调整决策模型
    }
}
```

---

## 三、三维向量空间架构

### 3.1 整体架构：三层递进

```
┌─────────────────────────────────────────────────────────────┐
│          Memory 2.0 WebUI: 三维向量空间架构                  │
│        （内容 × 时间 × 智能 = 9维决策空间）                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  【层一】感知层（Perception Layer）                           │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • CLI 四维：History/ExecutionLogger/LlmLogger/Context        │
│  • WebUI 新增：SessionManager/ChartHistory/ImageHistory       │
│              /UploadedFiles                                   │
│  • 统一数据模型：ContextChunk → 9维向量                       │
│  • 输出：原始数据 + 向量化表示                                 │
│                                                               │
│  【层二】理解层（Understanding Layer）                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • 文本理解：关键词 + 语义向量（CLI 已有）                     │
│  • 可视化理解：图表模式识别 + 参数提取（WebUI 新增）           │
│  • 数据理解：数据profiling + 特征工程（WebUI 新增）           │
│  • 会话理解：主题聚类 + 任务识别（WebUI 新增）                 │
│  • 交互理解：意图推断 + 焦点检测（WebUI 新增）                 │
│  • 输出：相关性评分 + 重要性评分 + 推荐置信度                  │
│                                                               │
│  【层三】编排层（Orchestration Layer）                         │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━           │
│  • 文本编排：Token 预算管理（CLI 已有）                       │
│  • 富媒体编排：图表+文本最优组合（WebUI 新增）                 │
│  • 会话编排：跨会话智能推荐（WebUI 新增）                      │
│  • 交互编排：实时建议生成（WebUI 新增）                        │
│  • 学习编排：从反馈中优化（WebUI 新增）                        │
│  • 输出：OptimizedContext（文本+可视化+推荐+预填充）          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 层一：感知层的 WebUI 扩展

**CLI 感知层（4维）**：
```rust
struct CLIPerceptionLayer {
    unified_tracer: Arc<UnifiedTracer>,
    context_tracker: Arc<RwLock<ContextTracker>>,
}
```

**WebUI 感知层（7维）**：
```rust
struct WebUIPerceptionLayer {
    // ===== CLI 四维（复用） =====
    unified_tracer: Arc<UnifiedTracer>,
    context_tracker: Arc<RwLock<ContextTracker>>,

    // ===== WebUI 新增三维 =====

    // 1. 会话维度：跨会话记忆
    session_manager: Arc<SessionManager>,

    // 2. 可视化维度：图表/图像记忆
    chart_history: Arc<RwLock<Vec<ChartHistoryEntry>>>,
    image_history: Arc<RwLock<Vec<ImageHistoryEntry>>>,

    // 3. 数据维度：上传的数据文件
    uploaded_files: Arc<UploadedFileManager>,
}

impl WebUIPerceptionLayer {
    /// 采集富媒体数据
    async fn collect_multimodal_data(
        &self,
        time_range: Option<TimeRange>,
    ) -> Result<Vec<MultimodalChunk>> {
        let mut chunks = Vec::new();

        // CLI 数据（文本）
        let text_chunks = self.unified_tracer.query(...).await?;
        chunks.extend(text_chunks.into_iter().map(MultimodalChunk::from));

        // WebUI 数据（可视化）
        let charts = self.chart_history.read().await;
        chunks.extend(charts.iter().map(|c| MultimodalChunk::from_chart(c)));

        let images = self.image_history.read().await;
        chunks.extend(images.iter().map(|i| MultimodalChunk::from_image(i)));

        // WebUI 数据（会话）
        let sessions = self.session_manager.list_sessions()?;
        chunks.extend(sessions.iter().map(|s| MultimodalChunk::from_session(s)));

        // WebUI 数据（上传文件）
        let files = self.uploaded_files.list()?;
        chunks.extend(files.iter().map(|f| MultimodalChunk::from_file(f)));

        Ok(chunks)
    }

    /// 向量化：将多模态数据转为9维向量
    fn vectorize(&self, chunk: &MultimodalChunk) -> MemoryStateVector {
        MemoryStateVector {
            // 内容维度
            text_weight: calculate_text_weight(chunk),
            visual_weight: calculate_visual_weight(chunk),
            data_weight: calculate_data_weight(chunk),

            // 时间维度
            working_relevance: calculate_working_relevance(chunk),
            session_relevance: calculate_session_relevance(chunk),
            long_term_value: calculate_long_term_value(chunk),

            // 智能维度
            perception_score: 1.0,  // 感知层默认满分
            understanding_score: 0.0,  // 待理解层填充
            orchestration_score: 0.0,  // 待编排层填充
        }
    }
}
```

### 3.3 层二：理解层的 WebUI 增强

**CLI 理解层（文本语义）**：
```rust
struct CLIUnderstandingLayer {
    keyword_matcher: KeywordMatcher,
    embedding_engine: Option<Arc<EmbeddingEngine>>,
}
```

**WebUI 理解层（多模态语义）**：
```rust
struct WebUIUnderstandingLayer {
    // ===== CLI 能力（复用） =====
    text_analyzer: TextAnalyzer,  // 关键词 + 语义向量

    // ===== WebUI 新增能力 =====

    // 1. 可视化理解
    chart_pattern_analyzer: ChartPatternAnalyzer,

    // 2. 数据理解
    data_profiler: DataProfiler,

    // 3. 会话理解
    session_clusterer: SessionClusterer,

    // 4. 交互理解
    interaction_intent_detector: InteractionIntentDetector,

    // 5. 跨模态关联
    cross_modal_linker: CrossModalLinker,
}

impl ChartPatternAnalyzer {
    /// 识别图表模式
    fn analyze_chart(&self, chart: &ChartData) -> ChartPattern {
        ChartPattern {
            chart_type: chart.chart_type.clone(),
            data_dimensions: chart.datasets.len(),
            x_axis_type: infer_axis_type(&chart.x_axis),
            y_axis_type: infer_axis_type(&chart.y_axis),

            // 成功指标（用户是否满意？）
            user_satisfaction: estimate_satisfaction(chart),

            // 复用价值
            reusability_score: calculate_reusability(chart),
        }
    }
}

impl SessionClusterer {
    /// 会话主题聚类
    fn cluster_sessions(&self, sessions: &[SerializableSession]) -> Vec<SessionCluster> {
        // 1. 提取会话特征向量
        let features: Vec<SessionFeatureVector> = sessions
            .iter()
            .map(|s| self.extract_features(s))
            .collect();

        // 2. K-means 聚类
        let clusters = kmeans_clustering(&features, k: 5);

        // 3. 为每个聚类打标签
        clusters.into_iter()
            .map(|cluster| SessionCluster {
                topic: infer_topic(&cluster),
                sessions: cluster.members,
                common_tools: extract_common_tools(&cluster),
                typical_workflow: extract_workflow(&cluster),
            })
            .collect()
    }
}

impl InteractionIntentDetector {
    /// 从用户交互推断意图
    fn detect_intent(&self, interaction: &UserInteraction) -> Intent {
        match interaction {
            UserInteraction::ChartZoom { area } => {
                Intent::FocusOnDetail {
                    suggestion: format!("是否需要查看 {} 的详细数据？", area),
                }
            }
            UserInteraction::ChartClick { data_point } => {
                Intent::InvestigateAnomaly {
                    suggestion: format!("这个数据点异常，是否需要解释？"),
                }
            }
            UserInteraction::FileUpload { filename } => {
                // 查找相似文件的历史分析
                let similar = self.find_similar_file_analysis(filename);
                Intent::AnalyzeData {
                    suggestion: format!("上次分析类似文件时，您使用了：{}", similar.workflow),
                    pre_fill: similar.commands,
                }
            }
        }
    }
}
```

### 3.4 层三：编排层的 WebUI 特化

**CLI 编排层（文本优化）**：
```rust
struct CLIOrchestrationLayer {
    token_counter: TokenCounter,
    compressor: ContextCompressor,
}
```

**WebUI 编排层（富媒体优化）**：
```rust
struct WebUIOrchestrationLayer {
    // ===== CLI 能力（复用） =====
    token_counter: TokenCounter,
    text_compressor: TextCompressor,

    // ===== WebUI 新增能力 =====

    // 1. 富媒体编排
    multimodal_composer: MultimodalComposer,

    // 2. 跨会话推荐
    cross_session_recommender: CrossSessionRecommender,

    // 3. 交互式建议
    interactive_suggester: InteractiveSuggester,

    // 4. 自适应学习
    adaptive_optimizer: AdaptiveOptimizer,
}

impl MultimodalComposer {
    /// 构建富媒体上下文
    ///
    /// 输入：文本 + 图表 + 数据
    /// 输出：最优组合，控制在 token 预算内
    fn compose_context(
        &self,
        text_chunks: Vec<TextChunk>,
        chart_chunks: Vec<ChartChunk>,
        data_chunks: Vec<DataChunk>,
        token_budget: usize,
    ) -> OptimizedMultimodalContext {
        // 1. 分配 token 预算
        let text_budget = (token_budget as f64 * 0.6) as usize;   // 60% 文本
        let visual_budget = (token_budget as f64 * 0.3) as usize; // 30% 可视化
        let data_budget = (token_budget as f64 * 0.1) as usize;   // 10% 数据

        // 2. 选择最相关的文本
        let selected_text = self.select_top_k(text_chunks, text_budget);

        // 3. 选择最有价值的图表
        let selected_charts = self.select_charts_by_value(chart_chunks, visual_budget);

        // 4. 选择关键数据摘要
        let selected_data = self.summarize_data(data_chunks, data_budget);

        // 5. 组合输出
        OptimizedMultimodalContext {
            text: selected_text,
            charts: selected_charts.into_iter()
                .map(|c| ChartReference {
                    id: c.id,
                    thumbnail_text: generate_chart_description(&c),
                    reuse_params: extract_reusable_params(&c),
                })
                .collect(),
            data_summary: selected_data,
            total_tokens: text_budget + visual_budget + data_budget,
        }
    }
}

impl CrossSessionRecommender {
    /// 跨会话智能推荐
    fn recommend_from_history(
        &self,
        current_task: &str,
        current_data: Option<&DataProfile>,
    ) -> Vec<Recommendation> {
        // 1. 分析当前任务特征
        let task_vector = self.vectorize_task(current_task, current_data);

        // 2. 从历史会话中查找相似任务
        let similar_sessions = self.find_similar_sessions(&task_vector, top_k: 5);

        // 3. 提取成功模式
        let success_patterns = similar_sessions.iter()
            .filter(|s| s.success_rate > 0.7)
            .map(|s| extract_pattern(s))
            .collect::<Vec<_>>();

        // 4. 生成推荐
        success_patterns.into_iter()
            .map(|pattern| Recommendation {
                type_: RecommendationType::WorkflowReuse,
                description: format!(
                    "上次处理类似任务时，您的流程是：{}",
                    pattern.workflow_summary
                ),
                confidence: pattern.success_rate,
                actions: pattern.commands.clone(),
                pre_fill_params: pattern.params.clone(),
            })
            .collect()
    }
}

impl AdaptiveOptimizer {
    /// 从用户反馈中学习
    fn learn_from_feedback(
        &mut self,
        recommendation: &Recommendation,
        user_action: UserAction,
    ) {
        match user_action {
            UserAction::Accepted => {
                // 正样本：增加权重
                self.update_pattern_weight(&recommendation.pattern_id, delta: 0.1);
            }
            UserAction::Rejected => {
                // 负样本：降低权重
                self.update_pattern_weight(&recommendation.pattern_id, delta: -0.1);
            }
            UserAction::Modified { changes } => {
                // 部分接受：学习修改模式
                self.learn_modification_pattern(&recommendation.pattern_id, changes);
            }
            UserAction::Ignored => {
                // 轻微降权
                self.update_pattern_weight(&recommendation.pattern_id, delta: -0.05);
            }
        }
    }
}
```

---

## 四、状态演化与智能编排

### 4.1 状态演化路径

**不是瞬间跳转，而是渐进演化**：

```rust
struct MemoryEvolutionPath {
    // 当前状态
    current: MemoryStateVector,

    // 目标状态
    target: MemoryStateVector,

    // 演化速度
    evolution_rate: f64,
}

impl MemoryEvolutionPath {
    /// 向目标状态演化一步
    fn evolve_step(&mut self) {
        // 不是跳转，而是线性插值
        self.current = self.current.lerp(&self.target, self.evolution_rate);
    }

    /// 平滑演化（考虑惯性）
    fn smooth_evolve(&mut self, target: MemoryStateVector) {
        // 目标突变时，不立即跳转，而是逐步调整
        self.target = self.target.lerp(&target, 0.3);  // 目标也平滑变化
        self.evolve_step();
    }
}

// 示例：用户从纯文本任务转向数据分析任务
fn example_evolution() {
    let mut state = MemoryStateVector {
        text_weight: 0.9,    // 初始：主要是文本
        visual_weight: 0.1,
        data_weight: 0.0,
        ...
    };

    // 用户上传了 CSV 文件 → 目标变化
    let new_target = MemoryStateVector {
        text_weight: 0.3,
        visual_weight: 0.4,   // 预期会生成图表
        data_weight: 0.3,     // 数据分析权重增加
        ...
    };

    // 不是立即跳转，而是逐步演化（5步）
    for _ in 0..5 {
        state.smooth_evolve(new_target.clone());
        // 每步调整推荐策略
    }
}
```

### 4.2 智能决策的动态调整

**64种基础决策 × 自适应权重 = 无穷变化**：

```rust
struct AdaptiveDecisionEngine {
    // 64种基础规则
    base_rules: HashMap<(ContentType, TimeContext), MemoryAction>,

    // 每条规则的动态权重
    rule_weights: HashMap<String, f64>,

    // 用户偏好模型
    user_preference: UserPreferenceModel,
}

impl AdaptiveDecisionEngine {
    fn decide(&self, state: &MemoryStateVector) -> MemoryAction {
        // 1. 确定内容类型和时间情境
        let content_type = classify_content(state);
        let time_context = classify_time(state);

        // 2. 获取基础决策
        let base_action = self.base_rules.get(&(content_type, time_context))
            .cloned()
            .unwrap_or(MemoryAction::FallbackToLLM);

        // 3. 根据用户偏好调整
        let adjusted_action = self.user_preference.adjust(base_action);

        // 4. 根据历史成功率调整
        let rule_id = format!("{:?}-{:?}", content_type, time_context);
        let weight = self.rule_weights.get(&rule_id).unwrap_or(&1.0);

        // 5. 返回加权后的决策
        adjusted_action.scale_confidence(*weight)
    }

    /// 从反馈中学习
    fn learn(&mut self, state: &MemoryStateVector, action: &MemoryAction, success: bool) {
        let content_type = classify_content(state);
        let time_context = classify_time(state);
        let rule_id = format!("{:?}-{:?}", content_type, time_context);

        // 更新权重
        let current_weight = self.rule_weights.entry(rule_id).or_insert(1.0);
        if success {
            *current_weight *= 1.1;  // 成功 → 增加权重
        } else {
            *current_weight *= 0.9;  // 失败 → 降低权重
        }

        // 限制范围
        *current_weight = current_weight.clamp(0.1, 2.0);
    }
}
```

---

## 五、核心数据结构

### 5.1 MultimodalChunk（统一多模态数据）

```rust
/// 多模态上下文片段
#[derive(Debug, Clone)]
pub struct MultimodalChunk {
    id: Uuid,
    timestamp: DateTime<Utc>,

    // 数据来源维度
    dimension: DataDimension,  // CLI 4维 + WebUI 3维

    // 内容（多模态）
    content: MultimodalContent,

    // 状态向量（9维）
    state_vector: MemoryStateVector,

    // 元数据
    metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub enum MultimodalContent {
    Text(String),
    Chart(ChartData),
    Image(ImageData),
    Data(DataSummary),
    Session(SessionSummary),
    Composite {
        text: Option<String>,
        chart: Option<ChartData>,
        data: Option<DataSummary>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum DataDimension {
    // CLI 四维
    History,
    ExecutionLogger,
    LlmLogger,
    Context,

    // WebUI 三维
    SessionManager,
    ChartHistory,
    ImageHistory,
    UploadedFiles,
}
```

### 5.2 OptimizedMultimodalContext（优化后的富媒体上下文）

```rust
/// 优化后的富媒体上下文
pub struct OptimizedMultimodalContext {
    // 文本部分
    pub text_chunks: Vec<TextChunk>,

    // 可视化部分（引用，不嵌入完整数据）
    pub chart_references: Vec<ChartReference>,
    pub image_references: Vec<ImageReference>,

    // 数据摘要
    pub data_summaries: Vec<DataSummary>,

    // 推荐
    pub recommendations: Vec<Recommendation>,

    // 预填充参数
    pub pre_filled_params: HashMap<String, Value>,

    // 总 token 数
    pub total_tokens: usize,

    // 元信息
    pub metadata: ContextMetadata,
}

#[derive(Debug, Serialize)]
pub struct ChartReference {
    pub id: Uuid,
    pub thumbnail_text: String,  // 文本描述（用于 LLM）
    pub reuse_params: HashMap<String, Value>,  // 可复用的参数
    pub success_score: f64,  // 历史成功评分
}

#[derive(Debug, Serialize)]
pub struct Recommendation {
    pub type_: RecommendationType,
    pub description: String,
    pub confidence: f64,
    pub actions: Vec<String>,  // 建议的命令
    pub pre_fill_params: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
pub enum RecommendationType {
    WorkflowReuse,      // 复用历史流程
    ChartTemplateReuse, // 复用图表模板
    DataAnalysisPattern,// 数据分析模式
    InteractiveSuggestion, // 基于交互的建议
}
```

### 5.3 SmartWebUIOrchestrator（主接口）

```rust
/// Memory 2.0 WebUI: 智能富媒体编排器
pub struct SmartWebUIOrchestrator {
    // 三层架构
    perception: WebUIPerceptionLayer,
    understanding: WebUIUnderstandingLayer,
    orchestration: WebUIOrchestrationLayer,

    // 决策引擎
    decision_engine: AdaptiveDecisionEngine,

    // 学习引擎
    learning_engine: LearningEngine,
}

impl SmartWebUIOrchestrator {
    /// 核心方法：为当前任务提取相关上下文（WebUI 增强版）
    pub async fn extract_relevant_context(
        &self,
        task: &str,
        current_session: Option<&SessionId>,
        token_budget: usize,
    ) -> Result<OptimizedMultimodalContext> {
        // 1. 感知：采集多模态数据
        let chunks = self.perception.collect_multimodal_data(None).await?;

        // 2. 向量化
        let vectors: Vec<_> = chunks.iter()
            .map(|c| self.perception.vectorize(c))
            .collect();

        // 3. 理解：分析相关性
        let scored_chunks = self.understanding.score_relevance(task, chunks, vectors).await?;

        // 4. 编排：优化组合
        let context = self.orchestration.compose_context(scored_chunks, token_budget).await?;

        Ok(context)
    }

    /// WebUI 特有：跨会话智能推荐
    pub async fn recommend_from_sessions(
        &self,
        current_data: Option<&DataProfile>,
    ) -> Result<Vec<Recommendation>> {
        // 查找相似会话
        let similar_sessions = self.understanding.find_similar_sessions(current_data).await?;

        // 生成推荐
        let recommendations = self.orchestration.generate_recommendations(similar_sessions)?;

        Ok(recommendations)
    }

    /// WebUI 特有：交互式建议
    pub async fn suggest_next_action(
        &self,
        interaction: &UserInteraction,
    ) -> Result<Vec<String>> {
        // 推断意图
        let intent = self.understanding.detect_intent(interaction).await?;

        // 生成建议
        let suggestions = self.orchestration.generate_suggestions(intent)?;

        Ok(suggestions)
    }

    /// 学习反馈
    pub async fn learn_from_feedback(
        &mut self,
        recommendation: &Recommendation,
        user_action: UserAction,
    ) -> Result<()> {
        // 更新学习引擎
        self.learning_engine.update(recommendation, user_action).await?;

        // 调整决策引擎
        self.decision_engine.adjust_weights(&self.learning_engine).await?;

        Ok(())
    }
}
```

---

## 六、实施路线图

### 6.1 四阶段渐进式实施（3-4 个月）

```
Phase 1: WebUI 基础设施（3 周）
  ↓
Phase 2: 可视化记忆（3 周）
  ↓
Phase 3: 跨会话智能（4 周）
  ↓
Phase 4: 自适应学习（2 周）
```

#### Phase 1：WebUI 基础设施（3 周）

**Week 1-2：扩展感知层**

任务清单：
- [ ] 定义 `MultimodalChunk` 数据结构
- [ ] 实现 `WebUIPerceptionLayer`
  - [ ] 集成 SessionManager
  - [ ] 集成 ChartHistory/ImageHistory
  - [ ] 集成 UploadedFileManager
- [ ] 实现向量化：`vectorize()` 方法
- [ ] 单元测试：从 WebUI 7维采集数据

验收标准：
- ✅ 能从 WebUI 7维采集 1000 条数据
- ✅ MultimodalChunk 转换无损失
- ✅ 向量化准确率 > 90%

**Week 3：理解层基础**

任务清单：
- [ ] 实现 `ChartPatternAnalyzer`（图表模式识别）
- [ ] 实现 `DataProfiler`（数据特征提取）
- [ ] 实现基础的相关性评分
- [ ] 单元测试：评分准确性

验收标准：
- ✅ 图表模式识别准确率 > 80%
- ✅ 数据特征提取完整
- ✅ 相关性评分合理

#### Phase 2：可视化记忆（3 周）

**Week 4-5：图表记忆与复用**

任务清单：
- [ ] 实现 `ChartMemoryIndex`（图表向量索引）
- [ ] 实现图表参数提取和推荐
- [ ] 实现图表模板匹配
- [ ] 集成测试：生成图表 → 记忆 → 复用

验收标准：
- ✅ 图表检索准确率 > 85%
- ✅ 参数推荐命中率 > 70%
- ✅ 用户满意度 > 75%

**Week 6：数据分析模式识别**

任务清单：
- [ ] 实现 `DataAnalysisPatternDetector`
- [ ] 识别常见分析流程（趋势/对比/分布...）
- [ ] 自动建议下一步分析
- [ ] 集成测试：上传 CSV → 自动建议

验收标准：
- ✅ 模式识别准确率 > 75%
- ✅ 建议相关性 > 70%

#### Phase 3：跨会话智能（4 周）

**Week 7-8：会话聚类与主题识别**

任务清单：
- [ ] 实现 `SessionClusterer`（会话聚类）
- [ ] 实现会话主题提取
- [ ] 实现相似会话检索
- [ ] 集成测试：加载会话 → 查找相似 → 推荐

验收标准：
- ✅ 聚类质量（Silhouette score > 0.6）
- ✅ 主题识别准确率 > 80%
- ✅ 相似会话检索准确率 > 85%

**Week 9-10：跨会话推荐**

任务清单：
- [ ] 实现 `CrossSessionRecommender`
- [ ] 工作流程提取
- [ ] 智能预填充
- [ ] 集成测试：新任务 → 自动推荐历史流程

验收标准：
- ✅ 推荐命中率 > 70%
- ✅ 预填充准确率 > 80%
- ✅ 用户采纳率 > 60%

#### Phase 4：自适应学习（2 周）

**Week 11：反馈机制**

任务清单：
- [ ] 实现用户反馈收集（接受/拒绝/修改）
- [ ] 实现 `AdaptiveOptimizer`
- [ ] 权重自动调整
- [ ] 集成测试：推荐 → 反馈 → 学习 → 优化

验收标准：
- ✅ 反馈收集完整
- ✅ 权重调整合理
- ✅ 推荐质量持续提升

**Week 12：性能优化与发布准备**

任务清单：
- [ ] 并发优化（采集、分析、编排并行）
- [ ] 缓存优化（LRU 缓存热点数据）
- [ ] 压力测试（5000 条数据 < 2s）
- [ ] 文档完善

验收标准：
- ✅ 所有测试通过
- ✅ 性能达标
- ✅ 文档齐全
- ✅ 准备好发布

### 6.2 成功标准（Definition of Done）

**功能完整性**：
- ✅ 三层架构全部实现（感知/理解/编排）
- ✅ WebUI 7维数据采集完整
- ✅ 可视化记忆、跨会话推荐、自适应学习全部可用

**性能指标**：
- ✅ 1000 条数据处理 < 1s
- ✅ 5000 条数据处理 < 2s
- ✅ Token 预算控制准确率 > 95%

**质量指标**：
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试通过率 100%
- ✅ 图表检索准确率 > 85%
- ✅ 跨会话推荐命中率 > 70%
- ✅ 用户采纳率 > 60%

**用户体验**：
- ✅ 文档完整（用户指南 + 开发者文档）
- ✅ 错误提示清晰
- ✅ 响应格式友好
- ✅ 推荐解释清楚

---

## 七、哲学思考的体现

### 7.1 一分为三的三个层次

**第一层：基础概念**（从二元到三元）
- CLI Memory 1.0: 记录/不记录（二元）
- CLI Memory 2.0: 感知/理解/编排（三元）
- WebUI Memory 2.0: 保持三元结构，扩展每层能力

**第二层：深化智慧**（从固定状态到向量空间）
- 不是3个固定状态，而是3×3 = 9维向量空间
- 内容维度（文本/可视化/数据）
- 时间维度（短期/中期/长期）
- 智能维度（感知/理解/编排）

**第三层：实践升华**（从规则到演化）
- 64种基础决策（8×8 易经思想）
- 状态演化路径（不是跳转，是渐进）
- 自适应学习（从反馈中优化）

### 7.2 易经智慧的映射

**道（规律）**：
- 系统的本质：为 LLM 和用户提供最优上下文

**一（整体）**：
- Memory 2.0 的统一抽象：智能编排器

**二（阴阳）**：
- 阴：被动记录（感知层）
- 阳：主动选择（编排层）
- 平衡：智能理解（理解层，调和阴阳）

**三（变化的基础）**：
- 三层架构：感知 → 理解 → 编排
- 三个维度：内容 × 时间 × 智能

**八卦（变化的特征）**：
- 8种内容类型：纯文本、纯可视化、纯数据、文本+可视化...
- 8种时间情境：当前对话、当前会话、最近会话...

**64卦（组合情境）**：
- 8×8 = 64种智能决策
- 每种情境有特定的最优策略

**384爻（演化细节）**：
- 64×6 = 384个演化阶段
- 状态向量的平滑演化，不是跳转

**错综互卦（转换规律）**：
- 错卦（反转）：success → failure 的对称处理
- 综卦（颠倒）：user → ai 的视角转换
- 互卦（核心）：提取变化的本质规律

### 7.3 与 Phase A（元数据提取器）的哲学一致性

**Phase A**：
- **一分为三**：类型维度 / 职责维度 / 状态维度
- **极简主义**：80% 代码消除，新类型仅需 20 行
- **易变适应**：Trait 编译期展开，零运行时开销

**Phase B**：
- **一分为三**：内容维度 / 时间维度 / 智能维度（9维向量空间）
- **极简主义**：复用 CLI Memory 2.0 架构，扩展而非重写
- **易变适应**：状态演化路径，64种决策自适应调整

**哲学升华**：
- Phase A: 从代码重复到统一抽象
- Phase B: 从静态状态到动态演化
- 共同点: 变化有规律，规律可组合

---

## 八、风险与机遇

### 8.1 风险矩阵

| 风险类型 | 概率 | 影响 | 应对策略 |
|---------|------|------|---------|
| **技术风险** | | | |
| 多模态数据处理复杂度高 | 高 | 高 | ✅ 分层实施，先文本+图表，再扩展 |
| 向量索引性能不达标 | 中 | 中 | ✅ 使用成熟库（faiss/milvus），或降级到关键词 |
| 跨会话推荐准确率低 | 中 | 中 | ✅ A/B测试，用户反馈驱动优化 |
| **产品风险** | | | |
| 用户不理解复杂推荐 | 高 | 高 | ✅ 清晰的解释 + 可选关闭 |
| 推荐打扰用户工作流 | 中 | 高 | ✅ 智能时机选择，非侵入式 |
| 学习周期长，初期效果差 | 高 | 中 | ✅ 预置模式库，冷启动优化 |
| **架构风险** | | | |
| 与 CLI Memory 2.0 不兼容 | 低 | 高 | ✅ 共享核心架构，WebUI 作为扩展层 |
| 内存占用过高 | 中 | 中 | ✅ 流式处理 + LRU 缓存 |
| 复杂度导致难维护 | 中 | 高 | ✅ 充分文档 + 单元测试 > 80% |

### 8.2 机遇分析

**短期机遇**（3-6个月）：
1. **差异化竞争力**：智能推荐是同类产品稀缺能力
2. **用户粘性提升**：跨会话记忆大幅提升体验
3. **降低门槛**：非技术用户也能高效使用

**中期机遇**（6-12个月）：
1. **数据资产积累**：用户行为数据成为核心竞争力
2. **AI 助手进化**：从被动响应到主动建议
3. **生态扩展**：开放推荐 API，第三方插件

**长期机遇**（1年+）：
1. **预测式智能**：不等用户问，主动发现问题
2. **个性化定制**：每个用户有独特的 Memory 模型
3. **知识图谱**：从记忆到知识的演化

### 8.3 成功的关键因素

1. **哲学指导实践**：
   - 不为了技术而技术
   - 始终以"最优上下文"为北极星
   - 变化有规律，规律可组合

2. **用户反馈驱动**：
   - 快速迭代，小步快跑
   - A/B 测试验证假设
   - 从数据中学习，不是主观臆断

3. **工程质量保证**：
   - 单元测试 > 80%
   - 性能基准持续监控
   - 文档与代码同步更新

4. **团队认知对齐**：
   - 全员理解"一分为三"哲学
   - 共识：这不是简单修复，是架构创新
   - 敢于试错，但快速调整

---

## 九、总结与展望

### 9.1 核心价值

**Memory 2.0 WebUI 的独特价值**：

1. **不是 CLI 的移植，而是 WebUI 的重构**
   - 利用 WebUI 的可视化、持久化、交互式能力
   - 解决 WebUI 特有的跨会话、富媒体、非技术用户挑战

2. **不是功能堆砌，而是哲学指导的设计**
   - 一分为三：9维向量空间，不是3个固定状态
   - 易经智慧：64种决策，384个演化细节
   - 状态演化：渐进路径，不是瞬间跳转

3. **不是静态系统，而是自适应学习**
   - 从用户反馈中学习
   - 权重动态调整
   - 持续优化推荐质量

### 9.2 与现有系统的关系

```
┌────────────────────────────────────────────────────────────┐
│              RealConsole 完整架构（v1.53.0+）                │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  【CLI 模式】                                                │
│    • Agent（核心调度）                                        │
│    • UnifiedTracer（四维观测：History/ExecutionLogger/       │
│      LlmLogger/Context）                                    │
│    • Memory 2.0 CLI（智能上下文编排器）                       │
│                                                              │
│  【WebUI 模式】                                              │
│    • WebSocket Handler（通信层）                             │
│    • SessionManager（会话持久化）                             │
│    • MetadataExtractor（元数据提取器，Phase A ✅）           │
│    • Memory 2.0 WebUI（富媒体智能编排器，Phase B 🚀）        │
│                                                              │
│  【共享基础】                                                │
│    • 一分为三哲学（philosophy.md）                           │
│    • 易经智慧（状态演化、规律组合）                           │
│    • 极简主义（最小化重复，清晰抽象）                         │
│                                                              │
└────────────────────────────────────────────────────────────┘
```

### 9.3 下一步行动

**立即行动**（本周）：
1. [ ] 团队评审本设计文档
2. [ ] 确认技术选型（向量库、聚类算法）
3. [ ] 建立开发分支：`feature/memory-2.0-webui`

**Phase 1 启动**（下周）：
1. [ ] 定义核心数据结构
2. [ ] 实现 WebUIPerceptionLayer
3. [ ] 第一个集成测试：采集 WebUI 7维数据

**长期愿景**（3-6个月后）：
- Memory 2.0 WebUI 成为 RealConsole 的杀手级功能
- 用户粘性显著提升（会话复用率 > 70%）
- 非技术用户也能高效使用（推荐采纳率 > 60%）

---

**文档状态**: 深度设计完成 - 待评审
**下一步**: 团队评审，决定是否启动 Phase 1 实施
**负责人**: hongxin
**设计者**: Claude + hongxin
**创建日期**: 2025-11-24

---

## 附录

### 附录 A：术语表

| 术语 | 定义 |
|------|------|
| **一分为三** | 超越二元对立，将状态视为向量空间中的演化路径 |
| **状态向量** | 多维空间中的一个点，代表系统的当前状态 |
| **演化路径** | 状态从一个点到另一个点的渐进变化过程 |
| **64卦决策** | 8种内容×8种时间情境=64种智能决策组合 |
| **自适应学习** | 从用户反馈中调整权重，持续优化推荐质量 |
| **富媒体编排** | 文本+可视化+数据的最优组合，控制在token预算内 |
| **跨会话智能** | 从历史会话中学习，为新任务提供智能推荐 |

### 附录 B：参考资料

**内部文档**：
- `docs/00-core/philosophy.md` - 一分为三哲学（高级理解）
- `docs/04-reports/features/memory/memory-2.0-smart-context-orchestrator-design.md` - CLI Memory 2.0 设计
- `docs/04-reports/features/memory/memory-system-redesign.md` - Memory 1.0 问题分析
- `docs/04-reports/refactoring/metadata-extractor-unified-design.md` - Phase A 设计（元数据提取器）

**外部参考**：
- Claude Code 的上下文管理机制
- LangChain Memory 模块设计
- 易经六十四卦哲学思想
- 向量数据库技术（faiss, milvus）
- 推荐系统原理（协同过滤、内容推荐）

### 附录 C：设计决策记录

#### 决策 #1：9维向量空间而非离散状态

**日期**: 2025-11-24
**决策**: 采用 3×3 = 9维向量空间表示状态
**理由**:
- 符合哲学高级理解：状态是向量，不是离散点
- 支持状态演化：渐进变化，不是跳转
- 灵活扩展：可增加更多维度

**替代方案**: 固定的几个状态（如：Text, Visual, Data）
**为何不选**: 过于僵化，无法表达复杂情境

#### 决策 #2：64种基础决策 + 自适应权重

**日期**: 2025-11-24
**决策**: 8×8易经思想映射，每种决策可自适应调整
**理由**:
- 符合易经智慧：简单规律组合成复杂行为
- 支持学习：权重动态调整
- 可解释性：每种决策都有明确含义

**替代方案**: 纯机器学习黑盒
**为何不选**: 缺乏可解释性，难以调试

#### 决策 #3：分层实施而非一次到位

**日期**: 2025-11-24
**决策**: 4阶段渐进式实施，每阶段3-4周
**理由**:
- 符合易变哲学：可进可退
- 降低风险：每阶段可评估
- 快速验证：早期反馈指导后续

**替代方案**: 6个月一次性开发
**为何不选**: 风险太高，难以调整

---

**版本历史**：
| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|---------|
| v2.0-webui | 2025-11-24 | Claude + hongxin | 初始版本，深度设计 |
