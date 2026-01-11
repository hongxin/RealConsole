//! 存储可观测性层
//!
//! v1.68.0: v2.0 探路期收尾 - 存储指标与监控
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的可观测性架构：
//! - **采集层**: 原子操作收集指标
//! - **计算层**: 百分位数、吞吐量计算
//! - **展示层**: 格式化输出、导出
//!
//! ## 指标类型
//!
//! ```text
//! ┌───────────────────────────────────────────────────────┐
//! │                   StorageMetrics                      │
//! ├───────────────────────────────────────────────────────┤
//! │                                                       │
//! │  延迟指标 (Latency):                                  │
//! │    - 读取延迟: p50, p95, p99, max                    │
//! │    - 写入延迟: p50, p95, p99, max                    │
//! │    - 删除延迟: p50, p95, p99, max                    │
//! │                                                       │
//! │  吞吐量指标 (Throughput):                             │
//! │    - 操作数/秒 (ops/sec)                             │
//! │    - 字节数/秒 (bytes/sec)                           │
//! │                                                       │
//! │  错误指标 (Errors):                                   │
//! │    - 读取错误数                                       │
//! │    - 写入错误数                                       │
//! │    - 总错误率                                         │
//! │                                                       │
//! └───────────────────────────────────────────────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{MetricsStorage, FileStorage};
//!
//! let backend = FileStorage::new("/path/to/data");
//! let storage = MetricsStorage::new(backend);
//!
//! // 执行操作
//! storage.write("key1", b"data").await?;
//! storage.read("key1").await?;
//!
//! // 查看指标
//! let report = storage.metrics_report();
//! println!("{}", report);
//!
//! // 获取详细指标
//! let metrics = storage.detailed_metrics();
//! println!("Read p99: {:?}", metrics.read_latency.p99());
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// 延迟样本数量（用于百分位计算）
const LATENCY_SAMPLE_SIZE: usize = 1000;

/// 延迟直方图桶边界（微秒）
const LATENCY_BUCKETS: [u64; 12] = [
    10,      // 10µs
    50,      // 50µs
    100,     // 100µs
    500,     // 500µs
    1000,    // 1ms
    5000,    // 5ms
    10000,   // 10ms
    50000,   // 50ms
    100000,  // 100ms
    500000,  // 500ms
    1000000, // 1s
    u64::MAX,
];

/// 延迟追踪器
#[derive(Debug)]
pub struct LatencyTracker {
    /// 样本缓冲区（微秒）
    samples: RwLock<Vec<u64>>,
    /// 样本索引（循环写入）
    sample_index: AtomicU64,
    /// 总操作数
    count: AtomicU64,
    /// 总延迟（微秒）
    total_micros: AtomicU64,
    /// 最小延迟（微秒）
    min_micros: AtomicU64,
    /// 最大延迟（微秒）
    max_micros: AtomicU64,
    /// 直方图桶计数
    histogram: [AtomicU64; 12],
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    /// 创建新的延迟追踪器
    pub fn new() -> Self {
        Self {
            samples: RwLock::new(Vec::with_capacity(LATENCY_SAMPLE_SIZE)),
            sample_index: AtomicU64::new(0),
            count: AtomicU64::new(0),
            total_micros: AtomicU64::new(0),
            min_micros: AtomicU64::new(u64::MAX),
            max_micros: AtomicU64::new(0),
            histogram: Default::default(),
        }
    }

    /// 记录延迟
    pub fn record(&self, duration: Duration) {
        let micros = duration.as_micros() as u64;

        // 更新计数和总延迟
        self.count.fetch_add(1, Ordering::Relaxed);
        self.total_micros.fetch_add(micros, Ordering::Relaxed);

        // 更新最小值
        let mut current_min = self.min_micros.load(Ordering::Relaxed);
        while micros < current_min {
            match self.min_micros.compare_exchange_weak(
                current_min,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_min = x,
            }
        }

        // 更新最大值
        let mut current_max = self.max_micros.load(Ordering::Relaxed);
        while micros > current_max {
            match self.max_micros.compare_exchange_weak(
                current_max,
                micros,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(x) => current_max = x,
            }
        }

        // 更新直方图
        for (i, &bucket) in LATENCY_BUCKETS.iter().enumerate() {
            if micros <= bucket {
                self.histogram[i].fetch_add(1, Ordering::Relaxed);
                break;
            }
        }

        // 存储样本（循环缓冲区）
        let index = self.sample_index.fetch_add(1, Ordering::Relaxed) as usize % LATENCY_SAMPLE_SIZE;
        let mut samples = self.samples.write().unwrap();
        if samples.len() <= index {
            samples.push(micros);
        } else {
            samples[index] = micros;
        }
    }

