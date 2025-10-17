//! RealConsole Library
//!
//! An intelligent CLI agent combining Eastern philosophy wisdom with modern AI.
//!
//! This library exposes the core components for building CLI agents with:
//! - LLM integration (Ollama, Deepseek, OpenAI)
//! - Function Calling support
//! - Tool system with execution engine
//! - Type system for DSL
//! - Memory and execution logging

// Public modules
pub mod advanced_tools;
pub mod agent;
pub mod builtin_tools;
pub mod command;
pub mod command_router;    // ✨ Phase 10.1: 智能命令路由系统
pub mod commands;
pub mod config;
pub mod conversation;    // ✨ Phase 8 Week 2: 多轮对话支持
pub mod display;
pub mod dsl;
pub mod error;
pub mod error_fixer;        // ✨ Phase 9.1 Week 2: 错误自动修复
pub mod execution_logger;
pub mod git_assistant;     // ✨ Phase 6: Git 智能助手
pub mod history;           // ✨ Phase 8: 命令历史记录管理
pub mod llm;
pub mod llm_manager;
pub mod log_analyzer;      // ✨ Phase 6: 日志分析工具
pub mod memory;
pub mod project_context;   // ✨ Phase 6: 项目上下文感知
pub mod shell_executor;
pub mod spinner;
pub mod stats;             // ✨ Phase 9: 统计与可视化系统
pub mod system_monitor;    // ✨ Phase 6: 系统监控工具
pub mod task;              // ✨ Phase 10: 任务分解与规划系统
pub mod tool;
pub mod tool_cache;        // ✨ Week 3 Day 2: 工具缓存系统
pub mod tool_executor;
pub mod wizard;

// Re-export commonly used types
pub use agent::Agent;
pub use config::Config;
pub use display::{Display, DisplayMode};
pub use error::{ErrorCode, FixSuggestion, RealError};
pub use error_fixer::{
    ErrorAnalysis, ErrorAnalyzer, ErrorCategory, ErrorSeverity, FeedbackLearner, FeedbackRecord,
    FeedbackType, FixOutcome, FixStrategy, LearningSummary,
};
pub use llm::{ChatResponse, FunctionCall, LlmClient, LlmError, Message, ToolCall};
pub use shell_executor::{ExecutionResult, ShellExecutorWithFixer};
pub use task::{
    ExecutionContext, ExecutionPlan, PlanAnalysis, ProgressCallback, SubTask, TaskDecomposer,
    TaskError, TaskExecutor, TaskPlanner, TaskProgress, TaskResult, TaskStatus, TaskType,
};
pub use tool::{Parameter, ParameterType, Tool, ToolRegistry};
pub use tool_executor::{ExecutionMode, ToolCallRequest, ToolCallResult, ToolExecutor};
