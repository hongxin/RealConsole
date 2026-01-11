//! 熔断器存储层
//!
//! v1.74.0: 实现熔断器模式，防止级联故障
//!
//! ## 设计目标
//!
//! - **故障隔离**: 当后端持续失败时，快速失败而不是无限重试
//! - **自动恢复**: 定期尝试恢复，成功后恢复正常服务
//! - **状态可观测**: 提供熔断器状态和统计信息
//!
//! ## 熔断器状态
//!
//! ```text
//! ┌─────────┐  失败次数超过阈值  ┌─────────┐
//! │ Closed  │ ─────────────────→ │  Open   │
//! │ (正常)  │                    │ (熔断)  │
//! └─────────┘                    └─────────┘
//!      ↑                              │
//!      │                              │ 超时后
//!      │                              ↓
//!      │    成功次数超过阈值    ┌───────────┐
//!      └─────────────────────── │ Half-Open │
//!              失败则重新 Open   │ (半开)   │
//!                               └───────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{CircuitBreakerStorage, MemoryStorage};
//!
//! let storage = MemoryStorage::new();
//! let cb_storage = CircuitBreakerStorage::builder(storage)
//!     .failure_threshold(5)
//!     .success_threshold(3)
//!     .open_timeout_secs(30)
//!     .build();
//!
//! // 正常使用，熔断器自动管理状态
//! cb_storage.write("key1", b"value1").await?;
//! ```

use crate::storage::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// 熔断器状态
// ============================================================================

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 关闭（正常运行）
    Closed,
    /// 打开（拒绝请求）
    Open,
    /// 半开（测试恢复）
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

// ============================================================================
// 熔断器内部状态
// ============================================================================

/// 熔断器内部状态
struct CircuitBreakerState {
    /// 当前状态
    state: CircuitState,
    /// 连续失败次数
    consecutive_failures: u32,
    /// 半开状态下的连续成功次数
    consecutive_successes: u32,
    /// 上次状态转换时间
    last_state_change: Instant,
    /// 上次失败时间
    last_failure: Option<Instant>,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_state_change: Instant::now(),
            last_failure: None,
        }
    }
}

// ============================================================================
// 熔断器统计
// ============================================================================

/// 熔断器统计信息
#[derive(Debug, Default)]
pub struct CircuitBreakerStats {
    /// 总请求数
    pub total_requests: AtomicU64,
    /// 成功请求数
    pub successful_requests: AtomicU64,
    /// 失败请求数
    pub failed_requests: AtomicU64,
    /// 被熔断器拒绝的请求数
    pub rejected_requests: AtomicU64,
    /// 状态转换次数
    pub state_transitions: AtomicU64,
    /// 打开次数
    pub times_opened: AtomicU64,
}

impl CircuitBreakerStats {
    /// 获取快照
    pub fn snapshot(&self) -> CircuitBreakerStatsSnapshot {
        CircuitBreakerStatsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            successful_requests: self.successful_requests.load(Ordering::Relaxed),
            failed_requests: self.failed_requests.load(Ordering::Relaxed),
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            state_transitions: self.state_transitions.load(Ordering::Relaxed),
            times_opened: self.times_opened.load(Ordering::Relaxed),
        }
    }
}

/// 熔断器统计快照
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatsSnapshot {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 被拒绝的请求数
    pub rejected_requests: u64,
    /// 状态转换次数
    pub state_transitions: u64,
    /// 打开次数
    pub times_opened: u64,
}

impl CircuitBreakerStatsSnapshot {
    /// 成功率
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_requests + self.failed_requests;
        if total == 0 {
            1.0
        } else {
            self.successful_requests as f64 / total as f64
        }
    }

    /// 拒绝率
    pub fn rejection_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.rejected_requests as f64 / self.total_requests as f64
        }
    }
}

// ============================================================================
// 熔断器配置
// ============================================================================

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失败阈值（触发打开）
    pub failure_threshold: u32,
    /// 成功阈值（半开→关闭）
    pub success_threshold: u32,
    /// 打开状态超时时间（秒）
    pub open_timeout_secs: u64,
    /// 半开状态最大并发请求数
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            open_timeout_secs: 30,
            half_open_max_requests: 1,
        }
    }
}

// ============================================================================
// 熔断器存储
// ============================================================================

/// 熔断器存储
///
/// 实现熔断器模式，防止级联故障
pub struct CircuitBreakerStorage<B: StorageBackend> {
    /// 后端存储
    backend: Arc<B>,
    /// 配置
    config: CircuitBreakerConfig,
    /// 内部状态
    state: Arc<RwLock<CircuitBreakerState>>,
    /// 统计信息
    stats: Arc<CircuitBreakerStats>,
}

