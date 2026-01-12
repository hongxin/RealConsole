//! v1.91.0: 恢复编排器
//!
//! 协调错误恢复流程：
//! - 监控健康状态变化
//! - 触发恢复动作
//! - 管理熔断状态
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::recovery::{RecoveryOrchestrator, RecoveryConfig};
//!
//! let orchestrator = RecoveryOrchestrator::new(RecoveryConfig::default());
//!
//! // 注册恢复动作
//! orchestrator.on_unhealthy("llm", |event| async {
//!     // 切换到备用 LLM
//! });
//!
//! // 记录失败
//! orchestrator.record_failure("llm", "Connection timeout").await;
//!
//! // 检查是否应该熔断
//! if orchestrator.should_break("llm").await {
//!     // 使用备用方案
//! }
//! ```

use super::health::HealthStatus;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// ============================================================================
// 熔断状态
// ============================================================================

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CircuitState {
    /// 关闭（正常）
    #[default]
    Closed,
    /// 打开（熔断中）
    Open,
    /// 半开（测试恢复）
    HalfOpen,
}

// ============================================================================
// 恢复事件
// ============================================================================

/// 恢复事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    /// 组件名称
    pub component: String,
    /// 事件类型
    pub event_type: RecoveryEventType,
    /// 事件消息
    pub message: String,
    /// 时间戳
    pub timestamp: u64,
    /// 之前状态
    pub previous_state: Option<CircuitState>,
    /// 当前状态
    pub current_state: CircuitState,
}

/// 恢复事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryEventType {
    /// 失败记录
    FailureRecorded,
    /// 成功记录
    SuccessRecorded,
    /// 熔断器打开
    CircuitOpened,
    /// 熔断器半开
    CircuitHalfOpen,
    /// 熔断器关闭
    CircuitClosed,
    /// 恢复动作触发
    RecoveryTriggered,
}

// ============================================================================
// 恢复动作
// ============================================================================

/// 恢复动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// 无动作
    None,
    /// 重试
    Retry { max_attempts: u32, delay_ms: u64 },
    /// 切换到备用
    Fallback { target: String },
    /// 降级服务
    Degrade { message: String },
    /// 通知
    Notify { message: String },
    /// 自定义动作
    Custom { action_name: String },
}

// ============================================================================
// 组件恢复状态
// ============================================================================

/// 组件恢复状态
#[derive(Debug, Clone)]
struct ComponentRecoveryState {
    /// 组件名称
    name: String,
    /// 熔断器状态
    circuit_state: CircuitState,
    /// 连续失败次数
    consecutive_failures: u32,
    /// 连续成功次数
    consecutive_successes: u32,
    /// 最后失败时间
    last_failure: Option<Instant>,
    /// 最后成功时间
    last_success: Option<Instant>,
    /// 熔断器打开时间
    circuit_opened_at: Option<Instant>,
    /// 总失败次数
    total_failures: u64,
    /// 总成功次数
    total_successes: u64,
    /// 配置的恢复动作
    recovery_action: RecoveryAction,
}

impl ComponentRecoveryState {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            circuit_state: CircuitState::Closed,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_failure: None,
            last_success: None,
            circuit_opened_at: None,
            total_failures: 0,
            total_successes: 0,
            recovery_action: RecoveryAction::None,
        }
    }
}

// ============================================================================
// 恢复配置
// ============================================================================

/// 恢复配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    /// 失败阈值（触发熔断）
    pub failure_threshold: u32,
    /// 成功阈值（恢复）
    pub success_threshold: u32,
    /// 熔断超时（毫秒）
    pub circuit_timeout_ms: u64,
    /// 半开状态允许的测试请求数
    pub half_open_requests: u32,
    /// 事件历史保留数量
    pub event_history_size: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            circuit_timeout_ms: 30000, // 30秒
            half_open_requests: 3,
            event_history_size: 100,
        }
    }
}

// ============================================================================
// 恢复统计
// ============================================================================

