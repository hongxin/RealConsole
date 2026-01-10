//! 缓存加速存储层
//!
//! v1.60.0: v2.0 探路期 - 组合 FileStorage + TieredCache
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的缓存存储架构：
//! - **持久层**: FileStorage 提供数据持久化
//! - **缓存层**: TieredCache 提供读取加速
//! - **协调层**: CachedStorage 协调两者，确保一致性
//!
//! ## 缓存策略
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                   CachedStorage                       │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Read:                                                │
//! │    1. 检查缓存 (TieredCache)                          │
//! │    2. 缓存命中 → 返回                                 │
//! │    3. 缓存未命中 → 读取后端 → 写入缓存 → 返回          │
//! │                                                       │
//! │  Write (Write-Through):                               │
//! │    1. 写入后端 (FileStorage)                          │
//! │    2. 写入缓存                                        │
//! │                                                       │
//! │  Delete:                                              │
//! │    1. 从缓存删除                                      │
//! │    2. 从后端删除                                      │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{CachedStorage, FileStorage, CachedStorageConfig};
//!
//! // 创建带缓存的文件存储
//! let file_storage = FileStorage::new("/path/to/data");
//! let cached = CachedStorage::new(file_storage);
//!
//! // 写入（同时写入缓存和文件）
//! cached.write("key1", b"value1").await?;
//!
//! // 读取（优先从缓存读取）
//! let data = cached.read("key1").await?;  // 缓存命中
//!
//! // 查看统计
//! let stats = cached.cache_stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! ```

use super::tiered_cache::{DetailedCacheStats, TieredCache, TieredCacheConfig};
use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};

/// 缓存存储配置
#[derive(Debug, Clone)]
pub struct CachedStorageConfig {
    /// 缓存配置
    pub cache_config: TieredCacheConfig,
    /// 是否启用写入缓存（write-through）
    pub cache_on_write: bool,
    /// 是否在读取未命中时填充缓存
    pub cache_on_read_miss: bool,
}

impl Default for CachedStorageConfig {
    fn default() -> Self {
        Self {
            cache_config: TieredCacheConfig {
                hot_capacity: 50,
                warm_capacity: 200,
                cold_capacity: 1000,
                promotion_threshold: 3,
            },
            cache_on_write: true,
            cache_on_read_miss: true,
        }
    }
}

/// 缓存加速存储
///
/// 组合任意 StorageBackend 与 TieredCache，提供缓存加速
pub struct CachedStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 读缓存
    cache: TieredCache<String, Vec<u8>>,
    /// 配置
    config: CachedStorageConfig,
    /// 统计
    stats: CachedStorageStats,
}

/// 缓存存储统计
struct CachedStorageStats {
    /// 缓存命中
    cache_hits: AtomicU64,
    /// 缓存未命中
    cache_misses: AtomicU64,
    /// 后端读取次数
    backend_reads: AtomicU64,
    /// 后端写入次数
    backend_writes: AtomicU64,
}

impl Default for CachedStorageStats {
    fn default() -> Self {
        Self {
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            backend_reads: AtomicU64::new(0),
            backend_writes: AtomicU64::new(0),
        }
    }
}