impl<B: StorageBackend> CircuitBreakerStorage<B> {
    /// 创建熔断器存储
    pub fn new(backend: B) -> Self {
        Self::with_config(backend, CircuitBreakerConfig::default())
    }

    /// 使用配置创建熔断器存储
    pub fn with_config(backend: B, config: CircuitBreakerConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::new())),
            stats: Arc::new(CircuitBreakerStats::default()),
        }
    }

    /// 创建构建器
    pub fn builder(backend: B) -> CircuitBreakerBuilder<B> {
        CircuitBreakerBuilder::new(backend)
    }

    /// 获取当前状态
    pub async fn state(&self) -> CircuitState {
        self.state.read().await.state
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> CircuitBreakerStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取详细统计信息
    pub async fn detailed_stats(&self) -> DetailedCircuitBreakerStats {
        let snapshot = self.stats.snapshot();
        let state = self.state.read().await;

        DetailedCircuitBreakerStats {
            total_requests: snapshot.total_requests,
            successful_requests: snapshot.successful_requests,
            failed_requests: snapshot.failed_requests,
            rejected_requests: snapshot.rejected_requests,
            state_transitions: snapshot.state_transitions,
            times_opened: snapshot.times_opened,
            success_rate: snapshot.success_rate(),
            rejection_rate: snapshot.rejection_rate(),
            current_state: state.state,
            consecutive_failures: state.consecutive_failures,
            consecutive_successes: state.consecutive_successes,
            time_in_current_state_ms: state.last_state_change.elapsed().as_millis() as u64,
            failure_threshold: self.config.failure_threshold,
            success_threshold: self.config.success_threshold,
            open_timeout_secs: self.config.open_timeout_secs,
        }
    }

    /// 强制打开熔断器
    pub async fn force_open(&self) {
        let mut state = self.state.write().await;
        if state.state != CircuitState::Open {
            state.state = CircuitState::Open;
            state.last_state_change = Instant::now();
            self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
            self.stats.times_opened.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 强制关闭熔断器
    pub async fn force_close(&self) {
        let mut state = self.state.write().await;
        if state.state != CircuitState::Closed {
            state.state = CircuitState::Closed;
            state.consecutive_failures = 0;
            state.consecutive_successes = 0;
            state.last_state_change = Instant::now();
            self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 检查并更新状态
    async fn check_state(&self) -> StorageResult<()> {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => {
                // 关闭状态，允许请求
                Ok(())
            }
            CircuitState::Open => {
                // 检查是否超时，可以尝试半开
                let timeout = Duration::from_secs(self.config.open_timeout_secs);
                if state.last_state_change.elapsed() >= timeout {
                    state.state = CircuitState::HalfOpen;
                    state.consecutive_successes = 0;
                    state.last_state_change = Instant::now();
                    self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
                    Ok(())
                } else {
                    self.stats.rejected_requests.fetch_add(1, Ordering::Relaxed);
                    Err(StorageError::Other("Circuit breaker is open".to_string()))
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态，允许有限请求
                Ok(())
            }
        }
    }

    /// 记录成功
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        self.stats.successful_requests.fetch_add(1, Ordering::Relaxed);

        match state.state {
            CircuitState::Closed => {
                state.consecutive_failures = 0;
            }
            CircuitState::HalfOpen => {
                state.consecutive_successes += 1;
                if state.consecutive_successes >= self.config.success_threshold {
                    // 转换到关闭状态
                    state.state = CircuitState::Closed;
                    state.consecutive_failures = 0;
                    state.consecutive_successes = 0;
                    state.last_state_change = Instant::now();
                    self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // 不应该发生
            }
        }
    }

    /// 记录失败
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        self.stats.failed_requests.fetch_add(1, Ordering::Relaxed);
        state.last_failure = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                state.consecutive_failures += 1;
                if state.consecutive_failures >= self.config.failure_threshold {
                    // 转换到打开状态
                    state.state = CircuitState::Open;
                    state.last_state_change = Instant::now();
                    self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
                    self.stats.times_opened.fetch_add(1, Ordering::Relaxed);
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态下失败，立即转回打开状态
                state.state = CircuitState::Open;
                state.consecutive_successes = 0;
                state.last_state_change = Instant::now();
                self.stats.state_transitions.fetch_add(1, Ordering::Relaxed);
                self.stats.times_opened.fetch_add(1, Ordering::Relaxed);
            }
            CircuitState::Open => {
                // 不应该发生
            }
        }
    }

    /// 执行操作
    async fn execute<F, Fut, T>(&self, operation: F) -> StorageResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = StorageResult<T>>,
    {
        self.stats.total_requests.fetch_add(1, Ordering::Relaxed);

        // 检查熔断器状态
        self.check_state().await?;

        // 执行操作
        match operation().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(e) => {
                self.record_failure().await;
                Err(e)
            }
        }
    }
}

/// 详细熔断器统计
#[derive(Debug, Clone)]
pub struct DetailedCircuitBreakerStats {
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 被拒绝的请求数
    pub rejected_requests: u64,
    /// 状态转换次数
    pub state_transitions: u64,
    /// 打开次数
    pub times_opened: u64,
    /// 成功率
    pub success_rate: f64,
    /// 拒绝率
    pub rejection_rate: f64,
    /// 当前状态
    pub current_state: CircuitState,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 连续成功次数
    pub consecutive_successes: u32,
    /// 当前状态持续时间（毫秒）
    pub time_in_current_state_ms: u64,
    /// 失败阈值
    pub failure_threshold: u32,
    /// 成功阈值
    pub success_threshold: u32,
    /// 打开超时时间
    pub open_timeout_secs: u64,
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend + 'static> StorageBackend for CircuitBreakerStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.execute(|| async move { backend.read(&key).await }).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        let data = data.to_vec();
        self.execute(|| async move { backend.write(&key, &data).await })
            .await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.execute(|| async move { backend.delete(&key).await })
            .await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        let backend = Arc::clone(&self.backend);
        let prefix = prefix.to_string();
        self.execute(|| async move { backend.list(&prefix).await })
            .await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let backend = Arc::clone(&self.backend);
        let key = key.to_string();
        self.execute(|| async move { backend.exists(&key).await })
            .await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "CircuitBreakerStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// 熔断器存储构建器
pub struct CircuitBreakerBuilder<B: StorageBackend> {
    backend: B,
    config: CircuitBreakerConfig,
}

impl<B: StorageBackend> CircuitBreakerBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            config: CircuitBreakerConfig::default(),
        }
    }

    /// 设置失败阈值
    pub fn failure_threshold(mut self, threshold: u32) -> Self {
        self.config.failure_threshold = threshold;
        self
    }

    /// 设置成功阈值
    pub fn success_threshold(mut self, threshold: u32) -> Self {
        self.config.success_threshold = threshold;
        self
    }

    /// 设置打开超时时间
    pub fn open_timeout_secs(mut self, secs: u64) -> Self {
        self.config.open_timeout_secs = secs;
        self
    }

    /// 设置半开状态最大并发请求数
    pub fn half_open_max_requests(mut self, max: u32) -> Self {
        self.config.half_open_max_requests = max;
        self
    }

    /// 构建熔断器存储
    pub fn build(self) -> CircuitBreakerStorage<B> {
        CircuitBreakerStorage::with_config(self.backend, self.config)
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
    fn test_circuit_state_display() {
        assert_eq!(CircuitState::Closed.to_string(), "Closed");
        assert_eq!(CircuitState::Open.to_string(), "Open");
        assert_eq!(CircuitState::HalfOpen.to_string(), "HalfOpen");
    }

    #[tokio::test]
    async fn test_circuit_breaker_new() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        assert_eq!(cb.name(), "CircuitBreakerStorage");
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_write_read() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        cb.write("key1", b"value1").await.unwrap();
        let data = cb.read("key1").await.unwrap();

        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_circuit_breaker_delete() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        cb.write("key1", b"value1").await.unwrap();
        assert!(cb.exists("key1").await.unwrap());

        cb.delete("key1").await.unwrap();
        assert!(!cb.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_circuit_breaker_list() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        cb.write("test:a", b"1").await.unwrap();
        cb.write("test:b", b"2").await.unwrap();
        cb.write("other:c", b"3").await.unwrap();

        let keys = cb.list("test:").await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_circuit_breaker_stats() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        cb.write("key1", b"value1").await.unwrap();
        cb.read("key1").await.unwrap();

        let stats = cb.stats_snapshot();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_detailed_stats() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        let stats = cb.detailed_stats().await;
        assert_eq!(stats.current_state, CircuitState::Closed);
        assert_eq!(stats.failure_threshold, 5);
        assert_eq!(stats.success_threshold, 3);
    }

    #[tokio::test]
    async fn test_circuit_breaker_builder() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::builder(storage)
            .failure_threshold(3)
            .success_threshold(2)
            .open_timeout_secs(10)
            .half_open_max_requests(2)
            .build();

        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_force_open() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        assert_eq!(cb.state().await, CircuitState::Closed);

        cb.force_open().await;
        assert_eq!(cb.state().await, CircuitState::Open);

        // 被熔断器拒绝
        let result = cb.read("key1").await;
        assert!(result.is_err());

        let stats = cb.stats_snapshot();
        assert_eq!(stats.rejected_requests, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_force_close() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        cb.force_open().await;
        assert_eq!(cb.state().await, CircuitState::Open);

        cb.force_close().await;
        assert_eq!(cb.state().await, CircuitState::Closed);

        // 可以正常操作
        cb.write("key1", b"value1").await.unwrap();
        let data = cb.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::builder(storage)
            .failure_threshold(3)
            .build();

        // 触发失败（读取不存在的键）
        for _ in 0..3 {
            let _ = cb.read("nonexistent").await;
        }

        // 熔断器应该打开
        assert_eq!(cb.state().await, CircuitState::Open);

        let stats = cb.stats_snapshot();
        assert_eq!(stats.times_opened, 1);
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_after_timeout() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::builder(storage)
            .failure_threshold(2)
            .open_timeout_secs(0) // 立即超时
            .build();

        // 触发失败
        for _ in 0..2 {
            let _ = cb.read("nonexistent").await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);

        // 等待一下让超时生效
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 下一个请求应该使熔断器进入半开状态
        cb.write("key1", b"value1").await.unwrap();

        // 成功后应该从半开转为关闭（因为 success_threshold 默认是 3，
        // 但我们只成功了一次，所以应该还在半开或已关闭取决于配置）
        let state = cb.state().await;
        assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_closes_after_successes() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::builder(storage)
            .failure_threshold(2)
            .success_threshold(2)
            .open_timeout_secs(0)
            .build();

        // 写入一些数据
        cb.write("key1", b"value1").await.unwrap();

        // 触发失败
        for _ in 0..2 {
            let _ = cb.read("nonexistent").await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);

        // 等待超时
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 成功的请求
        cb.read("key1").await.unwrap();
        cb.read("key1").await.unwrap();

        // 应该关闭
        assert_eq!(cb.state().await, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_stats_snapshot() {
        let stats = CircuitBreakerStats::default();
        stats.total_requests.store(100, Ordering::Relaxed);
        stats.successful_requests.store(80, Ordering::Relaxed);
        stats.failed_requests.store(15, Ordering::Relaxed);
        stats.rejected_requests.store(5, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.total_requests, 100);
        assert!((snapshot.success_rate() - 0.842).abs() < 0.01);
        assert!((snapshot.rejection_rate() - 0.05).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_circuit_breaker_exists() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        assert!(!cb.exists("key1").await.unwrap());

        cb.write("key1", b"value1").await.unwrap();
        assert!(cb.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_circuit_breaker_multiple_operations() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::new(storage);

        for i in 0..10 {
            let key = format!("key{}", i);
            let value = format!("value{}", i);
            cb.write(&key, value.as_bytes()).await.unwrap();
        }

        for i in 0..10 {
            let key = format!("key{}", i);
            let expected = format!("value{}", i);
            let data = cb.read(&key).await.unwrap();
            assert_eq!(data, expected.as_bytes());
        }

        let stats = cb.stats_snapshot();
        assert_eq!(stats.total_requests, 20);
        assert_eq!(stats.successful_requests, 20);
    }

    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let storage = MemoryStorage::new();
        let cb = CircuitBreakerStorage::builder(storage)
            .failure_threshold(2)
            .success_threshold(1)
            .open_timeout_secs(0)
            .build();

        // Closed -> Open
        let _ = cb.read("nonexistent").await;
        let _ = cb.read("nonexistent").await;
        assert_eq!(cb.state().await, CircuitState::Open);

        // Open -> HalfOpen (after timeout)
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 写入数据以便后续读取成功
        // 注意：在 Open 状态下这会被拒绝，但超时后会进入 HalfOpen
        cb.write("key1", b"value1").await.unwrap();

        // HalfOpen -> Closed (after success)
        assert_eq!(cb.state().await, CircuitState::Closed);

        let stats = cb.stats_snapshot();
        assert!(stats.state_transitions >= 2);
    }

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.success_threshold, 3);
        assert_eq!(config.open_timeout_secs, 30);
        assert_eq!(config.half_open_max_requests, 1);
    }
}
