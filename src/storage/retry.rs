//! 重试存储层
//!
//! v1.73.0: 提供自动重试和退避策略，处理瞬态故障
//!
//! ## 设计目标
//!
//! - **自动重试**: 遇到可重试错误时自动重试
//! - **退避策略**: 支持多种退避算法（固定、线性、指数）
//! - **抖动**: 添加随机抖动避免雷群效应
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{RetryStorage, MemoryStorage, BackoffStrategy};
//!
//! let storage = MemoryStorage::new();
//! let retry_storage = RetryStorage::builder(storage)
//!     .max_retries(3)
//!     .backoff(BackoffStrategy::Exponential { base_ms: 100, max_ms: 5000 })
//!     .with_jitter(true)
//!     .build();
//!
//! // 失败时自动重试
//! retry_storage.write("key1", b"value1").await?;
//! ```

use crate::storage::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 退避策略
// ============================================================================

/// 退避策略
#[derive(Debug, Clone)]
pub enum BackoffStrategy {
    /// 固定延迟
    Fixed {
        /// 延迟时间（毫秒）
        delay_ms: u64,
    },
    /// 线性增长
    Linear {
        /// 初始延迟（毫秒）
        initial_ms: u64,
        /// 每次增加的延迟（毫秒）
        increment_ms: u64,
        /// 最大延迟（毫秒）
        max_ms: u64,
    },
    /// 指数增长
    Exponential {
        /// 基础延迟（毫秒）
        base_ms: u64,
        /// 最大延迟（毫秒）
        max_ms: u64,
    },
    /// 无延迟（立即重试）
    None,
}

impl BackoffStrategy {
    /// 计算第 n 次重试的延迟（毫秒）
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        match self {
            BackoffStrategy::Fixed { delay_ms } => *delay_ms,
            BackoffStrategy::Linear {
                initial_ms,
                increment_ms,
                max_ms,
            } => {
                let delay = initial_ms + (attempt as u64 * increment_ms);
                delay.min(*max_ms)
            }
            BackoffStrategy::Exponential { base_ms, max_ms } => {
                let delay = base_ms.saturating_mul(2u64.saturating_pow(attempt));
                delay.min(*max_ms)
            }
            BackoffStrategy::None => 0,
        }
    }

    /// 添加抖动
    pub fn with_jitter(&self, delay_ms: u64) -> u64 {
        if delay_ms == 0 {
            return 0;
        }
        // 添加 0-25% 的随机抖动
        let jitter = (delay_ms as f64 * 0.25 * rand::random::<f64>()) as u64;
        delay_ms + jitter
    }
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        BackoffStrategy::Exponential {
            base_ms: 100,
            max_ms: 10000,
        }
    }
}

// ============================================================================
// 重试条件
// ============================================================================

/// 重试条件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryCondition {
    /// 所有错误都重试
    All,
    /// 只重试 IO 错误
    IoOnly,
    /// 只重试超时错误
    TimeoutOnly,
    /// 永不重试
    Never,
}

impl RetryCondition {
    /// 判断错误是否应该重试
    pub fn should_retry(&self, error: &StorageError) -> bool {
        match self {
            RetryCondition::All => !matches!(error, StorageError::NotFound(_)),
            RetryCondition::IoOnly => matches!(error, StorageError::Io(_)),
            RetryCondition::TimeoutOnly => {
                if let StorageError::Io(io_err) = error {
                    io_err.kind() == std::io::ErrorKind::TimedOut
                } else {
                    false
                }
            }
            RetryCondition::Never => false,
        }
    }
}

impl Default for RetryCondition {
    fn default() -> Self {
        RetryCondition::All
    }
}

// ============================================================================
// 重试统计
// ============================================================================

/// 重试统计信息
#[derive(Debug, Default)]
pub struct RetryStats {
    /// 总操作数
    pub total_operations: AtomicU64,
    /// 成功操作数（无需重试）
    pub immediate_successes: AtomicU64,
    /// 重试后成功数
    pub retry_successes: AtomicU64,
    /// 最终失败数（用尽重试次数）
    pub final_failures: AtomicU64,
    /// 总重试次数
    pub total_retries: AtomicU64,
    /// 跳过重试的错误数（不满足重试条件）
    pub skipped_retries: AtomicU64,
}

