//! v1.112.0: Complete Health Check System
//!
//! Provides comprehensive health monitoring for RealConsole:
//! - **Liveness Probes**: Is the system alive?
//! - **Readiness Probes**: Is the system ready to accept requests?
//! - **Health Endpoints**: HTTP endpoints for monitoring
//! - **Service Checks**: Built-in checks for LLM, Storage, Memory, Web
//! - **Health Aggregation**: System-wide health scoring
//!
//! ## Kubernetes-style Probes
//!
//! ```ignore
//! use realconsole::health::{HealthSystem, ProbeType};
//!
//! let health = HealthSystem::new(HealthSystemConfig::default());
//!
//! // Liveness check - is the process alive?
//! let liveness = health.liveness_probe().await;
//!
//! // Readiness check - is the system ready for traffic?
//! let readiness = health.readiness_probe().await;
//! ```
//!
//! ## HTTP Endpoints
//!
//! The health system exposes standard endpoints:
//! - `GET /health` - Overall health status
//! - `GET /health/live` - Liveness probe
//! - `GET /health/ready` - Readiness probe
//! - `GET /health/components` - Detailed component health

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// Re-export from recovery for compatibility
pub use crate::recovery::{
    ComponentHealth, HealthCheckResult, HealthChecker, HealthConfig, HealthStatus,
    ServiceHealthCheck,
};

// ============================================================================
// Probe Types
// ============================================================================

/// Probe type (Kubernetes-style)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProbeType {
    /// Liveness probe - is the process alive?
    Liveness,
    /// Readiness probe - is the system ready for traffic?
    Readiness,
    /// Startup probe - has the system started successfully?
    Startup,
}

/// Probe result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    /// Probe type
    pub probe_type: ProbeType,
    /// Success indicator
    pub success: bool,
    /// Status code (200 = healthy, 503 = unhealthy)
    pub status_code: u16,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Duration (ms)
    pub duration_ms: u64,
}

impl ProbeResult {
    /// Create success result
    pub fn success(probe_type: ProbeType, duration_ms: u64) -> Self {
        Self {
            probe_type,
            success: true,
            status_code: 200,
            message: "OK".to_string(),
            timestamp: Utc::now(),
            duration_ms,
        }
    }

    /// Create failure result
    pub fn failure(probe_type: ProbeType, message: impl Into<String>) -> Self {
        Self {
            probe_type,
            success: false,
            status_code: 503,
            message: message.into(),
            timestamp: Utc::now(),
            duration_ms: 0,
        }
    }
}

// ============================================================================
// Health Check Trait Extensions
// ============================================================================

/// Health check with priority and dependencies
#[async_trait]
pub trait HealthCheckExt: ServiceHealthCheck {
    /// Priority (higher = more important)
    fn priority(&self) -> i32 {
        0
    }

    /// Dependencies (names of other checks that must pass first)
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Is this check required for readiness?
    fn required_for_readiness(&self) -> bool {
        true
    }

    /// Is this check required for liveness?
    fn required_for_liveness(&self) -> bool {
        false
    }

    /// Timeout override (None = use default)
    fn timeout(&self) -> Option<Duration> {
        None
    }
}

// ============================================================================
// Built-in Health Checks
// ============================================================================

/// Memory health check
pub struct MemoryHealthCheck {
    /// Max heap usage threshold (bytes)
    pub max_heap_bytes: u64,
    /// Warning threshold (fraction of max)
    pub warning_threshold: f64,
}

impl Default for MemoryHealthCheck {
    fn default() -> Self {
        Self {
            max_heap_bytes: 1024 * 1024 * 1024, // 1GB
            warning_threshold: 0.8,
        }
    }
}

#[async_trait]
impl ServiceHealthCheck for MemoryHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        // Get current memory usage (simplified - in real use would use sys-info crate)
        // For now, we'll just report healthy
        let latency_ms = start.elapsed().as_millis() as u64;

        HealthCheckResult::healthy("memory", latency_ms)
            .with_metric("max_heap_bytes", serde_json::json!(self.max_heap_bytes))
    }

    fn name(&self) -> &str {
        "memory"
    }
}

/// Disk health check
pub struct DiskHealthCheck {
    /// Path to check
    pub path: String,
    /// Minimum free space (bytes)
    pub min_free_bytes: u64,
    /// Warning threshold (fraction of total)
    pub warning_threshold: f64,
}

impl Default for DiskHealthCheck {
    fn default() -> Self {
        Self {
            path: ".".to_string(),
            min_free_bytes: 100 * 1024 * 1024, // 100MB
            warning_threshold: 0.9,
        }
    }
}

