//! 统一指标收集系统
//!
//! v1.110.0: 提供应用级指标收集、聚合和导出功能
//!
//! # 功能特性
//! - 计数器（Counter）：单调递增的计数
//! - 仪表盘（Gauge）：可增可减的即时值
//! - 直方图（Histogram）：值分布统计
//! - 标签支持：多维度指标
//! - 多种导出格式：JSON、Prometheus
//!
//! # 使用示例
//!
//! ```ignore
//! use realconsole::metrics::{MetricsRegistry, Counter, Gauge};
//!
//! let registry = MetricsRegistry::new();
//!
//! // 创建计数器
//! let requests = registry.counter("requests_total", "Total requests");
//! requests.inc();
//! requests.add(5);
//!
//! // 创建仪表盘
//! let active = registry.gauge("active_connections", "Active connections");
//! active.set(10);
//! active.inc();
//! active.dec();
//!
//! // 导出指标
//! let json = registry.export_json();
//! let prom = registry.export_prometheus();
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetricType {
    /// 计数器（单调递增）
    Counter,
    /// 仪表盘（可增可减）
    Gauge,
    /// 直方图（值分布）
    Histogram,
}

/// 指标元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricMetadata {
    /// 指标名称
    pub name: String,
    /// 指标描述
    pub description: String,
    /// 指标类型
    pub metric_type: MetricType,
    /// 单位（可选）
    pub unit: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

impl MetricMetadata {
    pub fn new(name: impl Into<String>, description: impl Into<String>, metric_type: MetricType) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type,
            unit: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }
}

/// 标签集
pub type Labels = HashMap<String, String>;

/// 计数器
///
/// 单调递增的计数器，适用于请求数、错误数等
#[derive(Debug)]
pub struct Counter {
    metadata: MetricMetadata,
    value: AtomicU64,
    labeled_values: RwLock<HashMap<String, AtomicU64>>,
}

impl Counter {
    /// 创建新计数器
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Counter),
            value: AtomicU64::new(0),
            labeled_values: RwLock::new(HashMap::new()),
        }
    }

    /// 获取元数据
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }

    /// 递增 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加指定值
    pub fn add(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// 获取当前值
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// 带标签递增
    pub fn inc_with_labels(&self, labels: &Labels) {
        self.add_with_labels(labels, 1);
    }

    /// 带标签增加
    pub fn add_with_labels(&self, labels: &Labels, value: u64) {
        let key = labels_to_key(labels);
        let mut labeled = self.labeled_values.write().unwrap();
        labeled
            .entry(key)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(value, Ordering::Relaxed);
    }

    /// 获取带标签的值
    pub fn get_with_labels(&self, labels: &Labels) -> u64 {
        let key = labels_to_key(labels);
        let labeled = self.labeled_values.read().unwrap();
        labeled
            .get(&key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 获取所有标签值
    pub fn get_all_labeled(&self) -> HashMap<String, u64> {
        let labeled = self.labeled_values.read().unwrap();
        labeled
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// 重置
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
        let mut labeled = self.labeled_values.write().unwrap();
        labeled.clear();
    }
}

/// 仪表盘
///
/// 可增可减的即时值，适用于连接数、队列长度等
#[derive(Debug)]
pub struct Gauge {
    metadata: MetricMetadata,
    value: AtomicI64,
    labeled_values: RwLock<HashMap<String, AtomicI64>>,
}

impl Gauge {
    /// 创建新仪表盘
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Gauge),
            value: AtomicI64::new(0),
            labeled_values: RwLock::new(HashMap::new()),
        }
    }

    /// 获取元数据
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }

    /// 设置值
    pub fn set(&self, value: i64) {
        self.value.store(value, Ordering::Relaxed);
    }

    /// 递增 1
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// 递减 1
    pub fn dec(&self) {
        self.value.fetch_sub(1, Ordering::Relaxed);
    }

    /// 增加指定值
    pub fn add(&self, value: i64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// 减少指定值
    pub fn sub(&self, value: i64) {
        self.value.fetch_sub(value, Ordering::Relaxed);
    }

    /// 获取当前值
    pub fn get(&self) -> i64 {
        self.value.load(Ordering::Relaxed)
    }

    /// 带标签设置
    pub fn set_with_labels(&self, labels: &Labels, value: i64) {
        let key = labels_to_key(labels);
        let mut labeled = self.labeled_values.write().unwrap();
        labeled
            .entry(key)
            .or_insert_with(|| AtomicI64::new(0))
            .store(value, Ordering::Relaxed);
    }

    /// 获取带标签的值
    pub fn get_with_labels(&self, labels: &Labels) -> i64 {
        let key = labels_to_key(labels);
        let labeled = self.labeled_values.read().unwrap();
        labeled
            .get(&key)
            .map(|v| v.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// 获取所有标签值
    pub fn get_all_labeled(&self) -> HashMap<String, i64> {
        let labeled = self.labeled_values.read().unwrap();
        labeled
            .iter()
            .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
            .collect()
    }

    /// 重置
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
        let mut labeled = self.labeled_values.write().unwrap();
        labeled.clear();
    }
}

/// 直方图桶配置
#[derive(Debug, Clone)]
pub struct HistogramBuckets {
    /// 桶边界值
    pub boundaries: Vec<f64>,
}

impl Default for HistogramBuckets {
    fn default() -> Self {
        // 默认桶：适用于延迟测量（毫秒）
        Self {
            boundaries: vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0],
        }
    }
}

