//! RateLimitedStorage - 速率限制存储层
//!
//! v1.78.0: 提供存储操作的速率限制
//!
//! ## 功能特性
//!
//! - **令牌桶算法**: 平滑的速率控制
//! - **操作级别限制**: 读/写分别限流
//! - **突发容量**: 支持短期突发请求
//! - **策略选择**: Wait（等待）/ Reject（拒绝）
//! - **统计追踪**: 限流次数、等待时间等
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{RateLimitedStorage, MemoryStorage, RateLimitConfig};
//!
//! let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
//!     .read_rate(100.0)      // 100 reads/sec
//!     .write_rate(50.0)      // 50 writes/sec
//!     .burst_size(10)        // 突发容量
//!     .build();
//!
//! // 操作会自动限流
//! storage.write("key1", b"value1").await?;
//! storage.read("key1").await?;
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// ============================================================================
// 速率限制策略
// ============================================================================

/// 速率限制策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RateLimitPolicy {
    /// 等待直到有可用令牌
    #[default]
    Wait,
    /// 立即拒绝超限请求
    Reject,
}

// ============================================================================
// 速率限制错误
// ============================================================================

/// 速率限制错误
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitError {
    /// 读取速率超限
    ReadRateLimited {
        wait_time: Duration,
    },
    /// 写入速率超限
    WriteRateLimited {
        wait_time: Duration,
    },
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::ReadRateLimited { wait_time } => {
                write!(f, "Read rate limited, wait {:?}", wait_time)
            }
            RateLimitError::WriteRateLimited { wait_time } => {
                write!(f, "Write rate limited, wait {:?}", wait_time)
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

// ============================================================================
// 令牌桶
// ============================================================================

/// 令牌桶实现
struct TokenBucket {
    /// 令牌生成速率（每秒）
    rate: f64,
    /// 桶容量（突发大小）
    capacity: f64,
    /// 当前令牌数
    tokens: f64,
    /// 上次更新时间
    last_update: Instant,
}

impl TokenBucket {
    /// 创建新的令牌桶
    fn new(rate: f64, capacity: f64) -> Self {
        Self {
            rate,
            capacity,
            tokens: capacity, // 初始满桶
            last_update: Instant::now(),
        }
    }

    /// 尝试获取令牌
    fn try_acquire(&mut self) -> Option<Duration> {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            None // 成功，无需等待
        } else {
            // 计算需要等待的时间
            let needed = 1.0 - self.tokens;
            let wait_secs = needed / self.rate;
            Some(Duration::from_secs_f64(wait_secs))
        }
    }

    /// 获取令牌（等待模式）
    async fn acquire(&mut self) {
        if let Some(wait_time) = self.try_acquire() {
            tokio::time::sleep(wait_time).await;
            // 重新尝试
            self.refill();
            self.tokens -= 1.0;
        }
    }

    /// 补充令牌
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity);
        self.last_update = now;
    }

    /// 获取当前可用令牌数
    fn available(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

// ============================================================================
// 配置
// ============================================================================

/// 速率限制配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 读取速率（每秒）
    pub read_rate: Option<f64>,
    /// 写入速率（每秒）
    pub write_rate: Option<f64>,
    /// 突发容量
    pub burst_size: usize,
    /// 限流策略
    pub policy: RateLimitPolicy,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            read_rate: None,
            write_rate: None,
            burst_size: 10,
            policy: RateLimitPolicy::Wait,
        }
    }
}

// ============================================================================
// 统计信息
// ============================================================================

/// 速率限制统计
#[derive(Debug, Default)]
pub struct RateLimitStats {
    /// 读取请求总数
    read_requests: AtomicU64,
    /// 写入请求总数
    write_requests: AtomicU64,
    /// 读取被限流次数
    read_limited: AtomicU64,
    /// 写入被限流次数
    write_limited: AtomicU64,
    /// 读取被拒绝次数
    read_rejected: AtomicU64,
    /// 写入被拒绝次数
    write_rejected: AtomicU64,
    /// 总等待时间（微秒）
    total_wait_us: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct RateLimitStatsSnapshot {
    pub read_requests: u64,
    pub write_requests: u64,
    pub read_limited: u64,
    pub write_limited: u64,
    pub read_rejected: u64,
    pub write_rejected: u64,
    pub total_wait_us: u64,
}

impl RateLimitStatsSnapshot {
    /// 读取限流率
    pub fn read_limit_rate(&self) -> f64 {
        if self.read_requests == 0 {
            0.0
        } else {
            self.read_limited as f64 / self.read_requests as f64
        }
    }

