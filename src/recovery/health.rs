//! v1.91.0: 健康检查系统
//!
//! 提供服务和组件的健康状态监控：
//! - 多维健康指标
//! - 历史健康记录
//! - 健康聚合报告
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::recovery::{HealthChecker, HealthConfig};
//!
//! let mut checker = HealthChecker::new(HealthConfig::default());
//!
//! // 注册健康检查
//! checker.register("llm", Box::new(LlmHealthCheck::new(client)));
//! checker.register("storage", Box::new(StorageHealthCheck::new(backend)));
//!
//! // 执行检查
//! let report = checker.check_all().await;
//! println!("System health: {:?}", report.overall_status);
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// 健康状态
// ============================================================================

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级（部分功能可用）
    Degraded,
    /// 不健康
    Unhealthy,
    /// 未知（未执行检查）
    #[default]
    Unknown,
}

impl HealthStatus {
    /// 是否可用（健康或降级）
    pub fn is_available(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// 合并两个状态（取更差的状态）
    pub fn merge(&self, other: &HealthStatus) -> HealthStatus {
        match (self, other) {
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Unknown, _) | (_, HealthStatus::Unknown) => HealthStatus::Unknown,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            _ => HealthStatus::Healthy,
        }
    }
}


// ============================================================================
// 健康检查结果
// ============================================================================

/// 单次健康检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// 组件名称
    pub component: String,
    /// 健康状态
    pub status: HealthStatus,
    /// 响应时间（毫秒）
    pub latency_ms: u64,
    /// 详细信息
    pub message: Option<String>,
    /// 检查时间戳
    pub timestamp: u64,
    /// 附加指标
    pub metrics: HashMap<String, serde_json::Value>,
}

impl HealthCheckResult {
    /// 创建健康结果
    pub fn healthy(component: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            latency_ms,
            message: None,
            timestamp: Self::now_timestamp(),
            metrics: HashMap::new(),
        }
    }

    /// 创建降级结果
    pub fn degraded(component: impl Into<String>, latency_ms: u64, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Degraded,
            latency_ms,
            message: Some(message.into()),
            timestamp: Self::now_timestamp(),
            metrics: HashMap::new(),
        }
    }

    /// 创建不健康结果
    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            latency_ms: 0,
            message: Some(message.into()),
            timestamp: Self::now_timestamp(),
            metrics: HashMap::new(),
        }
    }

    /// 添加指标
    pub fn with_metric(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metrics.insert(key.into(), value);
        self
    }

    fn now_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================================
// 组件健康
// ============================================================================

/// 组件健康状态（含历史）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub name: String,
    /// 当前状态
    pub current_status: HealthStatus,
    /// 最后检查时间
    pub last_check: Option<u64>,
    /// 最后健康时间
    pub last_healthy: Option<u64>,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 总检查次数
    pub total_checks: u64,
    /// 成功次数
    pub successful_checks: u64,
    /// 平均响应时间（毫秒）
    pub avg_latency_ms: f64,
    /// 最近的检查结果
    pub recent_results: Vec<HealthCheckResult>,
}

impl ComponentHealth {
    /// 创建新的组件健康
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            current_status: HealthStatus::Unknown,
            last_check: None,
            last_healthy: None,
            consecutive_failures: 0,
            total_checks: 0,
            successful_checks: 0,
            avg_latency_ms: 0.0,
            recent_results: Vec::new(),
        }
    }

    /// 更新健康状态
    pub fn update(&mut self, result: HealthCheckResult, max_history: usize) {
        self.total_checks += 1;
        self.last_check = Some(result.timestamp);
        self.current_status = result.status;

        if result.status == HealthStatus::Healthy {
            self.successful_checks += 1;
            self.consecutive_failures = 0;
            self.last_healthy = Some(result.timestamp);
        } else if result.status == HealthStatus::Unhealthy {
            self.consecutive_failures += 1;
        }

        // 更新平均延迟
        if result.latency_ms > 0 {
            let total_latency = self.avg_latency_ms * (self.total_checks - 1) as f64;
            self.avg_latency_ms = (total_latency + result.latency_ms as f64) / self.total_checks as f64;
        }

        // 保留最近的结果
        self.recent_results.push(result);
        if self.recent_results.len() > max_history {
            self.recent_results.remove(0);
        }
    }

    /// 计算健康率
    pub fn health_rate(&self) -> f64 {
        if self.total_checks == 0 {
            0.0
        } else {
            self.successful_checks as f64 / self.total_checks as f64
        }
    }
}

// ============================================================================
// 健康检查接口
// ============================================================================

/// 服务健康检查接口
#[async_trait]
pub trait ServiceHealthCheck: Send + Sync {
    /// 执行健康检查
    async fn check(&self) -> HealthCheckResult;

    /// 获取组件名称
    fn name(&self) -> &str;
}

// ============================================================================
// 健康检查配置
// ============================================================================

/// 健康检查配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// 检查超时（毫秒）
    pub timeout_ms: u64,
    /// 历史记录保留数量
    pub history_size: usize,
    /// 连续失败阈值（超过则标记为不健康）
    pub failure_threshold: u32,
    /// 降级延迟阈值（毫秒）
    pub degraded_latency_ms: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            history_size: 10,
            failure_threshold: 3,
            degraded_latency_ms: 1000,
        }
    }
}