impl<B: StorageBackend> CachedStorage<B> {
    /// 创建缓存存储（默认配置）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, CachedStorageConfig::default())
    }

    /// 使用自定义配置创建缓存存储
    pub fn with_config(backend: B, config: CachedStorageConfig) -> Self {
        Self {
            backend,
            cache: TieredCache::with_config(config.cache_config.clone()),
            config,
            stats: CachedStorageStats::default(),
        }
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> &super::tiered_cache::CacheStats {
        self.cache.stats()
    }

    /// 获取详细缓存统计
    pub fn detailed_cache_stats(&self) -> DetailedCacheStats {
        self.cache.stats().detailed()
    }

    /// 获取缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.stats.cache_hits.load(Ordering::Relaxed);
        let misses = self.stats.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// 获取缓存大小（条目数）
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// 获取各层缓存大小
    pub fn cache_tier_sizes(&self) -> (usize, usize, usize) {
        self.cache.tier_sizes()
    }

    /// 清空缓存（不影响后端）
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// 预热缓存（从后端加载）
    pub async fn warm_cache(&self, keys: &[&str]) -> StorageResult<usize> {
        let mut loaded = 0;
        for key in keys {
            if !self.cache.contains(&key.to_string()) {
                if let Ok(data) = self.backend.read(key).await {
                    self.cache.insert(key.to_string(), data);
                    loaded += 1;
                }
            }
        }
        Ok(loaded)
    }

    /// 获取后端存储引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 获取配置
    pub fn config(&self) -> &CachedStorageConfig {
        &self.config
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for CachedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        // 1. 检查缓存
        if let Some(data) = self.cache.get(&key.to_string()) {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(data);
        }

        // 2. 缓存未命中，从后端读取
        self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
        self.stats.backend_reads.fetch_add(1, Ordering::Relaxed);

        let data = self.backend.read(key).await?;

        // 3. 填充缓存
        if self.config.cache_on_read_miss {
            self.cache.insert(key.to_string(), data.clone());
        }

        Ok(data)
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        // 1. 写入后端（Write-Through）
        self.stats.backend_writes.fetch_add(1, Ordering::Relaxed);
        self.backend.write(key, data).await?;

        // 2. 更新缓存
        if self.config.cache_on_write {
            self.cache.insert(key.to_string(), data.to_vec());
        }

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // 1. 从缓存删除
        self.cache.remove(&key.to_string());

        // 2. 从后端删除
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 直接从后端列出（缓存不维护键列表）
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // 先检查缓存
        if self.cache.contains(&key.to_string()) {
            return Ok(true);
        }

        // 再检查后端
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let backend_stats = self.backend.stats();
        let cache_stats = self.cache.stats().detailed();

        StorageStats {
            reads: backend_stats.reads,
            writes: backend_stats.writes,
            deletes: backend_stats.deletes,
            hits: cache_stats.hot_hits + cache_stats.warm_hits + cache_stats.cold_hits,
            misses: cache_stats.misses,
            total_bytes: backend_stats.total_bytes,
            key_count: backend_stats.key_count,
        }
    }

    fn name(&self) -> &'static str {
        "CachedStorage"
    }
}

/// 组合统计信息
#[derive(Debug, Clone)]
pub struct CombinedStorageStats {
    /// 后端统计
    pub backend: StorageStats,
    /// 缓存统计
    pub cache: DetailedCacheStats,
    /// 缓存命中次数
    pub cache_hits: u64,
    /// 缓存未命中次数
    pub cache_misses: u64,
    /// 后端读取次数
    pub backend_reads: u64,
    /// 后端写入次数
    pub backend_writes: u64,
}

impl CombinedStorageStats {
    /// 整体缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }

    /// 缓存节省的后端读取比例
    pub fn backend_read_savings(&self) -> f64 {
        let total_reads = self.cache_hits + self.backend_reads;
        if total_reads == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_reads as f64
        }
    }
}

impl<B: StorageBackend> CachedStorage<B> {
    /// 获取组合统计信息
    pub fn combined_stats(&self) -> CombinedStorageStats {
        CombinedStorageStats {
            backend: self.backend.stats(),
            cache: self.cache.stats().detailed(),
            cache_hits: self.stats.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.stats.cache_misses.load(Ordering::Relaxed),
            backend_reads: self.stats.backend_reads.load(Ordering::Relaxed),
            backend_writes: self.stats.backend_writes.load(Ordering::Relaxed),
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
    async fn test_cached_storage_new() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        assert_eq!(cached.cache_size(), 0);
        assert_eq!(cached.name(), "CachedStorage");
    }

    #[tokio::test]
    async fn test_cached_storage_write_read() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"hello").await.unwrap();

        // 第一次读取（从缓存）
        let data = cached.read("key1").await.unwrap();
        assert_eq!(data, b"hello");

