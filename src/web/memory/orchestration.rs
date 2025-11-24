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

        // 2. 贪心选择：优先选择高分片段
        let selected_chunks = self.greedy_select(chunks, token_budget);

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
            "[Orchestration] Selected {} chunks, using {} / {} tokens",
            selected.len(),
            used_tokens,
            token_budget
        );

        selected
    }

    /// 按内容类型分组
    fn group_by_content_type(
        &self,
        chunks: Vec<MultimodalChunk>,
    ) -> HashMap<ContentType, Vec<MultimodalChunk>> {
        let mut groups: HashMap<ContentType, Vec<MultimodalChunk>> = HashMap::new();

        for chunk in chunks {
            let content_type = chunk.content.content_type();
            groups.entry(content_type).or_insert_with(Vec::new).push(chunk);
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
#[derive(Debug, Serialize)]
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
#[derive(Debug, Serialize)]
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
        assert!(selected.len() > 0);
        assert!(selected[0].state_vector.understanding_score > 0.5);
    }
}