// ============================================================================
// 健康检查器
// ============================================================================

/// 统一健康检查器
pub struct HealthChecker {
    /// 配置
    config: HealthConfig,
    /// 注册的检查器
    checks: RwLock<HashMap<String, Arc<dyn ServiceHealthCheck>>>,
    /// 组件健康状态
    health_states: RwLock<HashMap<String, ComponentHealth>>,
    /// 统计
    stats: HealthCheckerStats,
}

/// 健康检查器统计
#[derive(Debug, Default)]
pub struct HealthCheckerStats {
    total_checks: AtomicU64,
    successful_checks: AtomicU64,
    failed_checks: AtomicU64,
}

impl HealthChecker {
    /// 创建新的健康检查器
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            checks: RwLock::new(HashMap::new()),
            health_states: RwLock::new(HashMap::new()),
            stats: HealthCheckerStats::default(),
        }
    }

    /// 注册健康检查
    pub async fn register(&self, name: impl Into<String>, check: Arc<dyn ServiceHealthCheck>) {
        let name = name.into();
        let mut checks = self.checks.write().await;
        let mut states = self.health_states.write().await;

        checks.insert(name.clone(), check);
        states.insert(name.clone(), ComponentHealth::new(name));
    }

    /// 移除健康检查
    pub async fn unregister(&self, name: &str) {
        let mut checks = self.checks.write().await;
        let mut states = self.health_states.write().await;

        checks.remove(name);
        states.remove(name);
    }

    /// 检查单个组件
    pub async fn check_component(&self, name: &str) -> Option<HealthCheckResult> {
        let checks = self.checks.read().await;
        let check = checks.get(name)?;

        let _start = Instant::now();
        let timeout = Duration::from_millis(self.config.timeout_ms);

        let result = match tokio::time::timeout(timeout, check.check()).await {
            Ok(result) => result,
            Err(_) => HealthCheckResult::unhealthy(name, "Health check timeout"),
        };

        // 更新统计
        self.stats.total_checks.fetch_add(1, Ordering::Relaxed);
        if result.status == HealthStatus::Healthy {
            self.stats.successful_checks.fetch_add(1, Ordering::Relaxed);
        } else if result.status == HealthStatus::Unhealthy {
            self.stats.failed_checks.fetch_add(1, Ordering::Relaxed);
        }

        // 更新组件状态
        let mut states = self.health_states.write().await;
        if let Some(state) = states.get_mut(name) {
            state.update(result.clone(), self.config.history_size);
        }

        Some(result)
    }

    /// 检查所有组件
    pub async fn check_all(&self) -> HealthReport {
        let checks = self.checks.read().await;
        let names: Vec<String> = checks.keys().cloned().collect();
        drop(checks);

        let mut results = Vec::new();
        let mut overall = HealthStatus::Healthy;

        for name in names {
            if let Some(result) = self.check_component(&name).await {
                overall = overall.merge(&result.status);
                results.push(result);
            }
        }

        HealthReport {
            overall_status: overall,
            components: results,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// 获取组件健康状态
    pub async fn get_health(&self, name: &str) -> Option<ComponentHealth> {
        let states = self.health_states.read().await;
        states.get(name).cloned()
    }

    /// 获取所有组件健康状态
    pub async fn get_all_health(&self) -> HashMap<String, ComponentHealth> {
        let states = self.health_states.read().await;
        states.clone()
    }

    /// 获取总体健康状态
    pub async fn overall_status(&self) -> HealthStatus {
        let states = self.health_states.read().await;
        let mut overall = HealthStatus::Healthy;

        for state in states.values() {
            overall = overall.merge(&state.current_status);
        }

        overall
    }

    /// 获取统计
    pub fn stats(&self) -> HealthCheckerStatsSnapshot {
        HealthCheckerStatsSnapshot {
            total_checks: self.stats.total_checks.load(Ordering::Relaxed),
            successful_checks: self.stats.successful_checks.load(Ordering::Relaxed),
            failed_checks: self.stats.failed_checks.load(Ordering::Relaxed),
        }
    }
}

/// 健康报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// 总体状态
    pub overall_status: HealthStatus,
    /// 各组件结果
    pub components: Vec<HealthCheckResult>,
    /// 时间戳
    pub timestamp: u64,
}

impl HealthReport {
    /// 生成摘要
    pub fn summary(&self) -> String {
        let healthy = self.components.iter().filter(|c| c.status == HealthStatus::Healthy).count();
        let degraded = self.components.iter().filter(|c| c.status == HealthStatus::Degraded).count();
        let unhealthy = self.components.iter().filter(|c| c.status == HealthStatus::Unhealthy).count();

        format!(
            "Health: {:?} ({} healthy, {} degraded, {} unhealthy)",
            self.overall_status, healthy, degraded, unhealthy
        )
    }
}

/// 统计快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckerStatsSnapshot {
    pub total_checks: u64,
    pub successful_checks: u64,
    pub failed_checks: u64,
}

