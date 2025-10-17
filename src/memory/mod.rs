//! Memory 模块
//!
//! 提供上下文追踪和记忆管理功能
//!
//! Phase 9.1: 上下文理解增强

pub mod context_tracker;
pub mod memory_core;

// 导出 memory_core 的所有公共类型
pub use memory_core::{EntryType, Memory, MemoryEntry};

// 导出 context_tracker 的公共类型
pub use context_tracker::{
    ContextStats, ContextTracker, Entity, EntityType, ReferenceRecord, WorkingContext,
    WorkingContextUpdate,
};