/// 恢复统计
#[derive(Debug, Default)]
pub struct RecoveryStats {
    /// 总失败记录数
    total_failures: AtomicU64,
    /// 总成功记录数
    total_successes: AtomicU64,
    /// 熔断器打开次数
    circuits_opened: AtomicU64,
    /// 熔断器关闭次数
    circuits_closed: AtomicU64,
    /// 恢复动作触发次数
    recoveries_triggered: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStatsSnapshot {
    pub total_failures: u64,
    pub total_successes: u64,
    pub circuits_opened: u64,
    pub circuits_closed: u64,
    pub recoveries_triggered: u64,
}

impl RecoveryStats {
    fn snapshot(&self) -> RecoveryStatsSnapshot {
        RecoveryStatsSnapshot {
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_successes: self.total_successes.load(Ordering::Relaxed),
            circuits_opened: self.circuits_opened.load(Ordering::Relaxed),
            circuits_closed: self.circuits_closed.load(Ordering::Relaxed),
            recoveries_triggered: self.recoveries_triggered.load(Ordering::Relaxed),
        }
    }
}

// ============================================================================
// 恢复编排器
// ============================================================================

/// 恢复编排器
pub struct RecoveryOrchestrator {
    /// 配置
    config: RecoveryConfig,
    /// 组件状态
    components: RwLock<HashMap<String, ComponentRecoveryState>>,
    /// 事件历史
    events: RwLock<Vec<RecoveryEvent>>,
    /// 统计
    stats: RecoveryStats,
}

impl RecoveryOrchestrator {
    /// 创建新的恢复编排器
    pub fn new(config: RecoveryConfig) -> Self {
        Self {
            config,
            components: RwLock::new(HashMap::new()),
            events: RwLock::new(Vec::new()),
            stats: RecoveryStats::default(),
        }
    }

    /// 注册组件
    pub async fn register(&self, name: impl Into<String>) {
        let name = name.into();
        let mut components = self.components.write().await;
        if !components.contains_key(&name) {
            components.insert(name.clone(), ComponentRecoveryState::new(name));
        }
    }

    /// 设置组件的恢复动作
    pub async fn set_recovery_action(&self, name: &str, action: RecoveryAction) {
        let mut components = self.components.write().await;
        if let Some(state) = components.get_mut(name) {
            state.recovery_action = action;
        }
    }

    /// 记录失败
    pub async fn record_failure(&self, name: &str, message: impl Into<String>) -> RecoveryEvent {
        let message = message.into();
        self.stats.total_failures.fetch_add(1, Ordering::Relaxed);

        let mut components = self.components.write().await;

        // 确保组件存在
        if !components.contains_key(name) {
            components.insert(name.to_string(), ComponentRecoveryState::new(name));
        }

        let state = components.get_mut(name).unwrap();
        let previous_state = state.circuit_state;

        state.consecutive_failures += 1;
        state.consecutive_successes = 0;
        state.last_failure = Some(Instant::now());
        state.total_failures += 1;

        // 检查是否需要打开熔断器
        let mut event_type = RecoveryEventType::FailureRecorded;
        if state.circuit_state == CircuitState::Closed
            && state.consecutive_failures >= self.config.failure_threshold
        {
            state.circuit_state = CircuitState::Open;
            state.circuit_opened_at = Some(Instant::now());
            event_type = RecoveryEventType::CircuitOpened;
            self.stats.circuits_opened.fetch_add(1, Ordering::Relaxed);
        } else if state.circuit_state == CircuitState::HalfOpen {
            // 半开状态失败，重新打开
            state.circuit_state = CircuitState::Open;
            state.circuit_opened_at = Some(Instant::now());
            event_type = RecoveryEventType::CircuitOpened;
        }

        let event = RecoveryEvent {
            component: name.to_string(),
            event_type,
            message,
            timestamp: Self::now_timestamp(),
            previous_state: Some(previous_state),
            current_state: state.circuit_state,
        };

        drop(components);
        self.add_event(event.clone()).await;

        event
    }

