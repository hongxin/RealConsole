//! 错误自动修复模块 - Phase 9.1 Week 2-3
//!
//! 功能：
//! - 错误模式识别（命令不存在、权限错误、语法错误等）
//! - 错误分类与分析
//! - LLM 生成修复建议
//! - 安全的自动修复应用
//! - 用户反馈学习（Week 3）
//!
//! 设计理念（一分为三）：
//! - 识别层：快速识别常见错误模式
//! - 分析层：深度分析错误原因和上下文
//! - 修复层：生成并应用安全的修复方案
//! - 学习层（Week 3）：从用户反馈中学习和优化

pub mod analyzer;
pub mod feedback;
pub mod fixer;
pub mod patterns;

pub use analyzer::{ErrorAnalysis, ErrorAnalyzer, ErrorCategory, ErrorSeverity};
pub use feedback::{
    FeedbackLearner, FeedbackRecord, FeedbackType, FixOutcome, LearningSummary, PatternStats,
    StrategyStats,
};
pub use fixer::{ErrorFixer, FixResult, FixStrategy};
pub use patterns::ErrorPattern;
