//! TTLStorage - 过期时间存储层
//!
//! v1.80.0: 提供数据自动过期和清理
//!
//! ## 功能特性
//!
//! - **默认 TTL**: 所有键的默认过期时间
//! - **独立 TTL**: 每个键可设置独立过期时间
//! - **惰性过期**: 读取时检查过期
//! - **滑动过期**: 访问时刷新 TTL（可选）
//! - **过期回调**: 数据过期时通知
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{TTLStorage, MemoryStorage};
//! use std::time::Duration;
//!
//! let storage = TTLStorageBuilder::new(MemoryStorage::new())
//!     .default_ttl(Duration::from_secs(3600))  // 1 hour default
//!     .build();
//!
//! // 使用默认 TTL 写入
//! storage.write("key1", b"value1").await?;
//!
//! // 使用自定义 TTL 写入
//! storage.write_with_ttl("key2", b"value2", Duration::from_secs(60)).await?;
//!
//! // 过期后读取返回 NotFound
//! tokio::time::sleep(Duration::from_secs(61)).await;
//! let result = storage.read("key2").await;  // Err(NotFound)
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Type alias for callback types to satisfy clippy::type_complexity
type ExpirationCallback = Arc<dyn Fn(&str) + Send + Sync>;

// ============================================================================
// 过期信息
// ============================================================================

/// 键的过期信息
#[derive(Debug, Clone)]
struct ExpirationInfo {
    /// 过期时间点
    expires_at: Instant,
    /// 原始 TTL（用于滑动过期）
    ttl: Duration,
}

