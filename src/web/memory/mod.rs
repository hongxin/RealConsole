//! Memory 2.0 WebUI: 智能上下文编排器
//!
//! v1.54.0: 基于"一分为三"哲学的富媒体智能记忆系统
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
//! ```
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
//! ```rust
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

// 重新导出常用类型
pub use types::{
    MultimodalChunk, MemoryStateVector, MultimodalContent,
    ContentType, TimeContext, MemoryAction, DataDimension,
};
pub use orchestration::{
    TextChunk, ChartReference, ImageReference, DataSummary,
    RecommendationType,
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
        })
    }

    /// 核心方法：为当前任务提取相关上下文
    ///
    /// # 参数
    /// - `task`: 当前任务描述
    /// - `time_range`: 时间范围（None = 全部）
    /// - `token_budget`: Token 预算（如 2000）
    ///
    /// # 返回
    /// 优化后的富媒体上下文
    ///
    /// # 示例
    ///
    /// ```rust
    /// let context = orchestrator
    ///     .extract_relevant_context("调试 Rust trait 问题", None, 2000)
    ///     .await?;
    ///
    /// println!("找到 {} 个相关片段", context.text_chunks.len());
    /// println!("推荐: {:?}", context.recommendations);
    /// ```
    pub async fn extract_relevant_context(
        &self,
        task: &str,
        time_range: Option<TimeRange>,
        token_budget: usize,
    ) -> Result<OptimizedMultimodalContext> {
        eprintln!("\n[Memory 2.0] Extracting context for: \"{}\"", task);
        eprintln!("[Memory 2.0] Token budget: {}", token_budget);

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
        let scored_chunks = self.understanding
            .score_relevance(task, chunks)
            .await?;

        // 3. 编排层：优化组合
        let context = self.orchestration
            .build_optimized_context(scored_chunks, token_budget)
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
    /// Phase 1: 基础实现（基于相似度）
    /// Phase 3: 升级为复杂的模式识别
    pub async fn recommend_from_sessions(
        &self,
        _current_task: Option<&str>,
    ) -> Result<Vec<Recommendation>> {
        // Phase 1: 简化实现
        // TODO: Phase 3 实现完整的跨会话推荐

        Ok(vec![])
    }

    /// 快速查询：仅返回最相关的前 N 个片段
    ///
    /// 用于快速预览，不进行完整的编排
    pub async fn quick_search(
        &self,
        task: &str,
        top_k: usize,
    ) -> Result<Vec<MultimodalChunk>> {
        // 1. 采集数据（最近7天）
        let chunks = self.perception
            .collect_multimodal_data(Some(TimeRange::recent_days(7)), None)
            .await?;

        // 2. 评分
        let mut scored_chunks = self.understanding
            .score_relevance(task, chunks)
            .await?;

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
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // 集成测试需要实际的依赖
    // 在实际环境中运行

    #[test]
    fn test_module_compiles() {
        // 确保模块可以编译
        assert!(true);
    }
}
