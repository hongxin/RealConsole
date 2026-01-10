//! 异步批量写入器
//!
//! v1.61.0: v2.0 探路期 - 批量写入优化
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的写入优化架构：
//! - **缓冲层**: 内存中收集写入请求
//! - **调度层**: 基于策略决定何时刷新
//! - **执行层**: 批量写入后端存储
//!
//! ## 刷新策略
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                    BatchWriter                        │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Write Buffer:                                        │
//! │    [key1 → data1] [key2 → data2] [key3 → data3]      │
//! │                                                       │
//! │  Flush Triggers:                                      │
//! │    1. 缓冲区满 (buffer_size >= max_buffer_size)       │
//! │    2. 手动刷新 (flush())                              │
//! │    3. 删除操作触发 (确保一致性)                        │
//! │                                                       │
//! │  Benefits:                                            │
//! │    - 减少 I/O 次数 (N writes → 1 batch)              │
//! │    - 合并重复键写入 (只保留最新值)                    │
//! │    - 提高写入吞吐量                                   │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{BatchWriter, FileStorage, BatchWriterConfig};
//!
//! // 创建批量写入器
//! let backend = FileStorage::new("/path/to/data");
//! let writer = BatchWriter::new(backend);
//!
//! // 写入数据（缓冲）
//! writer.write("key1", b"value1").await?;
//! writer.write("key2", b"value2").await?;
//!
//! // 手动刷新
//! writer.flush().await?;
//!
//! // 或自动刷新（当缓冲区满时）
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// 批量写入器配置
#[derive(Debug, Clone)]
pub struct BatchWriterConfig {
    /// 最大缓冲条目数（达到后自动刷新）
    pub max_buffer_size: usize,
    /// 最大缓冲字节数（达到后自动刷新）
    pub max_buffer_bytes: usize,
    /// 是否在读取时先检查缓冲区
    pub read_from_buffer: bool,
    /// 是否在删除时刷新缓冲区
    pub flush_on_delete: bool,
}

impl Default for BatchWriterConfig {
    fn default() -> Self {
        Self {
            max_buffer_size: 100,
            max_buffer_bytes: 1024 * 1024, // 1MB
            read_from_buffer: true,
            flush_on_delete: true,
        }
    }
}

/// 批量写入统计
#[derive(Debug, Default)]
struct BatchWriterStats {
    /// 缓冲的写入次数
    buffered_writes: AtomicU64,
    /// 实际后端写入次数
    backend_writes: AtomicU64,
    /// 刷新次数
    flushes: AtomicU64,
    /// 合并的重复写入次数
    merged_writes: AtomicU64,
    /// 从缓冲区读取的次数
    buffer_reads: AtomicU64,
    /// 从后端读取的次数
    backend_reads: AtomicU64,
}

/// 详细批量写入统计
#[derive(Debug, Clone)]
pub struct DetailedBatchStats {
    /// 缓冲的写入次数
    pub buffered_writes: u64,
    /// 实际后端写入次数
    pub backend_writes: u64,
    /// 刷新次数
    pub flushes: u64,
    /// 合并的重复写入次数
    pub merged_writes: u64,
    /// 从缓冲区读取的次数
    pub buffer_reads: u64,
    /// 从后端读取的次数
    pub backend_reads: u64,
}

impl DetailedBatchStats {
    /// 写入合并率（节省的写入比例）
    pub fn merge_rate(&self) -> f64 {
        let total = self.buffered_writes + self.merged_writes;
        if total == 0 {
            0.0
        } else {
            self.merged_writes as f64 / total as f64
        }
    }

    /// I/O 节省率
    pub fn io_savings(&self) -> f64 {
        if self.buffered_writes == 0 {
            0.0
        } else {
            1.0 - (self.backend_writes as f64 / self.buffered_writes as f64)
        }
    }

    /// 缓冲区读取命中率
    pub fn buffer_hit_rate(&self) -> f64 {
        let total = self.buffer_reads + self.backend_reads;
        if total == 0 {
            0.0
        } else {
            self.buffer_reads as f64 / total as f64
        }
    }
}

/// 缓冲条目
struct BufferEntry {
    data: Vec<u8>,
}

/// 异步批量写入器
///
/// 缓冲写入操作，批量刷新到后端存储
pub struct BatchWriter<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 写入缓冲区
    buffer: RwLock<HashMap<String, BufferEntry>>,
    /// 当前缓冲字节数
    buffer_bytes: AtomicU64,
    /// 配置
    config: BatchWriterConfig,
    /// 统计
    stats: BatchWriterStats,
}