impl HistogramBuckets {
    /// 创建线性桶
    pub fn linear(start: f64, width: f64, count: usize) -> Self {
        let boundaries: Vec<f64> = (0..count).map(|i| start + width * i as f64).collect();
        Self { boundaries }
    }

    /// 创建指数桶
    pub fn exponential(start: f64, factor: f64, count: usize) -> Self {
        let boundaries: Vec<f64> = (0..count).map(|i| start * factor.powi(i as i32)).collect();
        Self { boundaries }
    }
}

/// 直方图
///
/// 值分布统计，适用于延迟、大小等
#[derive(Debug)]
pub struct Histogram {
    metadata: MetricMetadata,
    buckets: HistogramBuckets,
    bucket_counts: Vec<AtomicU64>,
    sum: AtomicU64,
    count: AtomicU64,
}

impl Histogram {
    /// 创建新直方图
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self::with_buckets(name, description, HistogramBuckets::default())
    }

    /// 使用自定义桶创建
    pub fn with_buckets(
        name: impl Into<String>,
        description: impl Into<String>,
        buckets: HistogramBuckets,
    ) -> Self {
        let bucket_counts = (0..=buckets.boundaries.len())
            .map(|_| AtomicU64::new(0))
            .collect();

        Self {
            metadata: MetricMetadata::new(name, description, MetricType::Histogram),
            buckets,
            bucket_counts,
            sum: AtomicU64::new(0),
            count: AtomicU64::new(0),
        }
    }

    /// 获取元数据
    pub fn metadata(&self) -> &MetricMetadata {
        &self.metadata
    }

    /// 观测一个值
    pub fn observe(&self, value: f64) {
        // 找到对应的桶
        let bucket_idx = self
            .buckets
            .boundaries
            .iter()
            .position(|&b| value <= b)
            .unwrap_or(self.buckets.boundaries.len());

        self.bucket_counts[bucket_idx].fetch_add(1, Ordering::Relaxed);
        self.sum.fetch_add((value * 1000.0) as u64, Ordering::Relaxed); // 存储为微秒精度
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取总和
    pub fn sum(&self) -> f64 {
        self.sum.load(Ordering::Relaxed) as f64 / 1000.0
    }

    /// 获取计数
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// 获取平均值
    pub fn mean(&self) -> f64 {
        let count = self.count();
        if count == 0 {
            0.0
        } else {
            self.sum() / count as f64
        }
    }

    /// 获取桶统计
    pub fn buckets(&self) -> Vec<(f64, u64)> {
        self.buckets
            .boundaries
            .iter()
            .zip(self.bucket_counts.iter())
            .map(|(&b, c)| (b, c.load(Ordering::Relaxed)))
            .collect()
    }

    /// 获取累积桶统计（Prometheus 格式）
    pub fn cumulative_buckets(&self) -> Vec<(f64, u64)> {
        let mut cumulative = 0u64;
        let mut result = Vec::new();

        for (i, boundary) in self.buckets.boundaries.iter().enumerate() {
            cumulative += self.bucket_counts[i].load(Ordering::Relaxed);
            result.push((*boundary, cumulative));
        }

        // +Inf 桶
        cumulative += self.bucket_counts[self.buckets.boundaries.len()].load(Ordering::Relaxed);
        result.push((f64::INFINITY, cumulative));

        result
    }

    /// 重置
    pub fn reset(&self) {
        for bucket in &self.bucket_counts {
            bucket.store(0, Ordering::Relaxed);
        }
        self.sum.store(0, Ordering::Relaxed);
        self.count.store(0, Ordering::Relaxed);
    }
}

