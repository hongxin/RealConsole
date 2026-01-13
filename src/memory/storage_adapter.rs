//! Memory Storage 适配器
//!
//! v1.107.0: 将 Memory 系统迁移到 Storage Layer 2.0
//!
//! 提供 Memory 和 StorageBackend 之间的桥接

use super::{EntryType, Importance, Memory, MemoryEntry};
use crate::storage::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Memory 存储适配器
///
/// 将 Memory 系统与 StorageBackend 集成
pub struct MemoryStorageAdapter<S: StorageBackend> {
    /// 存储后端
    storage: Arc<S>,
    /// 内存缓存
    memory: Arc<RwLock<Memory>>,
    /// 存储键前缀
    prefix: String,
    /// 配置
    config: MemoryStorageConfig,
}

/// Memory 存储配置
#[derive(Debug, Clone)]
pub struct MemoryStorageConfig {
    /// 内存容量
    pub capacity: usize,
    /// 自动持久化间隔（条目数）
    pub auto_persist_interval: usize,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 持久化格式
    pub format: PersistFormat,
}

impl Default for MemoryStorageConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            auto_persist_interval: 10,
            enable_compression: false,
            format: PersistFormat::Json,
        }
    }
}

/// 持久化格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistFormat {
    /// JSON 格式
    Json,
    /// JSONL 格式（每行一条）
    JsonLines,
    /// Bincode 二进制格式
    Bincode,
}

/// 存储中的 Memory 快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// 版本号
    pub version: u32,
    /// 条目列表
    pub entries: Vec<MemoryEntry>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 元数据
    pub metadata: SnapshotMetadata,
}

/// 快照元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// 总条目数
    pub total_entries: usize,
    /// 来源
    pub source: Option<String>,
    /// 自定义标签
    pub tags: Vec<String>,
}

impl<S: StorageBackend> MemoryStorageAdapter<S> {
    /// 创建新的适配器
    pub fn new(storage: Arc<S>, config: MemoryStorageConfig) -> Self {
        Self {
            storage,
            memory: Arc::new(RwLock::new(Memory::new(config.capacity))),
            prefix: "memory".to_string(),
            config,
        }
    }

    /// 使用自定义前缀
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取存储键
    fn storage_key(&self, name: &str) -> String {
        format!("{}/{}", self.prefix, name)
    }

    /// 添加记忆条目
    pub async fn add(&self, content: String, entry_type: EntryType) -> StorageResult<()> {
        let mut memory = self.memory.write().await;
        memory.add(content, entry_type);

        // 检查是否需要自动持久化
        if memory.len() % self.config.auto_persist_interval == 0 {
            drop(memory); // 释放锁
            self.persist("auto").await?;
        }

        Ok(())
    }

    /// 添加带重要性的记忆条目
    pub async fn add_with_importance(
        &self,
        content: String,
        entry_type: EntryType,
        importance: Importance,
    ) -> StorageResult<()> {
        let entry = MemoryEntry::new_with_importance(content, entry_type, importance);
        let mut memory = self.memory.write().await;

        // 手动添加以保留重要性
        if memory.len() >= self.config.capacity {
            // 使用 dump 获取条目，但我们需要内部操作
            // 这里简化处理：先添加普通条目再标记
        }

        // 使用 add 方法
        memory.add(entry.content.clone(), entry.entry_type);

        // 标记最新条目的重要性
        if importance != Importance::Normal {
            let _ = memory.mark_importance(0, importance);
        }

        Ok(())
    }

