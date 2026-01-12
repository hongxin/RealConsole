//! 复制存储层
//!
//! v1.72.0: 提供多后端数据复制，支持高可用和故障转移
//!
//! ## 设计目标
//!
//! - **数据冗余**: 写入数据自动复制到多个后端
//! - **故障转移**: 主后端失败时自动切换到副本
//! - **一致性级别**: 支持不同的写入一致性要求
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{ReplicatedStorage, MemoryStorage, ConsistencyLevel};
//!
//! let primary = MemoryStorage::new();
//! let replica1 = MemoryStorage::new();
//! let replica2 = MemoryStorage::new();
//!
//! let replicated = ReplicatedStorage::new(primary)
//!     .with_replica(replica1)
//!     .with_replica(replica2)
//!     .with_consistency(ConsistencyLevel::Quorum);
//!
//! // 写入自动复制到所有后端
//! replicated.write("key1", b"value1").await?;
//!
//! // 读取优先从主后端，失败则从副本读取
//! let data = replicated.read("key1").await?;
//! ```

use crate::storage::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// 一致性级别
// ============================================================================

/// 写入一致性级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ConsistencyLevel {
    /// 只写入主后端
    One,
    /// 写入主后端和至少一个副本
    Two,
    /// 写入大多数后端（包括主后端）
    Quorum,
    /// 写入所有后端
    #[default]
    All,
}

impl ConsistencyLevel {
    /// 计算需要成功的后端数量
    pub fn required_successes(&self, total: usize) -> usize {
        match self {
            ConsistencyLevel::One => 1,
            ConsistencyLevel::Two => 2.min(total),
            ConsistencyLevel::Quorum => (total / 2) + 1,
            ConsistencyLevel::All => total,
        }
    }
}


// ============================================================================
// 读取策略
// ============================================================================

/// 读取策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum ReadStrategy {
    /// 只从主后端读取
    PrimaryOnly,
    /// 优先主后端，失败则从副本读取
    #[default]
    PrimaryWithFallback,
    /// 从任意可用后端读取（负载均衡）
    Any,
}


// ============================================================================
// 后端状态
// ============================================================================

/// 后端状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendStatus {
    /// 健康
    Healthy,
    /// 降级（部分失败）
    Degraded,
    /// 不可用
    Unavailable,
}

impl std::fmt::Display for BackendStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackendStatus::Healthy => write!(f, "Healthy"),
            BackendStatus::Degraded => write!(f, "Degraded"),
            BackendStatus::Unavailable => write!(f, "Unavailable"),
        }
    }
}

// ============================================================================
// 后端包装器
// ============================================================================

/// 后端包装器（带状态追踪）
struct BackendWrapper<B: StorageBackend> {
    /// 后端实例
    backend: Arc<B>,
    /// 后端名称
    name: String,
    /// 是否为主后端
    is_primary: bool,
    /// 连续失败次数
    consecutive_failures: AtomicU64,
    /// 总成功次数
    total_successes: AtomicU64,
    /// 总失败次数
    total_failures: AtomicU64,
}

impl<B: StorageBackend> BackendWrapper<B> {
    fn new(backend: B, name: impl Into<String>, is_primary: bool) -> Self {
        Self {
            backend: Arc::new(backend),
            name: name.into(),
            is_primary,
            consecutive_failures: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
        }
    }

    fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::Relaxed);
        self.total_successes.fetch_add(1, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.consecutive_failures.fetch_add(1, Ordering::Relaxed);
        self.total_failures.fetch_add(1, Ordering::Relaxed);
    }

    fn status(&self) -> BackendStatus {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures == 0 {
            BackendStatus::Healthy
        } else if failures < 3 {
            BackendStatus::Degraded
        } else {
            BackendStatus::Unavailable
        }
    }

    fn is_available(&self) -> bool {
        self.status() != BackendStatus::Unavailable
    }
}

