//! 存储构建器
//!
//! v1.66.0: v2.0 探路期 - 存储层组合
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的存储构建架构：
//! - **基础层**: FileStorage / MemoryStorage
//! - **优化层**: 缓存、压缩、批量写入
//! - **功能层**: 版本控制、类型安全
//!
//! ## 构建器模式
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                   StorageBuilder                      │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Fluent API:                                          │
//! │                                                       │
//! │    StorageBuilder::file("/path")                     │
//! │        .with_compression(Default)                    │
//! │        .with_cache(config)                           │
//! │        .with_versioning(KeepLast(10))               │
//! │        .build()                                      │
//! │                                                       │
//! │  Layer Composition:                                   │
//! │                                                       │
//! │    ┌─────────────┐                                   │
//! │    │  Versioned  │  ← 最外层：版本控制               │
//! │    ├─────────────┤                                   │
//! │    │   Cached    │  ← 缓存层                         │
//! │    ├─────────────┤                                   │
//! │    │ Compressed  │  ← 压缩层                         │
//! │    ├─────────────┤                                   │
//! │    │    File     │  ← 基础层                         │
//! │    └─────────────┘                                   │
//! │                                                       │
//! │  Presets:                                             │
//! │    - development: 内存存储，无优化                    │
//! │    - production: 文件存储 + 缓存 + 压缩              │
//! │    - archival: 文件 + 压缩(Best) + 版本控制          │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{StorageBuilder, CompressionLevel, RetentionPolicy};
//!
//! // 简单构建
//! let storage = StorageBuilder::file("/data").build();
//!
//! // 完整配置
//! let storage = StorageBuilder::file("/data")
//!     .with_compression(CompressionLevel::Default)
//!     .with_cache_default()
//!     .with_versioning(RetentionPolicy::KeepLast(10))
//!     .build();
//!
//! // 使用预设
//! let storage = StorageBuilder::production("/data");
//! ```

use super::{
    CachedStorage, CachedStorageConfig, CompressedStorage, CompressedStorageConfig,
    CompressionLevel, FileStorage, MemoryStorage, RetentionPolicy, StorageBackend,
    TieredCacheConfig, VersionedStorage, VersionedStorageConfig,
};
use std::path::Path;
use std::sync::Arc;

/// 存储层配置
#[derive(Debug, Clone)]
pub struct StorageLayerConfig {
    /// 压缩配置
    pub compression: Option<CompressedStorageConfig>,
    /// 缓存配置
    pub cache: Option<CachedStorageConfig>,
    /// 版本配置
    pub versioning: Option<VersionedStorageConfig>,
}

impl Default for StorageLayerConfig {
    fn default() -> Self {
        Self {
            compression: None,
            cache: None,
            versioning: None,
        }
    }
}

/// 存储构建器
///
/// 提供流畅的 API 来组合存储层
pub struct StorageBuilder {
    /// 基础存储类型
    base: StorageBase,
    /// 层配置
    config: StorageLayerConfig,
}

/// 基础存储类型
enum StorageBase {
    /// 文件存储
    File(String),
    /// 内存存储
    Memory,
}