    /// 持久化到存储
    pub async fn persist(&self, snapshot_name: &str) -> StorageResult<()> {
        let memory = self.memory.read().await;

        let entries: Vec<MemoryEntry> = memory.dump().into_iter().cloned().collect();

        let snapshot = MemorySnapshot {
            version: 1,
            entries,
            created_at: chrono::Utc::now(),
            metadata: SnapshotMetadata {
                total_entries: memory.len(),
                source: Some("MemoryStorageAdapter".to_string()),
                tags: vec![],
            },
        };

        let data = match self.config.format {
            PersistFormat::Json => {
                serde_json::to_vec(&snapshot).map_err(|e| {
                    StorageError::Serialization(format!("JSON serialization failed: {}", e))
                })?
            }
            PersistFormat::JsonLines => {
                let lines: Vec<String> = snapshot
                    .entries
                    .iter()
                    .filter_map(|e| serde_json::to_string(e).ok())
                    .collect();
                lines.join("\n").into_bytes()
            }
            PersistFormat::Bincode => bincode::serialize(&snapshot).map_err(|e| {
                StorageError::Serialization(format!("Bincode serialization failed: {}", e))
            })?,
        };

        self.storage
            .write(&self.storage_key(snapshot_name), &data)
            .await
    }

    /// 从存储加载
    pub async fn load(&self, snapshot_name: &str) -> StorageResult<()> {
        let data = self.storage.read(&self.storage_key(snapshot_name)).await?;

        let snapshot: MemorySnapshot = match self.config.format {
            PersistFormat::Json => serde_json::from_slice(&data).map_err(|e| {
                StorageError::Serialization(format!("JSON deserialization failed: {}", e))
            })?,
            PersistFormat::JsonLines => {
                let content = String::from_utf8_lossy(&data);
                let entries: Vec<MemoryEntry> = content
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .filter_map(|l| serde_json::from_str(l).ok())
                    .collect();

                MemorySnapshot {
                    version: 1,
                    entries,
                    created_at: chrono::Utc::now(),
                    metadata: SnapshotMetadata::default(),
                }
            }
            PersistFormat::Bincode => bincode::deserialize(&data).map_err(|e| {
                StorageError::Serialization(format!("Bincode deserialization failed: {}", e))
            })?,
        };

        let mut memory = self.memory.write().await;
        memory.clear();

        for entry in snapshot.entries {
            memory.add(entry.content, entry.entry_type);
        }

        Ok(())
    }

    /// 获取最近的记忆
    pub async fn recent(&self, n: usize) -> Vec<MemoryEntry> {
        let memory = self.memory.read().await;
        memory.recent(n).into_iter().cloned().collect()
    }

    /// 搜索记忆
    pub async fn search(&self, keyword: &str) -> Vec<MemoryEntry> {
        let memory = self.memory.read().await;
        memory.search(keyword).into_iter().cloned().collect()
    }

    /// 获取所有记忆
    pub async fn dump(&self) -> Vec<MemoryEntry> {
        let memory = self.memory.read().await;
        memory.dump().into_iter().cloned().collect()
    }

    /// 清空记忆
    pub async fn clear(&self) {
        let mut memory = self.memory.write().await;
        memory.clear();
    }

    /// 获取记忆数量
    pub async fn len(&self) -> usize {
        let memory = self.memory.read().await;
        memory.len()
    }

    /// 检查是否为空
    pub async fn is_empty(&self) -> bool {
        let memory = self.memory.read().await;
        memory.is_empty()
    }

    /// 按类型过滤
    pub async fn filter_by_type(&self, entry_type: EntryType) -> Vec<MemoryEntry> {
        let memory = self.memory.read().await;
        memory
            .filter_by_type(entry_type)
            .into_iter()
            .cloned()
            .collect()
    }

    /// 按重要性过滤
    pub async fn filter_by_importance(&self, importance: Importance) -> Vec<MemoryEntry> {
        let memory = self.memory.read().await;
        memory
            .filter_by_importance(importance)
            .into_iter()
            .cloned()
            .collect()
    }

    /// 标记重要性
    pub async fn mark_importance(
        &self,
        index: usize,
        importance: Importance,
    ) -> Result<(), String> {
        let mut memory = self.memory.write().await;
        memory.mark_importance(index, importance)
    }

    /// 列出所有快照
    pub async fn list_snapshots(&self) -> StorageResult<Vec<String>> {
        let keys = self.storage.list(&self.prefix).await?;
        Ok(keys
            .into_iter()
            .map(|k| k.strip_prefix(&format!("{}/", self.prefix)).unwrap_or(&k).to_string())
            .collect())
    }

