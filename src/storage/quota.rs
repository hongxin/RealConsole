//! QuotaStorage - 配额管理存储层
//!
//! v1.77.0: 提供存储配额和限制管理
//!
//! ## 功能特性
//!
//! - **键数量限制**: 限制最大键数量
//! - **存储大小限制**: 限制总存储字节数
//! - **单键大小限制**: 限制单个值的最大字节数
//! - **配额策略**: Reject（拒绝）/ Warn（警告）
//! - **使用量追踪**: 实时追踪存储使用情况
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{QuotaStorage, MemoryStorage, QuotaConfig};
//!
//! let storage = QuotaStorageBuilder::new(MemoryStorage::new())
//!     .max_keys(1000)
//!     .max_total_bytes(10 * 1024 * 1024)  // 10 MB
//!     .max_value_bytes(1024 * 1024)        // 1 MB per value
//!     .build();
//!
//! // 正常写入
//! storage.write("key1", b"value1").await?;
//!
//! // 查看使用量
//! let usage = storage.usage();
//! println!("Keys: {}/{}", usage.key_count, usage.max_keys);
//! println!("Bytes: {}/{}", usage.total_bytes, usage.max_bytes);
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// 配额错误
// ============================================================================

/// 配额错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaError {
    /// 超过键数量限制
    MaxKeysExceeded { current: usize, limit: usize },
    /// 超过总存储大小限制
    MaxBytesExceeded { current: u64, limit: u64, needed: usize },
    /// 超过单值大小限制
    ValueTooLarge { size: usize, limit: usize },
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaError::MaxKeysExceeded { current, limit } => {
                write!(f, "Max keys exceeded: {} / {}", current, limit)
            }
            QuotaError::MaxBytesExceeded { current, limit, needed } => {
                write!(
                    f,
                    "Max bytes exceeded: {} + {} > {}",
                    current, needed, limit
                )
            }
            QuotaError::ValueTooLarge { size, limit } => {
                write!(f, "Value too large: {} bytes (limit: {})", size, limit)
            }
        }
    }
}

impl std::error::Error for QuotaError {}

// ============================================================================
// 配额策略
// ============================================================================

/// 配额策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuotaPolicy {
    /// 拒绝超配额的写入
    #[default]
    Reject,
    /// 警告但允许写入
    Warn,
}

// ============================================================================
// 配额配置
// ============================================================================

/// 配额配置
#[derive(Debug, Clone)]
pub struct QuotaConfig {
    /// 最大键数量（None = 无限制）
    pub max_keys: Option<usize>,
    /// 最大总存储字节数（None = 无限制）
    pub max_total_bytes: Option<u64>,
    /// 单个值的最大字节数（None = 无限制）
    pub max_value_bytes: Option<usize>,
    /// 配额策略
    pub policy: QuotaPolicy,
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            max_keys: None,
            max_total_bytes: None,
            max_value_bytes: None,
            policy: QuotaPolicy::Reject,
        }
    }
}

impl QuotaConfig {
    /// 创建新配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大键数量
    pub fn with_max_keys(mut self, max: usize) -> Self {
        self.max_keys = Some(max);
        self
    }

    /// 设置最大总字节数
    pub fn with_max_total_bytes(mut self, max: u64) -> Self {
        self.max_total_bytes = Some(max);
        self
    }

    /// 设置单值最大字节数
    pub fn with_max_value_bytes(mut self, max: usize) -> Self {
        self.max_value_bytes = Some(max);
        self
    }

    /// 设置配额策略
    pub fn with_policy(mut self, policy: QuotaPolicy) -> Self {
        self.policy = policy;
        self
    }
}

// ============================================================================
// 使用量信息
// ============================================================================

