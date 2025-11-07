//! 意图拆解模块
//!
//! 将自然语言意图拆解为可执行的步骤计划，并提供执行引擎。
//!
//! # 核心组件
//!
//! - [`IntentDecomposer`] - 意图拆解器，将自然语言转换为执行计划
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

pub use types::{ExecutionPlan, ExecutionStep, StepStatus, StepProgress, ExecutionResult};
pub use decomposer::IntentDecomposer;
pub use executor::PlanExecutor;
