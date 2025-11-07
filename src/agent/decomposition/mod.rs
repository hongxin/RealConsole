//! 意图拆解模块
//!
//! 将自然语言意图拆解为可执行的步骤计划，并提供执行引擎。
//!
//! # 核心组件
//!
//! - [`IntentDecomposer`] - 意图拆解器，将自然语言转换为执行计划
//! - [`IntentRouter`] - Intent 路由器，快速识别简单意图（v1.31.0）
//! - [`ToolRouter`] - 工具路由器，智能映射 Intent 到专用工具（v1.32.0）
//! - [`PlanExecutor`] - 计划执行器，顺序执行拆解后的步骤
//! - [`ExecutionPlan`] - 执行计划数据结构
//! - [`ExecutionStep`] - 单个执行步骤
//!
//! # 使用示例
//!
//! ```no_run
//! use realconsole::agent::decomposition::{IntentDecomposer, PlanExecutor};
//!
//! async fn example(llm: Arc<dyn LlmClient>) {
//!     // 1. 创建拆解器
//!     let decomposer = IntentDecomposer::new(llm);
//!
//!     // 2. 拆解意图
//!     let plan = decomposer.decompose("加载 data.csv 并显示前 10 行").await?;
//!
//!     // 3. 执行计划
//!     let executor = PlanExecutor::new(tool_executor);
//!     let result = executor.execute(plan).await?;
//! }
//! ```

pub mod types;
pub mod decomposer;
pub mod executor;
pub mod intent_router; // v1.31.0
pub mod tool_router; // v1.32.0

pub use types::{ExecutionPlan, ExecutionStep, StepStatus, StepProgress, ExecutionResult};
pub use decomposer::IntentDecomposer;
pub use executor::PlanExecutor;
pub use intent_router::IntentRouter; // v1.31.0
pub use tool_router::ToolRouter; // v1.32.0