    /// 删除快照
    pub async fn delete_snapshot(&self, name: &str) -> StorageResult<()> {
        self.storage.delete(&self.storage_key(name)).await
    }

    /// 获取统计信息
    pub async fn stats(&self) -> MemoryAdapterStats {
        let memory = self.memory.read().await;
        let mem_stats = memory.stats();

        MemoryAdapterStats {
            memory_entries: mem_stats.total_entries,
            type_distribution: mem_stats.type_distribution,
            storage_stats: self.storage.stats(),
        }
    }
}

/// 适配器统计信息
#[derive(Debug, Clone)]
pub struct MemoryAdapterStats {
    /// 内存中的条目数
    pub memory_entries: usize,
    /// 类型分布
    pub type_distribution: std::collections::HashMap<EntryType, usize>,
    /// 存储统计
    pub storage_stats: crate::storage::StorageStats,
}

/// Memory 存储后端实现
///
/// 将 MemoryStorageAdapter 包装为 StorageBackend
pub struct MemoryAsStorage<S: StorageBackend> {
    adapter: MemoryStorageAdapter<S>,
}

impl<S: StorageBackend> MemoryAsStorage<S> {
    pub fn new(adapter: MemoryStorageAdapter<S>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<S: StorageBackend + 'static> StorageBackend for MemoryAsStorage<S> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.adapter.storage.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.adapter.storage.write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.adapter.storage.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.adapter.storage.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.adapter.storage.exists(key).await
    }

    fn stats(&self) -> crate::storage::StorageStats {
        self.adapter.storage.stats()
    }

