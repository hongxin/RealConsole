//! 统计模块
//!
//! ✨ Phase 9: 系统可视化基础
//!
//! 功能：
//! - 实时统计收集（LLM、工具、命令）
//! - 系统仪表板显示
//! - 性能指标追踪

pub mod collector;
pub mod dashboard;
pub mod metrics;

pub use collector::{StatsCollector, StatEvent};
pub use dashboard::Dashboard;