// ============================================================================
// 复制统计
// ============================================================================

/// 复制统计信息
#[derive(Debug, Default)]
pub struct ReplicationStats {
    /// 写入次数
    pub writes: AtomicU64,
    /// 成功复制次数
    pub successful_replications: AtomicU64,
    /// 部分复制次数（未达到一致性级别但仍成功）
    pub partial_replications: AtomicU64,
    /// 失败的复制次数
    pub failed_replications: AtomicU64,
    /// 读取次数
    pub reads: AtomicU64,
    /// 故障转移次数
    pub failovers: AtomicU64,
}

impl ReplicationStats {
    /// 获取快照
    pub fn snapshot(&self) -> ReplicationStatsSnapshot {
        ReplicationStatsSnapshot {
            writes: self.writes.load(Ordering::Relaxed),
            successful_replications: self.successful_replications.load(Ordering::Relaxed),
            partial_replications: self.partial_replications.load(Ordering::Relaxed),
            failed_replications: self.failed_replications.load(Ordering::Relaxed),
            reads: self.reads.load(Ordering::Relaxed),
            failovers: self.failovers.load(Ordering::Relaxed),
        }
    }
}

/// 复制统计快照
#[derive(Debug, Clone)]
pub struct ReplicationStatsSnapshot {
    /// 写入次数
    pub writes: u64,
    /// 成功复制次数
    pub successful_replications: u64,
    /// 部分复制次数
    pub partial_replications: u64,
    /// 失败的复制次数
    pub failed_replications: u64,
    /// 读取次数
    pub reads: u64,
    /// 故障转移次数
    pub failovers: u64,
}

impl ReplicationStatsSnapshot {
    /// 复制成功率
    pub fn replication_success_rate(&self) -> f64 {
        let total = self.successful_replications + self.partial_replications + self.failed_replications;
        if total == 0 {
            1.0
        } else {
            self.successful_replications as f64 / total as f64
        }
    }

    /// 故障转移率
    pub fn failover_rate(&self) -> f64 {
        if self.reads == 0 {
            0.0
        } else {
            self.failovers as f64 / self.reads as f64
        }
    }
}

// ============================================================================
// 复制存储配置
// ============================================================================

/// 复制存储配置
#[derive(Debug, Clone)]
pub struct ReplicatedStorageConfig {
    /// 写入一致性级别
    pub consistency_level: ConsistencyLevel,
    /// 读取策略
    pub read_strategy: ReadStrategy,
    /// 健康检查间隔（秒）
    pub health_check_interval_secs: u64,
    /// 最大连续失败次数（超过则标记为不可用）
    pub max_consecutive_failures: u64,
}

impl Default for ReplicatedStorageConfig {
    fn default() -> Self {
        Self {
            consistency_level: ConsistencyLevel::All,
            read_strategy: ReadStrategy::PrimaryWithFallback,
            health_check_interval_secs: 30,
            max_consecutive_failures: 3,
        }
    }
}

// ============================================================================
// 复制存储
// ============================================================================

/// 复制存储
///
/// 将数据复制到多个后端，支持故障转移
pub struct ReplicatedStorage<B: StorageBackend> {
    /// 后端列表（第一个为主后端）
    backends: Arc<RwLock<Vec<BackendWrapper<B>>>>,
    /// 配置
    config: ReplicatedStorageConfig,
    /// 统计信息
    stats: Arc<ReplicationStats>,
    /// 读取索引（用于轮询）
    read_index: AtomicU64,
}

impl<B: StorageBackend> ReplicatedStorage<B> {
    /// 创建复制存储（单个主后端）
    pub fn new(primary: B) -> Self {
        Self::with_config(primary, ReplicatedStorageConfig::default())
    }

    /// 使用配置创建复制存储
    pub fn with_config(primary: B, config: ReplicatedStorageConfig) -> Self {
        let wrapper = BackendWrapper::new(primary, "primary", true);
        Self {
            backends: Arc::new(RwLock::new(vec![wrapper])),
            config,
            stats: Arc::new(ReplicationStats::default()),
            read_index: AtomicU64::new(0),
        }
    }

