//! WatchableStorage - 变更通知存储层
//!
//! v1.75.0: 提供数据变更的订阅和通知机制
//!
//! ## 功能特性
//!
//! - **事件类型**: Created / Updated / Deleted
//! - **订阅模式**: 精确键匹配 / 前缀匹配 / 全局监听
//! - **异步通知**: 基于 tokio broadcast channel
//! - **订阅管理**: 支持取消订阅
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{WatchableStorage, MemoryStorage, StorageEvent};
//!
//! let storage = WatchableStorage::new(MemoryStorage::new());
//!
//! // 订阅特定键
//! let mut rx = storage.watch_key("user:123").await;
//!
//! // 订阅前缀
//! let mut rx = storage.watch_prefix("user:").await;
//!
//! // 全局监听
//! let mut rx = storage.watch_all().await;
//!
//! // 接收事件
//! while let Ok(event) = rx.recv().await {
//!     match event.event_type {
//!         EventType::Created => println!("Created: {}", event.key),
//!         EventType::Updated => println!("Updated: {}", event.key),
//!         EventType::Deleted => println!("Deleted: {}", event.key),
//!     }
//! }
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

// ============================================================================
// 事件类型
// ============================================================================

/// 存储事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    /// 新建数据
    Created,
    /// 更新数据
    Updated,
    /// 删除数据
    Deleted,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::Created => write!(f, "Created"),
            EventType::Updated => write!(f, "Updated"),
            EventType::Deleted => write!(f, "Deleted"),
        }
    }
}

/// 存储事件
#[derive(Debug, Clone)]
pub struct StorageEvent {
    /// 事件类型
    pub event_type: EventType,
    /// 数据键
    pub key: String,
    /// 数据大小（字节）
    pub size: usize,
    /// 事件时间戳
    pub timestamp: std::time::Instant,
}

impl StorageEvent {
    /// 创建新事件
    pub fn new(event_type: EventType, key: String, size: usize) -> Self {
        Self {
            event_type,
            key,
            size,
            timestamp: std::time::Instant::now(),
        }
    }

    /// 创建 Created 事件
    pub fn created(key: impl Into<String>, size: usize) -> Self {
        Self::new(EventType::Created, key.into(), size)
    }

    /// 创建 Updated 事件
    pub fn updated(key: impl Into<String>, size: usize) -> Self {
        Self::new(EventType::Updated, key.into(), size)
    }

    /// 创建 Deleted 事件
    pub fn deleted(key: impl Into<String>) -> Self {
        Self::new(EventType::Deleted, key.into(), 0)
    }
}

// ============================================================================
// 订阅模式
// ============================================================================

/// 订阅模式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchPattern {
    /// 精确键匹配
    ExactKey(String),
    /// 前缀匹配
    Prefix(String),
    /// 全局监听（所有事件）
    All,
}

impl WatchPattern {
    /// 检查键是否匹配此模式
    pub fn matches(&self, key: &str) -> bool {
        match self {
            WatchPattern::ExactKey(k) => k == key,
            WatchPattern::Prefix(p) => key.starts_with(p),
            WatchPattern::All => true,
        }
    }
}

/// 订阅 ID
pub type SubscriptionId = u64;

/// 订阅信息
struct Subscription {
    /// 订阅 ID
    id: SubscriptionId,
    /// 匹配模式
    pattern: WatchPattern,
    /// 事件发送器
    sender: broadcast::Sender<StorageEvent>,
}

// ============================================================================
// 配置
// ============================================================================

/// WatchableStorage 配置
#[derive(Debug, Clone)]
pub struct WatchableStorageConfig {
    /// 每个订阅的 channel 容量
    pub channel_capacity: usize,
    /// 是否在发送失败时记录警告
    pub log_send_failures: bool,
}