    /// 记录成功
    pub async fn record_success(&self, name: &str) -> RecoveryEvent {
        self.stats.total_successes.fetch_add(1, Ordering::Relaxed);

        let mut components = self.components.write().await;

        if !components.contains_key(name) {
            components.insert(name.to_string(), ComponentRecoveryState::new(name));
        }

        let state = components.get_mut(name).unwrap();
        let previous_state = state.circuit_state;

        state.consecutive_successes += 1;
        state.consecutive_failures = 0;
        state.last_success = Some(Instant::now());
        state.total_successes += 1;

        // 检查是否可以关闭熔断器
        let mut event_type = RecoveryEventType::SuccessRecorded;
        if state.circuit_state == CircuitState::HalfOpen
            && state.consecutive_successes >= self.config.success_threshold
        {
            state.circuit_state = CircuitState::Closed;
            state.circuit_opened_at = None;
            event_type = RecoveryEventType::CircuitClosed;
            self.stats.circuits_closed.fetch_add(1, Ordering::Relaxed);
        }

        let event = RecoveryEvent {
            component: name.to_string(),
            event_type,
            message: "Success".to_string(),
            timestamp: Self::now_timestamp(),
            previous_state: Some(previous_state),
            current_state: state.circuit_state,
        };

        drop(components);
        self.add_event(event.clone()).await;

        event
    }

    /// 检查是否应该熔断
    pub async fn should_break(&self, name: &str) -> bool {
        let mut components = self.components.write().await;

        if let Some(state) = components.get_mut(name) {
            match state.circuit_state {
                CircuitState::Closed => false,
                CircuitState::Open => {
                    // 检查是否超时，可以尝试半开
                    if let Some(opened_at) = state.circuit_opened_at {
                        let timeout = Duration::from_millis(self.config.circuit_timeout_ms);
                        if opened_at.elapsed() >= timeout {
                            state.circuit_state = CircuitState::HalfOpen;
                            state.consecutive_successes = 0;
                            false // 允许尝试
                        } else {
                            true // 继续熔断
                        }
                    } else {
                        true
                    }
                }
                CircuitState::HalfOpen => false, // 允许测试请求
            }
        } else {
            false
        }
    }

    /// 获取组件状态
    pub async fn get_state(&self, name: &str) -> Option<CircuitState> {
        let components = self.components.read().await;
        components.get(name).map(|s| s.circuit_state)
    }

    /// 获取恢复动作
    pub async fn get_recovery_action(&self, name: &str) -> RecoveryAction {
        let components = self.components.read().await;
        components
            .get(name)
            .map(|s| s.recovery_action.clone())
            .unwrap_or(RecoveryAction::None)
    }

    /// 手动重置组件状态
    pub async fn reset(&self, name: &str) {
        let mut components = self.components.write().await;
        if let Some(state) = components.get_mut(name) {
            state.circuit_state = CircuitState::Closed;
            state.consecutive_failures = 0;
            state.consecutive_successes = 0;
            state.circuit_opened_at = None;
        }
    }

    /// 获取统计
    pub fn stats(&self) -> RecoveryStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取事件历史
    pub async fn events(&self) -> Vec<RecoveryEvent> {
        let events = self.events.read().await;
        events.clone()
    }

    /// 获取组件摘要
    pub async fn component_summary(&self, name: &str) -> Option<ComponentSummary> {
        let components = self.components.read().await;
        components.get(name).map(|s| ComponentSummary {
            name: s.name.clone(),
            circuit_state: s.circuit_state,
            consecutive_failures: s.consecutive_failures,
            consecutive_successes: s.consecutive_successes,
            total_failures: s.total_failures,
            total_successes: s.total_successes,
            failure_rate: if s.total_failures + s.total_successes > 0 {
                s.total_failures as f64 / (s.total_failures + s.total_successes) as f64
            } else {
                0.0
            },
        })
    }

    /// 获取所有组件摘要
    pub async fn all_summaries(&self) -> Vec<ComponentSummary> {
        let components = self.components.read().await;
        components
            .values()
            .map(|s| ComponentSummary {
                name: s.name.clone(),
                circuit_state: s.circuit_state,
                consecutive_failures: s.consecutive_failures,
                consecutive_successes: s.consecutive_successes,
                total_failures: s.total_failures,
                total_successes: s.total_successes,
                failure_rate: if s.total_failures + s.total_successes > 0 {
                    s.total_failures as f64 / (s.total_failures + s.total_successes) as f64
                } else {
                    0.0
                },
            })
            .collect()
    }

