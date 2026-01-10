//! 存储抽象层
//!
//! v1.58.0: v2.0 探路期 - 验证 3-5x I/O 性能提升
//!
//! ## 设计目标
//!
//! 提供统一的存储抽象，支持多种后端实现：
//! - **FileStorage**: 文件系统存储（默认）
//! - **MemoryStorage**: 内存存储（测试/缓存）
//! - **SqliteStorage**: SQLite 存储（探索）
//!
//! ## 架构
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Application Layer             │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │         StorageBackend Trait            │
//! │  read / write / delete / list / exists  │
//! └─────────────────┬───────────────────────┘
//!                   │
//!     ┌─────────────┼─────────────┐
//!     │             │             │
//! ┌───▼───┐   ┌─────▼─────┐  ┌────▼────┐
//! │ File  │   │  Memory   │  │ SQLite  │
//! │Storage│   │  Storage  │  │ Storage │
//! └───────┘   └───────────┘  └─────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{StorageBackend, FileStorage, MemoryStorage};
//!
//! // 文件存储
//! let file_storage = FileStorage::new("~/.realconsole/data");
//! file_storage.write("key1", b"value1").await?;
//! let data = file_storage.read("key1").await?;
//!
//! // 内存存储（测试）
//! let mem_storage = MemoryStorage::new();
//! mem_storage.write("key1", b"value1").await?;
//! ```

mod batch;
mod cached;
mod file;
mod memory;
mod optimized;
pub mod tiered_cache;

pub use batch::{BatchWriter, BatchWriterConfig, DetailedBatchStats};
pub use cached::{CachedStorage, CachedStorageConfig, CombinedStorageStats};
pub use file::FileStorage;
pub use memory::MemoryStorage;
pub use optimized::{DetailedOptimizationStats, OptimizedStorage, OptimizedStorageConfig};
pub use tiered_cache::{CacheStats, CacheTier, DetailedCacheStats, TieredCache, TieredCacheConfig};

use async_trait::async_trait;
use std::fmt;

/// 存储错误类型
#[derive(Debug)]
pub enum StorageError {
    /// 键不存在
    NotFound(String),
    /// IO 错误
    Io(std::io::Error),
    /// 序列化错误
    Serialization(String),
    /// 其他错误
    Other(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound(key) => write!(f, "Key not found: {}", key),
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Serialization(s) => write!(f, "Serialization error: {}", s),
            StorageError::Other(s) => write!(f, "Storage error: {}", s),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            StorageError::NotFound(e.to_string())
        } else {
            StorageError::Io(e)
        }
    }
}

/// 存储结果类型
pub type StorageResult<T> = Result<T, StorageError>;

/// 存储后端 trait
///
/// 定义统一的存储接口，支持多种后端实现
///
/// # 设计原则
///
/// - **异步优先**: 所有操作都是异步的，支持高并发
/// - **键值模型**: 简单的键值存储，键为字符串，值为字节数组
/// - **命名空间**: 通过前缀支持命名空间隔离
///
/// # 实现要求
///
/// - `read`: 读取不存在的键应返回 `StorageError::NotFound`
/// - `write`: 写入应该是原子的（或尽可能原子）
/// - `delete`: 删除不存在的键应该静默成功
/// - `list`: 列出指定前缀的所有键
/// - `exists`: 检查键是否存在
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// 读取数据
    ///
    /// # 参数
    /// - `key`: 数据键
    ///
    /// # 返回
    /// - `Ok(Vec<u8>)`: 数据内容
    /// - `Err(StorageError::NotFound)`: 键不存在
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>>;

    /// 写入数据
    ///
    /// # 参数
    /// - `key`: 数据键
    /// - `data`: 数据内容
    ///
    /// # 返回
    /// - `Ok(())`: 写入成功
    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()>;

    /// 删除数据
    ///
    /// # 参数
    /// - `key`: 数据键
    ///
    /// # 返回
    /// - `Ok(())`: 删除成功（即使键不存在）
    async fn delete(&self, key: &str) -> StorageResult<()>;

    /// 列出指定前缀的所有键
    ///
    /// # 参数
    /// - `prefix`: 键前缀（空字符串表示列出所有）
    ///
    /// # 返回
    /// - `Ok(Vec<String>)`: 匹配的键列表
    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>>;

    /// 检查键是否存在
    ///
    /// # 参数
    /// - `key`: 数据键
    ///
    /// # 返回
    /// - `Ok(bool)`: 是否存在
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// 获取存储统计信息
    fn stats(&self) -> StorageStats;

    /// 获取存储名称
    fn name(&self) -> &'static str;
}

/// 存储统计信息
#[derive(Debug, Clone, Default)]
pub struct StorageStats {
    /// 读取次数
    pub reads: u64,
    /// 写入次数
    pub writes: u64,
    /// 删除次数
    pub deletes: u64,
    /// 命中次数（缓存）
    pub hits: u64,
    /// 未命中次数（缓存）
    pub misses: u64,
    /// 总数据量（字节）
    pub total_bytes: u64,
    /// 键数量
    pub key_count: usize,
}

impl StorageStats {
    /// 命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_error_display() {
        let not_found = StorageError::NotFound("key1".to_string());
        assert!(not_found.to_string().contains("not found"));

        let io_err = StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test error",
        ));
        assert!(io_err.to_string().contains("IO error"));
    }

    #[test]
    fn test_storage_stats_hit_rate() {
        let mut stats = StorageStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.hits = 80;
        stats.misses = 20;
        assert!((stats.hit_rate() - 0.8).abs() < 0.001);
    }
}
