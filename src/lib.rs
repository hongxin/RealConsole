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

// Allow unused code for library exports and future use
#![allow(dead_code)]
#![allow(unused_imports)]

// Public modules
pub mod advanced_tools;
pub mod agent;
pub mod bagua; // ✨ Phase 4.4: 八卦记忆宫殿（八维知识图谱）
pub mod builtin_tools;
pub mod command;
pub mod command_router; // ✨ Phase 10.1: 智能命令路由系统
pub mod commands;
pub mod completion; // ✨ Phase 1: Tab 补全系统（静态补全）
pub mod config;
pub mod conversation; // ✨ Phase 8 Week 2: 多轮对话支持
pub mod display;
pub mod display_helper; // 显示辅助工具（emoji/纯文本切换）
pub mod dsl;
pub mod error;
pub mod error_formatter; // ✨ v1.84.0: 增强错误消息格式化
pub mod error_fixer; // ✨ Phase 9.1 Week 2: 错误自动修复
pub mod execution_logger;
pub mod git_assistant; // ✨ Phase 6: Git 智能助手
pub mod history; // ✨ Phase 8: 命令历史记录管理
pub mod i18n; // ✨ Phase 11: 多语言支持
pub mod likan; // ✨ Phase 4.3: 离坎炼化炉（自主学习循环）
pub mod liangyyi; // ✨ v1.9.0: 两仪演化系统（先天八卦·竖看·体）
pub mod llm;
pub mod llm_manager;
pub mod log_analyzer; // ✨ Phase 6: 日志分析工具
pub mod lunar_tool; // ✨ 农历工具：公历/农历转换、节气、干支生肖
pub mod markdown_renderer; // ✨ v1.25.0: Markdown 渲染器 - LLM 输出美化
pub mod memory;
pub mod path_resolver; // ✨ UX 改进：统一的配置文件路径搜索
pub mod progress; // ✨ v1.86.0: 统一进度指示器（进度条、计时、多任务）
pub mod project_context; // ✨ Phase 6: 项目上下文感知
pub mod recovery; // ✨ v1.91.0: 优雅错误恢复（健康检查、熔断、恢复编排）
pub mod repl; // REPL 循环（包含 Tab 补全集成）
pub mod resource; // ✨ v1.92.0: 资源管理（内存监控、自动清理）
pub mod services; // ✨ Phase 2: 服务层架构（v1.3.0）
pub mod shell_executor;
pub mod spinner;
pub mod storage; // ✨ v1.58.0: 存储抽象层（FileStorage, MemoryStorage）
pub mod stats; // ✨ Phase 9: 统计与可视化系统
pub mod suggestion; // ✨ Phase 4.1: 主动建议系统（三源融合）
pub mod system_monitor; // ✨ Phase 6: 系统监控工具
pub mod task; // ✨ Phase 10: 任务分解与规划系统
pub mod tool;
pub mod tool_cache; // ✨ Week 3 Day 2: 工具缓存系统
pub mod tool_executor;
pub mod trace_context; // ✨ v1.5.1: 追踪上下文（TraceContext, ExecutionSpan）
pub mod tracer; // ✨ Phase 2 (Memory Redesign): 统一追踪系统（四维观测）
pub mod utils; // ✨ Phase 2: 软阈值工具（连续场重构） + 字符串处理等通用工具
pub mod voice; // ✨ 语音播报系统
pub mod visualization; // ✨ v1.44.0: 可视化系统（数据图表、Notebook）
pub mod web; // ✨ v1.23.0: Web 终端（浏览器访问）
pub mod wizard;

// Re-export commonly used types
pub use agent::Agent;
pub use completion::{
    Candidate, CompletionCache, CompletionConfig, CompletionContext, CompletionSource,
    CompletionType, IntelligentCompleter, MultiDimensionalCompleter, SemanticCompleter,
    StaticCompleter,
};
pub use config::Config;
pub use display::{Display, DisplayMode};
pub use error::{ErrorCode, FixSuggestion, RealError};
pub use error_formatter::ErrorFormatter;
pub use error_fixer::{
    ErrorAnalysis, ErrorAnalyzer, ErrorCategory, ErrorSeverity, FeedbackLearner, FeedbackRecord,
    FeedbackType, FixOutcome, FixStrategy, LearningSummary,
};
pub use likan::{
    CycleReport, FurnaceConfig, KanExtractor, LiEnhancer, LiKanFurnace, Pattern,
};
pub use llm::{
    CachedLlmClient, CacheWarmer, ChatResponse, FunctionCall, LlmCacheConfig, LlmCacheStats,
    LlmClient, LlmError, Message, ToolCall,
};
pub use progress::{
    MultiProgressManager, ProgressConfig, ProgressIndicator, ProgressTask, ProgressType,
    with_progress, with_progress_iter,
};
pub use shell_executor::{ExecutionResult, ShellExecutorWithFixer};
pub use suggestion::{
    ContextSuggester, FileType, HistorySuggester, LlmSuggester, Suggestion, SuggestionCategory,
    SuggestionConfig, SuggestionContext, SuggestionEngine, SuggestionRanker, SuggestionSource,
    SuggestionTrigger,
};
pub use task::{
    ExecutionContext, ExecutionPlan, PlanAnalysis, ProgressCallback, SubTask, TaskDecomposer,
    TaskError, TaskExecutor, TaskPlanner, TaskProgress, TaskResult, TaskStatus, TaskType,
};
pub use tool::{Parameter, ParameterType, Tool, ToolRegistry};
pub use tool_executor::{
    ConversationRound, ExecutionMode, ToolCallRequest, ToolCallResult, ToolCallSummary,
    ToolExecutor,
};
