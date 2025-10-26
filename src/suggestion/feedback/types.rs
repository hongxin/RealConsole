//! 反馈系统数据类型定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 建议反馈记录
///
/// 记录单次建议展示和用户响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeedback {
    /// 反馈 ID（唯一标识）
    pub id: String,

    /// 建议内容
    pub suggestion: String,

    /// 建议来源
    pub source: String,

    /// 原始评分
    pub original_score: f64,

    /// 反馈类型
    pub feedback_type: FeedbackType,

    /// 选择的索引（如果接受）- 0-based
    pub selected_index: Option<usize>,

    /// 建议总数
    pub total_suggestions: usize,

    /// 上下文信息
    pub context: FeedbackContext,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

impl SuggestionFeedback {
    /// 创建新的反馈记录
    pub fn new(
        suggestion: String,
        source: String,
        original_score: f64,
        feedback_type: FeedbackType,
        context: FeedbackContext,
    ) -> Self {
        Self {
            id: Self::generate_id(),
            suggestion,
            source,
            original_score,
            feedback_type,
            selected_index: None,
            total_suggestions: 0,
            context,
            timestamp: Utc::now(),
        }
    }

    /// 生成唯一 ID
    fn generate_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("fb_{}", timestamp)
    }

    /// 设置选择索引
    pub fn with_selection(mut self, index: usize, total: usize) -> Self {
        self.selected_index = Some(index);
        self.total_suggestions = total;
        self
    }
}

/// 反馈类型
///
/// 三态反馈系统
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// 接受（用户选择了建议）
    Accepted,

    /// 跳过（用户看到但未选择）
    Skipped,

    /// 拒绝（用户明确拒绝，未来功能）
    #[allow(dead_code)]
    Rejected,
}

impl FeedbackType {
    /// 获取反馈类型的权重
    ///
    /// 用于评分调整
    pub fn weight(&self) -> f64 {
        match self {
            FeedbackType::Accepted => 1.0,  // 积极信号
            FeedbackType::Skipped => 0.0,   // 中性信号
            FeedbackType::Rejected => -1.0, // 消极信号
        }
    }
}

/// 反馈上下文
///
/// 记录建议生成时的环境信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    /// 当前目录
    pub current_dir: String,

    /// 项目类型
    pub project_type: Option<String>,

    /// 失败的命令
    pub failed_command: Option<String>,

    /// 错误输出（截断到前500字符）
    pub error_output: Option<String>,

    /// 最近命令（最多3条）
    pub recent_commands: Vec<String>,
}

impl FeedbackContext {
    /// 创建新的反馈上下文
    pub fn new(current_dir: String) -> Self {
        Self {
            current_dir,
            project_type: None,
            failed_command: None,
            error_output: None,
            recent_commands: Vec::new(),
        }
    }

    /// 从 SuggestionContext 创建
    pub fn from_suggestion_context(ctx: &crate::suggestion::SuggestionContext) -> Self {
        Self {
            current_dir: ctx
                .current_dir
                .to_str()
                .unwrap_or("unknown")
                .to_string(),
            project_type: ctx.project_type.as_ref().map(|t| format!("{:?}", t)),
            failed_command: ctx.recent_commands.first().cloned(),
            error_output: ctx
                .last_command_output
                .as_ref()
                .map(|s| s.chars().take(500).collect()),
            recent_commands: ctx.recent_commands.iter().take(3).cloned().collect(),
        }
    }
}

/// 建议使用统计
///
/// 聚合某个建议的历史使用数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionStats {
    /// 建议命令（模式）
    pub command_pattern: String,

    /// 总展示次数
    pub shown_count: usize,

    /// 被接受次数
    pub accepted_count: usize,

    /// 被跳过次数
    pub skipped_count: usize,

    /// 接受率（0.0-1.0）
    pub acceptance_rate: f64,

    /// 平均选择位置（1-based）
    ///
    /// 越小越好，1 表示总是第一个被选择
    pub avg_position: f64,

    /// 最后更新时间
    pub last_updated: DateTime<Utc>,

    /// 首次出现时间
    pub first_seen: DateTime<Utc>,
}

impl SuggestionStats {
    /// 创建新的统计记录
    pub fn new(command_pattern: String) -> Self {
        let now = Utc::now();
        Self {
            command_pattern,
            shown_count: 0,
            accepted_count: 0,
            skipped_count: 0,
            acceptance_rate: 0.0,
            avg_position: 0.0,
            last_updated: now,
            first_seen: now,
        }
    }

    /// 更新统计数据
    pub fn update(&mut self, feedback: &SuggestionFeedback) {
        self.shown_count += 1;
        self.last_updated = Utc::now();

        match feedback.feedback_type {
            FeedbackType::Accepted => {
                self.accepted_count += 1;

                // 更新平均位置（1-based）
                if let Some(index) = feedback.selected_index {
                    let position = (index + 1) as f64;
                    if self.avg_position == 0.0 {
                        self.avg_position = position;
                    } else {
                        // 指数移动平均
                        self.avg_position = self.avg_position * 0.7 + position * 0.3;
                    }
                }
            }
            FeedbackType::Skipped => {
                self.skipped_count += 1;
            }
            FeedbackType::Rejected => {
                // 未来功能
            }
        }

        // 更新接受率
        self.acceptance_rate = self.accepted_count as f64 / self.shown_count as f64;
    }