/// 存储使用量
#[derive(Debug, Clone)]
pub struct QuotaUsage {
    /// 当前键数量
    pub key_count: usize,
    /// 最大键数量限制
    pub max_keys: Option<usize>,
    /// 当前总字节数
    pub total_bytes: u64,
    /// 最大总字节数限制
    pub max_total_bytes: Option<u64>,
    /// 单值最大字节数限制
    pub max_value_bytes: Option<usize>,
}

impl QuotaUsage {
    /// 键数量使用百分比
    pub fn key_usage_percent(&self) -> Option<f64> {
        self.max_keys.map(|max| {
            if max == 0 {
                100.0
            } else {
                (self.key_count as f64 / max as f64) * 100.0
            }
        })
    }

    /// 存储空间使用百分比
    pub fn bytes_usage_percent(&self) -> Option<f64> {
        self.max_total_bytes.map(|max| {
            if max == 0 {
                100.0
            } else {
                (self.total_bytes as f64 / max as f64) * 100.0
            }
        })
    }

    /// 剩余可用键数量
    pub fn remaining_keys(&self) -> Option<usize> {
        self.max_keys.map(|max| max.saturating_sub(self.key_count))
    }

    /// 剩余可用字节数
    pub fn remaining_bytes(&self) -> Option<u64> {
        self.max_total_bytes
            .map(|max| max.saturating_sub(self.total_bytes))
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 配额统计
#[derive(Debug, Default)]
pub struct QuotaStats {
    /// 拒绝的写入次数
    rejected_writes: AtomicU64,
    /// 警告的写入次数
    warned_writes: AtomicU64,
    /// 成功的写入次数
    successful_writes: AtomicU64,
    /// 因键数量限制拒绝
    rejected_by_keys: AtomicU64,
    /// 因总大小限制拒绝
    rejected_by_bytes: AtomicU64,
    /// 因单值大小限制拒绝
    rejected_by_value_size: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct QuotaStatsSnapshot {
    pub rejected_writes: u64,
    pub warned_writes: u64,
    pub successful_writes: u64,
    pub rejected_by_keys: u64,
    pub rejected_by_bytes: u64,
    pub rejected_by_value_size: u64,
}

impl QuotaStatsSnapshot {
    /// 拒绝率
    pub fn rejection_rate(&self) -> f64 {
        let total = self.rejected_writes + self.successful_writes;
        if total == 0 {
            0.0
        } else {
            self.rejected_writes as f64 / total as f64
        }
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedQuotaStats {
    /// 快照统计
    pub snapshot: QuotaStatsSnapshot,
    /// 使用量
    pub usage: QuotaUsage,
    /// 底层存储统计
    pub backend_stats: StorageStats,
}

// ============================================================================
// 键大小追踪
// ============================================================================

/// 键大小映射
struct KeySizeTracker {
    sizes: RwLock<std::collections::HashMap<String, usize>>,
}

impl KeySizeTracker {
    fn new() -> Self {
        Self {
            sizes: RwLock::new(std::collections::HashMap::new()),
        }
    }

    async fn set(&self, key: &str, size: usize) -> Option<usize> {
        self.sizes.write().await.insert(key.to_string(), size)
    }

    async fn get(&self, key: &str) -> Option<usize> {
        self.sizes.read().await.get(key).copied()
    }

    async fn remove(&self, key: &str) -> Option<usize> {
        self.sizes.write().await.remove(key)
    }

    async fn count(&self) -> usize {
        self.sizes.read().await.len()
    }

    async fn total_size(&self) -> u64 {
        self.sizes.read().await.values().map(|&s| s as u64).sum()
    }
}

// ============================================================================
// QuotaStorage 实现
// ============================================================================

/// 配额存储层
///
/// 装饰器模式，包装底层存储并强制执行配额限制
pub struct QuotaStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配额配置
    config: QuotaConfig,
    /// 键大小追踪
    key_sizes: KeySizeTracker,
    /// 当前键数量
    key_count: AtomicUsize,
    /// 当前总字节数
    total_bytes: AtomicU64,
    /// 统计信息
    stats: Arc<QuotaStats>,
    /// 警告回调
    warn_callback: Option<Arc<dyn Fn(QuotaError) + Send + Sync>>,
}

impl<B: StorageBackend> QuotaStorage<B> {
    /// 创建新的 QuotaStorage
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, QuotaConfig::default())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_config(backend, QuotaConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(backend: B, config: QuotaConfig) -> Self {
        Self::from_arc_with_config(Arc::new(backend), config)
    }

    /// 从 Arc 使用配置创建
    pub fn from_arc_with_config(backend: Arc<B>, config: QuotaConfig) -> Self {
        Self {
            backend,
            config,
            key_sizes: KeySizeTracker::new(),
            key_count: AtomicUsize::new(0),
            total_bytes: AtomicU64::new(0),
            stats: Arc::new(QuotaStats::default()),
            warn_callback: None,
        }
    }

    /// 设置警告回调
    pub fn with_warn_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(QuotaError) + Send + Sync + 'static,
    {
        self.warn_callback = Some(Arc::new(callback));
        self
    }

    /// 获取使用量
    pub fn usage(&self) -> QuotaUsage {
        QuotaUsage {
            key_count: self.key_count.load(Ordering::SeqCst),
            max_keys: self.config.max_keys,
            total_bytes: self.total_bytes.load(Ordering::SeqCst),
            max_total_bytes: self.config.max_total_bytes,
            max_value_bytes: self.config.max_value_bytes,
        }
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> QuotaStatsSnapshot {
        QuotaStatsSnapshot {
            rejected_writes: self.stats.rejected_writes.load(Ordering::SeqCst),
            warned_writes: self.stats.warned_writes.load(Ordering::SeqCst),
            successful_writes: self.stats.successful_writes.load(Ordering::SeqCst),
            rejected_by_keys: self.stats.rejected_by_keys.load(Ordering::SeqCst),
            rejected_by_bytes: self.stats.rejected_by_bytes.load(Ordering::SeqCst),
            rejected_by_value_size: self.stats.rejected_by_value_size.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub fn detailed_stats(&self) -> DetailedQuotaStats {
        DetailedQuotaStats {
            snapshot: self.stats_snapshot(),
            usage: self.usage(),
            backend_stats: self.backend.stats(),
        }
    }

    /// 检查配额
    async fn check_quota(&self, key: &str, data_size: usize) -> Result<(), QuotaError> {
        // 检查单值大小限制
        if let Some(max_value) = self.config.max_value_bytes {
            if data_size > max_value {
                return Err(QuotaError::ValueTooLarge {
                    size: data_size,
                    limit: max_value,
                });
            }
        }

        // 检查是否是新键
        let existing_size = self.key_sizes.get(key).await;
        let is_new_key = existing_size.is_none();
        let size_diff = if let Some(old_size) = existing_size {
            data_size as i64 - old_size as i64
        } else {
            data_size as i64
        };

        // 检查键数量限制
        if is_new_key {
            if let Some(max_keys) = self.config.max_keys {
                let current = self.key_count.load(Ordering::SeqCst);
                if current >= max_keys {
                    return Err(QuotaError::MaxKeysExceeded {
                        current,
                        limit: max_keys,
                    });
                }
            }
        }

        // 检查总大小限制
        if size_diff > 0 {
            if let Some(max_bytes) = self.config.max_total_bytes {
                let current = self.total_bytes.load(Ordering::SeqCst);
                if current + size_diff as u64 > max_bytes {
                    return Err(QuotaError::MaxBytesExceeded {
                        current,
                        limit: max_bytes,
                        needed: size_diff as usize,
                    });
                }
            }
        }

        Ok(())
    }

    /// 处理配额违规
    fn handle_quota_violation(&self, error: &QuotaError) -> bool {
        match self.config.policy {
            QuotaPolicy::Reject => {
                self.stats.rejected_writes.fetch_add(1, Ordering::SeqCst);
                match error {
                    QuotaError::MaxKeysExceeded { .. } => {
                        self.stats.rejected_by_keys.fetch_add(1, Ordering::SeqCst);
                    }
                    QuotaError::MaxBytesExceeded { .. } => {
                        self.stats.rejected_by_bytes.fetch_add(1, Ordering::SeqCst);
                    }
                    QuotaError::ValueTooLarge { .. } => {
                        self.stats.rejected_by_value_size.fetch_add(1, Ordering::SeqCst);
                    }
                }
                false // 不允许继续
            }
            QuotaPolicy::Warn => {
                self.stats.warned_writes.fetch_add(1, Ordering::SeqCst);
                if let Some(ref callback) = self.warn_callback {
                    callback(error.clone());
                }
                true // 允许继续
            }
        }
    }

    /// 更新追踪信息
    async fn update_tracking(&self, key: &str, new_size: usize) {
        let old_size = self.key_sizes.set(key, new_size).await;

        if let Some(old) = old_size {
            // 更新现有键
            if new_size > old {
                self.total_bytes
                    .fetch_add((new_size - old) as u64, Ordering::SeqCst);
            } else {
                self.total_bytes
                    .fetch_sub((old - new_size) as u64, Ordering::SeqCst);
            }
        } else {
            // 新键
            self.key_count.fetch_add(1, Ordering::SeqCst);
            self.total_bytes.fetch_add(new_size as u64, Ordering::SeqCst);
        }
    }

    /// 移除追踪信息
    async fn remove_tracking(&self, key: &str) {
        if let Some(size) = self.key_sizes.remove(key).await {
            self.key_count.fetch_sub(1, Ordering::SeqCst);
            self.total_bytes.fetch_sub(size as u64, Ordering::SeqCst);
        }
    }

    /// 同步使用量（从底层存储重新计算）
    pub async fn sync_usage(&self) -> StorageResult<()> {
        let keys = self.backend.list("").await?;
        let mut total_bytes = 0u64;

        {
            let mut sizes = self.key_sizes.sizes.write().await;
            sizes.clear();

            for key in &keys {
                if let Ok(data) = self.backend.read(key).await {
                    let size = data.len();
                    sizes.insert(key.clone(), size);
                    total_bytes += size as u64;
                }
            }
        }

        self.key_count.store(keys.len(), Ordering::SeqCst);
        self.total_bytes.store(total_bytes, Ordering::SeqCst);

        Ok(())
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for QuotaStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        // 检查配额
        if let Err(error) = self.check_quota(key, data.len()).await {
            if !self.handle_quota_violation(&error) {
                return Err(StorageError::Other(error.to_string()));
            }
        }

        // 执行写入
        self.backend.write(key, data).await?;

        // 更新追踪
        self.update_tracking(key, data.len()).await;
        self.stats.successful_writes.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // 先检查是否存在
        let existed = self.key_sizes.get(key).await.is_some();

        // 执行删除
        self.backend.delete(key).await?;

        // 更新追踪
        if existed {
            self.remove_tracking(key).await;
        }

        Ok(())
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let mut stats = self.backend.stats();
        stats.key_count = self.key_count.load(Ordering::SeqCst);
        stats.total_bytes = self.total_bytes.load(Ordering::SeqCst);
        stats
    }

    fn name(&self) -> &'static str {
        "QuotaStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// QuotaStorage 构建器
pub struct QuotaStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: QuotaConfig,
    warn_callback: Option<Arc<dyn Fn(QuotaError) + Send + Sync>>,
}

impl<B: StorageBackend> QuotaStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: QuotaConfig::default(),
            warn_callback: None,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: QuotaConfig::default(),
            warn_callback: None,
        }
    }

    /// 设置最大键数量
    pub fn max_keys(mut self, max: usize) -> Self {
        self.config.max_keys = Some(max);
        self
    }

    /// 设置最大总字节数
    pub fn max_total_bytes(mut self, max: u64) -> Self {
        self.config.max_total_bytes = Some(max);
        self
    }

    /// 设置单值最大字节数
    pub fn max_value_bytes(mut self, max: usize) -> Self {
        self.config.max_value_bytes = Some(max);
        self
    }

    /// 设置配额策略
    pub fn policy(mut self, policy: QuotaPolicy) -> Self {
        self.config.policy = policy;
        self
    }

    /// 设置警告回调
    pub fn warn_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(QuotaError) + Send + Sync + 'static,
    {
        self.warn_callback = Some(Arc::new(callback));
        self
    }

    /// 构建
    pub fn build(self) -> QuotaStorage<B> {
        let mut storage = QuotaStorage::from_arc_with_config(self.backend, self.config);
        storage.warn_callback = self.warn_callback;
        storage
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
    async fn test_quota_storage_basic() {
        let storage = QuotaStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_max_keys_limit() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(2)
            .build();

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        // 第三个键应该被拒绝
        let result = storage.write("key3", b"v3").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max keys exceeded"));
    }

    #[tokio::test]
    async fn test_max_total_bytes_limit() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_total_bytes(10)
            .build();

        storage.write("key1", b"12345").await.unwrap(); // 5 bytes
        storage.write("key2", b"123").await.unwrap(); // 3 bytes, total 8

        // 超过限制的写入应该被拒绝
        let result = storage.write("key3", b"12345").await; // 需要 5 bytes，总计 13 > 10
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Max bytes exceeded"));
    }

    #[tokio::test]
    async fn test_max_value_bytes_limit() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_value_bytes(5)
            .build();

        storage.write("key1", b"12345").await.unwrap(); // 正好 5 bytes

        // 超过单值限制的写入应该被拒绝
        let result = storage.write("key2", b"123456").await; // 6 bytes > 5
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Value too large"));
    }

    #[tokio::test]
    async fn test_update_existing_key() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(2)
            .max_total_bytes(20)
            .build();

        storage.write("key1", b"12345").await.unwrap(); // 5 bytes
        storage.write("key2", b"12345").await.unwrap(); // 5 bytes, total 10

        // 更新现有键不应该增加键数量
        storage.write("key1", b"1234567890").await.unwrap(); // 10 bytes

        let usage = storage.usage();
        assert_eq!(usage.key_count, 2);
        assert_eq!(usage.total_bytes, 15); // 10 + 5
    }

    #[tokio::test]
    async fn test_delete_updates_tracking() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(2)
            .build();

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        assert_eq!(storage.usage().key_count, 2);

        // 删除一个键后应该可以添加新键
        storage.delete("key1").await.unwrap();
        assert_eq!(storage.usage().key_count, 1);

        storage.write("key3", b"v3").await.unwrap();
        assert_eq!(storage.usage().key_count, 2);
    }

    #[tokio::test]
    async fn test_warn_policy() {
        use std::sync::Mutex;

        let warnings = Arc::new(Mutex::new(Vec::new()));
        let warnings_clone = Arc::clone(&warnings);

        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(1)
            .policy(QuotaPolicy::Warn)
            .warn_callback(move |e| {
                warnings_clone.lock().unwrap().push(e);
            })
            .build();

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap(); // 超过限制但允许

        // 验证写入成功
        assert!(storage.exists("key2").await.unwrap());

        // 验证警告被触发
        let warns = warnings.lock().unwrap();
        assert_eq!(warns.len(), 1);
        assert!(matches!(warns[0], QuotaError::MaxKeysExceeded { .. }));
    }

    #[tokio::test]
    async fn test_usage_percentages() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(10)
            .max_total_bytes(100)
            .build();

        storage.write("key1", b"0123456789").await.unwrap(); // 10 bytes

        let usage = storage.usage();
        assert_eq!(usage.key_usage_percent(), Some(10.0));
        assert_eq!(usage.bytes_usage_percent(), Some(10.0));
        assert_eq!(usage.remaining_keys(), Some(9));
        assert_eq!(usage.remaining_bytes(), Some(90));
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(2)
            .build();

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();
        let _ = storage.write("key3", b"v3").await; // 被拒绝

        let stats = storage.stats_snapshot();
        assert_eq!(stats.successful_writes, 2);
        assert_eq!(stats.rejected_writes, 1);
        assert_eq!(stats.rejected_by_keys, 1);
    }