impl RetryStats {
    /// 获取快照
    pub fn snapshot(&self) -> RetryStatsSnapshot {
        RetryStatsSnapshot {
            total_operations: self.total_operations.load(Ordering::Relaxed),
            immediate_successes: self.immediate_successes.load(Ordering::Relaxed),
            retry_successes: self.retry_successes.load(Ordering::Relaxed),
            final_failures: self.final_failures.load(Ordering::Relaxed),
            total_retries: self.total_retries.load(Ordering::Relaxed),
            skipped_retries: self.skipped_retries.load(Ordering::Relaxed),
        }
    }
}

/// 重试统计快照
#[derive(Debug, Clone)]
pub struct RetryStatsSnapshot {
    /// 总操作数
    pub total_operations: u64,
    /// 成功操作数（无需重试）
    pub immediate_successes: u64,
    /// 重试后成功数
    pub retry_successes: u64,
    /// 最终失败数
    pub final_failures: u64,
    /// 总重试次数
    pub total_retries: u64,
    /// 跳过重试的错误数
    pub skipped_retries: u64,
}

impl RetryStatsSnapshot {
    /// 成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            1.0
        } else {
            (self.immediate_successes + self.retry_successes) as f64 / self.total_operations as f64
        }
    }

    /// 重试率
    pub fn retry_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.total_retries as f64 / self.total_operations as f64
        }
    }

    /// 平均重试次数（对于需要重试的操作）
    pub fn avg_retries_per_retry_operation(&self) -> f64 {
        let retry_ops = self.retry_successes + self.final_failures;
        if retry_ops == 0 {
            0.0
        } else {
            self.total_retries as f64 / retry_ops as f64
        }
    }
}

// ============================================================================
// 重试存储配置
// ============================================================================

/// 重试存储配置
#[derive(Debug, Clone)]
pub struct RetryStorageConfig {
    /// 最大重试次数
    pub max_retries: u32,
    /// 退避策略
    pub backoff: BackoffStrategy,
    /// 是否添加抖动
    pub jitter: bool,
    /// 重试条件
    pub condition: RetryCondition,
}

impl Default for RetryStorageConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff: BackoffStrategy::default(),
            jitter: true,
            condition: RetryCondition::default(),
        }
    }
}

// ============================================================================
// 重试存储
// ============================================================================

/// 重试存储
///
/// 自动重试失败的操作
pub struct RetryStorage<B: StorageBackend> {
    /// 后端存储
    backend: Arc<B>,
    /// 配置
    config: RetryStorageConfig,
    /// 统计信息
    stats: Arc<RetryStats>,
}

