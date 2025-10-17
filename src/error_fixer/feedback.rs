//! 用户反馈学习系统 - Phase 9.1 Week 3
//!
//! 功能：
//! - 收集用户对修复建议的反馈
//! - 追踪修复效果
//! - 学习用户偏好
//! - 优化策略排序
//!
//! 设计理念（一分为三）：
//! - 收集层：记录反馈数据
//! - 分析层：学习模式和趋势
//! - 应用层：优化决策和推荐

use super::{ErrorAnalysis, FixStrategy};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 用户反馈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeedbackType {
    /// 采纳建议
    Accepted,
    /// 拒绝建议
    Rejected,
    /// 修改后采纳
    Modified,
    /// 跳过
    Skipped,
}

/// 修复结果
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixOutcome {
    /// 成功解决问题
    Success,
    /// 失败（问题未解决）
    Failure,
    /// 部分成功
    Partial,
    /// 未知（未执行）
    Unknown,
}

/// 反馈记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackRecord {
    /// 记录 ID
    pub id: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 错误信息
    pub error_pattern: String,

    /// 错误类别
    pub error_category: String,

    /// 原始命令
    pub original_command: String,

    /// 建议的修复策略名称
    pub strategy_name: String,

    /// 策略内容
    pub strategy_command: String,

    /// 用户反馈
    pub feedback: FeedbackType,

    /// 修复结果
    pub outcome: FixOutcome,

    /// 用户修改后的命令（如果有）
    pub modified_command: Option<String>,

    /// 上下文信息
    pub context: HashMap<String, String>,
}

impl FeedbackRecord {
    /// 创建新的反馈记录
    pub fn new(
        analysis: &ErrorAnalysis,
        strategy: &FixStrategy,
        feedback: FeedbackType,
        outcome: FixOutcome,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();

        Self {
            id,
            timestamp: Utc::now(),
            error_pattern: analysis.pattern_name.clone().unwrap_or_default(),
            error_category: format!("{:?}", analysis.category),
            original_command: analysis.command.clone(),
            strategy_name: strategy.name.clone(),
            strategy_command: strategy.command.clone(),
            feedback,
            outcome,
            modified_command: None,
            context: HashMap::new(),
        }
    }

    /// 设置修改后的命令
    pub fn with_modified_command(mut self, command: String) -> Self {
        self.modified_command = Some(command);
        self
    }

    /// 添加上下文信息
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    /// 判断是否为正面反馈
    pub fn is_positive(&self) -> bool {
        matches!(
            self.feedback,
            FeedbackType::Accepted | FeedbackType::Modified
        ) && matches!(
            self.outcome,
            FixOutcome::Success | FixOutcome::Partial
        )
    }
}

/// 策略统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStats {
    /// 策略名称
    pub name: String,

    /// 总使用次数
    pub total_uses: u32,

    /// 采纳次数
    pub accepted_count: u32,

    /// 成功次数
    pub success_count: u32,

    /// 失败次数
    pub failure_count: u32,

    /// 采纳率
    pub acceptance_rate: f64,

    /// 成功率
    pub success_rate: f64,

    /// 最后使用时间
    pub last_used: DateTime<Utc>,

    /// 平均效果得分 (0.0 - 1.0)
    pub effectiveness_score: f64,
}

impl StrategyStats {
    /// 创建新的统计信息
    pub fn new(name: String) -> Self {
        Self {
            name,
            total_uses: 0,
            accepted_count: 0,
            success_count: 0,
            failure_count: 0,
            acceptance_rate: 0.0,
            success_rate: 0.0,
            last_used: Utc::now(),
            effectiveness_score: 0.5, // 默认中等分数
        }
    }