/// 指标注册表
pub struct MetricsRegistry {
    /// 计数器
    counters: RwLock<HashMap<String, Arc<Counter>>>,
    /// 仪表盘
    gauges: RwLock<HashMap<String, Arc<Gauge>>>,
    /// 直方图
    histograms: RwLock<HashMap<String, Arc<Histogram>>>,
    /// 全局标签
    global_labels: RwLock<Labels>,
    /// 创建时间
    created_at: DateTime<Utc>,
}

impl MetricsRegistry {
    /// 创建新注册表
    pub fn new() -> Self {
        Self {
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            histograms: RwLock::new(HashMap::new()),
            global_labels: RwLock::new(HashMap::new()),
            created_at: Utc::now(),
        }
    }

    /// 设置全局标签
    pub fn set_global_label(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut labels = self.global_labels.write().unwrap();
        labels.insert(key.into(), value.into());
    }

    /// 获取或创建计数器
    pub fn counter(&self, name: impl Into<String>, description: impl Into<String>) -> Arc<Counter> {
        let name = name.into();
        let mut counters = self.counters.write().unwrap();

        counters
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Counter::new(name, description.into())))
            .clone()
    }

    /// 获取或创建仪表盘
    pub fn gauge(&self, name: impl Into<String>, description: impl Into<String>) -> Arc<Gauge> {
        let name = name.into();
        let mut gauges = self.gauges.write().unwrap();

        gauges
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Gauge::new(name, description.into())))
            .clone()
    }

    /// 获取或创建直方图
    pub fn histogram(&self, name: impl Into<String>, description: impl Into<String>) -> Arc<Histogram> {
        let name = name.into();
        let mut histograms = self.histograms.write().unwrap();

        histograms
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Histogram::new(name, description.into())))
            .clone()
    }

    /// 获取或创建带自定义桶的直方图
    pub fn histogram_with_buckets(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        buckets: HistogramBuckets,
    ) -> Arc<Histogram> {
        let name = name.into();
        let mut histograms = self.histograms.write().unwrap();

        histograms
            .entry(name.clone())
            .or_insert_with(|| Arc::new(Histogram::with_buckets(name, description.into(), buckets)))
            .clone()
    }

    /// 导出为 JSON
    pub fn export_json(&self) -> serde_json::Value {
        let counters = self.counters.read().unwrap();
        let gauges = self.gauges.read().unwrap();
        let histograms = self.histograms.read().unwrap();
        let global_labels = self.global_labels.read().unwrap();

        serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "global_labels": *global_labels,
            "counters": counters.iter().map(|(name, c)| {
                serde_json::json!({
                    "name": name,
                    "description": c.metadata().description,
                    "value": c.get(),
                    "labeled": c.get_all_labeled()
                })
            }).collect::<Vec<_>>(),
            "gauges": gauges.iter().map(|(name, g)| {
                serde_json::json!({
                    "name": name,
                    "description": g.metadata().description,
                    "value": g.get(),
                    "labeled": g.get_all_labeled()
                })
            }).collect::<Vec<_>>(),
            "histograms": histograms.iter().map(|(name, h)| {
                serde_json::json!({
                    "name": name,
                    "description": h.metadata().description,
                    "count": h.count(),
                    "sum": h.sum(),
                    "mean": h.mean(),
                    "buckets": h.buckets()
                })
            }).collect::<Vec<_>>()
        })
    }

    /// 导出为 Prometheus 格式
    pub fn export_prometheus(&self) -> String {
        let counters = self.counters.read().unwrap();
        let gauges = self.gauges.read().unwrap();
        let histograms = self.histograms.read().unwrap();
        let global_labels = self.global_labels.read().unwrap();

        let mut output = String::new();

        // 计数器
        for (name, counter) in counters.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, counter.metadata().description));
            output.push_str(&format!("# TYPE {} counter\n", name));

            let labels_str = format_prometheus_labels(&global_labels);
            output.push_str(&format!("{}{} {}\n", name, labels_str, counter.get()));

            // 带标签的值
            for (label_key, value) in counter.get_all_labeled() {
                let mut all_labels = global_labels.clone();
                all_labels.extend(parse_label_key(&label_key));
                let labels_str = format_prometheus_labels(&all_labels);
                output.push_str(&format!("{}{} {}\n", name, labels_str, value));
            }
        }

        // 仪表盘
        for (name, gauge) in gauges.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, gauge.metadata().description));
            output.push_str(&format!("# TYPE {} gauge\n", name));

            let labels_str = format_prometheus_labels(&global_labels);
            output.push_str(&format!("{}{} {}\n", name, labels_str, gauge.get()));

            // 带标签的值
            for (label_key, value) in gauge.get_all_labeled() {
                let mut all_labels = global_labels.clone();
                all_labels.extend(parse_label_key(&label_key));
                let labels_str = format_prometheus_labels(&all_labels);
                output.push_str(&format!("{}{} {}\n", name, labels_str, value));
            }
        }

        // 直方图
        for (name, histogram) in histograms.iter() {
            output.push_str(&format!("# HELP {} {}\n", name, histogram.metadata().description));
            output.push_str(&format!("# TYPE {} histogram\n", name));

            let labels_str = format_prometheus_labels(&global_labels);

            // 桶
            for (le, count) in histogram.cumulative_buckets() {
                let le_str = if le.is_infinite() {
                    "+Inf".to_string()
                } else {
                    le.to_string()
                };
                output.push_str(&format!(
                    "{}_bucket{{le=\"{}\"{}}} {}\n",
                    name,
                    le_str,
                    if labels_str.is_empty() {
                        String::new()
                    } else {
                        format!(",{}", &labels_str[1..labels_str.len() - 1])
                    },
                    count
                ));
            }

            output.push_str(&format!("{}_sum{} {}\n", name, labels_str, histogram.sum()));
            output.push_str(&format!("{}_count{} {}\n", name, labels_str, histogram.count()));
        }

        output
    }

    /// 获取所有指标名称
    pub fn metric_names(&self) -> Vec<String> {
        let mut names = Vec::new();

        let counters = self.counters.read().unwrap();
        names.extend(counters.keys().cloned());

        let gauges = self.gauges.read().unwrap();
        names.extend(gauges.keys().cloned());

        let histograms = self.histograms.read().unwrap();
        names.extend(histograms.keys().cloned());

        names.sort();
        names
    }

    /// 获取指标数量
    pub fn metric_count(&self) -> usize {
        let counters = self.counters.read().unwrap();
        let gauges = self.gauges.read().unwrap();
        let histograms = self.histograms.read().unwrap();

        counters.len() + gauges.len() + histograms.len()
    }

    /// 重置所有指标
    pub fn reset_all(&self) {
        let counters = self.counters.read().unwrap();
        for counter in counters.values() {
            counter.reset();
        }

        let gauges = self.gauges.read().unwrap();
        for gauge in gauges.values() {
            gauge.reset();
        }

        let histograms = self.histograms.read().unwrap();
        for histogram in histograms.values() {
            histogram.reset();
        }
    }

    /// 获取运行时间
    pub fn uptime(&self) -> std::time::Duration {
        let now = Utc::now();
        let duration = now - self.created_at;
        std::time::Duration::from_secs(duration.num_seconds().max(0) as u64)
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 将标签转换为键
fn labels_to_key(labels: &Labels) -> String {
    let mut pairs: Vec<_> = labels.iter().collect();
    pairs.sort_by_key(|(k, _)| *k);
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",")
}

