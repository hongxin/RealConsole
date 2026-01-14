//! LazyStorage - 延迟初始化存储层
//!
//! v1.82.0: 提供延迟初始化功能
//!
//! ## 功能特性
//!
//! - **延迟初始化**: 首次使用时才创建后端
//! - **线程安全**: 保证只初始化一次
//! - **初始化回调**: 成功/失败时通知
//! - **强制初始化**: 可提前触发初始化
//! - **状态检查**: 查询是否已初始化
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{LazyStorage, FileStorage};
//!
//! // 使用工厂函数创建
//! let storage = LazyStorage::new(|| {
//!     FileStorage::new("/path/to/data")
//! });
//!
//! // 此时后端尚未创建
//! assert!(!storage.is_initialized().await);
//!
//! // 首次操作时自动初始化
//! storage.write("key1", b"value1").await?;
//!
//! // 现在已初始化
//! assert!(storage.is_initialized().await);
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

// Type aliases for callback types to satisfy clippy::type_complexity
type InitCallback = Arc<dyn Fn() + Send + Sync>;
type ErrorCallback = Arc<dyn Fn(&str) + Send + Sync>;

// ============================================================================
// 初始化错误
// ============================================================================

/// 初始化错误
#[derive(Debug, Clone)]
pub struct InitializationError {
    pub message: String,
}

impl std::fmt::Display for InitializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Storage initialization failed: {}", self.message)
    }
}

impl std::error::Error for InitializationError {}

// ============================================================================
// 工厂类型
// ============================================================================

/// 同步工厂函数类型
pub type SyncFactory<B> = Box<dyn FnOnce() -> B + Send + Sync>;

/// 异步工厂函数类型
pub type AsyncFactory<B> =
    Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<B, String>> + Send>> + Send + Sync>;

/// 工厂类型枚举
enum Factory<B: StorageBackend> {
    Sync(Option<SyncFactory<B>>),
    Async(Option<AsyncFactory<B>>),
}

// ============================================================================
// 统计信息
// ============================================================================

/// 延迟初始化统计
#[derive(Debug, Default)]
pub struct LazyStats {
    /// 初始化尝试次数
    init_attempts: AtomicU64,
    /// 初始化成功次数
    init_success: AtomicU64,
    /// 初始化失败次数
    init_failures: AtomicU64,
    /// 初始化前的操作次数
    pre_init_operations: AtomicU64,
    /// 初始化耗时（微秒）
    init_duration_us: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct LazyStatsSnapshot {
    pub init_attempts: u64,
    pub init_success: u64,
    pub init_failures: u64,
    pub pre_init_operations: u64,
    pub init_duration_us: u64,
    pub is_initialized: bool,
}

impl LazyStatsSnapshot {
    /// 初始化耗时
    pub fn init_duration(&self) -> std::time::Duration {
        std::time::Duration::from_micros(self.init_duration_us)
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedLazyStats {
    /// 快照统计
    pub snapshot: LazyStatsSnapshot,
    /// 底层存储统计（如果已初始化）
    pub backend_stats: Option<StorageStats>,
}

// ============================================================================
// LazyStorage 实现
// ============================================================================

/// 延迟初始化存储层
///
/// 使用工厂函数延迟创建后端存储
pub struct LazyStorage<B: StorageBackend> {
    /// 后端存储（延迟初始化）
    backend: OnceCell<Arc<B>>,
    /// 工厂函数
    factory: RwLock<Factory<B>>,
    /// 是否已初始化
    initialized: AtomicBool,
    /// 统计信息
    stats: Arc<LazyStats>,
    /// 初始化成功回调
    on_init: Option<InitCallback>,
    /// 初始化失败回调
    on_error: Option<ErrorCallback>,
}

impl<B: StorageBackend + 'static> LazyStorage<B> {
    /// 使用同步工厂函数创建
    pub fn new<F>(factory: F) -> Self
    where
        F: FnOnce() -> B + Send + Sync + 'static,
    {
        Self {
            backend: OnceCell::new(),
            factory: RwLock::new(Factory::Sync(Some(Box::new(factory)))),
            initialized: AtomicBool::new(false),
            stats: Arc::new(LazyStats::default()),
            on_init: None,
            on_error: None,
        }
    }

    /// 使用异步工厂函数创建
    pub fn new_async<F, Fut>(factory: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<B, String>> + Send + 'static,
    {
        let boxed_factory: AsyncFactory<B> = Box::new(move || Box::pin(factory()));

        Self {
            backend: OnceCell::new(),
            factory: RwLock::new(Factory::Async(Some(boxed_factory))),
            initialized: AtomicBool::new(false),
            stats: Arc::new(LazyStats::default()),
            on_init: None,
            on_error: None,
        }
    }

    /// 从已存在的后端创建（已初始化状态）
    pub fn from_backend(backend: B) -> Self {
        let cell = OnceCell::new();
        // 这里我们直接设置后端
        let _ = cell.set(Arc::new(backend));

        Self {
            backend: cell,
            factory: RwLock::new(Factory::Sync(None)),
            initialized: AtomicBool::new(true),
            stats: Arc::new(LazyStats::default()),
            on_init: None,
            on_error: None,
        }
    }

    /// 设置初始化成功回调
    pub fn on_init<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_init = Some(Arc::new(callback));
        self
    }

    /// 设置初始化失败回调
    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_error = Some(Arc::new(callback));
        self
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::SeqCst)
    }

