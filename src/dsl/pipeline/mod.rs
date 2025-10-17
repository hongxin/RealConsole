//! Pipeline DSL - 组合式命令执行计划
//!
//! **设计哲学**：从"枚举"到"组合"，从"数"到"变"
//!
//! ## 核心思想
//!
//! 不是预定义所有可能的命令，而是定义基础操作（象）和参数（爻），
//! 通过组合生成无穷多的命令变体。
//!
//! ## 易经映射
//!
//! - **象（不变）**：基础操作（FindFiles, SortFiles, LimitFiles）
//! - **爻（变化）**：参数（排序方向、文件类型、数量）
//! - **卦（组合）**：执行计划（操作的组合）
//!
//! ## 示例
//!
//! ```rust
//! use realconsole::dsl::pipeline::{ExecutionPlan, BaseOperation, Field, Direction};
//!
//! // 查找最小的 rs 文件
//! let plan = ExecutionPlan {
//!     operations: vec![
//!         BaseOperation::FindFiles {
//!             path: ".".to_string(),
//!             pattern: "*.rs".to_string(),
//!         },
//!         BaseOperation::SortFiles {
//!             field: Field::Size,
//!             direction: Direction::Ascending,  // 最小 = 升序
//!         },
//!         BaseOperation::LimitFiles {
//!             count: 1,
//!         },
//!     ],
//! };
//!
//! let command = plan.to_shell_command();
//! // → "find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -h | head -n 1"
//! ```

pub mod operations;
pub mod plan;

pub use operations::{BaseOperation, Direction, Field};
pub use plan::ExecutionPlan;
