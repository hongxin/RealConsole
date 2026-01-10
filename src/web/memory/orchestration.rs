//! Memory 2.0 WebUI 编排层
//!
//! v1.54.0: 上下文优化与 Token 管理
//!
//! ## 职责
//!
//! 1. Token 预算管理：严格控制不超出限制
//! 2. 贪心选择：按综合得分选择最优片段
//! 3. 富媒体编排：文本+图表+数据的最优组合
//! 4. 推荐生成：跨会话智能推荐
//! 5. 去重优化：避免重复内容

use super::types::{
    ContentType, MemoryAction, MemoryStateVector, MultimodalChunk, MultimodalContent, TimeContext,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// 选择策略（v1.55.0）
// ============================================================================

/// 选择策略
///
/// v1.55.0: 支持多种片段选择策略
#[derive(Debug, Clone)]
pub enum SelectionStrategy {
    /// 快速：Top-K 最高分
    /// 适用：简单查询、实时响应
    TopK { k: usize },

    /// 时间优先：最近的内容
    /// 适用：日常对话、上下文连续
    Recency { decay_factor: f64 },

    /// 贪心：分数+Token优化
    /// 适用：复杂分析、深度查询
    Greedy { budget: usize },

    /// 混合：多策略融合
    /// 适用：不确定场景
    #[allow(dead_code)]
    Hybrid {
        strategies: Vec<(Box<SelectionStrategy>, f64)>,  // (策略, 权重)
    },
}

/// 自动选择策略
///
/// 根据任务复杂度自动选择合适的策略
pub fn auto_select_strategy(complexity: super::understanding::TaskComplexity) -> SelectionStrategy {
    use super::understanding::TaskComplexity;

    match complexity {
        TaskComplexity::Simple => {
            SelectionStrategy::TopK { k: 10 }
        }
        TaskComplexity::Medium => {
            SelectionStrategy::Recency { decay_factor: 0.3 }
        }
        TaskComplexity::Complex => {
            SelectionStrategy::Greedy { budget: 8000 }
        }
    }
}

// ============================================================================
// 编排层
// ============================================================================

/// WebUI 编排层
///
/// Phase 1 实现：基础版（Token 预算 + 贪心选择）
/// Phase 3 升级：智能压缩 + 跨会话推荐
pub struct WebUIOrchestrationLayer {
    /// Token 计数器
    token_counter: TokenCounter,

    /// 决策引擎（64卦）
    decision_engine: DecisionEngine,
}

impl WebUIOrchestrationLayer {
    pub fn new() -> Self {
        Self {
            token_counter: TokenCounter::new(),
            decision_engine: DecisionEngine::new(),
        }
    }

    /// 构建优化的富媒体上下文（核心方法）
    ///
    /// # 参数
    /// - `chunks`: 已评分的片段列表
    /// - `token_budget`: Token 预算（如 2000）
    ///
    /// # 返回
    /// 优化后的富媒体上下文
    pub async fn build_optimized_context(
        &self,
        mut chunks: Vec<MultimodalChunk>,
        token_budget: usize,
    ) -> Result<OptimizedMultimodalContext> {
        eprintln!(
            "[Orchestration] Building context from {} chunks, budget: {} tokens",
            chunks.len(),
            token_budget
        );

        // 1. 按综合得分排序（高到低）
        chunks.sort_by(|a, b| {
            let score_a = a.state_vector.overall_score();
            let score_b = b.state_vector.overall_score();
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // 2. v1.55.0: 使用策略选择（默认 Greedy）
        let strategy = SelectionStrategy::Greedy { budget: token_budget };
        let selected_chunks = self.select_by_strategy(chunks, &strategy);

        // 3. 按内容类型分组
        let grouped = self.group_by_content_type(selected_chunks);

        // 4. 构建富媒体上下文
        let context = self.compose_context(grouped, token_budget);

        // 5. 生成推荐（基于选中的片段）
        let recommendations = self.generate_recommendations(&context.chunks);

        Ok(OptimizedMultimodalContext {
            text_chunks: context.text_chunks,
            chart_references: context.chart_references,
            image_references: context.image_references,
            data_summaries: context.data_summaries,
            recommendations,
            pre_filled_params: HashMap::new(), // TODO: Phase 3 实现
            total_tokens: context.total_tokens,
            metadata: ContextMetadata {
                chunk_count: context.chunks.len(),
                dimension_distribution: self.calculate_dimension_distribution(&context.chunks),
                avg_relevance: context.avg_score,
            },
            chunks: context.chunks,
            avg_score: context.avg_score,
        })
    }

    /// 贪心选择算法
    ///
    /// 按综合得分从高到低选择，直到达到 token 预算
    fn greedy_select(
        &self,
        chunks: Vec<MultimodalChunk>,
        token_budget: usize,
    ) -> Vec<MultimodalChunk> {
        let mut selected = Vec::new();
        let mut used_tokens = 0;

        for chunk in chunks {
            let chunk_tokens = self.token_counter.estimate_tokens(&chunk);

            // 检查是否超出预算
            if used_tokens + chunk_tokens > token_budget {
                continue;
            }

            // 检查得分阈值（太低的不选）
            let score = chunk.state_vector.overall_score();
            if score < 0.3 {
                break; // 已排序，后面的更低
            }

            selected.push(chunk);
            used_tokens += chunk_tokens;
        }

        eprintln!(
            "[Orchestration] Greedy selected {} chunks, using {} / {} tokens",
            selected.len(),
            used_tokens,
            token_budget
        );

        selected
    }

    /// Top-K 选择（v1.55.0）
    ///
    /// 选择得分最高的 K 个片段
    fn select_top_k(&self, mut chunks: Vec<MultimodalChunk>, k: usize) -> Vec<MultimodalChunk> {
        // 按得分排序
        chunks.sort_by(|a, b| {
            b.state_vector.overall_score()
                .partial_cmp(&a.state_vector.overall_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 取前 K 个
        chunks.truncate(k);

        eprintln!("[Orchestration] TopK selected {} chunks (K={})", chunks.len(), k);

        chunks
    }

    /// 时间优先选择（v1.55.0）
    ///
    /// 优先选择最近的内容，结合时间衰减
    fn select_recent(&self, mut chunks: Vec<MultimodalChunk>, decay_factor: f64) -> Vec<MultimodalChunk> {
        use chrono::Utc;

        // 计算时间加权得分
        let now = Utc::now();
        for chunk in &mut chunks {
            let age = now - chunk.timestamp;
            let age_hours = age.num_hours() as f64;

            // 时间衰减：最近的内容权重更高
            let time_weight = (-age_hours * decay_factor / 24.0).exp();

            // 综合得分 = 原始得分 × 时间权重
            let base_score = chunk.state_vector.overall_score();
            chunk.state_vector.understanding_score = base_score * time_weight;
        }

        // 按新得分排序
        chunks.sort_by(|a, b| {
            b.state_vector.understanding_score
                .partial_cmp(&a.state_vector.understanding_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 选择前 15 个（合理数量）
        chunks.truncate(15);

        eprintln!("[Orchestration] Recency selected {} chunks (decay={})", chunks.len(), decay_factor);

        chunks
    }

    /// 统一选择接口（v1.55.0）
    ///
    /// 根据策略选择片段
    fn select_by_strategy(
        &self,
        chunks: Vec<MultimodalChunk>,
        strategy: &SelectionStrategy,
    ) -> Vec<MultimodalChunk> {
        eprintln!("[Orchestration] Using strategy: {:?}", strategy);

        match strategy {
            SelectionStrategy::TopK { k } => self.select_top_k(chunks, *k),
            SelectionStrategy::Recency { decay_factor } => self.select_recent(chunks, *decay_factor),
            SelectionStrategy::Greedy { budget } => self.greedy_select(chunks, *budget),
            SelectionStrategy::Hybrid { .. } => {
                // Phase 1: 简化实现，暂时使用 Greedy
                eprintln!("[Orchestration] Hybrid strategy not yet implemented, falling back to Greedy");
                self.greedy_select(chunks, 4000)
            }
        }
    }

    /// 按内容类型分组
    fn group_by_content_type(
        &self,
        chunks: Vec<MultimodalChunk>,
    ) -> HashMap<ContentType, Vec<MultimodalChunk>> {
        let mut groups: HashMap<ContentType, Vec<MultimodalChunk>> = HashMap::new();

        for chunk in chunks {
            let content_type = chunk.content.content_type();
            groups.entry(content_type).or_default().push(chunk);
        }

        groups
    }

    /// 组合上下文
    fn compose_context(
        &self,
        grouped: HashMap<ContentType, Vec<MultimodalChunk>>,
        _token_budget: usize,
    ) -> ComposedContext {
        let mut text_chunks = Vec::new();
        let mut chart_references = Vec::new();
        let image_references = Vec::new();
        let mut data_summaries = Vec::new();
        let mut all_chunks = Vec::new();
        let mut total_tokens = 0;
        let mut total_score = 0.0;

        // 处理文本类型
        if let Some(text_group) = grouped.get(&ContentType::TextOnly) {
            for chunk in text_group {
                if let MultimodalContent::Text { content } = &chunk.content {
                    text_chunks.push(TextChunk {
                        id: chunk.id,
                        content: content.clone(),
                        dimension: chunk.dimension,
                        timestamp: chunk.timestamp,
                        relevance_score: chunk.state_vector.understanding_score,
                    });

                    total_tokens += self.token_counter.estimate_tokens(chunk);
                    total_score += chunk.state_vector.overall_score();
                }
                all_chunks.push(chunk.clone());
            }
        }

        // 处理图表类型
        if let Some(chart_group) = grouped.get(&ContentType::VisualOnly) {
            for chunk in chart_group {
                if let MultimodalContent::Chart { chart, description } = &chunk.content {
                    chart_references.push(ChartReference {
                        id: chunk.id,
                        thumbnail_text: description.clone(),
                        reuse_params: extract_chart_params(chart),
                        success_score: chunk.state_vector.overall_score(),
                    });

                    total_tokens += self.token_counter.estimate_tokens(chunk);
                    total_score += chunk.state_vector.overall_score();
                }
                all_chunks.push(chunk.clone());
            }
        }

        // 处理数据摘要
        if let Some(data_group) = grouped.get(&ContentType::DataOnly) {
            for chunk in data_group {
                if let MultimodalContent::DataSummary { filename, summary, .. } = &chunk.content {
                    data_summaries.push(DataSummary {
                        filename: filename.clone(),
                        summary: summary.clone(),
                    });

                    total_tokens += self.token_counter.estimate_tokens(chunk);
                    total_score += chunk.state_vector.overall_score();
                }
                all_chunks.push(chunk.clone());
            }
        }

        let avg_score = if all_chunks.is_empty() {
            0.0
        } else {
            total_score / all_chunks.len() as f64
        };

        ComposedContext {
            text_chunks,
            chart_references,
            image_references,
            data_summaries,
            chunks: all_chunks,
            total_tokens,
            avg_score,
        }
    }

    /// 生成推荐
    fn generate_recommendations(&self, chunks: &[MultimodalChunk]) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Phase 1: 简单推荐（基于高分片段）
        for chunk in chunks.iter().take(3) {
            // 取前 3 个高分片段
            if chunk.state_vector.overall_score() > 0.7 {
                let rec = match &chunk.content {
                    MultimodalContent::Chart { chart, .. } => Recommendation {
                        type_: RecommendationType::ChartTemplateReuse,
                        description: format!(
                            "上次您使用了 {:?} 类型的图表，是否继续使用？",
                            chart.chart_type
                        ),
                        confidence: chunk.state_vector.overall_score(),
                        actions: vec![],
                        pre_fill_params: extract_chart_params(chart),
                    },
                    MultimodalContent::SessionSummary { name, topic, .. } => Recommendation {
                        type_: RecommendationType::WorkflowReuse,
                        description: format!(
                            "会话 \"{}\" 可能与当前任务相关 (主题: {})",
                            name,
                            topic.as_deref().unwrap_or("未知")
                        ),
                        confidence: chunk.state_vector.overall_score(),
                        actions: vec![],
                        pre_fill_params: HashMap::new(),
                    },
                    _ => continue,
                };

                recommendations.push(rec);
            }
        }

        recommendations
    }

    /// 计算维度分布
    fn calculate_dimension_distribution(
        &self,
        chunks: &[MultimodalChunk],
    ) -> HashMap<String, usize> {
        let mut dist = HashMap::new();

        for chunk in chunks {
            let dim_name = chunk.dimension.name().to_string();
            *dist.entry(dim_name).or_insert(0) += 1;
        }

        dist
    }
}

impl Default for WebUIOrchestrationLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Token 计数器
// ============================================================================

/// Token 计数器
///
/// Phase 1: 简化估算（字数 / 0.75）
/// Phase 3: 可升级为 tiktoken-rs
pub struct TokenCounter;

impl TokenCounter {
    pub fn new() -> Self {
        Self
    }

    /// 估算 token 数量
    pub fn estimate_tokens(&self, chunk: &MultimodalChunk) -> usize {
        let text = chunk.content.to_text();

        // 简化估算：英文 ~0.75 token/word，中文 ~1.5 token/char
        let words = text.split_whitespace().count();
        let chars = text.chars().count();

        // 取较大值（保守估计）
        ((words as f64 / 0.75) as usize).max((chars as f64 * 1.5) as usize)
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 决策引擎（64卦）
// ============================================================================

/// 决策引擎
///
/// 基于易经64卦思想：8种内容 × 8种时间情境 = 64种决策
pub struct DecisionEngine {
    /// 规则权重（可自适应调整）
    rule_weights: HashMap<String, f64>,
}

impl DecisionEngine {
    pub fn new() -> Self {
        Self {
            rule_weights: HashMap::new(),
        }
    }

    /// 智能决策（64种组合之一）
    #[allow(dead_code)]
    pub fn decide(
        &self,
        content_type: ContentType,
        time_context: TimeContext,
    ) -> MemoryAction {
        // Phase 1: 简化实现，仅几种典型情况
        match (content_type, time_context) {
            (ContentType::TextOnly, TimeContext::CurrentDialog) => MemoryAction::HighPriority {
                action: "直接加入 LLM 上下文".to_string(),
                token_budget: 2000,
            },
            (ContentType::VisualOnly, TimeContext::CurrentSession) => {
                MemoryAction::MediumPriority {
                    action: "生成图表缩略图 + 文本摘要".to_string(),
                    token_budget: 500,
                }
            }
            (ContentType::TextAndVisual, TimeContext::RecentSession) => {
                MemoryAction::RecommendReuse {
                    suggestion: "上次的分析报告可以复用，是否继续？".to_string(),
                    template_id: None,
                }
            }
            _ => MemoryAction::FallbackToLLM,
        }
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 数据结构
// ============================================================================

/// 优化后的富媒体上下文
#[derive(Debug, Clone, Serialize)]
pub struct OptimizedMultimodalContext {
    /// 文本片段
    pub text_chunks: Vec<TextChunk>,

    /// 图表引用
    pub chart_references: Vec<ChartReference>,

    /// 图像引用
    pub image_references: Vec<ImageReference>,

    /// 数据摘要
    pub data_summaries: Vec<DataSummary>,

    /// 推荐
    pub recommendations: Vec<Recommendation>,

    /// 预填充参数
    pub pre_filled_params: HashMap<String, serde_json::Value>,

    /// 总 Token 数
    pub total_tokens: usize,

    /// 元信息
    pub metadata: ContextMetadata,

    // 内部使用（跳过序列化但公开访问）
    #[serde(skip)]
    pub chunks: Vec<MultimodalChunk>,

    #[serde(skip)]
    pub avg_score: f64,
}

/// 文本片段
#[derive(Debug, Serialize, Clone)]
pub struct TextChunk {
    pub id: Uuid,
    pub content: String,
    pub dimension: super::types::DataDimension,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub relevance_score: f64,
}

/// 图表引用
#[derive(Debug, Serialize, Clone)]
pub struct ChartReference {
    pub id: Uuid,
    pub thumbnail_text: String,
    pub reuse_params: HashMap<String, serde_json::Value>,
    pub success_score: f64,
}

/// 图像引用
#[derive(Debug, Serialize, Clone)]
pub struct ImageReference {
    pub id: Uuid,
    pub description: String,
}

/// 数据摘要
#[derive(Debug, Serialize, Clone)]
pub struct DataSummary {
    pub filename: String,
    pub summary: String,
}

/// 推荐
#[derive(Debug, Serialize, Clone)]
pub struct Recommendation {
    pub type_: RecommendationType,
    pub description: String,
    pub confidence: f64,
    pub actions: Vec<String>,
    pub pre_fill_params: HashMap<String, serde_json::Value>,
}

/// 推荐类型
#[derive(Debug, Serialize, Clone)]
pub enum RecommendationType {
    WorkflowReuse,
    ChartTemplateReuse,
    DataAnalysisPattern,
    InteractiveSuggestion,
}

/// 上下文元信息
#[derive(Debug, Clone, Serialize)]
pub struct ContextMetadata {
    pub chunk_count: usize,
    pub dimension_distribution: HashMap<String, usize>,
    pub avg_relevance: f64,
}

/// 组合上下文（内部使用）
struct ComposedContext {
    text_chunks: Vec<TextChunk>,
    chart_references: Vec<ChartReference>,
    image_references: Vec<ImageReference>,
    data_summaries: Vec<DataSummary>,
    chunks: Vec<MultimodalChunk>,
    total_tokens: usize,
    avg_score: f64,
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 提取图表参数
fn extract_chart_params(
    chart: &crate::visualization::ChartData,
) -> HashMap<String, serde_json::Value> {
    let mut params = HashMap::new();

    params.insert(
        "chart_type".to_string(),
        serde_json::json!(format!("{:?}", chart.chart_type)),
    );

    params.insert("title".to_string(), serde_json::json!(&chart.title));

    // TODO: 提取更多参数（颜色、坐标轴范围等）

    params
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
    // TokenCounter 测试
    // ========================================================================

    #[test]
    fn test_token_estimation() {
        let counter = TokenCounter::new();

        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "This is a test message with ten words total".to_string(),
            },
            Utc::now(),
        );

        let tokens = counter.estimate_tokens(&chunk);
        assert!(tokens > 0);
        assert!(tokens < 100); // 应该远小于100
    }

    #[test]
    fn test_token_estimation_chinese() {
        let counter = TokenCounter::new();

        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "这是一段中文测试内容，用于验证Token估算".to_string(),
            },
            Utc::now(),
        );

        let tokens = counter.estimate_tokens(&chunk);
        assert!(tokens > 0);
        // 中文字符每个 ~1.5 tokens
        assert!(tokens > 20);
    }

    #[test]
    fn test_token_counter_default() {
        let counter = TokenCounter::default();
        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "test".to_string(),
            },
            Utc::now(),
        );
        assert!(counter.estimate_tokens(&chunk) > 0);
    }

    // ========================================================================
    // SelectionStrategy 测试
    // ========================================================================

    #[test]
    fn test_auto_select_strategy_simple() {
        use crate::web::memory::understanding::TaskComplexity;

        let strategy = auto_select_strategy(TaskComplexity::Simple);
        match strategy {
            SelectionStrategy::TopK { k } => assert_eq!(k, 10),
            _ => panic!("Expected TopK strategy for Simple complexity"),
        }
    }

    #[test]
    fn test_auto_select_strategy_medium() {
        use crate::web::memory::understanding::TaskComplexity;

        let strategy = auto_select_strategy(TaskComplexity::Medium);
        match strategy {
            SelectionStrategy::Recency { decay_factor } => {
                assert!((decay_factor - 0.3).abs() < 0.01);
            }
            _ => panic!("Expected Recency strategy for Medium complexity"),
        }
    }

    #[test]
    fn test_auto_select_strategy_complex() {
        use crate::web::memory::understanding::TaskComplexity;

        let strategy = auto_select_strategy(TaskComplexity::Complex);
        match strategy {
            SelectionStrategy::Greedy { budget } => assert_eq!(budget, 8000),
            _ => panic!("Expected Greedy strategy for Complex complexity"),
        }
    }

    // ========================================================================
    // WebUIOrchestrationLayer 测试
    // ========================================================================

    #[test]
    fn test_greedy_select() {
        let orchestration = WebUIOrchestrationLayer::new();

        // 创建一些测试片段
        let chunks = vec![
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: "High score chunk".to_string(),
                    },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = 0.9;
                chunk
            },
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: "Low score chunk".to_string(),
                    },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = 0.2;
                chunk
            },
        ];

        let selected = orchestration.greedy_select(chunks, 1000);

        // 应该优先选择高分片段
        assert!(!selected.is_empty());
        assert!(selected[0].state_vector.understanding_score > 0.5);
    }

    #[test]
    fn test_greedy_select_budget_limit() {
        let orchestration = WebUIOrchestrationLayer::new();

        // 创建大量片段
        let chunks: Vec<MultimodalChunk> = (0..100)
            .map(|i| {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: format!("Chunk {} with some content to take up tokens", i),
                    },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = 0.8;
                chunk
            })
            .collect();

        // 小 token 预算应该限制选择数量
        let selected = orchestration.greedy_select(chunks, 500);
        assert!(selected.len() < 100);
    }

    #[test]
    fn test_select_top_k() {
        let orchestration = WebUIOrchestrationLayer::new();

        let chunks: Vec<MultimodalChunk> = (0..10)
            .map(|i| {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: format!("Chunk {}", i),
                    },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = i as f64 / 10.0;
                chunk
            })
            .collect();

        let selected = orchestration.select_top_k(chunks, 3);
        assert_eq!(selected.len(), 3);
        // 应该选择最高分的3个
        assert!(selected[0].state_vector.understanding_score > 0.7);
    }

    #[test]
    fn test_select_recent() {
        let orchestration = WebUIOrchestrationLayer::new();
        use chrono::Duration;

        // 创建不同年龄的片段
        let now = Utc::now();
        let chunks = vec![
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: "Old chunk".to_string(),
                    },
                    now - Duration::days(10),
                );
                chunk.state_vector.understanding_score = 0.9;
                chunk
            },
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text {
                        content: "New chunk".to_string(),
                    },
                    now - Duration::hours(1),
                );
                chunk.state_vector.understanding_score = 0.5;
                chunk
            },
        ];

        let selected = orchestration.select_recent(chunks, 0.3);
        assert!(!selected.is_empty());
        // 新片段应该排在前面（时间权重）
    }

    #[test]
    fn test_select_by_strategy_greedy() {
        let orchestration = WebUIOrchestrationLayer::new();

        let chunks = vec![MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "Test".to_string(),
            },
            Utc::now(),
        )];

        let strategy = SelectionStrategy::Greedy { budget: 1000 };
        let selected = orchestration.select_by_strategy(chunks, &strategy);
        assert!(!selected.is_empty());
    }

    #[test]
    fn test_select_by_strategy_hybrid_fallback() {
        let orchestration = WebUIOrchestrationLayer::new();

        let chunks = vec![{
            let mut chunk = MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text {
                    content: "Test".to_string(),
                },
                Utc::now(),
            );
            chunk.state_vector.understanding_score = 0.8;
            chunk
        }];

        // Hybrid 策略应该回退到 Greedy
        let strategy = SelectionStrategy::Hybrid {
            strategies: vec![(Box::new(SelectionStrategy::TopK { k: 5 }), 0.5)],
        };
        let selected = orchestration.select_by_strategy(chunks, &strategy);
        assert!(!selected.is_empty());
    }

    #[test]
    fn test_group_by_content_type() {
        let orchestration = WebUIOrchestrationLayer::new();

        let chunks = vec![
            MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text {
                    content: "Text content".to_string(),
                },
                Utc::now(),
            ),
            MultimodalChunk::new(
                DataDimension::ChartHistory,
                MultimodalContent::DataSummary {
                    filename: "data.csv".to_string(),
                    rows: 100,
                    columns: 5,
                    headers: vec!["col1".to_string(), "col2".to_string()],
                    summary: "Test data".to_string(),
                },
                Utc::now(),
            ),
        ];

        let grouped = orchestration.group_by_content_type(chunks);
        assert!(grouped.contains_key(&ContentType::TextOnly));
        assert!(grouped.contains_key(&ContentType::DataOnly));
    }

    #[test]
    fn test_orchestration_default() {
        let orchestration = WebUIOrchestrationLayer::default();
        assert!(orchestration.token_counter.estimate_tokens(&MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "test".to_string(),
            },
            Utc::now(),
        )) > 0);
    }

    // ========================================================================
    // DecisionEngine 测试
    // ========================================================================

    #[test]
    fn test_decision_engine_new() {
        let engine = DecisionEngine::new();
        assert!(engine.rule_weights.is_empty());
    }

    #[test]
    fn test_decision_engine_default() {
        let engine = DecisionEngine::default();
        assert!(engine.rule_weights.is_empty());
    }

    #[test]
    fn test_decision_engine_decide_text_current() {
        let engine = DecisionEngine::new();
        let action = engine.decide(ContentType::TextOnly, TimeContext::CurrentDialog);

        match action {
            MemoryAction::HighPriority { token_budget, .. } => {
                assert_eq!(token_budget, 2000);
            }
            _ => panic!("Expected HighPriority action"),
        }
    }

    #[test]
    fn test_decision_engine_decide_visual_session() {
        let engine = DecisionEngine::new();
        let action = engine.decide(ContentType::VisualOnly, TimeContext::CurrentSession);

        match action {
            MemoryAction::MediumPriority { token_budget, .. } => {
                assert_eq!(token_budget, 500);
            }
            _ => panic!("Expected MediumPriority action"),
        }
    }

    #[test]
    fn test_decision_engine_decide_mixed_recent() {
        let engine = DecisionEngine::new();
        let action = engine.decide(ContentType::TextAndVisual, TimeContext::RecentSession);

        match action {
            MemoryAction::RecommendReuse { template_id, .. } => {
                assert!(template_id.is_none());
            }
            _ => panic!("Expected RecommendReuse action"),
        }
    }

    #[test]
    fn test_decision_engine_decide_fallback() {
        let engine = DecisionEngine::new();
        let action = engine.decide(ContentType::DataOnly, TimeContext::LongTermSession);

        match action {
            MemoryAction::FallbackToLLM => {}
            _ => panic!("Expected FallbackToLLM action"),
        }
    }

    // ========================================================================
    // 数据结构测试
    // ========================================================================

    #[test]
    fn test_recommendation_type_debug() {
        let rec_type = RecommendationType::ChartTemplateReuse;
        let debug_str = format!("{:?}", rec_type);
        assert!(debug_str.contains("ChartTemplateReuse"));
    }

    #[test]
    fn test_context_metadata_serialize() {
        let metadata = ContextMetadata {
            chunk_count: 5,
            dimension_distribution: HashMap::new(),
            avg_relevance: 0.75,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("chunk_count"));
        assert!(json.contains("5"));
    }
}