/// 解析标签键
fn parse_label_key(key: &str) -> Labels {
    let mut labels = Labels::new();
    for pair in key.split(',') {
        if let Some((k, v)) = pair.split_once('=') {
            labels.insert(k.to_string(), v.to_string());
        }
    }
    labels
}

/// 格式化 Prometheus 标签
fn format_prometheus_labels(labels: &Labels) -> String {
    if labels.is_empty() {
        return String::new();
    }

    let pairs: Vec<_> = labels
        .iter()
        .map(|(k, v)| format!("{}=\"{}\"", k, v))
        .collect();

    format!("{{{}}}", pairs.join(","))
}

/// 预定义指标名称
pub mod names {
    /// 请求相关
    pub const REQUESTS_TOTAL: &str = "requests_total";
    pub const REQUESTS_DURATION: &str = "requests_duration_ms";
    pub const REQUESTS_ERRORS: &str = "requests_errors_total";

    /// LLM 相关
    pub const LLM_REQUESTS: &str = "llm_requests_total";
    pub const LLM_TOKENS: &str = "llm_tokens_total";
    pub const LLM_LATENCY: &str = "llm_latency_ms";

    /// 工具相关
    pub const TOOL_CALLS: &str = "tool_calls_total";
    pub const TOOL_DURATION: &str = "tool_duration_ms";
    pub const TOOL_ERRORS: &str = "tool_errors_total";

