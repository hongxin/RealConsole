//! Memory 模块
//!
//! 提供上下文追踪和记忆管理功能
//!
//! Phase 9.1: 上下文理解增强
//! v1.16.0 Phase 3: MemoryManager 适配层（统一到 UnifiedTracer）
//! v1.107.0: Memory Storage 迁移（Storage Layer 2.0 集成）

pub mod context_tracker;
pub mod manager; // v1.16.0 Phase 3
pub mod memory_core;
pub mod migration; // v1.16.0 Phase 3
pub mod storage_adapter; // v1.107.0: Storage Layer 集成

// 导出 memory_core 的所有公共类型
#[allow(unused_imports)]
pub use memory_core::MemoryEntry;
pub use memory_core::{EntryType, Importance, Memory, MemoryStats};

// 导出 context_tracker 的公共类型
#[allow(unused_imports)]
pub use context_tracker::{ContextStats, Entity, EntityType, ReferenceRecord, WorkingContext};
pub use context_tracker::{ContextTracker, WorkingContextUpdate};

// v1.16.0 Phase 3: 导出 MemoryManager 和迁移工具
pub use manager::MemoryManager;
pub use migration::{MemoryMigrator, MigrationReport};

// v1.107.0: 导出 Storage Layer 集成
pub use storage_adapter::{
    MemoryAdapterStats, MemoryAsStorage, MemorySnapshot, MemoryStorageAdapter,
    MemoryStorageConfig, PersistFormat, SnapshotMetadata,
};
