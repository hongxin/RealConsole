//! ReadOnlyStorage - 只读存储层
//!
//! v1.81.0: 提供只读访问控制
//!
//! ## 功能特性
//!
//! - **写入阻止**: 阻止所有 write/delete 操作
//! - **读取允许**: 允许 read/list/exists 操作
//! - **白名单模式**: 允许特定键可写（可选）
//! - **访问统计**: 追踪被拒绝的操作
//! - **拒绝回调**: 写入被拒绝时通知
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{ReadOnlyStorage, MemoryStorage};
//!
//! // 先写入数据
//! let backend = MemoryStorage::new();
//! backend.write("key1", b"value1").await?;
//!
//! // 包装为只读
//! let storage = ReadOnlyStorage::new(backend);
//!
//! // 读取成功
//! let data = storage.read("key1").await?;
//!
//! // 写入被拒绝
//! let result = storage.write("key2", b"value2").await;
//! assert!(result.is_err());
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// 只读错误
// ============================================================================

/// 只读错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOnlyError {
    /// 写入被拒绝
    WriteNotAllowed { key: String },
    /// 删除被拒绝
    DeleteNotAllowed { key: String },
}

impl std::fmt::Display for ReadOnlyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadOnlyError::WriteNotAllowed { key } => {
                write!(f, "Write not allowed on read-only storage: {}", key)
            }
            ReadOnlyError::DeleteNotAllowed { key } => {
                write!(f, "Delete not allowed on read-only storage: {}", key)
            }
        }
    }
}

impl std::error::Error for ReadOnlyError {}

// ============================================================================
// 配置
// ============================================================================

/// 只读模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadOnlyMode {
    /// 完全只读
    #[default]
    Strict,
    /// 允许白名单中的键可写
    Whitelist,
    /// 允许特定前缀的键可写
    PrefixWhitelist,
}

/// 只读配置
#[derive(Debug, Clone, Default)]
pub struct ReadOnlyConfig {
    /// 只读模式
    pub mode: ReadOnlyMode,
    /// 是否允许删除（独立于写入控制）
    pub allow_delete: bool,
}

// ============================================================================
// 统计信息
// ============================================================================

/// 只读统计
#[derive(Debug, Default)]
pub struct ReadOnlyStats {
    /// 读取次数
    reads: AtomicU64,
    /// 写入尝试次数
    write_attempts: AtomicU64,
    /// 写入拒绝次数
    write_denied: AtomicU64,
    /// 写入允许次数（白名单）
    write_allowed: AtomicU64,
    /// 删除尝试次数
    delete_attempts: AtomicU64,
    /// 删除拒绝次数
    delete_denied: AtomicU64,
    /// 删除允许次数
    delete_allowed: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct ReadOnlyStatsSnapshot {
    pub reads: u64,
    pub write_attempts: u64,
    pub write_denied: u64,
    pub write_allowed: u64,
    pub delete_attempts: u64,
    pub delete_denied: u64,
    pub delete_allowed: u64,
}

impl ReadOnlyStatsSnapshot {
    /// 写入拒绝率
    pub fn write_deny_rate(&self) -> f64 {
        if self.write_attempts == 0 {
            0.0
        } else {
            self.write_denied as f64 / self.write_attempts as f64
        }
    }

    /// 删除拒绝率
    pub fn delete_deny_rate(&self) -> f64 {
        if self.delete_attempts == 0 {
            0.0
        } else {
            self.delete_denied as f64 / self.delete_attempts as f64
        }
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedReadOnlyStats {
    /// 快照统计
    pub snapshot: ReadOnlyStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 白名单键数量
    pub whitelist_count: usize,
    /// 白名单前缀数量
    pub prefix_whitelist_count: usize,
}

// ============================================================================
// ReadOnlyStorage 实现
// ============================================================================

/// 只读存储层
///
/// 装饰器模式，包装底层存储并阻止写入操作
pub struct ReadOnlyStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配置
    config: ReadOnlyConfig,
    /// 可写键白名单
    key_whitelist: Arc<RwLock<HashSet<String>>>,
    /// 可写前缀白名单
    prefix_whitelist: Arc<RwLock<Vec<String>>>,
    /// 统计信息
    stats: Arc<ReadOnlyStats>,
    /// 拒绝回调
    deny_callback: Option<Arc<dyn Fn(&str, &str) + Send + Sync>>,
}

impl<B: StorageBackend> ReadOnlyStorage<B> {
    /// 创建新的 ReadOnlyStorage（完全只读）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, ReadOnlyConfig::default())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_config(backend, ReadOnlyConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(backend: B, config: ReadOnlyConfig) -> Self {
        Self::from_arc_with_config(Arc::new(backend), config)
    }

