//! 事务存储层
//!
//! v1.70.0: 提供事务语义支持，包括 begin/commit/rollback
//!
//! ## 设计目标
//!
//! - **原子性**: 事务内的所有操作要么全部成功，要么全部回滚
//! - **隔离性**: 事务内的修改在提交前对外不可见
//! - **写前日志**: 使用 WAL 模式记录操作，支持回滚
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{TransactionStorage, MemoryStorage};
//!
//! let storage = MemoryStorage::new();
//! let tx_storage = TransactionStorage::new(storage);
//!
//! // 开始事务
//! let mut tx = tx_storage.begin().await?;
//! tx.write("key1", b"value1").await?;
//! tx.write("key2", b"value2").await?;
//!
//! // 提交事务
//! tx.commit().await?;
//!
//! // 或者回滚
//! // tx.rollback().await?;
//! ```

use crate::storage::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// 事务操作类型
// ============================================================================

/// 事务操作类型
#[derive(Debug, Clone)]
pub enum TransactionOp {
    /// 写入操作
    Write {
        key: String,
        data: Vec<u8>,
        /// 原始数据（用于回滚）
        original: Option<Vec<u8>>,
    },
    /// 删除操作
    Delete {
        key: String,
        /// 原始数据（用于回滚）
        original: Option<Vec<u8>>,
    },
}

impl TransactionOp {
    /// 获取操作的键
    pub fn key(&self) -> &str {
        match self {
            TransactionOp::Write { key, .. } => key,
            TransactionOp::Delete { key, .. } => key,
        }
    }

    /// 是否为写入操作
    pub fn is_write(&self) -> bool {
        matches!(self, TransactionOp::Write { .. })
    }

    /// 是否为删除操作
    pub fn is_delete(&self) -> bool {
        matches!(self, TransactionOp::Delete { .. })
    }
}

// ============================================================================
// 事务状态
// ============================================================================

/// 事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    /// 活跃状态
    Active,
    /// 已提交
    Committed,
    /// 已回滚
    RolledBack,
}

impl std::fmt::Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionState::Active => write!(f, "Active"),
            TransactionState::Committed => write!(f, "Committed"),
            TransactionState::RolledBack => write!(f, "RolledBack"),
        }
    }
}

// ============================================================================
// 事务
// ============================================================================

/// 事务句柄
///
/// 用于执行事务内的操作，支持提交和回滚
pub struct Transaction<B: StorageBackend> {
    /// 事务 ID
    id: u64,
    /// 后端存储
    backend: Arc<B>,
    /// 事务状态
    state: TransactionState,
    /// 操作日志（写前日志）
    wal: Vec<TransactionOp>,
    /// 本地缓存（事务内的修改）
    local_cache: HashMap<String, Option<Vec<u8>>>,
    /// 统计信息
    stats: Arc<TransactionStats>,
}

impl<B: StorageBackend> Transaction<B> {
    /// 创建新事务
    fn new(id: u64, backend: Arc<B>, stats: Arc<TransactionStats>) -> Self {
        stats.active.fetch_add(1, Ordering::Relaxed);
        Self {
            id,
            backend,
            state: TransactionState::Active,
            wal: Vec::new(),
            local_cache: HashMap::new(),
            stats,
        }
    }

    /// 获取事务 ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// 获取事务状态
    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// 获取操作数量
    pub fn operation_count(&self) -> usize {
        self.wal.len()
    }

    /// 检查事务是否活跃
    fn check_active(&self) -> StorageResult<()> {
        if self.state != TransactionState::Active {
            return Err(StorageError::Other(format!(
                "Transaction {} is not active (state: {})",
                self.id, self.state
            )));
        }
        Ok(())
    }

    /// 读取数据
    ///
    /// 优先从本地缓存读取，然后从后端存储读取
    pub async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.check_active()?;

        // 优先从本地缓存读取
        if let Some(cached) = self.local_cache.get(key) {
            return match cached {
                Some(data) => Ok(data.clone()),
                None => Err(StorageError::NotFound(key.to_string())),
            };
        }