    /// 获取操作计数
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// 获取平均延迟
    pub fn avg(&self) -> Option<Duration> {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return None;
        }
        let avg_micros = self.total_micros.load(Ordering::Relaxed) / count;
        Some(Duration::from_micros(avg_micros))
    }

    /// 获取最小延迟
    pub fn min(&self) -> Option<Duration> {
        let min = self.min_micros.load(Ordering::Relaxed);
        if min == u64::MAX {
            None
        } else {
            Some(Duration::from_micros(min))
        }
    }

    /// 获取最大延迟
    pub fn max(&self) -> Option<Duration> {
        let max = self.max_micros.load(Ordering::Relaxed);
        if max == 0 && self.count.load(Ordering::Relaxed) == 0 {
            None
        } else {
            Some(Duration::from_micros(max))
        }
    }

    /// 获取百分位延迟
    pub fn percentile(&self, p: f64) -> Option<Duration> {
        let samples = self.samples.read().unwrap();
        if samples.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = samples.clone();
        sorted.sort_unstable();

        let index = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        let index = index.min(sorted.len() - 1);
        Some(Duration::from_micros(sorted[index]))
    }

    /// 获取 p50
    pub fn p50(&self) -> Option<Duration> {
        self.percentile(50.0)
    }

    /// 获取 p95
    pub fn p95(&self) -> Option<Duration> {
        self.percentile(95.0)
    }

    /// 获取 p99
    pub fn p99(&self) -> Option<Duration> {
        self.percentile(99.0)
    }

    /// 获取延迟统计快照
    pub fn snapshot(&self) -> LatencySnapshot {
        LatencySnapshot {
            count: self.count(),
            avg: self.avg(),
            min: self.min(),
            max: self.max(),
            p50: self.p50(),
            p95: self.p95(),
            p99: self.p99(),
        }
    }
}

/// 延迟统计快照
#[derive(Debug, Clone)]
pub struct LatencySnapshot {
    /// 操作计数
    pub count: u64,
    /// 平均延迟
    pub avg: Option<Duration>,
    /// 最小延迟
    pub min: Option<Duration>,
    /// 最大延迟
    pub max: Option<Duration>,
    /// p50 延迟
    pub p50: Option<Duration>,
    /// p95 延迟
    pub p95: Option<Duration>,
    /// p99 延迟
    pub p99: Option<Duration>,
}

impl LatencySnapshot {
    /// 格式化为字符串
    pub fn format(&self) -> String {
        if self.count == 0 {
            return "no data".to_string();
        }
        format!(
            "count={}, avg={:?}, p50={:?}, p95={:?}, p99={:?}, max={:?}",
            self.count,
            self.avg.unwrap_or_default(),
            self.p50.unwrap_or_default(),
            self.p95.unwrap_or_default(),
            self.p99.unwrap_or_default(),
            self.max.unwrap_or_default(),
        )
    }
}

/// 吞吐量追踪器
#[derive(Debug, Default)]
pub struct ThroughputTracker {
    /// 总操作数
    operations: AtomicU64,
    /// 总字节数
    bytes: AtomicU64,
    /// 开始时间
    start_time: RwLock<Option<Instant>>,
}

impl ThroughputTracker {
    /// 创建新的吞吐量追踪器
    pub fn new() -> Self {
        Self {
            operations: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            start_time: RwLock::new(None),
        }
    }