    #[tokio::test]
    async fn test_rejection_rate() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(1)
            .build();

        storage.write("key1", b"v1").await.unwrap();
        let _ = storage.write("key2", b"v2").await;
        let _ = storage.write("key3", b"v3").await;

        let stats = storage.stats_snapshot();
        // 1 成功，2 拒绝，拒绝率 = 2/3 ≈ 0.667
        assert!((stats.rejection_rate() - 0.667).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_sync_usage() {
        let backend = Arc::new(MemoryStorage::new());
        backend.write("key1", b"12345").await.unwrap();
        backend.write("key2", b"67890").await.unwrap();

        let storage = QuotaStorage::from_arc(backend);

        // 初始追踪为空
        assert_eq!(storage.usage().key_count, 0);

        // 同步后应该正确
        storage.sync_usage().await.unwrap();
        let usage = storage.usage();
        assert_eq!(usage.key_count, 2);
        assert_eq!(usage.total_bytes, 10);
    }

    #[tokio::test]
    async fn test_builder() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_keys(100)
            .max_total_bytes(1024)
            .max_value_bytes(256)
            .policy(QuotaPolicy::Reject)
            .build();

        let usage = storage.usage();
        assert_eq!(usage.max_keys, Some(100));
        assert_eq!(usage.max_total_bytes, Some(1024));
        assert_eq!(usage.max_value_bytes, Some(256));
    }