    /// 添加副本后端
    pub async fn add_replica(&self, replica: B) {
        let mut backends = self.backends.write().await;
        let name = format!("replica-{}", backends.len());
        backends.push(BackendWrapper::new(replica, name, false));
    }

    /// 获取后端数量
    pub async fn backend_count(&self) -> usize {
        self.backends.read().await.len()
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> ReplicationStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取详细统计信息
    pub async fn detailed_stats(&self) -> DetailedReplicationStats {
        let snapshot = self.stats.snapshot();
        let backends = self.backends.read().await;

        let backend_statuses: Vec<_> = backends
            .iter()
            .map(|b| BackendStatusInfo {
                name: b.name.clone(),
                is_primary: b.is_primary,
                status: b.status(),
                consecutive_failures: b.consecutive_failures.load(Ordering::Relaxed),
                total_successes: b.total_successes.load(Ordering::Relaxed),
                total_failures: b.total_failures.load(Ordering::Relaxed),
            })
            .collect();

        DetailedReplicationStats {
            writes: snapshot.writes,
            successful_replications: snapshot.successful_replications,
            partial_replications: snapshot.partial_replications,
            failed_replications: snapshot.failed_replications,
            reads: snapshot.reads,
            failovers: snapshot.failovers,
            replication_success_rate: snapshot.replication_success_rate(),
            failover_rate: snapshot.failover_rate(),
            backend_count: backends.len(),
            healthy_backends: backends.iter().filter(|b| b.status() == BackendStatus::Healthy).count(),
            consistency_level: self.config.consistency_level,
            read_strategy: self.config.read_strategy,
            backend_statuses,
        }
    }

    /// 获取健康后端数量
    pub async fn healthy_backend_count(&self) -> usize {
        self.backends
            .read()
            .await
            .iter()
            .filter(|b| b.is_available())
            .count()
    }

    /// 执行写入复制
    async fn replicate_write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.stats.writes.fetch_add(1, Ordering::Relaxed);

        let backends = self.backends.read().await;
        let total = backends.len();
        let required = self.config.consistency_level.required_successes(total);

        let mut successes = 0;
        let mut last_error = None;

        for backend in backends.iter() {
            match backend.backend.write(key, data).await {
                Ok(()) => {
                    backend.record_success();
                    successes += 1;
                }
                Err(e) => {
                    backend.record_failure();
                    last_error = Some(e);
                }
            }
        }

        if successes >= required {
            if successes == total {
                self.stats.successful_replications.fetch_add(1, Ordering::Relaxed);
            } else {
                self.stats.partial_replications.fetch_add(1, Ordering::Relaxed);
            }
            Ok(())
        } else {
            self.stats.failed_replications.fetch_add(1, Ordering::Relaxed);
            Err(last_error.unwrap_or_else(|| {
                StorageError::Other(format!(
                    "Replication failed: only {}/{} backends succeeded, required {}",
                    successes, total, required
                ))
            }))
        }
    }

    /// 执行删除复制
    async fn replicate_delete(&self, key: &str) -> StorageResult<()> {
        let backends = self.backends.read().await;
        let total = backends.len();
        let required = self.config.consistency_level.required_successes(total);

        let mut successes = 0;
        let mut last_error = None;

        for backend in backends.iter() {
            match backend.backend.delete(key).await {
                Ok(()) => {
                    backend.record_success();
                    successes += 1;
                }
                Err(e) => {
                    backend.record_failure();
                    last_error = Some(e);
                }
            }
        }

        if successes >= required {
            Ok(())
        } else {
            Err(last_error.unwrap_or_else(|| {
                StorageError::Other(format!(
                    "Delete replication failed: only {}/{} backends succeeded",
                    successes, total
                ))
            }))
        }
    }