        // 验证缓存命中
        let stats = cached.combined_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 0);
    }

    #[tokio::test]
    async fn test_cached_storage_cache_miss_then_hit() {
        let backend = MemoryStorage::new();

        // 直接写入后端
        backend.write("key1", b"world").await.unwrap();

        let cached = CachedStorage::new(backend);

        // 第一次读取（缓存未命中，从后端读取）
        let data1 = cached.read("key1").await.unwrap();
        assert_eq!(data1, b"world");

        let stats1 = cached.combined_stats();
        assert_eq!(stats1.cache_hits, 0);
        assert_eq!(stats1.cache_misses, 1);
        assert_eq!(stats1.backend_reads, 1);

        // 第二次读取（缓存命中）
        let data2 = cached.read("key1").await.unwrap();
        assert_eq!(data2, b"world");

        let stats2 = cached.combined_stats();
        assert_eq!(stats2.cache_hits, 1);
        assert_eq!(stats2.cache_misses, 1);
        assert_eq!(stats2.backend_reads, 1); // 没有额外的后端读取
    }

    #[tokio::test]
    async fn test_cached_storage_delete() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data").await.unwrap();
        assert!(cached.exists("key1").await.unwrap());

        cached.delete("key1").await.unwrap();
        assert!(!cached.exists("key1").await.unwrap());

        // 验证缓存也被清除
        assert!(!cached.cache.contains(&"key1".to_string()));
    }

    #[tokio::test]
    async fn test_cached_storage_list() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();
        cached.write("key2", b"data2").await.unwrap();

        let keys = cached.list("").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_cached_storage_exists() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        assert!(!cached.exists("key1").await.unwrap());

        cached.write("key1", b"data").await.unwrap();
        assert!(cached.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_cached_storage_clear_cache() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();
        cached.write("key2", b"data2").await.unwrap();
        assert_eq!(cached.cache_size(), 2);

        cached.clear_cache();
        assert_eq!(cached.cache_size(), 0);

        // 后端数据仍然存在
        assert!(cached.backend().exists("key1").await.unwrap());
        assert!(cached.backend().exists("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_cached_storage_warm_cache() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"data1").await.unwrap();
        backend.write("key2", b"data2").await.unwrap();
        backend.write("key3", b"data3").await.unwrap();

        let cached = CachedStorage::new(backend);
        assert_eq!(cached.cache_size(), 0);

        // 预热缓存
        let loaded = cached.warm_cache(&["key1", "key2"]).await.unwrap();
        assert_eq!(loaded, 2);
        assert_eq!(cached.cache_size(), 2);

        // 再次预热相同的键不会增加
        let loaded2 = cached.warm_cache(&["key1", "key2"]).await.unwrap();
        assert_eq!(loaded2, 0);
    }

    #[tokio::test]
    async fn test_cached_storage_hit_rate() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();

        // 5 次读取（全部缓存命中）
        for _ in 0..5 {
            cached.read("key1").await.unwrap();
        }

        let hit_rate = cached.cache_hit_rate();
        assert!((hit_rate - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cached_storage_cache_on_write_disabled() {
        let backend = MemoryStorage::new();
        let config = CachedStorageConfig {
            cache_on_write: false,
            ..Default::default()
        };
        let cached = CachedStorage::with_config(backend, config);

        cached.write("key1", b"data").await.unwrap();

        // 缓存应该为空（因为禁用了写入缓存）
        assert_eq!(cached.cache_size(), 0);

        // 读取会触发缓存填充
        cached.read("key1").await.unwrap();
        assert_eq!(cached.cache_size(), 1);
    }

    #[tokio::test]
    async fn test_cached_storage_cache_on_read_miss_disabled() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"data").await.unwrap();

        let config = CachedStorageConfig {
            cache_on_read_miss: false,
            ..Default::default()
        };
        let cached = CachedStorage::with_config(backend, config);

        // 读取（不填充缓存）
        cached.read("key1").await.unwrap();
        assert_eq!(cached.cache_size(), 0);

        // 再次读取仍然是缓存未命中
        cached.read("key1").await.unwrap();
        let stats = cached.combined_stats();
        assert_eq!(stats.cache_misses, 2);
        assert_eq!(stats.backend_reads, 2);
    }

    #[tokio::test]
    async fn test_cached_storage_tier_sizes() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();

        let (hot, warm, cold) = cached.cache_tier_sizes();
        assert_eq!(cold, 1); // 新写入的数据在冷层
        assert_eq!(warm, 0);
        assert_eq!(hot, 0);
    }

    #[tokio::test]
    async fn test_cached_storage_combined_stats() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();
        cached.read("key1").await.unwrap(); // 缓存命中

        let stats = cached.combined_stats();
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.backend_writes, 1);
        assert!(stats.hit_rate() > 0.0);
    }

    #[tokio::test]
    async fn test_cached_storage_backend_read_savings() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"data1").await.unwrap();

        // 10 次读取
        for _ in 0..10 {
            cached.read("key1").await.unwrap();
        }

        let stats = cached.combined_stats();
        // 所有读取都是缓存命中，节省了 100% 的后端读取
        assert!((stats.backend_read_savings() - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cached_storage_overwrite() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        cached.write("key1", b"original").await.unwrap();
        cached.write("key1", b"updated").await.unwrap();

        let data = cached.read("key1").await.unwrap();
        assert_eq!(data, b"updated");

        // 后端也应该更新
        let backend_data = cached.backend().read("key1").await.unwrap();
        assert_eq!(backend_data, b"updated");
    }

    #[tokio::test]
    async fn test_cached_storage_default_config() {
        let config = CachedStorageConfig::default();

        assert!(config.cache_on_write);
        assert!(config.cache_on_read_miss);
        assert_eq!(config.cache_config.hot_capacity, 50);
        assert_eq!(config.cache_config.warm_capacity, 200);
        assert_eq!(config.cache_config.cold_capacity, 1000);
    }

    #[tokio::test]
    async fn test_cached_storage_read_not_found() {
        let backend = MemoryStorage::new();
        let cached = CachedStorage::new(backend);

        let result = cached.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
}