    /// 添加事件
    async fn add_event(&self, event: RecoveryEvent) {
        let mut events = self.events.write().await;
        events.push(event);

        // 保持历史大小
        if events.len() > self.config.event_history_size {
            events.remove(0);
        }
    }

    fn now_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// 组件摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSummary {
    pub name: String,
    pub circuit_state: CircuitState,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub total_failures: u64,
    pub total_successes: u64,
    pub failure_rate: f64,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_basic() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        orchestrator.register("test").await;

        // 初始状态应该是关闭
        assert_eq!(orchestrator.get_state("test").await, Some(CircuitState::Closed));
        assert!(!orchestrator.should_break("test").await);
    }

    #[tokio::test]
    async fn test_orchestrator_circuit_open() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        orchestrator.register("test").await;

        // 记录失败
        orchestrator.record_failure("test", "error 1").await;
        orchestrator.record_failure("test", "error 2").await;
        assert!(!orchestrator.should_break("test").await);

        // 第三次失败应该触发熔断
        let event = orchestrator.record_failure("test", "error 3").await;
        assert_eq!(event.event_type, RecoveryEventType::CircuitOpened);
        assert!(orchestrator.should_break("test").await);
    }

    #[tokio::test]
    async fn test_orchestrator_circuit_recovery() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig {
            failure_threshold: 2,
            success_threshold: 2,
            circuit_timeout_ms: 50, // 50ms 超时，足够测试
            ..Default::default()
        });

        orchestrator.register("test").await;

        // 触发熔断
        orchestrator.record_failure("test", "error 1").await;
        orchestrator.record_failure("test", "error 2").await;
        assert!(orchestrator.should_break("test").await);

        // 超时后变为半开
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(!orchestrator.should_break("test").await);
        assert_eq!(orchestrator.get_state("test").await, Some(CircuitState::HalfOpen));

        // 成功恢复
        orchestrator.record_success("test").await;
        orchestrator.record_success("test").await;
        assert_eq!(orchestrator.get_state("test").await, Some(CircuitState::Closed));
    }

    #[tokio::test]
    async fn test_orchestrator_stats() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig::default());

        orchestrator.record_failure("test", "error").await;
        orchestrator.record_success("test").await;

        let stats = orchestrator.stats();
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.total_successes, 1);
    }

    #[tokio::test]
    async fn test_orchestrator_events() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig::default());

        orchestrator.record_failure("test", "error").await;
        orchestrator.record_success("test").await;

        let events = orchestrator.events().await;
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_orchestrator_reset() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig {
            failure_threshold: 2,
            ..Default::default()
        });

        orchestrator.register("test").await;
        orchestrator.record_failure("test", "error 1").await;
        orchestrator.record_failure("test", "error 2").await;
        assert!(orchestrator.should_break("test").await);

        // 重置
        orchestrator.reset("test").await;
        assert!(!orchestrator.should_break("test").await);
        assert_eq!(orchestrator.get_state("test").await, Some(CircuitState::Closed));
    }

    #[tokio::test]
    async fn test_component_summary() {
        let orchestrator = RecoveryOrchestrator::new(RecoveryConfig::default());

        orchestrator.record_failure("test", "error").await;
        orchestrator.record_success("test").await;
        orchestrator.record_success("test").await;

        let summary = orchestrator.component_summary("test").await.unwrap();
        assert_eq!(summary.total_failures, 1);
        assert_eq!(summary.total_successes, 2);
        assert!((summary.failure_rate - 0.333).abs() < 0.01);
    }

    #[test]
    fn test_recovery_action_serialization() {
        let action = RecoveryAction::Retry {
            max_attempts: 3,
            delay_ms: 1000,
        };
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("Retry"));
    }
}
