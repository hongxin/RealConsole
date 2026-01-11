//! 工具函数模块
//!
//! 提供项目中常用的工具函数，包括：
//! - 字符串处理（安全截断等）
//! - 软阈值工具（连续场重构）
//! - 延迟初始化工具（v1.89.0）

pub mod lazy_init; // ✨ v1.89.0: 延迟初始化工具
pub mod soft_threshold;
pub mod string;

// 重新导出常用函数
pub use lazy_init::{LazyInit, LazyInitError, LazyStatsSnapshot, StartupReport, StartupReports, StartupTimer};
pub use string::{truncate_chars, truncate_safe, truncate_smart};
