//! 全优化存储层
//!
//! v1.62.0: v2.0 探路期 - 读写双优化存储
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的全优化存储架构：
//! - **读优化**: TieredCache 三层缓存加速读取
//! - **写优化**: 批量缓冲减少 I/O 次数
//! - **协调层**: 确保读写一致性
//!
//! ## 架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    OptimizedStorage                         │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌─────────────────┐    ┌─────────────────────────────┐    │
//! │  │   Write Buffer  │    │       Read Cache            │    │
//! │  │   (批量缓冲)     │    │   (TieredCache)            │    │
//! │  │                 │    │   Hot → Warm → Cold        │    │
//! │  │  key1 → data1   │    │                             │    │
//! │  │  key2 → data2   │    │                             │    │
//! │  └────────┬────────┘    └──────────────┬──────────────┘    │
//! │           │                            │                    │
//! │           │     Write-Through          │                    │
//! │           │     (写入时同步更新缓存)     │                    │
//! │           │                            │                    │
//! │           └────────────┬───────────────┘                    │
//! │                        │                                    │
//! │                        ▼                                    │
//! │              ┌─────────────────┐                            │
//! │              │  Backend Store  │                            │
//! │              │  (FileStorage)  │                            │
//! │              └─────────────────┘                            │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## 读写流程
//!
//! **写入**:
//! 1. 写入缓冲区 (Write Buffer)
//! 2. 同步更新读缓存 (TieredCache) - Write-Through
//! 3. 缓冲区满时批量刷新到后端
//!
//! **读取**:
//! 1. 检查写缓冲区 (最新数据)
//! 2. 检查读缓存 (TieredCache)
//! 3. 从后端读取并填充缓存
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{OptimizedStorage, FileStorage};
//!
//! let backend = FileStorage::new("/path/to/data");
//! let storage = OptimizedStorage::new(backend);
//!
//! // 写入（缓冲 + 缓存）
//! storage.write("key1", b"value1").await?;
//!
//! // 读取（缓存优先）
//! let data = storage.read("key1").await?;
//!
//! // 刷新到后端
//! storage.flush().await?;
//!
//! // 查看统计
//! let stats = storage.optimization_stats();
//! println!("Read hit rate: {:.2}%", stats.read_hit_rate() * 100.0);
//! println!("Write savings: {:.2}%", stats.write_io_savings() * 100.0);
//! ```

use super::tiered_cache::{TieredCache, TieredCacheConfig};
use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// 全优化存储配置
#[derive(Debug, Clone)]
pub struct OptimizedStorageConfig {
    /// 读缓存配置
    pub cache_config: TieredCacheConfig,
    /// 最大写缓冲条目数
    pub max_write_buffer_size: usize,
    /// 最大写缓冲字节数
    pub max_write_buffer_bytes: usize,
    /// 是否启用读缓存
    pub enable_read_cache: bool,
    /// 是否启用写缓冲
    pub enable_write_buffer: bool,
    /// 删除时是否刷新写缓冲
    pub flush_on_delete: bool,
}

impl Default for OptimizedStorageConfig {
    fn default() -> Self {
        Self {
            cache_config: TieredCacheConfig {
                hot_capacity: 50,
                warm_capacity: 200,
                cold_capacity: 1000,
                promotion_threshold: 3,
            },
            max_write_buffer_size: 100,
            max_write_buffer_bytes: 1024 * 1024, // 1MB
            enable_read_cache: true,
            enable_write_buffer: true,
            flush_on_delete: true,
        }
    }
}

/// 优化统计
#[derive(Debug, Default)]
struct OptimizationStats {
    // 读统计
    read_cache_hits: AtomicU64,
    read_buffer_hits: AtomicU64,
    read_backend_hits: AtomicU64,
    // 写统计
    buffered_writes: AtomicU64,
    merged_writes: AtomicU64,
    backend_writes: AtomicU64,
    flushes: AtomicU64,
}

/// 详细优化统计
#[derive(Debug, Clone)]
pub struct DetailedOptimizationStats {
    // 读统计
    pub read_cache_hits: u64,
    pub read_buffer_hits: u64,
    pub read_backend_hits: u64,
    // 写统计
    pub buffered_writes: u64,
    pub merged_writes: u64,
    pub backend_writes: u64,
    pub flushes: u64,
}