    /// 从 Arc 使用配置创建
    pub fn from_arc_with_config(backend: Arc<B>, config: ReadOnlyConfig) -> Self {
        Self {
            backend,
            config,
            key_whitelist: Arc::new(RwLock::new(HashSet::new())),
            prefix_whitelist: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(ReadOnlyStats::default()),
            deny_callback: None,
        }
    }

    /// 设置拒绝回调
    pub fn with_deny_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.deny_callback = Some(Arc::new(callback));
        self
    }

    /// 添加可写键到白名单
    pub async fn allow_key(&self, key: impl Into<String>) {
        self.key_whitelist.write().await.insert(key.into());
    }

    /// 从白名单移除键
    pub async fn disallow_key(&self, key: &str) {
        self.key_whitelist.write().await.remove(key);
    }

    /// 添加可写前缀到白名单
    pub async fn allow_prefix(&self, prefix: impl Into<String>) {
        self.prefix_whitelist.write().await.push(prefix.into());
    }

    /// 检查键是否可写
    async fn is_writable(&self, key: &str) -> bool {
        match self.config.mode {
            ReadOnlyMode::Strict => false,
            ReadOnlyMode::Whitelist => self.key_whitelist.read().await.contains(key),
            ReadOnlyMode::PrefixWhitelist => {
                let prefixes = self.prefix_whitelist.read().await;
                prefixes.iter().any(|p| key.starts_with(p))
            }
        }
    }

    /// 处理写入拒绝
    fn handle_write_denied(&self, key: &str) {
        self.stats.write_denied.fetch_add(1, Ordering::SeqCst);
        if let Some(ref callback) = self.deny_callback {
            callback("write", key);
        }
    }