impl Default for WatchableStorageConfig {
    fn default() -> Self {
        Self {
            channel_capacity: 256,
            log_send_failures: false,
        }
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 订阅统计
#[derive(Debug, Default)]
pub struct WatchStats {
    /// 活跃订阅数
    active_subscriptions: AtomicU64,
    /// 总订阅数
    total_subscriptions: AtomicU64,
    /// 取消订阅数
    unsubscriptions: AtomicU64,
    /// 发送的事件数
    events_sent: AtomicU64,
    /// 发送失败数（无接收者）
    events_dropped: AtomicU64,
    /// Created 事件数
    created_events: AtomicU64,
    /// Updated 事件数
    updated_events: AtomicU64,
    /// Deleted 事件数
    deleted_events: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct WatchStatsSnapshot {
    pub active_subscriptions: u64,
    pub total_subscriptions: u64,
    pub unsubscriptions: u64,
    pub events_sent: u64,
    pub events_dropped: u64,
    pub created_events: u64,
    pub updated_events: u64,
    pub deleted_events: u64,
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedWatchStats {
    /// 快照统计
    pub snapshot: WatchStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 各模式订阅数
    pub subscriptions_by_pattern: SubscriptionsByPattern,
}

/// 按模式分类的订阅统计
#[derive(Debug, Clone, Default)]
pub struct SubscriptionsByPattern {
    pub exact_key: u64,
    pub prefix: u64,
    pub all: u64,
}

// ============================================================================
// WatchableStorage 实现
// ============================================================================

/// 可订阅变更的存储层
///
/// 装饰器模式，包装底层存储并提供变更通知
pub struct WatchableStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配置
    config: WatchableStorageConfig,
    /// 订阅列表
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Subscription>>>,
    /// 下一个订阅 ID
    next_id: AtomicU64,
    /// 统计信息
    stats: Arc<WatchStats>,
}

impl<B: StorageBackend> WatchableStorage<B> {
    /// 创建新的 WatchableStorage
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, WatchableStorageConfig::default())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_config(backend, WatchableStorageConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(backend: B, config: WatchableStorageConfig) -> Self {
        Self::from_arc_with_config(Arc::new(backend), config)
    }

    /// 从 Arc 使用配置创建
    pub fn from_arc_with_config(backend: Arc<B>, config: WatchableStorageConfig) -> Self {
        Self {
            backend,
            config,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicU64::new(1),
            stats: Arc::new(WatchStats::default()),
        }
    }

    /// 订阅特定键的变更
    pub async fn watch_key(&self, key: impl Into<String>) -> broadcast::Receiver<StorageEvent> {
        self.subscribe(WatchPattern::ExactKey(key.into())).await
    }

    /// 订阅前缀匹配的变更
    pub async fn watch_prefix(&self, prefix: impl Into<String>) -> broadcast::Receiver<StorageEvent> {
        self.subscribe(WatchPattern::Prefix(prefix.into())).await
    }

    /// 订阅所有变更
    pub async fn watch_all(&self) -> broadcast::Receiver<StorageEvent> {
        self.subscribe(WatchPattern::All).await
    }

    /// 通用订阅方法
    pub async fn subscribe(&self, pattern: WatchPattern) -> broadcast::Receiver<StorageEvent> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = broadcast::channel(self.config.channel_capacity);

        let subscription = Subscription {
            id,
            pattern,
            sender,
        };

        self.subscriptions.write().await.insert(id, subscription);

        self.stats.active_subscriptions.fetch_add(1, Ordering::SeqCst);
        self.stats.total_subscriptions.fetch_add(1, Ordering::SeqCst);

        receiver
    }

    /// 带 ID 的订阅（用于后续取消）
    pub async fn subscribe_with_id(&self, pattern: WatchPattern) -> (SubscriptionId, broadcast::Receiver<StorageEvent>) {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (sender, receiver) = broadcast::channel(self.config.channel_capacity);

        let subscription = Subscription {
            id,
            pattern,
            sender,
        };

        self.subscriptions.write().await.insert(id, subscription);

        self.stats.active_subscriptions.fetch_add(1, Ordering::SeqCst);
        self.stats.total_subscriptions.fetch_add(1, Ordering::SeqCst);

        (id, receiver)
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, id: SubscriptionId) -> bool {
        let removed = self.subscriptions.write().await.remove(&id).is_some();
        if removed {
            self.stats.active_subscriptions.fetch_sub(1, Ordering::SeqCst);
            self.stats.unsubscriptions.fetch_add(1, Ordering::SeqCst);
        }
        removed
    }

    /// 获取活跃订阅数
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    /// 通知所有匹配的订阅者
    async fn notify(&self, event: StorageEvent) {
        // 更新事件类型统计
        match event.event_type {
            EventType::Created => self.stats.created_events.fetch_add(1, Ordering::SeqCst),
            EventType::Updated => self.stats.updated_events.fetch_add(1, Ordering::SeqCst),
            EventType::Deleted => self.stats.deleted_events.fetch_add(1, Ordering::SeqCst),
        };

        let subscriptions = self.subscriptions.read().await;
        let mut to_remove = Vec::new();

        for (id, sub) in subscriptions.iter() {
            if sub.pattern.matches(&event.key) {
                match sub.sender.send(event.clone()) {
                    Ok(_) => {
                        self.stats.events_sent.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        // 没有接收者，标记删除
                        self.stats.events_dropped.fetch_add(1, Ordering::SeqCst);
                        to_remove.push(*id);
                    }
                }
            }
        }

        // 释放读锁
        drop(subscriptions);

        // 清理无效订阅
        if !to_remove.is_empty() {
            let mut subs = self.subscriptions.write().await;
            for id in to_remove {
                if subs.remove(&id).is_some() {
                    self.stats.active_subscriptions.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> WatchStatsSnapshot {
        WatchStatsSnapshot {
            active_subscriptions: self.stats.active_subscriptions.load(Ordering::SeqCst),
            total_subscriptions: self.stats.total_subscriptions.load(Ordering::SeqCst),
            unsubscriptions: self.stats.unsubscriptions.load(Ordering::SeqCst),
            events_sent: self.stats.events_sent.load(Ordering::SeqCst),
            events_dropped: self.stats.events_dropped.load(Ordering::SeqCst),
            created_events: self.stats.created_events.load(Ordering::SeqCst),
            updated_events: self.stats.updated_events.load(Ordering::SeqCst),
            deleted_events: self.stats.deleted_events.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> DetailedWatchStats {
        let subscriptions = self.subscriptions.read().await;
        let mut by_pattern = SubscriptionsByPattern::default();

        for sub in subscriptions.values() {
            match &sub.pattern {
                WatchPattern::ExactKey(_) => by_pattern.exact_key += 1,
                WatchPattern::Prefix(_) => by_pattern.prefix += 1,
                WatchPattern::All => by_pattern.all += 1,
            }
        }

        DetailedWatchStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            subscriptions_by_pattern: by_pattern,
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for WatchableStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        // 检查是否存在以确定事件类型
        let exists = self.backend.exists(key).await.unwrap_or(false);

        // 执行写入
        self.backend.write(key, data).await?;

        // 发送事件
        let event = if exists {
            StorageEvent::updated(key, data.len())
        } else {
            StorageEvent::created(key, data.len())
        };
        self.notify(event).await;

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // 检查是否存在
        let exists = self.backend.exists(key).await.unwrap_or(false);

        // 执行删除
        self.backend.delete(key).await?;

        // 只在确实删除了数据时发送事件
        if exists {
            self.notify(StorageEvent::deleted(key)).await;
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
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "WatchableStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// WatchableStorage 构建器
pub struct WatchableStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: WatchableStorageConfig,
}

impl<B: StorageBackend> WatchableStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: WatchableStorageConfig::default(),
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: WatchableStorageConfig::default(),
        }
    }

    /// 设置 channel 容量
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.config.channel_capacity = capacity;
        self
    }

    /// 设置是否记录发送失败
    pub fn log_send_failures(mut self, log: bool) -> Self {
        self.config.log_send_failures = log;
        self
    }

    /// 构建
    pub fn build(self) -> WatchableStorage<B> {
        WatchableStorage::from_arc_with_config(self.backend, self.config)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_watchable_storage_basic() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        // 写入数据
        storage.write("key1", b"value1").await.unwrap();

        // 读取数据
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_watch_key_created_event() {
        let storage = WatchableStorage::new(MemoryStorage::new());
        let mut rx = storage.watch_key("key1").await;

        // 写入新数据
        storage.write("key1", b"value1").await.unwrap();

        // 接收事件
        let event = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, EventType::Created);
        assert_eq!(event.key, "key1");
        assert_eq!(event.size, 6);
    }

    #[tokio::test]
    async fn test_watch_key_updated_event() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        // 先写入数据
        storage.write("key1", b"value1").await.unwrap();

        // 然后订阅
        let mut rx = storage.watch_key("key1").await;

        // 更新数据
        storage.write("key1", b"new_value").await.unwrap();

        // 接收事件
        let event = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, EventType::Updated);
        assert_eq!(event.key, "key1");
    }

    #[tokio::test]
    async fn test_watch_key_deleted_event() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        // 先写入数据
        storage.write("key1", b"value1").await.unwrap();

        // 订阅
        let mut rx = storage.watch_key("key1").await;

        // 删除数据
        storage.delete("key1").await.unwrap();

        // 接收事件
        let event = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(event.event_type, EventType::Deleted);
        assert_eq!(event.key, "key1");
    }

    #[tokio::test]
    async fn test_watch_prefix() {
        let storage = WatchableStorage::new(MemoryStorage::new());
        let mut rx = storage.watch_prefix("user:").await;

        // 写入匹配的键
        storage.write("user:123", b"data1").await.unwrap();
        storage.write("user:456", b"data2").await.unwrap();

        // 写入不匹配的键
        storage.write("order:789", b"data3").await.unwrap();

        // 应该只收到两个事件
        let event1 = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event1.key, "user:123");

        let event2 = timeout(Duration::from_millis(100), rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event2.key, "user:456");

        // 第三个事件应该超时
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_watch_all() {
        let storage = WatchableStorage::new(MemoryStorage::new());
        let mut rx = storage.watch_all().await;

        // 写入多个不同的键
        storage.write("key1", b"data1").await.unwrap();
        storage.write("user:123", b"data2").await.unwrap();
        storage.write("order:456", b"data3").await.unwrap();

        // 应该收到所有三个事件
        for expected_key in ["key1", "user:123", "order:456"] {
            let event = timeout(Duration::from_millis(100), rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(event.key, expected_key);
        }
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        let (id, mut rx) = storage.subscribe_with_id(WatchPattern::All).await;
        assert_eq!(storage.subscription_count().await, 1);

        // 取消订阅
        let removed = storage.unsubscribe(id).await;
        assert!(removed);
        assert_eq!(storage.subscription_count().await, 0);

        // 再次取消应该返回 false
        let removed_again = storage.unsubscribe(id).await;
        assert!(!removed_again);

        // 写入后不应再收到事件
        storage.write("key1", b"data").await.unwrap();
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err() || result.unwrap().is_err());
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        let mut rx1 = storage.watch_key("key1").await;
        let mut rx2 = storage.watch_key("key1").await;
        let mut rx3 = storage.watch_all().await;

        storage.write("key1", b"data").await.unwrap();

        // 所有订阅者都应该收到事件
        for rx in [&mut rx1, &mut rx2, &mut rx3] {
            let event = timeout(Duration::from_millis(100), rx.recv())
                .await
                .unwrap()
                .unwrap();
            assert_eq!(event.key, "key1");
        }
    }

    #[tokio::test]
    async fn test_no_event_on_nonexistent_delete() {
        let storage = WatchableStorage::new(MemoryStorage::new());
        let mut rx = storage.watch_all().await;

        // 删除不存在的键
        storage.delete("nonexistent").await.unwrap();

        // 不应该收到事件
        let result = timeout(Duration::from_millis(50), rx.recv()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_watch_pattern_matches() {
        assert!(WatchPattern::ExactKey("key1".to_string()).matches("key1"));
        assert!(!WatchPattern::ExactKey("key1".to_string()).matches("key2"));

        assert!(WatchPattern::Prefix("user:".to_string()).matches("user:123"));
        assert!(WatchPattern::Prefix("user:".to_string()).matches("user:"));
        assert!(!WatchPattern::Prefix("user:".to_string()).matches("order:123"));

        assert!(WatchPattern::All.matches("anything"));
        assert!(WatchPattern::All.matches(""));
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        // 创建订阅
        let (id1, _rx1) = storage.subscribe_with_id(WatchPattern::All).await;
        let (_id2, _rx2) = storage.subscribe_with_id(WatchPattern::Prefix("user:".to_string())).await;
        let (_id3, _rx3) = storage.subscribe_with_id(WatchPattern::ExactKey("key1".to_string())).await;

        let stats = storage.stats_snapshot();
        assert_eq!(stats.active_subscriptions, 3);
        assert_eq!(stats.total_subscriptions, 3);

        // 取消一个订阅
        storage.unsubscribe(id1).await;

        let stats = storage.stats_snapshot();
        assert_eq!(stats.active_subscriptions, 2);
        assert_eq!(stats.unsubscriptions, 1);
    }

    #[tokio::test]
    async fn test_event_type_stats() {
        let storage = WatchableStorage::new(MemoryStorage::new());
        let _rx = storage.watch_all().await;

        // 创建事件
        storage.write("key1", b"data").await.unwrap();
        let stats = storage.stats_snapshot();
        assert_eq!(stats.created_events, 1);

        // 更新事件
        storage.write("key1", b"new_data").await.unwrap();
        let stats = storage.stats_snapshot();
        assert_eq!(stats.updated_events, 1);

        // 删除事件
        storage.delete("key1").await.unwrap();
        let stats = storage.stats_snapshot();
        assert_eq!(stats.deleted_events, 1);
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = WatchableStorage::new(MemoryStorage::new());

        storage.subscribe(WatchPattern::ExactKey("k1".to_string())).await;
        storage.subscribe(WatchPattern::ExactKey("k2".to_string())).await;
        storage.subscribe(WatchPattern::Prefix("user:".to_string())).await;
        storage.subscribe(WatchPattern::All).await;

        let detailed = storage.detailed_stats().await;
        assert_eq!(detailed.subscriptions_by_pattern.exact_key, 2);
        assert_eq!(detailed.subscriptions_by_pattern.prefix, 1);
        assert_eq!(detailed.subscriptions_by_pattern.all, 1);
    }

    #[tokio::test]
    async fn test_builder() {
        let storage = WatchableStorageBuilder::new(MemoryStorage::new())
            .channel_capacity(128)
            .log_send_failures(true)
            .build();

        assert_eq!(storage.config.channel_capacity, 128);
        assert!(storage.config.log_send_failures);
    }

    #[tokio::test]
    async fn test_storage_event_helpers() {
        let created = StorageEvent::created("key1", 100);
        assert_eq!(created.event_type, EventType::Created);
        assert_eq!(created.key, "key1");
        assert_eq!(created.size, 100);

        let updated = StorageEvent::updated("key2", 200);
        assert_eq!(updated.event_type, EventType::Updated);

        let deleted = StorageEvent::deleted("key3");
        assert_eq!(deleted.event_type, EventType::Deleted);
        assert_eq!(deleted.size, 0);
    }

    #[tokio::test]
    async fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::Created), "Created");
        assert_eq!(format!("{}", EventType::Updated), "Updated");
        assert_eq!(format!("{}", EventType::Deleted), "Deleted");
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = WatchableStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_concurrent_writes_and_watches() {
        let storage = Arc::new(WatchableStorage::new(MemoryStorage::new()));
        let mut rx = storage.watch_all().await;

        let storage_clone = Arc::clone(&storage);
        let write_handle = tokio::spawn(async move {
            for i in 0..10 {
                storage_clone.write(&format!("key{}", i), b"data").await.unwrap();
            }
        });

        let read_handle = tokio::spawn(async move {
            let mut count = 0;
            while count < 10 {
                if let Ok(Ok(_)) = timeout(Duration::from_millis(500), rx.recv()).await {
                    count += 1;
                }
            }
            count
        });

        write_handle.await.unwrap();
        let received = read_handle.await.unwrap();
        assert_eq!(received, 10);
    }
}