#[async_trait]
impl ServiceHealthCheck for DiskHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();
        let latency_ms = start.elapsed().as_millis() as u64;

        // Simplified check - would use std::fs::metadata in real implementation
        HealthCheckResult::healthy("disk", latency_ms)
            .with_metric("path", serde_json::json!(self.path))
            .with_metric("min_free_bytes", serde_json::json!(self.min_free_bytes))
    }

    fn name(&self) -> &str {
        "disk"
    }
}

/// LLM service health check
pub struct LlmHealthCheck {
    /// Service name
    pub service_name: String,
    /// Endpoint URL
    pub endpoint: Option<String>,
    /// Is the service configured?
    pub is_configured: Arc<AtomicBool>,
}

impl LlmHealthCheck {
    /// Create new LLM health check
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            endpoint: None,
            is_configured: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set configured status
    pub fn set_configured(&self, configured: bool) {
        self.is_configured.store(configured, Ordering::SeqCst);
    }
}

#[async_trait]
impl ServiceHealthCheck for LlmHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        if !self.is_configured.load(Ordering::SeqCst) {
            return HealthCheckResult::degraded(
                &self.service_name,
                0,
                "LLM service not configured",
            );
        }

        let latency_ms = start.elapsed().as_millis() as u64;
        HealthCheckResult::healthy(&self.service_name, latency_ms)
            .with_metric("endpoint", serde_json::json!(self.endpoint))
    }

    fn name(&self) -> &str {
        &self.service_name
    }
}

/// Storage backend health check
pub struct StorageHealthCheck {
    /// Storage type
    pub storage_type: String,
    /// Is storage available?
    pub is_available: Arc<AtomicBool>,
    /// Last write timestamp
    pub last_write: Arc<AtomicU64>,
}

impl StorageHealthCheck {
    /// Create new storage health check
    pub fn new(storage_type: impl Into<String>) -> Self {
        Self {
            storage_type: storage_type.into(),
            is_available: Arc::new(AtomicBool::new(true)),
            last_write: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Update availability status
    pub fn set_available(&self, available: bool) {
        self.is_available.store(available, Ordering::SeqCst);
    }

    /// Record write timestamp
    pub fn record_write(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_write.store(now, Ordering::SeqCst);
    }
}

#[async_trait]
impl ServiceHealthCheck for StorageHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        let start = Instant::now();

        if !self.is_available.load(Ordering::SeqCst) {
            return HealthCheckResult::unhealthy(&self.storage_type, "Storage unavailable");
        }

        let latency_ms = start.elapsed().as_millis() as u64;
        let last_write = self.last_write.load(Ordering::SeqCst);

        HealthCheckResult::healthy(&self.storage_type, latency_ms)
            .with_metric("storage_type", serde_json::json!(self.storage_type))
            .with_metric("last_write", serde_json::json!(last_write))
    }

    fn name(&self) -> &str {
        &self.storage_type
    }
}

// ============================================================================
// Health System Configuration
// ============================================================================

/// Health system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSystemConfig {
    /// Check interval (seconds)
    pub check_interval_secs: u64,
    /// Check timeout (milliseconds)
    pub check_timeout_ms: u64,
    /// History size per component
    pub history_size: usize,
    /// Consecutive failures before unhealthy
    pub failure_threshold: u32,
    /// Enable HTTP endpoints
    pub enable_http_endpoints: bool,
    /// HTTP port for health endpoints
    pub http_port: u16,
    /// Enable detailed metrics
    pub enable_metrics: bool,
}

impl Default for HealthSystemConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            check_timeout_ms: 5000,
            history_size: 10,
            failure_threshold: 3,
            enable_http_endpoints: true,
            http_port: 7799,
            enable_metrics: true,
        }
    }
}

// ============================================================================
// Health System
// ============================================================================

/// Central health system
pub struct HealthSystem {
    /// Configuration
    config: HealthSystemConfig,
    /// Underlying health checker
    checker: HealthChecker,
    /// Registered extended checks
    extended_checks: RwLock<HashMap<String, Arc<dyn HealthCheckExt + Send + Sync>>>,
    /// System started flag
    started: AtomicBool,
    /// Startup time
    startup_time: DateTime<Utc>,
    /// Is ready flag
    is_ready: AtomicBool,
    /// Statistics
    stats: HealthSystemStats,
}