impl<B: StorageBackend> RetryStorage<B> {
    /// 创建重试存储
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, RetryStorageConfig::default())
    }

    /// 使用配置创建重试存储
    pub fn with_config(backend: B, config: RetryStorageConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
            stats: Arc::new(RetryStats::default()),
        }
    }

    /// 创建构建器
    pub fn builder(backend: B) -> RetryStorageBuilder<B> {
        RetryStorageBuilder::new(backend)
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> RetryStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取详细统计信息
    pub fn detailed_stats(&self) -> DetailedRetryStats {
        let snapshot = self.stats.snapshot();
        DetailedRetryStats {
            total_operations: snapshot.total_operations,
            immediate_successes: snapshot.immediate_successes,
            retry_successes: snapshot.retry_successes,
            final_failures: snapshot.final_failures,
            total_retries: snapshot.total_retries,
            skipped_retries: snapshot.skipped_retries,
            success_rate: snapshot.success_rate(),
            retry_rate: snapshot.retry_rate(),
            avg_retries: snapshot.avg_retries_per_retry_operation(),
            max_retries: self.config.max_retries,
            jitter_enabled: self.config.jitter,
        }
    }

    /// 执行带重试的操作
    async fn with_retry<F, Fut, T>(&self, operation: F) -> StorageResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = StorageResult<T>>,
    {
        self.stats.total_operations.fetch_add(1, Ordering::Relaxed);

        let mut attempts = 0;
        loop {
            match operation().await {
                Ok(result) => {
                    if attempts == 0 {
                        self.stats.immediate_successes.fetch_add(1, Ordering::Relaxed);
                    } else {
                        self.stats.retry_successes.fetch_add(1, Ordering::Relaxed);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    // 检查是否应该重试
                    if !self.config.condition.should_retry(&e) {
                        self.stats.skipped_retries.fetch_add(1, Ordering::Relaxed);
                        self.stats.final_failures.fetch_add(1, Ordering::Relaxed);
                        return Err(e);
                    }

                    attempts += 1;

                    // 检查是否超过最大重试次数
                    if attempts > self.config.max_retries {
                        self.stats.final_failures.fetch_add(1, Ordering::Relaxed);
                        return Err(e);
                    }

                    self.stats.total_retries.fetch_add(1, Ordering::Relaxed);

                    // 计算延迟
                    let delay_ms = self.config.backoff.delay_for_attempt(attempts - 1);
                    let delay_ms = if self.config.jitter {
                        self.config.backoff.with_jitter(delay_ms)
                    } else {
                        delay_ms
                    };

                    if delay_ms > 0 {
                        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    }
                }
            }
        }
    }
}

/// 详细重试统计
#[derive(Debug, Clone)]
pub struct DetailedRetryStats {
    /// 总操作数
    pub total_operations: u64,
    /// 成功操作数（无需重试）
    pub immediate_successes: u64,
    /// 重试后成功数
    pub retry_successes: u64,
    /// 最终失败数
    pub final_failures: u64,
    /// 总重试次数
    pub total_retries: u64,
    /// 跳过重试的错误数
    pub skipped_retries: u64,
    /// 成功率
    pub success_rate: f64,
    /// 重试率
    pub retry_rate: f64,
    /// 平均重试次数
    pub avg_retries: f64,
    /// 最大重试次数配置
    pub max_retries: u32,
    /// 是否启用抖动
    pub jitter_enabled: bool,
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend + 'static> StorageBackend for RetryStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.with_retry(|| {
            let backend = Arc::clone(&backend);
            let key = key.clone();
            async move { backend.read(&key).await }
        })
        .await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        let data = data.to_vec();
        self.with_retry(|| {
            let backend = Arc::clone(&backend);
            let key = key.clone();
            let data = data.clone();
            async move { backend.write(&key, &data).await }
        })
        .await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.with_retry(|| {
            let backend = Arc::clone(&backend);
            let key = key.clone();
            async move { backend.delete(&key).await }
        })
        .await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let backend = Arc::clone(&self.backend);
        let prefix = prefix.to_string();
        self.with_retry(|| {
            let backend = Arc::clone(&backend);
            let prefix = prefix.clone();
            async move { backend.list(&prefix).await }
        })
        .await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.with_retry(|| {
            let backend = Arc::clone(&backend);
            let key = key.clone();
            async move { backend.exists(&key).await }
        })
        .await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "RetryStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// 重试存储构建器
pub struct RetryStorageBuilder<B: StorageBackend> {
    backend: B,
    config: RetryStorageConfig,
}

