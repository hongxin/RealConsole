//! 统一追踪系统
//!
//! 提供四维观测体系的统一接口，聚合 History、log、llm-log、Context 四个数据源
//!
//! # 设计理念
//!
//! 基于"四象"哲学理论，将系统的观测维度抽象为四个互补视角：
//!
//! - **📊 Statistics (统计维度)**: 宏观规律，命令频率分析
//! - **🔗 Coordination (协同维度)**: 端到端追踪，任务协同
//! - **🤖 BlackBox (黑盒维度)**: LLM 透视，模型行为观测
//! - **💭 Memory (记忆维度)**: 对话连贯，上下文延续
//!
//! # 模块结构
//!
//! - `types`: 核心类型定义（Dimension, EntryType, Status）
//! - `entry`: 统一追踪条目（TraceEntry）
//! - `unified_tracer`: 统一追踪器（UnifiedTracer，Phase 3 实现）
//!
//! # 快速开始
//!
//! ```rust
//! use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
//!
//! // 创建追踪条目
//! let entry = TraceEntry::new(
//!     Dimension::Statistics,
//!     EntryType::ShellCommand,
//!     "ls -la".to_string(),
//!     Status::Success,
//! );
//!
//! // 格式化输出
//! println!("{}", entry.format());
//! ```
//!
//! # 相关文档
//!
//! - `docs/04-reports/trace-command-design.md` - 详细设计文档
//! - `docs/04-reports/trace-implementation-plan.md` - 实施计划
//! - `docs/04-reports/four-dimensions-philosophy.md` - 哲学理论基础

pub mod entry;
pub mod types;
pub mod unified_tracer;

#[cfg(test)]
mod benchmarks;

// 重新导出核心类型，方便使用
pub use entry::TraceEntry;
pub use types::{Dimension, EntryType, Status};
pub use unified_tracer::{TraceStats, UnifiedTracer};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 测试核心类型可以正常导入
        let _dim = Dimension::Statistics;
        let _entry_type = EntryType::ShellCommand;
        let _status = Status::Success;
        let _entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test".to_string(),
            Status::Success,
        );
    }
}
