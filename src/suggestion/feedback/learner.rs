//! 反馈学习器
//!
//! 基于用户反馈分析和调整建议评分

use super::collector::FeedbackCollector;
use super::types::{LearningConfig, SuggestionStats};
use crate::suggestion::{Suggestion, SuggestionContext};
use std::sync::Arc;

/// 反馈学习器
///
/// 分析用户反馈历史，动态调整建议评分
///
/// ## 三层学习机制
///
/// ```text
/// 即时学习（Instant）     →  基于质量分数直接调整（0.5-1.5x）
/// 短期学习（Short-term）  →  最近表现趋势（接受率）
/// 长期学习（Long-term）   →  历史数据质量评估（质量分数）
/// ```
///
/// ## 评分调整公式
///
/// ```text
/// adjusted_score = original_score × quality_multiplier
///
/// quality_multiplier = 1.0 + (quality_score - 0.5) × adjustment_magnitude
///                    = 1.0 + (0.0 ~ 1.0 - 0.5) × 0.2
///                    = 0.9 ~ 1.1  (默认配置)
///
/// 其中：
/// - quality_score = acceptance_rate × 0.7 + position_score × 0.3
/// - adjustment_magnitude = 0.2 (默认)
/// ```
pub struct FeedbackLearner {
    /// 反馈收集器
    collector: Arc<FeedbackCollector>,

    /// 学习配置
    config: LearningConfig,
}

impl FeedbackLearner {
    /// 创建新的学习器
    pub fn new(collector: Arc<FeedbackCollector>, config: LearningConfig) -> Self {
        Self { collector, config }
    }

    /// 使用默认配置创建学习器
    pub fn with_default_config(collector: Arc<FeedbackCollector>) -> Self {
        Self::new(collector, LearningConfig::default())
    }

