//! 用户反馈学习系统
//!
//! 基于"一分为三"哲学的智能反馈学习系统：
//! - **收集层（Collector）**: 记录用户行为
//! - **存储层（Storage）**: 持久化反馈数据
//! - **学习层（Learner）**: 分析并优化建议
//!
//! ## 设计理念
//!
//! ### 三态反馈
//! ```text
//! 接受（Accepted）   →  积极信号，提升评分
//! 跳过（Skipped）    →  中性信号，保持评分
//! 拒绝（Rejected）   →  消极信号，降低评分（未来）
//! ```ignore
//!
//! ### 三层学习
//! ```text
//! 即时学习（Instant）     →  单次反馈立即调整
//! 短期学习（Short-term）  →  最近 N 次反馈的模式
//! 长期学习（Long-term）   →  历史数据的趋势分析
//! ```ignore
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use realconsole::suggestion::feedback::{FeedbackCollector, FeedbackLearner};
//! use realconsole::suggestion::{Suggestion, SuggestionContext};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // 创建收集器
//! let collector = FeedbackCollector::new("~/.realconsole/feedback")?;
//!
//! // 记录建议展示
//! let suggestions = vec![/* ... */];
//! let context = SuggestionContext::from_env();
//! let feedback_id = collector.record_suggestion_shown(&suggestions, &context).await?;
//!
//! // 记录用户选择
//! collector.record_selection(&feedback_id, 0).await?;
//!
//! // 使用学习器调整评分
//! let learner = FeedbackLearner::new(collector);
//! let adjusted_score = learner.adjust_score(&suggestions[0], &context).await;
//! # Ok(())
//! # }
//! ```ignore

mod collector;
mod learner;
mod storage;
mod types;

pub use collector::FeedbackCollector;
pub use learner::FeedbackLearner;
pub use storage::{FeedbackStorage, StorageInfo};
pub use types::{
    FeedbackContext, FeedbackRecord, FeedbackType, LearningConfig, SuggestionFeedback,
    SuggestionStats,
};