impl<B: StorageBackend> BatchWriter<B> {
    /// 创建批量写入器（默认配置）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, BatchWriterConfig::default())
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: BatchWriterConfig) -> Self {
        Self {
            backend,
            buffer: RwLock::new(HashMap::new()),
            buffer_bytes: AtomicU64::new(0),
            config,
            stats: BatchWriterStats::default(),
        }
    }

    /// 获取当前缓冲区大小（条目数）
    pub fn buffer_size(&self) -> usize {
        self.buffer.read().unwrap().len()
    }

    /// 获取当前缓冲字节数
    pub fn buffer_bytes(&self) -> u64 {
        self.buffer_bytes.load(Ordering::Relaxed)
    }

    /// 检查是否需要刷新
    fn should_flush(&self) -> bool {
        let buffer = self.buffer.read().unwrap();
        buffer.len() >= self.config.max_buffer_size
            || self.buffer_bytes.load(Ordering::Relaxed) >= self.config.max_buffer_bytes as u64
    }

    /// 刷新缓冲区到后端
    pub async fn flush(&self) -> StorageResult<usize> {
        let entries: Vec<(String, Vec<u8>)> = {
            let mut buffer = self.buffer.write().unwrap();
            let entries: Vec<_> = buffer
                .drain()
                .map(|(k, v)| (k, v.data))
                .collect();
            self.buffer_bytes.store(0, Ordering::Relaxed);
            entries
        };

        if entries.is_empty() {
            return Ok(0);
        }

        let count = entries.len();

        // 批量写入后端
        for (key, data) in entries {
            self.backend.write(&key, &data).await?;
            self.stats.backend_writes.fetch_add(1, Ordering::Relaxed);
        }

        self.stats.flushes.fetch_add(1, Ordering::Relaxed);

        Ok(count)
    }

    /// 获取详细统计
    pub fn detailed_stats(&self) -> DetailedBatchStats {
        DetailedBatchStats {
            buffered_writes: self.stats.buffered_writes.load(Ordering::Relaxed),
            backend_writes: self.stats.backend_writes.load(Ordering::Relaxed),
            flushes: self.stats.flushes.load(Ordering::Relaxed),
            merged_writes: self.stats.merged_writes.load(Ordering::Relaxed),
            buffer_reads: self.stats.buffer_reads.load(Ordering::Relaxed),
            backend_reads: self.stats.backend_reads.load(Ordering::Relaxed),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &BatchWriterConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 清空缓冲区（不写入后端）
    pub fn clear_buffer(&self) {
        let mut buffer = self.buffer.write().unwrap();
        buffer.clear();
        self.buffer_bytes.store(0, Ordering::Relaxed);
    }

    /// 检查键是否在缓冲区中
    pub fn is_buffered(&self, key: &str) -> bool {
        self.buffer.read().unwrap().contains_key(key)
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for BatchWriter<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        // 1. 如果配置允许，先检查缓冲区
        if self.config.read_from_buffer {
            let buffer = self.buffer.read().unwrap();
            if let Some(entry) = buffer.get(key) {
                self.stats.buffer_reads.fetch_add(1, Ordering::Relaxed);
                return Ok(entry.data.clone());
            }
        }

        // 2. 从后端读取
        self.stats.backend_reads.fetch_add(1, Ordering::Relaxed);
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.buffered_writes.fetch_add(1, Ordering::Relaxed);

        // 1. 写入缓冲区
        {
            let mut buffer = self.buffer.write().unwrap();
            let data_len = data.len() as u64;

            if let Some(old_entry) = buffer.get(key) {
                // 合并写入（替换旧值）
                self.stats.merged_writes.fetch_add(1, Ordering::Relaxed);
                let old_len = old_entry.data.len() as u64;
                self.buffer_bytes.fetch_sub(old_len, Ordering::Relaxed);
            }

            buffer.insert(
                key.to_string(),
                BufferEntry {
                    data: data.to_vec(),
                },
            );
            self.buffer_bytes.fetch_add(data_len, Ordering::Relaxed);
        }

        // 2. 检查是否需要刷新
        if self.should_flush() {
            self.flush().await?;
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // 1. 如果配置要求，先刷新缓冲区（确保一致性）
        if self.config.flush_on_delete {
            self.flush().await?;
        } else {
            // 仅从缓冲区移除
            let mut buffer = self.buffer.write().unwrap();
            if let Some(entry) = buffer.remove(key) {
                let len = entry.data.len() as u64;
                self.buffer_bytes.fetch_sub(len, Ordering::Relaxed);
            }
        }

        // 2. 从后端删除
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 合并缓冲区和后端的键
        let buffer_keys: Vec<String> = {
            let buffer = self.buffer.read().unwrap();
            buffer
                .keys()
                .filter(|k| prefix.is_empty() || k.starts_with(prefix))
                .cloned()
                .collect()
        };

        let mut backend_keys = self.backend.list(prefix).await?;

        // 合并并去重
        for key in buffer_keys {
            if !backend_keys.contains(&key) {
                backend_keys.push(key);
            }
        }

        backend_keys.sort();
        Ok(backend_keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // 先检查缓冲区
        if self.buffer.read().unwrap().contains_key(key) {
            return Ok(true);
        }

        // 再检查后端
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let backend_stats = self.backend.stats();
        let detailed = self.detailed_stats();

        StorageStats {
            reads: detailed.buffer_reads + detailed.backend_reads,
            writes: detailed.buffered_writes,
            deletes: backend_stats.deletes,
            hits: detailed.buffer_reads,
            misses: detailed.backend_reads,
            total_bytes: self.buffer_bytes.load(Ordering::Relaxed),
            key_count: self.buffer_size(),
        }
    }

    fn name(&self) -> &'static str {
        "BatchWriter"
    }
}

/// Drop 时自动刷新（同步版本，仅记录警告）
impl<B: StorageBackend> Drop for BatchWriter<B> {
    fn drop(&mut self) {
        let buffer_size = self.buffer.read().unwrap().len();
        if buffer_size > 0 {
            // 注意：在 Drop 中无法执行异步操作
            // 用户应该在丢弃前手动调用 flush()
            eprintln!(
                "Warning: BatchWriter dropped with {} unflushed entries",
                buffer_size
            );
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[tokio::test]
    async fn test_batch_writer_new() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        assert_eq!(writer.buffer_size(), 0);
        assert_eq!(writer.buffer_bytes(), 0);
        assert_eq!(writer.name(), "BatchWriter");
    }

    #[tokio::test]
    async fn test_batch_writer_write_buffered() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        writer.write("key1", b"hello").await.unwrap();

        // 数据应该在缓冲区
        assert_eq!(writer.buffer_size(), 1);
        assert!(writer.is_buffered("key1"));

        // 后端应该还没有数据（未刷新）
        assert!(!writer.backend().exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_writer_flush() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        writer.write("key1", b"hello").await.unwrap();
        writer.write("key2", b"world").await.unwrap();

        let flushed = writer.flush().await.unwrap();
        assert_eq!(flushed, 2);

        // 缓冲区应该为空
        assert_eq!(writer.buffer_size(), 0);

        // 后端应该有数据
        assert!(writer.backend().exists("key1").await.unwrap());
        assert!(writer.backend().exists("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_writer_read_from_buffer() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        writer.write("key1", b"buffered").await.unwrap();

        // 从缓冲区读取
        let data = writer.read("key1").await.unwrap();
        assert_eq!(data, b"buffered");

        let stats = writer.detailed_stats();
        assert_eq!(stats.buffer_reads, 1);
        assert_eq!(stats.backend_reads, 0);
    }

    #[tokio::test]
    async fn test_batch_writer_read_from_backend() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"backend").await.unwrap();

        let writer = BatchWriter::new(backend);

        // 从后端读取
        let data = writer.read("key1").await.unwrap();
        assert_eq!(data, b"backend");

        let stats = writer.detailed_stats();
        assert_eq!(stats.buffer_reads, 0);
        assert_eq!(stats.backend_reads, 1);
    }

    #[tokio::test]
    async fn test_batch_writer_merged_writes() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        // 多次写入同一个键
        writer.write("key1", b"v1").await.unwrap();
        writer.write("key1", b"v2").await.unwrap();
        writer.write("key1", b"v3").await.unwrap();

        // 缓冲区应该只有一个条目
        assert_eq!(writer.buffer_size(), 1);

        // 应该记录合并
        let stats = writer.detailed_stats();
        assert_eq!(stats.buffered_writes, 3);
        assert_eq!(stats.merged_writes, 2);

        // 刷新后只写入一次
        writer.flush().await.unwrap();

        let data = writer.backend().read("key1").await.unwrap();
        assert_eq!(data, b"v3"); // 最新值
    }

    #[tokio::test]
    async fn test_batch_writer_auto_flush() {
        let backend = MemoryStorage::new();
        let config = BatchWriterConfig {
            max_buffer_size: 3,
            ..Default::default()
        };
        let writer = BatchWriter::with_config(backend, config);

        // 写入 3 个键，应该触发自动刷新
        writer.write("key1", b"v1").await.unwrap();
        writer.write("key2", b"v2").await.unwrap();
        writer.write("key3", b"v3").await.unwrap();

        // 缓冲区应该为空（已刷新）
        assert_eq!(writer.buffer_size(), 0);

        // 后端应该有数据
        assert!(writer.backend().exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_writer_auto_flush_by_bytes() {
        let backend = MemoryStorage::new();
        let config = BatchWriterConfig {
            max_buffer_bytes: 10,
            max_buffer_size: 1000, // 不限制数量
            ..Default::default()
        };
        let writer = BatchWriter::with_config(backend, config);

        // 写入超过 10 字节
        writer.write("key1", b"12345").await.unwrap(); // 5 bytes
        assert_eq!(writer.buffer_size(), 1);

        writer.write("key2", b"67890ab").await.unwrap(); // 7 bytes, total > 10

        // 应该触发刷新
        assert_eq!(writer.buffer_size(), 0);
    }

    #[tokio::test]
    async fn test_batch_writer_delete() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"existing").await.unwrap();

        let writer = BatchWriter::new(backend);

        // 写入缓冲区
        writer.write("key2", b"buffered").await.unwrap();

        // 删除后端键
        writer.delete("key1").await.unwrap();
        assert!(!writer.exists("key1").await.unwrap());

        // 删除应该触发刷新（默认配置）
        assert_eq!(writer.buffer_size(), 0);
    }

    #[tokio::test]
    async fn test_batch_writer_delete_no_flush() {
        let backend = MemoryStorage::new();
        let config = BatchWriterConfig {
            flush_on_delete: false,
            ..Default::default()
        };
        let writer = BatchWriter::with_config(backend, config);

        writer.write("key1", b"buffered").await.unwrap();
        writer.write("key2", b"also buffered").await.unwrap();

        // 删除不触发刷新
        writer.delete("key1").await.unwrap();

        // 缓冲区应该只剩一个
        assert_eq!(writer.buffer_size(), 1);
        assert!(!writer.is_buffered("key1"));
        assert!(writer.is_buffered("key2"));
    }

    #[tokio::test]
    async fn test_batch_writer_list() {
        let backend = MemoryStorage::new();
        backend.write("backend1", b"data").await.unwrap();

        let writer = BatchWriter::new(backend);
        writer.write("buffer1", b"data").await.unwrap();

        let keys = writer.list("").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"backend1".to_string()));
        assert!(keys.contains(&"buffer1".to_string()));
    }

    #[tokio::test]
    async fn test_batch_writer_exists() {
        let backend = MemoryStorage::new();
        backend.write("backend_key", b"data").await.unwrap();

        let writer = BatchWriter::new(backend);
        writer.write("buffer_key", b"data").await.unwrap();

        // 缓冲区中的键
        assert!(writer.exists("buffer_key").await.unwrap());

        // 后端中的键
        assert!(writer.exists("backend_key").await.unwrap());

        // 不存在的键
        assert!(!writer.exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_writer_clear_buffer() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        writer.write("key1", b"data1").await.unwrap();
        writer.write("key2", b"data2").await.unwrap();
        assert_eq!(writer.buffer_size(), 2);

        writer.clear_buffer();
        assert_eq!(writer.buffer_size(), 0);
        assert_eq!(writer.buffer_bytes(), 0);

        // 后端应该没有数据
        assert!(!writer.backend().exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_batch_writer_stats() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        writer.write("key1", b"hello").await.unwrap();
        writer.read("key1").await.unwrap();
        writer.flush().await.unwrap();

        let stats = writer.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.hits, 1); // buffer read
    }

    #[tokio::test]
    async fn test_batch_writer_io_savings() {
        let backend = MemoryStorage::new();
        let config = BatchWriterConfig {
            max_buffer_size: 100,
            ..Default::default()
        };
        let writer = BatchWriter::with_config(backend, config);

        // 写入 10 次同一个键
        for i in 0..10 {
            writer.write("key1", format!("v{}", i).as_bytes()).await.unwrap();
        }

        writer.flush().await.unwrap();

        let stats = writer.detailed_stats();
        assert_eq!(stats.buffered_writes, 10);
        assert_eq!(stats.backend_writes, 1); // 只写入一次
        assert_eq!(stats.merged_writes, 9);
        assert!((stats.io_savings() - 0.9).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_batch_writer_default_config() {
        let config = BatchWriterConfig::default();

        assert_eq!(config.max_buffer_size, 100);
        assert_eq!(config.max_buffer_bytes, 1024 * 1024);
        assert!(config.read_from_buffer);
        assert!(config.flush_on_delete);
    }

    #[tokio::test]
    async fn test_batch_writer_read_not_found() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        let result = writer.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_batch_writer_flush_empty() {
        let backend = MemoryStorage::new();
        let writer = BatchWriter::new(backend);

        // 刷新空缓冲区
        let flushed = writer.flush().await.unwrap();
        assert_eq!(flushed, 0);
    }

    #[tokio::test]
    async fn test_batch_writer_read_from_buffer_disabled() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"backend").await.unwrap();

        let config = BatchWriterConfig {
            read_from_buffer: false,
            ..Default::default()
        };
        let writer = BatchWriter::with_config(backend, config);

        // 写入缓冲区
        writer.write("key1", b"buffered").await.unwrap();

        // 应该从后端读取（忽略缓冲区）
        let data = writer.read("key1").await.unwrap();
        assert_eq!(data, b"backend");
    }
}
