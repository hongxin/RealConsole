//! Resource Monitor - Memory and resource usage tracking
//!
//! Provides real-time monitoring of system resources with configurable thresholds.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// Resource Configuration
// ============================================================================

/// Resource monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    /// Memory warning threshold (bytes)
    pub memory_warning_bytes: u64,
    /// Memory critical threshold (bytes)
    pub memory_critical_bytes: u64,
    /// Component count warning threshold
    pub component_count_warning: usize,
    /// Monitoring interval (milliseconds)
    pub monitor_interval_ms: u64,
    /// Enable automatic cleanup
    pub auto_cleanup: bool,
    /// History retention (samples to keep)
    pub history_size: usize,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            memory_warning_bytes: 100 * 1024 * 1024,    // 100 MB
            memory_critical_bytes: 500 * 1024 * 1024,   // 500 MB
            component_count_warning: 1000,
            monitor_interval_ms: 5000, // 5 seconds
            auto_cleanup: true,
            history_size: 100,
        }
    }
}

// ============================================================================
// Resource Threshold
// ============================================================================

/// Resource threshold level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceThreshold {
    /// Normal usage
    Normal,
    /// Warning level - consider cleanup
    Warning,
    /// Critical level - immediate cleanup needed
    Critical,
}

impl ResourceThreshold {
    /// Check if action is needed
    pub fn needs_action(&self) -> bool {
        matches!(self, Self::Warning | Self::Critical)
    }

    /// Check if critical
    pub fn is_critical(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

// ============================================================================
// Memory Snapshot
// ============================================================================

/// Memory usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// Timestamp (epoch milliseconds)
    pub timestamp_ms: u64,
    /// Estimated heap usage (bytes)
    pub heap_bytes: u64,
    /// Number of tracked allocations
    pub allocation_count: usize,
    /// Largest single allocation (bytes)
    pub largest_allocation: u64,
}

impl MemorySnapshot {
    /// Create a new snapshot
    pub fn new() -> Self {
        Self {
            timestamp_ms: Self::now_ms(),
            heap_bytes: 0,
            allocation_count: 0,
            largest_allocation: 0,
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

impl Default for MemorySnapshot {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Component Usage
// ============================================================================

/// Per-component resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentUsage {
    /// Component name
    pub name: String,
    /// Estimated memory usage (bytes)
    pub memory_bytes: u64,
    /// Item count
    pub item_count: usize,
    /// Last activity timestamp
    pub last_activity_ms: u64,
    /// Creation timestamp
    pub created_ms: u64,
}

impl ComponentUsage {
    /// Create new component usage
    pub fn new(name: impl Into<String>) -> Self {
        let now = MemorySnapshot::now_ms();
        Self {
            name: name.into(),
            memory_bytes: 0,
            item_count: 0,
            last_activity_ms: now,
            created_ms: now,
        }
    }

    /// Age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = MemorySnapshot::now_ms();
        (now.saturating_sub(self.created_ms)) / 1000
    }

    /// Idle time in seconds
    pub fn idle_seconds(&self) -> u64 {
        let now = MemorySnapshot::now_ms();
        (now.saturating_sub(self.last_activity_ms)) / 1000
    }

    /// Update activity timestamp
    pub fn touch(&mut self) {
        self.last_activity_ms = MemorySnapshot::now_ms();
    }
}

// ============================================================================
// Resource Usage
// ============================================================================

/// Overall resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Total memory usage (bytes)
    pub total_memory_bytes: u64,
    /// Total component count
    pub component_count: usize,
    /// Memory threshold level
    pub memory_level: ResourceThreshold,
    /// Component count threshold level
    pub component_level: ResourceThreshold,
    /// Per-component breakdown
    pub components: Vec<ComponentUsage>,
    /// Timestamp
    pub timestamp_ms: u64,
}

impl ResourceUsage {
    /// Overall threshold level (worst of all)
    pub fn overall_level(&self) -> ResourceThreshold {
        if self.memory_level.is_critical() || self.component_level.is_critical() {
            ResourceThreshold::Critical
        } else if self.memory_level.needs_action() || self.component_level.needs_action() {
            ResourceThreshold::Warning
        } else {
            ResourceThreshold::Normal
        }
    }