    /// 调整建议评分
    ///
    /// 基于历史反馈数据调整建议的评分
    ///
    /// # 参数
    /// - `suggestion`: 待调整的建议
    /// - `_context`: 建议上下文（预留用于上下文相关的学习）
    ///
    /// # 返回
    /// 调整后的评分（0.0-1.0）
    ///
    /// # 示例
    /// ```ignore
    /// # use realconsole::suggestion::feedback::{FeedbackCollector, FeedbackLearner};
    /// # use realconsole::suggestion::{Suggestion, SuggestionContext};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let collector = FeedbackCollector::from_default_location().await?;
    /// let learner = FeedbackLearner::with_default_config(Arc::new(collector));
    /// let mut suggestion = Suggestion::new("cargo build", "Build the project", 0.8, SuggestionSource::Context);
    /// let context = SuggestionContext::from_env();
    ///
    /// let adjusted_score = learner.adjust_score(&suggestion, &context).await;
    /// println!("Original: {}, Adjusted: {}", suggestion.score, adjusted_score);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn adjust_score(
        &self,
        suggestion: &Suggestion,
        _context: &SuggestionContext,
    ) -> f64 {
        // 如果学习未启用，直接返回原始评分
        if !self.config.enabled {
            return suggestion.score;
        }

        // 获取建议的历史统计
        let stats = match self.get_stats(&suggestion.command).await {
            Some(stats) => stats,
            None => {
                // 没有历史数据，返回原始评分
                return suggestion.score;
            }
        };

        // 检查是否有足够的样本
        if stats.shown_count < self.config.min_samples {
            return suggestion.score;
        }

        // 计算质量分数
        let quality_score = self.calculate_quality_score(&stats);

        // 计算调整倍数
        let multiplier = self.calculate_multiplier(quality_score);

        // 应用调整
        let adjusted_score = suggestion.score * multiplier;

        // 确保评分在有效范围内
        adjusted_score.clamp(0.0, 1.0)
    }

    /// 批量调整建议评分
    ///
    /// # 示例
    /// ```ignore
    /// # use realconsole::suggestion::feedback::{FeedbackCollector, FeedbackLearner};
    /// # use realconsole::suggestion::{Suggestion, SuggestionContext};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let collector = FeedbackCollector::from_default_location().await?;
    /// let learner = FeedbackLearner::with_default_config(Arc::new(collector));
    /// let mut suggestions = vec![/* ... */];
    /// let context = SuggestionContext::from_env();
    ///
    /// learner.adjust_scores(&mut suggestions, &context).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn adjust_scores(
        &self,
        suggestions: &mut [Suggestion],
        context: &SuggestionContext,
    ) {
        for suggestion in suggestions.iter_mut() {
            suggestion.score = self.adjust_score(suggestion, context).await;
        }
    }

    /// 获取建议的统计信息
    ///
    /// # 示例
    /// ```ignore
    /// # use realconsole::suggestion::feedback::{FeedbackCollector, FeedbackLearner};
    /// # async fn example() -> anyhow::Result<()> {
    /// # let collector = FeedbackCollector::from_default_location().await?;
    /// let learner = FeedbackLearner::with_default_config(Arc::new(collector));
    ///
    /// if let Some(stats) = learner.get_stats("cargo build").await {
    ///     println!("Acceptance rate: {:.2}%", stats.acceptance_rate * 100.0);
    ///     println!("Quality score: {:.2}", stats.quality_score());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_stats(&self, command_pattern: &str) -> Option<SuggestionStats> {
        let storage = self.collector.storage();
        let storage = storage.read().await;
        storage.get_stats(command_pattern).await.ok().flatten()
    }

    /// 获取所有高质量建议
    pub async fn get_high_quality_suggestions(&self) -> Vec<SuggestionStats> {
        let storage = self.collector.storage();
        let storage = storage.read().await;
        storage
            .get_high_quality_suggestions()
            .await
            .unwrap_or_default()
    }

    /// 获取所有低质量建议
    pub async fn get_low_quality_suggestions(&self) -> Vec<SuggestionStats> {
        let storage = self.collector.storage();
        let storage = storage.read().await;
        storage
            .get_low_quality_suggestions()
            .await
            .unwrap_or_default()
    }

    /// 计算质量分数
    ///
    /// 融合接受率和位置权重
    fn calculate_quality_score(&self, stats: &SuggestionStats) -> f64 {
        // 接受率权重
        let acceptance_score = stats.acceptance_rate * self.config.acceptance_weight;

        // 位置权重（位置越靠前越好）
        let position_score = if stats.avg_position > 0.0 {
            (1.0 / stats.avg_position).min(1.0) * self.config.position_weight
        } else {
            0.0
        };

        // 融合分数
        (acceptance_score + position_score).clamp(0.0, 1.0)
    }

    /// 计算调整倍数
    ///
    /// 基于质量分数计算评分调整倍数
    ///
    /// # 公式
    /// ```text
    /// multiplier = 1.0 + (quality_score - 0.5) × adjustment_magnitude
    ///
    /// 示例（adjustment_magnitude = 0.2）：
    /// - quality_score = 0.0  →  multiplier = 0.9   (降低10%)
    /// - quality_score = 0.5  →  multiplier = 1.0   (不变)
    /// - quality_score = 1.0  →  multiplier = 1.1   (提升10%)
    /// ```
    fn calculate_multiplier(&self, quality_score: f64) -> f64 {
        // 质量分数 0.5 为中性点
        let delta = quality_score - 0.5;

        // 应用调整幅度
        let multiplier = 1.0 + delta * self.config.adjustment_magnitude;

        // 限制倍数范围（避免过度调整）
        multiplier.clamp(0.5, 1.5)
    }

    /// 获取学习配置
    pub fn config(&self) -> &LearningConfig {
        &self.config
    }

    /// 更新学习配置
    pub fn set_config(&mut self, config: LearningConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggestion::feedback::{FeedbackContext, FeedbackStorage, FeedbackType};
    use crate::suggestion::{SuggestionCategory, SuggestionSource};
    use tempfile::TempDir;

    async fn create_test_learner() -> (FeedbackLearner, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = FeedbackStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        let collector = Arc::new(FeedbackCollector::new(storage));
        let learner = FeedbackLearner::with_default_config(collector);
        (learner, temp_dir)
    }

    fn create_test_suggestion(cmd: &str, score: f64) -> Suggestion {
        Suggestion {
            command: cmd.to_string(),
            description: format!("Test: {}", cmd),
            score,
            source: SuggestionSource::Context,
            category: SuggestionCategory::General,
            needs_confirmation: false,
        }
    }

    #[tokio::test]
    async fn test_adjust_score_no_history() {
        let (learner, _temp) = create_test_learner().await;

        let suggestion = create_test_suggestion("unknown_cmd", 0.8);
        let context = SuggestionContext::from_env();

        let adjusted = learner.adjust_score(&suggestion, &context).await;

        // 没有历史数据，应该返回原始评分
        assert_eq!(adjusted, 0.8);
    }

    #[tokio::test]
    async fn test_adjust_score_insufficient_samples() {
        let (learner, _temp) = create_test_learner().await;

        // 创建少量历史数据（< min_samples）
        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());
        let feedback = super::super::types::SuggestionFeedback::new(
            "test_cmd".to_string(),
            "Test".to_string(),
            0.8,
            FeedbackType::Accepted,
            context,
        );
        storage.update_stats(&feedback).await.unwrap();
        drop(storage);

        let suggestion = create_test_suggestion("test_cmd", 0.8);
        let ctx = SuggestionContext::from_env();

        let adjusted = learner.adjust_score(&suggestion, &ctx).await;

        // 样本不足，应该返回原始评分
        assert_eq!(adjusted, 0.8);
    }

    #[tokio::test]
    async fn test_adjust_score_high_quality() {
        let (learner, _temp) = create_test_learner().await;

        // 创建高质量历史数据
        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());

        // 添加5次，全部被接受，位置都是第1个
        for _ in 0..5 {
            let feedback = super::super::types::SuggestionFeedback::new(
                "good_cmd".to_string(),
                "Test".to_string(),
                0.8,
                FeedbackType::Accepted,
                context.clone(),
            )
            .with_selection(0, 3);

            storage.update_stats(&feedback).await.unwrap();
        }
        drop(storage);

        let suggestion = create_test_suggestion("good_cmd", 0.8);
        let ctx = SuggestionContext::from_env();

        let adjusted = learner.adjust_score(&suggestion, &ctx).await;

        // 高质量建议，评分应该提升
        assert!(adjusted > 0.8, "Expected score > 0.8, got {}", adjusted);
    }

    #[tokio::test]
    async fn test_adjust_score_low_quality() {
        let (learner, _temp) = create_test_learner().await;

        // 创建低质量历史数据
        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());

        // 添加10次，只有1次被接受
        for i in 0..10 {
            let feedback_type = if i == 0 {
                FeedbackType::Accepted
            } else {
                FeedbackType::Skipped
            };

            let mut feedback = super::super::types::SuggestionFeedback::new(
                "bad_cmd".to_string(),
                "Test".to_string(),
                0.8,
                feedback_type,
                context.clone(),
            );

            if i == 0 {
                feedback = feedback.with_selection(2, 3); // 第3个位置
            }

            storage.update_stats(&feedback).await.unwrap();
        }
        drop(storage);

        let suggestion = create_test_suggestion("bad_cmd", 0.8);
        let ctx = SuggestionContext::from_env();

        let adjusted = learner.adjust_score(&suggestion, &ctx).await;

        // 低质量建议，评分应该降低
        assert!(adjusted < 0.8, "Expected score < 0.8, got {}", adjusted);
    }

    #[tokio::test]
    async fn test_adjust_scores_batch() {
        let (learner, _temp) = create_test_learner().await;

        // 创建历史数据
        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());

        // cmd_1: 高质量
        for _ in 0..5 {
            let feedback = super::super::types::SuggestionFeedback::new(
                "cmd_1".to_string(),
                "Test".to_string(),
                0.8,
                FeedbackType::Accepted,
                context.clone(),
            )
            .with_selection(0, 3);
            storage.update_stats(&feedback).await.unwrap();
        }

        // cmd_2: 低质量
        for i in 0..10 {
            let feedback_type = if i == 0 {
                FeedbackType::Accepted
            } else {
                FeedbackType::Skipped
            };
            let feedback = super::super::types::SuggestionFeedback::new(
                "cmd_2".to_string(),
                "Test".to_string(),
                0.8,
                feedback_type,
                context.clone(),
            );
            storage.update_stats(&feedback).await.unwrap();
        }
        drop(storage);

        let mut suggestions = vec![
            create_test_suggestion("cmd_1", 0.8),
            create_test_suggestion("cmd_2", 0.8),
            create_test_suggestion("cmd_3", 0.8), // 无历史
        ];

        let ctx = SuggestionContext::from_env();
        learner.adjust_scores(&mut suggestions, &ctx).await;

        // cmd_1 应该提升
        assert!(
            suggestions[0].score > 0.8,
            "cmd_1 score should increase"
        );

        // cmd_2 应该降低
        assert!(suggestions[1].score < 0.8, "cmd_2 score should decrease");

        // cmd_3 应该保持不变
        assert_eq!(suggestions[2].score, 0.8, "cmd_3 score should remain");
    }

    #[tokio::test]
    async fn test_get_stats() {
        let (learner, _temp) = create_test_learner().await;

        // 创建统计数据
        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());
        let feedback = super::super::types::SuggestionFeedback::new(
            "test_cmd".to_string(),
            "Test".to_string(),
            0.8,
            FeedbackType::Accepted,
            context,
        );
        storage.update_stats(&feedback).await.unwrap();
        drop(storage);

        let stats = learner.get_stats("test_cmd").await;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.command_pattern, "test_cmd");
        assert_eq!(stats.shown_count, 1);
    }

    #[tokio::test]
    async fn test_calculate_quality_score() {
        let (learner, _temp) = create_test_learner().await;

        let mut stats = SuggestionStats::new("test".to_string());

        // 100% 接受率，位置 1
        stats.shown_count = 5;
        stats.accepted_count = 5;
        stats.acceptance_rate = 1.0;
        stats.avg_position = 1.0;

        let score = learner.calculate_quality_score(&stats);
        assert!(score > 0.9, "High quality should have score > 0.9");

        // 10% 接受率，位置 3
        stats.accepted_count = 1;
        stats.acceptance_rate = 0.1;
        stats.avg_position = 3.0;

        let score = learner.calculate_quality_score(&stats);
        assert!(score < 0.3, "Low quality should have score < 0.3");
    }

    #[tokio::test]
    async fn test_calculate_multiplier() {
        let (learner, _temp) = create_test_learner().await;

        // 质量分数 0.5 → 倍数 1.0（不变）
        let mult = learner.calculate_multiplier(0.5);
        assert_eq!(mult, 1.0);

        // 质量分数 1.0 → 倍数 > 1.0（提升）
        let mult = learner.calculate_multiplier(1.0);
        assert!(mult > 1.0);
        assert!(mult <= 1.5); // 不超过上限

        // 质量分数 0.0 → 倍数 < 1.0（降低）
        let mult = learner.calculate_multiplier(0.0);
        assert!(mult < 1.0);
        assert!(mult >= 0.5); // 不低于下限
    }

    #[tokio::test]
    async fn test_learning_disabled() {
        let temp_dir = TempDir::new().unwrap();
        let storage = FeedbackStorage::new(temp_dir.path().to_path_buf())
            .await
            .unwrap();
        let collector = Arc::new(FeedbackCollector::new(storage));

        // 创建禁用学习的配置
        let mut config = LearningConfig::default();
        config.enabled = false;

        let learner = FeedbackLearner::new(collector, config);

        let suggestion = create_test_suggestion("test", 0.8);
        let context = SuggestionContext::from_env();

        let adjusted = learner.adjust_score(&suggestion, &context).await;

        // 学习禁用，应该返回原始评分
        assert_eq!(adjusted, 0.8);
    }

    #[tokio::test]
    async fn test_get_high_quality_suggestions() {
        let (learner, _temp) = create_test_learner().await;

        let storage = learner.collector.storage();
        let storage = storage.write().await;

        let context = FeedbackContext::new("/test".to_string());

        // 创建高质量建议
        for _ in 0..10 {
            let feedback = super::super::types::SuggestionFeedback::new(
                "excellent_cmd".to_string(),
                "Test".to_string(),
                0.9,
                FeedbackType::Accepted,
                context.clone(),
            )
            .with_selection(0, 3);
            storage.update_stats(&feedback).await.unwrap();
        }
        drop(storage);

        let high_quality = learner.get_high_quality_suggestions().await;
        assert_eq!(high_quality.len(), 1);
        assert_eq!(high_quality[0].command_pattern, "excellent_cmd");
    }
}
