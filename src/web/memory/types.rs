//! Memory 2.0 WebUI 核心数据类型
//!
//! v1.54.0: 智能上下文编排器 - 基于"一分为三"的9维向量空间设计
//!
//! ## 设计哲学
//!
//! **9维向量空间** = 3 × 3 × 3
//! - 内容维度（Content）: 文本 / 可视化 / 数据
//! - 时间维度（Temporal）: 短期 / 中期 / 长期
//! - 智能维度（Intelligence）: 感知 / 理解 / 编排
//!
//! **状态演化**：不是跳转，而是向量空间中的渐进路径

use crate::visualization::{ChartData, ImageData};
use crate::web::session::{ChartHistoryEntry, ImageHistoryEntry, ConversationRound, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// 核心状态向量
// ============================================================================

/// 记忆状态向量（9维）
///
/// 这不是3个固定状态，而是9维向量空间中的一个点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStateVector {
    // ===== 维度一：内容维度（3维）=====
    /// 文本内容的权重 [0.0, 1.0]
    pub text_weight: f64,

    /// 可视化内容的权重 [0.0, 1.0]
    pub visual_weight: f64,

    /// 数据内容的权重 [0.0, 1.0]
    pub data_weight: f64,

    // ===== 维度二：时间维度（3维）=====
    /// 与当前对话的相关性（工作记忆：9轮以内）[0.0, 1.0]
    pub working_relevance: f64,

    /// 与当前会话的相关性（会话记忆：本次会话）[0.0, 1.0]
    pub session_relevance: f64,

    /// 长期价值（跨会话记忆）[0.0, 1.0]
    pub long_term_value: f64,

    // ===== 维度三：智能维度（3维）=====
    /// 感知层评分（数据采集质量）[0.0, 1.0]
    pub perception_score: f64,

    /// 理解层评分（模式识别准确性）[0.0, 1.0]
    pub understanding_score: f64,

    /// 编排层评分（推荐效果）[0.0, 1.0]
    pub orchestration_score: f64,
}

impl Default for MemoryStateVector {
    fn default() -> Self {
        Self {
            // 内容维度：默认均衡
            text_weight: 0.33,
            visual_weight: 0.33,
            data_weight: 0.34,

            // 时间维度：默认关注当前
            working_relevance: 1.0,
            session_relevance: 0.5,
            long_term_value: 0.1,

            // 智能维度：感知默认满分，其他待计算
            perception_score: 1.0,
            understanding_score: 0.0,
            orchestration_score: 0.0,
        }
    }
}

impl MemoryStateVector {
    /// 线性插值（Linear Interpolation）
    ///
    /// 用于状态演化：从当前状态向目标状态平滑过渡
    ///
    /// # 参数
    /// - `target`: 目标状态
    /// - `t`: 插值参数 [0.0, 1.0]，0.0 = 当前状态，1.0 = 目标状态
    pub fn lerp(&self, target: &Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp_value = |a: f64, b: f64| a + (b - a) * t;

        Self {
            // 内容维度
            text_weight: lerp_value(self.text_weight, target.text_weight),
            visual_weight: lerp_value(self.visual_weight, target.visual_weight),
            data_weight: lerp_value(self.data_weight, target.data_weight),

            // 时间维度
            working_relevance: lerp_value(self.working_relevance, target.working_relevance),
            session_relevance: lerp_value(self.session_relevance, target.session_relevance),
            long_term_value: lerp_value(self.long_term_value, target.long_term_value),

            // 智能维度
            perception_score: lerp_value(self.perception_score, target.perception_score),
            understanding_score: lerp_value(self.understanding_score, target.understanding_score),
            orchestration_score: lerp_value(self.orchestration_score, target.orchestration_score),
        }
    }

    /// 计算综合得分
    ///
    /// 用于排序和优先级决策
    pub fn overall_score(&self) -> f64 {
        // 加权平均：内容 40% + 时间 30% + 智能 30%
        let content_score = (self.text_weight + self.visual_weight + self.data_weight) / 3.0;
        let temporal_score = (self.working_relevance + self.session_relevance + self.long_term_value) / 3.0;
        let intelligence_score = (self.perception_score + self.understanding_score + self.orchestration_score) / 3.0;

        content_score * 0.4 + temporal_score * 0.3 + intelligence_score * 0.3
    }

    /// 归一化内容权重（确保三者之和为1.0）
    pub fn normalize_content_weights(&mut self) {
        let sum = self.text_weight + self.visual_weight + self.data_weight;
        if sum > 0.0 {
            self.text_weight /= sum;
            self.visual_weight /= sum;
            self.data_weight /= sum;
        }
    }
}

