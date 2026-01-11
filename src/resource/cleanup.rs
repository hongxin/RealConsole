//! Cleanup Manager - Automatic resource cleanup
//!
//! Provides automatic cleanup of resources when thresholds are exceeded.

use super::monitor::{ComponentUsage, ResourceMonitor, ResourceThreshold};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// Cleanup Configuration
// ============================================================================

/// Cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Enable automatic cleanup
    pub enabled: bool,
    /// Minimum idle time before cleanup (seconds)
    pub min_idle_seconds: u64,
    /// Maximum memory per component before cleanup (bytes)
    pub max_component_memory: u64,
    /// Maximum items per component before cleanup
    pub max_component_items: usize,
    /// Cleanup batch size
    pub batch_size: usize,
    /// Cool-down between cleanups (milliseconds)
    pub cooldown_ms: u64,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_idle_seconds: 300, // 5 minutes
            max_component_memory: 50 * 1024 * 1024, // 50 MB
            max_component_items: 10000,
            batch_size: 100,
            cooldown_ms: 10000, // 10 seconds
        }
    }
}

// ============================================================================
// Cleanup Trigger
// ============================================================================

/// What triggered the cleanup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CleanupTrigger {
    /// Threshold exceeded
    ThresholdExceeded,
    /// Component idle too long
    IdleTimeout,
    /// Component too large
    SizeLimit,
    /// Manual cleanup request
    Manual,
    /// Scheduled cleanup
    Scheduled,
}

// ============================================================================
// Cleanup Action
// ============================================================================

/// Cleanup action to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupAction {
    /// Target component
    pub component: String,
    /// Trigger reason
    pub trigger: CleanupTrigger,
    /// Items to remove
    pub items_to_remove: usize,
    /// Bytes to free
    pub bytes_to_free: u64,
    /// Priority (higher = more urgent)
    pub priority: u8,
}

impl CleanupAction {
    /// Create action for idle component
    pub fn for_idle(comp: &ComponentUsage) -> Self {
        Self {
            component: comp.name.clone(),
            trigger: CleanupTrigger::IdleTimeout,
            items_to_remove: comp.item_count,
            bytes_to_free: comp.memory_bytes,
            priority: 50,
        }
    }

    /// Create action for oversized component
    pub fn for_size(comp: &ComponentUsage, max_items: usize, max_bytes: u64) -> Self {
        let items_over = comp.item_count.saturating_sub(max_items);
        let bytes_over = comp.memory_bytes.saturating_sub(max_bytes);

        Self {
            component: comp.name.clone(),
            trigger: CleanupTrigger::SizeLimit,
            items_to_remove: items_over,
            bytes_to_free: bytes_over,
            priority: 75,
        }
    }

    /// Create action for threshold exceeded
    pub fn for_threshold(comp: &ComponentUsage, target_reduction_percent: u8) -> Self {
        let items_to_remove = comp.item_count * target_reduction_percent as usize / 100;
        let bytes_to_free = comp.memory_bytes * target_reduction_percent as u64 / 100;

        Self {
            component: comp.name.clone(),
            trigger: CleanupTrigger::ThresholdExceeded,
            items_to_remove,
            bytes_to_free,
            priority: 100,
        }
    }

    /// Create manual cleanup action
    pub fn manual(component: impl Into<String>, items: usize, bytes: u64) -> Self {
        Self {
            component: component.into(),
            trigger: CleanupTrigger::Manual,
            items_to_remove: items,
            bytes_to_free: bytes,
            priority: 100,
        }
    }
}

// ============================================================================
// Cleanup Result
// ============================================================================

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupResult {
    /// Component cleaned
    pub component: String,
    /// Items removed
    pub items_removed: usize,
    /// Bytes freed
    pub bytes_freed: u64,
    /// Whether cleanup succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl CleanupResult {
    /// Create successful result
    pub fn success(component: impl Into<String>, items: usize, bytes: u64, duration_ms: u64) -> Self {
        Self {
            component: component.into(),
            items_removed: items,
            bytes_freed: bytes,
            success: true,
            error: None,
            duration_ms,
        }
    }

    /// Create failed result
    pub fn failure(component: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            items_removed: 0,
            bytes_freed: 0,
            success: false,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }
}

// ============================================================================
// Cleanup Statistics
// ============================================================================

