//! Memory 2.0 WebUI: 智能上下文编排器
//!
//! v1.55.0: 基于"一分为三"哲学的富媒体智能记忆系统
//!
//! ## 设计哲学
//!
//! **9维向量空间** = 内容 × 时间 × 智能
//! - 内容维度（3）: 文本 / 可视化 / 数据
//! - 时间维度（3）: 短期 / 中期 / 长期
//! - 智能维度（3）: 感知 / 理解 / 编排
//!
//! **易经64卦决策**：8种内容 × 8种时间情境 = 64种智能决策
//!
//! **状态演化**：向量空间中的渐进路径，不是跳转
//!
//! ## 三层架构
//!
//! ```text
//! 感知层（Perception）
//!   ↓ 采集 CLI 4维 + WebUI 3维数据
//! 理解层（Understanding）
//!   ↓ 语义分析 + 相关性评分 + 时间衰减
//! 编排层（Orchestration）
//!   ↓ Token 管理 + 贪心选择 + 富媒体组合
//! 输出：OptimizedMultimodalContext
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! let orchestrator = SmartWebUIOrchestrator::new(...);
//!
//! // 为当前任务提取相关上下文
//! let context = orchestrator
//!     .extract_relevant_context("分析Q2销售数据", None, 2000)
//!     .await?;
//!
//! // 获取跨会话推荐
//! let recommendations = orchestrator
//!     .recommend_from_sessions(Some(&data_profile))
//!     .await?;
//! ```

pub mod types;
pub mod perception;
pub mod understanding;
pub mod orchestration;

use anyhow::Result;
use perception::{TimeRange, WebUIPerceptionLayer};
use understanding::WebUIUnderstandingLayer;
use orchestration::{WebUIOrchestrationLayer, OptimizedMultimodalContext, Recommendation};
use crate::web::session_manager::SessionManager;
use crate::web::session::{ChartHistoryEntry, ImageHistoryEntry};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// 重新导出常用类型
pub use types::{
    MultimodalChunk, MemoryStateVector, MultimodalContent,
    ContentType, TimeContext, MemoryAction, DataDimension,
};
pub use orchestration::{
    TextChunk, ChartReference, ImageReference, DataSummary,
    RecommendationType,
};
pub use understanding::{
    TaskComplexity, TaskComplexityAnalyzer, adaptive_token_budget,
    TopicExtractor,
};

// ============================================================================
// 主接口：SmartWebUIOrchestrator
// ============================================================================

/// Memory 2.0 WebUI: 智能上下文编排器
///
/// 统一接口，整合三层架构
pub struct SmartWebUIOrchestrator {
    /// 感知层
    perception: WebUIPerceptionLayer,

    /// 理解层
    understanding: WebUIUnderstandingLayer,

    /// 编排层
    orchestration: WebUIOrchestrationLayer,

    /// 智能缓存（v1.55.0）- 使用 Mutex 实现内部可变性
    cache: Mutex<QueryCache>,
}

impl SmartWebUIOrchestrator {
    /// 创建编排器
    ///
    /// # 参数
    /// - `chart_history`: 图表历史（共享状态）
    /// - `image_history`: 图像历史（共享状态）
    ///
    /// # 注意
    /// SessionManager 在内部创建（因为它是无状态的）
    pub fn new(
        chart_history: Arc<RwLock<Vec<ChartHistoryEntry>>>,
        image_history: Arc<RwLock<Vec<ImageHistoryEntry>>>,
    ) -> Result<Self> {
        // 创建 SessionManager（无状态，可多实例）
        let session_manager = Arc::new(SessionManager::new()?);

        Ok(Self {
            perception: WebUIPerceptionLayer::new(
                session_manager,
                chart_history,
                image_history,
            ),
            understanding: WebUIUnderstandingLayer::new(),
            orchestration: WebUIOrchestrationLayer::new(),
            cache: Mutex::new(QueryCache::new()),
        })
    }

