//! Web Session Storage 适配器
//!
//! v1.108.0: 将 Web Session 存储迁移到 Storage Layer 2.0
//!
//! 提供 SessionManager 和 StorageBackend 之间的桥接

use super::session_manager::{SerializableSession, SessionListItem, SessionMetadata};
use crate::storage::{StorageBackend, StorageError, StorageResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Session 存储适配器
///
/// 将 Web Session 管理与 StorageBackend 集成
pub struct SessionStorageAdapter<S: StorageBackend> {
    /// 存储后端
    storage: Arc<S>,
    /// 存储键前缀
    prefix: String,
    /// 配置
    config: SessionStorageConfig,
}

/// Session 存储配置
#[derive(Debug, Clone)]
pub struct SessionStorageConfig {
    /// 是否压缩存储
    pub compress: bool,
    /// 是否存储元数据
    pub store_metadata: bool,
    /// 索引刷新间隔（秒）
    pub index_refresh_interval: u64,
}

impl Default for SessionStorageConfig {
    fn default() -> Self {
        Self {
            compress: false,
            store_metadata: true,
            index_refresh_interval: 60,
        }
    }
}

/// Session 索引
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIndex {
    /// 版本号
    pub version: u32,
    /// Session 列表
    pub sessions: Vec<SessionIndexEntry>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

impl Default for SessionIndex {
    fn default() -> Self {
        Self {
            version: 1,
            sessions: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

/// Session 索引条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionIndexEntry {
    /// Session ID
    pub id: String,
    /// Session 名称
    pub name: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
    /// 回合数
    pub round_count: usize,
}

impl From<&SerializableSession> for SessionIndexEntry {
    fn from(session: &SerializableSession) -> Self {
        Self {
            id: session.id.clone(),
            name: session.name.clone(),
            created_at: session.created_at,
            updated_at: session.updated_at,
            round_count: session.rounds.len(),
        }
    }
}

impl<S: StorageBackend> SessionStorageAdapter<S> {
    /// 创建新的适配器
    pub fn new(storage: Arc<S>) -> Self {
        Self::with_config(storage, SessionStorageConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(storage: Arc<S>, config: SessionStorageConfig) -> Self {
        Self {
            storage,
            prefix: "sessions".to_string(),
            config,
        }
    }

    /// 设置前缀
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// 获取存储键
    fn storage_key(&self, id: &str) -> String {
        format!("{}/session-{}", self.prefix, id)
    }

    /// 获取索引键
    fn index_key(&self) -> String {
        format!("{}/index", self.prefix)
    }

    /// 保存 Session
    pub async fn save(&self, session: &SerializableSession) -> StorageResult<()> {
        let data = serde_json::to_vec_pretty(session).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize session: {}", e))
        })?;

        self.storage
            .write(&self.storage_key(&session.id), &data)
            .await?;

        // 更新索引
        self.update_index(session).await?;

        Ok(())
    }

    /// 加载 Session
    pub async fn load(&self, id: &str) -> StorageResult<SerializableSession> {
        let data = self.storage.read(&self.storage_key(id)).await?;

        serde_json::from_slice(&data).map_err(|e| {
            StorageError::Serialization(format!("Failed to deserialize session: {}", e))
        })
    }

    /// 删除 Session
    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        self.storage.delete(&self.storage_key(id)).await?;

        // 从索引中移除
        self.remove_from_index(id).await?;

        Ok(())
    }

    /// 检查 Session 是否存在
    pub async fn exists(&self, id: &str) -> StorageResult<bool> {
        self.storage.exists(&self.storage_key(id)).await
    }

    /// 列出所有 Session
    pub async fn list(&self) -> StorageResult<Vec<SessionListItem>> {
        let index = self.load_index().await?;

        Ok(index
            .sessions
            .iter()
            .map(|entry| SessionListItem {
                id: entry.id.clone(),
                name: entry.name.clone(),
                created_at: entry.created_at,
                updated_at: entry.updated_at,
                round_count: entry.round_count,
                last_message: String::new(), // 索引不包含消息预览
            })
            .collect())
    }

    /// 列出完整 Session 信息（包含预览）
    pub async fn list_full(&self) -> StorageResult<Vec<SessionListItem>> {
        let keys = self.storage.list(&self.prefix).await?;

        let mut items = Vec::new();
        for key in keys {
            if key.ends_with("/index") {
                continue;
            }

            if let Some(id) = key
                .strip_prefix(&format!("{}/session-", self.prefix))
            {
                if let Ok(session) = self.load(id).await {
                    items.push(SessionListItem::from(&session));
                }
            }
        }

        // 按更新时间倒序
        items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(items)
    }

    /// 重命名 Session
    pub async fn rename(&self, id: &str, new_name: &str) -> StorageResult<()> {
        let mut session = self.load(id).await?;
        session.name = new_name.to_string();
        session.updated_at = Utc::now();
        self.save(&session).await
    }

    /// 加载索引
    async fn load_index(&self) -> StorageResult<SessionIndex> {
        match self.storage.read(&self.index_key()).await {
            Ok(data) => serde_json::from_slice(&data).map_err(|e| {
                StorageError::Serialization(format!("Failed to deserialize index: {}", e))
            }),
            Err(StorageError::NotFound(_)) => Ok(SessionIndex::default()),
            Err(e) => Err(e),
        }
    }

    /// 保存索引
    async fn save_index(&self, index: &SessionIndex) -> StorageResult<()> {
        let data = serde_json::to_vec(index).map_err(|e| {
            StorageError::Serialization(format!("Failed to serialize index: {}", e))
        })?;

        self.storage.write(&self.index_key(), &data).await
    }

    /// 更新索引
    async fn update_index(&self, session: &SerializableSession) -> StorageResult<()> {
        let mut index = self.load_index().await?;

        // 移除已存在的条目
        index.sessions.retain(|e| e.id != session.id);

        // 添加新条目
        index.sessions.push(SessionIndexEntry::from(session));

        // 按更新时间排序
        index.sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        index.updated_at = Utc::now();

        self.save_index(&index).await
    }

    /// 从索引中移除
    async fn remove_from_index(&self, id: &str) -> StorageResult<()> {
        let mut index = self.load_index().await?;
        index.sessions.retain(|e| e.id != id);
        index.updated_at = Utc::now();
        self.save_index(&index).await
    }

    /// 重建索引
    pub async fn rebuild_index(&self) -> StorageResult<usize> {
        let keys = self.storage.list(&self.prefix).await?;

        let mut index = SessionIndex::default();

        for key in keys {
            if key.ends_with("/index") {
                continue;
            }

            if let Some(id) = key.strip_prefix(&format!("{}/session-", self.prefix)) {
                if let Ok(session) = self.load(id).await {
                    index.sessions.push(SessionIndexEntry::from(&session));
                }
            }
        }

        // 按更新时间排序
        index.sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        let count = index.sessions.len();
        self.save_index(&index).await?;

        Ok(count)
    }

    /// 获取统计信息
    pub async fn stats(&self) -> StorageResult<SessionStorageStats> {
        let index = self.load_index().await?;

        let total_rounds: usize = index.sessions.iter().map(|e| e.round_count).sum();

        let oldest = index.sessions.iter().map(|e| e.created_at).min();
        let newest = index.sessions.iter().map(|e| e.updated_at).max();

        Ok(SessionStorageStats {
            session_count: index.sessions.len(),
            total_rounds,
            oldest_session: oldest,
            newest_session: newest,
            index_updated_at: index.updated_at,
        })
    }

    /// 搜索 Session
    pub async fn search(&self, query: &str) -> StorageResult<Vec<SessionListItem>> {
        let index = self.load_index().await?;
        let query_lower = query.to_lowercase();

        let matches: Vec<SessionListItem> = index
            .sessions
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&query_lower) || e.id.contains(query))
            .map(|e| SessionListItem {
                id: e.id.clone(),
                name: e.name.clone(),
                created_at: e.created_at,
                updated_at: e.updated_at,
                round_count: e.round_count,
                last_message: String::new(),
            })
            .collect();

        Ok(matches)
    }

    /// 清空所有 Session
    pub async fn clear(&self) -> StorageResult<usize> {
        let keys = self.storage.list(&self.prefix).await?;
        let count = keys.len();

        for key in keys {
            self.storage.delete(&key).await?;
        }

        Ok(count)
    }
}

/// Session 存储统计
#[derive(Debug, Clone)]
pub struct SessionStorageStats {
    /// Session 数量
    pub session_count: usize,
    /// 总回合数
    pub total_rounds: usize,
    /// 最早的 Session
    pub oldest_session: Option<DateTime<Utc>>,
    /// 最新的 Session
    pub newest_session: Option<DateTime<Utc>>,
    /// 索引更新时间
    pub index_updated_at: DateTime<Utc>,
}

/// Session 存储后端包装
///
/// 将 SessionStorageAdapter 包装为 StorageBackend
pub struct SessionAsStorage<S: StorageBackend> {
    adapter: SessionStorageAdapter<S>,
}

impl<S: StorageBackend> SessionAsStorage<S> {
    pub fn new(adapter: SessionStorageAdapter<S>) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<S: StorageBackend + 'static> StorageBackend for SessionAsStorage<S> {
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
        "SessionAsStorage"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    fn create_test_adapter() -> SessionStorageAdapter<MemoryStorage> {
        let storage = Arc::new(MemoryStorage::new());
        SessionStorageAdapter::new(storage)
    }

    fn create_test_session(id: &str, name: &str) -> SerializableSession {
        SerializableSession {
            id: id.to_string(),
            name: name.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            conversation_id: format!("conv-{}", id),
            rounds: vec![],
            chart_history: vec![],
            image_history: vec![],
            metadata: None,
            version: "1.0".to_string(),
        }
    }

    #[tokio::test]
    async fn test_adapter_new() {
        let adapter = create_test_adapter();
        let list = adapter.list().await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_save_and_load() {
        let adapter = create_test_adapter();

        let session = create_test_session("test-1", "Test Session");
        adapter.save(&session).await.unwrap();

        let loaded = adapter.load("test-1").await.unwrap();
        assert_eq!(loaded.id, "test-1");
        assert_eq!(loaded.name, "Test Session");
    }

    #[tokio::test]
    async fn test_delete() {
        let adapter = create_test_adapter();

        let session = create_test_session("test-del", "Delete Me");
        adapter.save(&session).await.unwrap();

        assert!(adapter.exists("test-del").await.unwrap());

        adapter.delete("test-del").await.unwrap();

        assert!(!adapter.exists("test-del").await.unwrap());
    }

    #[tokio::test]
    async fn test_list() {
        let adapter = create_test_adapter();

        adapter
            .save(&create_test_session("s1", "Session 1"))
            .await
            .unwrap();
        adapter
            .save(&create_test_session("s2", "Session 2"))
            .await
            .unwrap();
        adapter
            .save(&create_test_session("s3", "Session 3"))
            .await
            .unwrap();

        let list = adapter.list().await.unwrap();
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn test_rename() {
        let adapter = create_test_adapter();

        let session = create_test_session("rename-test", "Old Name");
        adapter.save(&session).await.unwrap();

        adapter.rename("rename-test", "New Name").await.unwrap();

        let loaded = adapter.load("rename-test").await.unwrap();
        assert_eq!(loaded.name, "New Name");
    }

    #[tokio::test]
    async fn test_search() {
        let adapter = create_test_adapter();

        adapter
            .save(&create_test_session("a1", "Alpha Session"))
            .await
            .unwrap();
        adapter
            .save(&create_test_session("b1", "Beta Session"))
            .await
            .unwrap();
        adapter
            .save(&create_test_session("a2", "Another Alpha"))
            .await
            .unwrap();

        let results = adapter.search("alpha").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_rebuild_index() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = SessionStorageAdapter::new(storage.clone());

        // 直接写入数据（绕过索引）
        let session = create_test_session("rebuild-1", "Rebuild Test");
        let data = serde_json::to_vec(&session).unwrap();
        storage
            .write("sessions/session-rebuild-1", &data)
            .await
            .unwrap();

        // 索引应该是空的
        let list = adapter.list().await.unwrap();
        assert!(list.is_empty());

        // 重建索引
        let count = adapter.rebuild_index().await.unwrap();
        assert_eq!(count, 1);

        // 现在应该能找到
        let list = adapter.list().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let adapter = create_test_adapter();

        let mut session = create_test_session("stats-1", "Stats Test");
        // 添加两个回合来测试统计
        session.rounds = vec![
            crate::web::session::ConversationRound::new(
                1,
                crate::web::session::RoundType::Llm,
                "test input 1".to_string(),
                "test-model".to_string(),
            ),
            crate::web::session::ConversationRound::new(
                2,
                crate::web::session::RoundType::Llm,
                "test input 2".to_string(),
                "test-model".to_string(),
            ),
        ];
        adapter.save(&session).await.unwrap();

        let stats = adapter.stats().await.unwrap();
        assert_eq!(stats.session_count, 1);
        assert_eq!(stats.total_rounds, 2);
    }

    #[tokio::test]
    async fn test_clear() {
        let adapter = create_test_adapter();

        adapter
            .save(&create_test_session("c1", "Clear 1"))
            .await
            .unwrap();
        adapter
            .save(&create_test_session("c2", "Clear 2"))
            .await
            .unwrap();

        let count = adapter.clear().await.unwrap();
        // 清除 2 个 session + 1 个 index = 3
        assert!(count >= 2);

        let list = adapter.list().await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_with_prefix() {
        let storage = Arc::new(MemoryStorage::new());
        let adapter = SessionStorageAdapter::new(storage.clone()).with_prefix("custom");

        let session = create_test_session("prefix-1", "Prefix Test");
        adapter.save(&session).await.unwrap();

        // 验证使用了自定义前缀
        let exists = storage.exists("custom/session-prefix-1").await.unwrap();
        assert!(exists);
    }

    #[tokio::test]
    async fn test_exists() {
        let adapter = create_test_adapter();

        assert!(!adapter.exists("nonexistent").await.unwrap());

        adapter
            .save(&create_test_session("exists-test", "Test"))
            .await
            .unwrap();

        assert!(adapter.exists("exists-test").await.unwrap());
    }
}
