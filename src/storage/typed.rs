//! 类型安全的序列化存储层
//!
//! v1.63.0: v2.0 探路期 - 类型安全存储
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的类型安全存储架构：
//! - **类型层**: 编译时类型检查
//! - **序列化层**: 自动序列化/反序列化
//! - **存储层**: 底层 StorageBackend
//!
//! ## 序列化格式
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                   TypedStorage                        │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  Rust Type ─────► Serializer ─────► Bytes            │
//! │                                                       │
//! │  Supported Formats:                                   │
//! │    - JSON   (可读, 调试友好)                          │
//! │    - Bincode (紧凑, 性能优先)                         │
//! │                                                       │
//! │  Bytes ─────────► StorageBackend                     │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{TypedStorage, MemoryStorage, SerializationFormat};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct User {
//!     name: String,
//!     age: u32,
//! }
//!
//! let backend = MemoryStorage::new();
//! let storage = TypedStorage::new(backend);
//!
//! // 类型安全的存储
//! let user = User { name: "Alice".to_string(), age: 30 };
//! storage.set("user:1", &user).await?;
//!
//! // 类型安全的读取
//! let loaded: User = storage.get("user:1").await?;
//! assert_eq!(loaded, user);
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// 序列化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SerializationFormat {
    /// JSON 格式（可读，调试友好）
    #[default]
    Json,
    /// Bincode 格式（紧凑，性能优先）
    Bincode,
}

/// 类型存储配置
#[derive(Debug, Clone)]
pub struct TypedStorageConfig {
    /// 序列化格式
    pub format: SerializationFormat,
    /// 是否美化 JSON 输出
    pub pretty_json: bool,
}

impl Default for TypedStorageConfig {
    fn default() -> Self {
        Self {
            format: SerializationFormat::Json,
            pretty_json: false,
        }
    }
}

/// 类型存储统计
#[derive(Debug, Default)]
struct TypedStorageStats {
    serializations: AtomicU64,
    deserializations: AtomicU64,
    serialization_bytes: AtomicU64,
}

/// 详细类型存储统计
#[derive(Debug, Clone)]
pub struct DetailedTypedStats {
    /// 序列化次数
    pub serializations: u64,
    /// 反序列化次数
    pub deserializations: u64,
    /// 序列化总字节数
    pub serialization_bytes: u64,
    /// 平均序列化大小
    pub avg_serialization_size: f64,
}

/// 类型安全的序列化存储
///
/// 提供类型安全的 API，自动处理序列化/反序列化
pub struct TypedStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 配置
    config: TypedStorageConfig,
    /// 统计
    stats: TypedStorageStats,
}