impl DetailedOptimizationStats {
    /// 读取命中率（缓存 + 缓冲区）
    pub fn read_hit_rate(&self) -> f64 {
        let hits = self.read_cache_hits + self.read_buffer_hits;
        let total = hits + self.read_backend_hits;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// 写入 I/O 节省率
    pub fn write_io_savings(&self) -> f64 {
        if self.buffered_writes == 0 {
            0.0
        } else {
            1.0 - (self.backend_writes as f64 / self.buffered_writes as f64)
        }
    }

    /// 写入合并率
    pub fn write_merge_rate(&self) -> f64 {
        let total = self.buffered_writes + self.merged_writes;
        if total == 0 {
            0.0
        } else {
            self.merged_writes as f64 / total as f64
        }
    }

    /// 总读取次数
    pub fn total_reads(&self) -> u64 {
        self.read_cache_hits + self.read_buffer_hits + self.read_backend_hits
    }

    /// 总写入次数
    pub fn total_writes(&self) -> u64 {
        self.buffered_writes
    }
}

/// 写缓冲条目
struct WriteBufferEntry {
    data: Vec<u8>,
}

/// 全优化存储
///
/// 组合读缓存和写缓冲，提供全面的 I/O 优化
pub struct OptimizedStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 读缓存
    read_cache: TieredCache<String, Vec<u8>>,
    /// 写缓冲区
    write_buffer: RwLock<HashMap<String, WriteBufferEntry>>,
    /// 写缓冲字节数
    write_buffer_bytes: AtomicU64,
    /// 配置
    config: OptimizedStorageConfig,
    /// 统计
    stats: OptimizationStats,
}

