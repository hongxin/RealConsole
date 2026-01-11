//! Shutdown Coordinator - Graceful shutdown orchestration
//!
//! Coordinates cleanup across components during shutdown.

use super::handler::{SignalHandler, SignalType};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// Shutdown Configuration
// ============================================================================

/// Shutdown configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownConfig {
    /// Timeout per cleanup phase (milliseconds)
    pub phase_timeout_ms: u64,
    /// Continue on cleanup error
    pub continue_on_error: bool,
    /// Save state on shutdown
    pub save_state: bool,
    /// Clean temporary files
    pub clean_temp_files: bool,
}

impl Default for ShutdownConfig {
    fn default() -> Self {
        Self {
            phase_timeout_ms: 5000, // 5 seconds per phase
            continue_on_error: true,
            save_state: true,
            clean_temp_files: true,
        }
    }
}

// ============================================================================
// Shutdown Phases
// ============================================================================

/// Shutdown phases (executed in order)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ShutdownPhase {
    /// Cancel ongoing operations (streams, requests)
    CancelOperations = 0,
    /// Save state (memory, session, config)
    SaveState = 1,
    /// Release resources (connections, handles)
    ReleaseResources = 2,
    /// Final cleanup (temp files, locks)
    FinalCleanup = 3,
}

impl ShutdownPhase {
    /// Get all phases in order
    pub fn all() -> Vec<ShutdownPhase> {
        vec![
            ShutdownPhase::CancelOperations,
            ShutdownPhase::SaveState,
            ShutdownPhase::ReleaseResources,
            ShutdownPhase::FinalCleanup,
        ]
    }

    /// Phase name
    pub fn name(&self) -> &'static str {
        match self {
            ShutdownPhase::CancelOperations => "Cancel Operations",
            ShutdownPhase::SaveState => "Save State",
            ShutdownPhase::ReleaseResources => "Release Resources",
            ShutdownPhase::FinalCleanup => "Final Cleanup",
        }
    }
}

// ============================================================================
// Shutdown Result
// ============================================================================

/// Result of a cleanup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResult {
    /// Hook name
    pub name: String,
    /// Phase
    pub phase: ShutdownPhase,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ShutdownResult {
    /// Create success result
    pub fn success(name: impl Into<String>, phase: ShutdownPhase, duration_ms: u64) -> Self {
        Self {
            name: name.into(),
            phase,
            success: true,
            error: None,
            duration_ms,
        }
    }

    /// Create failure result
    pub fn failure(
        name: impl Into<String>,
        phase: ShutdownPhase,
        error: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            phase,
            success: false,
            error: Some(error.into()),
            duration_ms: 0,
        }
    }
}

// ============================================================================
// Cleanup Hook
// ============================================================================

/// Type alias for cleanup hook function
pub type CleanupFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

/// Cleanup hook registration
pub struct CleanupHook {
    /// Hook name
    pub name: String,
    /// Shutdown phase
    pub phase: ShutdownPhase,
    /// Priority within phase (higher = runs first)
    pub priority: u8,
    /// Cleanup function
    pub cleanup: CleanupFn,
}

impl CleanupHook {
    /// Create a new cleanup hook
    pub fn new(
        name: impl Into<String>,
        phase: ShutdownPhase,
        priority: u8,
        cleanup: CleanupFn,
    ) -> Self {
        Self {
            name: name.into(),
            phase,
            priority,
            cleanup,
        }
    }

    /// Create for cancel operations phase
    pub fn cancel(name: impl Into<String>, priority: u8, cleanup: CleanupFn) -> Self {
        Self::new(name, ShutdownPhase::CancelOperations, priority, cleanup)
    }

    /// Create for save state phase
    pub fn save(name: impl Into<String>, priority: u8, cleanup: CleanupFn) -> Self {
        Self::new(name, ShutdownPhase::SaveState, priority, cleanup)
    }

    /// Create for release resources phase
    pub fn release(name: impl Into<String>, priority: u8, cleanup: CleanupFn) -> Self {
        Self::new(name, ShutdownPhase::ReleaseResources, priority, cleanup)
    }

    /// Create for final cleanup phase
    pub fn final_cleanup(name: impl Into<String>, priority: u8, cleanup: CleanupFn) -> Self {
        Self::new(name, ShutdownPhase::FinalCleanup, priority, cleanup)
    }
}

// ============================================================================
// Shutdown Statistics
// ============================================================================