impl StorageBuilder {
    /// 创建文件存储构建器
    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self {
            base: StorageBase::File(path.as_ref().to_string_lossy().to_string()),
            config: StorageLayerConfig::default(),
        }
    }

    /// 创建内存存储构建器
    pub fn memory() -> Self {
        Self {
            base: StorageBase::Memory,
            config: StorageLayerConfig::default(),
        }
    }

    // ========================================================================
    // 预设配置
    // ========================================================================

    /// 开发模式预设（内存存储，无优化）
    pub fn development() -> BuiltStorage {
        Self::memory().build()
    }

    /// 生产模式预设（文件存储 + 缓存 + 压缩）
    pub fn production<P: AsRef<Path>>(path: P) -> BuiltStorage {
        Self::file(path)
            .with_compression_default()
            .with_cache_default()
            .build()
    }

    /// 归档模式预设（文件 + 最佳压缩 + 版本控制）
    pub fn archival<P: AsRef<Path>>(path: P) -> BuiltStorage {
        Self::file(path)
            .with_compression(CompressionLevel::Best)
            .with_versioning(RetentionPolicy::KeepAll)
            .build()
    }

    /// 快速模式预设（文件 + 快速压缩 + 缓存）
    pub fn fast<P: AsRef<Path>>(path: P) -> BuiltStorage {
        Self::file(path)
            .with_compression(CompressionLevel::Fast)
            .with_cache_default()
            .build()
    }

    /// 版本控制模式预设（文件 + 缓存 + 版本控制）
    pub fn versioned<P: AsRef<Path>>(path: P) -> BuiltStorage {
        Self::file(path)
            .with_cache_default()
            .with_versioning(RetentionPolicy::KeepLast(10))
            .build()
    }

    // ========================================================================
    // 压缩层配置
    // ========================================================================

    /// 添加压缩层（指定级别）
    pub fn with_compression(mut self, level: CompressionLevel) -> Self {
        self.config.compression = Some(CompressedStorageConfig {
            level,
            ..Default::default()
        });
        self
    }

    /// 添加压缩层（默认配置）
    pub fn with_compression_default(mut self) -> Self {
        self.config.compression = Some(CompressedStorageConfig::default());
        self
    }

    /// 添加压缩层（自定义配置）
    pub fn with_compression_config(mut self, config: CompressedStorageConfig) -> Self {
        self.config.compression = Some(config);
        self
    }

    /// 不使用压缩
    pub fn without_compression(mut self) -> Self {
        self.config.compression = None;
        self
    }

    // ========================================================================
    // 缓存层配置
    // ========================================================================

    /// 添加缓存层（默认配置）
    pub fn with_cache_default(mut self) -> Self {
        self.config.cache = Some(CachedStorageConfig::default());
        self
    }

    /// 添加缓存层（自定义配置）
    pub fn with_cache(mut self, config: CachedStorageConfig) -> Self {
        self.config.cache = Some(config);
        self
    }

    /// 添加缓存层（指定缓存配置）
    pub fn with_tiered_cache(mut self, cache_config: TieredCacheConfig) -> Self {
        self.config.cache = Some(CachedStorageConfig {
            cache_config,
            ..Default::default()
        });
        self
    }

    /// 不使用缓存
    pub fn without_cache(mut self) -> Self {
        self.config.cache = None;
        self
    }

    // ========================================================================
    // 版本层配置
    // ========================================================================

    /// 添加版本控制层（指定保留策略）
    pub fn with_versioning(mut self, retention: RetentionPolicy) -> Self {
        self.config.versioning = Some(VersionedStorageConfig {
            retention,
            ..Default::default()
        });
        self
    }

    /// 添加版本控制层（默认配置）
    pub fn with_versioning_default(mut self) -> Self {
        self.config.versioning = Some(VersionedStorageConfig::default());
        self
    }

    /// 添加版本控制层（自定义配置）
    pub fn with_versioning_config(mut self, config: VersionedStorageConfig) -> Self {
        self.config.versioning = Some(config);
        self
    }

    /// 不使用版本控制
    pub fn without_versioning(mut self) -> Self {
        self.config.versioning = None;
        self
    }

    // ========================================================================
    // 构建
    // ========================================================================

    /// 构建存储
    ///
    /// 层顺序（从内到外）：
    /// 1. Base (File/Memory)
    /// 2. Compression
    /// 3. Cache
    /// 4. Versioning
    pub fn build(self) -> BuiltStorage {
        // 保存配置快照
        let config_snapshot = self.config.clone();

        // 创建基础存储
        let base: Box<dyn StorageBackend> = match self.base {
            StorageBase::File(path) => Box::new(FileStorage::new(&path)),
            StorageBase::Memory => Box::new(MemoryStorage::new()),
        };

        // 应用压缩层
        let with_compression: Box<dyn StorageBackend> = match self.config.compression {
            Some(config) => Box::new(CompressedStorage::with_config(
                BoxedStorage(base),
                config,
            )),
            None => base,
        };

        // 应用缓存层
        let with_cache: Box<dyn StorageBackend> = match self.config.cache {
            Some(config) => Box::new(CachedStorage::with_config(
                BoxedStorage(with_compression),
                config,
            )),
            None => with_compression,
        };

        // 应用版本层
        let with_versioning: Box<dyn StorageBackend> = match self.config.versioning {
            Some(config) => Box::new(VersionedStorage::with_config(
                BoxedStorage(with_cache),
                config,
            )),
            None => with_cache,
        };

        BuiltStorage {
            inner: Arc::new(BoxedStorage(with_versioning)),
            config: config_snapshot,
        }
    }

    /// 获取当前配置
    pub fn config(&self) -> &StorageLayerConfig {
        &self.config
    }
}

/// 包装 Box<dyn StorageBackend> 实现 StorageBackend
struct BoxedStorage(Box<dyn StorageBackend>);