impl<B: StorageBackend> OptimizedStorage<B> {
    /// 创建全优化存储（默认配置）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, OptimizedStorageConfig::default())
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: OptimizedStorageConfig) -> Self {
        Self {
            backend,
            read_cache: TieredCache::with_config(config.cache_config.clone()),
            write_buffer: RwLock::new(HashMap::new()),
            write_buffer_bytes: AtomicU64::new(0),
            config,
            stats: OptimizationStats::default(),
        }
    }

    /// 获取写缓冲区大小
    pub fn write_buffer_size(&self) -> usize {
        self.write_buffer.read().unwrap().len()
    }

    /// 获取写缓冲字节数
    pub fn write_buffer_bytes(&self) -> u64 {
        self.write_buffer_bytes.load(Ordering::Relaxed)
    }

    /// 获取读缓存大小
    pub fn read_cache_size(&self) -> usize {
        self.read_cache.len()
    }

    /// 获取读缓存各层大小
    pub fn read_cache_tier_sizes(&self) -> (usize, usize, usize) {
        self.read_cache.tier_sizes()
    }

    /// 检查是否需要刷新写缓冲
    fn should_flush(&self) -> bool {
        if !self.config.enable_write_buffer {
            return true; // 禁用缓冲时立即刷新
        }
        let buffer = self.write_buffer.read().unwrap();
        buffer.len() >= self.config.max_write_buffer_size
            || self.write_buffer_bytes.load(Ordering::Relaxed)
                >= self.config.max_write_buffer_bytes as u64
    }

    /// 刷新写缓冲到后端
    pub async fn flush(&self) -> StorageResult<usize> {
        let entries: Vec<(String, Vec<u8>)> = {
            let mut buffer = self.write_buffer.write().unwrap();
            let entries: Vec<_> = buffer.drain().map(|(k, v)| (k, v.data)).collect();
            self.write_buffer_bytes.store(0, Ordering::Relaxed);
            entries
        };

        if entries.is_empty() {
            return Ok(0);
        }

        let count = entries.len();

        for (key, data) in entries {
            self.backend.write(&key, &data).await?;
            self.stats.backend_writes.fetch_add(1, Ordering::Relaxed);
        }

        self.stats.flushes.fetch_add(1, Ordering::Relaxed);

        Ok(count)
    }

    /// 获取详细优化统计
    pub fn optimization_stats(&self) -> DetailedOptimizationStats {
        DetailedOptimizationStats {
            read_cache_hits: self.stats.read_cache_hits.load(Ordering::Relaxed),
            read_buffer_hits: self.stats.read_buffer_hits.load(Ordering::Relaxed),
            read_backend_hits: self.stats.read_backend_hits.load(Ordering::Relaxed),
            buffered_writes: self.stats.buffered_writes.load(Ordering::Relaxed),
            merged_writes: self.stats.merged_writes.load(Ordering::Relaxed),
            backend_writes: self.stats.backend_writes.load(Ordering::Relaxed),
            flushes: self.stats.flushes.load(Ordering::Relaxed),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &OptimizedStorageConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 清空读缓存
    pub fn clear_read_cache(&self) {
        self.read_cache.clear();
    }

    /// 清空写缓冲（不写入后端）
    pub fn clear_write_buffer(&self) {
        let mut buffer = self.write_buffer.write().unwrap();
        buffer.clear();
        self.write_buffer_bytes.store(0, Ordering::Relaxed);
    }

    /// 清空所有缓存和缓冲
    pub fn clear_all(&self) {
        self.clear_read_cache();
        self.clear_write_buffer();
    }

    /// 预热读缓存
    pub async fn warm_cache(&self, keys: &[&str]) -> StorageResult<usize> {
        let mut loaded = 0;
        for key in keys {
            if !self.read_cache.contains(&key.to_string()) {
                // 先检查写缓冲
                let in_buffer = self.write_buffer.read().unwrap().contains_key(*key);
                if !in_buffer {
                    if let Ok(data) = self.backend.read(key).await {
                        self.read_cache.insert(key.to_string(), data);
                        loaded += 1;
                    }
                }
            }
        }
        Ok(loaded)
    }

    /// 检查键是否在写缓冲中
    pub fn is_write_buffered(&self, key: &str) -> bool {
        self.write_buffer.read().unwrap().contains_key(key)
    }

    /// 检查键是否在读缓存中
    pub fn is_read_cached(&self, key: &str) -> bool {
        self.read_cache.contains(&key.to_string())
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for OptimizedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        // 1. 检查写缓冲区（最新数据）
        {
            let buffer = self.write_buffer.read().unwrap();
            if let Some(entry) = buffer.get(key) {
                self.stats.read_buffer_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(entry.data.clone());
            }
        }

        // 2. 检查读缓存
        if self.config.enable_read_cache {
            if let Some(data) = self.read_cache.get(&key.to_string()) {
                self.stats.read_cache_hits.fetch_add(1, Ordering::Relaxed);
                return Ok(data);
            }
        }

        // 3. 从后端读取
        self.stats.read_backend_hits.fetch_add(1, Ordering::Relaxed);
        let data = self.backend.read(key).await?;

        // 4. 填充读缓存
        if self.config.enable_read_cache {
            self.read_cache.insert(key.to_string(), data.clone());
        }

        Ok(data)
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.buffered_writes.fetch_add(1, Ordering::Relaxed);

        // 1. 更新读缓存（Write-Through）
        if self.config.enable_read_cache {
            self.read_cache.insert(key.to_string(), data.to_vec());
        }

        // 2. 写入缓冲区
        if self.config.enable_write_buffer {
            {
                let mut buffer = self.write_buffer.write().unwrap();
                let data_len = data.len() as u64;

                if let Some(old_entry) = buffer.get(key) {
                    // 合并写入
                    self.stats.merged_writes.fetch_add(1, Ordering::Relaxed);
                    let old_len = old_entry.data.len() as u64;
                    self.write_buffer_bytes.fetch_sub(old_len, Ordering::Relaxed);
                }

                buffer.insert(key.to_string(), WriteBufferEntry { data: data.to_vec() });
                self.write_buffer_bytes.fetch_add(data_len, Ordering::Relaxed);
            } // buffer guard dropped here

            // 3. 检查是否需要刷新
            if self.should_flush() {
                self.flush().await?;
            }
        } else {
            // 禁用缓冲，直接写入后端
            self.backend.write(key, data).await?;
            self.stats.backend_writes.fetch_add(1, Ordering::Relaxed);
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // 1. 从读缓存删除
        self.read_cache.remove(&key.to_string());

        // 2. 处理写缓冲
        if self.config.flush_on_delete {
            self.flush().await?;
        } else {
            let mut buffer = self.write_buffer.write().unwrap();
            if let Some(entry) = buffer.remove(key) {
                let len = entry.data.len() as u64;
                self.write_buffer_bytes.fetch_sub(len, Ordering::Relaxed);
            }
        }

        // 3. 从后端删除
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 合并写缓冲区和后端的键
        let buffer_keys: Vec<String> = {
            let buffer = self.write_buffer.read().unwrap();
            buffer
                .keys()
                .filter(|k| prefix.is_empty() || k.starts_with(prefix))
                .cloned()
                .collect()
        };

        let mut backend_keys = self.backend.list(prefix).await?;

        for key in buffer_keys {
            if !backend_keys.contains(&key) {
                backend_keys.push(key);
            }
        }

        backend_keys.sort();
        Ok(backend_keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // 检查写缓冲区
        if self.write_buffer.read().unwrap().contains_key(key) {
            return Ok(true);
        }

        // 检查读缓存
        if self.read_cache.contains(&key.to_string()) {
            return Ok(true);
        }

        // 检查后端
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let opt_stats = self.optimization_stats();

        StorageStats {
            reads: opt_stats.total_reads(),
            writes: opt_stats.total_writes(),
            deletes: self.backend.stats().deletes,
            hits: opt_stats.read_cache_hits + opt_stats.read_buffer_hits,
            misses: opt_stats.read_backend_hits,
            total_bytes: self.write_buffer_bytes.load(Ordering::Relaxed),
            key_count: self.write_buffer_size() + self.read_cache_size(),
        }
    }

    fn name(&self) -> &'static str {
        "OptimizedStorage"
    }
}

impl<B: StorageBackend> Drop for OptimizedStorage<B> {
    fn drop(&mut self) {
        let buffer_size = self.write_buffer.read().unwrap().len();
        if buffer_size > 0 {
            eprintln!(
                "Warning: OptimizedStorage dropped with {} unflushed entries",
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
    async fn test_optimized_storage_new() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        assert_eq!(storage.write_buffer_size(), 0);
        assert_eq!(storage.read_cache_size(), 0);
        assert_eq!(storage.name(), "OptimizedStorage");
    }

    #[tokio::test]
    async fn test_optimized_storage_write_read() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"hello").await.unwrap();

        // 读取（从写缓冲区）
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"hello");

        let stats = storage.optimization_stats();
        assert_eq!(stats.read_buffer_hits, 1);
    }

    #[tokio::test]
    async fn test_optimized_storage_read_from_cache() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"cached").await.unwrap();

        let storage = OptimizedStorage::new(backend);

        // 第一次读取（从后端，填充缓存）
        let data1 = storage.read("key1").await.unwrap();
        assert_eq!(data1, b"cached");

        // 第二次读取（从缓存）
        let data2 = storage.read("key1").await.unwrap();
        assert_eq!(data2, b"cached");

        let stats = storage.optimization_stats();
        assert_eq!(stats.read_backend_hits, 1);
        assert_eq!(stats.read_cache_hits, 1);
    }

    #[tokio::test]
    async fn test_optimized_storage_write_through() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"value1").await.unwrap();

        // 写入应该同时更新缓存
        assert!(storage.is_read_cached("key1"));
        assert!(storage.is_write_buffered("key1"));
    }

    #[tokio::test]
    async fn test_optimized_storage_flush() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        let flushed = storage.flush().await.unwrap();
        assert_eq!(flushed, 2);
        assert_eq!(storage.write_buffer_size(), 0);

        // 后端应该有数据
        assert!(storage.backend().exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_optimized_storage_auto_flush() {
        let backend = MemoryStorage::new();
        let config = OptimizedStorageConfig {
            max_write_buffer_size: 3,
            ..Default::default()
        };
        let storage = OptimizedStorage::with_config(backend, config);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();
        storage.write("key3", b"v3").await.unwrap();

        // 应该已自动刷新
        assert_eq!(storage.write_buffer_size(), 0);
    }

    #[tokio::test]
    async fn test_optimized_storage_write_merge() {
        let backend = MemoryStorage::new();
        let config = OptimizedStorageConfig {
            max_write_buffer_size: 100,
            ..Default::default()
        };
        let storage = OptimizedStorage::with_config(backend, config);

        // 多次写入同一键
        for i in 0..5 {
            storage
                .write("key1", format!("v{}", i).as_bytes())
                .await
                .unwrap();
        }

        assert_eq!(storage.write_buffer_size(), 1);

        let stats = storage.optimization_stats();
        assert_eq!(stats.buffered_writes, 5);
        assert_eq!(stats.merged_writes, 4);
    }

    #[tokio::test]
    async fn test_optimized_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());

        // 缓存也应该被清除
        assert!(!storage.is_read_cached("key1"));
    }

    #[tokio::test]
    async fn test_optimized_storage_list() {
        let backend = MemoryStorage::new();
        backend.write("backend1", b"data").await.unwrap();

        let storage = OptimizedStorage::new(backend);
        storage.write("buffer1", b"data").await.unwrap();

        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"backend1".to_string()));
        assert!(keys.contains(&"buffer1".to_string()));
    }

    #[tokio::test]
    async fn test_optimized_storage_exists() {
        let backend = MemoryStorage::new();
        backend.write("backend_key", b"data").await.unwrap();

        let storage = OptimizedStorage::new(backend);
        storage.write("buffer_key", b"data").await.unwrap();

        assert!(storage.exists("backend_key").await.unwrap());
        assert!(storage.exists("buffer_key").await.unwrap());
        assert!(!storage.exists("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_optimized_storage_clear_all() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        assert!(storage.write_buffer_size() > 0);
        assert!(storage.read_cache_size() > 0);

        storage.clear_all();

        assert_eq!(storage.write_buffer_size(), 0);
        assert_eq!(storage.read_cache_size(), 0);
    }

    #[tokio::test]
    async fn test_optimized_storage_warm_cache() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"v1").await.unwrap();
        backend.write("key2", b"v2").await.unwrap();

        let storage = OptimizedStorage::new(backend);

        let loaded = storage.warm_cache(&["key1", "key2"]).await.unwrap();
        assert_eq!(loaded, 2);
        assert_eq!(storage.read_cache_size(), 2);
    }

    #[tokio::test]
    async fn test_optimized_storage_hit_rate() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();

        // 5 次读取（从缓冲区/缓存）
        for _ in 0..5 {
            storage.read("key1").await.unwrap();
        }

        let stats = storage.optimization_stats();
        assert!((stats.read_hit_rate() - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_optimized_storage_io_savings() {
        let backend = MemoryStorage::new();
        let config = OptimizedStorageConfig {
            max_write_buffer_size: 100,
            ..Default::default()
        };
        let storage = OptimizedStorage::with_config(backend, config);

        // 10 次写入同一键
        for i in 0..10 {
            storage
                .write("key1", format!("v{}", i).as_bytes())
                .await
                .unwrap();
        }

        storage.flush().await.unwrap();

        let stats = storage.optimization_stats();
        assert_eq!(stats.buffered_writes, 10);
        assert_eq!(stats.backend_writes, 1);
        assert!((stats.write_io_savings() - 0.9).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_optimized_storage_disabled_cache() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"data").await.unwrap();

        let config = OptimizedStorageConfig {
            enable_read_cache: false,
            ..Default::default()
        };
        let storage = OptimizedStorage::with_config(backend, config);

        // 读取不填充缓存
        storage.read("key1").await.unwrap();
        assert_eq!(storage.read_cache_size(), 0);
    }

    #[tokio::test]
    async fn test_optimized_storage_disabled_buffer() {
        let backend = MemoryStorage::new();
        let config = OptimizedStorageConfig {
            enable_write_buffer: false,
            ..Default::default()
        };
        let storage = OptimizedStorage::with_config(backend, config);

        storage.write("key1", b"data").await.unwrap();

        // 应该直接写入后端
        assert_eq!(storage.write_buffer_size(), 0);
        assert!(storage.backend().exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_optimized_storage_tier_sizes() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();

        let (hot, warm, cold) = storage.read_cache_tier_sizes();
        assert_eq!(cold, 1); // 新数据在冷层
        assert_eq!(warm, 0);
        assert_eq!(hot, 0);
    }

    #[tokio::test]
    async fn test_optimized_storage_default_config() {
        let config = OptimizedStorageConfig::default();

        assert_eq!(config.max_write_buffer_size, 100);
        assert_eq!(config.max_write_buffer_bytes, 1024 * 1024);
        assert!(config.enable_read_cache);
        assert!(config.enable_write_buffer);
        assert!(config.flush_on_delete);
    }

    #[tokio::test]
    async fn test_optimized_storage_read_not_found() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_optimized_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = OptimizedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.stats();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.hits, 1);
    }
}