    /// 写入限流率
    pub fn write_limit_rate(&self) -> f64 {
        if self.write_requests == 0 {
            0.0
        } else {
            self.write_limited as f64 / self.write_requests as f64
        }
    }

    /// 平均等待时间
    pub fn avg_wait_time(&self) -> Duration {
        let total = self.read_limited + self.write_limited;
        if total == 0 {
            Duration::ZERO
        } else {
            Duration::from_micros(self.total_wait_us / total)
        }
    }

    /// 总等待时间
    pub fn total_wait_time(&self) -> Duration {
        Duration::from_micros(self.total_wait_us)
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedRateLimitStats {
    /// 快照统计
    pub snapshot: RateLimitStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 当前读取可用令牌
    pub read_tokens_available: f64,
    /// 当前写入可用令牌
    pub write_tokens_available: f64,
}

// ============================================================================
// RateLimitedStorage 实现
// ============================================================================

/// 速率限制存储层
///
/// 装饰器模式，包装底层存储并实施速率限制
pub struct RateLimitedStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配置
    config: RateLimitConfig,
    /// 读取令牌桶
    read_bucket: Option<Mutex<TokenBucket>>,
    /// 写入令牌桶
    write_bucket: Option<Mutex<TokenBucket>>,
    /// 统计信息
    stats: Arc<RateLimitStats>,
}

impl<B: StorageBackend> RateLimitedStorage<B> {
    /// 创建新的 RateLimitedStorage
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, RateLimitConfig::default())
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self::from_arc_with_config(backend, RateLimitConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(backend: B, config: RateLimitConfig) -> Self {
        Self::from_arc_with_config(Arc::new(backend), config)
    }

    /// 从 Arc 使用配置创建
    pub fn from_arc_with_config(backend: Arc<B>, config: RateLimitConfig) -> Self {
        let read_bucket = config.read_rate.map(|rate| {
            Mutex::new(TokenBucket::new(rate, config.burst_size as f64))
        });

        let write_bucket = config.write_rate.map(|rate| {
            Mutex::new(TokenBucket::new(rate, config.burst_size as f64))
        });

        Self {
            backend,
            config,
            read_bucket,
            write_bucket,
            stats: Arc::new(RateLimitStats::default()),
        }
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> RateLimitStatsSnapshot {
        RateLimitStatsSnapshot {
            read_requests: self.stats.read_requests.load(Ordering::SeqCst),
            write_requests: self.stats.write_requests.load(Ordering::SeqCst),
            read_limited: self.stats.read_limited.load(Ordering::SeqCst),
            write_limited: self.stats.write_limited.load(Ordering::SeqCst),
            read_rejected: self.stats.read_rejected.load(Ordering::SeqCst),
            write_rejected: self.stats.write_rejected.load(Ordering::SeqCst),
            total_wait_us: self.stats.total_wait_us.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub async fn detailed_stats(&self) -> DetailedRateLimitStats {
        let read_tokens = if let Some(ref bucket) = self.read_bucket {
            bucket.lock().await.available()
        } else {
            f64::INFINITY
        };

        let write_tokens = if let Some(ref bucket) = self.write_bucket {
            bucket.lock().await.available()
        } else {
            f64::INFINITY
        };

        DetailedRateLimitStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            read_tokens_available: read_tokens,
            write_tokens_available: write_tokens,
        }
    }

    /// 获取读取令牌
    async fn acquire_read(&self) -> Result<(), RateLimitError> {
        self.stats.read_requests.fetch_add(1, Ordering::SeqCst);

        if let Some(ref bucket) = self.read_bucket {
            let mut bucket = bucket.lock().await;

            if let Some(wait_time) = bucket.try_acquire() {
                self.stats.read_limited.fetch_add(1, Ordering::SeqCst);

                match self.config.policy {
                    RateLimitPolicy::Wait => {
                        self.stats
                            .total_wait_us
                            .fetch_add(wait_time.as_micros() as u64, Ordering::SeqCst);
                        tokio::time::sleep(wait_time).await;
                        bucket.refill();
                        bucket.tokens -= 1.0;
                    }
                    RateLimitPolicy::Reject => {
                        self.stats.read_rejected.fetch_add(1, Ordering::SeqCst);
                        return Err(RateLimitError::ReadRateLimited { wait_time });
                    }
                }
            }
        }

        Ok(())
    }

    /// 获取写入令牌
    async fn acquire_write(&self) -> Result<(), RateLimitError> {
        self.stats.write_requests.fetch_add(1, Ordering::SeqCst);

        if let Some(ref bucket) = self.write_bucket {
            let mut bucket = bucket.lock().await;

            if let Some(wait_time) = bucket.try_acquire() {
                self.stats.write_limited.fetch_add(1, Ordering::SeqCst);

                match self.config.policy {
                    RateLimitPolicy::Wait => {
                        self.stats
                            .total_wait_us
                            .fetch_add(wait_time.as_micros() as u64, Ordering::SeqCst);
                        tokio::time::sleep(wait_time).await;
                        bucket.refill();
                        bucket.tokens -= 1.0;
                    }
                    RateLimitPolicy::Reject => {
                        self.stats.write_rejected.fetch_add(1, Ordering::SeqCst);
                        return Err(RateLimitError::WriteRateLimited { wait_time });
                    }
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for RateLimitedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        if let Err(e) = self.acquire_read().await {
            return Err(StorageError::Other(e.to_string()));
        }
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        if let Err(e) = self.acquire_write().await {
            return Err(StorageError::Other(e.to_string()));
        }
        self.backend.write(key, data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        // delete 计入写入限流
        if let Err(e) = self.acquire_write().await {
            return Err(StorageError::Other(e.to_string()));
        }
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        // list 计入读取限流
        if let Err(e) = self.acquire_read().await {
            return Err(StorageError::Other(e.to_string()));
        }
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        // exists 计入读取限流
        if let Err(e) = self.acquire_read().await {
            return Err(StorageError::Other(e.to_string()));
        }
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "RateLimitedStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// RateLimitedStorage 构建器
pub struct RateLimitedStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    config: RateLimitConfig,
}

impl<B: StorageBackend> RateLimitedStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: RateLimitConfig::default(),
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: RateLimitConfig::default(),
        }
    }

    /// 设置读取速率（每秒）
    pub fn read_rate(mut self, rate: f64) -> Self {
        self.config.read_rate = Some(rate);
        self
    }

    /// 设置写入速率（每秒）
    pub fn write_rate(mut self, rate: f64) -> Self {
        self.config.write_rate = Some(rate);
        self
    }

    /// 设置统一速率（读写相同）
    pub fn rate(mut self, rate: f64) -> Self {
        self.config.read_rate = Some(rate);
        self.config.write_rate = Some(rate);
        self
    }

    /// 设置突发容量
    pub fn burst_size(mut self, size: usize) -> Self {
        self.config.burst_size = size;
        self
    }

    /// 设置限流策略
    pub fn policy(mut self, policy: RateLimitPolicy) -> Self {
        self.config.policy = policy;
        self
    }

    /// 构建
    pub fn build(self) -> RateLimitedStorage<B> {
        RateLimitedStorage::from_arc_with_config(self.backend, self.config)
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
    async fn test_rate_limited_storage_basic() {
        let storage = RateLimitedStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_no_limit_passes() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .build();

        // 无限制时应该都能通过
        for i in 0..100 {
            storage.write(&format!("key{}", i), b"v").await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_burst_allows_initial() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(10.0) // 10/sec
            .burst_size(5)
            .policy(RateLimitPolicy::Reject)
            .build();

        // 突发容量内应该都能通过
        for i in 0..5 {
            storage.write(&format!("key{}", i), b"v").await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_reject_policy() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(10.0)
            .burst_size(2)
            .policy(RateLimitPolicy::Reject)
            .build();

        // 前两个应该通过
        storage.write("key1", b"v").await.unwrap();
        storage.write("key2", b"v").await.unwrap();

        // 第三个应该被拒绝
        let result = storage.write("key3", b"v").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("rate limited"));
    }

    #[tokio::test]
    async fn test_wait_policy() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(1000.0) // 高速率以快速测试
            .burst_size(1)
            .policy(RateLimitPolicy::Wait)
            .build();

        let start = Instant::now();

        // 连续写入几个
        for i in 0..3 {
            storage.write(&format!("key{}", i), b"v").await.unwrap();
        }

        // 应该有等待时间（但因为速率高，很短）
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 100); // 应该很快
    }

    #[tokio::test]
    async fn test_separate_read_write_limits() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .read_rate(10.0)
            .write_rate(10.0)
            .burst_size(2)
            .policy(RateLimitPolicy::Reject)
            .build();

        // 写入两个（用完写入突发）
        storage.write("key1", b"v1").await.unwrap();
        storage.write("key2", b"v2").await.unwrap();

        // 读取应该仍有自己的突发容量
        storage.read("key1").await.unwrap();
        storage.read("key2").await.unwrap();

        // 再写入应该被拒绝
        let result = storage.write("key3", b"v3").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(10.0)
            .burst_size(1)
            .policy(RateLimitPolicy::Reject)
            .build();

        storage.write("key1", b"v").await.unwrap();
        let _ = storage.write("key2", b"v").await; // 可能被拒绝

        let stats = storage.stats_snapshot();
        assert_eq!(stats.write_requests, 2);
        assert!(stats.write_limited >= 1 || stats.write_rejected == 0);
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(100.0) // 100/sec = 10ms per token
            .burst_size(1)
            .policy(RateLimitPolicy::Reject)
            .build();

        // 用完突发
        storage.write("key1", b"v").await.unwrap();

        // 立即尝试应该失败
        let result = storage.write("key2", b"v").await;
        assert!(result.is_err());

        // 等待令牌恢复
        tokio::time::sleep(Duration::from_millis(15)).await;

        // 现在应该成功
        storage.write("key3", b"v").await.unwrap();
    }

    #[tokio::test]
    async fn test_rate_limit_error_display() {
        let err1 = RateLimitError::ReadRateLimited {
            wait_time: Duration::from_millis(100),
        };
        assert!(err1.to_string().contains("Read rate limited"));

        let err2 = RateLimitError::WriteRateLimited {
            wait_time: Duration::from_millis(50),
        };
        assert!(err2.to_string().contains("Write rate limited"));
    }

    #[tokio::test]
    async fn test_stats_limit_rates() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(10.0)
            .burst_size(1)
            .policy(RateLimitPolicy::Reject)
            .build();

        // 1 成功，2 被限流
        storage.write("key1", b"v").await.unwrap();
        let _ = storage.write("key2", b"v").await;
        let _ = storage.write("key3", b"v").await;

        let stats = storage.stats_snapshot();
        // 写入限流率应该 > 0
        assert!(stats.write_limit_rate() > 0.0);
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .read_rate(100.0)
            .write_rate(100.0)
            .burst_size(5)
            .build();

        let detailed = storage.detailed_stats().await;
        assert_eq!(detailed.read_tokens_available, 5.0);
        assert_eq!(detailed.write_tokens_available, 5.0);
    }

    #[tokio::test]
    async fn test_builder_rate() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .rate(50.0) // 读写都是 50/sec
            .burst_size(3)
            .build();

        // 验证配置
        assert_eq!(storage.config.read_rate, Some(50.0));
        assert_eq!(storage.config.write_rate, Some(50.0));
        assert_eq!(storage.config.burst_size, 3);
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = RateLimitedStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_delete_uses_write_limit() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .write_rate(10.0)
            .burst_size(1)
            .policy(RateLimitPolicy::Reject)
            .build();

        storage.write("key1", b"v").await.unwrap();

        // delete 也使用写入限制
        let result = storage.delete("key1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_exists_use_read_limit() {
        let storage = RateLimitedStorageBuilder::new(MemoryStorage::new())
            .read_rate(10.0)
            .burst_size(2)
            .policy(RateLimitPolicy::Reject)
            .build();

        storage.list("").await.unwrap();
        storage.exists("key1").await.unwrap();

        // 第三个读操作应该被限制
        let result = storage.read("key1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_avg_wait_time() {
        let stats = RateLimitStatsSnapshot {
            read_requests: 10,
            write_requests: 10,
            read_limited: 5,
            write_limited: 5,
            read_rejected: 0,
            write_rejected: 0,
            total_wait_us: 100_000, // 100ms total
        };

        let avg = stats.avg_wait_time();
        assert_eq!(avg, Duration::from_micros(10_000)); // 10ms average
    }
}
