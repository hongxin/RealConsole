//! 服务层架构
//!
//! 将 Agent 的职责拆分为独立的服务，提升可维护性和可测试性。
//!
//! ## 设计原则
//! 1. **单一职责** - 每个服务只负责一个明确的领域
//! 2. **依赖注入** - 服务通过构造函数接收依赖
//! 3. **接口隔离** - 定义清晰的 Service trait
//! 4. **状态分离** - 状态管理独立于业务逻辑
//!
//! ## 服务划分
//! - **ShellService** - Shell 命令执行
//! - **IntentService** - Intent DSL 处理
//! - **ToolService** - 工具调用处理
//! - **LlmService** - LLM 对话处理
//! - **CommandService** - 系统命令处理
//!
//! ## Phase 2: Agent 重构
//! 版本: v1.3.0-dev
//! 作者: RealConsole Contributors
//! 日期: 2025-10-20

use async_trait::async_trait;
use std::sync::Arc;

// 重新导出子模块
pub mod state_manager;
pub mod intent_service;
pub mod llm_service;
pub mod shell_service;
pub mod registry; // v1.104.0: 服务注册中心
pub mod storage_service; // v1.104.0: 存储服务
// mod tool_service;
// mod command_service;

pub use state_manager::StateManager;
pub use intent_service::{IntentError, IntentRequest, IntentResponse, IntentService};
pub use llm_service::{LlmError, LlmMode, LlmRequest, LlmResponse, LlmService};
pub use shell_service::{ShellError, ShellRequest, ShellResponse, ShellService};
pub use registry::{
    HealthStatus, RegistryEvent, RegistryStats, ServiceDescriptor, ServiceError,
    ServiceId, ServiceLifecycle, ServiceRegistry, ServiceState,
};
pub use storage_service::{OperationCounts, StorageService, StorageServiceConfig, StorageServiceStats};

/// Service trait - 所有服务的基础抽象
///
/// 定义了服务的通用接口，支持：
/// - 同步和异步处理
/// - 结构化的请求/响应
/// - 统一的错误处理
#[async_trait]
pub trait Service: Send + Sync {
    /// 请求类型
    type Request;
    /// 响应类型
    type Response;
    /// 错误类型
    type Error;

    /// 处理请求（异步）
    async fn process(&self, request: Self::Request) -> Result<Self::Response, Self::Error>;

    /// 服务名称（用于日志和调试）
    fn name(&self) -> &str;

    /// 健康检查（可选实现）
    async fn health_check(&self) -> bool {
        true
    }
}

/// 服务响应的通用封装
///
/// 包含响应数据和元数据
#[derive(Debug, Clone)]
pub struct ServiceResponse<T> {
    /// 响应数据
    pub data: T,
    /// 是否成功
    pub success: bool,
    /// 可选的错误信息
    pub error: Option<String>,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
}

impl<T> ServiceResponse<T> {
    /// 创建成功响应
    pub fn success(data: T, duration_ms: u64) -> Self {
        Self {
            data,
            success: true,
            error: None,
            duration_ms,
        }
    }

    /// 创建失败响应
    pub fn failure(data: T, error: String, duration_ms: u64) -> Self {
        Self {
            data,
            success: false,
            error: Some(error),
            duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_response_success() {
        let resp = ServiceResponse::success("Hello".to_string(), 100);
        assert!(resp.success);
        assert_eq!(resp.data, "Hello");
        assert!(resp.error.is_none());
        assert_eq!(resp.duration_ms, 100);
    }

    #[test]
    fn test_service_response_failure() {
        let resp = ServiceResponse::failure("".to_string(), "Error".to_string(), 50);
        assert!(!resp.success);
        assert_eq!(resp.data, "");
        assert_eq!(resp.error, Some("Error".to_string()));
        assert_eq!(resp.duration_ms, 50);
    }
}