    /// 更新统计信息
    pub fn update(&mut self, record: &FeedbackRecord) {
        self.total_uses += 1;
        self.last_used = record.timestamp;

        if matches!(
            record.feedback,
            FeedbackType::Accepted | FeedbackType::Modified
        ) {
            self.accepted_count += 1;
        }

        match record.outcome {
            FixOutcome::Success => self.success_count += 1,
            FixOutcome::Failure => self.failure_count += 1,
            _ => {}
        }

        // 重新计算率
        self.acceptance_rate = self.accepted_count as f64 / self.total_uses as f64;

        let outcome_count = self.success_count + self.failure_count;
        self.success_rate = if outcome_count > 0 {
            self.success_count as f64 / outcome_count as f64
        } else {
            0.5 // 未知时给中等分数
        };

        // 效果得分 = 0.4 * 采纳率 + 0.6 * 成功率
        self.effectiveness_score = 0.4 * self.acceptance_rate + 0.6 * self.success_rate;
    }
}

/// 反馈学习系统
pub struct FeedbackLearner {
    /// 反馈记录（内存）
    records: Arc<RwLock<Vec<FeedbackRecord>>>,

    /// 策略统计（内存）
    strategy_stats: Arc<RwLock<HashMap<String, StrategyStats>>>,

    /// 持久化路径
    storage_path: Option<PathBuf>,

    /// 最大记录数
    max_records: usize,
}

impl FeedbackLearner {
    /// 创建新的学习器
    pub fn new() -> Self {
        Self {
            records: Arc::new(RwLock::new(Vec::new())),
            strategy_stats: Arc::new(RwLock::new(HashMap::new())),
            storage_path: None,
            max_records: 1000,
        }
    }

    /// 设置持久化路径
    pub fn with_storage(mut self, path: PathBuf) -> Self {
        self.storage_path = Some(path);
        self
    }

