//! AuditStorage - 审计日志存储层
//!
//! v1.76.0: 提供存储操作的审计日志记录
//!
//! ## 功能特性
//!
//! - **操作记录**: 记录所有存储操作（读/写/删除/列表/存在检查）
//! - **审计级别**: All / WriteOnly / ReadWrite / None
//! - **审计后端**: 可插拔的审计日志存储（内存/回调/自定义）
//! - **查询过滤**: 按时间/操作类型/键前缀查询
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{AuditStorage, MemoryStorage, AuditLevel};
//!
//! let storage = AuditStorage::new(MemoryStorage::new());
//!
//! // 执行操作（自动记录审计日志）
//! storage.write("key1", b"value1").await?;
//! storage.read("key1").await?;
//!
//! // 查询审计日志
//! let entries = storage.query_entries(AuditQuery::all()).await;
//! for entry in entries {
//!     println!("{}: {} on {}", entry.timestamp, entry.operation, entry.key);
//! }
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// 审计操作类型
// ============================================================================

/// 审计操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditOperation {
    /// 读取操作
    Read,
    /// 写入操作
    Write,
    /// 删除操作
    Delete,
    /// 列表操作
    List,
    /// 存在检查
    Exists,
}

impl std::fmt::Display for AuditOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditOperation::Read => write!(f, "READ"),
            AuditOperation::Write => write!(f, "WRITE"),
            AuditOperation::Delete => write!(f, "DELETE"),
            AuditOperation::List => write!(f, "LIST"),
            AuditOperation::Exists => write!(f, "EXISTS"),
        }
    }
}

/// 操作结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuditResult {
    /// 成功
    Success,
    /// 未找到
    NotFound,
    /// 错误
    Error(String),
}

impl std::fmt::Display for AuditResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditResult::Success => write!(f, "SUCCESS"),
            AuditResult::NotFound => write!(f, "NOT_FOUND"),
            AuditResult::Error(e) => write!(f, "ERROR: {}", e),
        }
    }
}

// ============================================================================
// 审计条目
// ============================================================================

/// 审计日志条目
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// 条目 ID
    pub id: u64,
    /// 操作类型
    pub operation: AuditOperation,
    /// 操作的键
    pub key: String,
    /// 操作结果
    pub result: AuditResult,
    /// 数据大小（字节，仅写入操作）
    pub data_size: Option<usize>,
    /// 操作耗时
    pub duration: Duration,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl AuditEntry {
    /// 创建新的审计条目
    pub fn new(
        id: u64,
        operation: AuditOperation,
        key: String,
        result: AuditResult,
        data_size: Option<usize>,
        duration: Duration,
    ) -> Self {
        Self {
            id,
            operation,
            key,
            result,
            data_size,
            duration,
            timestamp: chrono::Utc::now(),
        }
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self.result, AuditResult::Success)
    }

    /// 是否失败
    pub fn is_error(&self) -> bool {
        matches!(self.result, AuditResult::Error(_))
    }
}

// ============================================================================
// 审计级别
// ============================================================================

/// 审计级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuditLevel {
    /// 记录所有操作
    #[default]
    All,
    /// 仅记录写入操作（Write/Delete）
    WriteOnly,
    /// 记录读写操作（Read/Write/Delete）
    ReadWrite,
    /// 不记录
    None,
}

impl AuditLevel {
    /// 检查操作是否应该被审计
    pub fn should_audit(&self, op: AuditOperation) -> bool {
        match self {
            AuditLevel::All => true,
            AuditLevel::WriteOnly => matches!(op, AuditOperation::Write | AuditOperation::Delete),
            AuditLevel::ReadWrite => matches!(
                op,
                AuditOperation::Read | AuditOperation::Write | AuditOperation::Delete
            ),
            AuditLevel::None => false,
        }
    }
}

// ============================================================================
// 审计后端
// ============================================================================

/// 审计后端 trait
#[async_trait]
pub trait AuditBackend: Send + Sync {
    /// 记录审计条目
    async fn record(&self, entry: AuditEntry);

    /// 查询审计条目
    async fn query(&self, query: &AuditQuery) -> Vec<AuditEntry>;