    /// 核心方法：为当前任务提取相关上下文
    ///
    /// # 参数
    /// - `task`: 当前任务描述
    /// - `time_range`: 时间范围（None = 全部）
    /// - `token_budget`: Token 预算（None = 自适应）
    ///
    /// # 返回
    /// 优化后的富媒体上下文
    ///
    /// # 示例
    ///
    /// ```ignore
    /// // 使用自适应 budget（v1.55.0+）
    /// let context = orchestrator
    ///     .extract_relevant_context("调试 Rust trait 问题", None, None)
    ///     .await?;
    ///
    /// // 使用固定 budget
    /// let context = orchestrator
    ///     .extract_relevant_context("分析数据", None, Some(4000))
    ///     .await?;
    /// ```
    pub async fn extract_relevant_context(
        &self,
        task: &str,
        time_range: Option<TimeRange>,
        token_budget: Option<usize>,
    ) -> Result<OptimizedMultimodalContext> {
        eprintln!("\n[Memory 2.0] Extracting context for: \"{}\"", task);

        // v1.55.0: 自适应 Token Budget
        let budget = token_budget.unwrap_or_else(|| adaptive_token_budget(task));
        eprintln!("[Memory 2.0] Token budget: {} {}",
            budget,
            if token_budget.is_none() { "(adaptive)" } else { "(manual)" }
        );

        // 生成缓存键（基于任务和时间范围）
        let cache_key = generate_cache_key((task, format!("{:?}", time_range)));

        // 1-2. 感知层 + 理解层（带缓存）
        let scored_chunks = {
            // 检查缓存（使用 Mutex 锁定）
            let cached_result = {
                let mut cache = self.cache.lock().await;
                cache.get_search_cached(cache_key)
            };

            if let Some(cached) = cached_result {
                eprintln!("[Memory 2.0] ✨ Using cached scored chunks");
                cached
            } else {
                // 缓存未命中，执行完整流程
                eprintln!("[Memory 2.0] Cache miss, performing full perception + understanding");

                // 1. 感知层：采集多模态数据
                let chunks = self.perception
                    .collect_multimodal_data(time_range, None)
                    .await?;

                if chunks.is_empty() {
                    eprintln!("[Memory 2.0] No data collected, returning empty context");
                    return Ok(OptimizedMultimodalContext {
                        text_chunks: vec![],
                        chart_references: vec![],
                        image_references: vec![],
                        data_summaries: vec![],
                        recommendations: vec![],
                        pre_filled_params: std::collections::HashMap::new(),
                        total_tokens: 0,
                        metadata: orchestration::ContextMetadata {
                            chunk_count: 0,
                            dimension_distribution: std::collections::HashMap::new(),
                            avg_relevance: 0.0,
                        },
                        chunks: vec![],
                        avg_score: 0.0,
                    });
                }

                // 2. 理解层：分析相关性
                let scored = self.understanding
                    .score_relevance(task, chunks)
                    .await?;

                // 存入缓存
                {
                    let mut cache = self.cache.lock().await;
                    cache.put_search_cached(cache_key, scored.clone());
                }

                scored
            }
        };

        // 3. 编排层：优化组合（不缓存，因为 budget 可能不同）
        let context = self.orchestration
            .build_optimized_context(scored_chunks, budget)
            .await?;

        eprintln!("[Memory 2.0] Context built:");
        eprintln!("  - Text chunks: {}", context.text_chunks.len());
        eprintln!("  - Chart references: {}", context.chart_references.len());
        eprintln!("  - Recommendations: {}", context.recommendations.len());
        eprintln!("  - Total tokens: {}", context.total_tokens);

        Ok(context)
    }

