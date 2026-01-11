//! 命名空间存储层
//!
//! v1.69.0: 存储层扩展 - 命名空间隔离
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的命名空间架构：
//! - **应用层**: 使用简单键名
//! - **隔离层**: 自动添加命名空间前缀
//! - **存储层**: 实际的键值存储
//!
//! ## 命名空间管理
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                  NamespacedStorage                    │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  键映射:                                              │
//! │    "key" → "namespace/key"                           │
//! │                                                       │
//! │  命名空间结构:                                        │
//! │    conversations/                                     │
//! │      ├── conv_001                                    │
//! │      ├── conv_002                                    │
//! │      └── conv_003                                    │
//! │    context/                                           │
//! │      ├── project                                     │
//! │      └── session                                     │
//! │    preferences/                                       │
//! │      └── settings                                    │
//! │                                                       │
//! │  操作:                                                │
//! │    - read/write/delete: 自动添加前缀                 │
//! │    - list: 只返回本命名空间内的键                    │
//! │    - clear: 清空命名空间                             │
//! │    - exists_in_namespace: 检查命名空间内键           │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{NamespacedStorage, MemoryStorage};
//!
//! let backend = MemoryStorage::new();
//!
//! // 创建不同命名空间的存储
//! let conversations = NamespacedStorage::new(backend.clone(), "conversations");
//! let context = NamespacedStorage::new(backend.clone(), "context");
//!
//! // 写入数据（自动添加前缀）
//! conversations.write("conv_001", b"Hello").await?;  // 实际键: conversations/conv_001
//! context.write("session", b"data").await?;          // 实际键: context/session
//!
//! // 列出命名空间内的键
//! let keys = conversations.list("").await?;  // ["conv_001"]
//!
//! // 清空命名空间
//! conversations.clear().await?;
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 命名空间配置
#[derive(Debug, Clone)]
pub struct NamespacedStorageConfig {
    /// 命名空间名称
    pub namespace: String,
    /// 分隔符（默认 "/"）
    pub separator: String,
    /// 是否在列表时去除前缀
    pub strip_prefix_on_list: bool,
}

impl NamespacedStorageConfig {
    /// 创建配置
    pub fn new(namespace: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            separator: "/".to_string(),
            strip_prefix_on_list: true,
        }
    }

    /// 设置分隔符
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// 设置是否在列表时去除前缀
    pub fn with_strip_prefix(mut self, strip: bool) -> Self {
        self.strip_prefix_on_list = strip;
        self
    }
}

/// 命名空间统计
#[derive(Debug, Default)]
struct NamespaceStats {
    /// 读取次数
    reads: AtomicU64,
    /// 写入次数
    writes: AtomicU64,
    /// 删除次数
    deletes: AtomicU64,
}

/// 详细命名空间统计
#[derive(Debug, Clone)]
pub struct DetailedNamespaceStats {
    /// 命名空间名称
    pub namespace: String,
    /// 读取次数
    pub reads: u64,
    /// 写入次数
    pub writes: u64,
    /// 删除次数
    pub deletes: u64,
    /// 键数量（估计）
    pub key_count: usize,
}

/// 命名空间存储
///
/// 为底层存储添加命名空间隔离，自动管理键前缀
pub struct NamespacedStorage<B: StorageBackend> {
    /// 后端存储
    backend: Arc<B>,
    /// 配置
    config: NamespacedStorageConfig,
    /// 统计
    stats: NamespaceStats,
}

impl<B: StorageBackend> NamespacedStorage<B> {
    /// 创建命名空间存储
    pub fn new(backend: B, namespace: impl Into<String>) -> Self {
        Self::with_config(backend, NamespacedStorageConfig::new(namespace))
    }