impl<B: StorageBackend> RetryStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            config: RetryStorageConfig::default(),
        }
    }

    /// 设置最大重试次数
    pub fn max_retries(mut self, max: u32) -> Self {
        self.config.max_retries = max;
        self
    }

    /// 设置退避策略
    pub fn backoff(mut self, strategy: BackoffStrategy) -> Self {
        self.config.backoff = strategy;
        self
    }

    /// 设置固定延迟退避
    pub fn fixed_backoff(mut self, delay_ms: u64) -> Self {
        self.config.backoff = BackoffStrategy::Fixed { delay_ms };
        self
    }

    /// 设置线性退避
    pub fn linear_backoff(mut self, initial_ms: u64, increment_ms: u64, max_ms: u64) -> Self {
        self.config.backoff = BackoffStrategy::Linear {
            initial_ms,
            increment_ms,
            max_ms,
        };
        self
    }

    /// 设置指数退避
    pub fn exponential_backoff(mut self, base_ms: u64, max_ms: u64) -> Self {
        self.config.backoff = BackoffStrategy::Exponential { base_ms, max_ms };
        self
    }

    /// 设置是否启用抖动
    pub fn with_jitter(mut self, enabled: bool) -> Self {
        self.config.jitter = enabled;
        self
    }

    /// 设置重试条件
    pub fn condition(mut self, condition: RetryCondition) -> Self {
        self.config.condition = condition;
        self
    }

    /// 构建重试存储
    pub fn build(self) -> RetryStorage<B> {
        RetryStorage::with_config(self.backend, self.config)
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
    fn test_backoff_strategy_fixed() {
        let strategy = BackoffStrategy::Fixed { delay_ms: 100 };
        assert_eq!(strategy.delay_for_attempt(0), 100);
        assert_eq!(strategy.delay_for_attempt(1), 100);
        assert_eq!(strategy.delay_for_attempt(5), 100);
    }

    #[test]
    fn test_backoff_strategy_linear() {
        let strategy = BackoffStrategy::Linear {
            initial_ms: 100,
            increment_ms: 50,
            max_ms: 300,
        };
        assert_eq!(strategy.delay_for_attempt(0), 100);
        assert_eq!(strategy.delay_for_attempt(1), 150);
        assert_eq!(strategy.delay_for_attempt(2), 200);
        assert_eq!(strategy.delay_for_attempt(10), 300); // capped at max
    }

    #[test]
    fn test_backoff_strategy_exponential() {
        let strategy = BackoffStrategy::Exponential {
            base_ms: 100,
            max_ms: 1000,
        };
        assert_eq!(strategy.delay_for_attempt(0), 100);
        assert_eq!(strategy.delay_for_attempt(1), 200);
        assert_eq!(strategy.delay_for_attempt(2), 400);
        assert_eq!(strategy.delay_for_attempt(3), 800);
        assert_eq!(strategy.delay_for_attempt(4), 1000); // capped at max
    }

    #[test]
    fn test_backoff_strategy_none() {
        let strategy = BackoffStrategy::None;
        assert_eq!(strategy.delay_for_attempt(0), 0);
        assert_eq!(strategy.delay_for_attempt(10), 0);
    }

    #[test]
    fn test_backoff_with_jitter() {
        let strategy = BackoffStrategy::Fixed { delay_ms: 100 };

        // 多次调用，验证抖动在范围内
        for _ in 0..100 {
            let delay = strategy.with_jitter(100);
            assert!(delay >= 100 && delay <= 125);
        }
    }

    #[test]
    fn test_retry_condition_all() {
        let condition = RetryCondition::All;
        assert!(condition.should_retry(&StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test"
        ))));
        assert!(condition.should_retry(&StorageError::Other("test".to_string())));
        // NotFound 不重试
        assert!(!condition.should_retry(&StorageError::NotFound("key".to_string())));
    }

    #[test]
    fn test_retry_condition_io_only() {
        let condition = RetryCondition::IoOnly;
        assert!(condition.should_retry(&StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test"
        ))));
        assert!(!condition.should_retry(&StorageError::Other("test".to_string())));
    }

    #[test]
    fn test_retry_condition_never() {
        let condition = RetryCondition::Never;
        assert!(!condition.should_retry(&StorageError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "test"
        ))));
        assert!(!condition.should_retry(&StorageError::Other("test".to_string())));
    }

    #[tokio::test]
    async fn test_retry_storage_new() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        assert_eq!(retry.name(), "RetryStorage");
    }

    #[tokio::test]
    async fn test_retry_storage_write_read() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        retry.write("key1", b"value1").await.unwrap();
        let data = retry.read("key1").await.unwrap();

        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_retry_storage_delete() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        retry.write("key1", b"value1").await.unwrap();
        assert!(retry.exists("key1").await.unwrap());

        retry.delete("key1").await.unwrap();
        assert!(!retry.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_retry_storage_list() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        retry.write("test:a", b"1").await.unwrap();
        retry.write("test:b", b"2").await.unwrap();
        retry.write("other:c", b"3").await.unwrap();

        let keys = retry.list("test:").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_retry_storage_stats() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        retry.write("key1", b"value1").await.unwrap();
        retry.read("key1").await.unwrap();

        let stats = retry.stats_snapshot();
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.immediate_successes, 2);
        assert_eq!(stats.total_retries, 0);
    }

    #[tokio::test]
    async fn test_retry_storage_detailed_stats() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        let stats = retry.detailed_stats();
        assert_eq!(stats.max_retries, 3);
        assert!(stats.jitter_enabled);
    }

    #[tokio::test]
    async fn test_retry_storage_builder() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::builder(storage)
            .max_retries(5)
            .exponential_backoff(50, 2000)
            .with_jitter(false)
            .condition(RetryCondition::IoOnly)
            .build();

        retry.write("key1", b"value1").await.unwrap();
        assert_eq!(retry.read("key1").await.unwrap(), b"value1");
    }

    #[tokio::test]
    async fn test_retry_storage_builder_fixed_backoff() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::builder(storage)
            .fixed_backoff(100)
            .build();

        retry.write("key1", b"value1").await.unwrap();
    }

    #[tokio::test]
    async fn test_retry_storage_builder_linear_backoff() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::builder(storage)
            .linear_backoff(50, 25, 200)
            .build();

        retry.write("key1", b"value1").await.unwrap();
    }

    #[test]
    fn test_retry_stats_snapshot() {
        let stats = RetryStats::default();
        stats.total_operations.store(100, Ordering::Relaxed);
        stats.immediate_successes.store(80, Ordering::Relaxed);
        stats.retry_successes.store(15, Ordering::Relaxed);
        stats.final_failures.store(5, Ordering::Relaxed);
        stats.total_retries.store(30, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_operations, 100);
        assert!((snapshot.success_rate() - 0.95).abs() < 0.001);
        assert!((snapshot.retry_rate() - 0.3).abs() < 0.001);
    }

    #[test]
    fn test_avg_retries_per_retry_operation() {
        let snapshot = RetryStatsSnapshot {
            total_operations: 100,
            immediate_successes: 80,
            retry_successes: 15,
            final_failures: 5,
            total_retries: 40,
            skipped_retries: 0,
        };

        // 20 operations needed retries, 40 total retries = 2.0 avg
        assert!((snapshot.avg_retries_per_retry_operation() - 2.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_retry_storage_not_found_no_retry() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        // NotFound 错误不应该重试
        let result = retry.read("nonexistent").await;
        assert!(result.is_err());

        let stats = retry.stats_snapshot();
        assert_eq!(stats.total_operations, 1);
        assert_eq!(stats.skipped_retries, 1);
        assert_eq!(stats.total_retries, 0);
    }

    #[test]
    fn test_backoff_strategy_default() {
        let strategy = BackoffStrategy::default();
        match strategy {
            BackoffStrategy::Exponential { base_ms, max_ms } => {
                assert_eq!(base_ms, 100);
                assert_eq!(max_ms, 10000);
            }
            _ => panic!("Expected Exponential strategy"),
        }
    }

    #[test]
    fn test_retry_condition_default() {
        assert_eq!(RetryCondition::default(), RetryCondition::All);
    }

    #[tokio::test]
    async fn test_retry_storage_exists() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        assert!(!retry.exists("key1").await.unwrap());

        retry.write("key1", b"value1").await.unwrap();
        assert!(retry.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_retry_storage_multiple_operations() {
        let storage = MemoryStorage::new();
        let retry = RetryStorage::new(storage);

        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            retry.write(&key, value.as_bytes()).await.unwrap();
        }

        for i in 0..10 {
            let key = format!("key{}", i);
            let expected = format!("value{}", i);
            let data = retry.read(&key).await.unwrap();
            assert_eq!(data, expected.as_bytes());
        }

        let stats = retry.stats_snapshot();
        assert_eq!(stats.total_operations, 20);
        assert_eq!(stats.immediate_successes, 20);
    }
}