    /// 执行读取（带故障转移）
    async fn read_with_fallback(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.stats.reads.fetch_add(1, Ordering::Relaxed);

        let backends = self.backends.read().await;

        match self.config.read_strategy {
            ReadStrategy::PrimaryOnly => {
                // 只从主后端读取
                if let Some(primary) = backends.first() {
                    match primary.backend.read(key).await {
                        Ok(data) => {
                            primary.record_success();
                            Ok(data)
                        }
                        Err(e) => {
                            primary.record_failure();
                            Err(e)
                        }
                    }
                } else {
                    Err(StorageError::Other("No backends available".to_string()))
                }
            }
            ReadStrategy::PrimaryWithFallback => {
                // 优先主后端，失败则从副本读取
                let mut last_error = None;
                let mut tried_fallback = false;

                for (i, backend) in backends.iter().enumerate() {
                    if !backend.is_available() && i > 0 {
                        continue;
                    }

                    match backend.backend.read(key).await {
                        Ok(data) => {
                            backend.record_success();
                            if tried_fallback {
                                self.stats.failovers.fetch_add(1, Ordering::Relaxed);
                            }
                            return Ok(data);
                        }
                        Err(e) => {
                            backend.record_failure();
                            last_error = Some(e);
                            if i == 0 {
                                tried_fallback = true;
                            }
                        }
                    }
                }

                Err(last_error.unwrap_or_else(|| StorageError::NotFound(key.to_string())))
            }
            ReadStrategy::Any => {
                // 轮询读取
                let start = self.read_index.fetch_add(1, Ordering::Relaxed) as usize;
                let available: Vec<_> = backends.iter().filter(|b| b.is_available()).collect();

                if available.is_empty() {
                    return Err(StorageError::Other("No available backends".to_string()));
                }

                let idx = start % available.len();
                match available[idx].backend.read(key).await {
                    Ok(data) => {
                        available[idx].record_success();
                        Ok(data)
                    }
                    Err(e) => {
                        available[idx].record_failure();
                        // 尝试其他后端
                        for (i, backend) in available.iter().enumerate() {
                            if i == idx {
                                continue;
                            }
                            if let Ok(data) = backend.backend.read(key).await {
                                backend.record_success();
                                self.stats.failovers.fetch_add(1, Ordering::Relaxed);
                                return Ok(data);
                            } else {
                                backend.record_failure();
                            }
                        }
                        Err(e)
                    }
                }
            }
        }
    }
}

/// 后端状态信息
#[derive(Debug, Clone)]
pub struct BackendStatusInfo {
    /// 后端名称
    pub name: String,
    /// 是否为主后端
    pub is_primary: bool,
    /// 状态
    pub status: BackendStatus,
    /// 连续失败次数
    pub consecutive_failures: u64,
    /// 总成功次数
    pub total_successes: u64,
    /// 总失败次数
    pub total_failures: u64,
}

/// 详细复制统计
#[derive(Debug, Clone)]
pub struct DetailedReplicationStats {
    /// 写入次数
    pub writes: u64,
    /// 成功复制次数
    pub successful_replications: u64,
    /// 部分复制次数
    pub partial_replications: u64,
    /// 失败的复制次数
    pub failed_replications: u64,
    /// 读取次数
    pub reads: u64,
    /// 故障转移次数
    pub failovers: u64,
    /// 复制成功率
    pub replication_success_rate: f64,
    /// 故障转移率
    pub failover_rate: f64,
    /// 后端数量
    pub backend_count: usize,
    /// 健康后端数量
    pub healthy_backends: usize,
    /// 一致性级别
    pub consistency_level: ConsistencyLevel,
    /// 读取策略
    pub read_strategy: ReadStrategy,
    /// 各后端状态
    pub backend_statuses: Vec<BackendStatusInfo>,
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend + 'static> StorageBackend for ReplicatedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.read_with_fallback(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        self.replicate_write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.replicate_delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // 从主后端列出
        let backends = self.backends.read().await;
        if let Some(primary) = backends.first() {
            primary.backend.list(prefix).await
        } else {
            Ok(vec![])
        }
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // 从任意可用后端检查
        let backends = self.backends.read().await;
        for backend in backends.iter() {
            if backend.is_available() {
                if let Ok(exists) = backend.backend.exists(key).await {
                    return Ok(exists);
                }
            }
        }
        Ok(false)
    }

    fn stats(&self) -> StorageStats {
        // 返回主后端的统计信息
        // 注意：这是同步方法，无法获取锁
        StorageStats::default()
    }

    fn name(&self) -> &'static str {
        "ReplicatedStorage"
    }
}