// ============================================================================
// 数据维度枚举
// ============================================================================

/// 数据来源维度
///
/// CLI 4维 + WebUI 3维 = 7维感知能力
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataDimension {
    // ===== CLI 四维（复用现有系统）=====
    /// 统计维度：命令使用习惯
    History,

    /// 协同维度：端到端执行追踪
    ExecutionLogger,

    /// 黑盒维度：LLM API 详情
    LlmLogger,

    /// 记忆维度：对话工作记忆（9轮）
    Context,

    // ===== WebUI 三维（新增）=====
    /// 会话维度：跨会话记忆
    SessionManager,

    /// 可视化维度：图表历史
    ChartHistory,

    /// 可视化维度：图像历史
    ImageHistory,

    /// 数据维度：上传的文件
    UploadedFiles,
}

impl DataDimension {
    /// 获取维度的可读名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::History => "History",
            Self::ExecutionLogger => "ExecutionLogger",
            Self::LlmLogger => "LlmLogger",
            Self::Context => "Context",
            Self::SessionManager => "SessionManager",
            Self::ChartHistory => "ChartHistory",
            Self::ImageHistory => "ImageHistory",
            Self::UploadedFiles => "UploadedFiles",
        }
    }

    /// 是否是 WebUI 特有维度
    pub fn is_webui_specific(&self) -> bool {
        matches!(
            self,
            Self::SessionManager | Self::ChartHistory | Self::ImageHistory | Self::UploadedFiles
        )
    }
}

// ============================================================================
// 多模态内容
// ============================================================================

/// 多模态内容
///
/// 统一表示文本、图表、图像、数据等不同形式的内容
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MultimodalContent {
    /// 纯文本内容
    Text { content: String },

    /// 图表内容
    Chart {
        chart: ChartData,
        /// 文本描述（用于 LLM 理解）
        description: String,
    },

    /// 图像内容
    Image {
        image: ImageData,
        /// 文本描述
        description: String,
    },

    /// 数据摘要
    DataSummary {
        /// 文件名
        filename: String,
        /// 行数
        rows: usize,
        /// 列数
        columns: usize,
        /// 列名
        headers: Vec<String>,
        /// 数据特征摘要
        summary: String,
    },

    /// 会话摘要
    SessionSummary {
        session_id: SessionId,
        name: String,
        round_count: usize,
        /// 会话主题（自动提取）
        topic: Option<String>,
        /// 最后一条消息
        last_message: String,
    },

    /// 复合内容（文本+图表+数据）
    Composite {
        text: Option<String>,
        chart: Option<ChartData>,
        data_summary: Option<String>,
    },
}

impl MultimodalContent {
    /// 提取文本表示（用于关键词匹配等）
    pub fn to_text(&self) -> String {
        match self {
            Self::Text { content } => content.clone(),
            Self::Chart { description, .. } => description.clone(),
            Self::Image { description, .. } => description.clone(),
            Self::DataSummary { filename, summary, .. } => {
                format!("文件: {}\n{}", filename, summary)
            }
            Self::SessionSummary { name, topic, last_message, .. } => {
                format!(
                    "会话: {}\n主题: {}\n最后消息: {}",
                    name,
                    topic.as_deref().unwrap_or("未知"),
                    last_message
                )
            }
            Self::Composite { text, data_summary, .. } => {
                let mut parts = Vec::new();
                if let Some(t) = text {
                    parts.push(t.clone());
                }
                if let Some(d) = data_summary {
                    parts.push(d.clone());
                }
                parts.join("\n")
            }
        }
    }

    /// 内容类型（用于64卦决策）
    pub fn content_type(&self) -> ContentType {
        match self {
            Self::Text { .. } => ContentType::TextOnly,
            Self::Chart { .. } => ContentType::VisualOnly,
            Self::Image { .. } => ContentType::VisualOnly,
            Self::DataSummary { .. } => ContentType::DataOnly,
            Self::SessionSummary { .. } => ContentType::TextOnly,
            Self::Composite { text, chart, data_summary } => {
                let has_text = text.is_some();
                let has_visual = chart.is_some();
                let has_data = data_summary.is_some();

                match (has_text, has_visual, has_data) {
                    (true, true, true) => ContentType::FullMix,
                    (true, true, false) => ContentType::TextAndVisual,
                    (true, false, true) => ContentType::TextAndData,
                    (false, true, true) => ContentType::VisualAndData,
                    _ => ContentType::TextOnly,
                }
            }
        }
    }
}

// ============================================================================
// 多模态上下文片段
// ============================================================================