    #[tokio::test]
    async fn test_quota_error_display() {
        let err1 = QuotaError::MaxKeysExceeded {
            current: 10,
            limit: 10,
        };
        assert!(err1.to_string().contains("Max keys exceeded"));

        let err2 = QuotaError::MaxBytesExceeded {
            current: 100,
            limit: 100,
            needed: 10,
        };
        assert!(err2.to_string().contains("Max bytes exceeded"));

        let err3 = QuotaError::ValueTooLarge {
            size: 1000,
            limit: 100,
        };
        assert!(err3.to_string().contains("Value too large"));
    }

    #[tokio::test]
    async fn test_quota_config_builder() {
        let config = QuotaConfig::new()
            .with_max_keys(50)
            .with_max_total_bytes(1024)
            .with_max_value_bytes(128)
            .with_policy(QuotaPolicy::Warn);

        assert_eq!(config.max_keys, Some(50));
        assert_eq!(config.max_total_bytes, Some(1024));
        assert_eq!(config.max_value_bytes, Some(128));
        assert_eq!(config.policy, QuotaPolicy::Warn);
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = QuotaStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        assert_eq!(storage.usage().key_count, 1);
    }

    #[tokio::test]
    async fn test_no_limits() {
        let storage = QuotaStorage::new(MemoryStorage::new());

        // 无限制时应该都能写入
        for i in 0..100 {
            storage
                .write(&format!("key{}", i), b"value")
                .await
                .unwrap();
        }

        assert_eq!(storage.usage().key_count, 100);
    }

    #[tokio::test]
    async fn test_shrink_value_size() {
        let storage = QuotaStorageBuilder::new(MemoryStorage::new())
            .max_total_bytes(100)
            .build();

        storage.write("key1", b"0123456789").await.unwrap(); // 10 bytes
        assert_eq!(storage.usage().total_bytes, 10);

        // 缩小值
        storage.write("key1", b"12345").await.unwrap(); // 5 bytes
        assert_eq!(storage.usage().total_bytes, 5);
    }
}