    /// 处理删除拒绝
    fn handle_delete_denied(&self, key: &str) {
        self.stats.delete_denied.fetch_add(1, Ordering::SeqCst);
        if let Some(ref callback) = self.deny_callback {
            callback("delete", key);
        }
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> ReadOnlyStatsSnapshot {
        ReadOnlyStatsSnapshot {
            reads: self.stats.reads.load(Ordering::SeqCst),
            write_attempts: self.stats.write_attempts.load(Ordering::SeqCst),
            write_denied: self.stats.write_denied.load(Ordering::SeqCst),
            write_allowed: self.stats.write_allowed.load(Ordering::SeqCst),
            delete_attempts: self.stats.delete_attempts.load(Ordering::SeqCst),
            delete_denied: self.stats.delete_denied.load(Ordering::SeqCst),
            delete_allowed: self.stats.delete_allowed.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> DetailedReadOnlyStats {
        DetailedReadOnlyStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            whitelist_count: self.key_whitelist.read().await.len(),
            prefix_whitelist_count: self.prefix_whitelist.read().await.len(),
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for ReadOnlyStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.stats.reads.fetch_add(1, Ordering::SeqCst);
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.write_attempts.fetch_add(1, Ordering::SeqCst);

        if self.is_writable(key).await {
            self.stats.write_allowed.fetch_add(1, Ordering::SeqCst);
            self.backend.write(key, data).await
        } else {
            self.handle_write_denied(key);
            Err(StorageError::Other(
                ReadOnlyError::WriteNotAllowed {
                    key: key.to_string(),
                }
                .to_string(),
            ))
        }
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.stats.delete_attempts.fetch_add(1, Ordering::SeqCst);

        // 检查是否允许删除
        let allowed = if self.config.allow_delete {
            true
        } else {
            self.is_writable(key).await
        };

        if allowed {
            self.stats.delete_allowed.fetch_add(1, Ordering::SeqCst);
            self.backend.delete(key).await
        } else {
            self.handle_delete_denied(key);
            Err(StorageError::Other(
                ReadOnlyError::DeleteNotAllowed {
                    key: key.to_string(),
                }
                .to_string(),
            ))
        }
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "ReadOnlyStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// ReadOnlyStorage 构建器
pub struct ReadOnlyStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: ReadOnlyConfig,
    key_whitelist: HashSet<String>,
    prefix_whitelist: Vec<String>,
    deny_callback: Option<Arc<dyn Fn(&str, &str) + Send + Sync>>,
}

impl<B: StorageBackend> ReadOnlyStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: ReadOnlyConfig::default(),
            key_whitelist: HashSet::new(),
            prefix_whitelist: Vec::new(),
            deny_callback: None,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: ReadOnlyConfig::default(),
            key_whitelist: HashSet::new(),
            prefix_whitelist: Vec::new(),
            deny_callback: None,
        }
    }

    /// 设置只读模式
    pub fn mode(mut self, mode: ReadOnlyMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// 允许删除操作
    pub fn allow_delete(mut self, allow: bool) -> Self {
        self.config.allow_delete = allow;
        self
    }

    /// 添加可写键
    pub fn allow_key(mut self, key: impl Into<String>) -> Self {
        self.config.mode = ReadOnlyMode::Whitelist;
        self.key_whitelist.insert(key.into());
        self
    }

    /// 添加多个可写键
    pub fn allow_keys(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.mode = ReadOnlyMode::Whitelist;
        for key in keys {
            self.key_whitelist.insert(key.into());
        }
        self
    }

    /// 添加可写前缀
    pub fn allow_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.mode = ReadOnlyMode::PrefixWhitelist;
        self.prefix_whitelist.push(prefix.into());
        self
    }

    /// 添加多个可写前缀
    pub fn allow_prefixes(mut self, prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config.mode = ReadOnlyMode::PrefixWhitelist;
        for prefix in prefixes {
            self.prefix_whitelist.push(prefix.into());
        }
        self
    }

    /// 设置拒绝回调
    pub fn deny_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.deny_callback = Some(Arc::new(callback));
        self
    }