/// 多模态上下文片段
///
/// 统一表示来自不同维度的记忆片段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalChunk {
    /// 唯一 ID
    pub id: Uuid,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 数据来源维度
    pub dimension: DataDimension,

    /// 多模态内容
    pub content: MultimodalContent,

    /// 状态向量（9维）
    pub state_vector: MemoryStateVector,

    /// 元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MultimodalChunk {
    /// 创建新的片段
    pub fn new(
        dimension: DataDimension,
        content: MultimodalContent,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp,
            dimension,
            content,
            state_vector: MemoryStateVector::default(),
            metadata: HashMap::new(),
        }
    }

    /// 从对话回合创建
    pub fn from_conversation_round(round: &ConversationRound) -> Self {
        let content = MultimodalContent::Text {
            content: format!(
                "用户: {}\nAI: {}",
                round.user_input, round.ai_response
            ),
        };

        let mut chunk = Self::new(DataDimension::Context, content, round.timestamp);

        // 设置状态向量
        chunk.state_vector.text_weight = 1.0;
        chunk.state_vector.visual_weight = 0.0;
        chunk.state_vector.data_weight = 0.0;
        chunk.state_vector.normalize_content_weights();

        // 元数据
        chunk.metadata.insert("round_index".to_string(), round.index.into());
        chunk.metadata.insert("model".to_string(), round.model.clone().into());
        chunk.metadata.insert("execution_time".to_string(), round.execution_time.into());

        chunk
    }

    /// 从图表历史创建
    pub fn from_chart_history(entry: &ChartHistoryEntry) -> Self {
        let description = format!(
            "图表: {} (类型: {:?})",
            &entry.title,
            entry.chart_data.chart_type
        );

        let content = MultimodalContent::Chart {
            chart: entry.chart_data.clone(),
            description,
        };

        let mut chunk = Self::new(DataDimension::ChartHistory, content, entry.timestamp);

        // 设置状态向量
        chunk.state_vector.text_weight = 0.0;
        chunk.state_vector.visual_weight = 1.0;
        chunk.state_vector.data_weight = 0.0;
        chunk.state_vector.normalize_content_weights();

        // 元数据
        chunk.metadata.insert("chart_id".to_string(), entry.id.clone().into());

        chunk
    }

    /// 从图像历史创建
    pub fn from_image_history(entry: &ImageHistoryEntry) -> Self {
        let description = format!(
            "图像: {}",
            entry.image_data.filename.as_deref().unwrap_or("未知")
        );

        let content = MultimodalContent::Image {
            image: entry.image_data.clone(),
            description,
        };

        let mut chunk = Self::new(DataDimension::ImageHistory, content, entry.timestamp);

        // 设置状态向量
        chunk.state_vector.text_weight = 0.0;
        chunk.state_vector.visual_weight = 1.0;
        chunk.state_vector.data_weight = 0.0;
        chunk.state_vector.normalize_content_weights();

        // 元数据
        chunk.metadata.insert("image_id".to_string(), entry.id.clone().into());

        chunk
    }

    /// 计算与目标任务的相关性得分 [0.0, 1.0]
    ///
    /// 基础实现：关键词匹配（后续可升级为语义向量）
    pub fn relevance_to(&self, task: &str) -> f64 {
        let text = self.content.to_text().to_lowercase();
        let task_lower = task.to_lowercase();

        // 简单关键词匹配
        let task_words: Vec<&str> = task_lower.split_whitespace().collect();
        if task_words.is_empty() {
            return 0.0;
        }

        let matched = task_words.iter()
            .filter(|word| text.contains(*word))
            .count();

        (matched as f64) / (task_words.len() as f64)
    }
}

// ============================================================================
// 易经64卦：内容类型（内卦，8种）
// ============================================================================

/// 内容类型（8种基础特征）
///
/// 对应易经的"内卦"：描述内容的本质特征
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentType {
    /// 纯文本（乾☰）
    TextOnly,

    /// 纯可视化（坤☷）
    VisualOnly,

    /// 纯数据（震☳）
    DataOnly,

    /// 文本+可视化（巽☴）
    TextAndVisual,

    /// 文本+数据（坎☵）
    TextAndData,

    /// 可视化+数据（离☲）
    VisualAndData,

    /// 三者混合（艮☶）
    FullMix,

    /// 交互操作（兑☱）
    Interactive,
}

// ============================================================================
// 易经64卦：时间情境（外卦，8种）
// ============================================================================