/// Cleanup statistics
#[derive(Debug, Default)]
pub struct CleanupStats {
    /// Total cleanup runs
    total_runs: AtomicU64,
    /// Successful cleanups
    successful: AtomicU64,
    /// Failed cleanups
    failed: AtomicU64,
    /// Total items removed
    items_removed: AtomicU64,
    /// Total bytes freed
    bytes_freed: AtomicU64,
    /// Cleanups by threshold
    by_threshold: AtomicU64,
    /// Cleanups by idle
    by_idle: AtomicU64,
    /// Cleanups by size
    by_size: AtomicU64,
    /// Manual cleanups
    manual: AtomicU64,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStatsSnapshot {
    pub total_runs: u64,
    pub successful: u64,
    pub failed: u64,
    pub items_removed: u64,
    pub bytes_freed: u64,
    pub by_threshold: u64,
    pub by_idle: u64,
    pub by_size: u64,
    pub manual: u64,
}

impl CleanupStats {
    fn snapshot(&self) -> CleanupStatsSnapshot {
        CleanupStatsSnapshot {
            total_runs: self.total_runs.load(Ordering::Relaxed),
            successful: self.successful.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            items_removed: self.items_removed.load(Ordering::Relaxed),
            bytes_freed: self.bytes_freed.load(Ordering::Relaxed),
            by_threshold: self.by_threshold.load(Ordering::Relaxed),
            by_idle: self.by_idle.load(Ordering::Relaxed),
            by_size: self.by_size.load(Ordering::Relaxed),
            manual: self.manual.load(Ordering::Relaxed),
        }
    }

