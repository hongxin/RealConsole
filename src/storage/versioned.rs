//! 版本化存储层
//!
//! v1.65.0: v2.0 探路期 - 数据版本控制
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的版本化存储架构：
//! - **当前层**: 最新版本数据
//! - **历史层**: 版本历史记录
//! - **策略层**: 版本保留策略
//!
//! ## 版本管理策略
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                  VersionedStorage                     │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Write:                                               │
//! │    Data ─────► Create Version ─────► Backend         │
//! │                    │                                  │
//! │                    └──► Apply Retention Policy        │
//! │                                                       │
//! │  Read:                                                │
//! │    Backend ─────► Latest Version ─────► Data         │
//! │                                                       │
//! │  History:                                             │
//! │    key:v1 ─► key:v2 ─► key:v3 ─► key:latest         │
//! │                                                       │
//! │  Retention Policies:                                  │
//! │    - KeepAll: 保留所有版本                            │
//! │    - KeepLast(n): 保留最近 n 个版本                   │
//! │    - KeepDays(d): 保留 d 天内的版本                   │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{VersionedStorage, FileStorage, RetentionPolicy};
//!
//! let backend = FileStorage::new("/path/to/data");
//! let storage = VersionedStorage::new(backend);
//!
//! // 写入（自动创建新版本）
//! storage.write("key1", b"version 1").await?;
//! storage.write("key1", b"version 2").await?;
//!
//! // 读取最新版本
//! let data = storage.read("key1").await?;
//!
//! // 读取特定版本
//! let v1 = storage.read_version("key1", 1).await?;
//!
//! // 列出版本
//! let versions = storage.list_versions("key1").await?;
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::collections::HashMap;

/// 版本保留策略
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)] // Keep prefix is intentional for clarity
pub enum RetentionPolicy {
    /// 保留所有版本
    KeepAll,
    /// 保留最近 n 个版本
    KeepLast(usize),
    /// 保留 n 天内的版本
    KeepDays(u32),
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        RetentionPolicy::KeepLast(10)
    }
}

/// 版本化存储配置
#[derive(Debug, Clone)]
pub struct VersionedStorageConfig {
    /// 版本保留策略
    pub retention: RetentionPolicy,
    /// 是否在写入时自动清理旧版本
    pub auto_cleanup: bool,
    /// 版本元数据前缀
    pub meta_prefix: String,
}

impl Default for VersionedStorageConfig {
    fn default() -> Self {
        Self {
            retention: RetentionPolicy::KeepLast(10),
            auto_cleanup: true,
            meta_prefix: "_versions/".to_string(),
        }
    }
}

/// 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// 版本号
    pub version: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 数据大小（字节）
    pub size: usize,
    /// 可选的版本描述
    pub description: Option<String>,
}

/// 版本元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct VersionMetadata {
    /// 当前版本号
    current_version: u64,
    /// 版本列表
    versions: Vec<VersionInfo>,
}

/// 版本统计
#[derive(Debug, Default)]
struct VersioningStats {
    /// 版本创建次数
    versions_created: AtomicU64,
    /// 版本读取次数
    versions_read: AtomicU64,
    /// 版本删除次数
    versions_deleted: AtomicU64,
    /// 清理操作次数
    cleanups: AtomicU64,
}

/// 详细版本统计
#[derive(Debug, Clone)]
pub struct DetailedVersioningStats {
    /// 版本创建次数
    pub versions_created: u64,
    /// 版本读取次数
    pub versions_read: u64,
    /// 版本删除次数
    pub versions_deleted: u64,
    /// 清理操作次数
    pub cleanups: u64,
    /// 总版本数（估计）
    pub total_versions: u64,
    /// 唯一键数
    pub unique_keys: usize,
}

impl DetailedVersioningStats {
    /// 平均每个键的版本数
    pub fn avg_versions_per_key(&self) -> f64 {
        if self.unique_keys == 0 {
            0.0
        } else {
            self.total_versions as f64 / self.unique_keys as f64
        }
    }
}

/// 版本化存储
///
/// 自动为每次写入创建新版本，支持版本历史查询
pub struct VersionedStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 配置
    config: VersionedStorageConfig,
    /// 统计
    stats: VersioningStats,
    /// 版本计数缓存（key -> current version）
    version_cache: RwLock<HashMap<String, u64>>,
}