/// Shutdown statistics
#[derive(Debug, Default)]
pub struct ShutdownStats {
    /// Total hooks registered
    hooks_registered: AtomicUsize,
    /// Hooks executed
    hooks_executed: AtomicU64,
    /// Hooks succeeded
    hooks_succeeded: AtomicU64,
    /// Hooks failed
    hooks_failed: AtomicU64,
    /// Total shutdown time (ms)
    total_shutdown_ms: AtomicU64,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownStatsSnapshot {
    pub hooks_registered: usize,
    pub hooks_executed: u64,
    pub hooks_succeeded: u64,
    pub hooks_failed: u64,
    pub total_shutdown_ms: u64,
}

impl ShutdownStats {
    fn snapshot(&self) -> ShutdownStatsSnapshot {
        ShutdownStatsSnapshot {
            hooks_registered: self.hooks_registered.load(Ordering::Relaxed),
            hooks_executed: self.hooks_executed.load(Ordering::Relaxed),
            hooks_succeeded: self.hooks_succeeded.load(Ordering::Relaxed),
            hooks_failed: self.hooks_failed.load(Ordering::Relaxed),
            total_shutdown_ms: self.total_shutdown_ms.load(Ordering::Relaxed),
        }
    }
}

// ============================================================================
// Shutdown Guard
// ============================================================================

/// RAII guard for automatic cleanup hook removal
pub struct ShutdownGuard {
    coordinator: Arc<ShutdownCoordinator>,
    hook_name: String,
}

impl ShutdownGuard {
    fn new(coordinator: Arc<ShutdownCoordinator>, hook_name: String) -> Self {
        Self {
            coordinator,
            hook_name,
        }
    }
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        // Remove hook when guard is dropped
        let coordinator = Arc::clone(&self.coordinator);
        let name = self.hook_name.clone();
        tokio::spawn(async move {
            coordinator.unregister(&name).await;
        });
    }
}

// ============================================================================
// Shutdown Coordinator
// ============================================================================

/// Coordinates graceful shutdown across components
pub struct ShutdownCoordinator {
    /// Configuration
    config: ShutdownConfig,
    /// Registered cleanup hooks
    hooks: RwLock<Vec<CleanupHook>>,
    /// Statistics
    stats: Arc<ShutdownStats>,
    /// Shutdown results
    results: RwLock<Vec<ShutdownResult>>,
}

impl ShutdownCoordinator {
    /// Create a new shutdown coordinator
    pub fn new(config: ShutdownConfig) -> Self {
        Self {
            config,
            hooks: RwLock::new(Vec::new()),
            stats: Arc::new(ShutdownStats::default()),
            results: RwLock::new(Vec::new()),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(ShutdownConfig::default())
    }

    /// Register a cleanup hook
    pub async fn register(&self, hook: CleanupHook) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        self.stats.hooks_registered.fetch_add(1, Ordering::Relaxed);
    }

    /// Register with guard for automatic removal
    pub async fn register_guarded(
        self: &Arc<Self>,
        hook: CleanupHook,
    ) -> ShutdownGuard {
        let name = hook.name.clone();
        self.register(hook).await;
        ShutdownGuard::new(Arc::clone(self), name)
    }

    /// Unregister a cleanup hook by name
    pub async fn unregister(&self, name: &str) -> bool {
        let mut hooks = self.hooks.write().await;
        let len_before = hooks.len();
        hooks.retain(|h| h.name != name);
        let removed = hooks.len() < len_before;
        if removed {
            self.stats.hooks_registered.fetch_sub(1, Ordering::Relaxed);
        }
        removed
    }

    /// Execute all cleanup hooks
    pub async fn shutdown(&self) -> Vec<ShutdownResult> {
        let start = Instant::now();
        let mut all_results = Vec::new();

        for phase in ShutdownPhase::all() {
            let phase_results = self.execute_phase(phase).await;
            all_results.extend(phase_results);
        }

        let total_ms = start.elapsed().as_millis() as u64;
        self.stats.total_shutdown_ms.store(total_ms, Ordering::Relaxed);

        // Store results
        let mut results = self.results.write().await;
        *results = all_results.clone();

        all_results
    }

