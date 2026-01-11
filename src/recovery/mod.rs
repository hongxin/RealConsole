//! v1.91.0: 优雅错误恢复系统
//!
//! 提供统一的健康检查和错误恢复能力：
//! - **HealthChecker**: 服务健康状态检查
//! - **RecoveryOrchestrator**: 协调错误恢复
//! - **CircuitState**: 熔断状态管理
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │           RecoveryOrchestrator              │
//! ├─────────────────────────────────────────────┤
//! │  ┌─────────┐  ┌─────────┐  ┌─────────┐     │
//! │  │ Health  │  │ Circuit │  │ Recovery│     │
//! │  │ Checker │  │ Breaker │  │ Actions │     │
//! │  └─────────┘  └─────────┘  └─────────┘     │
//! └─────────────────────────────────────────────┘
//! ```

pub mod health;
pub mod orchestrator;

pub use health::{
    ComponentHealth, HealthCheckResult, HealthChecker, HealthConfig, HealthStatus,
    ServiceHealthCheck,
};
pub use orchestrator::{
    RecoveryAction, RecoveryConfig, RecoveryEvent, RecoveryOrchestrator, RecoveryStats,
};