impl<B: StorageBackend> VersionedStorage<B> {
    /// 创建版本化存储（默认配置）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, VersionedStorageConfig::default())
    }

    /// 使用保留策略创建
    pub fn with_retention(backend: B, retention: RetentionPolicy) -> Self {
        Self::with_config(
            backend,
            VersionedStorageConfig {
                retention,
                ..Default::default()
            },
        )
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: VersionedStorageConfig) -> Self {
        Self {
            backend,
            config,
            stats: VersioningStats::default(),
            version_cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取版本数据的存储键
    fn version_key(&self, key: &str, version: u64) -> String {
        format!("{}{}:v{}", self.config.meta_prefix, key, version)
    }

    /// 获取元数据的存储键
    fn meta_key(&self, key: &str) -> String {
        format!("{}{}.meta", self.config.meta_prefix, key)
    }

    /// 加载版本元数据
    async fn load_metadata(&self, key: &str) -> StorageResult<VersionMetadata> {
        let meta_key = self.meta_key(key);
        match self.backend.read(&meta_key).await {
            Ok(data) => {
                serde_json::from_slice(&data)
                    .map_err(|e| StorageError::Serialization(e.to_string()))
            }
            Err(StorageError::NotFound(_)) => Ok(VersionMetadata::default()),
            Err(e) => Err(e),
        }
    }

    /// 保存版本元数据
    async fn save_metadata(&self, key: &str, metadata: &VersionMetadata) -> StorageResult<()> {
        let meta_key = self.meta_key(key);
        let data = serde_json::to_vec(metadata)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        self.backend.write(&meta_key, &data).await
    }

    /// 读取特定版本
    pub async fn read_version(&self, key: &str, version: u64) -> StorageResult<Vec<u8>> {
        let version_key = self.version_key(key, version);
        self.stats.versions_read.fetch_add(1, Ordering::Relaxed);
        self.backend.read(&version_key).await
    }

    /// 列出所有版本
    pub async fn list_versions(&self, key: &str) -> StorageResult<Vec<VersionInfo>> {
        let metadata = self.load_metadata(key).await?;
        Ok(metadata.versions)
    }

    /// 获取当前版本号
    pub async fn current_version(&self, key: &str) -> StorageResult<Option<u64>> {
        // 先检查缓存
        {
            let cache = self.version_cache.read().unwrap();
            if let Some(&version) = cache.get(key) {
                return Ok(Some(version));
            }
        }

        // 从存储加载
        let metadata = self.load_metadata(key).await?;
        if metadata.current_version == 0 {
            Ok(None)
        } else {
            // 更新缓存
            {
                let mut cache = self.version_cache.write().unwrap();
                cache.insert(key.to_string(), metadata.current_version);
            }
            Ok(Some(metadata.current_version))
        }
    }

    /// 获取版本信息
    pub async fn get_version_info(&self, key: &str, version: u64) -> StorageResult<Option<VersionInfo>> {
        let metadata = self.load_metadata(key).await?;
        Ok(metadata.versions.iter().find(|v| v.version == version).cloned())
    }

    /// 应用保留策略
    async fn apply_retention(&self, key: &str, metadata: &mut VersionMetadata) -> StorageResult<usize> {
        let versions_to_delete = match &self.config.retention {
            RetentionPolicy::KeepAll => {
                return Ok(0);
            }
            RetentionPolicy::KeepLast(n) => {
                if metadata.versions.len() <= *n {
                    return Ok(0);
                }
                let count_to_remove = metadata.versions.len() - *n;
                metadata.versions.drain(0..count_to_remove).collect::<Vec<_>>()
            }
            RetentionPolicy::KeepDays(days) => {
                let cutoff = Utc::now() - chrono::Duration::days(*days as i64);
                let old_versions: Vec<_> = metadata
                    .versions
                    .iter()
                    .filter(|v| v.created_at < cutoff)
                    .cloned()
                    .collect();
                metadata.versions.retain(|v| v.created_at >= cutoff);
                old_versions
            }
        };

        // 删除旧版本数据
        let mut deleted = 0;
        for version_info in &versions_to_delete {
            let version_key = self.version_key(key, version_info.version);
            if self.backend.delete(&version_key).await.is_ok() {
                deleted += 1;
                self.stats.versions_deleted.fetch_add(1, Ordering::Relaxed);
            }
        }

        if deleted > 0 {
            self.stats.cleanups.fetch_add(1, Ordering::Relaxed);
        }

        Ok(deleted)
    }

    /// 手动清理旧版本
    pub async fn cleanup(&self, key: &str) -> StorageResult<usize> {
        let mut metadata = self.load_metadata(key).await?;
        let deleted = self.apply_retention(key, &mut metadata).await?;
        if deleted > 0 {
            self.save_metadata(key, &metadata).await?;
        }
        Ok(deleted)
    }

    /// 清理所有键的旧版本
    pub async fn cleanup_all(&self) -> StorageResult<usize> {
        let keys = self.list_versioned_keys().await?;
        let mut total_deleted = 0;
        for key in keys {
            total_deleted += self.cleanup(&key).await?;
        }
        Ok(total_deleted)
    }

    /// 列出所有有版本的键
    pub async fn list_versioned_keys(&self) -> StorageResult<Vec<String>> {
        let prefix = &self.config.meta_prefix;
        let all_keys = self.backend.list(prefix).await?;

        // 过滤出 .meta 文件并提取键名
        let keys: Vec<String> = all_keys
            .iter()
            .filter(|k| k.ends_with(".meta"))
            .map(|k| {
                k.trim_start_matches(prefix)
                    .trim_end_matches(".meta")
                    .to_string()
            })
            .collect();

        Ok(keys)
    }

    /// 获取详细版本统计
    pub fn versioning_stats(&self) -> DetailedVersioningStats {
        let cache = self.version_cache.read().unwrap();
        let total_versions: u64 = cache.values().sum();

        DetailedVersioningStats {
            versions_created: self.stats.versions_created.load(Ordering::Relaxed),
            versions_read: self.stats.versions_read.load(Ordering::Relaxed),
            versions_deleted: self.stats.versions_deleted.load(Ordering::Relaxed),
            cleanups: self.stats.cleanups.load(Ordering::Relaxed),
            total_versions,
            unique_keys: cache.len(),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &VersionedStorageConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 回滚到特定版本
    pub async fn rollback(&self, key: &str, version: u64) -> StorageResult<()> {
        // 读取指定版本
        let data = self.read_version(key, version).await?;

        // 写入为新版本（这会创建一个新版本）
        self.write(key, &data).await
    }

    /// 比较两个版本
    pub async fn diff_versions(&self, key: &str, v1: u64, v2: u64) -> StorageResult<(Vec<u8>, Vec<u8>)> {
        let data1 = self.read_version(key, v1).await?;
        let data2 = self.read_version(key, v2).await?;
        Ok((data1, data2))
    }

    /// 删除特定版本
    pub async fn delete_version(&self, key: &str, version: u64) -> StorageResult<()> {
        let mut metadata = self.load_metadata(key).await?;

        // 不能删除当前版本
        if metadata.current_version == version {
            return Err(StorageError::Other("Cannot delete current version".into()));
        }

        // 删除版本数据
        let version_key = self.version_key(key, version);
        self.backend.delete(&version_key).await?;

        // 更新元数据
        metadata.versions.retain(|v| v.version != version);
        self.save_metadata(key, &metadata).await?;

        self.stats.versions_deleted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for VersionedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let metadata = self.load_metadata(key).await?;
        if metadata.current_version == 0 {
            return Err(StorageError::NotFound(key.to_string()));
        }

        self.read_version(key, metadata.current_version).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        // 加载或创建元数据
        let mut metadata = self.load_metadata(key).await?;

        // 创建新版本
        let new_version = metadata.current_version + 1;
        let version_info = VersionInfo {
            version: new_version,
            created_at: Utc::now(),
            size: data.len(),
            description: None,
        };

        // 保存版本数据
        let version_key = self.version_key(key, new_version);
        self.backend.write(&version_key, data).await?;

        // 更新元数据
        metadata.current_version = new_version;
        metadata.versions.push(version_info);

        // 应用保留策略
        if self.config.auto_cleanup {
            self.apply_retention(key, &mut metadata).await?;
        }

        // 保存元数据
        self.save_metadata(key, &metadata).await?;

        // 更新缓存
        {
            let mut cache = self.version_cache.write().unwrap();
            cache.insert(key.to_string(), new_version);
        }

        self.stats.versions_created.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let metadata = self.load_metadata(key).await?;

        // 删除所有版本数据
        for version_info in &metadata.versions {
            let version_key = self.version_key(key, version_info.version);
            let _ = self.backend.delete(&version_key).await;
        }

        // 删除元数据
        let meta_key = self.meta_key(key);
        self.backend.delete(&meta_key).await?;

        // 清除缓存
        {
            let mut cache = self.version_cache.write().unwrap();
            cache.remove(key);
        }

        self.stats.versions_deleted.fetch_add(metadata.versions.len() as u64, Ordering::Relaxed);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 列出所有有版本的键，过滤前缀
        let keys = self.list_versioned_keys().await?;
        Ok(keys.into_iter().filter(|k| k.starts_with(prefix)).collect())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        match self.current_version(key).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn stats(&self) -> StorageStats {
        let versioning_stats = self.versioning_stats();
        let backend_stats = self.backend.stats();

        StorageStats {
            reads: versioning_stats.versions_read,
            writes: versioning_stats.versions_created,
            deletes: versioning_stats.versions_deleted,
            hits: 0,
            misses: 0,
            total_bytes: backend_stats.total_bytes,
            key_count: versioning_stats.unique_keys,
        }
    }

    fn name(&self) -> &'static str {
        "VersionedStorage"
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
    async fn test_versioned_storage_new() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        assert_eq!(storage.name(), "VersionedStorage");
        assert_eq!(storage.config().retention, RetentionPolicy::KeepLast(10));
    }

    #[tokio::test]
    async fn test_versioned_storage_write_read() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_versioned_storage_multiple_versions() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"version 1").await.unwrap();
        storage.write("key1", b"version 2").await.unwrap();
        storage.write("key1", b"version 3").await.unwrap();

        // 读取最新版本
        let latest = storage.read("key1").await.unwrap();
        assert_eq!(latest, b"version 3");

        // 读取特定版本
        let v1 = storage.read_version("key1", 1).await.unwrap();
        assert_eq!(v1, b"version 1");

        let v2 = storage.read_version("key1", 2).await.unwrap();
        assert_eq!(v2, b"version 2");
    }

    #[tokio::test]
    async fn test_versioned_storage_list_versions() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key1", b"v2").await.unwrap();

        let versions = storage.list_versions("key1").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 1);
        assert_eq!(versions[1].version, 2);
    }

    #[tokio::test]
    async fn test_versioned_storage_current_version() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        // 不存在时返回 None
        assert_eq!(storage.current_version("key1").await.unwrap(), None);

        storage.write("key1", b"data").await.unwrap();
        assert_eq!(storage.current_version("key1").await.unwrap(), Some(1));

        storage.write("key1", b"data2").await.unwrap();
        assert_eq!(storage.current_version("key1").await.unwrap(), Some(2));
    }

    #[tokio::test]
    async fn test_versioned_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();
        storage.write("key1", b"data2").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());

        // 版本也应该被删除
        assert!(storage.read_version("key1", 1).await.is_err());
    }

    #[tokio::test]
    async fn test_versioned_storage_retention_keep_last() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::with_retention(backend, RetentionPolicy::KeepLast(2));

        // 写入 5 个版本
        for i in 1..=5 {
            storage.write("key1", format!("version {}", i).as_bytes()).await.unwrap();
        }

        // 应该只保留最后 2 个版本
        let versions = storage.list_versions("key1").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].version, 4);
        assert_eq!(versions[1].version, 5);
    }

    #[tokio::test]
    async fn test_versioned_storage_retention_keep_all() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::with_retention(backend, RetentionPolicy::KeepAll);

        for i in 1..=10 {
            storage.write("key1", format!("v{}", i).as_bytes()).await.unwrap();
        }

        let versions = storage.list_versions("key1").await.unwrap();
        assert_eq!(versions.len(), 10);
    }

    #[tokio::test]
    async fn test_versioned_storage_rollback() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::with_retention(backend, RetentionPolicy::KeepAll);

        storage.write("key1", b"original").await.unwrap();
        storage.write("key1", b"modified").await.unwrap();

        // 验证当前是修改后的版本
        assert_eq!(storage.read("key1").await.unwrap(), b"modified");

        // 回滚到版本 1
        storage.rollback("key1", 1).await.unwrap();

        // 现在应该是原始数据（作为新版本 3）
        assert_eq!(storage.read("key1").await.unwrap(), b"original");
        assert_eq!(storage.current_version("key1").await.unwrap(), Some(3));
    }

    #[tokio::test]
    async fn test_versioned_storage_get_version_info() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();

        let info = storage.get_version_info("key1", 1).await.unwrap().unwrap();
        assert_eq!(info.version, 1);
        assert_eq!(info.size, 4);
    }

    #[tokio::test]
    async fn test_versioned_storage_diff_versions() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"version A").await.unwrap();
        storage.write("key1", b"version B").await.unwrap();

        let (v1, v2) = storage.diff_versions("key1", 1, 2).await.unwrap();
        assert_eq!(v1, b"version A");
        assert_eq!(v2, b"version B");
    }

    #[tokio::test]
    async fn test_versioned_storage_delete_version() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::with_retention(backend, RetentionPolicy::KeepAll);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key1", b"v2").await.unwrap();
        storage.write("key1", b"v3").await.unwrap();

        // 删除版本 2
        storage.delete_version("key1", 2).await.unwrap();

        let versions = storage.list_versions("key1").await.unwrap();
        assert_eq!(versions.len(), 2);
        assert!(versions.iter().all(|v| v.version != 2));
    }

    #[tokio::test]
    async fn test_versioned_storage_cannot_delete_current() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data").await.unwrap();

        // 不能删除当前版本
        let result = storage.delete_version("key1", 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_versioned_storage_list_versioned_keys() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();
        storage.write("key3", b"data3").await.unwrap();

        let keys = storage.list_versioned_keys().await.unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));
    }

    #[tokio::test]
    async fn test_versioned_storage_list_with_prefix() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("user/1", b"data1").await.unwrap();
        storage.write("user/2", b"data2").await.unwrap();
        storage.write("config/1", b"data3").await.unwrap();

        let user_keys = storage.list("user/").await.unwrap();
        assert_eq!(user_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_versioned_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key1", b"data2").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.versioning_stats();
        assert_eq!(stats.versions_created, 2);
        assert_eq!(stats.versions_read, 1);
    }

    #[tokio::test]
    async fn test_versioned_storage_cleanup() {
        let backend = MemoryStorage::new();
        let config = VersionedStorageConfig {
            retention: RetentionPolicy::KeepLast(2),
            auto_cleanup: false, // 手动清理
            ..Default::default()
        };
        let storage = VersionedStorage::with_config(backend, config);

        // 写入 5 个版本
        for i in 1..=5 {
            storage.write("key1", format!("v{}", i).as_bytes()).await.unwrap();
        }

        // 因为 auto_cleanup = false，应该有 5 个版本
        assert_eq!(storage.list_versions("key1").await.unwrap().len(), 5);

        // 手动清理
        let deleted = storage.cleanup("key1").await.unwrap();
        assert_eq!(deleted, 3); // 删除了 3 个旧版本

        // 现在应该只有 2 个版本
        assert_eq!(storage.list_versions("key1").await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_versioned_storage_read_not_found() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_versioned_storage_version_info_size() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::new(backend);

        let data = vec![0u8; 1000];
        storage.write("key1", &data).await.unwrap();

        let info = storage.get_version_info("key1", 1).await.unwrap().unwrap();
        assert_eq!(info.size, 1000);
    }

    #[tokio::test]
    async fn test_detailed_stats_avg_versions() {
        let backend = MemoryStorage::new();
        let storage = VersionedStorage::with_retention(backend, RetentionPolicy::KeepAll);

        // 写入 3 个键，各 2 个版本
        for key in &["k1", "k2", "k3"] {
            storage.write(key, b"v1").await.unwrap();
            storage.write(key, b"v2").await.unwrap();
        }

        let stats = storage.versioning_stats();
        assert_eq!(stats.unique_keys, 3);
        // total_versions 是从缓存计算的当前版本号之和
        assert_eq!(stats.total_versions, 6); // 2 + 2 + 2
        assert!((stats.avg_versions_per_key() - 2.0).abs() < 0.001);
    }
}