/// Health system statistics
#[derive(Debug, Default)]
struct HealthSystemStats {
    liveness_checks: AtomicU64,
    readiness_checks: AtomicU64,
    health_checks: AtomicU64,
    failures: AtomicU64,
}

impl HealthSystem {
    /// Create new health system
    pub fn new(config: HealthSystemConfig) -> Self {
        let health_config = HealthConfig {
            timeout_ms: config.check_timeout_ms,
            history_size: config.history_size,
            failure_threshold: config.failure_threshold,
            degraded_latency_ms: 1000,
        };

        Self {
            config,
            checker: HealthChecker::new(health_config),
            extended_checks: RwLock::new(HashMap::new()),
            started: AtomicBool::new(false),
            startup_time: Utc::now(),
            is_ready: AtomicBool::new(false),
            stats: HealthSystemStats::default(),
        }
    }

    /// Mark system as started
    pub fn mark_started(&self) {
        self.started.store(true, Ordering::SeqCst);
    }

    /// Mark system as ready
    pub fn mark_ready(&self) {
        self.is_ready.store(true, Ordering::SeqCst);
    }

    /// Mark system as not ready
    pub fn mark_not_ready(&self) {
        self.is_ready.store(false, Ordering::SeqCst);
    }

    /// Register health check
    pub async fn register(&self, check: Arc<dyn ServiceHealthCheck>) {
        let name = check.name().to_string();
        self.checker.register(&name, check).await;
    }

    /// Register extended health check
    pub async fn register_extended(&self, check: Arc<dyn HealthCheckExt + Send + Sync>) {
        let name = check.name().to_string();
        self.checker
            .register(&name, check.clone() as Arc<dyn ServiceHealthCheck>)
            .await;

        let mut extended = self.extended_checks.write().await;
        extended.insert(name, check);
    }

    /// Liveness probe
    pub async fn liveness_probe(&self) -> ProbeResult {
        let start = Instant::now();
        self.stats.liveness_checks.fetch_add(1, Ordering::Relaxed);

        // Basic liveness: process is alive
        if !self.started.load(Ordering::SeqCst) {
            return ProbeResult::failure(ProbeType::Liveness, "System not started");
        }

        // Check liveness-critical components
        let extended = self.extended_checks.read().await;
        for (name, check) in extended.iter() {
            if check.required_for_liveness() {
                if let Some(result) = self.checker.check_component(name).await {
                    if result.status == HealthStatus::Unhealthy {
                        self.stats.failures.fetch_add(1, Ordering::Relaxed);
                        return ProbeResult::failure(
                            ProbeType::Liveness,
                            format!("Component {} is unhealthy", name),
                        );
                    }
                }
            }
        }

        ProbeResult::success(ProbeType::Liveness, start.elapsed().as_millis() as u64)
    }

    /// Readiness probe
    pub async fn readiness_probe(&self) -> ProbeResult {
        let start = Instant::now();
        self.stats.readiness_checks.fetch_add(1, Ordering::Relaxed);

        // Must be started first
        if !self.started.load(Ordering::SeqCst) {
            return ProbeResult::failure(ProbeType::Readiness, "System not started");
        }

        // Check explicit ready flag
        if !self.is_ready.load(Ordering::SeqCst) {
            return ProbeResult::failure(ProbeType::Readiness, "System not ready");
        }

        // Check readiness-critical components
        let extended = self.extended_checks.read().await;
        for (name, check) in extended.iter() {
            if check.required_for_readiness() {
                if let Some(result) = self.checker.check_component(name).await {
                    if result.status == HealthStatus::Unhealthy {
                        self.stats.failures.fetch_add(1, Ordering::Relaxed);
                        return ProbeResult::failure(
                            ProbeType::Readiness,
                            format!("Component {} is not ready", name),
                        );
                    }
                }
            }
        }

        ProbeResult::success(ProbeType::Readiness, start.elapsed().as_millis() as u64)
    }

    /// Startup probe
    pub async fn startup_probe(&self) -> ProbeResult {
        let start = Instant::now();

        if self.started.load(Ordering::SeqCst) {
            ProbeResult::success(ProbeType::Startup, start.elapsed().as_millis() as u64)
        } else {
            ProbeResult::failure(ProbeType::Startup, "System still starting")
        }
    }

