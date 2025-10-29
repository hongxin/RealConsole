//! 错误自动修复模块
//!
//! ## Phase 9.1 (已完成)
//! - 错误模式识别（命令不存在、权限错误、语法错误等）
//! - 错误分类与分析
//! - LLM 生成修复建议
//! - 安全的自动修复应用
//! - 用户反馈学习
//!
//! ## Phase 2 / v1.16.0 (新增)
//! - 多维度建议系统（相关性、可行性、安全性、学习价值）
//! - 风险评估系统（4级风险分类）
//! - 下一步预测引擎
//!
//! 设计理念（一分为三）：
//! - 识别层：快速识别常见错误模式
//! - 分析层：深度分析错误原因和上下文
//! - 修复层：生成并应用安全的修复方案
//! - 学习层：从用户反馈中学习和优化

// Phase 9.1 模块
pub mod analyzer;
pub mod feedback;
pub mod fixer;
pub mod patterns;

// Phase 2 新增模块
pub mod predictor;
pub mod risk;
pub mod suggestion;

// Phase 9.1 导出
pub use analyzer::{ErrorAnalysis, ErrorAnalyzer, ErrorCategory, ErrorSeverity};
#[allow(unused_imports)]
pub use feedback::{
    FeedbackLearner, FeedbackRecord, FeedbackType, FixOutcome, LearningSummary, PatternStats,
    StrategyStats,
};
pub use fixer::{ErrorFixer, FixResult, FixStrategy};
#[allow(unused_imports)]
pub use patterns::ErrorPattern;

// Phase 2 新增导出
pub use predictor::{NextStep, NextStepCategory, NextStepPredictor, ProbabilityLevel};
pub use risk::{RiskAssessment, RiskAssessor, RiskFactor, RiskFactorType, RiskLevel};
pub use suggestion::{ScoreLevel, SortDimension, Suggestion, SuggestionList};