    /// 记录操作
    pub fn record(&self, bytes: u64) {
        // 懒初始化开始时间
        {
            let mut start = self.start_time.write().unwrap();
            if start.is_none() {
                *start = Some(Instant::now());
            }
        }

        self.operations.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// 获取操作数/秒
    pub fn ops_per_sec(&self) -> f64 {
        let start = self.start_time.read().unwrap();
        match *start {
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    self.operations.load(Ordering::Relaxed) as f64 / elapsed
                } else {
                    0.0
                }
            }
            None => 0.0,
        }
    }

    /// 获取字节数/秒
    pub fn bytes_per_sec(&self) -> f64 {
        let start = self.start_time.read().unwrap();
        match *start {
            Some(t) => {
                let elapsed = t.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    self.bytes.load(Ordering::Relaxed) as f64 / elapsed
                } else {
                    0.0
                }
            }
            None => 0.0,
        }
    }

    /// 获取总操作数
    pub fn total_operations(&self) -> u64 {
        self.operations.load(Ordering::Relaxed)
    }

    /// 获取总字节数
    pub fn total_bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    /// 获取吞吐量快照
    pub fn snapshot(&self) -> ThroughputSnapshot {
        ThroughputSnapshot {
            total_operations: self.total_operations(),
            total_bytes: self.total_bytes(),
            ops_per_sec: self.ops_per_sec(),
            bytes_per_sec: self.bytes_per_sec(),
        }
    }
}

/// 吞吐量快照
#[derive(Debug, Clone)]
pub struct ThroughputSnapshot {
    /// 总操作数
    pub total_operations: u64,
    /// 总字节数
    pub total_bytes: u64,
    /// 操作数/秒
    pub ops_per_sec: f64,
    /// 字节数/秒
    pub bytes_per_sec: f64,
}

impl ThroughputSnapshot {
    /// 格式化字节数/秒为人类可读格式
    pub fn format_bytes_per_sec(&self) -> String {
        let bps = self.bytes_per_sec;
        if bps >= 1_000_000_000.0 {
            format!("{:.2} GB/s", bps / 1_000_000_000.0)
        } else if bps >= 1_000_000.0 {
            format!("{:.2} MB/s", bps / 1_000_000.0)
        } else if bps >= 1_000.0 {
            format!("{:.2} KB/s", bps / 1_000.0)
        } else {
            format!("{:.2} B/s", bps)
        }
    }
}

/// 错误追踪器
#[derive(Debug, Default)]
pub struct ErrorTracker {
    /// 读取错误数
    read_errors: AtomicU64,
    /// 写入错误数
    write_errors: AtomicU64,
    /// 删除错误数
    delete_errors: AtomicU64,
    /// 其他错误数
    other_errors: AtomicU64,
}