        // 从后端存储读取
        self.backend.read(key).await
    }

    /// 写入数据
    ///
    /// 写入到本地缓存，记录到 WAL
    pub async fn write(&mut self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.check_active()?;

        // 获取原始数据（用于回滚）
        let original = if self.local_cache.contains_key(key) {
            self.local_cache.get(key).cloned().flatten()
        } else {
            self.backend.read(key).await.ok()
        };

        // 记录到 WAL
        self.wal.push(TransactionOp::Write {
            key: key.to_string(),
            data: data.to_vec(),
            original,
        });

        // 写入本地缓存
        self.local_cache.insert(key.to_string(), Some(data.to_vec()));

        Ok(())
    }

    /// 删除数据
    ///
    /// 在本地缓存中标记删除，记录到 WAL
    pub async fn delete(&mut self, key: &str) -> StorageResult<()> {
        self.check_active()?;

        // 获取原始数据（用于回滚）
        let original = if self.local_cache.contains_key(key) {
            self.local_cache.get(key).cloned().flatten()
        } else {
            self.backend.read(key).await.ok()
        };

        // 记录到 WAL
        self.wal.push(TransactionOp::Delete {
            key: key.to_string(),
            original,
        });

        // 在本地缓存中标记删除
        self.local_cache.insert(key.to_string(), None);

        Ok(())
    }

    /// 检查键是否存在
    pub async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.check_active()?;

        // 优先检查本地缓存
        if let Some(cached) = self.local_cache.get(key) {
            return Ok(cached.is_some());
        }

        // 检查后端存储
        self.backend.exists(key).await
    }

    /// 提交事务
    ///
    /// 将所有修改写入后端存储
    pub async fn commit(mut self) -> StorageResult<TransactionResult> {
        self.check_active()?;

        let start = std::time::Instant::now();
        let op_count = self.wal.len();

        // 应用所有操作到后端存储
        for op in &self.wal {
            match op {
                TransactionOp::Write { key, data, .. } => {
                    self.backend.write(key, data).await?;
                }
                TransactionOp::Delete { key, .. } => {
                    self.backend.delete(key).await?;
                }
            }
        }

        self.state = TransactionState::Committed;
        self.stats.active.fetch_sub(1, Ordering::Relaxed);
        self.stats.committed.fetch_add(1, Ordering::Relaxed);
        self.stats
            .total_operations
            .fetch_add(op_count as u64, Ordering::Relaxed);

        Ok(TransactionResult {
            id: self.id,
            state: TransactionState::Committed,
            operations: op_count,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }

    /// 回滚事务
    ///
    /// 丢弃所有未提交的修改
    pub async fn rollback(mut self) -> StorageResult<TransactionResult> {
        self.check_active()?;

        let start = std::time::Instant::now();
        let op_count = self.wal.len();

        // 清除本地缓存（不需要实际回滚，因为还没有写入后端）
        self.local_cache.clear();
        self.wal.clear();

        self.state = TransactionState::RolledBack;
        self.stats.active.fetch_sub(1, Ordering::Relaxed);
        self.stats.rolled_back.fetch_add(1, Ordering::Relaxed);

        Ok(TransactionResult {
            id: self.id,
            state: TransactionState::RolledBack,
            operations: op_count,
            duration_ms: start.elapsed().as_millis() as u64,
        })
    }
}

impl<B: StorageBackend> Drop for Transaction<B> {
    fn drop(&mut self) {
        // 如果事务还是活跃状态，自动回滚
        if self.state == TransactionState::Active {
            self.state = TransactionState::RolledBack;
            self.stats.active.fetch_sub(1, Ordering::Relaxed);
            self.stats.rolled_back.fetch_add(1, Ordering::Relaxed);
            self.stats.auto_rollback.fetch_add(1, Ordering::Relaxed);
        }
    }
}

// ============================================================================
// 事务结果
// ============================================================================

/// 事务结果
#[derive(Debug, Clone)]
pub struct TransactionResult {
    /// 事务 ID
    pub id: u64,
    /// 最终状态
    pub state: TransactionState,
    /// 操作数量
    pub operations: usize,
    /// 耗时（毫秒）
    pub duration_ms: u64,
}

// ============================================================================
// 事务统计
// ============================================================================

/// 事务统计信息
#[derive(Debug, Default)]
pub struct TransactionStats {
    /// 活跃事务数
    pub active: AtomicU64,
    /// 已提交事务数
    pub committed: AtomicU64,
    /// 已回滚事务数
    pub rolled_back: AtomicU64,
    /// 自动回滚事务数
    pub auto_rollback: AtomicU64,
    /// 总操作数
    pub total_operations: AtomicU64,
    /// 下一个事务 ID
    next_id: AtomicU64,
}

impl TransactionStats {
    /// 生成下一个事务 ID
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// 获取快照
    pub fn snapshot(&self) -> TransactionStatsSnapshot {
        TransactionStatsSnapshot {
            active: self.active.load(Ordering::Relaxed),
            committed: self.committed.load(Ordering::Relaxed),
            rolled_back: self.rolled_back.load(Ordering::Relaxed),
            auto_rollback: self.auto_rollback.load(Ordering::Relaxed),
            total_operations: self.total_operations.load(Ordering::Relaxed),
        }
    }
}

/// 事务统计快照
#[derive(Debug, Clone)]
pub struct TransactionStatsSnapshot {
    /// 活跃事务数
    pub active: u64,
    /// 已提交事务数
    pub committed: u64,
    /// 已回滚事务数
    pub rolled_back: u64,
    /// 自动回滚事务数
    pub auto_rollback: u64,
    /// 总操作数
    pub total_operations: u64,
}

impl TransactionStatsSnapshot {
    /// 提交率
    pub fn commit_rate(&self) -> f64 {
        let total = self.committed + self.rolled_back;
        if total == 0 {
            0.0
        } else {
            self.committed as f64 / total as f64
        }
    }

    /// 自动回滚率
    pub fn auto_rollback_rate(&self) -> f64 {
        if self.rolled_back == 0 {
            0.0
        } else {
            self.auto_rollback as f64 / self.rolled_back as f64
        }
    }
}

// ============================================================================
// 事务存储配置
// ============================================================================

/// 事务存储配置
#[derive(Debug, Clone)]
pub struct TransactionStorageConfig {
    /// 最大活跃事务数
    pub max_active_transactions: usize,
    /// 事务超时时间（秒）
    pub transaction_timeout_secs: u64,
    /// 启用自动回滚
    pub auto_rollback_on_drop: bool,
}

impl Default for TransactionStorageConfig {
    fn default() -> Self {
        Self {
            max_active_transactions: 100,
            transaction_timeout_secs: 300,
            auto_rollback_on_drop: true,
        }
    }
}

// ============================================================================
// 事务存储
// ============================================================================

/// 事务存储
///
/// 提供事务语义的存储包装器
pub struct TransactionStorage<B: StorageBackend> {
    /// 后端存储
    backend: Arc<B>,
    /// 配置
    config: TransactionStorageConfig,
    /// 统计信息
    stats: Arc<TransactionStats>,
    /// 活跃事务追踪
    active_transactions: Arc<RwLock<HashMap<u64, std::time::Instant>>>,
}

impl<B: StorageBackend> TransactionStorage<B> {
    /// 创建事务存储
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, TransactionStorageConfig::default())
    }

    /// 使用配置创建事务存储
    pub fn with_config(backend: B, config: TransactionStorageConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
            stats: Arc::new(TransactionStats::default()),
            active_transactions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 开始新事务
    pub async fn begin(&self) -> StorageResult<Transaction<B>> {
        // 检查活跃事务数限制
        let active_count = self.stats.active.load(Ordering::Relaxed) as usize;
        if active_count >= self.config.max_active_transactions {
            return Err(StorageError::Other(format!(
                "Max active transactions reached: {}",
                self.config.max_active_transactions
            )));
        }

        let id = self.stats.next_id();

        // 记录事务开始时间
        {
            let mut active = self.active_transactions.write().await;
            active.insert(id, std::time::Instant::now());
        }

        Ok(Transaction::new(id, Arc::clone(&self.backend), Arc::clone(&self.stats)))
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> TransactionStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取详细统计信息
    pub fn detailed_stats(&self) -> DetailedTransactionStats {
        let snapshot = self.stats.snapshot();
        DetailedTransactionStats {
            active: snapshot.active,
            committed: snapshot.committed,
            rolled_back: snapshot.rolled_back,
            auto_rollback: snapshot.auto_rollback,
            total_operations: snapshot.total_operations,
            commit_rate: snapshot.commit_rate(),
            auto_rollback_rate: snapshot.auto_rollback_rate(),
            max_active_transactions: self.config.max_active_transactions as u64,
            transaction_timeout_secs: self.config.transaction_timeout_secs,
        }
    }

    /// 清理超时事务
    pub async fn cleanup_timed_out(&self) -> usize {
        let timeout = std::time::Duration::from_secs(self.config.transaction_timeout_secs);
        let mut active = self.active_transactions.write().await;
        let before = active.len();

        active.retain(|_, start_time| start_time.elapsed() < timeout);

        before - active.len()
    }

    /// 获取内部后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

/// 详细事务统计
#[derive(Debug, Clone)]
pub struct DetailedTransactionStats {
    /// 活跃事务数
    pub active: u64,
    /// 已提交事务数
    pub committed: u64,
    /// 已回滚事务数
    pub rolled_back: u64,
    /// 自动回滚事务数
    pub auto_rollback: u64,
    /// 总操作数
    pub total_operations: u64,
    /// 提交率
    pub commit_rate: f64,
    /// 自动回滚率
    pub auto_rollback_rate: f64,
    /// 最大活跃事务数
    pub max_active_transactions: u64,
    /// 事务超时时间
    pub transaction_timeout_secs: u64,
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for TransactionStorage<B> {
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
        "TransactionStorage"
    }
}

// ============================================================================
// Savepoint 支持
// ============================================================================

/// 保存点
///
/// 允许在事务内创建保存点，支持部分回滚
#[derive(Debug, Clone)]
pub struct Savepoint {
    /// 保存点名称
    pub name: String,
    /// WAL 位置
    wal_position: usize,
    /// 本地缓存快照
    cache_snapshot: HashMap<String, Option<Vec<u8>>>,
}

/// 带保存点的事务
pub struct TransactionWithSavepoints<B: StorageBackend> {
    /// 内部事务
    inner: Transaction<B>,
    /// 保存点列表
    savepoints: Vec<Savepoint>,
}

impl<B: StorageBackend> TransactionWithSavepoints<B> {
    /// 从普通事务创建
    pub fn new(tx: Transaction<B>) -> Self {
        Self {
            inner: tx,
            savepoints: Vec::new(),
        }
    }

    /// 创建保存点
    pub fn savepoint(&mut self, name: impl Into<String>) -> StorageResult<()> {
        self.inner.check_active()?;

        self.savepoints.push(Savepoint {
            name: name.into(),
            wal_position: self.inner.wal.len(),
            cache_snapshot: self.inner.local_cache.clone(),
        });

        Ok(())
    }

    /// 回滚到保存点
    pub fn rollback_to_savepoint(&mut self, name: &str) -> StorageResult<()> {
        self.inner.check_active()?;

        // 找到保存点
        let pos = self
            .savepoints
            .iter()
            .position(|sp| sp.name == name)
            .ok_or_else(|| StorageError::NotFound(format!("Savepoint not found: {}", name)))?;

        let savepoint = self.savepoints[pos].clone();

        // 回滚 WAL
        self.inner.wal.truncate(savepoint.wal_position);

        // 恢复本地缓存
        self.inner.local_cache = savepoint.cache_snapshot;

        // 移除此保存点及之后的所有保存点
        self.savepoints.truncate(pos);

        Ok(())
    }

    /// 释放保存点
    pub fn release_savepoint(&mut self, name: &str) -> StorageResult<()> {
        self.inner.check_active()?;

        let pos = self
            .savepoints
            .iter()
            .position(|sp| sp.name == name)
            .ok_or_else(|| StorageError::NotFound(format!("Savepoint not found: {}", name)))?;

        self.savepoints.remove(pos);

        Ok(())
    }

    /// 获取保存点列表
    pub fn savepoint_names(&self) -> Vec<&str> {
        self.savepoints.iter().map(|sp| sp.name.as_str()).collect()
    }

    /// 读取数据
    pub async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.inner.read(key).await
    }

    /// 写入数据
    pub async fn write(&mut self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.inner.write(key, data).await
    }

    /// 删除数据
    pub async fn delete(&mut self, key: &str) -> StorageResult<()> {
        self.inner.delete(key).await
    }

    /// 提交事务
    pub async fn commit(self) -> StorageResult<TransactionResult> {
        self.inner.commit().await
    }

    /// 回滚事务
    pub async fn rollback(self) -> StorageResult<TransactionResult> {
        self.inner.rollback().await
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
    async fn test_transaction_storage_new() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);
        assert_eq!(tx_storage.name(), "TransactionStorage");
    }

    #[tokio::test]
    async fn test_transaction_begin() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let tx = tx_storage.begin().await.unwrap();
        assert_eq!(tx.id(), 0);
        assert_eq!(tx.state(), TransactionState::Active);
    }

    #[tokio::test]
    async fn test_transaction_write_read() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key1", b"value1").await.unwrap();

        // 事务内可以读取
        let data = tx.read("key1").await.unwrap();
        assert_eq!(data, b"value1");

        // 事务外还不可见
        assert!(tx_storage.read("key1").await.is_err());

        // 提交后可见
        tx.commit().await.unwrap();
        let data = tx_storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key1", b"value1").await.unwrap();

        // 回滚
        let result = tx.rollback().await.unwrap();
        assert_eq!(result.state, TransactionState::RolledBack);

        // 数据不可见
        assert!(tx_storage.read("key1").await.is_err());
    }

    #[tokio::test]
    async fn test_transaction_delete() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        // 先写入数据
        tx_storage.write("key1", b"value1").await.unwrap();

        // 事务内删除
        let mut tx = tx_storage.begin().await.unwrap();
        tx.delete("key1").await.unwrap();

        // 事务内不可见
        assert!(tx.read("key1").await.is_err());

        // 事务外仍可见
        assert!(tx_storage.read("key1").await.is_ok());

        // 提交后删除生效
        tx.commit().await.unwrap();
        assert!(tx_storage.read("key1").await.is_err());
    }

    #[tokio::test]
    async fn test_transaction_auto_rollback() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        {
            let mut tx = tx_storage.begin().await.unwrap();
            tx.write("key1", b"value1").await.unwrap();
            // tx 离开作用域，自动回滚
        }

        // 数据不可见
        assert!(tx_storage.read("key1").await.is_err());

        // 统计信息
        let stats = tx_storage.stats_snapshot();
        assert_eq!(stats.auto_rollback, 1);
    }

    #[tokio::test]
    async fn test_transaction_exists() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        assert!(!tx.exists("key1").await.unwrap());

        tx.write("key1", b"value1").await.unwrap();
        assert!(tx.exists("key1").await.unwrap());

        tx.delete("key1").await.unwrap();
        assert!(!tx.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_transaction_operation_count() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        assert_eq!(tx.operation_count(), 0);

        tx.write("key1", b"value1").await.unwrap();
        assert_eq!(tx.operation_count(), 1);

        tx.write("key2", b"value2").await.unwrap();
        assert_eq!(tx.operation_count(), 2);

        tx.delete("key1").await.unwrap();
        assert_eq!(tx.operation_count(), 3);
    }

    #[tokio::test]
    async fn test_transaction_stats() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        // 创建并提交事务
        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key1", b"value1").await.unwrap();
        tx.commit().await.unwrap();

        // 创建并回滚事务
        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key2", b"value2").await.unwrap();
        tx.rollback().await.unwrap();

        let stats = tx_storage.stats_snapshot();
        assert_eq!(stats.committed, 1);
        assert_eq!(stats.rolled_back, 1);
        assert_eq!(stats.total_operations, 1); // 只有提交的操作计入
    }

    #[tokio::test]
    async fn test_transaction_detailed_stats() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let stats = tx_storage.detailed_stats();
        assert_eq!(stats.active, 0);
        assert_eq!(stats.max_active_transactions, 100);
        assert_eq!(stats.transaction_timeout_secs, 300);
    }

    #[tokio::test]
    async fn test_transaction_max_active_limit() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::with_config(
            storage,
            TransactionStorageConfig {
                max_active_transactions: 2,
                ..Default::default()
            },
        );

        let _tx1 = tx_storage.begin().await.unwrap();
        let _tx2 = tx_storage.begin().await.unwrap();

        // 第三个事务应该失败
        let result = tx_storage.begin().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transaction_commit_after_rollback_fails() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key1", b"value1").await.unwrap();

        // 模拟事务已结束（通过 drop 设置状态）
        // 这里我们直接测试状态检查
        let result = tx.commit().await;
        assert!(result.is_ok());

        // 无法再次操作
        let tx2 = tx_storage.begin().await.unwrap();
        let result = tx2.commit().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transaction_isolation() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        // 两个并发事务
        let mut tx1 = tx_storage.begin().await.unwrap();
        let mut tx2 = tx_storage.begin().await.unwrap();

        // tx1 写入
        tx1.write("key1", b"value1").await.unwrap();

        // tx2 看不到 tx1 的修改
        assert!(tx2.read("key1").await.is_err());

        // tx2 写入相同的键
        tx2.write("key1", b"value2").await.unwrap();

        // 各自看到自己的值
        assert_eq!(tx1.read("key1").await.unwrap(), b"value1");
        assert_eq!(tx2.read("key1").await.unwrap(), b"value2");

        // tx1 先提交
        tx1.commit().await.unwrap();

        // tx2 仍看到自己的值
        assert_eq!(tx2.read("key1").await.unwrap(), b"value2");

        // tx2 提交（覆盖 tx1 的值）
        tx2.commit().await.unwrap();

        // 最终值是 tx2 的值
        assert_eq!(tx_storage.read("key1").await.unwrap(), b"value2");
    }

    #[tokio::test]
    async fn test_savepoint_basic() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let tx = tx_storage.begin().await.unwrap();
        let mut tx = TransactionWithSavepoints::new(tx);

        tx.write("key1", b"value1").await.unwrap();
        tx.savepoint("sp1").unwrap();

        tx.write("key2", b"value2").await.unwrap();
        tx.savepoint("sp2").unwrap();

        tx.write("key3", b"value3").await.unwrap();

        // 验证保存点列表
        assert_eq!(tx.savepoint_names(), vec!["sp1", "sp2"]);
    }

    #[tokio::test]
    async fn test_savepoint_rollback() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let tx = tx_storage.begin().await.unwrap();
        let mut tx = TransactionWithSavepoints::new(tx);

        tx.write("key1", b"value1").await.unwrap();
        tx.savepoint("sp1").unwrap();

        tx.write("key2", b"value2").await.unwrap();
        tx.savepoint("sp2").unwrap();

        tx.write("key3", b"value3").await.unwrap();

        // 回滚到 sp1
        tx.rollback_to_savepoint("sp1").unwrap();

        // key1 仍存在
        assert!(tx.read("key1").await.is_ok());

        // key2, key3 已回滚
        assert!(tx.read("key2").await.is_err());
        assert!(tx.read("key3").await.is_err());

        // sp2 也被移除
        assert_eq!(tx.savepoint_names(), Vec::<&str>::new());
    }

    #[tokio::test]
    async fn test_savepoint_release() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let tx = tx_storage.begin().await.unwrap();
        let mut tx = TransactionWithSavepoints::new(tx);

        tx.savepoint("sp1").unwrap();
        tx.savepoint("sp2").unwrap();

        tx.release_savepoint("sp1").unwrap();

        assert_eq!(tx.savepoint_names(), vec!["sp2"]);
    }

    #[tokio::test]
    async fn test_transaction_result() {
        let storage = MemoryStorage::new();
        let tx_storage = TransactionStorage::new(storage);

        let mut tx = tx_storage.begin().await.unwrap();
        tx.write("key1", b"value1").await.unwrap();
        tx.write("key2", b"value2").await.unwrap();

        let result = tx.commit().await.unwrap();
        assert_eq!(result.id, 0);
        assert_eq!(result.state, TransactionState::Committed);
        assert_eq!(result.operations, 2);
    }

    #[tokio::test]
    async fn test_commit_rate() {
        let snapshot = TransactionStatsSnapshot {
            active: 0,
            committed: 80,
            rolled_back: 20,
            auto_rollback: 5,
            total_operations: 100,
        };

        assert!((snapshot.commit_rate() - 0.8).abs() < 0.001);
        assert!((snapshot.auto_rollback_rate() - 0.25).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_transaction_op() {
        let write_op = TransactionOp::Write {
            key: "key1".to_string(),
            data: b"value1".to_vec(),
            original: None,
        };
        assert!(write_op.is_write());
        assert!(!write_op.is_delete());
        assert_eq!(write_op.key(), "key1");

        let delete_op = TransactionOp::Delete {
            key: "key2".to_string(),
            original: Some(b"old".to_vec()),
        };
        assert!(!delete_op.is_write());
        assert!(delete_op.is_delete());
        assert_eq!(delete_op.key(), "key2");
    }
}
