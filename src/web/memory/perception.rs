//! Memory 2.0 WebUI 感知层
//!
//! v1.54.0: 从 CLI 4维 + WebUI 3维采集多模态数据
//!
//! ## 职责
//!
//! 1. 从 7 个维度采集原始数据
//! 2. 统一数据模型（→ MultimodalChunk）
//! 3. 基础过滤（时间范围、类型筛选）
//! 4. 向量化（→ MemoryStateVector）

use super::types::{
    DataDimension, MemoryStateVector, MultimodalChunk, MultimodalContent, TimeContext,
};
use crate::web::session::{ChartHistoryEntry, ConversationRound, ImageHistoryEntry, SessionId};
use crate::web::session_manager::{SerializableSession, SessionManager};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// 时间范围过滤
// ============================================================================

/// 时间范围
#[derive(Debug, Clone)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl TimeRange {
    /// 最近 N 天
    pub fn recent_days(days: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::days(days);
        Self { start, end }
    }

    /// 最近 N 小时
    pub fn recent_hours(hours: i64) -> Self {
        let end = Utc::now();
        let start = end - Duration::hours(hours);
        Self { start, end }
    }

    /// 包含指定时间戳
    pub fn contains(&self, timestamp: &DateTime<Utc>) -> bool {
        timestamp >= &self.start && timestamp <= &self.end
    }
}

// ============================================================================
// WebUI 感知层
// ============================================================================

/// WebUI 感知层
///
/// 从 7 个维度采集多模态数据：
/// - CLI 4维：History/ExecutionLogger/LlmLogger/Context（待集成）
/// - WebUI 3维：SessionManager/ChartHistory/ImageHistory
pub struct WebUIPerceptionLayer {
    // ===== WebUI 数据源 =====
    /// 会话管理器
    session_manager: Arc<SessionManager>,

    /// 图表历史（共享状态）
    chart_history: Arc<RwLock<Vec<ChartHistoryEntry>>>,

    /// 图像历史（共享状态）
    image_history: Arc<RwLock<Vec<ImageHistoryEntry>>>,

    // TODO: 后续集成 CLI 四维
    // unified_tracer: Arc<UnifiedTracer>,
    // context_tracker: Arc<RwLock<ContextTracker>>,
}

impl WebUIPerceptionLayer {
    /// 创建感知层
    pub fn new(
        session_manager: Arc<SessionManager>,
        chart_history: Arc<RwLock<Vec<ChartHistoryEntry>>>,
        image_history: Arc<RwLock<Vec<ImageHistoryEntry>>>,
    ) -> Self {
        Self {
            session_manager,
            chart_history,
            image_history,
        }
    }

    /// 采集多模态数据（核心方法）
    ///
    /// # 参数
    /// - `time_range`: 时间范围过滤（None = 全部）
    /// - `dimensions`: 指定采集的维度（None = 全部）
    ///
    /// # 返回
    /// 多模态上下文片段列表
    pub async fn collect_multimodal_data(
        &self,
        time_range: Option<TimeRange>,
        dimensions: Option<Vec<DataDimension>>,
    ) -> Result<Vec<MultimodalChunk>> {
        let mut chunks = Vec::new();

        // 默认采集所有 WebUI 维度
        let target_dimensions = dimensions.unwrap_or_else(|| {
            vec![
                DataDimension::SessionManager,
                DataDimension::ChartHistory,
                DataDimension::ImageHistory,
            ]
        });

        for dimension in &target_dimensions {
            match dimension {
                DataDimension::SessionManager => {
                    let session_chunks = self.collect_session_data(time_range.as_ref()).await?;
                    chunks.extend(session_chunks);
                }
                DataDimension::ChartHistory => {
                    let chart_chunks = self.collect_chart_data(time_range.as_ref()).await?;
                    chunks.extend(chart_chunks);
                }
                DataDimension::ImageHistory => {
                    let image_chunks = self.collect_image_data(time_range.as_ref()).await?;
                    chunks.extend(image_chunks);
                }
                // CLI 维度待实现
                _ => {
                    eprintln!(
                        "[Perception] Dimension {:?} not yet implemented",
                        dimension
                    );
                }
            }
        }

        // 按时间倒序排列（最新的在前）
        chunks.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        eprintln!(
            "[Perception] Collected {} chunks from {} dimensions",
            chunks.len(),
            target_dimensions.len()
        );

        Ok(chunks)
    }