impl<B: StorageBackend> TypedStorage<B> {
    /// 创建类型存储（默认 JSON 格式）
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, TypedStorageConfig::default())
    }

    /// 使用 Bincode 格式创建
    pub fn with_bincode(backend: B) -> Self {
        Self::with_config(
            backend,
            TypedStorageConfig {
                format: SerializationFormat::Bincode,
                ..Default::default()
            },
        )
    }

    /// 使用自定义配置创建
    pub fn with_config(backend: B, config: TypedStorageConfig) -> Self {
        Self {
            backend,
            config,
            stats: TypedStorageStats::default(),
        }
    }

    /// 序列化数据
    fn serialize<T: Serialize>(&self, value: &T) -> StorageResult<Vec<u8>> {
        let bytes = match self.config.format {
            SerializationFormat::Json => {
                if self.config.pretty_json {
                    serde_json::to_vec_pretty(value)
                } else {
                    serde_json::to_vec(value)
                }
                .map_err(|e| StorageError::Serialization(e.to_string()))?
            }
            SerializationFormat::Bincode => {
                bincode::serialize(value).map_err(|e| StorageError::Serialization(e.to_string()))?
            }
        };

        self.stats.serializations.fetch_add(1, Ordering::Relaxed);
        self.stats
            .serialization_bytes
            .fetch_add(bytes.len() as u64, Ordering::Relaxed);

        Ok(bytes)
    }

    /// 反序列化数据
    fn deserialize<T: DeserializeOwned>(&self, bytes: &[u8]) -> StorageResult<T> {
        self.stats.deserializations.fetch_add(1, Ordering::Relaxed);

        match self.config.format {
            SerializationFormat::Json => {
                serde_json::from_slice(bytes).map_err(|e| StorageError::Serialization(e.to_string()))
            }
            SerializationFormat::Bincode => bincode::deserialize(bytes)
                .map_err(|e| StorageError::Serialization(e.to_string())),
        }
    }

    /// 存储类型化数据
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> StorageResult<()> {
        let bytes = self.serialize(value)?;
        self.backend.write(key, &bytes).await
    }

    /// 读取类型化数据
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> StorageResult<T> {
        let bytes = self.backend.read(key).await?;
        self.deserialize(&bytes)
    }

    /// 读取类型化数据（返回 Option）
    pub async fn get_opt<T: DeserializeOwned>(&self, key: &str) -> StorageResult<Option<T>> {
        match self.backend.read(key).await {
            Ok(bytes) => Ok(Some(self.deserialize(&bytes)?)),
            Err(StorageError::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// 删除数据
    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        self.backend.delete(key).await
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    /// 列出键
    pub async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    /// 批量存储
    pub async fn set_many<T: Serialize>(
        &self,
        items: &[(&str, &T)],
    ) -> StorageResult<usize> {
        let mut count = 0;
        for (key, value) in items {
            self.set(key, value).await?;
            count += 1;
        }
        Ok(count)
    }

    /// 批量读取
    pub async fn get_many<T: DeserializeOwned>(
        &self,
        keys: &[&str],
    ) -> StorageResult<Vec<(String, T)>> {
        let mut results = Vec::new();
        for key in keys {
            if let Some(value) = self.get_opt::<T>(key).await? {
                results.push((key.to_string(), value));
            }
        }
        Ok(results)
    }

    /// 获取详细统计
    pub fn typed_stats(&self) -> DetailedTypedStats {
        let serializations = self.stats.serializations.load(Ordering::Relaxed);
        let bytes = self.stats.serialization_bytes.load(Ordering::Relaxed);

        DetailedTypedStats {
            serializations,
            deserializations: self.stats.deserializations.load(Ordering::Relaxed),
            serialization_bytes: bytes,
            avg_serialization_size: if serializations == 0 {
                0.0
            } else {
                bytes as f64 / serializations as f64
            },
        }
    }

    /// 获取配置
    pub fn config(&self) -> &TypedStorageConfig {
        &self.config
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// 获取序列化格式
    pub fn format(&self) -> SerializationFormat {
        self.config.format
    }
}

/// TypedStorage 也实现 StorageBackend（作为字节存储）
#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for TypedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.backend.write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.backend.delete(key).await
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
        "TypedStorage"
    }
}

// ============================================================================
// 便捷类型别名
// ============================================================================

/// JSON 类型存储
pub type JsonStorage<B> = TypedStorage<B>;

/// 创建 JSON 存储的便捷函数
pub fn json_storage<B: StorageBackend>(backend: B) -> TypedStorage<B> {
    TypedStorage::new(backend)
}

/// 创建 Bincode 存储的便捷函数
pub fn bincode_storage<B: StorageBackend>(backend: B) -> TypedStorage<B> {
    TypedStorage::with_bincode(backend)
}

// ============================================================================
// 集合存储扩展
// ============================================================================

/// 类型化集合存储（用于存储同类型的多个值）
pub struct TypedCollection<B: StorageBackend, T> {
    storage: TypedStorage<B>,
    prefix: String,
    _marker: PhantomData<T>,
}

impl<B: StorageBackend, T: Serialize + DeserializeOwned> TypedCollection<B, T> {
    /// 创建集合存储
    pub fn new(backend: B, prefix: &str) -> Self {
        Self {
            storage: TypedStorage::new(backend),
            prefix: prefix.to_string(),
            _marker: PhantomData,
        }
    }

    /// 生成完整键
    fn full_key(&self, id: &str) -> String {
        format!("{}:{}", self.prefix, id)
    }

    /// 存储项目
    pub async fn insert(&self, id: &str, value: &T) -> StorageResult<()> {
        self.storage.set(&self.full_key(id), value).await
    }

    /// 读取项目
    pub async fn get(&self, id: &str) -> StorageResult<T> {
        self.storage.get(&self.full_key(id)).await
    }

    /// 读取项目（可选）
    pub async fn get_opt(&self, id: &str) -> StorageResult<Option<T>> {
        self.storage.get_opt(&self.full_key(id)).await
    }

    /// 删除项目
    pub async fn remove(&self, id: &str) -> StorageResult<()> {
        self.storage.delete(&self.full_key(id)).await
    }

    /// 检查项目是否存在
    pub async fn contains(&self, id: &str) -> StorageResult<bool> {
        self.storage.exists(&self.full_key(id)).await
    }

    /// 列出所有 ID
    pub async fn list_ids(&self) -> StorageResult<Vec<String>> {
        let keys = self.storage.list(&self.prefix).await?;
        let prefix_len = self.prefix.len() + 1; // +1 for ':'
        Ok(keys
            .into_iter()
            .filter_map(|k| {
                if k.len() > prefix_len {
                    Some(k[prefix_len..].to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    /// 获取所有项目
    pub async fn get_all(&self) -> StorageResult<Vec<(String, T)>> {
        let ids = self.list_ids().await?;
        let mut results = Vec::new();
        for id in ids {
            if let Some(value) = self.get_opt(&id).await? {
                results.push((id, value));
            }
        }
        Ok(results)
    }

    /// 获取项目数量
    pub async fn count(&self) -> StorageResult<usize> {
        Ok(self.list_ids().await?.len())
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestUser {
        name: String,
        age: u32,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestConfig {
        debug: bool,
        max_items: usize,
        tags: Vec<String>,
    }

    #[tokio::test]
    async fn test_typed_storage_new() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        assert_eq!(storage.format(), SerializationFormat::Json);
        assert_eq!(storage.name(), "TypedStorage");
    }

    #[tokio::test]
    async fn test_typed_storage_set_get() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let user = TestUser {
            name: "Alice".to_string(),
            age: 30,
        };

        storage.set("user:1", &user).await.unwrap();
        let loaded: TestUser = storage.get("user:1").await.unwrap();

        assert_eq!(loaded, user);
    }

    #[tokio::test]
    async fn test_typed_storage_get_opt() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let user = TestUser {
            name: "Bob".to_string(),
            age: 25,
        };

        // 不存在
        let none: Option<TestUser> = storage.get_opt("user:1").await.unwrap();
        assert!(none.is_none());

        // 存在
        storage.set("user:1", &user).await.unwrap();
        let some: Option<TestUser> = storage.get_opt("user:1").await.unwrap();
        assert_eq!(some, Some(user));
    }

    #[tokio::test]
    async fn test_typed_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let user = TestUser {
            name: "Charlie".to_string(),
            age: 35,
        };

        storage.set("user:1", &user).await.unwrap();
        assert!(storage.exists("user:1").await.unwrap());

        storage.delete("user:1").await.unwrap();
        assert!(!storage.exists("user:1").await.unwrap());
    }

    #[tokio::test]
    async fn test_typed_storage_complex_type() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let config = TestConfig {
            debug: true,
            max_items: 100,
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        storage.set("config", &config).await.unwrap();
        let loaded: TestConfig = storage.get("config").await.unwrap();

        assert_eq!(loaded, config);
    }

    #[tokio::test]
    async fn test_typed_storage_bincode() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::with_bincode(backend);

        let user = TestUser {
            name: "Dave".to_string(),
            age: 40,
        };

        storage.set("user:1", &user).await.unwrap();
        let loaded: TestUser = storage.get("user:1").await.unwrap();

        assert_eq!(loaded, user);
        assert_eq!(storage.format(), SerializationFormat::Bincode);
    }

    #[tokio::test]
    async fn test_typed_storage_set_many() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let users = vec![
            TestUser { name: "A".to_string(), age: 1 },
            TestUser { name: "B".to_string(), age: 2 },
            TestUser { name: "C".to_string(), age: 3 },
        ];

        let items: Vec<(&str, &TestUser)> = vec![
            ("user:1", &users[0]),
            ("user:2", &users[1]),
            ("user:3", &users[2]),
        ];

        let count = storage.set_many(&items).await.unwrap();
        assert_eq!(count, 3);

        let loaded: TestUser = storage.get("user:2").await.unwrap();
        assert_eq!(loaded, users[1]);
    }

    #[tokio::test]
    async fn test_typed_storage_get_many() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        for i in 1..=3 {
            let user = TestUser {
                name: format!("User{}", i),
                age: i * 10,
            };
            storage.set(&format!("user:{}", i), &user).await.unwrap();
        }

        let results: Vec<(String, TestUser)> = storage
            .get_many(&["user:1", "user:3", "user:999"])
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_typed_storage_list() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        storage.set("user:1", &1i32).await.unwrap();
        storage.set("user:2", &2i32).await.unwrap();
        storage.set("config", &3i32).await.unwrap();

        let all_keys = storage.list("").await.unwrap();
        assert_eq!(all_keys.len(), 3);

        let user_keys = storage.list("user").await.unwrap();
        assert_eq!(user_keys.len(), 2);
    }

    #[tokio::test]
    async fn test_typed_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let user = TestUser {
            name: "Eve".to_string(),
            age: 28,
        };

        storage.set("user:1", &user).await.unwrap();
        let _: TestUser = storage.get("user:1").await.unwrap();

        let stats = storage.typed_stats();
        assert_eq!(stats.serializations, 1);
        assert_eq!(stats.deserializations, 1);
        assert!(stats.serialization_bytes > 0);
    }

    #[tokio::test]
    async fn test_typed_storage_pretty_json() {
        let backend = MemoryStorage::new();
        let config = TypedStorageConfig {
            format: SerializationFormat::Json,
            pretty_json: true,
        };
        let storage = TypedStorage::with_config(backend, config);

        let user = TestUser {
            name: "Frank".to_string(),
            age: 50,
        };

        storage.set("user:1", &user).await.unwrap();

        // 验证 JSON 是美化格式（包含换行）
        let bytes = storage.backend().read("user:1").await.unwrap();
        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains('\n'));
    }

    #[tokio::test]
    async fn test_typed_storage_not_found() {
        let backend = MemoryStorage::new();
        let storage = TypedStorage::new(backend);

        let result: StorageResult<TestUser> = storage.get("nonexistent").await;
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    // ========================================================================
    // TypedCollection 测试
    // ========================================================================

    #[tokio::test]
    async fn test_typed_collection_insert_get() {
        let backend = MemoryStorage::new();
        let collection: TypedCollection<_, TestUser> = TypedCollection::new(backend, "users");

        let user = TestUser {
            name: "Grace".to_string(),
            age: 33,
        };

        collection.insert("1", &user).await.unwrap();
        let loaded = collection.get("1").await.unwrap();

        assert_eq!(loaded, user);
    }

    #[tokio::test]
    async fn test_typed_collection_list_ids() {
        let backend = MemoryStorage::new();
        let collection: TypedCollection<_, TestUser> = TypedCollection::new(backend, "users");

        for i in 1..=3 {
            let user = TestUser {
                name: format!("User{}", i),
                age: i * 10,
            };
            collection.insert(&i.to_string(), &user).await.unwrap();
        }

        let ids = collection.list_ids().await.unwrap();
        assert_eq!(ids.len(), 3);
    }

    #[tokio::test]
    async fn test_typed_collection_get_all() {
        let backend = MemoryStorage::new();
        let collection: TypedCollection<_, TestUser> = TypedCollection::new(backend, "users");

        for i in 1..=2 {
            let user = TestUser {
                name: format!("User{}", i),
                age: i * 10,
            };
            collection.insert(&i.to_string(), &user).await.unwrap();
        }

        let all = collection.get_all().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_typed_collection_remove() {
        let backend = MemoryStorage::new();
        let collection: TypedCollection<_, TestUser> = TypedCollection::new(backend, "users");

        let user = TestUser {
            name: "Henry".to_string(),
            age: 45,
        };

        collection.insert("1", &user).await.unwrap();
        assert!(collection.contains("1").await.unwrap());

        collection.remove("1").await.unwrap();
        assert!(!collection.contains("1").await.unwrap());
    }

    #[tokio::test]
    async fn test_typed_collection_count() {
        let backend = MemoryStorage::new();
        let collection: TypedCollection<_, i32> = TypedCollection::new(backend, "numbers");

        assert_eq!(collection.count().await.unwrap(), 0);

        collection.insert("a", &1).await.unwrap();
        collection.insert("b", &2).await.unwrap();

        assert_eq!(collection.count().await.unwrap(), 2);
    }

    // ========================================================================
    // 便捷函数测试
    // ========================================================================

    #[tokio::test]
    async fn test_json_storage_helper() {
        let backend = MemoryStorage::new();
        let storage = json_storage(backend);

        assert_eq!(storage.format(), SerializationFormat::Json);
    }

    #[tokio::test]
    async fn test_bincode_storage_helper() {
        let backend = MemoryStorage::new();
        let storage = bincode_storage(backend);

        assert_eq!(storage.format(), SerializationFormat::Bincode);
    }

    #[tokio::test]
    async fn test_default_config() {
        let config = TypedStorageConfig::default();

        assert_eq!(config.format, SerializationFormat::Json);
        assert!(!config.pretty_json);
    }
}