    /// WebUI 特有：跨会话智能推荐
    ///
    /// v2.2.1: 基于会话相似度的智能推荐
    ///
    /// # 参数
    /// - `current_task`: 当前任务描述（None = 通用推荐）
    ///
    /// # 返回
    /// 推荐列表（最多3条）
    pub async fn recommend_from_sessions(
        &self,
        current_task: Option<&str>,
    ) -> Result<Vec<Recommendation>> {
        // 1. 采集所有会话数据（最近7天）
        let time_range = TimeRange::recent_days(7);
        let chunks = self.perception
            .collect_multimodal_data(Some(time_range), Some(vec![types::DataDimension::SessionManager]))
            .await?;

        if chunks.is_empty() {
            return Ok(vec![]);
        }

        // 2. 如果有当前任务，计算相关性评分
        let scored_chunks = if let Some(task) = current_task {
            self.understanding.score_relevance(task, chunks).await?
        } else {
            // 无任务时，按时间排序
            chunks
        };

        // 3. 生成推荐（取前3个高相关会话）
        let mut recommendations = Vec::new();
        for chunk in scored_chunks.iter().take(5) {
            // 只推荐相关性 > 0.3 的会话
            let relevance = chunk.state_vector.understanding_score;
            if current_task.is_some() && relevance < 0.3 {
                continue;
            }

            if let MultimodalContent::SessionSummary {
                session_id,
                name,
                topic,
                round_count,
                last_message,
            } = &chunk.content
            {
                let description = if current_task.is_some() {
                    format!(
                        "会话「{}」与当前任务相关 (相关度: {:.0}%)\n主题: {}\n最近消息: {}",
                        name,
                        relevance * 100.0,
                        topic.as_deref().unwrap_or("未知"),
                        truncate_message(last_message, 50)
                    )
                } else {
                    format!(
                        "最近活跃的会话「{}」({} 轮对话)\n主题: {}",
                        name,
                        round_count,
                        topic.as_deref().unwrap_or("未知")
                    )
                };

                let rec = Recommendation {
                    type_: RecommendationType::WorkflowReuse,
                    description,
                    confidence: if current_task.is_some() { relevance } else { 0.5 },
                    actions: vec![format!("切换到会话: {}", session_id)],
                    pre_fill_params: std::collections::HashMap::new(),
                };

                recommendations.push(rec);

                if recommendations.len() >= 3 {
                    break;
                }
            }
        }

        eprintln!(
            "[Memory 2.0] Generated {} cross-session recommendations",
            recommendations.len()
        );

        Ok(recommendations)
    }

    /// 快速查询：仅返回最相关的前 N 个片段
    ///
    /// 用于快速预览，不进行完整的编排
    pub async fn quick_search(
        &self,
        task: &str,
        top_k: usize,
    ) -> Result<Vec<MultimodalChunk>> {
        // 生成缓存键（基于任务和固定的 7 天时间范围）
        let time_range = TimeRange::recent_days(7);
        let cache_key = generate_cache_key((task, format!("{:?}", time_range)));

        // 检查缓存
        let mut scored_chunks = {
            let cached_result = {
                let mut cache = self.cache.lock().await;
                cache.get_search_cached(cache_key)
            };

            if let Some(cached) = cached_result {
                eprintln!("[Quick Search] ✨ Using cached result");
                cached
            } else {
                // 缓存未命中，执行完整流程
                eprintln!("[Quick Search] Cache miss, performing full search");

                // 1. 采集数据（最近7天）
                let chunks = self.perception
                    .collect_multimodal_data(Some(time_range), None)
                    .await?;

                // 2. 评分
                let scored = self.understanding
                    .score_relevance(task, chunks)
                    .await?;

                // 存入缓存
                {
                    let mut cache = self.cache.lock().await;
                    cache.put_search_cached(cache_key, scored.clone());
                }

                scored
            }
        };

        // 3. 返回前 K 个
        scored_chunks.truncate(top_k);

        Ok(scored_chunks)
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> Result<MemoryStats> {
        // 采集所有数据（不限时间）
        let chunks = self.perception
            .collect_multimodal_data(None, None)
            .await?;

        let total_chunks = chunks.len();

        // 按维度统计
        let mut dimension_counts = std::collections::HashMap::new();
        for chunk in &chunks {
            *dimension_counts.entry(chunk.dimension).or_insert(0) += 1;
        }

        Ok(MemoryStats {
            total_chunks,
            dimension_counts,
        })
    }
}

// ============================================================================
// 智能缓存（v1.55.0）
// ============================================================================

/// 查询缓存
///
/// v1.55.0: 模仿人脑程序性记忆的 LRU 缓存
/// - 第一次：思考 → 记忆（慢）
/// - 第二次：直接调用记忆（快）
/// - 第N次：自动化反应（极快）
pub struct QueryCache {
    /// 搜索结果缓存：`(query_hash, time_range) → Vec<MultimodalChunk>`
    search_cache: LruCache<u64, CacheEntry<Vec<MultimodalChunk>>>,

    /// 上下文构建缓存：`(task_hash, budget) → OptimizedMultimodalContext`
    context_cache: LruCache<u64, CacheEntry<OptimizedMultimodalContext>>,

    /// TTL: 5 minutes（缓存过期时间）
    ttl: std::time::Duration,
}

/// 缓存条目（带时间戳）
struct CacheEntry<T> {
    data: T,
    timestamp: std::time::Instant,
}

impl<T> CacheEntry<T> {
    fn new(data: T) -> Self {
        Self {
            data,
            timestamp: std::time::Instant::now(),
        }
    }

