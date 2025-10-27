//! 八卦记忆宫殿（Bagua Memory Palace）
//!
//! ## 哲学基础
//!
//! 基于易经八卦的八维记忆空间：
//! - 乾 ☰: 意图目标 (Intent/Goal)
//! - 坤 ☷: 原始数据 (Raw Data)
//! - 震 ☳: 触发行动 (Action)
//! - 巽 ☴: 趋势变化 (Trend)
//! - 坎 ☵: 深层模式 (Pattern) ⭐ 离坎核心
//! - 离 ☲: 显性知识 (Knowledge) ⭐ 离坎核心
//! - 艮 ☶: 状态检查 (Checkpoint)
//! - 兑 ☱: 交互反馈 (Feedback)
//!
//! ## 设计理念
//!
//! **一分为三 + 八卦 + 64卦**：
//! - 一 → 阴阳中（三态）
//! - 三 → 八卦（八维）
//! - 八 × 八 → 六十四卦（64 态）
//!
//! ## 实现策略
//!
//! **极简主义**：
//! - Phase 1: 八维基础（当前）
//! - Phase 2: 离坎循环（已有 likan 模块）
//! - Phase 3: Suggest 集成
//! - Phase 4: 卦象观测
//! - Phase 5: 可视化

pub mod dimension;
pub mod entry;
pub mod palace;
pub mod storage; // ✨ v1.8.4 Phase 4: 持久化存储

pub use dimension::BaguaDimension;
pub use entry::{MemoryContent, MemoryEntry};
pub use palace::BaguaMemoryPalace;
pub use storage::{BaguaStorage, StorageStats};
