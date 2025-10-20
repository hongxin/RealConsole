//! Shell 服务
//!
//! 负责处理 Shell 命令执行，包括：
//! - 命令执行（通过 ShellExecutorWithFixer）
//! - 错误分析和修复建议
//! - 危险命令检测
//! - 输出限制处理
//!
//! ## 设计原则
//! 1. **安全第一** - 危险命令检测和确认
//! 2. **智能修复** - 自动分析错误并提供修复建议
//! 3. **可观测性** - 记录执行历史和统计信息

use crate::services::{Service, ServiceResponse};
use crate::shell_executor::{ExecutionResult, ShellExecutorWithFixer};
use async_trait::async_trait;
use std::sync::Arc;

/// Shell 服务请求
#[derive(Debug, Clone)]
pub struct ShellRequest {
    /// Shell 命令
    pub command: String,
    /// 是否强制执行（跳过危险命令检测）
    pub force: bool,
    /// 是否应用错误修复（如果有）
    pub auto_fix: bool,
}

impl ShellRequest {
    /// 创建普通 Shell 请求
    pub fn new(command: String) -> Self {
        Self {
            command,
            force: false,
            auto_fix: false,
        }
    }

    /// 创建强制执行请求（跳过危险命令检测）
    pub fn forced(command: String) -> Self {
        Self {
            command,
            force: true,
            auto_fix: false,
        }
    }

    /// 创建自动修复请求
    pub fn with_auto_fix(command: String) -> Self {
        Self {
            command,
            force: false,
            auto_fix: true,
        }
    }
}

/// Shell 服务响应
#[derive(Debug, Clone)]
pub struct ShellResponse {
    /// 执行结果
    pub result: ExecutionResult,
    /// 是否被拦截（危险命令）
    pub blocked: bool,
    /// 修复建议（如果有错误）
    pub fix_suggestions: Vec<String>,
}

/// Shell 服务错误
#[derive(Debug, Clone)]
pub enum ShellError {
    /// 执行失败
    ExecutionFailed(String),
    /// 危险命令被拦截
    DangerousCommandBlocked(String),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::ExecutionFailed(e) => write!(f, "执行失败: {}", e),
            ShellError::DangerousCommandBlocked(cmd) => {
                write!(f, "危险命令被拦截: {}", cmd)
            }
        }
    }
}

impl std::error::Error for ShellError {}

/// Shell 服务
///
/// 统一管理所有 Shell 命令执行
pub struct ShellService {
    /// Shell 执行器（带错误修复）
    executor: Arc<ShellExecutorWithFixer>,
}

impl ShellService {
    /// 创建新的 Shell 服务
    pub fn new(executor: Arc<ShellExecutorWithFixer>) -> Self {
        Self { executor }
    }

    /// 检查是否是危险命令
    fn is_dangerous_command(&self, cmd: &str) -> bool {
        let dangerous_patterns = [
            "rm -rf /",
            "rm -rf /*",
            "mkfs",
            "dd if=",
            "> /dev/sda",
            "mv /* ",
            "chmod -R 777 /",
        ];

        dangerous_patterns
            .iter()
            .any(|pattern| cmd.contains(pattern))
    }

    /// 执行 Shell 命令
    async fn execute_command(&self, cmd: &str) -> ExecutionResult {
        // 使用 tokio::task::spawn_blocking 来运行同步代码
        let executor = Arc::clone(&self.executor);
        let cmd = cmd.to_string();

        tokio::task::spawn_blocking(move || {
            // 使用 tokio runtime 执行异步 execute_with_analysis
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async { executor.execute_with_analysis(&cmd).await })
        })
        .await
        .unwrap_or_else(|e| ExecutionResult {
            success: false,
            output: format!("执行任务失败: {}", e),
            error_analysis: None,
            fix_strategies: vec![],
        })
    }
}

#[async_trait]
impl Service for ShellService {
    type Request = ShellRequest;
    type Response = ShellResponse;
    type Error = ShellError;

    async fn process(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        // 1. 危险命令检测
        if !request.force && self.is_dangerous_command(&request.command) {
            return Err(ShellError::DangerousCommandBlocked(
                request.command.clone(),
            ));
        }

        // 2. 执行命令
        let result = self.execute_command(&request.command).await;

        // 3. 收集修复建议
        let fix_suggestions = if !result.success {
            result
                .fix_strategies
                .iter()
                .map(|s| s.command.clone())
                .collect()
        } else {
            vec![]
        };

        Ok(ShellResponse {
            result,
            blocked: false,
            fix_suggestions,
        })
    }

    fn name(&self) -> &str {
        "ShellService"
    }

    async fn health_check(&self) -> bool {
        // 尝试执行一个简单的命令来验证 Shell 可用性
        let result = self.execute_command("echo test").await;
        result.success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_service_simple_command() {
        let executor = Arc::new(ShellExecutorWithFixer::new());
        let service = ShellService::new(executor);

        let request = ShellRequest::new("echo 'Hello, RealConsole!'".to_string());
        let result = service.process(request).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.result.success);
        assert!(!response.blocked);
    }

    #[tokio::test]
    async fn test_shell_service_dangerous_command() {
        let executor = Arc::new(ShellExecutorWithFixer::new());
        let service = ShellService::new(executor);

        let request = ShellRequest::new("rm -rf /".to_string());
        let result = service.process(request).await;

        assert!(result.is_err());
        match result {
            Err(ShellError::DangerousCommandBlocked(_)) => {} // 预期错误
            _ => panic!("Expected DangerousCommandBlocked error"),
        }
    }

    #[tokio::test]
    async fn test_shell_service_forced_execution() {
        let executor = Arc::new(ShellExecutorWithFixer::new());
        let service = ShellService::new(executor);

        // 注意：这里只是测试 force 标志，不会真的执行危险命令
        let request = ShellRequest::forced("echo 'forced command'".to_string());
        let result = service.process(request).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shell_service_health_check() {
        let executor = Arc::new(ShellExecutorWithFixer::new());
        let service = ShellService::new(executor);

        assert!(service.health_check().await);
    }
}