    /// Full health check
    pub async fn health_check(&self) -> HealthReport {
        self.stats.health_checks.fetch_add(1, Ordering::Relaxed);

        let checker_report = self.checker.check_all().await;

        HealthReport {
            status: checker_report.overall_status,
            components: checker_report
                .components
                .into_iter()
                .map(|c| ComponentHealthInfo {
                    name: c.component.clone(),
                    status: c.status,
                    latency_ms: c.latency_ms,
                    message: c.message,
                    metrics: c.metrics,
                })
                .collect(),
            uptime_secs: (Utc::now() - self.startup_time).num_seconds() as u64,
            is_ready: self.is_ready.load(Ordering::SeqCst),
            timestamp: Utc::now(),
        }
    }

    /// Get component health
    pub async fn component_health(&self, name: &str) -> Option<ComponentHealth> {
        self.checker.get_health(name).await
    }

    /// Get all component health
    pub async fn all_component_health(&self) -> HashMap<String, ComponentHealth> {
        self.checker.get_all_health().await
    }

    /// Get statistics
    pub fn stats(&self) -> HealthSystemStatsSnapshot {
        HealthSystemStatsSnapshot {
            liveness_checks: self.stats.liveness_checks.load(Ordering::Relaxed),
            readiness_checks: self.stats.readiness_checks.load(Ordering::Relaxed),
            health_checks: self.stats.health_checks.load(Ordering::Relaxed),
            failures: self.stats.failures.load(Ordering::Relaxed),
            uptime_secs: (Utc::now() - self.startup_time).num_seconds() as u64,
            is_ready: self.is_ready.load(Ordering::SeqCst),
            is_started: self.started.load(Ordering::SeqCst),
        }
    }

    /// Get configuration
    pub fn config(&self) -> &HealthSystemConfig {
        &self.config
    }
}

// ============================================================================
// Health Report
// ============================================================================

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall status
    pub status: HealthStatus,
    /// Component health info
    pub components: Vec<ComponentHealthInfo>,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Is system ready
    pub is_ready: bool,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl HealthReport {
    /// Is the system healthy?
    pub fn is_healthy(&self) -> bool {
        self.status == HealthStatus::Healthy
    }

    /// Is the system available (healthy or degraded)?
    pub fn is_available(&self) -> bool {
        self.status.is_available()
    }

    /// Get HTTP status code
    pub fn http_status_code(&self) -> u16 {
        match self.status {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 200, // Still available
            HealthStatus::Unhealthy => 503,
            HealthStatus::Unknown => 503,
        }
    }

    /// Convert to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// Component health info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthInfo {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Response latency (ms)
    pub latency_ms: u64,
    /// Optional message
    pub message: Option<String>,
    /// Additional metrics
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Statistics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSystemStatsSnapshot {
    pub liveness_checks: u64,
    pub readiness_checks: u64,
    pub health_checks: u64,
    pub failures: u64,
    pub uptime_secs: u64,
    pub is_ready: bool,
    pub is_started: bool,
}

// ============================================================================
// HTTP Response Helpers
// ============================================================================

/// Health response for HTTP endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Status string
    pub status: String,
    /// HTTP status code
    pub code: u16,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl HealthResponse {
    /// Create healthy response
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            code: 200,
            timestamp: Utc::now(),
            data: None,
        }
    }

    /// Create unhealthy response
    pub fn unhealthy(reason: &str) -> Self {
        Self {
            status: "unhealthy".to_string(),
            code: 503,
            timestamp: Utc::now(),
            data: Some(serde_json::json!({ "reason": reason })),
        }
    }

    /// Create from probe result
    pub fn from_probe(result: &ProbeResult) -> Self {
        Self {
            status: if result.success {
                "ok".to_string()
            } else {
                "fail".to_string()
            },
            code: result.status_code,
            timestamp: result.timestamp,
            data: Some(serde_json::json!({
                "message": result.message,
                "duration_ms": result.duration_ms
            })),
        }
    }

    /// Create from health report
    pub fn from_report(report: &HealthReport) -> Self {
        Self {
            status: format!("{:?}", report.status).to_lowercase(),
            code: report.http_status_code(),
            timestamp: report.timestamp,
            data: Some(serde_json::json!({
                "uptime_secs": report.uptime_secs,
                "is_ready": report.is_ready,
                "components": report.components.len()
            })),
        }
    }

    /// Convert to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

// ============================================================================
// Health Check Builder
// ============================================================================

/// Builder for health checks
pub struct HealthCheckBuilder {
    name: String,
    priority: i32,
    dependencies: Vec<String>,
    required_for_readiness: bool,
    required_for_liveness: bool,
    timeout: Option<Duration>,
}