impl ErrorTracker {
    /// 创建新的错误追踪器
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录读取错误
    pub fn record_read_error(&self) {
        self.read_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录写入错误
    pub fn record_write_error(&self) {
        self.write_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录删除错误
    pub fn record_delete_error(&self) {
        self.delete_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录其他错误
    pub fn record_other_error(&self) {
        self.other_errors.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取总错误数
    pub fn total_errors(&self) -> u64 {
        self.read_errors.load(Ordering::Relaxed)
            + self.write_errors.load(Ordering::Relaxed)
            + self.delete_errors.load(Ordering::Relaxed)
            + self.other_errors.load(Ordering::Relaxed)
    }

    /// 获取错误快照
    pub fn snapshot(&self) -> ErrorSnapshot {
        ErrorSnapshot {
            read_errors: self.read_errors.load(Ordering::Relaxed),
            write_errors: self.write_errors.load(Ordering::Relaxed),
            delete_errors: self.delete_errors.load(Ordering::Relaxed),
            other_errors: self.other_errors.load(Ordering::Relaxed),
        }
    }
}

/// 错误快照
#[derive(Debug, Clone)]
pub struct ErrorSnapshot {
    /// 读取错误数
    pub read_errors: u64,
    /// 写入错误数
    pub write_errors: u64,
    /// 删除错误数
    pub delete_errors: u64,
    /// 其他错误数
    pub other_errors: u64,
}

impl ErrorSnapshot {
    /// 总错误数
    pub fn total(&self) -> u64 {
        self.read_errors + self.write_errors + self.delete_errors + self.other_errors
    }
}

/// 存储指标收集器
#[derive(Debug, Default)]
pub struct StorageMetricsCollector {
    /// 读取延迟
    pub read_latency: LatencyTracker,
    /// 写入延迟
    pub write_latency: LatencyTracker,
    /// 删除延迟
    pub delete_latency: LatencyTracker,
    /// 读取吞吐量
    pub read_throughput: ThroughputTracker,
    /// 写入吞吐量
    pub write_throughput: ThroughputTracker,
    /// 错误追踪
    pub errors: ErrorTracker,
}

impl StorageMetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取完整指标快照
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            read_latency: self.read_latency.snapshot(),
            write_latency: self.write_latency.snapshot(),
            delete_latency: self.delete_latency.snapshot(),
            read_throughput: self.read_throughput.snapshot(),
            write_throughput: self.write_throughput.snapshot(),
            errors: self.errors.snapshot(),
        }
    }

    /// 生成文本报告
    pub fn report(&self) -> String {
        let snapshot = self.snapshot();
        let mut report = String::new();

        report.push_str("=== Storage Metrics Report ===\n\n");

        report.push_str("Read Latency:\n");
        report.push_str(&format!("  {}\n\n", snapshot.read_latency.format()));

        report.push_str("Write Latency:\n");
        report.push_str(&format!("  {}\n\n", snapshot.write_latency.format()));

        report.push_str("Delete Latency:\n");
        report.push_str(&format!("  {}\n\n", snapshot.delete_latency.format()));

        report.push_str("Read Throughput:\n");
        report.push_str(&format!(
            "  ops={}, {:.2} ops/s, {}\n\n",
            snapshot.read_throughput.total_operations,
            snapshot.read_throughput.ops_per_sec,
            snapshot.read_throughput.format_bytes_per_sec()
        ));

        report.push_str("Write Throughput:\n");
        report.push_str(&format!(
            "  ops={}, {:.2} ops/s, {}\n\n",
            snapshot.write_throughput.total_operations,
            snapshot.write_throughput.ops_per_sec,
            snapshot.write_throughput.format_bytes_per_sec()
        ));

        report.push_str("Errors:\n");
        report.push_str(&format!(
            "  read={}, write={}, delete={}, other={}, total={}\n",
            snapshot.errors.read_errors,
            snapshot.errors.write_errors,
            snapshot.errors.delete_errors,
            snapshot.errors.other_errors,
            snapshot.errors.total()
        ));

        report
    }
}

/// 完整指标快照
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    /// 读取延迟
    pub read_latency: LatencySnapshot,
    /// 写入延迟
    pub write_latency: LatencySnapshot,
    /// 删除延迟
    pub delete_latency: LatencySnapshot,
    /// 读取吞吐量
    pub read_throughput: ThroughputSnapshot,
    /// 写入吞吐量
    pub write_throughput: ThroughputSnapshot,
    /// 错误统计
    pub errors: ErrorSnapshot,
}

/// 带指标的存储
///
/// 包装任意 StorageBackend，添加指标收集功能
pub struct MetricsStorage<B: StorageBackend> {
    /// 后端存储
    backend: B,
    /// 指标收集器
    metrics: StorageMetricsCollector,
}

impl<B: StorageBackend> MetricsStorage<B> {
    /// 创建带指标的存储
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            metrics: StorageMetricsCollector::new(),
        }
    }

    /// 获取指标收集器引用
    pub fn metrics(&self) -> &StorageMetricsCollector {
        &self.metrics
    }

    /// 获取详细指标快照
    pub fn detailed_metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    /// 生成指标报告
    pub fn metrics_report(&self) -> String {
        self.metrics.report()
    }

    /// 获取后端引用
    pub fn backend(&self) -> &B {
        &self.backend
    }
}