    /// 获取条目数量
    async fn count(&self) -> usize;

    /// 清空审计日志
    async fn clear(&self);
}

/// 内存审计后端
pub struct MemoryAuditBackend {
    /// 审计条目列表
    entries: RwLock<Vec<AuditEntry>>,
    /// 最大条目数
    max_entries: usize,
}

impl MemoryAuditBackend {
    /// 创建新的内存审计后端
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// 创建带容量限制的内存审计后端
    pub fn with_capacity(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            max_entries,
        }
    }
}

impl Default for MemoryAuditBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditBackend for MemoryAuditBackend {
    async fn record(&self, entry: AuditEntry) {
        let mut entries = self.entries.write().await;

        // 如果超过容量，删除最老的条目
        if entries.len() >= self.max_entries {
            entries.remove(0);
        }

        entries.push(entry);
    }

    async fn query(&self, query: &AuditQuery) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;

        entries
            .iter()
            .filter(|e| query.matches(e))
            .skip(query.offset)
            .take(query.limit)
            .cloned()
            .collect()
    }

    async fn count(&self) -> usize {
        self.entries.read().await.len()
    }

    async fn clear(&self) {
        self.entries.write().await.clear();
    }
}

/// 回调审计后端
pub struct CallbackAuditBackend<F>
where
    F: Fn(AuditEntry) + Send + Sync,
{
    callback: F,
}

impl<F> CallbackAuditBackend<F>
where
    F: Fn(AuditEntry) + Send + Sync,
{
    /// 创建回调审计后端
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

#[async_trait]
impl<F> AuditBackend for CallbackAuditBackend<F>
where
    F: Fn(AuditEntry) + Send + Sync,
{
    async fn record(&self, entry: AuditEntry) {
        (self.callback)(entry);
    }

    async fn query(&self, _query: &AuditQuery) -> Vec<AuditEntry> {
        // 回调后端不支持查询
        Vec::new()
    }

    async fn count(&self) -> usize {
        0
    }

    async fn clear(&self) {
        // 回调后端无需清空
    }
}

// ============================================================================
// 审计查询
// ============================================================================

/// 审计查询条件
#[derive(Debug, Clone, Default)]
pub struct AuditQuery {
    /// 操作类型过滤
    pub operations: Option<Vec<AuditOperation>>,
    /// 键前缀过滤
    pub key_prefix: Option<String>,
    /// 精确键匹配
    pub key_exact: Option<String>,
    /// 开始时间
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 结束时间
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// 仅成功
    pub success_only: bool,
    /// 仅失败
    pub error_only: bool,
    /// 偏移量
    pub offset: usize,
    /// 限制数量
    pub limit: usize,
}

impl AuditQuery {
    /// 查询所有条目
    pub fn all() -> Self {
        Self {
            limit: usize::MAX,
            ..Default::default()
        }
    }

    /// 按操作类型过滤
    pub fn with_operations(mut self, ops: Vec<AuditOperation>) -> Self {
        self.operations = Some(ops);
        self
    }

    /// 按键前缀过滤
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = Some(prefix.into());
        self
    }

    /// 精确键匹配
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key_exact = Some(key.into());
        self
    }

    /// 设置时间范围
    pub fn with_time_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// 仅成功操作
    pub fn success_only(mut self) -> Self {
        self.success_only = true;
        self.error_only = false;
        self
    }

    /// 仅失败操作
    pub fn error_only(mut self) -> Self {
        self.error_only = true;
        self.success_only = false;
        self
    }

    /// 设置分页
    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.offset = offset;
        self.limit = limit;
        self
    }

    /// 检查条目是否匹配查询条件
    pub fn matches(&self, entry: &AuditEntry) -> bool {
        // 操作类型过滤
        if let Some(ref ops) = self.operations {
            if !ops.contains(&entry.operation) {
                return false;
            }
        }

        // 键前缀过滤
        if let Some(ref prefix) = self.key_prefix {
            if !entry.key.starts_with(prefix) {
                return false;
            }
        }

        // 精确键匹配
        if let Some(ref key) = self.key_exact {
            if &entry.key != key {
                return false;
            }
        }

        // 时间范围过滤
        if let Some(start) = self.start_time {
            if entry.timestamp < start {
                return false;
            }
        }
        if let Some(end) = self.end_time {
            if entry.timestamp > end {
                return false;
            }
        }

        // 成功/失败过滤
        if self.success_only && !entry.is_success() {
            return false;
        }
        if self.error_only && !entry.is_error() {
            return false;
        }

        true
    }
}