    fn record(&self, result: &CleanupResult, trigger: CleanupTrigger) {
        self.total_runs.fetch_add(1, Ordering::Relaxed);

        if result.success {
            self.successful.fetch_add(1, Ordering::Relaxed);
            self.items_removed
                .fetch_add(result.items_removed as u64, Ordering::Relaxed);
            self.bytes_freed
                .fetch_add(result.bytes_freed, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }

        match trigger {
            CleanupTrigger::ThresholdExceeded => {
                self.by_threshold.fetch_add(1, Ordering::Relaxed);
            }
            CleanupTrigger::IdleTimeout => {
                self.by_idle.fetch_add(1, Ordering::Relaxed);
            }
            CleanupTrigger::SizeLimit => {
                self.by_size.fetch_add(1, Ordering::Relaxed);
            }
            CleanupTrigger::Manual => {
                self.manual.fetch_add(1, Ordering::Relaxed);
            }
            CleanupTrigger::Scheduled => {
                // No specific counter
            }
        }
    }
}

// ============================================================================
// Cleanup Handler Type
// ============================================================================

/// Type alias for cleanup handler function
pub type CleanupHandler = Box<
    dyn Fn(CleanupAction) -> Pin<Box<dyn Future<Output = CleanupResult> + Send>> + Send + Sync,
>;

// ============================================================================
// Cleanup Manager
// ============================================================================

/// Cleanup manager for automatic resource cleanup
pub struct CleanupManager {
    /// Configuration
    config: CleanupConfig,
    /// Cleanup handlers per component
    handlers: RwLock<std::collections::HashMap<String, CleanupHandler>>,
    /// Statistics
    stats: Arc<CleanupStats>,
    /// Last cleanup time (epoch ms)
    last_cleanup_ms: AtomicU64,
}

impl CleanupManager {
    /// Create a new cleanup manager
    pub fn new(config: CleanupConfig) -> Self {
        Self {
            config,
            handlers: RwLock::new(std::collections::HashMap::new()),
            stats: Arc::new(CleanupStats::default()),
            last_cleanup_ms: AtomicU64::new(0),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(CleanupConfig::default())
    }

    /// Register a cleanup handler for a component
    pub async fn register_handler(&self, component: impl Into<String>, handler: CleanupHandler) {
        let mut handlers = self.handlers.write().await;
        handlers.insert(component.into(), handler);
    }

    /// Check if cleanup is needed and generate actions
    pub async fn analyze(&self, monitor: &ResourceMonitor) -> Vec<CleanupAction> {
        if !self.config.enabled {
            return Vec::new();
        }

        let mut actions = Vec::new();
        let usage = monitor.usage().await;

        // Check if threshold exceeded
        if usage.overall_level().needs_action() {
            // Target 25% reduction for warning, 50% for critical
            let reduction = if usage.overall_level().is_critical() {
                50
            } else {
                25
            };

            for comp in &usage.components {
                if comp.memory_bytes > 0 {
                    actions.push(CleanupAction::for_threshold(comp, reduction));
                }
            }
        }

        // Check for idle components
        let idle = monitor.idle_components(self.config.min_idle_seconds).await;
        for comp in idle {
            if comp.memory_bytes > 0 {
                actions.push(CleanupAction::for_idle(&comp));
            }
        }

        // Check for oversized components
        for comp in &usage.components {
            if comp.memory_bytes > self.config.max_component_memory
                || comp.item_count > self.config.max_component_items
            {
                actions.push(CleanupAction::for_size(
                    comp,
                    self.config.max_component_items,
                    self.config.max_component_memory,
                ));
            }
        }

        // Sort by priority
        actions.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Deduplicate by component (keep highest priority)
        let mut seen = std::collections::HashSet::new();
        actions.retain(|a| seen.insert(a.component.clone()));

        actions
    }

    /// Check if cooldown has passed
    pub fn can_cleanup(&self) -> bool {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last = self.last_cleanup_ms.load(Ordering::Relaxed);
        now_ms.saturating_sub(last) >= self.config.cooldown_ms
    }

    /// Execute a cleanup action
    pub async fn execute(&self, action: CleanupAction) -> CleanupResult {
        let handlers = self.handlers.read().await;

        let result = if let Some(handler) = handlers.get(&action.component) {
            let trigger = action.trigger;
            let result = handler(action).await;
            self.stats.record(&result, trigger);
            result
        } else {
            CleanupResult::failure(&action.component, "No cleanup handler registered")
        };

        // Update last cleanup time
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_cleanup_ms.store(now_ms, Ordering::Relaxed);

        result
    }

    /// Execute multiple cleanup actions
    pub async fn execute_all(&self, actions: Vec<CleanupAction>) -> Vec<CleanupResult> {
        let mut results = Vec::with_capacity(actions.len());

        for action in actions {
            let result = self.execute(action).await;
            results.push(result);
        }

        results
    }

    /// Run automatic cleanup based on monitor state
    pub async fn auto_cleanup(&self, monitor: &ResourceMonitor) -> Vec<CleanupResult> {
        if !self.config.enabled || !self.can_cleanup() {
            return Vec::new();
        }

        let actions = self.analyze(monitor).await;
        if actions.is_empty() {
            return Vec::new();
        }

        // Record cleanup triggered in monitor
        monitor.record_cleanup();

        self.execute_all(actions).await
    }

    /// Get statistics
    pub fn stats(&self) -> CleanupStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get configuration
    pub fn config(&self) -> &CleanupConfig {
        &self.config
    }

    /// Generate cleanup report
    pub fn report(&self) -> String {
        let stats = self.stats();

        let mut report = String::new();
        report.push_str("=== Cleanup Manager Report ===\n\n");

        report.push_str(&format!("Total Runs: {}\n", stats.total_runs));
        report.push_str(&format!(
            "Success Rate: {:.1}%\n",
            if stats.total_runs > 0 {
                stats.successful as f64 / stats.total_runs as f64 * 100.0
            } else {
                100.0
            }
        ));
        report.push_str(&format!("Items Removed: {}\n", stats.items_removed));
        report.push_str(&format!(
            "Bytes Freed: {}\n\n",
            Self::format_bytes(stats.bytes_freed)
        ));

        report.push_str("By Trigger:\n");
        report.push_str(&format!("  Threshold: {}\n", stats.by_threshold));
        report.push_str(&format!("  Idle: {}\n", stats.by_idle));
        report.push_str(&format!("  Size: {}\n", stats.by_size));
        report.push_str(&format!("  Manual: {}\n", stats.manual));

        report
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
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cleanup_manager_basic() {
        let manager = CleanupManager::default_config();
        assert!(manager.config().enabled);
    }

    #[tokio::test]
    async fn test_analyze_empty() {
        let manager = CleanupManager::default_config();
        let monitor = ResourceMonitor::default_config();

        let actions = manager.analyze(&monitor).await;
        assert!(actions.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_threshold() {
        let cleanup_config = CleanupConfig::default();
        let manager = CleanupManager::new(cleanup_config);

        let monitor_config = super::super::monitor::ResourceConfig {
            memory_warning_bytes: 100,
            memory_critical_bytes: 500,
            ..Default::default()
        };
        let monitor = ResourceMonitor::new(monitor_config);

        // Add component exceeding threshold
        monitor.update_component("test", 200, 10).await;

        let actions = manager.analyze(&monitor).await;
        assert!(!actions.is_empty());
        assert_eq!(actions[0].trigger, CleanupTrigger::ThresholdExceeded);
    }

    #[tokio::test]
    async fn test_analyze_size_limit() {
        let cleanup_config = CleanupConfig {
            max_component_memory: 100,
            max_component_items: 5,
            ..Default::default()
        };
        let manager = CleanupManager::new(cleanup_config);
        let monitor = ResourceMonitor::default_config();

        // Add oversized component
        monitor.update_component("test", 200, 10).await;

        let actions = manager.analyze(&monitor).await;
        assert!(!actions.is_empty());
    }

    #[tokio::test]
    async fn test_execute_no_handler() {
        let manager = CleanupManager::default_config();
        let action = CleanupAction::manual("test", 10, 1000);

        let result = manager.execute(action).await;
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_execute_with_handler() {
        let manager = CleanupManager::default_config();

        // Register handler
        manager
            .register_handler("test", Box::new(|action| {
                Box::pin(async move {
                    CleanupResult::success(
                        action.component,
                        action.items_to_remove,
                        action.bytes_to_free,
                        10,
                    )
                })
            }))
            .await;

        let action = CleanupAction::manual("test", 10, 1000);
        let result = manager.execute(action).await;

        assert!(result.success);
        assert_eq!(result.items_removed, 10);
        assert_eq!(result.bytes_freed, 1000);
    }

    #[tokio::test]
    async fn test_stats() {
        let manager = CleanupManager::default_config();

        manager
            .register_handler("test", Box::new(|action| {
                Box::pin(async move {
                    CleanupResult::success(action.component, 5, 500, 5)
                })
            }))
            .await;

        let action = CleanupAction::manual("test", 5, 500);
        manager.execute(action).await;

        let stats = manager.stats();
        assert_eq!(stats.total_runs, 1);
        assert_eq!(stats.successful, 1);
        assert_eq!(stats.items_removed, 5);
        assert_eq!(stats.bytes_freed, 500);
        assert_eq!(stats.manual, 1);
    }

    #[test]
    fn test_cleanup_action_for_idle() {
        let comp = ComponentUsage::new("test");
        let action = CleanupAction::for_idle(&comp);
        assert_eq!(action.trigger, CleanupTrigger::IdleTimeout);
    }

    #[test]
    fn test_cleanup_action_for_size() {
        let mut comp = ComponentUsage::new("test");
        comp.memory_bytes = 2000;
        comp.item_count = 200;

        let action = CleanupAction::for_size(&comp, 100, 1000);
        assert_eq!(action.trigger, CleanupTrigger::SizeLimit);
        assert_eq!(action.items_to_remove, 100);
        assert_eq!(action.bytes_to_free, 1000);
    }

    #[test]
    fn test_cleanup_action_for_threshold() {
        let mut comp = ComponentUsage::new("test");
        comp.memory_bytes = 1000;
        comp.item_count = 100;

        let action = CleanupAction::for_threshold(&comp, 25);
        assert_eq!(action.trigger, CleanupTrigger::ThresholdExceeded);
        assert_eq!(action.items_to_remove, 25);
        assert_eq!(action.bytes_to_free, 250);
    }

    #[test]
    fn test_cleanup_result() {
        let success = CleanupResult::success("test", 10, 1000, 5);
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = CleanupResult::failure("test", "error");
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[tokio::test]
    async fn test_cooldown() {
        let config = CleanupConfig {
            cooldown_ms: 100,
            ..Default::default()
        };
        let manager = CleanupManager::new(config);

        // First cleanup should be allowed
        assert!(manager.can_cleanup());

        // Register handler and execute
        manager
            .register_handler("test", Box::new(|action| {
                Box::pin(async move {
                    CleanupResult::success(action.component, 0, 0, 0)
                })
            }))
            .await;

        manager.execute(CleanupAction::manual("test", 0, 0)).await;

        // Immediately after, should be in cooldown
        assert!(!manager.can_cleanup());

        // After cooldown
        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
        assert!(manager.can_cleanup());
    }

    #[test]
    fn test_report() {
        let manager = CleanupManager::default_config();
        let report = manager.report();
        assert!(report.contains("Cleanup Manager Report"));
    }
}
