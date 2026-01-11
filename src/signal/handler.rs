//! Signal Handler - Intercept OS signals
//!
//! Provides unified handling for SIGINT (Ctrl+C) and SIGTERM signals.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

// ============================================================================
// Signal Types
// ============================================================================

/// Types of signals handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// SIGINT (Ctrl+C)
    Interrupt,
    /// SIGTERM
    Terminate,
    /// Custom shutdown request
    Shutdown,
}

impl std::fmt::Display for SignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignalType::Interrupt => write!(f, "SIGINT"),
            SignalType::Terminate => write!(f, "SIGTERM"),
            SignalType::Shutdown => write!(f, "SHUTDOWN"),
        }
    }
}

// ============================================================================
// Signal Configuration
// ============================================================================

/// Signal handling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalConfig {
    /// Enable double Ctrl+C for force quit
    pub double_ctrl_c_force_quit: bool,
    /// Timeout for double Ctrl+C (milliseconds)
    pub double_ctrl_c_timeout_ms: u64,
    /// Grace period before force exit (milliseconds)
    pub grace_period_ms: u64,
    /// Maximum shutdown time before force exit (milliseconds)
    pub max_shutdown_time_ms: u64,
}

impl Default for SignalConfig {
    fn default() -> Self {
        Self {
            double_ctrl_c_force_quit: true,
            double_ctrl_c_timeout_ms: 2000, // 2 seconds
            grace_period_ms: 1000,          // 1 second
            max_shutdown_time_ms: 10000,    // 10 seconds
        }
    }
}

// ============================================================================
// Signal Statistics
// ============================================================================

/// Signal handling statistics
#[derive(Debug, Default)]
pub struct SignalStats {
    /// SIGINT received count
    sigint_count: AtomicU64,
    /// SIGTERM received count
    sigterm_count: AtomicU64,
    /// Shutdown requests count
    shutdown_requests: AtomicU64,
    /// Force quit triggered
    force_quit_triggered: AtomicBool,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStatsSnapshot {
    pub sigint_count: u64,
    pub sigterm_count: u64,
    pub shutdown_requests: u64,
    pub force_quit_triggered: bool,
}

impl SignalStats {
    fn snapshot(&self) -> SignalStatsSnapshot {
        SignalStatsSnapshot {
            sigint_count: self.sigint_count.load(Ordering::Relaxed),
            sigterm_count: self.sigterm_count.load(Ordering::Relaxed),
            shutdown_requests: self.shutdown_requests.load(Ordering::Relaxed),
            force_quit_triggered: self.force_quit_triggered.load(Ordering::Relaxed),
        }
    }
}

// ============================================================================
// Signal Handler
// ============================================================================

/// Signal handler for graceful shutdown
pub struct SignalHandler {
    /// Configuration
    config: SignalConfig,
    /// Shutdown signal sender
    shutdown_tx: broadcast::Sender<SignalType>,
    /// Whether shutdown has been initiated
    shutdown_initiated: AtomicBool,
    /// Last Ctrl+C timestamp (for double Ctrl+C detection)
    last_ctrl_c_ms: AtomicU64,
    /// Statistics
    stats: Arc<SignalStats>,
}

impl SignalHandler {
    /// Create a new signal handler
    pub fn new(config: SignalConfig) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        Self {
            config,
            shutdown_tx,
            shutdown_initiated: AtomicBool::new(false),
            last_ctrl_c_ms: AtomicU64::new(0),
            stats: Arc::new(SignalStats::default()),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(SignalConfig::default())
    }

    /// Subscribe to shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<SignalType> {
        self.shutdown_tx.subscribe()
    }

    /// Check if shutdown has been initiated
    pub fn is_shutdown_initiated(&self) -> bool {
        self.shutdown_initiated.load(Ordering::SeqCst)
    }

