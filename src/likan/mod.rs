//! 离坎炼化炉 - 自主学习循环
//!
//! ## 哲学基础
//!
//! 离坎二卦"别有深意"，是系统的反熵机制：
//! - **坎 ☵（水）**：向下，汇聚，沉淀 → 提取深层模式
//! - **离 ☲（火）**：向上，挥发，照亮 → 输出优化建议
//!
//! ## 设计原则
//!
//! **简易**：最小实现，专注核心
//! **变易**：保留演化空间，不追求完美
//! **不易**：循环本质，永续动力
//!
//! ## 实现策略
//!
//! 顺势而为，利用现有系统：
//! - 坎：从 Tracer + Feedback 提取
//! - 离：增强 Suggestion 输出
//! - 循环：自主触发，持续优化

pub mod types;
pub mod kan;
pub mod li;
pub mod furnace;
pub mod statusbar;
pub mod trigger;

pub use furnace::LiKanFurnace;
pub use kan::KanExtractor;
pub use li::LiEnhancer;
pub use statusbar::{FurnaceStatus, LiKanStatusBar};
pub use trigger::LiKanTrigger;
pub use types::*;