#[async_trait::async_trait]
impl StorageBackend for BoxedStorage {
    async fn read(&self, key: &str) -> super::StorageResult<Vec<u8>> {
        self.0.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> super::StorageResult<()> {
        self.0.write(key, data).await
    }

    async fn delete(&self, key: &str) -> super::StorageResult<()> {
        self.0.delete(key).await
    }

    async fn list(&self, prefix: &str) -> super::StorageResult<Vec<String>> {
        self.0.list(prefix).await
    }

    async fn exists(&self, key: &str) -> super::StorageResult<bool> {
        self.0.exists(key).await
    }

    fn stats(&self) -> super::StorageStats {
        self.0.stats()
    }

    fn name(&self) -> &'static str {
        "BoxedStorage"
    }
}

/// 构建完成的存储
pub struct BuiltStorage {
    /// 内部存储
    inner: Arc<BoxedStorage>,
    /// 配置快照
    config: StorageLayerConfig,
}

impl BuiltStorage {
    /// 获取配置
    pub fn config(&self) -> &StorageLayerConfig {
        &self.config
    }

    /// 是否启用压缩
    pub fn has_compression(&self) -> bool {
        self.config.compression.is_some()
    }

    /// 是否启用缓存
    pub fn has_cache(&self) -> bool {
        self.config.cache.is_some()
    }

    /// 是否启用版本控制
    pub fn has_versioning(&self) -> bool {
        self.config.versioning.is_some()
    }

    /// 获取层描述
    pub fn describe_layers(&self) -> Vec<&'static str> {
        let mut layers = vec!["Base"];
        if self.config.compression.is_some() {
            layers.push("Compression");
        }
        if self.config.cache.is_some() {
            layers.push("Cache");
        }
        if self.config.versioning.is_some() {
            layers.push("Versioning");
        }
        layers
    }
}

#[async_trait::async_trait]
impl StorageBackend for BuiltStorage {
    async fn read(&self, key: &str) -> super::StorageResult<Vec<u8>> {
        self.inner.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> super::StorageResult<()> {
        self.inner.write(key, data).await
    }

    async fn delete(&self, key: &str) -> super::StorageResult<()> {
        self.inner.delete(key).await
    }

    async fn list(&self, prefix: &str) -> super::StorageResult<Vec<String>> {
        self.inner.list(prefix).await
    }

    async fn exists(&self, key: &str) -> super::StorageResult<bool> {
        self.inner.exists(key).await
    }

    fn stats(&self) -> super::StorageStats {
        self.inner.stats()
    }

    fn name(&self) -> &'static str {
        "BuiltStorage"
    }
}