    /// 计算建议质量分数（0.0-1.0）
    ///
    /// 综合考虑接受率和平均位置
    pub fn quality_score(&self) -> f64 {
        if self.shown_count == 0 {
            return 0.5; // 默认中等分数
        }

        // 接受率权重 70%
        let acceptance_score = self.acceptance_rate * 0.7;

        // 位置权重 30%（位置越靠前越好）
        let position_score = if self.avg_position > 0.0 {
            // 位置 1 = 1.0, 位置 2 = 0.75, 位置 3 = 0.5, ...
            (1.0 / self.avg_position).min(1.0) * 0.3
        } else {
            0.0
        };

        (acceptance_score + position_score).clamp(0.0, 1.0)
    }

    /// 是否为高质量建议
    pub fn is_high_quality(&self) -> bool {
        self.quality_score() > 0.7
    }

    /// 是否为低质量建议
    pub fn is_low_quality(&self) -> bool {
        self.shown_count >= 5 && self.quality_score() < 0.3
    }
}

/// 反馈记录（兼容旧的 error_fixer 模块）
///
/// 用于与现有的 FeedbackLearner 集成
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    pub command: String,
    pub feedback_type: String,
    pub timestamp: DateTime<Utc>,
}

impl From<&SuggestionFeedback> for FeedbackRecord {
    fn from(feedback: &SuggestionFeedback) -> Self {
        Self {
            command: feedback.suggestion.clone(),
            feedback_type: match feedback.feedback_type {
                FeedbackType::Accepted => "accepted".to_string(),
                FeedbackType::Skipped => "skipped".to_string(),
                FeedbackType::Rejected => "rejected".to_string(),
            },
            timestamp: feedback.timestamp,
        }
    }
}

/// 学习配置
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// 是否启用学习
    pub enabled: bool,

    /// 最小样本数（低于此数不调整）
    pub min_samples: usize,

    /// 评分调整幅度（0.0-1.0）
    pub adjustment_magnitude: f64,

    /// 接受率权重
    pub acceptance_weight: f64,

    /// 位置权重
    pub position_weight: f64,

    /// 时间衰减因子（越老的数据权重越低）
    pub time_decay: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_samples: 3, // 至少3次展示才开始调整
            adjustment_magnitude: 0.2, // 最多调整 ±0.2
            acceptance_weight: 0.7,
            position_weight: 0.3,
            time_decay: 0.95, // 每天衰减5%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_feedback_creation() {
        let context = FeedbackContext::new("/test/dir".to_string());
        let feedback = SuggestionFeedback::new(
            "cargo build".to_string(),
            "Context".to_string(),
            0.85,
            FeedbackType::Accepted,
            context,
        );

        assert_eq!(feedback.suggestion, "cargo build");
        assert_eq!(feedback.original_score, 0.85);
        assert!(feedback.id.starts_with("fb_"));
    }

    #[test]
    fn test_feedback_type_weight() {
        assert_eq!(FeedbackType::Accepted.weight(), 1.0);
        assert_eq!(FeedbackType::Skipped.weight(), 0.0);
        assert_eq!(FeedbackType::Rejected.weight(), -1.0);
    }

    #[test]
    fn test_suggestion_stats_update() {
        let mut stats = SuggestionStats::new("cargo build".to_string());

        // 第一次展示，被接受
        let context = FeedbackContext::new("/test".to_string());
        let feedback = SuggestionFeedback::new(
            "cargo build".to_string(),
            "Context".to_string(),
            0.85,
            FeedbackType::Accepted,
            context.clone(),
        )
        .with_selection(0, 3);

        stats.update(&feedback);

        assert_eq!(stats.shown_count, 1);
        assert_eq!(stats.accepted_count, 1);
        assert_eq!(stats.acceptance_rate, 1.0);
        assert_eq!(stats.avg_position, 1.0);

        // 第二次展示，被跳过
        let feedback2 = SuggestionFeedback::new(
            "cargo build".to_string(),
            "Context".to_string(),
            0.85,
            FeedbackType::Skipped,
            context,
        );

        stats.update(&feedback2);

        assert_eq!(stats.shown_count, 2);
        assert_eq!(stats.accepted_count, 1);
        assert_eq!(stats.skipped_count, 1);
        assert_eq!(stats.acceptance_rate, 0.5);
    }

    #[test]
    fn test_suggestion_stats_quality_score() {
        let mut stats = SuggestionStats::new("test".to_string());

        // 没有数据时，默认中等分数
        assert_eq!(stats.quality_score(), 0.5);

        // 100% 接受率，位置 1
        stats.shown_count = 5;
        stats.accepted_count = 5;
        stats.acceptance_rate = 1.0;
        stats.avg_position = 1.0;

        let score = stats.quality_score();
        assert!(score > 0.9, "Expected high score, got {}", score);

        // 低接受率
        stats.accepted_count = 1;
        stats.acceptance_rate = 0.2;
        stats.avg_position = 3.0;

        let score = stats.quality_score();
        assert!(score < 0.4, "Expected low score, got {}", score);
    }

    #[test]
    fn test_suggestion_stats_quality_thresholds() {
        let mut stats = SuggestionStats::new("test".to_string());
        stats.shown_count = 10;
        stats.accepted_count = 9;
        stats.acceptance_rate = 0.9;
        stats.avg_position = 1.0;

        assert!(stats.is_high_quality());
        assert!(!stats.is_low_quality());

        // 低质量
        stats.accepted_count = 1;
        stats.acceptance_rate = 0.1;
        stats.shown_count = 10;
        stats.avg_position = 3.0; // 低位置（被选择时排在第3位）

        assert!(!stats.is_high_quality());
        assert!(stats.is_low_quality());
    }

    #[test]
    fn test_learning_config_default() {
        let config = LearningConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_samples, 3);
        assert_eq!(config.adjustment_magnitude, 0.2);
    }
}