    /// 会话相关
    pub const ACTIVE_SESSIONS: &str = "active_sessions";
    pub const SESSION_DURATION: &str = "session_duration_seconds";

    /// 系统相关
    pub const MEMORY_USAGE: &str = "memory_usage_bytes";
    pub const CPU_USAGE: &str = "cpu_usage_percent";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_basic() {
        let counter = Counter::new("test_counter", "Test counter");
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.add(5);
        assert_eq!(counter.get(), 6);
    }

    #[test]
    fn test_counter_with_labels() {
        let counter = Counter::new("labeled_counter", "Counter with labels");

        let mut labels = Labels::new();
        labels.insert("method".to_string(), "GET".to_string());

        counter.inc_with_labels(&labels);
        counter.add_with_labels(&labels, 4);

        assert_eq!(counter.get_with_labels(&labels), 5);
        assert_eq!(counter.get(), 0); // 无标签值不变
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new("reset_counter", "Reset test");
        counter.add(10);

        let mut labels = Labels::new();
        labels.insert("key".to_string(), "value".to_string());
        counter.add_with_labels(&labels, 5);

        counter.reset();

        assert_eq!(counter.get(), 0);
        assert_eq!(counter.get_with_labels(&labels), 0);
    }

    #[test]
    fn test_gauge_basic() {
        let gauge = Gauge::new("test_gauge", "Test gauge");
        assert_eq!(gauge.get(), 0);

        gauge.set(10);
        assert_eq!(gauge.get(), 10);

        gauge.inc();
        assert_eq!(gauge.get(), 11);

        gauge.dec();
        assert_eq!(gauge.get(), 10);

        gauge.add(5);
        assert_eq!(gauge.get(), 15);

        gauge.sub(3);
        assert_eq!(gauge.get(), 12);
    }

    #[test]
    fn test_gauge_with_labels() {
        let gauge = Gauge::new("labeled_gauge", "Gauge with labels");

        let mut labels = Labels::new();
        labels.insert("host".to_string(), "server1".to_string());

        gauge.set_with_labels(&labels, 100);
        assert_eq!(gauge.get_with_labels(&labels), 100);
    }

    #[test]
    fn test_histogram_basic() {
        let histogram = Histogram::new("test_histogram", "Test histogram");

        histogram.observe(5.0);
        histogram.observe(15.0);
        histogram.observe(50.0);

        assert_eq!(histogram.count(), 3);
        assert!((histogram.sum() - 70.0).abs() < 0.01);
        assert!((histogram.mean() - 23.33).abs() < 0.1);
    }