// ============================================================================
// Builder 模式
// ============================================================================

/// 复制存储构建器
pub struct ReplicatedStorageBuilder<B: StorageBackend> {
    primary: B,
    replicas: Vec<B>,
    config: ReplicatedStorageConfig,
}

impl<B: StorageBackend> ReplicatedStorageBuilder<B> {
    /// 创建构建器
    pub fn new(primary: B) -> Self {
        Self {
            primary,
            replicas: Vec::new(),
            config: ReplicatedStorageConfig::default(),
        }
    }

    /// 添加副本
    pub fn with_replica(mut self, replica: B) -> Self {
        self.replicas.push(replica);
        self
    }

    /// 设置一致性级别
    pub fn with_consistency(mut self, level: ConsistencyLevel) -> Self {
        self.config.consistency_level = level;
        self
    }

    /// 设置读取策略
    pub fn with_read_strategy(mut self, strategy: ReadStrategy) -> Self {
        self.config.read_strategy = strategy;
        self
    }

    /// 构建复制存储
    pub async fn build(self) -> ReplicatedStorage<B> {
        let storage = ReplicatedStorage::with_config(self.primary, self.config);
        for replica in self.replicas {
            storage.add_replica(replica).await;
        }
        storage
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_consistency_level_required_successes() {
        assert_eq!(ConsistencyLevel::One.required_successes(3), 1);
        assert_eq!(ConsistencyLevel::Two.required_successes(3), 2);
        assert_eq!(ConsistencyLevel::Quorum.required_successes(3), 2);
        assert_eq!(ConsistencyLevel::Quorum.required_successes(5), 3);
        assert_eq!(ConsistencyLevel::All.required_successes(3), 3);
    }

    #[tokio::test]
    async fn test_replicated_storage_new() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);