// ============================================================================
// 内置健康检查实现
// ============================================================================

/// 简单函数健康检查
pub struct FnHealthCheck<F>
where
    F: Fn() -> HealthCheckResult + Send + Sync,
{
    name: String,
    check_fn: F,
}

impl<F> FnHealthCheck<F>
where
    F: Fn() -> HealthCheckResult + Send + Sync,
{
    /// 创建新的函数健康检查
    pub fn new(name: impl Into<String>, check_fn: F) -> Self {
        Self {
            name: name.into(),
            check_fn,
        }
    }
}

#[async_trait]
impl<F> ServiceHealthCheck for FnHealthCheck<F>
where
    F: Fn() -> HealthCheckResult + Send + Sync,
{
    async fn check(&self) -> HealthCheckResult {
        (self.check_fn)()
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 异步函数健康检查
///
/// 使用 BoxFuture 来避免复杂的生命周期问题
pub struct AsyncFnHealthCheck {
    name: String,
    check_fn: Box<dyn Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = HealthCheckResult> + Send>> + Send + Sync>,
}

impl AsyncFnHealthCheck {
    /// 创建新的异步函数健康检查
    pub fn new<F, Fut>(name: impl Into<String>, check_fn: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = HealthCheckResult> + Send + 'static,
    {
        Self {
            name: name.into(),
            check_fn: Box::new(move || Box::pin(check_fn())),
        }
    }
}

#[async_trait]
impl ServiceHealthCheck for AsyncFnHealthCheck {
    async fn check(&self) -> HealthCheckResult {
        (self.check_fn)().await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_merge() {
        assert_eq!(
            HealthStatus::Healthy.merge(&HealthStatus::Healthy),
            HealthStatus::Healthy
        );
        assert_eq!(
            HealthStatus::Healthy.merge(&HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthStatus::Degraded.merge(&HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            HealthStatus::Healthy.merge(&HealthStatus::Unknown),
            HealthStatus::Unknown
        );
    }

    #[test]
    fn test_health_status_is_available() {
        assert!(HealthStatus::Healthy.is_available());
        assert!(HealthStatus::Degraded.is_available());
        assert!(!HealthStatus::Unhealthy.is_available());
        assert!(!HealthStatus::Unknown.is_available());
    }

    #[test]
    fn test_health_check_result() {
        let result = HealthCheckResult::healthy("test", 100)
            .with_metric("connections", serde_json::json!(5));

        assert_eq!(result.component, "test");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.latency_ms, 100);
        assert!(result.metrics.contains_key("connections"));
    }

    #[test]
    fn test_component_health_update() {
        let mut health = ComponentHealth::new("test");
        assert_eq!(health.current_status, HealthStatus::Unknown);
        assert_eq!(health.total_checks, 0);

        // 添加健康结果
        health.update(HealthCheckResult::healthy("test", 100), 10);
        assert_eq!(health.current_status, HealthStatus::Healthy);
        assert_eq!(health.total_checks, 1);
        assert_eq!(health.successful_checks, 1);
        assert_eq!(health.consecutive_failures, 0);

        // 添加不健康结果
        health.update(HealthCheckResult::unhealthy("test", "error"), 10);
        assert_eq!(health.current_status, HealthStatus::Unhealthy);
        assert_eq!(health.total_checks, 2);
        assert_eq!(health.consecutive_failures, 1);
    }

    #[test]
    fn test_component_health_rate() {
        let mut health = ComponentHealth::new("test");

        health.update(HealthCheckResult::healthy("test", 100), 10);
        health.update(HealthCheckResult::healthy("test", 100), 10);
        health.update(HealthCheckResult::unhealthy("test", "error"), 10);

        let rate = health.health_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_health_checker_basic() {
        let checker = HealthChecker::new(HealthConfig::default());

        // 注册简单检查
        let check = Arc::new(FnHealthCheck::new("test", || {
            HealthCheckResult::healthy("test", 50)
        }));
        checker.register("test", check).await;

        // 执行检查
        let result = checker.check_component("test").await;
        assert!(result.is_some());
        assert_eq!(result.unwrap().status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_checker_all() {
        let checker = HealthChecker::new(HealthConfig::default());

        checker.register("healthy", Arc::new(FnHealthCheck::new("healthy", || {
            HealthCheckResult::healthy("healthy", 50)
        }))).await;

        checker.register("unhealthy", Arc::new(FnHealthCheck::new("unhealthy", || {
            HealthCheckResult::unhealthy("unhealthy", "test error")
        }))).await;

        let report = checker.check_all().await;
        assert_eq!(report.overall_status, HealthStatus::Unhealthy);
        assert_eq!(report.components.len(), 2);
    }

    #[tokio::test]
    async fn test_health_report_summary() {
        let report = HealthReport {
            overall_status: HealthStatus::Degraded,
            components: vec![
                HealthCheckResult::healthy("a", 100),
                HealthCheckResult::degraded("b", 200, "slow"),
            ],
            timestamp: 0,
        };

        let summary = report.summary();
        assert!(summary.contains("1 healthy"));
        assert!(summary.contains("1 degraded"));
    }
}