    /// Summary string
    pub fn summary(&self) -> String {
        format!(
            "Memory: {} ({:?}), Components: {} ({:?})",
            Self::format_bytes(self.total_memory_bytes),
            self.memory_level,
            self.component_count,
            self.component_level
        )
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }
}

// ============================================================================
// Resource Statistics
// ============================================================================

/// Resource monitoring statistics
#[derive(Debug, Default)]
pub struct ResourceStats {
    /// Total samples collected
    samples_collected: AtomicU64,
    /// Warning events triggered
    warnings_triggered: AtomicU64,
    /// Critical events triggered
    criticals_triggered: AtomicU64,
    /// Cleanup operations triggered
    cleanups_triggered: AtomicU64,
    /// Peak memory usage (bytes)
    peak_memory_bytes: AtomicU64,
    /// Peak component count
    peak_component_count: AtomicUsize,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceStatsSnapshot {
    pub samples_collected: u64,
    pub warnings_triggered: u64,
    pub criticals_triggered: u64,
    pub cleanups_triggered: u64,
    pub peak_memory_bytes: u64,
    pub peak_component_count: usize,
}

impl ResourceStats {
    fn snapshot(&self) -> ResourceStatsSnapshot {
        ResourceStatsSnapshot {
            samples_collected: self.samples_collected.load(Ordering::Relaxed),
            warnings_triggered: self.warnings_triggered.load(Ordering::Relaxed),
            criticals_triggered: self.criticals_triggered.load(Ordering::Relaxed),
            cleanups_triggered: self.cleanups_triggered.load(Ordering::Relaxed),
            peak_memory_bytes: self.peak_memory_bytes.load(Ordering::Relaxed),
            peak_component_count: self.peak_component_count.load(Ordering::Relaxed),
        }
    }

    fn update_peak_memory(&self, bytes: u64) {
        let current = self.peak_memory_bytes.load(Ordering::Relaxed);
        if bytes > current {
            self.peak_memory_bytes.store(bytes, Ordering::Relaxed);
        }
    }

