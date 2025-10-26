//! 主动建议系统
//!
//! 基于"一分为三"哲学的智能建议系统，融合三种建议来源：
//! - Context：基于项目类型和当前上下文
//! - History：基于用户命令历史
//! - LLM：基于 AI 智能推理
//!
//! ## 架构
//!
//! ```text
//!                    SuggestionEngine
//!                           |
//!           +---------------+---------------+
//!           |               |               |
//!     ContextSuggester  HistorySuggester  LlmSuggester
//!           |               |               |
//!           +---------------+---------------+
//!                           |
//!                   SuggestionRanker
//!                           |
//!                    Ranked Suggestions
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use realconsole::suggestion::{SuggestionEngine, SuggestionConfig, SuggestionContext};
//! use realconsole::history::HistoryManager;
//! use std::sync::Arc;
//! use tokio::sync::RwLock;
//!
//! # async fn example() {
//! let history = Arc::new(RwLock::new(HistoryManager::new("history.json", 100)));
//! let config = SuggestionConfig::default();
//!
//! let engine = SuggestionEngine::new(history, config);
//!
//! let context = SuggestionContext::from_env();
//! let suggestions = engine.suggest(&context).await;
//!
//! for suggestion in suggestions {
//!     println!("{} - {}", suggestion.command, suggestion.description);
//! }
//! # }
//! ```

mod context_suggester;
mod engine;
mod error_patterns; // ✨ Phase 4.2: 错误模式识别
mod history_suggester;
mod llm_suggester;
mod ranker;
mod types;

// 公开主要接口
pub use context_suggester::ContextSuggester;
pub use engine::SuggestionEngine;
pub use history_suggester::HistorySuggester;
pub use llm_suggester::LlmSuggester;
pub use ranker::SuggestionRanker;
pub use types::{
    FileType, Suggestion, SuggestionCategory, SuggestionConfig, SuggestionContext,
    SuggestionSource, SuggestionTrigger,
};