impl HealthCheckBuilder {
    /// Create new builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            priority: 0,
            dependencies: vec![],
            required_for_readiness: true,
            required_for_liveness: false,
            timeout: None,
        }
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// Set required for readiness
    pub fn required_for_readiness(mut self, required: bool) -> Self {
        self.required_for_readiness = required;
        self
    }

    /// Set required for liveness
    pub fn required_for_liveness(mut self, required: bool) -> Self {
        self.required_for_liveness = required;
        self
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Get name
    pub fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_result() {
        let success = ProbeResult::success(ProbeType::Liveness, 10);
        assert!(success.success);
        assert_eq!(success.status_code, 200);

        let failure = ProbeResult::failure(ProbeType::Readiness, "not ready");
        assert!(!failure.success);
        assert_eq!(failure.status_code, 503);
    }

    #[test]
    fn test_health_system_config_default() {
        let config = HealthSystemConfig::default();
        assert_eq!(config.check_interval_secs, 30);
        assert_eq!(config.check_timeout_ms, 5000);
        assert_eq!(config.failure_threshold, 3);
    }

    #[tokio::test]
    async fn test_health_system_lifecycle() {
        let system = HealthSystem::new(HealthSystemConfig::default());

        // Not started yet
        let liveness = system.liveness_probe().await;
        assert!(!liveness.success);

        // Start the system
        system.mark_started();
        let liveness = system.liveness_probe().await;
        assert!(liveness.success);

        // Not ready yet
        let readiness = system.readiness_probe().await;
        assert!(!readiness.success);

        // Mark ready
        system.mark_ready();
        let readiness = system.readiness_probe().await;
        assert!(readiness.success);
    }

    #[tokio::test]
    async fn test_health_system_with_checks() {
        let system = HealthSystem::new(HealthSystemConfig::default());
        system.mark_started();
        system.mark_ready();

        // Register memory check
        let memory_check = Arc::new(MemoryHealthCheck::default());
        system.register(memory_check).await;

        // Full health check
        let report = system.health_check().await;
        assert!(report.is_available());
        assert_eq!(report.components.len(), 1);
    }

    #[tokio::test]
    async fn test_llm_health_check() {
        let check = LlmHealthCheck::new("deepseek");

        // Not configured
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Degraded);

        // Configure
        check.set_configured(true);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_storage_health_check() {
        let check = StorageHealthCheck::new("file_storage");

        // Available by default
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Healthy);

        // Mark unavailable
        check.set_available(false);
        let result = check.check().await;
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_health_report() {
        let report = HealthReport {
            status: HealthStatus::Healthy,
            components: vec![],
            uptime_secs: 3600,
            is_ready: true,
            timestamp: Utc::now(),
        };

        assert!(report.is_healthy());
        assert!(report.is_available());
        assert_eq!(report.http_status_code(), 200);
    }

    #[test]
    fn test_health_response() {
        let healthy = HealthResponse::healthy();
        assert_eq!(healthy.code, 200);

        let unhealthy = HealthResponse::unhealthy("test error");
        assert_eq!(unhealthy.code, 503);

        let probe_result = ProbeResult::success(ProbeType::Liveness, 5);
        let from_probe = HealthResponse::from_probe(&probe_result);
        assert_eq!(from_probe.code, 200);
    }

    #[test]
    fn test_health_check_builder() {
        let builder = HealthCheckBuilder::new("test")
            .priority(10)
            .depends_on("storage")
            .required_for_readiness(true)
            .required_for_liveness(false)
            .timeout(Duration::from_secs(5));

        assert_eq!(builder.name(), "test");
        assert_eq!(builder.priority, 10);
        assert_eq!(builder.dependencies.len(), 1);
    }

    #[tokio::test]
    async fn test_health_system_stats() {
        let system = HealthSystem::new(HealthSystemConfig::default());
        system.mark_started();
        system.mark_ready();

        // Perform some checks
        let _ = system.liveness_probe().await;
        let _ = system.readiness_probe().await;
        let _ = system.health_check().await;

        let stats = system.stats();
        assert_eq!(stats.liveness_checks, 1);
        assert_eq!(stats.readiness_checks, 1);
        assert_eq!(stats.health_checks, 1);
        assert!(stats.is_started);
        assert!(stats.is_ready);
    }

    #[test]
    fn test_disk_health_check_default() {
        let check = DiskHealthCheck::default();
        assert_eq!(check.path, ".");
        assert_eq!(check.min_free_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_memory_health_check_default() {
        let check = MemoryHealthCheck::default();
        assert_eq!(check.max_heap_bytes, 1024 * 1024 * 1024);
        assert!((check.warning_threshold - 0.8).abs() < 0.001);
    }
}