    /// 设置最大记录数
    pub fn with_max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }

    /// 记录反馈
    ///
    /// # Arguments
    /// * `record` - 反馈记录
    pub async fn record_feedback(&self, record: FeedbackRecord) {
        let strategy_name = record.strategy_name.clone();

        // 添加记录
        {
            let mut records = self.records.write().await;
            records.push(record.clone());

            // 限制记录数（保留最新的）
            let len = records.len();
            if len > self.max_records {
                records.drain(0..len - self.max_records);
            }
        }

        // 更新统计
        {
            let mut stats = self.strategy_stats.write().await;
            let strategy_stat = stats
                .entry(strategy_name.clone())
                .or_insert_with(|| StrategyStats::new(strategy_name));

            strategy_stat.update(&record);
        }

        // 持久化（异步，不等待）
        if let Some(ref path) = self.storage_path {
            let path = path.clone();
            let records = self.records.clone();
            let stats = self.strategy_stats.clone();

            tokio::spawn(async move {
                let _ = Self::save_to_disk(&path, records, stats).await;
            });
        }
    }

    /// 获取策略统计
    pub async fn get_strategy_stats(&self, strategy_name: &str) -> Option<StrategyStats> {
        let stats = self.strategy_stats.read().await;
        stats.get(strategy_name).cloned()
    }

    /// 获取所有策略统计
    pub async fn get_all_stats(&self) -> Vec<StrategyStats> {
        let stats = self.strategy_stats.read().await;
        let mut result: Vec<_> = stats.values().cloned().collect();

        // 按效果得分排序（降序）
        result.sort_by(|a, b| {
            b.effectiveness_score
                .partial_cmp(&a.effectiveness_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        result
    }

    /// 根据效果得分排序策略
    ///
    /// # Arguments
    /// * `strategies` - 原始策略列表
    ///
    /// # Returns
    /// 按学习到的效果得分重新排序的策略列表
    pub async fn rerank_strategies(&self, mut strategies: Vec<FixStrategy>) -> Vec<FixStrategy> {
        let stats = self.strategy_stats.read().await;

        // 按效果得分排序
        strategies.sort_by(|a, b| {
            let score_a = stats
                .get(&a.name)
                .map(|s| s.effectiveness_score)
                .unwrap_or(0.5);

            let score_b = stats
                .get(&b.name)
                .map(|s| s.effectiveness_score)
                .unwrap_or(0.5);

            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        strategies
    }

    /// 获取最近的反馈记录
    pub async fn get_recent_records(&self, limit: usize) -> Vec<FeedbackRecord> {
        let records = self.records.read().await;
        let start = if records.len() > limit {
            records.len() - limit
        } else {
            0
        };

        records[start..].to_vec()
    }

    /// 获取特定错误模式的统计
    pub async fn get_pattern_stats(&self, pattern: &str) -> PatternStats {
        let records = self.records.read().await;

        let pattern_records: Vec<_> = records
            .iter()
            .filter(|r| r.error_pattern == pattern)
            .collect();

        let total = pattern_records.len();
        if total == 0 {
            return PatternStats::default();
        }

        let successful = pattern_records.iter().filter(|r| r.is_positive()).count();

        let strategy_distribution: HashMap<String, u32> = pattern_records.iter().fold(
            HashMap::new(),
            |mut acc, r| {
                *acc.entry(r.strategy_name.clone()).or_insert(0) += 1;
                acc
            },
        );

        PatternStats {
            total_occurrences: total,
            successful_fixes: successful,
            success_rate: successful as f64 / total as f64,
            common_strategies: strategy_distribution,
        }
    }

    /// 从磁盘加载数据
    pub async fn load_from_disk(&self) -> Result<(), std::io::Error> {
        if let Some(ref path) = self.storage_path {
            if !path.exists() {
                return Ok(());
            }

            let content = tokio::fs::read_to_string(path).await?;
            let data: StorageData = serde_json::from_str(&content)?;

            *self.records.write().await = data.records;
            *self.strategy_stats.write().await = data.strategy_stats;
        }

        Ok(())
    }

    /// 保存到磁盘（内部方法）
    async fn save_to_disk(
        path: &Path,
        records: Arc<RwLock<Vec<FeedbackRecord>>>,
        stats: Arc<RwLock<HashMap<String, StrategyStats>>>,
    ) -> Result<(), std::io::Error> {
        let data = StorageData {
            records: records.read().await.clone(),
            strategy_stats: stats.read().await.clone(),
        };

        let content = serde_json::to_string_pretty(&data)?;

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(path, content).await?;

        Ok(())
    }

    /// 清除所有数据
    pub async fn clear(&self) {
        self.records.write().await.clear();
        self.strategy_stats.write().await.clear();
    }

    /// 获取统计摘要
    pub async fn get_summary(&self) -> LearningSummary {
        let records = self.records.read().await;
        let stats = self.strategy_stats.read().await;

        let total_records = records.len();
        let positive_count = records.iter().filter(|r| r.is_positive()).count();

        let top_strategies: Vec<_> = {
            let mut all_stats: Vec<_> = stats.values().cloned().collect();
            all_stats.sort_by(|a, b| {
                b.effectiveness_score
                    .partial_cmp(&a.effectiveness_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            all_stats.into_iter().take(5).collect()
        };

        LearningSummary {
            total_feedbacks: total_records,
            positive_feedbacks: positive_count,
            overall_success_rate: if total_records > 0 {
                positive_count as f64 / total_records as f64
            } else {
                0.0
            },
            total_strategies: stats.len(),
            top_strategies,
        }
    }
}

impl Default for FeedbackLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// 错误模式统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternStats {
    /// 总出现次数
    pub total_occurrences: usize,

    /// 成功修复次数
    pub successful_fixes: usize,

    /// 成功率
    pub success_rate: f64,

    /// 常用策略分布
    pub common_strategies: HashMap<String, u32>,
}

/// 学习摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSummary {
    /// 总反馈数
    pub total_feedbacks: usize,

    /// 正面反馈数
    pub positive_feedbacks: usize,

    /// 总体成功率
    pub overall_success_rate: f64,

    /// 总策略数
    pub total_strategies: usize,

    /// Top 5 策略
    pub top_strategies: Vec<StrategyStats>,
}

/// 持久化数据结构
#[derive(Debug, Serialize, Deserialize)]
struct StorageData {
    records: Vec<FeedbackRecord>,
    strategy_stats: HashMap<String, StrategyStats>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_fixer::{ErrorAnalysis, FixStrategy};

    #[test]
    fn test_feedback_record_creation() {
        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test", "fix", "desc", 3);

        let record = FeedbackRecord::new(&analysis, &strategy, FeedbackType::Accepted, FixOutcome::Success);

        assert_eq!(record.strategy_name, "test");
        assert_eq!(record.feedback, FeedbackType::Accepted);
        assert_eq!(record.outcome, FixOutcome::Success);
        assert!(record.is_positive());
    }

    #[test]
    fn test_feedback_record_negative() {
        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test", "fix", "desc", 3);

        let record = FeedbackRecord::new(&analysis, &strategy, FeedbackType::Rejected, FixOutcome::Failure);

        assert!(!record.is_positive());
    }

    #[test]
    fn test_strategy_stats_update() {
        let mut stats = StrategyStats::new("test".to_string());
        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test", "fix", "desc", 3);

        // 第一次：接受并成功
        let record1 = FeedbackRecord::new(&analysis, &strategy, FeedbackType::Accepted, FixOutcome::Success);
        stats.update(&record1);

        assert_eq!(stats.total_uses, 1);
        assert_eq!(stats.accepted_count, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.acceptance_rate, 1.0);
        assert_eq!(stats.success_rate, 1.0);

        // 第二次：拒绝并失败
        let record2 = FeedbackRecord::new(&analysis, &strategy, FeedbackType::Rejected, FixOutcome::Failure);
        stats.update(&record2);

        assert_eq!(stats.total_uses, 2);
        assert_eq!(stats.accepted_count, 1);
        assert_eq!(stats.failure_count, 1);
        assert_eq!(stats.acceptance_rate, 0.5);
        assert_eq!(stats.success_rate, 0.5);
    }

    #[tokio::test]
    async fn test_feedback_learner_basic() {
        let learner = FeedbackLearner::new();

        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test_strategy", "fix", "desc", 3);
        let record = FeedbackRecord::new(&analysis, &strategy, FeedbackType::Accepted, FixOutcome::Success);

        learner.record_feedback(record).await;

        let stats = learner.get_strategy_stats("test_strategy").await;
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.total_uses, 1);
        assert_eq!(stats.success_count, 1);
    }

    #[tokio::test]
    async fn test_rerank_strategies() {
        let learner = FeedbackLearner::new();

        // 创建两个策略
        let strategy1 = FixStrategy::new("low_score", "cmd1", "desc1", 3);
        let strategy2 = FixStrategy::new("high_score", "cmd2", "desc2", 3);

        // 给 strategy2 更好的反馈
        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());

        for _ in 0..3 {
            let record = FeedbackRecord::new(&analysis, &strategy2, FeedbackType::Accepted, FixOutcome::Success);
            learner.record_feedback(record).await;
        }

        let record = FeedbackRecord::new(&analysis, &strategy1, FeedbackType::Rejected, FixOutcome::Failure);
        learner.record_feedback(record).await;

        // 重新排序
        let strategies = vec![strategy1.clone(), strategy2.clone()];
        let ranked = learner.rerank_strategies(strategies).await;

        // high_score 应该排在前面
        assert_eq!(ranked[0].name, "high_score");
        assert_eq!(ranked[1].name, "low_score");
    }

    #[tokio::test]
    async fn test_get_summary() {
        let learner = FeedbackLearner::new();

        let analysis = ErrorAnalysis::new("error".to_string(), "cmd".to_string());
        let strategy = FixStrategy::new("test", "fix", "desc", 3);

        for i in 0..5 {
            let outcome = if i < 3 {
                FixOutcome::Success
            } else {
                FixOutcome::Failure
            };
            let feedback = if i < 3 {
                FeedbackType::Accepted
            } else {
                FeedbackType::Rejected
            };

            let record = FeedbackRecord::new(&analysis, &strategy, feedback, outcome);
            learner.record_feedback(record).await;
        }

        let summary = learner.get_summary().await;

        assert_eq!(summary.total_feedbacks, 5);
        assert_eq!(summary.positive_feedbacks, 3);
        assert_eq!(summary.overall_success_rate, 0.6);
    }
}