    fn is_expired(&self, ttl: std::time::Duration) -> bool {
        self.timestamp.elapsed() > ttl
    }
}

impl QueryCache {
    /// 创建缓存
    pub fn new() -> Self {
        Self {
            // 搜索缓存：100 条
            search_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            // 上下文缓存：50 条
            context_cache: LruCache::new(NonZeroUsize::new(50).unwrap()),
            // TTL: 5 分钟
            ttl: std::time::Duration::from_secs(300),
        }
    }

    /// 获取或计算搜索结果
    pub fn get_or_compute_search<F>(
        &mut self,
        key: u64,
        compute: F,
    ) -> Vec<MultimodalChunk>
    where
        F: FnOnce() -> Vec<MultimodalChunk>,
    {
        // 检查缓存
        if let Some(entry) = self.search_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                eprintln!("[Cache] 🎯 Search cache HIT for key: {}", key);
                return entry.data.clone();
            } else {
                eprintln!("[Cache] ⏰ Search cache EXPIRED for key: {}", key);
            }
        }

        // 缓存未命中，执行计算
        eprintln!("[Cache] ❌ Search cache MISS for key: {}", key);
        let result = compute();

        // 存入缓存
        self.search_cache.put(key, CacheEntry::new(result.clone()));

        result
    }

    /// 获取或计算上下文
    pub fn get_or_compute_context<F>(
        &mut self,
        key: u64,
        compute: F,
    ) -> OptimizedMultimodalContext
    where
        F: FnOnce() -> OptimizedMultimodalContext,
    {
        // 检查缓存
        if let Some(entry) = self.context_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                eprintln!("[Cache] 🎯 Context cache HIT for key: {}", key);
                return entry.data.clone();
            } else {
                eprintln!("[Cache] ⏰ Context cache EXPIRED for key: {}", key);
            }
        }

        // 缓存未命中，执行计算
        eprintln!("[Cache] ❌ Context cache MISS for key: {}", key);
        let result = compute();

        // 存入缓存
        self.context_cache.put(key, CacheEntry::new(result.clone()));

        result
    }

    /// 清空所有缓存（当新数据写入时调用）
    pub fn clear_all(&mut self) {
        eprintln!("[Cache] 🔄 Clearing all caches due to new data");
        self.search_cache.clear();
        self.context_cache.clear();
    }

    /// 获取缓存统计
    pub fn stats(&self) -> (usize, usize) {
        (self.search_cache.len(), self.context_cache.len())
    }

    /// 获取缓存的搜索结果（简化版，用于 async 场景）
    pub fn get_search_cached(&mut self, key: u64) -> Option<Vec<MultimodalChunk>> {
        if let Some(entry) = self.search_cache.get(&key) {
            if !entry.is_expired(self.ttl) {
                eprintln!("[Cache] 🎯 Search cache HIT for key: {}", key);
                return Some(entry.data.clone());
            } else {
                eprintln!("[Cache] ⏰ Search cache EXPIRED for key: {}", key);
            }
        }
        eprintln!("[Cache] ❌ Search cache MISS for key: {}", key);
        None
    }

    /// 存入搜索结果到缓存（简化版，用于 async 场景）
    pub fn put_search_cached(&mut self, key: u64, data: Vec<MultimodalChunk>) {
        eprintln!("[Cache] 💾 Storing search result in cache, key: {}", key);
        self.search_cache.put(key, CacheEntry::new(data));
    }
}

impl Default for QueryCache {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成缓存键
///
/// 基于多个参数生成唯一的哈希值
fn generate_cache_key<H: Hash>(params: H) -> u64 {
    let mut hasher = DefaultHasher::new();
    params.hash(&mut hasher);
    hasher.finish()
}

// ============================================================================
// 统计信息
// ============================================================================

/// Memory 统计信息
#[derive(Debug)]
pub struct MemoryStats {
    /// 总片段数
    pub total_chunks: usize,