    /// Trigger shutdown programmatically
    pub fn trigger_shutdown(&self) {
        if self
            .shutdown_initiated
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.stats.shutdown_requests.fetch_add(1, Ordering::Relaxed);
            let _ = self.shutdown_tx.send(SignalType::Shutdown);
        }
    }

    /// Handle SIGINT (Ctrl+C)
    pub fn handle_sigint(&self) -> bool {
        self.stats.sigint_count.fetch_add(1, Ordering::Relaxed);

        let now_ms = Self::now_ms();
        let last = self.last_ctrl_c_ms.swap(now_ms, Ordering::SeqCst);

        // Check for double Ctrl+C
        if self.config.double_ctrl_c_force_quit
            && self.shutdown_initiated.load(Ordering::SeqCst)
            && now_ms.saturating_sub(last) < self.config.double_ctrl_c_timeout_ms
        {
            // Force quit
            self.stats.force_quit_triggered.store(true, Ordering::Relaxed);
            return true; // Indicates force quit
        }

        // Initiate shutdown
        if self
            .shutdown_initiated
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = self.shutdown_tx.send(SignalType::Interrupt);
        }

        false // Normal shutdown
    }

    /// Handle SIGTERM
    pub fn handle_sigterm(&self) {
        self.stats.sigterm_count.fetch_add(1, Ordering::Relaxed);

        if self
            .shutdown_initiated
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = self.shutdown_tx.send(SignalType::Terminate);
        }
    }

    /// Get statistics
    pub fn stats(&self) -> SignalStatsSnapshot {
        self.stats.snapshot()
    }

    /// Get configuration
    pub fn config(&self) -> &SignalConfig {
        &self.config
    }

    /// Get grace period
    pub fn grace_period(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.config.grace_period_ms)
    }

    /// Get maximum shutdown time
    pub fn max_shutdown_time(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.config.max_shutdown_time_ms)
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Install global signal handlers (Unix)
    #[cfg(unix)]
    pub async fn install(self: Arc<Self>) {
        use tokio::signal::unix::{signal, SignalKind};

        let handler_int = Arc::clone(&self);
        let handler_term = Arc::clone(&self);

        // SIGINT handler
        tokio::spawn(async move {
            let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
            loop {
                sigint.recv().await;
                if handler_int.handle_sigint() {
                    // Force quit
                    std::process::exit(130); // 128 + SIGINT(2)
                }
            }
        });

        // SIGTERM handler
        tokio::spawn(async move {
            let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
            loop {
                sigterm.recv().await;
                handler_term.handle_sigterm();
            }
        });
    }

    /// Install global signal handlers (Windows)
    #[cfg(windows)]
    pub async fn install(self: Arc<Self>) {
        let handler = Arc::clone(&self);

        // Ctrl+C handler
        tokio::spawn(async move {
            loop {
                if tokio::signal::ctrl_c().await.is_ok() {
                    if handler.handle_sigint() {
                        std::process::exit(130);
                    }
                }
            }
        });
    }

    /// Wait for shutdown signal
    pub async fn wait_for_shutdown(&self) {
        let mut rx = self.subscribe();
        let _ = rx.recv().await;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_type_display() {
        assert_eq!(SignalType::Interrupt.to_string(), "SIGINT");
        assert_eq!(SignalType::Terminate.to_string(), "SIGTERM");
        assert_eq!(SignalType::Shutdown.to_string(), "SHUTDOWN");
    }

    #[test]
    fn test_signal_config_default() {
        let config = SignalConfig::default();
        assert!(config.double_ctrl_c_force_quit);
        assert_eq!(config.grace_period_ms, 1000);
    }

    #[tokio::test]
    async fn test_signal_handler_basic() {
        let handler = SignalHandler::default_config();
        assert!(!handler.is_shutdown_initiated());
    }

    #[tokio::test]
    async fn test_trigger_shutdown() {
        let handler = SignalHandler::default_config();
        let mut rx = handler.subscribe();

        handler.trigger_shutdown();

        assert!(handler.is_shutdown_initiated());

        let signal = rx.recv().await.unwrap();
        assert_eq!(signal, SignalType::Shutdown);
    }

    #[tokio::test]
    async fn test_handle_sigint() {
        let handler = SignalHandler::default_config();
        let mut rx = handler.subscribe();

        let force_quit = handler.handle_sigint();
        assert!(!force_quit);
        assert!(handler.is_shutdown_initiated());

        let signal = rx.recv().await.unwrap();
        assert_eq!(signal, SignalType::Interrupt);
    }

    #[tokio::test]
    async fn test_handle_sigterm() {
        let handler = SignalHandler::default_config();
        let mut rx = handler.subscribe();

        handler.handle_sigterm();
        assert!(handler.is_shutdown_initiated());

        let signal = rx.recv().await.unwrap();
        assert_eq!(signal, SignalType::Terminate);
    }

    #[tokio::test]
    async fn test_double_ctrl_c() {
        let config = SignalConfig {
            double_ctrl_c_force_quit: true,
            double_ctrl_c_timeout_ms: 2000,
            ..Default::default()
        };
        let handler = SignalHandler::new(config);

        // First Ctrl+C
        let force_quit = handler.handle_sigint();
        assert!(!force_quit);

        // Second Ctrl+C within timeout
        let force_quit = handler.handle_sigint();
        assert!(force_quit);

        let stats = handler.stats();
        assert!(stats.force_quit_triggered);
        assert_eq!(stats.sigint_count, 2);
    }

    #[tokio::test]
    async fn test_stats() {
        let handler = SignalHandler::default_config();

        handler.handle_sigint();
        handler.handle_sigterm();
        handler.trigger_shutdown();

        let stats = handler.stats();
        assert_eq!(stats.sigint_count, 1);
        assert_eq!(stats.sigterm_count, 1);
        // shutdown_requests is 0 because shutdown was already initiated by sigint
    }

    #[test]
    fn test_grace_period() {
        let config = SignalConfig {
            grace_period_ms: 5000,
            ..Default::default()
        };
        let handler = SignalHandler::new(config);
        assert_eq!(handler.grace_period().as_millis(), 5000);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let handler = SignalHandler::default_config();
        let mut rx1 = handler.subscribe();
        let mut rx2 = handler.subscribe();

        handler.trigger_shutdown();

        let signal1 = rx1.recv().await.unwrap();
        let signal2 = rx2.recv().await.unwrap();

        assert_eq!(signal1, SignalType::Shutdown);
        assert_eq!(signal2, SignalType::Shutdown);
    }

    #[test]
    fn test_shutdown_only_once() {
        let handler = SignalHandler::default_config();

        handler.trigger_shutdown();
        handler.trigger_shutdown();
        handler.trigger_shutdown();

        let stats = handler.stats();
        assert_eq!(stats.shutdown_requests, 1);
    }
}
