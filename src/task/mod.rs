//! 任务分解与规划系统
//!
//! Phase 10: Task Decomposition & Planning System
//!
//! 本模块提供完整的任务分解、规划和执行功能，使 RealConsole 能够：
//! - 智能分解复杂任务为可执行的子任务序列
//! - 分析任务间的依赖关系
//! - 生成最优执行计划（串行/并行）
//! - 自动化执行任务并提供进度反馈
//!
//! # 核心组件
//!
//! - [`TaskDecomposer`] - 任务分解器，使用 LLM 智能分解任务
//! - [`TaskPlanner`] - 任务规划器，分析依赖并生成执行计划
//! - [`TaskExecutor`] - 任务执行器，执行计划并提供进度反馈
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use realconsole::task::{TaskDecomposer, TaskPlanner, TaskExecutor};
//!
//! // 1. 分解任务
//! let decomposer = TaskDecomposer::new(llm_client);
//! let subtasks = decomposer.decompose("部署应用", &context).await?;
//!
//! // 2. 规划执行
//! let planner = TaskPlanner::new();
//! let plan = planner.plan("部署应用", subtasks)?;
//!
//! // 3. 执行任务
//! let executor = TaskExecutor::new(shell_executor);
//! let result = executor.execute(plan).await?;
//! ```

pub mod decomposer;
pub mod error;
pub mod executor;
pub mod planner;
pub mod types;

// 重新导出核心类型
pub use decomposer::TaskDecomposer;
pub use executor::TaskExecutor;
#[allow(unused_imports)]
pub use executor::ProgressCallback;
pub use planner::TaskPlanner;
#[allow(unused_imports)]
pub use planner::PlanAnalysis;
pub use types::{ExecutionContext, ExecutionMode, ExecutionPlan, ExecutionResult, TaskStatus};
#[allow(unused_imports)]
pub use types::{
    DependencyGraph, ExecutionStage, RetryPolicy, SubTask, TaskProgress,
    TaskResult as TaskExecutionResult, TaskType,
};
#[allow(unused_imports)]
pub use error::{TaskError, TaskResult};
