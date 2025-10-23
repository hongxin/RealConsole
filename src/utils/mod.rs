//! 工具函数模块
//!
//! 提供项目中常用的工具函数，包括：
//! - 字符串处理（安全截断等）
//! - 软阈值工具（连续场重构）

pub mod soft_threshold;
pub mod string;

// 重新导出常用函数
pub use string::{truncate_chars, truncate_safe, truncate_smart};