// ============================================================================
// 配置
// ============================================================================

/// AuditStorage 配置
#[derive(Debug, Clone)]
pub struct AuditStorageConfig {
    /// 审计级别
    pub level: AuditLevel,
    /// 是否记录数据大小
    pub record_data_size: bool,
}

impl Default for AuditStorageConfig {
    fn default() -> Self {
        Self {
            level: AuditLevel::All,
            record_data_size: true,
        }
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 审计统计
#[derive(Debug, Default)]
pub struct AuditStats {
    /// 总审计条目数
    total_entries: AtomicU64,
    /// 读取操作数
    read_ops: AtomicU64,
    /// 写入操作数
    write_ops: AtomicU64,
    /// 删除操作数
    delete_ops: AtomicU64,
    /// 列表操作数
    list_ops: AtomicU64,
    /// 存在检查数
    exists_ops: AtomicU64,
    /// 成功操作数
    success_count: AtomicU64,
    /// 失败操作数
    error_count: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct AuditStatsSnapshot {
    pub total_entries: u64,
    pub read_ops: u64,
    pub write_ops: u64,
    pub delete_ops: u64,
    pub list_ops: u64,
    pub exists_ops: u64,
    pub success_count: u64,
    pub error_count: u64,
}

impl AuditStatsSnapshot {
    /// 成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.error_count;
        if total == 0 {
            1.0
        } else {
            self.success_count as f64 / total as f64
        }
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedAuditStats {
    /// 快照统计
    pub snapshot: AuditStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 审计后端条目数
    pub audit_entry_count: usize,
}

// ============================================================================
// AuditStorage 实现
// ============================================================================

/// 审计存储层
///
/// 装饰器模式，包装底层存储并记录所有操作
pub struct AuditStorage<B: StorageBackend, A: AuditBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 审计后端
    audit_backend: Arc<A>,
    /// 配置
    config: AuditStorageConfig,
    /// 下一个条目 ID
    next_id: AtomicU64,
    /// 统计信息
    stats: Arc<AuditStats>,
}

impl<B: StorageBackend> AuditStorage<B, MemoryAuditBackend> {
    /// 创建新的 AuditStorage（使用内存审计后端）
    pub fn new(backend: B) -> Self {
        Self::with_audit_backend(backend, MemoryAuditBackend::new())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_audit_backend(backend, Arc::new(MemoryAuditBackend::new()))
    }
}

impl<B: StorageBackend, A: AuditBackend> AuditStorage<B, A> {
    /// 使用自定义审计后端创建
    pub fn with_audit_backend(backend: B, audit_backend: A) -> Self {
        Self::from_arc_with_audit_backend(Arc::new(backend), Arc::new(audit_backend))
    }

    /// 从 Arc 使用自定义审计后端创建
    pub fn from_arc_with_audit_backend(backend: Arc<B>, audit_backend: Arc<A>) -> Self {
        Self {
            backend,
            audit_backend,
            config: AuditStorageConfig::default(),
            next_id: AtomicU64::new(1),
            stats: Arc::new(AuditStats::default()),
        }
    }

    /// 设置配置
    pub fn with_config(mut self, config: AuditStorageConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置审计级别
    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.config.level = level;
        self
    }

    /// 查询审计条目
    pub async fn query_entries(&self, query: AuditQuery) -> Vec<AuditEntry> {
        self.audit_backend.query(&query).await
    }

    /// 获取所有审计条目
    pub async fn all_entries(&self) -> Vec<AuditEntry> {
        self.audit_backend.query(&AuditQuery::all()).await
    }

    /// 获取审计条目数量
    pub async fn entry_count(&self) -> usize {
        self.audit_backend.count().await
    }

    /// 清空审计日志
    pub async fn clear_audit_log(&self) {
        self.audit_backend.clear().await;
    }

    /// 记录审计条目
    async fn record_audit(
        &self,
        operation: AuditOperation,
        key: &str,
        result: AuditResult,
        data_size: Option<usize>,
        duration: Duration,
    ) {
        // 检查是否需要审计
        if !self.config.level.should_audit(operation) {
            return;
        }

        // 更新统计
        self.stats.total_entries.fetch_add(1, Ordering::SeqCst);
        match operation {
            AuditOperation::Read => self.stats.read_ops.fetch_add(1, Ordering::SeqCst),
            AuditOperation::Write => self.stats.write_ops.fetch_add(1, Ordering::SeqCst),
            AuditOperation::Delete => self.stats.delete_ops.fetch_add(1, Ordering::SeqCst),
            AuditOperation::List => self.stats.list_ops.fetch_add(1, Ordering::SeqCst),
            AuditOperation::Exists => self.stats.exists_ops.fetch_add(1, Ordering::SeqCst),
        };

        match &result {
            AuditResult::Success | AuditResult::NotFound => {
                self.stats.success_count.fetch_add(1, Ordering::SeqCst);
            }
            AuditResult::Error(_) => {
                self.stats.error_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        // 创建审计条目
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let entry = AuditEntry::new(
            id,
            operation,
            key.to_string(),
            result,
            if self.config.record_data_size {
                data_size
            } else {
                None
            },
            duration,
        );

        // 记录到审计后端
        self.audit_backend.record(entry).await;
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> AuditStatsSnapshot {
        AuditStatsSnapshot {
            total_entries: self.stats.total_entries.load(Ordering::SeqCst),
            read_ops: self.stats.read_ops.load(Ordering::SeqCst),
            write_ops: self.stats.write_ops.load(Ordering::SeqCst),
            delete_ops: self.stats.delete_ops.load(Ordering::SeqCst),
            list_ops: self.stats.list_ops.load(Ordering::SeqCst),
            exists_ops: self.stats.exists_ops.load(Ordering::SeqCst),
            success_count: self.stats.success_count.load(Ordering::SeqCst),
            error_count: self.stats.error_count.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> DetailedAuditStats {
        DetailedAuditStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            audit_entry_count: self.audit_backend.count().await,
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend, A: AuditBackend> StorageBackend for AuditStorage<B, A> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let start = Instant::now();
        let result = self.backend.read(key).await;
        let duration = start.elapsed();

        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(StorageError::NotFound(_)) => AuditResult::NotFound,
            Err(e) => AuditResult::Error(e.to_string()),
        };

        self.record_audit(AuditOperation::Read, key, audit_result, None, duration)
            .await;

        result
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let start = Instant::now();
        let data_size = data.len();
        let result = self.backend.write(key, data).await;
        let duration = start.elapsed();

        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::Error(e.to_string()),
        };

        self.record_audit(
            AuditOperation::Write,
            key,
            audit_result,
            Some(data_size),
            duration,
        )
        .await;

        result
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let start = Instant::now();
        let result = self.backend.delete(key).await;
        let duration = start.elapsed();

        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::Error(e.to_string()),
        };

        self.record_audit(AuditOperation::Delete, key, audit_result, None, duration)
            .await;

        result
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let start = Instant::now();
        let result = self.backend.list(prefix).await;
        let duration = start.elapsed();

        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::Error(e.to_string()),
        };

        self.record_audit(AuditOperation::List, prefix, audit_result, None, duration)
            .await;

        result
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let start = Instant::now();
        let result = self.backend.exists(key).await;
        let duration = start.elapsed();

        let audit_result = match &result {
            Ok(_) => AuditResult::Success,
            Err(e) => AuditResult::Error(e.to_string()),
        };

        self.record_audit(AuditOperation::Exists, key, audit_result, None, duration)
            .await;

        result
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "AuditStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// AuditStorage 构建器
pub struct AuditStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: AuditStorageConfig,
    max_entries: usize,
}

impl<B: StorageBackend> AuditStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: AuditStorageConfig::default(),
            max_entries: 10000,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: AuditStorageConfig::default(),
            max_entries: 10000,
        }
    }

    /// 设置审计级别
    pub fn level(mut self, level: AuditLevel) -> Self {
        self.config.level = level;
        self
    }

    /// 设置是否记录数据大小
    pub fn record_data_size(mut self, record: bool) -> Self {
        self.config.record_data_size = record;
        self
    }

    /// 设置最大审计条目数
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// 构建（使用内存审计后端）
    pub fn build(self) -> AuditStorage<B, MemoryAuditBackend> {
        let audit_backend = MemoryAuditBackend::with_capacity(self.max_entries);
        AuditStorage {
            backend: self.backend,
            audit_backend: Arc::new(audit_backend),
            config: self.config,
            next_id: AtomicU64::new(1),
            stats: Arc::new(AuditStats::default()),
        }
    }

    /// 使用自定义审计后端构建
    pub fn build_with_backend<A: AuditBackend>(self, audit_backend: A) -> AuditStorage<B, A> {
        AuditStorage {
            backend: self.backend,
            audit_backend: Arc::new(audit_backend),
            config: self.config,
            next_id: AtomicU64::new(1),
            stats: Arc::new(AuditStats::default()),
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
    async fn test_audit_storage_basic() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_audit_entry_recorded() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();

        let entries = storage.all_entries().await;
        assert_eq!(entries.len(), 3);

        assert_eq!(entries[0].operation, AuditOperation::Write);
        assert_eq!(entries[0].key, "key1");
        assert!(entries[0].is_success());

        assert_eq!(entries[1].operation, AuditOperation::Read);
        assert_eq!(entries[2].operation, AuditOperation::Delete);
    }

    #[tokio::test]
    async fn test_audit_data_size() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"hello").await.unwrap();

        let entries = storage.all_entries().await;
        assert_eq!(entries[0].data_size, Some(5));
    }

    #[tokio::test]
    async fn test_audit_not_found() {
        let storage = AuditStorage::new(MemoryStorage::new());

        let result = storage.read("nonexistent").await;
        assert!(result.is_err());

        let entries = storage.all_entries().await;
        assert_eq!(entries[0].result, AuditResult::NotFound);
    }

    #[tokio::test]
    async fn test_audit_level_write_only() {
        let storage = AuditStorage::new(MemoryStorage::new()).with_level(AuditLevel::WriteOnly);

        storage.write("key1", b"value1").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();
        let _ = storage.exists("key1").await;
        let _ = storage.list("").await;

        let entries = storage.all_entries().await;
        // 只有 Write 和 Delete 被记录
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].operation, AuditOperation::Write);
        assert_eq!(entries[1].operation, AuditOperation::Delete);
    }

    #[tokio::test]
    async fn test_audit_level_read_write() {
        let storage = AuditStorage::new(MemoryStorage::new()).with_level(AuditLevel::ReadWrite);

        storage.write("key1", b"value1").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();
        let _ = storage.exists("key1").await;
        let _ = storage.list("").await;

        let entries = storage.all_entries().await;
        // Read, Write, Delete 被记录
        assert_eq!(entries.len(), 3);
    }

    #[tokio::test]
    async fn test_audit_level_none() {
        let storage = AuditStorage::new(MemoryStorage::new()).with_level(AuditLevel::None);

        storage.write("key1", b"value1").await.unwrap();
        storage.read("key1").await.unwrap();

        let entries = storage.all_entries().await;
        assert_eq!(entries.len(), 0);
    }

    #[tokio::test]
    async fn test_query_by_operation() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();

        let writes = storage
            .query_entries(AuditQuery::all().with_operations(vec![AuditOperation::Write]))
            .await;
        assert_eq!(writes.len(), 2);

        let reads = storage
            .query_entries(AuditQuery::all().with_operations(vec![AuditOperation::Read]))
            .await;
        assert_eq!(reads.len(), 1);
    }

    #[tokio::test]
    async fn test_query_by_key_prefix() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("user:1", b"u1").await.unwrap();
        storage.write("user:2", b"u2").await.unwrap();
        storage.write("order:1", b"o1").await.unwrap();

        let user_entries = storage
            .query_entries(AuditQuery::all().with_key_prefix("user:"))
            .await;
        assert_eq!(user_entries.len(), 2);
    }

    #[tokio::test]
    async fn test_query_by_exact_key() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"v1").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        let key1_entries = storage
            .query_entries(AuditQuery::all().with_key("key1"))
            .await;
        assert_eq!(key1_entries.len(), 2);
    }

    #[tokio::test]
    async fn test_query_pagination() {
        let storage = AuditStorage::new(MemoryStorage::new());

        for i in 0..10 {
            storage.write(&format!("key{}", i), b"v").await.unwrap();
        }

        let page1 = storage
            .query_entries(AuditQuery::all().with_pagination(0, 3))
            .await;
        assert_eq!(page1.len(), 3);

        let page2 = storage
            .query_entries(AuditQuery::all().with_pagination(3, 3))
            .await;
        assert_eq!(page2.len(), 3);

        // 验证不同页
        assert_ne!(page1[0].key, page2[0].key);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();
        storage.read("key1").await.unwrap();
        storage.delete("key1").await.unwrap();
        let _ = storage.exists("key2").await;
        let _ = storage.list("").await;

        let stats = storage.stats_snapshot();
        assert_eq!(stats.write_ops, 2);
        assert_eq!(stats.read_ops, 1);
        assert_eq!(stats.delete_ops, 1);
        assert_eq!(stats.exists_ops, 1);
        assert_eq!(stats.list_ops, 1);
        assert_eq!(stats.total_entries, 6);
    }

    #[tokio::test]
    async fn test_success_rate() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"v1").await.unwrap();
        storage.read("key1").await.unwrap();
        let _ = storage.read("nonexistent").await; // NotFound 也算成功

        let stats = storage.stats_snapshot();
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_clear_audit_log() {
        let storage = AuditStorage::new(MemoryStorage::new());

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        assert_eq!(storage.entry_count().await, 2);

        storage.clear_audit_log().await;

        assert_eq!(storage.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_builder() {
        let storage = AuditStorageBuilder::new(MemoryStorage::new())
            .level(AuditLevel::WriteOnly)
            .record_data_size(false)
            .max_entries(100)
            .build();

        storage.write("key1", b"value1").await.unwrap();

        let entries = storage.all_entries().await;
        assert_eq!(entries[0].data_size, None);
    }

    #[tokio::test]
    async fn test_memory_backend_capacity() {
        let backend = MemoryAuditBackend::with_capacity(3);
        let storage = AuditStorage::with_audit_backend(MemoryStorage::new(), backend);

        for i in 0..5 {
            storage.write(&format!("key{}", i), b"v").await.unwrap();
        }

        // 只保留最后 3 个
        let entries = storage.all_entries().await;
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].key, "key2");
        assert_eq!(entries[2].key, "key4");
    }

    #[tokio::test]
    async fn test_callback_backend() {
        use std::sync::Mutex;

        let recorded = Arc::new(Mutex::new(Vec::new()));
        let recorded_clone = Arc::clone(&recorded);

        let callback = move |entry: AuditEntry| {
            recorded_clone.lock().unwrap().push(entry.key.clone());
        };

        let backend = CallbackAuditBackend::new(callback);
        let storage = AuditStorage::with_audit_backend(MemoryStorage::new(), backend);

        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        let keys = recorded.lock().unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys[0], "key1");
        assert_eq!(keys[1], "key2");
    }

    #[tokio::test]
    async fn test_audit_operation_display() {
        assert_eq!(format!("{}", AuditOperation::Read), "READ");
        assert_eq!(format!("{}", AuditOperation::Write), "WRITE");
        assert_eq!(format!("{}", AuditOperation::Delete), "DELETE");
        assert_eq!(format!("{}", AuditOperation::List), "LIST");
        assert_eq!(format!("{}", AuditOperation::Exists), "EXISTS");
    }

    #[tokio::test]
    async fn test_audit_result_display() {
        assert_eq!(format!("{}", AuditResult::Success), "SUCCESS");
        assert_eq!(format!("{}", AuditResult::NotFound), "NOT_FOUND");
        assert_eq!(
            format!("{}", AuditResult::Error("test".to_string())),
            "ERROR: test"
        );
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = AuditStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        assert_eq!(storage.entry_count().await, 1);
    }
}