    /// 采集会话数据
    async fn collect_session_data(
        &self,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<MultimodalChunk>> {
        let sessions = self.session_manager.list_sessions()?;

        let chunks: Vec<MultimodalChunk> = sessions
            .iter()
            .filter(|s| {
                if let Some(range) = time_range {
                    range.contains(&s.updated_at)
                } else {
                    true
                }
            })
            .map(|session_item| {
                // 为每个会话创建一个摘要 chunk
                let content = MultimodalContent::SessionSummary {
                    session_id: session_item.id.clone(),
                    name: session_item.name.clone(),
                    round_count: session_item.round_count,
                    topic: None, // TODO: 后续实现主题提取
                    last_message: session_item.last_message.clone(),
                };

                let mut chunk = MultimodalChunk::new(
                    DataDimension::SessionManager,
                    content,
                    session_item.updated_at,
                );

                // 向量化
                chunk.state_vector = self.vectorize_session_summary(session_item);

                chunk
            })
            .collect();

        Ok(chunks)
    }

    /// 采集图表数据
    async fn collect_chart_data(
        &self,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<MultimodalChunk>> {
        let history = self.chart_history.read().await;

        let chunks: Vec<MultimodalChunk> = history
            .iter()
            .filter(|entry| {
                if let Some(range) = time_range {
                    range.contains(&entry.timestamp)
                } else {
                    true
                }
            })
            .map(|entry| {
                let mut chunk = MultimodalChunk::from_chart_history(entry);

                // 向量化
                chunk.state_vector = self.vectorize_chart(entry);

                chunk
            })
            .collect();

        Ok(chunks)
    }

    /// 采集图像数据
    async fn collect_image_data(
        &self,
        time_range: Option<&TimeRange>,
    ) -> Result<Vec<MultimodalChunk>> {
        let history = self.image_history.read().await;

        let chunks: Vec<MultimodalChunk> = history
            .iter()
            .filter(|entry| {
                if let Some(range) = time_range {
                    range.contains(&entry.timestamp)
                } else {
                    true
                }
            })
            .map(|entry| {
                let mut chunk = MultimodalChunk::from_image_history(entry);

                // 向量化
                chunk.state_vector = self.vectorize_image(entry);

                chunk
            })
            .collect();

        Ok(chunks)
    }

    // ========================================================================
    // 向量化方法（将数据映射到 9 维向量空间）
    // ========================================================================

    /// 向量化会话摘要
    fn vectorize_session_summary(
        &self,
        session: &crate::web::session_manager::SessionListItem,
    ) -> MemoryStateVector {
        let age_seconds = (Utc::now() - session.updated_at).num_seconds();

        let mut vector = MemoryStateVector {
            text_weight: 0.8,
            visual_weight: 0.1,
            data_weight: 0.1,
            ..Default::default()
        };
        // 内容维度：会话主要是文本
        vector.normalize_content_weights();

        // 时间维度：根据会话年龄
        let time_context = TimeContext::from_age(age_seconds);
        match time_context {
            TimeContext::CurrentDialog | TimeContext::CurrentSession => {
                vector.working_relevance = 0.8;
                vector.session_relevance = 1.0;
                vector.long_term_value = 0.3;
            }
            TimeContext::RecentSession => {
                vector.working_relevance = 0.3;
                vector.session_relevance = 0.5;
                vector.long_term_value = 0.5;
            }
            _ => {
                vector.working_relevance = 0.0;
                vector.session_relevance = 0.2;
                vector.long_term_value = 0.7;
            }
        }

        // 智能维度：感知层默认满分
        vector.perception_score = 1.0;
        vector.understanding_score = 0.0; // 待理解层填充
        vector.orchestration_score = 0.0; // 待编排层填充

        vector
    }

    /// 向量化图表
    fn vectorize_chart(&self, entry: &ChartHistoryEntry) -> MemoryStateVector {
        let age_seconds = (Utc::now() - entry.timestamp).num_seconds();

        let mut vector = MemoryStateVector {
            text_weight: 0.1,
            visual_weight: 0.8,
            data_weight: 0.1,
            ..Default::default()
        };
        // 内容维度：图表主要是可视化
        vector.normalize_content_weights();

        // 时间维度
        let time_context = TimeContext::from_age(age_seconds);
        match time_context {
            TimeContext::CurrentDialog | TimeContext::CurrentSession => {
                vector.working_relevance = 0.9;
                vector.session_relevance = 1.0;
                vector.long_term_value = 0.4;
            }
            TimeContext::RecentSession => {
                vector.working_relevance = 0.5;
                vector.session_relevance = 0.6;
                vector.long_term_value = 0.6;
            }
            _ => {
                vector.working_relevance = 0.1;
                vector.session_relevance = 0.3;
                vector.long_term_value = 0.8;
            }
        }

        // 智能维度
        vector.perception_score = 1.0;

        vector
    }

    /// 向量化图像
    fn vectorize_image(&self, entry: &ImageHistoryEntry) -> MemoryStateVector {
        let age_seconds = (Utc::now() - entry.timestamp).num_seconds();

        let mut vector = MemoryStateVector {
            text_weight: 0.05,
            visual_weight: 0.9,
            data_weight: 0.05,
            ..Default::default()
        };
        // 内容维度：图像主要是可视化
        vector.normalize_content_weights();

        // 时间维度
        let time_context = TimeContext::from_age(age_seconds);
        match time_context {
            TimeContext::CurrentDialog | TimeContext::CurrentSession => {
                vector.working_relevance = 0.9;
                vector.session_relevance = 1.0;
                vector.long_term_value = 0.3;
            }
            TimeContext::RecentSession => {
                vector.working_relevance = 0.4;
                vector.session_relevance = 0.5;
                vector.long_term_value = 0.5;
            }
            _ => {
                vector.working_relevance = 0.0;
                vector.session_relevance = 0.2;
                vector.long_term_value = 0.7;
            }
        }

        // 智能维度
        vector.perception_score = 1.0;

        vector
    }

    /// 批量向量化
    pub fn batch_vectorize(&self, chunks: &mut [MultimodalChunk]) {
        for _chunk in chunks.iter_mut() {
            // 根据维度和内容类型重新计算向量
            // （已在采集时完成，此处为扩展接口）
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range_recent_days() {
        let now = Utc::now();
        let range = TimeRange::recent_days(7);

        // 使用稍早的时间避免边界问题
        let recent = now - Duration::hours(1);
        assert!(range.contains(&recent));

        let old = now - Duration::days(10);
        assert!(!range.contains(&old));
    }

    #[test]
    fn test_time_range_recent_hours() {
        let now = Utc::now();
        let range = TimeRange::recent_hours(24);

        // 使用稍早的时间避免边界问题
        let recent_1h = now - Duration::hours(1);
        assert!(range.contains(&recent_1h));

        let recent_12h = now - Duration::hours(12);
        assert!(range.contains(&recent_12h));

        let old = now - Duration::hours(30);
        assert!(!range.contains(&old));
    }

    // 集成测试需要实际的 SessionManager 等依赖
    // 在实际环境中运行
}