    #[test]
    fn test_histogram_buckets() {
        let buckets = HistogramBuckets {
            boundaries: vec![10.0, 50.0, 100.0],
        };
        let histogram = Histogram::with_buckets("bucket_test", "Bucket test", buckets);

        histogram.observe(5.0);   // <= 10
        histogram.observe(25.0);  // <= 50
        histogram.observe(75.0);  // <= 100
        histogram.observe(150.0); // > 100

        let cumulative = histogram.cumulative_buckets();
        assert_eq!(cumulative[0], (10.0, 1));  // <= 10: 1
        assert_eq!(cumulative[1], (50.0, 2));  // <= 50: 2
        assert_eq!(cumulative[2], (100.0, 3)); // <= 100: 3
        assert_eq!(cumulative[3].1, 4);        // +Inf: 4
    }

    #[test]
    fn test_histogram_linear_buckets() {
        let buckets = HistogramBuckets::linear(0.0, 10.0, 5);
        assert_eq!(buckets.boundaries, vec![0.0, 10.0, 20.0, 30.0, 40.0]);
    }

    #[test]
    fn test_histogram_exponential_buckets() {
        let buckets = HistogramBuckets::exponential(1.0, 2.0, 4);
        assert_eq!(buckets.boundaries, vec![1.0, 2.0, 4.0, 8.0]);
    }

    #[test]
    fn test_registry_counter() {
        let registry = MetricsRegistry::new();

        let counter1 = registry.counter("test", "Test counter");
        counter1.inc();

        let counter2 = registry.counter("test", "Test counter");
        counter2.inc();

        // 应该是同一个计数器
        assert_eq!(counter1.get(), 2);
        assert_eq!(counter2.get(), 2);
    }

    #[test]
    fn test_registry_gauge() {
        let registry = MetricsRegistry::new();

        let gauge = registry.gauge("connections", "Active connections");
        gauge.set(10);

        assert_eq!(gauge.get(), 10);
    }

    #[test]
    fn test_registry_histogram() {
        let registry = MetricsRegistry::new();

        let histogram = registry.histogram("latency", "Request latency");
        histogram.observe(10.0);
        histogram.observe(20.0);

        assert_eq!(histogram.count(), 2);
    }

    #[test]
    fn test_registry_global_labels() {
        let registry = MetricsRegistry::new();
        registry.set_global_label("app", "realconsole");
        registry.set_global_label("version", "1.110.0");

        let json = registry.export_json();
        let global_labels = json["global_labels"].as_object().unwrap();

        assert_eq!(global_labels["app"], "realconsole");
        assert_eq!(global_labels["version"], "1.110.0");
    }

    #[test]
    fn test_export_json() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("requests", "Total requests");
        counter.add(100);

        let gauge = registry.gauge("active", "Active items");
        gauge.set(5);

        let json = registry.export_json();

        assert!(json["timestamp"].is_string());
        assert!(json["counters"].is_array());
        assert!(json["gauges"].is_array());
    }

    #[test]
    fn test_export_prometheus() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("http_requests_total", "Total HTTP requests");
        counter.add(100);

        let prom = registry.export_prometheus();

        assert!(prom.contains("# HELP http_requests_total"));
        assert!(prom.contains("# TYPE http_requests_total counter"));
        assert!(prom.contains("http_requests_total 100"));
    }

    #[test]
    fn test_metric_names() {
        let registry = MetricsRegistry::new();

        registry.counter("z_counter", "Z");
        registry.gauge("a_gauge", "A");
        registry.histogram("m_histogram", "M");

        let names = registry.metric_names();
        assert_eq!(names, vec!["a_gauge", "m_histogram", "z_counter"]);
    }

    #[test]
    fn test_metric_count() {
        let registry = MetricsRegistry::new();

        registry.counter("c1", "C1");
        registry.counter("c2", "C2");
        registry.gauge("g1", "G1");

        assert_eq!(registry.metric_count(), 3);
    }

    #[test]
    fn test_reset_all() {
        let registry = MetricsRegistry::new();

        let counter = registry.counter("reset_test", "Reset test");
        counter.add(50);

        let gauge = registry.gauge("reset_gauge", "Reset gauge");
        gauge.set(100);

        registry.reset_all();

        assert_eq!(counter.get(), 0);
        assert_eq!(gauge.get(), 0);
    }

    #[test]
    fn test_predefined_names() {
        assert_eq!(names::REQUESTS_TOTAL, "requests_total");
        assert_eq!(names::LLM_LATENCY, "llm_latency_ms");
    }
}