    /// 构建
    pub fn build(self) -> ReadOnlyStorage<B> {
        ReadOnlyStorage {
            backend: self.backend,
            config: self.config,
            key_whitelist: Arc::new(RwLock::new(self.key_whitelist)),
            prefix_whitelist: Arc::new(RwLock::new(self.prefix_whitelist)),
            stats: Arc::new(ReadOnlyStats::default()),
            deny_callback: self.deny_callback,
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
    async fn test_read_only_storage_read() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"value1").await.unwrap();

        let storage = ReadOnlyStorage::new(backend);

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_read_only_storage_write_denied() {
        let storage = ReadOnlyStorage::new(MemoryStorage::new());

        let result = storage.write("key1", b"value1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_read_only_storage_delete_denied() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"value1").await.unwrap();

        let storage = ReadOnlyStorage::new(backend);

        let result = storage.delete("key1").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not allowed"));
    }

    #[tokio::test]
    async fn test_whitelist_mode() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_key("writable_key")
            .build();

        // 白名单键可写
        storage.write("writable_key", b"value1").await.unwrap();

        // 非白名单键不可写
        let result = storage.write("other_key", b"value2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prefix_whitelist_mode() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_prefix("temp:")
            .build();

        // 匹配前缀的键可写
        storage.write("temp:key1", b"value1").await.unwrap();
        storage.write("temp:key2", b"value2").await.unwrap();

        // 不匹配前缀的键不可写
        let result = storage.write("perm:key1", b"value3").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_allow_delete_option() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"value1").await.unwrap();

        let storage = ReadOnlyStorageBuilder::new(backend)
            .allow_delete(true)
            .build();

        // 写入仍然被拒绝
        let result = storage.write("key2", b"value2").await;
        assert!(result.is_err());

        // 但删除被允许
        storage.delete("key1").await.unwrap();
    }

    #[tokio::test]
    async fn test_dynamic_whitelist() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .mode(ReadOnlyMode::Whitelist)
            .build();

        // 初始不可写
        let result = storage.write("key1", b"value1").await;
        assert!(result.is_err());

        // 动态添加到白名单
        storage.allow_key("key1").await;

        // 现在可写
        storage.write("key1", b"value1").await.unwrap();

        // 从白名单移除
        storage.disallow_key("key1").await;

        // 又不可写了
        let result = storage.write("key1", b"value2").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_deny_callback() {
        use std::sync::Mutex;

        let denied = Arc::new(Mutex::new(Vec::new()));
        let denied_clone = Arc::clone(&denied);

        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .deny_callback(move |op, key| {
                denied_clone.lock().unwrap().push((op.to_string(), key.to_string()));
            })
            .build();

        let _ = storage.write("key1", b"v").await;
        let _ = storage.delete("key2").await;

        let denials = denied.lock().unwrap();
        assert_eq!(denials.len(), 2);
        assert_eq!(denials[0], ("write".to_string(), "key1".to_string()));
        assert_eq!(denials[1], ("delete".to_string(), "key2".to_string()));
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = ReadOnlyStorage::new(MemoryStorage::new());

        let _ = storage.read("key1").await;
        let _ = storage.write("key1", b"v").await;
        let _ = storage.write("key2", b"v").await;
        let _ = storage.delete("key1").await;

        let stats = storage.stats_snapshot();
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.write_attempts, 2);
        assert_eq!(stats.write_denied, 2);
        assert_eq!(stats.delete_attempts, 1);
        assert_eq!(stats.delete_denied, 1);
    }

    #[tokio::test]
    async fn test_deny_rates() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_key("writable")
            .build();

        storage.write("writable", b"v").await.unwrap();
        let _ = storage.write("readonly1", b"v").await;
        let _ = storage.write("readonly2", b"v").await;

        let stats = storage.stats_snapshot();
        // 3 attempts, 2 denied = 66.7%
        assert!((stats.write_deny_rate() - 0.667).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_key("key1")
            .allow_key("key2")
            .allow_prefix("temp:")
            .build();

        let detailed = storage.detailed_stats().await;
        assert_eq!(detailed.whitelist_count, 2);
        assert_eq!(detailed.prefix_whitelist_count, 1);
    }

    #[tokio::test]
    async fn test_list_and_exists_allowed() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"value1").await.unwrap();
        backend.write("key2", b"value2").await.unwrap();

        let storage = ReadOnlyStorage::new(backend);

        // list 允许
        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 2);

        // exists 允许
        assert!(storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_error_display() {
        let e1 = ReadOnlyError::WriteNotAllowed {
            key: "test".to_string(),
        };
        assert!(e1.to_string().contains("Write not allowed"));

        let e2 = ReadOnlyError::DeleteNotAllowed {
            key: "test".to_string(),
        };
        assert!(e2.to_string().contains("Delete not allowed"));
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        backend.write("key1", b"value1").await.unwrap();

        let storage = ReadOnlyStorage::from_arc(backend);

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_multiple_prefixes() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_prefixes(vec!["temp:", "cache:", "session:"])
            .build();

        storage.write("temp:key1", b"v").await.unwrap();
        storage.write("cache:key1", b"v").await.unwrap();
        storage.write("session:key1", b"v").await.unwrap();

        let result = storage.write("perm:key1", b"v").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_keys() {
        let storage = ReadOnlyStorageBuilder::new(MemoryStorage::new())
            .allow_keys(vec!["key1", "key2", "key3"])
            .build();

        storage.write("key1", b"v").await.unwrap();
        storage.write("key2", b"v").await.unwrap();
        storage.write("key3", b"v").await.unwrap();

        let result = storage.write("key4", b"v").await;
        assert!(result.is_err());
    }
}