    /// Execute a single phase
    async fn execute_phase(&self, phase: ShutdownPhase) -> Vec<ShutdownResult> {
        let hooks = self.hooks.read().await;

        // Filter and sort hooks for this phase
        let mut phase_hooks: Vec<_> = hooks
            .iter()
            .filter(|h| h.phase == phase)
            .collect();

        // Sort by priority (higher first)
        phase_hooks.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut results = Vec::new();
        let timeout = Duration::from_millis(self.config.phase_timeout_ms);

        for hook in phase_hooks {
            let start = Instant::now();
            self.stats.hooks_executed.fetch_add(1, Ordering::Relaxed);

            // Execute with timeout
            let cleanup_future = (hook.cleanup)();
            let result = match tokio::time::timeout(timeout, cleanup_future).await {
                Ok(Ok(())) => {
                    self.stats.hooks_succeeded.fetch_add(1, Ordering::Relaxed);
                    ShutdownResult::success(
                        &hook.name,
                        phase,
                        start.elapsed().as_millis() as u64,
                    )
                }
                Ok(Err(e)) => {
                    self.stats.hooks_failed.fetch_add(1, Ordering::Relaxed);
                    ShutdownResult::failure(&hook.name, phase, e)
                }
                Err(_) => {
                    self.stats.hooks_failed.fetch_add(1, Ordering::Relaxed);
                    ShutdownResult::failure(&hook.name, phase, "Timeout")
                }
            };

            results.push(result.clone());

            // Stop on error if configured
            if !result.success && !self.config.continue_on_error {
                break;
            }
        }

        results
    }

    /// Execute shutdown with signal handler
    pub async fn shutdown_with_handler(&self, handler: &SignalHandler) -> Vec<ShutdownResult> {
        // Wait for grace period
        tokio::time::sleep(handler.grace_period()).await;

        // Execute shutdown with max timeout
        let max_time = handler.max_shutdown_time();
        match tokio::time::timeout(max_time, self.shutdown()).await {
            Ok(results) => results,
            Err(_) => {
                vec![ShutdownResult::failure(
                    "shutdown",
                    ShutdownPhase::FinalCleanup,
                    "Shutdown timeout exceeded",
                )]
            }
        }
    }

    /// Get statistics
    pub fn stats(&self) -> ShutdownStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get results from last shutdown
    pub async fn results(&self) -> Vec<ShutdownResult> {
        self.results.read().await.clone()
    }

    /// Get configuration
    pub fn config(&self) -> &ShutdownConfig {
        &self.config
    }

