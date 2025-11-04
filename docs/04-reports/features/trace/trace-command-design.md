# /trace 命令详细设计

**创建时间**: 2025-10-22
**关联文档**: `memory-system-redesign.md`, `four-dimensions-philosophy.md`
**状态**: 设计阶段

---

## 目录

- [一、设计目标](#一设计目标)
- [二、核心理念](#二核心理念)
- [三、功能设计](#三功能设计)
- [四、实现方案](#四实现方案)
- [五、用户体验](#五用户体验)
- [六、技术细节](#六技术细节)

---

## 一、设计目标

### 1.1 核心问题

**用户痛点**：
```bash
# 当前：用户需要记住 4 个命令
/history         # 命令统计
/log             # 执行日志
/llm-log         # LLM 详情
/context         # 对话状态

# 问题：
# 1. 记不住各命令的职责
# 2. 不知道该用哪个
# 3. 查询分散，信息割裂
```

**设计目标**：
```bash
# 理想：一个统一入口
/trace           # 智能聚合，自动路由

# 优势：
# 1. 降低学习成本
# 2. 提供整体视图
# 3. 保留深度功能
```

### 1.2 设计原则

1. **统一入口，专业出口**
   - `/trace` 是聚合视图
   - 专用命令是深度功能
   - 两者互补，不替代

2. **智能路由，自动适配**
   - 根据参数自动判断意图
   - 智能选择最佳数据源
   - 减少用户决策负担

3. **渐进增强，向下兼容**
   - 不破坏现有命令
   - 提供快捷方式
   - 支持高级用户

---

## 二、核心理念

### 2.1 统一数据模型

**TraceEntry**：四维统一的抽象

```rust
/// 统一追踪条目
///
/// 聚合四个维度的记录，提供统一视图
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// 唯一 ID
    pub id: Uuid,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 来源维度
    pub dimension: Dimension,

    /// 条目类型
    pub entry_type: EntryType,

    /// 核心内容
    pub content: String,

    /// 状态
    pub status: Status,

    /// 元数据（维度特定）
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 观察维度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    /// 统计维度（History）
    Statistics,

    /// 协同维度（log）
    Coordination,

    /// 黑盒维度（llm-log）
    BlackBox,

    /// 记忆维度（Context）
    Memory,
}

/// 条目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    /// Shell 命令
    ShellCommand,

    /// 系统命令
    SystemCommand,

    /// LLM 对话
    LlmConversation,

    /// LLM API 调用
    LlmApiCall,

    /// 上下文事件
    ContextEvent,
}

/// 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Success,
    Failed,
    InProgress,
    Unknown,
}
```

### 2.2 智能聚合策略

**时序聚合**（默认）：
```
按时间排序，展示最近活动
来源：主要是 log，辅助其他维度
适用：/trace, /trace recent
```

**维度聚合**（专项查询）：
```
路由到特定维度
来源：单一维度系统
适用：/trace shell, /trace llm
```

**关键词聚合**（搜索）：
```
跨四维度搜索
来源：所有维度
适用：/trace search
```

**综合聚合**（仪表板）：
```
统计各维度关键指标
来源：所有维度的 stats
适用：/trace dashboard
```

---

## 三、功能设计

### 3.1 命令架构

```
/trace [子命令] [参数]

子命令：
├─ (无)          - 最近活动（默认）
├─ recent <n>    - 最近 N 条
├─ search <关键词> - 全局搜索
├─ shell         - Shell 命令（路由到 /history）
├─ llm           - LLM 交互（路由到 /llm-log）
├─ context       - 对话状态（路由到 /context）
├─ exec          - 执行日志（路由到 /log）
├─ dashboard     - 综合仪表板
└─ help          - 帮助信息
```

### 3.2 功能详解

#### 3.2.1 最近活动（默认）

**命令**：
```bash
/trace
/trace recent 20
```

**数据来源**：
- 主要：ExecutionLogger（协同维度）- 最全面的执行记录
- 辅助：其他维度的关键事件

**输出格式**：
```
最近的系统活动 (20 条)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

 1. [10:23:45] 📊 SHELL    ✓  git status (150ms)
 2. [10:24:12] 💬 LLM      ✓  解释 Rust 所有权 (2.3s)
    └─ 🤖 deepseek-chat | 1,234 tokens
 3. [10:25:03] ⚙️  CMD      ✓  /history search git (50ms)
 4. [10:26:30] 📊 SHELL    ✗  make buildd (failed)
 5. [10:27:15] 💭 CONTEXT  ⓘ  清除上下文 (空闲超时)
 ...

💡 提示：
  /trace search "git"  - 搜索包含 git 的所有记录
  /trace shell         - 查看 Shell 命令详情
  /trace dashboard     - 查看综合统计
```

**关键特性**：
- 混合展示各类型交互
- 突出显示失败和异常
- 提供维度标识（图标）
- 链接到详细视图

#### 3.2.2 全局搜索

**命令**：
```bash
/trace search "error"
/trace search "rust" --dim llm
/trace search "git" --time 7d
```

**搜索策略**：
```rust
impl UnifiedTracer {
    fn search(&self, keyword: &str, options: SearchOptions) -> SearchResult {
        let mut results = Vec::new();

        // 1. 搜索各维度（并行）
        let (shell_hits, exec_hits, llm_hits, ctx_hits) = tokio::join!(
            self.search_history(keyword),
            self.search_exec_log(keyword),
            self.search_llm_log(keyword),
            self.search_context(keyword),
        );

        // 2. 合并结果
        results.extend(shell_hits.into_trace_entries(Dimension::Statistics));
        results.extend(exec_hits.into_trace_entries(Dimension::Coordination));
        results.extend(llm_hits.into_trace_entries(Dimension::BlackBox));
        results.extend(ctx_hits.into_trace_entries(Dimension::Memory));

        // 3. 按维度过滤
        if let Some(dim) = options.dimension {
            results.retain(|e| e.dimension == dim);
        }

        // 4. 按时间过滤
        if let Some(time_range) = options.time_range {
            results.retain(|e| time_range.contains(e.timestamp));
        }

        // 5. 按相关性排序
        results.sort_by_relevance(keyword);

        // 6. 去重（相同内容的不同维度记录）
        results.dedup_by_content();

        SearchResult {
            keyword: keyword.to_string(),
            total: results.len(),
            entries: results,
            dimensions: self.count_by_dimension(&results),
        }
    }
}
```

**输出格式**：
```
搜索结果："error" (找到 15 条)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

按维度分布：
  📊 Shell (3)  💬 LLM (8)  ⚙️  Exec (4)  💭 Context (0)

结果列表：
 1. [10:26:30] 📊 SHELL ✗
    make buildd
    → error: command not found

 2. [10:15:22] 💬 LLM ✓
    解释这个 error
    → 这是一个常见的编译错误...

 3. [09:45:10] 💬 LLM ✓ (详细)
    Session: abc-123
    Request: "rust error handling"
    Response: 500 tokens
    ⚡ 查看详情: /llm-log replay abc-123

 ...

💡 提示：
  /trace search "error" --dim shell  - 仅搜索 Shell 命令
  /trace search "error" --time 1d    - 仅搜索最近 1 天
```

**高级特性**：
- 并行搜索提高性能
- 智能去重避免冗余
- 相关性排序
- 维度过滤
- 时间范围过滤

#### 3.2.3 维度路由

**命令**：
```bash
/trace shell         # → /history
/trace llm           # → /llm-log status
/trace context       # → /context show
/trace exec          # → /log recent
```

**实现策略**：
```rust
impl UnifiedTracer {
    fn route_to_dimension(&self, dim: Dimension, args: &str) -> String {
        match dim {
            Dimension::Statistics => {
                // 路由到 History
                self.history_cmd.execute(args)
            }
            Dimension::BlackBox => {
                // 路由到 llm-log
                self.llm_log_cmd.execute(args)
            }
            Dimension::Memory => {
                // 路由到 Context
                self.context_cmd.execute(args)
            }
            Dimension::Coordination => {
                // 路由到 log
                self.exec_log_cmd.execute(args)
            }
        }
    }
}
```

**输出格式**：
```
🔄 路由到专用命令: /history
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[接着显示 /history 的输出]

💡 提示：
  直接使用 /history 获得相同结果
  /trace 提供统一入口，专用命令提供深度功能
```

#### 3.2.4 综合仪表板

**命令**：
```bash
/trace dashboard
/trace dashboard --time 7d
```

**设计思路**：
```
四维联动，整体视图
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

┌─────────────────────┬─────────────────────┐
│  📊 统计维度         │  💬 黑盒维度         │
│  (History)          │  (llm-log)          │
│                     │                     │
│  Top 命令:          │  API 调用:          │
│   1. git status×50  │   • 总次数: 127     │
│   2. ls -la ×30     │   • 成功率: 98%     │
│   3. make build×20  │   • 平均延迟: 1.2s  │
│                     │   • Token: 125K     │
│  📈 趋势: ↑ 20%     │  💰 成本: $0.45     │
└─────────────────────┴─────────────────────┘

┌─────────────────────┬─────────────────────┐
│  ⚙️  协同维度        │  💭 记忆维度         │
│  (log)              │  (Context)          │
│                     │                     │
│  总执行: 342 次     │  当前状态: 活跃      │
│  成功率: 95%        │  对话轮次: 5/9      │
│  失败数: 17         │  Token 占用: 3.2K   │
│  平均耗时: 450ms    │  最后活动: 2分钟前   │
│  最慢: /llm (2.3s)  │  自动清理: 30分钟后  │
└─────────────────────┴─────────────────────┘

系统健康度: ● 良好 (95%)

最近异常:
 • [10:26:30] Shell 命令失败: make buildd
 • [09:15:22] LLM 响应慢: 5.2s (超过阈值)

建议:
 1. 检查失败的 Shell 命令
 2. 考虑优化 LLM 请求大小
 3. Context 即将超时，需要新输入

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
💡 快捷操作：
  /trace search "failed"  - 查看所有失败记录
  /log failed             - 查看失败详情
  /llm-log stats --days 7 - 查看 LLM 统计
```

**数据聚合**：
```rust
struct Dashboard {
    // 统计维度
    statistics: HistoryStats,

    // 协同维度
    coordination: ExecutionStats,

    // 黑盒维度
    blackbox: LlmStats,

    // 记忆维度
    memory: ContextStatus,

    // 系统整体
    health_score: f64,
    recent_anomalies: Vec<Anomaly>,
    recommendations: Vec<String>,
}
```

---

## 四、实现方案

### 4.1 核心架构

```rust
/// 统一追踪系统
///
/// 聚合四个维度的观察系统，提供统一查询接口
pub struct UnifiedTracer {
    /// 统计维度（History）
    history: Arc<RwLock<HistoryManager>>,

    /// 协同维度（ExecutionLogger）
    exec_logger: Arc<RwLock<ExecutionLogger>>,

    /// 黑盒维度（LlmLogger）
    llm_logger: Option<Arc<LlmLogger>>,

    /// 记忆维度（ContextManager）
    context: Arc<RwLock<ContextManager>>,
}

impl UnifiedTracer {
    /// 创建新的统一追踪器
    pub fn new(
        history: Arc<RwLock<HistoryManager>>,
        exec_logger: Arc<RwLock<ExecutionLogger>>,
        llm_logger: Option<Arc<LlmLogger>>,
        context: Arc<RwLock<ContextManager>>,
    ) -> Self {
        Self {
            history,
            exec_logger,
            llm_logger,
            context,
        }
    }

    /// 最近活动
    pub async fn recent(&self, n: usize) -> Vec<TraceEntry> {
        // 主要从 exec_logger 获取
        let exec_entries = self.exec_logger.read().await.recent(n);

        // 转换为统一格式
        exec_entries
            .into_iter()
            .map(|e| TraceEntry::from_execution_log(e))
            .collect()
    }

    /// 全局搜索
    pub async fn search(
        &self,
        keyword: &str,
        options: SearchOptions
    ) -> SearchResult {
        // 并行搜索各维度
        let (shell_hits, exec_hits, llm_hits, ctx_hits) = tokio::join!(
            self.search_history(keyword),
            self.search_exec_log(keyword),
            self.search_llm_log(keyword),
            self.search_context(keyword),
        );

        // 聚合和排序
        self.aggregate_search_results(
            keyword,
            shell_hits,
            exec_hits,
            llm_hits,
            ctx_hits,
            options
        )
    }

    /// 综合仪表板
    pub async fn dashboard(&self, time_range: Option<TimeRange>) -> Dashboard {
        Dashboard {
            statistics: self.history.read().await.stats(),
            coordination: self.exec_logger.read().await.stats(),
            blackbox: self.llm_logger.as_ref()
                .map(|l| l.get_statistics(time_range.map(|r| r.days)))
                .unwrap_or_default(),
            memory: self.context.read().await.status(),

            health_score: self.calculate_health_score().await,
            recent_anomalies: self.detect_anomalies().await,
            recommendations: self.generate_recommendations().await,
        }
    }

    /// 路由到专用命令
    pub fn route(&self, dimension: Dimension, args: &str) -> String {
        match dimension {
            Dimension::Statistics => {
                format!("🔄 路由到: /history {}\n\n{}",
                    args,
                    handle_history(args, Arc::clone(&self.history)))
            }
            // ... 其他维度
        }
    }
}
```

### 4.2 关键算法

#### 4.2.1 智能去重

```rust
impl UnifiedTracer {
    /// 智能去重
    ///
    /// 问题：同一事件可能在多个维度都有记录
    /// 策略：保留信息最丰富的记录
    fn dedup_entries(&self, entries: Vec<TraceEntry>) -> Vec<TraceEntry> {
        let mut deduped = Vec::new();
        let mut seen: HashMap<String, TraceEntry> = HashMap::new();

        for entry in entries {
            let key = self.generate_dedup_key(&entry);

            match seen.get_mut(&key) {
                Some(existing) => {
                    // 如果新记录更详细，替换
                    if entry.metadata.len() > existing.metadata.len() {
                        *existing = entry;
                    } else {
                        // 否则合并元数据
                        existing.metadata.extend(entry.metadata);
                    }
                }
                None => {
                    seen.insert(key, entry);
                }
            }
        }

        seen.into_values().collect()
    }

    /// 生成去重键
    fn generate_dedup_key(&self, entry: &TraceEntry) -> String {
        // 基于时间窗口和内容生成键
        let time_window = entry.timestamp.timestamp() / 10; // 10秒窗口
        format!("{}-{}", time_window, self.normalize_content(&entry.content))
    }

    /// 归一化内容（用于去重比较）
    fn normalize_content(&self, content: &str) -> String {
        // 移除前缀（!、/等）
        // 移除时间戳
        // 移除空白字符
        content
            .trim_start_matches(['!', '/'])
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}
```

#### 4.2.2 相关性排序

```rust
impl UnifiedTracer {
    /// 按相关性排序搜索结果
    fn sort_by_relevance(&self, entries: &mut Vec<TraceEntry>, keyword: &str) {
        entries.sort_by_cached_key(|entry| {
            let score = self.calculate_relevance_score(entry, keyword);
            // 负数用于降序排序
            std::cmp::Reverse((score * 1000.0) as i64)
        });
    }

    /// 计算相关性得分
    fn calculate_relevance_score(&self, entry: &TraceEntry, keyword: &str) -> f64 {
        let keyword_lower = keyword.to_lowercase();
        let content_lower = entry.content.to_lowercase();

        let mut score = 0.0;

        // 1. 完全匹配加分
        if content_lower == keyword_lower {
            score += 10.0;
        }

        // 2. 包含次数
        let count = content_lower.matches(&keyword_lower).count();
        score += count as f64 * 2.0;

        // 3. 位置权重（越靠前越重要）
        if let Some(pos) = content_lower.find(&keyword_lower) {
            let position_weight = 1.0 - (pos as f64 / content_lower.len() as f64);
            score += position_weight * 3.0;
        }

        // 4. 时间衰减（最近的更相关）
        let age = (Utc::now() - entry.timestamp).num_seconds();
        let recency_weight = (-age as f64 / 86400.0 / 7.0).exp(); // 7天半衰期
        score *= recency_weight;

        // 5. 维度权重
        let dim_weight = match entry.dimension {
            Dimension::Coordination => 1.2,  // 协同维度稍微加权
            _ => 1.0,
        };
        score *= dim_weight;

        score
    }
}
```

#### 4.2.3 健康度计算

```rust
impl UnifiedTracer {
    /// 计算系统健康度
    async fn calculate_health_score(&self) -> f64 {
        let exec_stats = self.exec_logger.read().await.stats();
        let llm_stats = self.llm_logger.as_ref()
            .map(|l| l.get_statistics(None))
            .unwrap_or_default();

        let mut score = 100.0;

        // 1. 执行成功率
        let exec_success_rate = exec_stats.success_rate();
        score *= exec_success_rate / 100.0;

        // 2. LLM 成功率
        if let Some(llm) = &self.llm_logger {
            let llm_success_rate = llm_stats.successful_requests as f64
                / llm_stats.total_requests as f64 * 100.0;
            score *= llm_success_rate / 100.0;
        }

        // 3. 响应时间（惩罚慢响应）
        if exec_stats.avg_duration_ms > 1000.0 {
            score *= 0.9; // 平均超过1秒，扣10分
        }

        if llm_stats.avg_latency_ms > 5000 {
            score *= 0.9; // LLM平均超过5秒，扣10分
        }

        // 4. 错误率
        let error_rate = exec_stats.failed as f64 / exec_stats.total as f64;
        if error_rate > 0.1 {
            score *= 0.8; // 错误率超过10%，扣20分
        }

        score.max(0.0).min(100.0)
    }
}
```

---

## 五、用户体验

### 5.1 学习曲线

**第一阶段：初学者**（只用 /trace）
```bash
/trace                    # 查看最近活动
/trace search "error"     # 搜索问题
```

**第二阶段：普通用户**（学会维度路由）
```bash
/trace shell              # 查看命令历史
/trace llm                # 查看 LLM 详情
/trace dashboard          # 查看整体状况
```

**第三阶段：高级用户**（直接用专用命令）
```bash
/history search git       # 精确命令统计
/llm-log replay <id>      # 详细 API 调用
/log failed               # 失败记录分析
```

### 5.2 渐进式披露

**信息层次**：
```
Level 1: 摘要（/trace）
  ├─ 显示最近活动
  ├─ 突出异常
  └─ 提示深入命令

Level 2: 分类（/trace shell/llm/...）
  ├─ 路由到专用命令
  ├─ 显示该维度概览
  └─ 提示详细操作

Level 3: 详情（专用命令）
  ├─ 完整的功能
  ├─ 高级选项
  └─ 导出和分析
```

### 5.3 快捷操作

**智能提示**：
```bash
# 当搜索结果较多时
💡 找到 50 条结果，仅显示前 20 条
   /trace search "error" --dim shell  # 缩小范围
   /trace search "error" --time 1d    # 限制时间

# 当发现异常时
⚠️  检测到 5 条失败记录
   /log failed                         # 查看详情
   /trace search "failed" --time 1h    # 搜索失败

# 当性能异常时
📊 平均响应时间较高 (1.5s)
   /log stats                          # 查看性能统计
   /trace dashboard                    # 查看整体健康度
```

---

## 六、技术细节

### 6.1 性能优化

#### 6.1.1 缓存策略

```rust
pub struct UnifiedTracer {
    // ... 其他字段

    /// 搜索结果缓存（LRU）
    search_cache: Arc<Mutex<LruCache<String, CachedSearchResult>>>,

    /// 仪表板缓存（时效性）
    dashboard_cache: Arc<RwLock<Option<(Dashboard, Instant)>>>,
}

impl UnifiedTracer {
    /// 带缓存的搜索
    pub async fn search_cached(
        &self,
        keyword: &str,
        options: SearchOptions
    ) -> SearchResult {
        let cache_key = format!("{}-{:?}", keyword, options);

        // 尝试从缓存获取
        if let Some(cached) = self.search_cache.lock().await.get(&cache_key) {
            if cached.is_fresh() {
                return cached.result.clone();
            }
        }

        // 缓存未命中，执行搜索
        let result = self.search(keyword, options).await;

        // 更新缓存
        self.search_cache.lock().await.put(
            cache_key,
            CachedSearchResult {
                result: result.clone(),
                timestamp: Instant::now(),
            }
        );

        result
    }

    /// 带缓存的仪表板
    pub async fn dashboard_cached(&self) -> Dashboard {
        // 检查缓存（30秒有效期）
        {
            let cache = self.dashboard_cache.read().await;
            if let Some((dashboard, timestamp)) = cache.as_ref() {
                if timestamp.elapsed() < Duration::from_secs(30) {
                    return dashboard.clone();
                }
            }
        }

        // 重新计算
        let dashboard = self.dashboard(None).await;

        // 更新缓存
        *self.dashboard_cache.write().await = Some((dashboard.clone(), Instant::now()));

        dashboard
    }
}
```

#### 6.1.2 并行查询

```rust
impl UnifiedTracer {
    /// 并行搜索各维度
    async fn parallel_search(&self, keyword: &str) -> SearchResult {
        // 使用 tokio::join! 并行执行
        let (history_results, exec_results, llm_results, ctx_results) = tokio::join!(
            async {
                self.history.read().await.search(keyword, Smart)
            },
            async {
                self.exec_logger.read().await.search(keyword)
            },
            async {
                if let Some(ref llm) = self.llm_logger {
                    llm.search_logs(keyword, None)
                } else {
                    vec![]
                }
            },
            async {
                // Context 搜索（如果需要）
                vec![]
            }
        );

        // 聚合结果
        self.aggregate_results(history_results, exec_results, llm_results, ctx_results)
    }
}
```

### 6.2 错误处理

```rust
/// 统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum TraceError {
    #[error("搜索失败: {0}")]
    SearchFailed(String),

    #[error("维度不可用: {0:?}")]
    DimensionUnavailable(Dimension),

    #[error("参数无效: {0}")]
    InvalidParameter(String),

    #[error("内部错误: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl UnifiedTracer {
    /// 安全的搜索（优雅降级）
    pub async fn search_safe(&self, keyword: &str) -> Result<SearchResult, TraceError> {
        // 即使某个维度失败，也继续其他维度
        let history_results = self.search_history(keyword).await
            .unwrap_or_else(|e| {
                eprintln!("⚠️  History 搜索失败: {}", e);
                vec![]
            });

        let exec_results = self.search_exec_log(keyword).await
            .unwrap_or_else(|e| {
                eprintln!("⚠️  ExecLog 搜索失败: {}", e);
                vec![]
            });

        // ... 其他维度

        Ok(self.aggregate_results(history_results, exec_results, ...))
    }
}
```

### 6.3 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// 测试最近活动
    #[tokio::test]
    async fn test_recent() {
        let tracer = create_test_tracer().await;
        let entries = tracer.recent(10).await;

        assert!(!entries.is_empty());
        assert!(entries.len() <= 10);

        // 验证时间排序
        for window in entries.windows(2) {
            assert!(window[0].timestamp >= window[1].timestamp);
        }
    }

    /// 测试全局搜索
    #[tokio::test]
    async fn test_search() {
        let tracer = create_test_tracer().await;

        // 添加测试数据
        add_test_data(&tracer).await;

        let result = tracer.search("test", SearchOptions::default()).await;

        assert!(result.total > 0);
        assert!(result.entries.iter().all(|e|
            e.content.to_lowercase().contains("test")
        ));
    }

    /// 测试去重
    #[tokio::test]
    async fn test_deduplication() {
        let tracer = create_test_tracer().await;

        // 创建重复条目
        let entries = vec![
            TraceEntry { content: "git status".to_string(), ... },
            TraceEntry { content: "git status".to_string(), ... },
        ];

        let deduped = tracer.dedup_entries(entries);
        assert_eq!(deduped.len(), 1);
    }
}
```

---

## 七、迁移和兼容性

### 7.1 向后兼容

**保留所有现有命令**：
```bash
# 旧命令继续工作
/history             # ✅ 正常工作
/log                 # ✅ 正常工作
/llm-log             # ✅ 正常工作
/context             # ✅ 正常工作

# 新命令作为补充
/trace               # ✨ 新增
```

### 7.2 渐进式迁移

**阶段 1**：添加 /trace（不影响现有）
**阶段 2**：在现有命令中提示 /trace
**阶段 3**：观察用户使用习惯
**阶段 4**：根据反馈调整

### 7.3 用户教育

**在现有命令中添加提示**：
```rust
// history_cmd.rs
fn handle_history(args: &str, ...) -> String {
    let result = /* 原有逻辑 */;

    format!(
        "{}\n\n{}",
        result,
        "💡 提示: 使用 /trace 可以跨多个维度搜索".dimmed()
    )
}
```

---

## 八、总结

### 核心价值

1. **降低学习成本**
   - 一个入口 vs 四个命令
   - 智能路由 vs 手动选择

2. **提供整体视图**
   - 跨维度聚合
   - 综合仪表板

3. **保留专业深度**
   - 不替代专用命令
   - 提供快捷入口

### 设计亮点

1. **统一数据模型**（TraceEntry）
2. **智能去重算法**
3. **相关性排序**
4. **并行查询优化**
5. **优雅降级处理**

### 下一步

1. 实现核心架构
2. 完成单元测试
3. 集成到 RealConsole
4. 收集用户反馈
5. 持续迭代优化

---

**相关文档**:
- `memory-system-redesign.md` - 整体设计方案
- `four-dimensions-philosophy.md` - 哲学基础

**变更历史**:
| 版本 | 日期 | 说明 |
|------|------|------|
| v1.0 | 2025-10-22 | 初始详细设计 |