    /// 各维度片段数
    pub dimension_counts: std::collections::HashMap<DataDimension, usize>,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 截断消息文本
fn truncate_message(message: &str, max_len: usize) -> String {
    if message.chars().count() <= max_len {
        message.to_string()
    } else {
        let truncated: String = message.chars().take(max_len).collect();
        format!("{}...", truncated)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::memory::types::{DataDimension, MultimodalContent};
    use chrono::Utc;

    // ========================================================================
    // QueryCache 测试
    // ========================================================================

    #[test]
    fn test_query_cache_new() {
        let cache = QueryCache::new();
        let (search_len, context_len) = cache.stats();
        assert_eq!(search_len, 0);
        assert_eq!(context_len, 0);
    }

    #[test]
    fn test_query_cache_default() {
        let cache = QueryCache::default();
        let (search_len, context_len) = cache.stats();
        assert_eq!(search_len, 0);
        assert_eq!(context_len, 0);
    }

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new(42);
        assert_eq!(entry.data, 42);
        assert!(!entry.is_expired(std::time::Duration::from_secs(60)));
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = CacheEntry::new("test");
        // 立即检查不应过期
        assert!(!entry.is_expired(std::time::Duration::from_secs(60)));
        // 0秒TTL应该立即过期
        assert!(entry.is_expired(std::time::Duration::from_secs(0)));
    }

    #[test]
    fn test_search_cache_hit_miss() {
        let mut cache = QueryCache::new();

        // First access - miss
        let result = cache.get_search_cached(12345);
        assert!(result.is_none());

        // Store data
        let chunks = vec![MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "test".to_string(),
            },
            Utc::now(),
        )];
        cache.put_search_cached(12345, chunks.clone());

        // Second access - hit
        let result = cache.get_search_cached(12345);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_get_or_compute_search() {
        let mut cache = QueryCache::new();
        let mut compute_count = 0;

        // First call - compute
        let result1 = cache.get_or_compute_search(12345, || {
            compute_count += 1;
            vec![MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text {
                    content: "computed".to_string(),
                },
                Utc::now(),
            )]
        });
        assert_eq!(compute_count, 1);
        assert_eq!(result1.len(), 1);

        // Second call - from cache (compute should not run)
        let result2 = cache.get_or_compute_search(12345, || {
            compute_count += 1;
            vec![]
        });
        assert_eq!(compute_count, 1); // Still 1, not incremented
        assert_eq!(result2.len(), 1);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = QueryCache::new();

        // Add some data
        cache.put_search_cached(1, vec![]);
        cache.put_search_cached(2, vec![]);

        let (search_len, _) = cache.stats();
        assert_eq!(search_len, 2);

        // Clear
        cache.clear_all();

        let (search_len, context_len) = cache.stats();
        assert_eq!(search_len, 0);
        assert_eq!(context_len, 0);
    }

    #[test]
    fn test_cache_lru_eviction() {
        // Create small cache for testing
        let mut cache = QueryCache {
            search_cache: lru::LruCache::new(std::num::NonZeroUsize::new(2).unwrap()),
            context_cache: lru::LruCache::new(std::num::NonZeroUsize::new(2).unwrap()),
            ttl: std::time::Duration::from_secs(300),
        };

        // Add 3 items to a cache with capacity 2
        cache.put_search_cached(1, vec![]);
        cache.put_search_cached(2, vec![]);
        cache.put_search_cached(3, vec![]); // Should evict key 1

        let (search_len, _) = cache.stats();
        assert_eq!(search_len, 2);

        // Key 1 should be evicted
        assert!(cache.get_search_cached(1).is_none());
        // Keys 2 and 3 should still exist
        assert!(cache.get_search_cached(2).is_some());
        assert!(cache.get_search_cached(3).is_some());
    }

    #[test]
    fn test_generate_cache_key() {
        let key1 = generate_cache_key("test");
        let key2 = generate_cache_key("test");
        let key3 = generate_cache_key("different");

        // Same input should give same key
        assert_eq!(key1, key2);
        // Different input should give different key
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_generate_cache_key_tuple() {
        let key1 = generate_cache_key(("query", 1000));
        let key2 = generate_cache_key(("query", 1000));
        let key3 = generate_cache_key(("query", 2000));

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    // ========================================================================
    // 其他测试
    // ========================================================================

    #[test]
    fn test_module_compiles() {
        // 确保模块可以编译
        assert!(true);
    }

    // ========================================================================
    // truncate_message 测试（v2.2.1）
    // ========================================================================

    #[test]
    fn test_truncate_message_short() {
        let result = super::truncate_message("Hello", 10);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_message_exact() {
        let result = super::truncate_message("Hello", 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_message_long() {
        let result = super::truncate_message("Hello World!", 5);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn test_truncate_message_chinese() {
        let result = super::truncate_message("你好世界测试", 3);
        assert_eq!(result, "你好世...");
    }

    #[test]
    fn test_truncate_message_empty() {
        let result = super::truncate_message("", 10);
        assert_eq!(result, "");
    }
}