        assert_eq!(storage.name(), "ReplicatedStorage");
        assert_eq!(storage.backend_count().await, 1);
    }

    #[tokio::test]
    async fn test_replicated_storage_add_replica() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);

        storage.add_replica(MemoryStorage::new()).await;
        storage.add_replica(MemoryStorage::new()).await;

        assert_eq!(storage.backend_count().await, 3);
    }

    #[tokio::test]
    async fn test_replicated_storage_write_read() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);
        storage.add_replica(MemoryStorage::new()).await;

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();

        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_replicated_storage_delete() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);

        storage.write("key1", b"value1").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());

        storage.delete("key1").await.unwrap();
        assert!(!storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_replicated_storage_list() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);

        storage.write("test:a", b"1").await.unwrap();
        storage.write("test:b", b"2").await.unwrap();
        storage.write("other:c", b"3").await.unwrap();

        let keys = storage.list("test:").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_replicated_storage_stats() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);

        storage.write("key1", b"value1").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.stats_snapshot();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.successful_replications, 1);
    }

    #[tokio::test]
    async fn test_replicated_storage_detailed_stats() {
        let primary = MemoryStorage::new();
        let storage = ReplicatedStorage::new(primary);
        storage.add_replica(MemoryStorage::new()).await;

        let stats = storage.detailed_stats().await;
        assert_eq!(stats.backend_count, 2);
        assert_eq!(stats.healthy_backends, 2);
    }

    #[tokio::test]
    async fn test_replicated_storage_builder() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_consistency(ConsistencyLevel::Quorum)
            .with_read_strategy(ReadStrategy::Any)
            .build()
            .await;

        assert_eq!(storage.backend_count().await, 3);
    }

    #[tokio::test]
    async fn test_replicated_storage_consistency_one() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_consistency(ConsistencyLevel::One)
            .build()
            .await;

        // 只需要一个后端成功
        storage.write("key1", b"value1").await.unwrap();
        let stats = storage.stats_snapshot();
        assert!(stats.successful_replications >= 1 || stats.partial_replications >= 1);
    }

    #[tokio::test]
    async fn test_replicated_storage_consistency_quorum() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_consistency(ConsistencyLevel::Quorum)
            .build()
            .await;

        // Quorum 需要 2/3 成功
        storage.write("key1", b"value1").await.unwrap();
        assert_eq!(storage.read("key1").await.unwrap(), b"value1");
    }

    #[tokio::test]
    async fn test_replicated_storage_read_strategy_primary_only() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_read_strategy(ReadStrategy::PrimaryOnly)
            .build()
            .await;

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_replicated_storage_read_strategy_any() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_read_strategy(ReadStrategy::Any)
            .build()
            .await;

        storage.write("key1", b"value1").await.unwrap();

        // 多次读取应该成功
        for _ in 0..5 {
            let data = storage.read("key1").await.unwrap();
            assert_eq!(data, b"value1");
        }
    }

    #[tokio::test]
    async fn test_backend_status() {
        assert_eq!(BackendStatus::Healthy.to_string(), "Healthy");
        assert_eq!(BackendStatus::Degraded.to_string(), "Degraded");
        assert_eq!(BackendStatus::Unavailable.to_string(), "Unavailable");
    }

    #[tokio::test]
    async fn test_replication_stats_snapshot() {
        let stats = ReplicationStats::default();
        stats.writes.store(10, Ordering::Relaxed);
        stats.successful_replications.store(8, Ordering::Relaxed);
        stats.partial_replications.store(1, Ordering::Relaxed);
        stats.failed_replications.store(1, Ordering::Relaxed);
        stats.reads.store(20, Ordering::Relaxed);
        stats.failovers.store(2, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.writes, 10);
        assert_eq!(snapshot.reads, 20);
        assert!((snapshot.replication_success_rate() - 0.8).abs() < 0.001);
        assert!((snapshot.failover_rate() - 0.1).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_healthy_backend_count() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .build()
            .await;

        assert_eq!(storage.healthy_backend_count().await, 3);
    }

    #[test]
    fn test_consistency_level_default() {
        assert_eq!(ConsistencyLevel::default(), ConsistencyLevel::All);
    }

    #[test]
    fn test_read_strategy_default() {
        assert_eq!(ReadStrategy::default(), ReadStrategy::PrimaryWithFallback);
    }

    #[tokio::test]
    async fn test_replicated_storage_exists() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .build()
            .await;

        assert!(!storage.exists("key1").await.unwrap());

        storage.write("key1", b"value1").await.unwrap();
        assert!(storage.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_replicated_storage_multiple_writes() {
        let storage = ReplicatedStorageBuilder::new(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .with_replica(MemoryStorage::new())
            .build()
            .await;

        // 多次写入和读取验证复制正常工作
        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            storage.write(&key, value.as_bytes()).await.unwrap();
        }

        // 验证所有数据可读
        for i in 0..10 {
            let key = format!("key{}", i);
            let expected = format!("value{}", i);
            let data = storage.read(&key).await.unwrap();
            assert_eq!(data, expected.as_bytes());
        }

        // 验证统计信息
        let stats = storage.stats_snapshot();
        assert_eq!(stats.writes, 10);
        assert_eq!(stats.reads, 10);
    }
}