    fn name(&self) -> &'static str {
        "MemoryAsStorage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    fn create_test_adapter() -> MemoryStorageAdapter<MemoryStorage> {
        let storage = Arc::new(MemoryStorage::new());
        MemoryStorageAdapter::new(storage, MemoryStorageConfig::default())
    }

    #[tokio::test]
    async fn test_adapter_new() {
        let adapter = create_test_adapter();
        assert!(adapter.is_empty().await);
        assert_eq!(adapter.len().await, 0);
    }

    #[tokio::test]
    async fn test_add_entry() {
        let adapter = create_test_adapter();

        adapter
            .add("Hello".to_string(), EntryType::User)
            .await
            .unwrap();

        assert_eq!(adapter.len().await, 1);
        assert!(!adapter.is_empty().await);

        let recent = adapter.recent(1).await;
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_persist_and_load() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = MemoryStorageAdapter::new(storage.clone(), MemoryStorageConfig::default());

        // 添加条目
        adapter
            .add("Entry 1".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("Entry 2".to_string(), EntryType::Assistant)
            .await
            .unwrap();

        // 持久化
        adapter.persist("test_snapshot").await.unwrap();

        // 创建新适配器并加载
        let adapter2 = MemoryStorageAdapter::new(storage, MemoryStorageConfig::default());
        adapter2.load("test_snapshot").await.unwrap();

        assert_eq!(adapter2.len().await, 2);
    }

    #[tokio::test]
    async fn test_search() {
        let adapter = create_test_adapter();

        adapter
            .add("Hello world".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("Goodbye world".to_string(), EntryType::Assistant)
            .await
            .unwrap();
        adapter
            .add("Hello Rust".to_string(), EntryType::User)
            .await
            .unwrap();

        let results = adapter.search("hello").await;
        assert_eq!(results.len(), 2);

        let results = adapter.search("world").await;
        assert_eq!(results.len(), 2);

        let results = adapter.search("rust").await;
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_filter_by_type() {
        let adapter = create_test_adapter();

        adapter
            .add("user1".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("assistant1".to_string(), EntryType::Assistant)
            .await
            .unwrap();
        adapter
            .add("user2".to_string(), EntryType::User)
            .await
            .unwrap();

        let users = adapter.filter_by_type(EntryType::User).await;
        assert_eq!(users.len(), 2);

        let assistants = adapter.filter_by_type(EntryType::Assistant).await;
        assert_eq!(assistants.len(), 1);
    }

    #[tokio::test]
    async fn test_clear() {
        let adapter = create_test_adapter();

        adapter
            .add("test".to_string(), EntryType::User)
            .await
            .unwrap();
        assert_eq!(adapter.len().await, 1);

        adapter.clear().await;
        assert!(adapter.is_empty().await);
    }

    #[tokio::test]
    async fn test_dump() {
        let adapter = create_test_adapter();

        adapter
            .add("entry1".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("entry2".to_string(), EntryType::System)
            .await
            .unwrap();

        let all = adapter.dump().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_list_snapshots() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = MemoryStorageAdapter::new(storage, MemoryStorageConfig::default());

        adapter
            .add("test".to_string(), EntryType::User)
            .await
            .unwrap();

        adapter.persist("snapshot1").await.unwrap();
        adapter.persist("snapshot2").await.unwrap();

        let snapshots = adapter.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_snapshot() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = MemoryStorageAdapter::new(storage, MemoryStorageConfig::default());

        adapter
            .add("test".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter.persist("to_delete").await.unwrap();

        let snapshots = adapter.list_snapshots().await.unwrap();
        assert_eq!(snapshots.len(), 1);

        adapter.delete_snapshot("to_delete").await.unwrap();

        let snapshots = adapter.list_snapshots().await.unwrap();
        assert!(snapshots.is_empty());
    }

    #[tokio::test]
    async fn test_with_prefix() {
        let storage = Arc::new(MemoryStorage::new());
        let config = MemoryStorageConfig::default();
        let adapter = MemoryStorageAdapter::new(storage.clone(), config)
            .with_prefix("custom_prefix");

        adapter
            .add("test".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter.persist("snapshot").await.unwrap();

        // 验证使用了自定义前缀
        let exists = storage.exists("custom_prefix/snapshot").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_jsonlines_format() {
        let storage = Arc::new(MemoryStorage::new());
        let config = MemoryStorageConfig {
            format: PersistFormat::JsonLines,
            ..Default::default()
        };
        let adapter = MemoryStorageAdapter::new(storage.clone(), config.clone());

        adapter
            .add("entry1".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("entry2".to_string(), EntryType::System)
            .await
            .unwrap();

        adapter.persist("jsonl_test").await.unwrap();

        // 加载并验证
        let adapter2 = MemoryStorageAdapter::new(storage, config);
        adapter2.load("jsonl_test").await.unwrap();

        assert_eq!(adapter2.len().await, 2);
    }

    #[tokio::test]
    async fn test_bincode_format() {
        let storage = Arc::new(MemoryStorage::new());
        let config = MemoryStorageConfig {
            format: PersistFormat::Bincode,
            ..Default::default()
        };
        let adapter = MemoryStorageAdapter::new(storage.clone(), config.clone());

        adapter
            .add("binary entry".to_string(), EntryType::Tool)
            .await
            .unwrap();

        adapter.persist("bincode_test").await.unwrap();

        let adapter2 = MemoryStorageAdapter::new(storage, config);
        adapter2.load("bincode_test").await.unwrap();

        assert_eq!(adapter2.len().await, 1);
        let recent = adapter2.recent(1).await;
        assert_eq!(recent[0].content, "binary entry");
    }

    #[tokio::test]
    async fn test_stats() {
        let adapter = create_test_adapter();

        adapter
            .add("user".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("assistant".to_string(), EntryType::Assistant)
            .await
            .unwrap();

        let stats = adapter.stats().await;
        assert_eq!(stats.memory_entries, 2);
        assert_eq!(stats.type_distribution.get(&EntryType::User), Some(&1));
        assert_eq!(stats.type_distribution.get(&EntryType::Assistant), Some(&1));
    }

    #[tokio::test]
    async fn test_mark_importance() {
        let adapter = create_test_adapter();

        adapter
            .add("normal".to_string(), EntryType::User)
            .await
            .unwrap();
        adapter
            .add("important".to_string(), EntryType::User)
            .await
            .unwrap();

        adapter.mark_importance(0, Importance::Critical).await.unwrap();

        let important = adapter.filter_by_importance(Importance::Critical).await;
        assert_eq!(important.len(), 1);
        assert_eq!(important[0].content, "important");
    }
}