    /// Generate shutdown report
    pub async fn report(&self) -> String {
        let stats = self.stats();
        let results = self.results().await;

        let mut report = String::new();
        report.push_str("=== Shutdown Report ===\n\n");

        // Statistics
        report.push_str("Statistics:\n");
        report.push_str(&format!("  Hooks Registered: {}\n", stats.hooks_registered));
        report.push_str(&format!("  Hooks Executed: {}\n", stats.hooks_executed));
        report.push_str(&format!("  Succeeded: {}\n", stats.hooks_succeeded));
        report.push_str(&format!("  Failed: {}\n", stats.hooks_failed));
        report.push_str(&format!("  Total Time: {}ms\n\n", stats.total_shutdown_ms));

        // Results by phase
        if !results.is_empty() {
            report.push_str("Results:\n");
            for phase in ShutdownPhase::all() {
                let phase_results: Vec<_> = results.iter().filter(|r| r.phase == phase).collect();
                if !phase_results.is_empty() {
                    report.push_str(&format!("\n  {}:\n", phase.name()));
                    for r in phase_results {
                        let status = if r.success { "OK" } else { "FAIL" };
                        report.push_str(&format!(
                            "    [{}] {} ({}ms)\n",
                            status, r.name, r.duration_ms
                        ));
                        if let Some(err) = &r.error {
                            report.push_str(&format!("      Error: {}\n", err));
                        }
                    }
                }
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

    #[test]
    fn test_shutdown_phase_order() {
        let phases = ShutdownPhase::all();
        assert_eq!(phases.len(), 4);
        assert_eq!(phases[0], ShutdownPhase::CancelOperations);
        assert_eq!(phases[3], ShutdownPhase::FinalCleanup);
    }

    #[test]
    fn test_shutdown_config_default() {
        let config = ShutdownConfig::default();
        assert!(config.continue_on_error);
        assert!(config.save_state);
    }

    #[tokio::test]
    async fn test_coordinator_basic() {
        let coordinator = ShutdownCoordinator::default_config();

        coordinator
            .register(CleanupHook::new(
                "test",
                ShutdownPhase::SaveState,
                50,
                Box::new(|| Box::pin(async { Ok(()) })),
            ))
            .await;

        let stats = coordinator.stats();
        assert_eq!(stats.hooks_registered, 1);
    }

    #[tokio::test]
    async fn test_shutdown_success() {
        let coordinator = ShutdownCoordinator::default_config();

        coordinator
            .register(CleanupHook::save(
                "test_save",
                50,
                Box::new(|| Box::pin(async { Ok(()) })),
            ))
            .await;

        let results = coordinator.shutdown().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        let stats = coordinator.stats();
        assert_eq!(stats.hooks_succeeded, 1);
    }

    #[tokio::test]
    async fn test_shutdown_failure() {
        let coordinator = ShutdownCoordinator::default_config();

        coordinator
            .register(CleanupHook::save(
                "test_fail",
                50,
                Box::new(|| Box::pin(async { Err("Test error".to_string()) })),
            ))
            .await;

        let results = coordinator.shutdown().await;
        assert_eq!(results.len(), 1);
        assert!(!results[0].success);
        assert_eq!(results[0].error.as_deref(), Some("Test error"));

        let stats = coordinator.stats();
        assert_eq!(stats.hooks_failed, 1);
    }

    #[tokio::test]
    async fn test_phase_ordering() {
        let coordinator = ShutdownCoordinator::default_config();
        let order = Arc::new(RwLock::new(Vec::new()));

        // Register hooks in reverse order
        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::final_cleanup(
                "final",
                50,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push(4);
                        Ok(())
                    })
                }),
            ))
            .await;

        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::cancel(
                "cancel",
                50,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push(1);
                        Ok(())
                    })
                }),
            ))
            .await;

        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::release(
                "release",
                50,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push(3);
                        Ok(())
                    })
                }),
            ))
            .await;

        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::save(
                "save",
                50,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push(2);
                        Ok(())
                    })
                }),
            ))
            .await;

        coordinator.shutdown().await;

        let executed = order.read().await;
        assert_eq!(*executed, vec![1, 2, 3, 4]);
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let coordinator = ShutdownCoordinator::default_config();
        let order = Arc::new(RwLock::new(Vec::new()));

        // Register hooks with different priorities
        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::save(
                "low",
                10,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push("low");
                        Ok(())
                    })
                }),
            ))
            .await;

        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::save(
                "high",
                100,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push("high");
                        Ok(())
                    })
                }),
            ))
            .await;

        let order_clone = Arc::clone(&order);
        coordinator
            .register(CleanupHook::save(
                "medium",
                50,
                Box::new(move || {
                    let order = Arc::clone(&order_clone);
                    Box::pin(async move {
                        order.write().await.push("medium");
                        Ok(())
                    })
                }),
            ))
            .await;

        coordinator.shutdown().await;

        let executed = order.read().await;
        assert_eq!(*executed, vec!["high", "medium", "low"]);
    }

    #[tokio::test]
    async fn test_unregister() {
        let coordinator = ShutdownCoordinator::default_config();

        coordinator
            .register(CleanupHook::save(
                "test",
                50,
                Box::new(|| Box::pin(async { Ok(()) })),
            ))
            .await;

        let removed = coordinator.unregister("test").await;
        assert!(removed);

        let stats = coordinator.stats();
        assert_eq!(stats.hooks_registered, 0);
    }

    #[tokio::test]
    async fn test_timeout() {
        let config = ShutdownConfig {
            phase_timeout_ms: 50,
            ..Default::default()
        };
        let coordinator = ShutdownCoordinator::new(config);

        coordinator
            .register(CleanupHook::save(
                "slow",
                50,
                Box::new(|| {
                    Box::pin(async {
                        tokio::time::sleep(Duration::from_millis(200)).await;
                        Ok(())
                    })
                }),
            ))
            .await;

        let results = coordinator.shutdown().await;
        assert!(!results[0].success);
        assert_eq!(results[0].error.as_deref(), Some("Timeout"));
    }

    #[tokio::test]
    async fn test_report() {
        let coordinator = ShutdownCoordinator::default_config();

        coordinator
            .register(CleanupHook::save(
                "test",
                50,
                Box::new(|| Box::pin(async { Ok(()) })),
            ))
            .await;

        coordinator.shutdown().await;

        let report = coordinator.report().await;
        assert!(report.contains("Shutdown Report"));
        assert!(report.contains("test"));
    }

    #[test]
    fn test_shutdown_result() {
        let success = ShutdownResult::success("test", ShutdownPhase::SaveState, 100);
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = ShutdownResult::failure("test", ShutdownPhase::SaveState, "error");
        assert!(!failure.success);
        assert!(failure.error.is_some());
    }

    #[test]
    fn test_cleanup_hook_constructors() {
        let _cancel = CleanupHook::cancel("c", 50, Box::new(|| Box::pin(async { Ok(()) })));
        let _save = CleanupHook::save("s", 50, Box::new(|| Box::pin(async { Ok(()) })));
        let _release = CleanupHook::release("r", 50, Box::new(|| Box::pin(async { Ok(()) })));
        let _final = CleanupHook::final_cleanup("f", 50, Box::new(|| Box::pin(async { Ok(()) })));
    }
}