impl Clone for BuiltStorage {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            config: self.config.clone(),
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_builder_memory() {
        let storage = StorageBuilder::memory().build();

        storage.write("key1", b"data1").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_builder_with_compression() {
        let storage = StorageBuilder::memory()
            .with_compression(CompressionLevel::Default)
            .build();

        assert!(storage.has_compression());

        let data = vec![0u8; 1000];
        storage.write("key1", &data).await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_builder_with_cache() {
        let storage = StorageBuilder::memory()
            .with_cache_default()
            .build();

        assert!(storage.has_cache());

        storage.write("key1", b"data1").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_builder_with_versioning() {
        let storage = StorageBuilder::memory()
            .with_versioning(RetentionPolicy::KeepLast(5))
            .build();

        assert!(storage.has_versioning());

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key1", b"v2").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"v2");
    }

    #[tokio::test]
    async fn test_builder_full_stack() {
        let storage = StorageBuilder::memory()
            .with_compression(CompressionLevel::Fast)
            .with_cache_default()
            .with_versioning(RetentionPolicy::KeepLast(3))
            .build();

        assert!(storage.has_compression());
        assert!(storage.has_cache());
        assert!(storage.has_versioning());

        let layers = storage.describe_layers();
        assert_eq!(layers, vec!["Base", "Compression", "Cache", "Versioning"]);

        // 测试写入和读取
        let data = vec![42u8; 500];
        storage.write("key1", &data).await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, data);
    }

    #[tokio::test]
    async fn test_builder_development_preset() {
        let storage = StorageBuilder::development();

        storage.write("key1", b"dev").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"dev");

        // 开发模式没有额外层
        assert!(!storage.has_compression());
        assert!(!storage.has_cache());
        assert!(!storage.has_versioning());
    }

    #[tokio::test]
    async fn test_builder_without_layers() {
        let storage = StorageBuilder::memory()
            .with_compression_default()
            .without_compression()
            .with_cache_default()
            .without_cache()
            .build();

        assert!(!storage.has_compression());
        assert!(!storage.has_cache());
    }

    #[tokio::test]
    async fn test_builder_clone() {
        let storage1 = StorageBuilder::memory().build();
        storage1.write("key1", b"data1").await.unwrap();

        let storage2 = storage1.clone();
        let loaded = storage2.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_builder_describe_layers_base_only() {
        let storage = StorageBuilder::memory().build();
        let layers = storage.describe_layers();
        assert_eq!(layers, vec!["Base"]);
    }

    #[tokio::test]
    async fn test_builder_describe_layers_partial() {
        let storage = StorageBuilder::memory()
            .with_compression_default()
            .with_versioning_default()
            .build();

        let layers = storage.describe_layers();
        assert_eq!(layers, vec!["Base", "Compression", "Versioning"]);
    }

    #[tokio::test]
    async fn test_builder_config_access() {
        let builder = StorageBuilder::memory()
            .with_compression(CompressionLevel::Best);

        let config = builder.config();
        assert!(config.compression.is_some());
        assert!(config.cache.is_none());
    }

    #[tokio::test]
    async fn test_builder_compression_levels() {
        // Fast
        let storage = StorageBuilder::memory()
            .with_compression(CompressionLevel::Fast)
            .build();
        assert!(storage.has_compression());

        // Best
        let storage = StorageBuilder::memory()
            .with_compression(CompressionLevel::Best)
            .build();
        assert!(storage.has_compression());

        // None
        let storage = StorageBuilder::memory()
            .with_compression(CompressionLevel::None)
            .build();
        assert!(storage.has_compression());
    }

    #[tokio::test]
    async fn test_builder_retention_policies() {
        // KeepAll
        let storage = StorageBuilder::memory()
            .with_versioning(RetentionPolicy::KeepAll)
            .build();
        assert!(storage.has_versioning());

        // KeepLast
        let storage = StorageBuilder::memory()
            .with_versioning(RetentionPolicy::KeepLast(5))
            .build();
        assert!(storage.has_versioning());

        // KeepDays
        let storage = StorageBuilder::memory()
            .with_versioning(RetentionPolicy::KeepDays(30))
            .build();
        assert!(storage.has_versioning());
    }

    #[tokio::test]
    async fn test_builder_multiple_writes() {
        let storage = StorageBuilder::memory()
            .with_compression_default()
            .with_cache_default()
            .build();

        for i in 0..10 {
            let key = format!("key{}", i);
            let data = format!("data{}", i).into_bytes();
            storage.write(&key, &data).await.unwrap();
        }

        for i in 0..10 {
            let key = format!("key{}", i);
            let expected = format!("data{}", i).into_bytes();
            let loaded = storage.read(&key).await.unwrap();
            assert_eq!(loaded, expected);
        }
    }

    #[tokio::test]
    async fn test_builder_delete() {
        let storage = StorageBuilder::memory()
            .with_cache_default()
            .build();

        storage.write("key1", b"data1").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_builder_list() {
        let storage = StorageBuilder::memory().build();

        storage.write("user/1", b"data1").await.unwrap();
        storage.write("user/2", b"data2").await.unwrap();
        storage.write("config/1", b"data3").await.unwrap();

        let all_keys = storage.list("").await.unwrap();
        assert_eq!(all_keys.len(), 3);

        let user_keys = storage.list("user/").await.unwrap();
        assert_eq!(user_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_builder_stats() {
        let storage = StorageBuilder::memory().build();

        storage.write("key1", b"data1").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.stats();
        assert!(stats.writes > 0 || stats.reads > 0 || stats.key_count > 0);
    }

    #[tokio::test]
    async fn test_builder_name() {
        let storage = StorageBuilder::memory().build();
        assert_eq!(storage.name(), "BuiltStorage");
    }

    #[tokio::test]
    async fn test_builder_with_custom_cache_config() {
        let cache_config = TieredCacheConfig {
            hot_capacity: 50,
            warm_capacity: 100,
            cold_capacity: 200,
            ..Default::default()
        };

        let storage = StorageBuilder::memory()
            .with_tiered_cache(cache_config)
            .build();

        assert!(storage.has_cache());
    }

    #[tokio::test]
    async fn test_builder_with_custom_compression_config() {
        let compression_config = CompressedStorageConfig {
            level: CompressionLevel::Custom(5),
            min_size_threshold: 128,
            skip_if_larger: true,
        };

        let storage = StorageBuilder::memory()
            .with_compression_config(compression_config)
            .build();

        assert!(storage.has_compression());
    }

    #[tokio::test]
    async fn test_builder_with_custom_versioning_config() {
        let versioning_config = VersionedStorageConfig {
            retention: RetentionPolicy::KeepLast(5),
            auto_cleanup: false,
            meta_prefix: "_my_versions/".to_string(),
        };

        let storage = StorageBuilder::memory()
            .with_versioning_config(versioning_config)
            .build();

        assert!(storage.has_versioning());
    }
}