#[async_trait]
impl<B: StorageBackend + Send + Sync> StorageBackend for MetricsStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let start = Instant::now();
        let result = self.backend.read(key).await;
        let elapsed = start.elapsed();

        self.metrics.read_latency.record(elapsed);

        match &result {
            Ok(data) => {
                self.metrics.read_throughput.record(data.len() as u64);
            }
            Err(_) => {
                self.metrics.errors.record_read_error();
            }
        }

        result
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let start = Instant::now();
        let result = self.backend.write(key, data).await;
        let elapsed = start.elapsed();

        self.metrics.write_latency.record(elapsed);

        match &result {
            Ok(_) => {
                self.metrics.write_throughput.record(data.len() as u64);
            }
            Err(_) => {
                self.metrics.errors.record_write_error();
            }
        }

        result
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let start = Instant::now();
        let result = self.backend.delete(key).await;
        let elapsed = start.elapsed();

        self.metrics.delete_latency.record(elapsed);

        if result.is_err() {
            self.metrics.errors.record_delete_error();
        }

        result
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        let metrics = self.detailed_metrics();
        StorageStats {
            reads: metrics.read_latency.count,
            writes: metrics.write_latency.count,
            deletes: metrics.delete_latency.count,
            hits: 0,
            misses: 0,
            total_bytes: metrics.write_throughput.total_bytes,
            key_count: 0,
        }
    }

    fn name(&self) -> &'static str {
        "MetricsStorage"
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;
    use std::time::Duration;

    #[test]
    fn test_latency_tracker_new() {
        let tracker = LatencyTracker::new();
        assert_eq!(tracker.count(), 0);
        assert!(tracker.avg().is_none());
    }

    #[test]
    fn test_latency_tracker_record() {
        let tracker = LatencyTracker::new();
        tracker.record(Duration::from_micros(100));
        tracker.record(Duration::from_micros(200));
        tracker.record(Duration::from_micros(300));

        assert_eq!(tracker.count(), 3);
        assert_eq!(tracker.min(), Some(Duration::from_micros(100)));
        assert_eq!(tracker.max(), Some(Duration::from_micros(300)));
        assert_eq!(tracker.avg(), Some(Duration::from_micros(200)));
    }

    #[test]
    fn test_latency_tracker_percentiles() {
        let tracker = LatencyTracker::new();

        // 记录 1-100 微秒的延迟
        for i in 1..=100 {
            tracker.record(Duration::from_micros(i));
        }

        let p50 = tracker.p50().unwrap().as_micros();
        let p95 = tracker.p95().unwrap().as_micros();
        let p99 = tracker.p99().unwrap().as_micros();

        // 允许一定误差
        assert!(p50 >= 48 && p50 <= 52);
        assert!(p95 >= 93 && p95 <= 97);
        assert!(p99 >= 97 && p99 <= 100);
    }

    #[test]
    fn test_latency_snapshot_format() {
        let tracker = LatencyTracker::new();
        tracker.record(Duration::from_micros(100));

        let snapshot = tracker.snapshot();
        let formatted = snapshot.format();

        assert!(formatted.contains("count=1"));
    }

    #[test]
    fn test_throughput_tracker_new() {
        let tracker = ThroughputTracker::new();
        assert_eq!(tracker.total_operations(), 0);
        assert_eq!(tracker.total_bytes(), 0);
    }

    #[test]
    fn test_throughput_tracker_record() {
        let tracker = ThroughputTracker::new();
        tracker.record(1024);
        tracker.record(2048);

        assert_eq!(tracker.total_operations(), 2);
        assert_eq!(tracker.total_bytes(), 3072);
    }

    #[test]
    fn test_throughput_snapshot_format() {
        let snapshot = ThroughputSnapshot {
            total_operations: 100,
            total_bytes: 1_500_000,
            ops_per_sec: 50.0,
            bytes_per_sec: 1_500_000.0,
        };

        let formatted = snapshot.format_bytes_per_sec();
        assert!(formatted.contains("MB/s"));
    }

    #[test]
    fn test_error_tracker() {
        let tracker = ErrorTracker::new();
        tracker.record_read_error();
        tracker.record_write_error();
        tracker.record_write_error();

        let snapshot = tracker.snapshot();
        assert_eq!(snapshot.read_errors, 1);
        assert_eq!(snapshot.write_errors, 2);
        assert_eq!(snapshot.total(), 3);
    }

    #[test]
    fn test_storage_metrics_collector() {
        let collector = StorageMetricsCollector::new();

        collector.read_latency.record(Duration::from_micros(100));
        collector.write_latency.record(Duration::from_micros(200));
        collector.read_throughput.record(1024);
        collector.errors.record_read_error();

        let snapshot = collector.snapshot();
        assert_eq!(snapshot.read_latency.count, 1);
        assert_eq!(snapshot.write_latency.count, 1);
        assert_eq!(snapshot.errors.read_errors, 1);
    }

    #[test]
    fn test_storage_metrics_collector_report() {
        let collector = StorageMetricsCollector::new();
        collector.read_latency.record(Duration::from_micros(100));

        let report = collector.report();
        assert!(report.contains("Storage Metrics Report"));
        assert!(report.contains("Read Latency"));
    }

    #[tokio::test]
    async fn test_metrics_storage_new() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        assert_eq!(storage.name(), "MetricsStorage");
    }

    #[tokio::test]
    async fn test_metrics_storage_write_read() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        let loaded = storage.read("key1").await.unwrap();
        assert_eq!(loaded, b"data1");

        let metrics = storage.detailed_metrics();
        assert_eq!(metrics.read_latency.count, 1);
        assert_eq!(metrics.write_latency.count, 1);
    }

    #[tokio::test]
    async fn test_metrics_storage_throughput() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        let data = vec![0u8; 1024];
        storage.write("key1", &data).await.unwrap();
        storage.read("key1").await.unwrap();

        let metrics = storage.detailed_metrics();
        assert_eq!(metrics.write_throughput.total_bytes, 1024);
        assert_eq!(metrics.read_throughput.total_bytes, 1024);
    }

    #[tokio::test]
    async fn test_metrics_storage_errors() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        // 读取不存在的键
        let _ = storage.read("nonexistent").await;

        let metrics = storage.detailed_metrics();
        assert_eq!(metrics.errors.read_errors, 1);
    }

    #[tokio::test]
    async fn test_metrics_storage_delete() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.delete("key1").await.unwrap();

        let metrics = storage.detailed_metrics();
        assert_eq!(metrics.delete_latency.count, 1);
    }

    #[tokio::test]
    async fn test_metrics_storage_report() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.read("key1").await.unwrap();

        let report = storage.metrics_report();
        assert!(report.contains("Read Latency"));
        assert!(report.contains("Write Latency"));
        assert!(report.contains("Throughput"));
    }

    #[tokio::test]
    async fn test_metrics_storage_multiple_ops() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        for i in 0..10 {
            let key = format!("key_{}", i);
            let data = format!("data_{}", i).into_bytes();
            storage.write(&key, &data).await.unwrap();
        }

        for i in 0..10 {
            let key = format!("key_{}", i);
            storage.read(&key).await.unwrap();
        }

        let metrics = storage.detailed_metrics();
        assert_eq!(metrics.read_latency.count, 10);
        assert_eq!(metrics.write_latency.count, 10);
    }

    #[tokio::test]
    async fn test_metrics_storage_latency_percentiles() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        // 执行多次操作以获得百分位数据
        for i in 0..100 {
            let key = format!("key_{}", i);
            storage.write(&key, b"data").await.unwrap();
        }

        let metrics = storage.detailed_metrics();
        assert!(metrics.write_latency.p50.is_some());
        assert!(metrics.write_latency.p95.is_some());
        assert!(metrics.write_latency.p99.is_some());
    }

    #[tokio::test]
    async fn test_metrics_storage_stats() {
        let backend = MemoryStorage::new();
        let storage = MetricsStorage::new(backend);

        storage.write("key1", b"data1").await.unwrap();
        storage.read("key1").await.unwrap();

        let stats = storage.stats();
        assert_eq!(stats.reads, 1);
        assert_eq!(stats.writes, 1);
    }

    #[test]
    fn test_latency_empty_percentile() {
        let tracker = LatencyTracker::new();
        assert!(tracker.percentile(50.0).is_none());
    }

    #[test]
    fn test_throughput_zero_time() {
        let tracker = ThroughputTracker::new();
        // 没有记录时，ops_per_sec 应该是 0
        assert_eq!(tracker.ops_per_sec(), 0.0);
    }
}