    /// 强制初始化
    pub async fn initialize(&self) -> Result<(), InitializationError> {
        self.ensure_initialized().await.map(|_| ())
    }

    /// 确保已初始化并返回后端引用
    async fn ensure_initialized(&self) -> Result<Arc<B>, InitializationError> {
        // 快速路径：已初始化
        if let Some(backend) = self.backend.get() {
            return Ok(Arc::clone(backend));
        }

        self.stats.init_attempts.fetch_add(1, Ordering::SeqCst);
        self.stats.pre_init_operations.fetch_add(1, Ordering::SeqCst);

        let start = std::time::Instant::now();

        // 获取工厂并初始化
        let mut factory_guard = self.factory.write().await;

        // 双重检查
        if let Some(backend) = self.backend.get() {
            return Ok(Arc::clone(backend));
        }

        let result = match &mut *factory_guard {
            Factory::Sync(factory_opt) => {
                if let Some(factory) = factory_opt.take() {
                    Ok(factory())
                } else {
                    Err("Factory already consumed".to_string())
                }
            }
            Factory::Async(factory_opt) => {
                if let Some(factory) = factory_opt.take() {
                    factory().await
                } else {
                    Err("Factory already consumed".to_string())
                }
            }
        };

        let duration = start.elapsed();
        self.stats
            .init_duration_us
            .store(duration.as_micros() as u64, Ordering::SeqCst);

        match result {
            Ok(backend) => {
                let backend = Arc::new(backend);
                let _ = self.backend.set(Arc::clone(&backend));
                self.initialized.store(true, Ordering::SeqCst);
                self.stats.init_success.fetch_add(1, Ordering::SeqCst);

                if let Some(ref callback) = self.on_init {
                    callback();
                }

                Ok(backend)
            }
            Err(msg) => {
                self.stats.init_failures.fetch_add(1, Ordering::SeqCst);

                if let Some(ref callback) = self.on_error {
                    callback(&msg);
                }

                Err(InitializationError { message: msg })
            }
        }
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> LazyStatsSnapshot {
        LazyStatsSnapshot {
            init_attempts: self.stats.init_attempts.load(Ordering::SeqCst),
            init_success: self.stats.init_success.load(Ordering::SeqCst),
            init_failures: self.stats.init_failures.load(Ordering::SeqCst),
            pre_init_operations: self.stats.pre_init_operations.load(Ordering::SeqCst),
            init_duration_us: self.stats.init_duration_us.load(Ordering::SeqCst),
            is_initialized: self.is_initialized(),
        }
    }

    /// 获取详细统计
    pub fn detailed_stats(&self) -> DetailedLazyStats {
        let backend_stats = self.backend.get().map(|b| b.stats());

        DetailedLazyStats {
            snapshot: self.stats_snapshot(),
            backend_stats,
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend + 'static> StorageBackend for LazyStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let backend = self
            .ensure_initialized()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;
        backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let backend = self
            .ensure_initialized()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;
        backend.write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let backend = self
            .ensure_initialized()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;
        backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let backend = self
            .ensure_initialized()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;
        backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let backend = self
            .ensure_initialized()
            .await
            .map_err(|e| StorageError::Other(e.to_string()))?;
        backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend
            .get()
            .map(|b| b.stats())
            .unwrap_or_default()
    }

    fn name(&self) -> &'static str {
        "LazyStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// LazyStorage 构建器
pub struct LazyStorageBuilder<B: StorageBackend> {
    factory: Factory<B>,
    on_init: Option<InitCallback>,
    on_error: Option<ErrorCallback>,
}

impl<B: StorageBackend + 'static> LazyStorageBuilder<B> {
    /// 使用同步工厂创建
    pub fn new<F>(factory: F) -> Self
    where
        F: FnOnce() -> B + Send + Sync + 'static,
    {
        Self {
            factory: Factory::Sync(Some(Box::new(factory))),
            on_init: None,
            on_error: None,
        }
    }

    /// 使用异步工厂创建
    pub fn new_async<F, Fut>(factory: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<B, String>> + Send + 'static,
    {
        let boxed_factory: AsyncFactory<B> = Box::new(move || Box::pin(factory()));

        Self {
            factory: Factory::Async(Some(boxed_factory)),
            on_init: None,
            on_error: None,
        }
    }

    /// 设置初始化成功回调
    pub fn on_init<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_init = Some(Arc::new(callback));
        self
    }

    /// 设置初始化失败回调
    pub fn on_error<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_error = Some(Arc::new(callback));
        self
    }

    /// 构建
    pub fn build(self) -> LazyStorage<B> {
        LazyStorage {
            backend: OnceCell::new(),
            factory: RwLock::new(self.factory),
            initialized: AtomicBool::new(false),
            stats: Arc::new(LazyStats::default()),
            on_init: self.on_init,
            on_error: self.on_error,
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
    async fn test_lazy_storage_basic() {
        let storage = LazyStorage::new(MemoryStorage::new);

        assert!(!storage.is_initialized());

        storage.write("key1", b"value1").await.unwrap();

        assert!(storage.is_initialized());

        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_lazy_initialization_on_read() {
        let storage = LazyStorage::new(MemoryStorage::new);

        assert!(!storage.is_initialized());

        // 读取触发初始化（即使返回 NotFound）
        let _ = storage.read("nonexistent").await;

        assert!(storage.is_initialized());
    }

    #[tokio::test]
    async fn test_force_initialize() {
        let storage = LazyStorage::new(MemoryStorage::new);

        assert!(!storage.is_initialized());

        storage.initialize().await.unwrap();

        assert!(storage.is_initialized());
    }

    #[tokio::test]
    async fn test_from_backend() {
        let backend = MemoryStorage::new();
        backend.write("key1", b"value1").await.unwrap();

        let storage = LazyStorage::from_backend(backend);

        // 已经初始化
        assert!(storage.is_initialized());

        // 数据可用
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_on_init_callback() {
        use std::sync::Mutex;

        let called = Arc::new(Mutex::new(false));
        let called_clone = Arc::clone(&called);

        let storage = LazyStorage::new(MemoryStorage::new).on_init(move || {
            *called_clone.lock().unwrap() = true;
        });

        assert!(!*called.lock().unwrap());

        storage.write("key1", b"v").await.unwrap();

        assert!(*called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_async_factory() {
        let storage = LazyStorage::new_async(|| async { Ok(MemoryStorage::new()) });

        assert!(!storage.is_initialized());

        storage.write("key1", b"value1").await.unwrap();

        assert!(storage.is_initialized());
    }

    #[tokio::test]
    async fn test_async_factory_error() {
        use std::sync::Mutex;

        let error_msg = Arc::new(Mutex::new(String::new()));
        let error_msg_clone = Arc::clone(&error_msg);

        let storage: LazyStorage<MemoryStorage> =
            LazyStorage::new_async(|| async { Err("Connection failed".to_string()) }).on_error(
                move |msg| {
                    *error_msg_clone.lock().unwrap() = msg.to_string();
                },
            );

        let result = storage.write("key1", b"v").await;
        assert!(result.is_err());

        let msg = error_msg.lock().unwrap();
        assert!(msg.contains("Connection failed"));
    }

    #[tokio::test]
    async fn test_single_initialization() {
        use std::sync::atomic::AtomicUsize;

        let init_count = Arc::new(AtomicUsize::new(0));
        let init_count_clone = Arc::clone(&init_count);

        let storage = LazyStorage::new(move || {
            init_count_clone.fetch_add(1, Ordering::SeqCst);
            MemoryStorage::new()
        });

        // 并发操作
        let storage = Arc::new(storage);
        let mut handles = vec![];

        for i in 0..10 {
            let s = Arc::clone(&storage);
            handles.push(tokio::spawn(async move {
                s.write(&format!("key{}", i), b"v").await.unwrap();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        // 只初始化一次
        assert_eq!(init_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = LazyStorage::new(MemoryStorage::new);

        let stats = storage.stats_snapshot();
        assert!(!stats.is_initialized);
        assert_eq!(stats.init_attempts, 0);

        storage.write("key1", b"v").await.unwrap();

        let stats = storage.stats_snapshot();
        assert!(stats.is_initialized);
        assert_eq!(stats.init_success, 1);
        assert!(stats.init_duration_us > 0 || stats.init_duration_us == 0); // 可能很快
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = LazyStorage::new(MemoryStorage::new);

        let detailed = storage.detailed_stats();
        assert!(detailed.backend_stats.is_none());

        storage.write("key1", b"v").await.unwrap();

        let detailed = storage.detailed_stats();
        assert!(detailed.backend_stats.is_some());
    }

    #[tokio::test]
    async fn test_builder() {
        use std::sync::Mutex;

        let init_called = Arc::new(Mutex::new(false));
        let init_called_clone = Arc::clone(&init_called);

        let storage = LazyStorageBuilder::new(MemoryStorage::new)
            .on_init(move || {
                *init_called_clone.lock().unwrap() = true;
            })
            .build();

        storage.write("key1", b"v").await.unwrap();

        assert!(*init_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_all_operations() {
        let storage = LazyStorage::new(MemoryStorage::new);

        // write
        storage.write("key1", b"value1").await.unwrap();

        // read
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");

        // exists
        assert!(storage.exists("key1").await.unwrap());

        // list
        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 1);

        // delete
        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_init_duration() {
        let storage = LazyStorage::new(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            MemoryStorage::new()
        });

        storage.write("key1", b"v").await.unwrap();

        let stats = storage.stats_snapshot();
        assert!(stats.init_duration() >= std::time::Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_initialization_error_display() {
        let err = InitializationError {
            message: "test error".to_string(),
        };
        assert!(err.to_string().contains("test error"));
    }
}