    /// 使用 Arc 包装的后端创建（支持共享）
    pub fn with_shared(backend: Arc<B>, namespace: impl Into<String>) -> Self {
        Self {
            backend,
            config: NamespacedStorageConfig::new(namespace),
            stats: NamespaceStats::default(),
        }
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: NamespacedStorageConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
            stats: NamespaceStats::default(),
        }
    }

    /// 获取完整的命名空间前缀
    pub fn prefix(&self) -> String {
        format!("{}{}", self.config.namespace, self.config.separator)
    }

    /// 将简单键转换为带前缀的键
    fn prefixed_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix(), key)
    }

    /// 从带前缀的键提取简单键
    fn strip_prefix(&self, key: &str) -> Option<String> {
        let prefix = self.prefix();
        if key.starts_with(&prefix) {
            Some(key[prefix.len()..].to_string())
        } else {
            None
        }
    }

    /// 获取命名空间名称
    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }

    /// 获取配置
    pub fn config(&self) -> &NamespacedStorageConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 清空命名空间内的所有数据
    pub async fn clear(&self) -> StorageResult<usize> {
        let keys = self.list("").await?;
        let mut deleted = 0;

        for key in keys {
            if self.delete(&key).await.is_ok() {
                deleted += 1;
            }
        }

        Ok(deleted)
    }

    /// 获取命名空间内的键数量
    pub async fn count(&self) -> StorageResult<usize> {
        let keys = self.list("").await?;
        Ok(keys.len())
    }

    /// 检查键是否存在于此命名空间
    pub async fn exists_in_namespace(&self, key: &str) -> StorageResult<bool> {
        let prefixed = self.prefixed_key(key);
        self.backend.exists(&prefixed).await
    }

    /// 复制键到另一个命名空间
    pub async fn copy_to<B2: StorageBackend>(
        &self,
        key: &str,
        target: &NamespacedStorage<B2>,
        target_key: &str,
    ) -> StorageResult<()> {
        let data = self.read(key).await?;
        target.write(target_key, &data).await
    }

    /// 移动键到另一个命名空间
    pub async fn move_to<B2: StorageBackend>(
        &self,
        key: &str,
        target: &NamespacedStorage<B2>,
        target_key: &str,
    ) -> StorageResult<()> {
        self.copy_to(key, target, target_key).await?;
        self.delete(key).await
    }

    /// 批量读取
    pub async fn read_many(&self, keys: &[&str]) -> StorageResult<Vec<(String, Vec<u8>)>> {
        let mut results = Vec::new();
        for key in keys {
            match self.read(key).await {
                Ok(data) => results.push((key.to_string(), data)),
                Err(StorageError::NotFound(_)) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    /// 批量写入
    pub async fn write_many(&self, items: &[(&str, &[u8])]) -> StorageResult<usize> {
        let mut written = 0;
        for (key, data) in items {
            self.write(key, data).await?;
            written += 1;
        }
        Ok(written)
    }

    /// 批量删除
    pub async fn delete_many(&self, keys: &[&str]) -> StorageResult<usize> {
        let mut deleted = 0;
        for key in keys {
            if self.delete(key).await.is_ok() {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> StorageResult<DetailedNamespaceStats> {
        let key_count = self.count().await?;
        Ok(DetailedNamespaceStats {
            namespace: self.config.namespace.clone(),
            reads: self.stats.reads.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            deletes: self.stats.deletes.load(Ordering::Relaxed),
            key_count,
        })
    }

    /// 创建子命名空间
    pub fn sub_namespace(&self, sub: impl Into<String>) -> NamespacedStorage<B> {
        let new_namespace = format!(
            "{}{}{}",
            self.config.namespace,
            self.config.separator,
            sub.into()
        );
        NamespacedStorage {
            backend: Arc::clone(&self.backend),
            config: NamespacedStorageConfig {
                namespace: new_namespace,
                separator: self.config.separator.clone(),
                strip_prefix_on_list: self.config.strip_prefix_on_list,
            },
            stats: NamespaceStats::default(),
        }
    }
}

impl<B: StorageBackend> Clone for NamespacedStorage<B> {
    fn clone(&self) -> Self {
        Self {
            backend: Arc::clone(&self.backend),
            config: self.config.clone(),
            stats: NamespaceStats::default(),
        }
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for NamespacedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let prefixed = self.prefixed_key(key);
        self.stats.reads.fetch_add(1, Ordering::Relaxed);
        self.backend.read(&prefixed).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let prefixed = self.prefixed_key(key);
        self.stats.writes.fetch_add(1, Ordering::Relaxed);
        self.backend.write(&prefixed, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let prefixed = self.prefixed_key(key);
        self.stats.deletes.fetch_add(1, Ordering::Relaxed);
        self.backend.delete(&prefixed).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 组合命名空间前缀和用户前缀
        let full_prefix = format!("{}{}", self.prefix(), prefix);
        let keys = self.backend.list(&full_prefix).await?;

        if self.config.strip_prefix_on_list {
            // 去除命名空间前缀
            Ok(keys
                .into_iter()
                .filter_map(|k| self.strip_prefix(&k))
                .collect())
        } else {
            Ok(keys)
        }
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let prefixed = self.prefixed_key(key);
        self.backend.exists(&prefixed).await
    }

    fn stats(&self) -> StorageStats {
        StorageStats {
            reads: self.stats.reads.load(Ordering::Relaxed),
            writes: self.stats.writes.load(Ordering::Relaxed),
            deletes: self.stats.deletes.load(Ordering::Relaxed),
            hits: 0,
            misses: 0,
            total_bytes: 0,
            key_count: 0,
        }
    }

    fn name(&self) -> &'static str {
        "NamespacedStorage"
    }
}

/// 命名空间管理器
///
/// 管理多个命名空间，提供统一的访问接口
pub struct NamespaceManager<B: StorageBackend> {
    /// 后端存储
    backend: Arc<B>,
    /// 默认分隔符
    separator: String,
}

impl<B: StorageBackend + Send + Sync> NamespaceManager<B> {
    /// 创建命名空间管理器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            separator: "/".to_string(),
        }
    }

    /// 设置分隔符
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// 获取命名空间存储
    pub fn namespace(&self, name: impl Into<String>) -> NamespacedStorage<B> {
        NamespacedStorage {
            backend: Arc::clone(&self.backend),
            config: NamespacedStorageConfig {
                namespace: name.into(),
                separator: self.separator.clone(),
                strip_prefix_on_list: true,
            },
            stats: NamespaceStats::default(),
        }
    }

    /// 列出所有命名空间
    pub async fn list_namespaces(&self) -> StorageResult<Vec<String>> {
        let all_keys = self.backend.list("").await?;
        let mut namespaces = std::collections::HashSet::new();

        for key in all_keys {
            if let Some(pos) = key.find(&self.separator) {
                namespaces.insert(key[..pos].to_string());
            }
        }

        let mut result: Vec<_> = namespaces.into_iter().collect();
        result.sort();
        Ok(result)
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 删除整个命名空间
    pub async fn delete_namespace(&self, name: &str) -> StorageResult<usize> {
        let ns = self.namespace(name);
        ns.clear().await
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
    async fn test_namespaced_storage_new() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        assert_eq!(storage.namespace(), "test");
        assert_eq!(storage.prefix(), "test/");
        assert_eq!(storage.name(), "NamespacedStorage");
    }

    #[tokio::test]
    async fn test_namespaced_storage_write_read() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_namespaced_storage_isolation() {
        let backend = MemoryStorage::new();
        let ns1 = NamespacedStorage::with_shared(Arc::new(backend), "ns1");
        let ns2 = NamespacedStorage::with_shared(Arc::clone(&ns1.backend), "ns2");

        // 写入不同命名空间
        ns1.write("key", b"data1").await.unwrap();
        ns2.write("key", b"data2").await.unwrap();

        // 读取各自的数据
        assert_eq!(ns1.read("key").await.unwrap(), b"data1");
        assert_eq!(ns2.read("key").await.unwrap(), b"data2");
    }

    #[tokio::test]
    async fn test_namespaced_storage_list() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();
        storage.write("other/key3", b"data3").await.unwrap();

        // 列出所有键
        let all_keys = storage.list("").await.unwrap();
        assert_eq!(all_keys.len(), 3);

        // 列出带前缀的键
        let other_keys = storage.list("other/").await.unwrap();
        assert_eq!(other_keys.len(), 1);
        assert_eq!(other_keys[0], "other/key3");
    }

    #[tokio::test]
    async fn test_namespaced_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_namespaced_storage_clear() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();
        storage.write("key3", b"data3").await.unwrap();

        let deleted = storage.clear().await.unwrap();
        assert_eq!(deleted, 3);
        assert_eq!(storage.count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_namespaced_storage_count() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        assert_eq!(storage.count().await.unwrap(), 0);

        storage.write("key1", b"data1").await.unwrap();
        storage.write("key2", b"data2").await.unwrap();

        assert_eq!(storage.count().await.unwrap(), 2);
    }

    #[tokio::test]
    async fn test_namespaced_storage_exists_in_namespace() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();

        assert!(storage.exists_in_namespace("key1").await.unwrap());
        assert!(!storage.exists_in_namespace("key2").await.unwrap());
    }

    #[tokio::test]
    async fn test_namespaced_storage_copy_to() {
        let backend = Arc::new(MemoryStorage::new());
        let ns1 = NamespacedStorage::with_shared(Arc::clone(&backend), "ns1");
        let ns2 = NamespacedStorage::with_shared(Arc::clone(&backend), "ns2");

        ns1.write("key1", b"data1").await.unwrap();
        ns1.copy_to("key1", &ns2, "key1_copy").await.unwrap();

        assert_eq!(ns1.read("key1").await.unwrap(), b"data1");
        assert_eq!(ns2.read("key1_copy").await.unwrap(), b"data1");
    }

    #[tokio::test]
    async fn test_namespaced_storage_move_to() {
        let backend = Arc::new(MemoryStorage::new());
        let ns1 = NamespacedStorage::with_shared(Arc::clone(&backend), "ns1");
        let ns2 = NamespacedStorage::with_shared(Arc::clone(&backend), "ns2");

        ns1.write("key1", b"data1").await.unwrap();
        ns1.move_to("key1", &ns2, "key1_moved").await.unwrap();

        assert!(!ns1.exists("key1").await.unwrap());
        assert_eq!(ns2.read("key1_moved").await.unwrap(), b"data1");
    }

    #[tokio::test]
    async fn test_namespaced_storage_batch_operations() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        // 批量写入
        let items: Vec<(&str, &[u8])> = vec![
            ("key1", b"data1"),
            ("key2", b"data2"),
            ("key3", b"data3"),
        ];
        let written = storage.write_many(&items).await.unwrap();
        assert_eq!(written, 3);

        // 批量读取
        let results = storage.read_many(&["key1", "key2", "key4"]).await.unwrap();
        assert_eq!(results.len(), 2); // key4 不存在

        // 批量删除
        let deleted = storage.delete_many(&["key1", "key2"]).await.unwrap();
        assert_eq!(deleted, 2);
    }

    #[tokio::test]
    async fn test_namespaced_storage_sub_namespace() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "root");

        let sub = storage.sub_namespace("child");
        assert_eq!(sub.namespace(), "root/child");
        assert_eq!(sub.prefix(), "root/child/");

        sub.write("key", b"data").await.unwrap();
        assert!(sub.exists("key").await.unwrap());
    }

    #[tokio::test]
    async fn test_namespaced_storage_custom_separator() {
        let backend = MemoryStorage::new();
        let config = NamespacedStorageConfig::new("test").with_separator("::");
        let storage = NamespacedStorage::with_config(backend, config);

        assert_eq!(storage.prefix(), "test::");

        storage.write("key1", b"data1").await.unwrap();
        let keys = storage.list("").await.unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "key1");
    }

    #[tokio::test]
    async fn test_namespaced_storage_detailed_stats() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();

        let stats = storage.detailed_stats().await.unwrap();
        assert_eq!(stats.namespace, "test");
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.deletes, 1);
    }

    #[tokio::test]
    async fn test_namespace_manager_new() {
        let backend = MemoryStorage::new();
        let manager = NamespaceManager::new(backend);

        let ns = manager.namespace("test");
        assert_eq!(ns.namespace(), "test");
    }

    #[tokio::test]
    async fn test_namespace_manager_list_namespaces() {
        let backend = MemoryStorage::new();
        let manager = NamespaceManager::new(backend);

        manager.namespace("ns1").write("key1", b"data").await.unwrap();
        manager.namespace("ns2").write("key2", b"data").await.unwrap();
        manager.namespace("ns3").write("key3", b"data").await.unwrap();

        let namespaces = manager.list_namespaces().await.unwrap();
        assert_eq!(namespaces.len(), 3);
        assert!(namespaces.contains(&"ns1".to_string()));
        assert!(namespaces.contains(&"ns2".to_string()));
        assert!(namespaces.contains(&"ns3".to_string()));
    }

    #[tokio::test]
    async fn test_namespace_manager_delete_namespace() {
        let backend = MemoryStorage::new();
        let manager = NamespaceManager::new(backend);

        let ns = manager.namespace("test");
        ns.write("key1", b"data1").await.unwrap();
        ns.write("key2", b"data2").await.unwrap();

        let deleted = manager.delete_namespace("test").await.unwrap();
        assert_eq!(deleted, 2);

        assert_eq!(manager.namespace("test").count().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_namespaced_storage_clone() {
        let backend = MemoryStorage::new();
        let storage1 = NamespacedStorage::new(backend, "test");

        storage1.write("key1", b"data1").await.unwrap();

        let storage2 = storage1.clone();
        let loaded = storage2.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");
    }

    #[tokio::test]
    async fn test_namespaced_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        storage.write("key1", b"data1").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.stats();
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.writes, 1);
    }

    #[tokio::test]
    async fn test_namespaced_storage_read_not_found() {
        let backend = MemoryStorage::new();
        let storage = NamespacedStorage::new(backend, "test");

        let result = storage.read("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }
}