/// 时间情境（8种）
///
/// 对应易经的"外卦"：描述时间背景
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeContext {
    /// 当前对话（9轮以内）（乾☰）
    CurrentDialog,

    /// 当前会话（本次会话）（坤☷）
    CurrentSession,

    /// 最近会话（3天内）（震☳）
    RecentSession,

    /// 中期会话（1周-1月）（巽☴）
    MediumTermSession,

    /// 长期会话（1月+）（坎☵）
    LongTermSession,

    /// 重复模式（跨会话的重复任务）（离☲）
    RepeatedPattern,

    /// 演化趋势（用户习惯的变化）（艮☶）
    EvolutionTrend,

    /// 稀有事件（偶尔出现）（兑☱）
    RareEvent,
}

impl TimeContext {
    /// 从时间跨度推断情境
    pub fn from_age(age_seconds: i64) -> Self {
        const HOUR: i64 = 3600;
        const DAY: i64 = 24 * HOUR;
        const WEEK: i64 = 7 * DAY;
        const MONTH: i64 = 30 * DAY;

        if age_seconds < HOUR {
            Self::CurrentDialog
        } else if age_seconds < DAY {
            Self::CurrentSession
        } else if age_seconds < 3 * DAY {
            Self::RecentSession
        } else if age_seconds < MONTH {
            Self::MediumTermSession
        } else {
            Self::LongTermSession
        }
    }
}

// ============================================================================
// 易经64卦：记忆动作
// ============================================================================

/// 记忆动作（64种智能决策的输出）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryAction {
    /// 高优先级：直接加入 LLM 上下文
    HighPriority {
        action: String,
        token_budget: usize,
    },

    /// 中优先级：生成摘要后加入
    MediumPriority {
        action: String,
        token_budget: usize,
    },

    /// 推荐复用：建议用户复用历史内容
    RecommendReuse {
        suggestion: String,
        template_id: Option<Uuid>,
    },

    /// 自动建议：预填充参数
    AutoSuggest {
        pre_fill: String,
        workflow: Vec<String>,
    },

    /// 降级到 LLM：无法自动决策
    FallbackToLLM,
}

impl MemoryAction {
    /// 调整置信度
    pub fn scale_confidence(self, weight: f64) -> Self {
        match self {
            Self::HighPriority { action, token_budget } => {
                if weight < 0.7 {
                    // 降级到中优先级
                    Self::MediumPriority { action, token_budget }
                } else {
                    Self::HighPriority { action, token_budget }
                }
            }
            Self::MediumPriority { action: _, token_budget: _ } => {
                if weight < 0.5 {
                    // 降级到 Fallback
                    Self::FallbackToLLM
                } else {
                    self
                }
            }
            _ => self,
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
    fn test_memory_state_vector_lerp() {
        let start = MemoryStateVector {
            text_weight: 0.9,
            visual_weight: 0.1,
            data_weight: 0.0,
            ..Default::default()
        };

        let end = MemoryStateVector {
            text_weight: 0.3,
            visual_weight: 0.4,
            data_weight: 0.3,
            ..Default::default()
        };

        // 中间状态（50%）
        let mid = start.lerp(&end, 0.5);
        assert!((mid.text_weight - 0.6).abs() < 0.01);
        assert!((mid.visual_weight - 0.25).abs() < 0.01);
        assert!((mid.data_weight - 0.15).abs() < 0.01);
    }

    #[test]
    fn test_content_type_classification() {
        let text = MultimodalContent::Text {
            content: "Hello".to_string(),
        };
        assert_eq!(text.content_type(), ContentType::TextOnly);

        let composite = MultimodalContent::Composite {
            text: Some("Test".to_string()),
            chart: None,
            data_summary: Some("Data".to_string()),
        };
        assert_eq!(composite.content_type(), ContentType::TextAndData);
    }

    #[test]
    fn test_time_context_from_age() {
        assert_eq!(TimeContext::from_age(1800), TimeContext::CurrentDialog); // 30分钟
        assert_eq!(TimeContext::from_age(7200), TimeContext::CurrentSession); // 2小时
        assert_eq!(TimeContext::from_age(86400 * 2), TimeContext::RecentSession); // 2天
        assert_eq!(TimeContext::from_age(86400 * 10), TimeContext::MediumTermSession); // 10天
    }

    #[test]
    fn test_multimodal_chunk_relevance() {
        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "Rust trait debugging error".to_string(),
            },
            Utc::now(),
        );

        // 高相关性
        let score1 = chunk.relevance_to("debugging Rust trait");
        assert!(score1 > 0.6);

        // 低相关性
        let score2 = chunk.relevance_to("Python machine learning");
        assert!(score2 < 0.3);
    }
}