    fn update_peak_components(&self, count: usize) {
        let current = self.peak_component_count.load(Ordering::Relaxed);
        if count > current {
            self.peak_component_count.store(count, Ordering::Relaxed);
        }
    }
}

// ============================================================================
// Resource Monitor
// ============================================================================

/// Resource monitor for tracking memory and component usage
pub struct ResourceMonitor {
    /// Configuration
    config: ResourceConfig,
    /// Registered components
    components: RwLock<HashMap<String, ComponentUsage>>,
    /// Memory history
    history: RwLock<Vec<MemorySnapshot>>,
    /// Statistics
    stats: Arc<ResourceStats>,
    /// Start time
    start_time: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(config: ResourceConfig) -> Self {
        Self {
            config,
            components: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
            stats: Arc::new(ResourceStats::default()),
            start_time: Instant::now(),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(ResourceConfig::default())
    }

    /// Register a component for tracking
    pub async fn register(&self, name: impl Into<String>) {
        let name = name.into();
        let mut components = self.components.write().await;
        if !components.contains_key(&name) {
            components.insert(name.clone(), ComponentUsage::new(name));
        }
    }

    /// Update component memory usage
    pub async fn update_component(&self, name: &str, memory_bytes: u64, item_count: usize) {
        let mut components = self.components.write().await;
        if let Some(comp) = components.get_mut(name) {
            comp.memory_bytes = memory_bytes;
            comp.item_count = item_count;
            comp.touch();
        } else {
            let mut comp = ComponentUsage::new(name);
            comp.memory_bytes = memory_bytes;
            comp.item_count = item_count;
            components.insert(name.to_string(), comp);
        }
    }

    /// Remove a component
    pub async fn unregister(&self, name: &str) -> Option<ComponentUsage> {
        let mut components = self.components.write().await;
        components.remove(name)
    }

    /// Get current resource usage
    pub async fn usage(&self) -> ResourceUsage {
        let components = self.components.read().await;

        let total_memory: u64 = components.values().map(|c| c.memory_bytes).sum();
        let component_count = components.len();

        let memory_level = self.evaluate_memory_threshold(total_memory);
        let component_level = self.evaluate_component_threshold(component_count);

        // Update stats
        self.stats.update_peak_memory(total_memory);
        self.stats.update_peak_components(component_count);

        if memory_level == ResourceThreshold::Warning
            || component_level == ResourceThreshold::Warning
        {
            self.stats.warnings_triggered.fetch_add(1, Ordering::Relaxed);
        }
        if memory_level == ResourceThreshold::Critical
            || component_level == ResourceThreshold::Critical
        {
            self.stats
                .criticals_triggered
                .fetch_add(1, Ordering::Relaxed);
        }

        ResourceUsage {
            total_memory_bytes: total_memory,
            component_count,
            memory_level,
            component_level,
            components: components.values().cloned().collect(),
            timestamp_ms: MemorySnapshot::now_ms(),
        }
    }

    /// Take a memory snapshot
    pub async fn snapshot(&self) -> MemorySnapshot {
        let components = self.components.read().await;

        let heap_bytes: u64 = components.values().map(|c| c.memory_bytes).sum();
        let allocation_count = components.len();
        let largest_allocation = components
            .values()
            .map(|c| c.memory_bytes)
            .max()
            .unwrap_or(0);

        let snapshot = MemorySnapshot {
            timestamp_ms: MemorySnapshot::now_ms(),
            heap_bytes,
            allocation_count,
            largest_allocation,
        };

        // Add to history
        let mut history = self.history.write().await;
        history.push(snapshot.clone());

        // Trim history
        while history.len() > self.config.history_size {
            history.remove(0);
        }

        self.stats.samples_collected.fetch_add(1, Ordering::Relaxed);

        snapshot
    }

    /// Get memory history
    pub async fn history(&self) -> Vec<MemorySnapshot> {
        self.history.read().await.clone()
    }

    /// Get components sorted by memory usage (descending)
    pub async fn top_components(&self, limit: usize) -> Vec<ComponentUsage> {
        let components = self.components.read().await;
        let mut sorted: Vec<_> = components.values().cloned().collect();
        sorted.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
        sorted.truncate(limit);
        sorted
    }

    /// Get idle components (no activity for specified duration)
    pub async fn idle_components(&self, idle_seconds: u64) -> Vec<ComponentUsage> {
        let components = self.components.read().await;
        components
            .values()
            .filter(|c| c.idle_seconds() >= idle_seconds)
            .cloned()
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> ResourceStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get configuration
    pub fn config(&self) -> &ResourceConfig {
        &self.config
    }

    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Record cleanup triggered
    pub fn record_cleanup(&self) {
        self.stats.cleanups_triggered.fetch_add(1, Ordering::Relaxed);
    }

    /// Evaluate memory threshold
    fn evaluate_memory_threshold(&self, bytes: u64) -> ResourceThreshold {
        if bytes >= self.config.memory_critical_bytes {
            ResourceThreshold::Critical
        } else if bytes >= self.config.memory_warning_bytes {
            ResourceThreshold::Warning
        } else {
            ResourceThreshold::Normal
        }
    }

    /// Evaluate component count threshold
    fn evaluate_component_threshold(&self, count: usize) -> ResourceThreshold {
        let warning = self.config.component_count_warning;
        let critical = warning * 2;

        if count >= critical {
            ResourceThreshold::Critical
        } else if count >= warning {
            ResourceThreshold::Warning
        } else {
            ResourceThreshold::Normal
        }
    }

    /// Generate health report
    pub async fn health_report(&self) -> String {
        let usage = self.usage().await;
        let stats = self.stats();
        let uptime = self.uptime();

        let mut report = String::new();
        report.push_str("=== Resource Monitor Health Report ===\n\n");

        // Uptime
        report.push_str(&format!(
            "Uptime: {}h {}m {}s\n\n",
            uptime.as_secs() / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        ));

        // Current usage
        report.push_str("Current Usage:\n");
        report.push_str(&format!("  {}\n\n", usage.summary()));

        // Statistics
        report.push_str("Statistics:\n");
        report.push_str(&format!("  Samples: {}\n", stats.samples_collected));
        report.push_str(&format!("  Warnings: {}\n", stats.warnings_triggered));
        report.push_str(&format!("  Criticals: {}\n", stats.criticals_triggered));
        report.push_str(&format!("  Cleanups: {}\n", stats.cleanups_triggered));
        report.push_str(&format!(
            "  Peak Memory: {}\n",
            ResourceUsage::format_bytes(stats.peak_memory_bytes)
        ));
        report.push_str(&format!(
            "  Peak Components: {}\n\n",
            stats.peak_component_count
        ));

        // Top components
        let top = self.top_components(5).await;
        if !top.is_empty() {
            report.push_str("Top Components by Memory:\n");
            for comp in top {
                report.push_str(&format!(
                    "  {} - {} ({} items)\n",
                    comp.name,
                    ResourceUsage::format_bytes(comp.memory_bytes),
                    comp.item_count
                ));
            }
        }

        report
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resource_monitor_basic() {
        let monitor = ResourceMonitor::default_config();

        monitor.register("test").await;
        monitor.update_component("test", 1024, 10).await;

        let usage = monitor.usage().await;
        assert_eq!(usage.total_memory_bytes, 1024);
        assert_eq!(usage.component_count, 1);
        assert_eq!(usage.memory_level, ResourceThreshold::Normal);
    }

    #[tokio::test]
    async fn test_memory_threshold_warning() {
        let config = ResourceConfig {
            memory_warning_bytes: 100,
            memory_critical_bytes: 500,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);

        monitor.update_component("test", 200, 1).await;

        let usage = monitor.usage().await;
        assert_eq!(usage.memory_level, ResourceThreshold::Warning);
    }

    #[tokio::test]
    async fn test_memory_threshold_critical() {
        let config = ResourceConfig {
            memory_warning_bytes: 100,
            memory_critical_bytes: 500,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);

        monitor.update_component("test", 600, 1).await;

        let usage = monitor.usage().await;
        assert_eq!(usage.memory_level, ResourceThreshold::Critical);
    }

    #[tokio::test]
    async fn test_snapshot_history() {
        let config = ResourceConfig {
            history_size: 3,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);

        for i in 0..5 {
            monitor.update_component("test", i * 100, 1).await;
            monitor.snapshot().await;
        }

        let history = monitor.history().await;
        assert_eq!(history.len(), 3);
    }

    #[tokio::test]
    async fn test_top_components() {
        let monitor = ResourceMonitor::default_config();

        monitor.update_component("small", 100, 1).await;
        monitor.update_component("medium", 500, 1).await;
        monitor.update_component("large", 1000, 1).await;

        let top = monitor.top_components(2).await;
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].name, "large");
        assert_eq!(top[1].name, "medium");
    }

    #[tokio::test]
    async fn test_idle_components() {
        let monitor = ResourceMonitor::default_config();

        monitor.update_component("active", 100, 1).await;
        monitor.update_component("idle", 100, 1).await;

        // Immediately after creation, no components are idle
        let idle = monitor.idle_components(0).await;
        assert!(!idle.is_empty()); // All components have idle_seconds >= 0
    }

    #[tokio::test]
    async fn test_stats() {
        let config = ResourceConfig {
            memory_warning_bytes: 100,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(config);

        monitor.update_component("test", 200, 1).await;
        let _ = monitor.usage().await;

        let stats = monitor.stats();
        assert_eq!(stats.warnings_triggered, 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let monitor = ResourceMonitor::default_config();

        monitor.register("test").await;
        monitor.update_component("test", 100, 1).await;

        let removed = monitor.unregister("test").await;
        assert!(removed.is_some());

        let usage = monitor.usage().await;
        assert_eq!(usage.component_count, 0);
    }

    #[test]
    fn test_resource_threshold() {
        assert!(!ResourceThreshold::Normal.needs_action());
        assert!(ResourceThreshold::Warning.needs_action());
        assert!(ResourceThreshold::Critical.needs_action());
        assert!(ResourceThreshold::Critical.is_critical());
    }

    #[test]
    fn test_component_usage() {
        let mut comp = ComponentUsage::new("test");
        assert_eq!(comp.age_seconds(), 0);
        comp.touch();
        assert_eq!(comp.idle_seconds(), 0);
    }

    #[tokio::test]
    async fn test_health_report() {
        let monitor = ResourceMonitor::default_config();
        monitor.update_component("test", 1024, 10).await;

        let report = monitor.health_report().await;
        assert!(report.contains("Resource Monitor Health Report"));
        assert!(report.contains("test"));
    }
}