impl ExpirationInfo {
    fn new(ttl: Duration) -> Self {
        Self {
            expires_at: Instant::now() + ttl,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    fn remaining(&self) -> Duration {
        let now = Instant::now();
        if now >= self.expires_at {
            Duration::ZERO
        } else {
            self.expires_at - now
        }
    }

    fn refresh(&mut self) {
        self.expires_at = Instant::now() + self.ttl;
    }
}

// ============================================================================
// 配置
// ============================================================================

/// TTL 配置
#[derive(Debug, Clone)]
pub struct TTLConfig {
    /// 默认 TTL（None = 永不过期）
    pub default_ttl: Option<Duration>,
    /// 是否启用滑动过期（读取时刷新 TTL）
    pub sliding_expiration: bool,
    /// 是否在读取时检查过期（惰性过期）
    pub lazy_expiration: bool,
}

impl Default for TTLConfig {
    fn default() -> Self {
        Self {
            default_ttl: None,
            sliding_expiration: false,
            lazy_expiration: true,
        }
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// TTL 统计
#[derive(Debug, Default)]
pub struct TTLStats {
    /// 写入次数
    writes: AtomicU64,
    /// 带 TTL 的写入次数
    writes_with_ttl: AtomicU64,
    /// 过期键数量
    expired_keys: AtomicU64,
    /// 惰性过期检测数量
    lazy_expirations: AtomicU64,
    /// 滑动过期刷新次数
    ttl_refreshes: AtomicU64,
    /// 清理操作次数
    cleanup_runs: AtomicU64,
    /// 清理删除的键数量
    cleanup_removed: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct TTLStatsSnapshot {
    pub writes: u64,
    pub writes_with_ttl: u64,
    pub expired_keys: u64,
    pub lazy_expirations: u64,
    pub ttl_refreshes: u64,
    pub cleanup_runs: u64,
    pub cleanup_removed: u64,
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedTTLStats {
    /// 快照统计
    pub snapshot: TTLStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 当前追踪的键数量
    pub tracked_keys: usize,
    /// 已过期但未清理的键数量
    pub pending_expiration: usize,
}

// ============================================================================
// TTLStorage 实现
// ============================================================================

/// TTL 存储层
///
/// 装饰器模式，包装底层存储并提供数据过期功能
pub struct TTLStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配置
    config: TTLConfig,
    /// 过期信息映射
    expirations: Arc<RwLock<HashMap<String, ExpirationInfo>>>,
    /// 统计信息
    stats: Arc<TTLStats>,
    /// 过期回调
    expiration_callback: Option<ExpirationCallback>,
}

impl<B: StorageBackend> TTLStorage<B> {
    /// 创建新的 TTLStorage
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, TTLConfig::default())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_config(backend, TTLConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(backend: B, config: TTLConfig) -> Self {
        Self::from_arc_with_config(Arc::new(backend), config)
    }

    /// 从 Arc 使用配置创建
    pub fn from_arc_with_config(backend: Arc<B>, config: TTLConfig) -> Self {
        Self {
            backend,
            config,
            expirations: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(TTLStats::default()),
            expiration_callback: None,
        }
    }

    /// 设置过期回调
    pub fn with_expiration_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.expiration_callback = Some(Arc::new(callback));
        self
    }

    /// 使用自定义 TTL 写入
    pub async fn write_with_ttl(&self, key: &str, data: &[u8], ttl: Duration) -> StorageResult<()> {
        self.backend.write(key, data).await?;

        self.expirations
            .write()
            .await
            .insert(key.to_string(), ExpirationInfo::new(ttl));

        self.stats.writes.fetch_add(1, Ordering::SeqCst);
        self.stats.writes_with_ttl.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    /// 获取键的剩余 TTL
    pub async fn ttl(&self, key: &str) -> Option<Duration> {
        let expirations = self.expirations.read().await;
        expirations.get(key).map(|info| info.remaining())
    }

    /// 设置键的 TTL（不影响数据）
    pub async fn set_ttl(&self, key: &str, ttl: Duration) -> bool {
        let mut expirations = self.expirations.write().await;
        if expirations.contains_key(key) {
            expirations.insert(key.to_string(), ExpirationInfo::new(ttl));
            true
        } else {
            // 检查键是否存在于后端
            if self.backend.exists(key).await.unwrap_or(false) {
                expirations.insert(key.to_string(), ExpirationInfo::new(ttl));
                true
            } else {
                false
            }
        }
    }

    /// 移除键的 TTL（永不过期）
    pub async fn persist(&self, key: &str) -> bool {
        self.expirations.write().await.remove(key).is_some()
    }

    /// 刷新键的 TTL
    pub async fn refresh_ttl(&self, key: &str) -> bool {
        let mut expirations = self.expirations.write().await;
        if let Some(info) = expirations.get_mut(key) {
            info.refresh();
            self.stats.ttl_refreshes.fetch_add(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    /// 检查键是否过期
    async fn is_expired(&self, key: &str) -> bool {
        let expirations = self.expirations.read().await;
        expirations.get(key).map(|i| i.is_expired()).unwrap_or(false)
    }

    /// 处理惰性过期
    async fn handle_lazy_expiration(&self, key: &str) -> bool {
        if !self.config.lazy_expiration {
            return false;
        }

        if self.is_expired(key).await {
            // 从后端删除
            let _ = self.backend.delete(key).await;

            // 从过期映射中移除
            self.expirations.write().await.remove(key);

            // 更新统计
            self.stats.expired_keys.fetch_add(1, Ordering::SeqCst);
            self.stats.lazy_expirations.fetch_add(1, Ordering::SeqCst);

            // 触发回调
            if let Some(ref callback) = self.expiration_callback {
                callback(key);
            }

            true
        } else {
            false
        }
    }

    /// 处理滑动过期
    async fn handle_sliding_expiration(&self, key: &str) {
        if self.config.sliding_expiration {
            self.refresh_ttl(key).await;
        }
    }

    /// 清理所有过期的键
    pub async fn cleanup(&self) -> usize {
        self.stats.cleanup_runs.fetch_add(1, Ordering::SeqCst);

        let expired_keys: Vec<String> = {
            let expirations = self.expirations.read().await;
            expirations
                .iter()
                .filter(|(_, info)| info.is_expired())
                .map(|(k, _)| k.clone())
                .collect()
        };

        let count = expired_keys.len();

        for key in &expired_keys {
            let _ = self.backend.delete(key).await;

            if let Some(ref callback) = self.expiration_callback {
                callback(key);
            }
        }

        {
            let mut expirations = self.expirations.write().await;
            for key in &expired_keys {
                expirations.remove(key);
            }
        }

        self.stats
            .expired_keys
            .fetch_add(count as u64, Ordering::SeqCst);
        self.stats
            .cleanup_removed
            .fetch_add(count as u64, Ordering::SeqCst);

        count
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> TTLStatsSnapshot {
        TTLStatsSnapshot {
            writes: self.stats.writes.load(Ordering::SeqCst),
            writes_with_ttl: self.stats.writes_with_ttl.load(Ordering::SeqCst),
            expired_keys: self.stats.expired_keys.load(Ordering::SeqCst),
            lazy_expirations: self.stats.lazy_expirations.load(Ordering::SeqCst),
            ttl_refreshes: self.stats.ttl_refreshes.load(Ordering::SeqCst),
            cleanup_runs: self.stats.cleanup_runs.load(Ordering::SeqCst),
            cleanup_removed: self.stats.cleanup_removed.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> DetailedTTLStats {
        let expirations = self.expirations.read().await;
        let pending = expirations.values().filter(|i| i.is_expired()).count();

        DetailedTTLStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            tracked_keys: expirations.len(),
            pending_expiration: pending,
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for TTLStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        // 惰性过期检查
        if self.handle_lazy_expiration(key).await {
            return Err(StorageError::NotFound(key.to_string()));
        }

        let result = self.backend.read(key).await;

        // 滑动过期处理
        if result.is_ok() {
            self.handle_sliding_expiration(key).await;
        }

        result
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.backend.write(key, data).await?;

        // 如果有默认 TTL，设置过期时间
        if let Some(ttl) = self.config.default_ttl {
            self.expirations
                .write()
                .await
                .insert(key.to_string(), ExpirationInfo::new(ttl));
            self.stats.writes_with_ttl.fetch_add(1, Ordering::SeqCst);
        }

        self.stats.writes.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.expirations.write().await.remove(key);
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 获取后端列表
        let keys = self.backend.list(prefix).await?;

        // 过滤掉已过期的键
        let expirations = self.expirations.read().await;
        let valid_keys: Vec<String> = keys
            .into_iter()
            .filter(|k| {
                expirations
                    .get(k)
                    .map(|i| !i.is_expired())
                    .unwrap_or(true)
            })
            .collect();

        Ok(valid_keys)
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // 惰性过期检查
        if self.handle_lazy_expiration(key).await {
            return Ok(false);
        }

        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "TTLStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// TTLStorage 构建器
pub struct TTLStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: TTLConfig,
    expiration_callback: Option<ExpirationCallback>,
}

impl<B: StorageBackend> TTLStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: TTLConfig::default(),
            expiration_callback: None,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: TTLConfig::default(),
            expiration_callback: None,
        }
    }

    /// 设置默认 TTL
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.config.default_ttl = Some(ttl);
        self
    }

    /// 启用滑动过期
    pub fn sliding_expiration(mut self, enabled: bool) -> Self {
        self.config.sliding_expiration = enabled;
        self
    }

    /// 启用惰性过期
    pub fn lazy_expiration(mut self, enabled: bool) -> Self {
        self.config.lazy_expiration = enabled;
        self
    }

    /// 设置过期回调
    pub fn expiration_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.expiration_callback = Some(Arc::new(callback));
        self
    }

    /// 构建
    pub fn build(self) -> TTLStorage<B> {
        TTLStorage {
            backend: self.backend,
            config: self.config,
            expirations: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(TTLStats::default()),
            expiration_callback: self.expiration_callback,
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
    async fn test_ttl_storage_basic() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_write_with_ttl() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"value1", Duration::from_millis(50))
            .await
            .unwrap();

        // 立即读取应该成功
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");

        // 等待过期
        tokio::time::sleep(Duration::from_millis(60)).await;

        // 过期后读取应该失败
        let result = storage.read("key1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_default_ttl() {
        let storage = TTLStorageBuilder::new(MemoryStorage::new())
            .default_ttl(Duration::from_millis(50))
            .build();

        storage.write("key1", b"value1").await.unwrap();

        // 立即读取应该成功
        assert!(storage.read("key1").await.is_ok());

        // 等待过期
        tokio::time::sleep(Duration::from_millis(60)).await;

        // 过期后读取应该失败
        assert!(storage.read("key1").await.is_err());
    }

    #[tokio::test]
    async fn test_get_ttl() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"value1", Duration::from_secs(10))
            .await
            .unwrap();

        let ttl = storage.ttl("key1").await;
        assert!(ttl.is_some());
        assert!(ttl.unwrap() <= Duration::from_secs(10));
        assert!(ttl.unwrap() > Duration::from_secs(9));
    }

    #[tokio::test]
    async fn test_set_ttl() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();

        // 初始无 TTL
        assert!(storage.ttl("key1").await.is_none());

        // 设置 TTL
        assert!(storage.set_ttl("key1", Duration::from_secs(10)).await);

        // 现在有 TTL
        assert!(storage.ttl("key1").await.is_some());
    }

    #[tokio::test]
    async fn test_persist() {
        let storage = TTLStorageBuilder::new(MemoryStorage::new())
            .default_ttl(Duration::from_millis(50))
            .build();

        storage.write("key1", b"value1").await.unwrap();

        // 有 TTL
        assert!(storage.ttl("key1").await.is_some());

        // 持久化（移除 TTL）
        assert!(storage.persist("key1").await);

        // 无 TTL
        assert!(storage.ttl("key1").await.is_none());

        // 等待原本会过期的时间
        tokio::time::sleep(Duration::from_millis(60)).await;

        // 仍然可读
        assert!(storage.read("key1").await.is_ok());
    }

    #[tokio::test]
    async fn test_sliding_expiration() {
        let storage = TTLStorageBuilder::new(MemoryStorage::new())
            .sliding_expiration(true)
            .build();

        storage
            .write_with_ttl("key1", b"value1", Duration::from_millis(100))
            .await
            .unwrap();

        // 多次读取，每次刷新 TTL
        for _ in 0..5 {
            tokio::time::sleep(Duration::from_millis(30)).await;
            let _ = storage.read("key1").await;
        }

        // 应该仍然有效（因为滑动过期）
        let stats = storage.stats_snapshot();
        assert!(stats.ttl_refreshes > 0);
    }

    #[tokio::test]
    async fn test_refresh_ttl() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"value1", Duration::from_millis(50))
            .await
            .unwrap();

        // 等待一些时间
        tokio::time::sleep(Duration::from_millis(30)).await;

        // 刷新 TTL
        assert!(storage.refresh_ttl("key1").await);

        // 再等待一些时间（总共会超过原始 TTL）
        tokio::time::sleep(Duration::from_millis(40)).await;

        // 应该仍然有效
        assert!(storage.read("key1").await.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"v1", Duration::from_millis(10))
            .await
            .unwrap();
        storage
            .write_with_ttl("key2", b"v2", Duration::from_millis(10))
            .await
            .unwrap();
        storage
            .write_with_ttl("key3", b"v3", Duration::from_secs(60))
            .await
            .unwrap();

        // 等待部分键过期
        tokio::time::sleep(Duration::from_millis(20)).await;

        // 清理
        let removed = storage.cleanup().await;
        assert_eq!(removed, 2);

        // key3 仍然存在
        assert!(storage.read("key3").await.is_ok());
    }

    #[tokio::test]
    async fn test_expiration_callback() {
        use std::sync::Mutex;

        let expired_keys = Arc::new(Mutex::new(Vec::new()));
        let expired_keys_clone = Arc::clone(&expired_keys);

        let storage = TTLStorageBuilder::new(MemoryStorage::new())
            .expiration_callback(move |key| {
                expired_keys_clone.lock().unwrap().push(key.to_string());
            })
            .build();

        storage
            .write_with_ttl("key1", b"v1", Duration::from_millis(10))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;

        // 触发惰性过期
        let _ = storage.read("key1").await;

        let keys = expired_keys.lock().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "key1");
    }

    #[tokio::test]
    async fn test_list_filters_expired() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"v1", Duration::from_millis(10))
            .await
            .unwrap();
        storage
            .write_with_ttl("key2", b"v2", Duration::from_secs(60))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;

        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"key2".to_string()));
    }

    #[tokio::test]
    async fn test_exists_checks_expiration() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"v1", Duration::from_millis(10))
            .await
            .unwrap();

        assert!(storage.exists("key1").await.unwrap());

        tokio::time::sleep(Duration::from_millis(20)).await;

        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_delete_removes_expiration() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"v1", Duration::from_secs(60))
            .await
            .unwrap();

        assert!(storage.ttl("key1").await.is_some());

        storage.delete("key1").await.unwrap();

        assert!(storage.ttl("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = TTLStorageBuilder::new(MemoryStorage::new())
            .default_ttl(Duration::from_millis(10))
            .build();

        storage.write("key1", b"v1").await.unwrap();
        storage
            .write_with_ttl("key2", b"v2", Duration::from_millis(10))
            .await
            .unwrap();

        let stats = storage.stats_snapshot();
        assert_eq!(stats.writes, 2);
        assert_eq!(stats.writes_with_ttl, 2);
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = TTLStorage::new(MemoryStorage::new());

        storage
            .write_with_ttl("key1", b"v1", Duration::from_millis(10))
            .await
            .unwrap();
        storage
            .write_with_ttl("key2", b"v2", Duration::from_secs(60))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(20)).await;

        let detailed = storage.detailed_stats().await;
        assert_eq!(detailed.tracked_keys, 2);
        assert_eq!(detailed.pending_expiration, 1);
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = TTLStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_no_ttl_never_expires() {
        let storage = TTLStorage::new(MemoryStorage::new());

        // 不使用 TTL 写入
        storage.write("key1", b"value1").await.unwrap();

        // 无 TTL
        assert!(storage.ttl("key1").await.is_none());

        // 多次读取仍然有效
        for _ in 0..10 {
            assert!(storage.read("key1").await.is_ok());
        }
    }
}
