//! 任务系统错误类型定义

use thiserror::Error;

/// 任务系统错误
#[derive(Error, Debug)]
pub enum TaskError {
    /// LLM 错误
    #[error("LLM 错误: {0}")]
    LlmError(String),

    /// JSON 解析错误
    #[error("解析错误: {0}")]
    ParseError(String),

    /// 循环依赖
    #[error("检测到循环依赖")]
    CyclicDependency,

    /// 无法解析的依赖
    #[error("无法解析的依赖关系")]
    UnresolvableDependencies,

    /// 关键任务失败
    #[error("关键任务失败: {0}")]
    CriticalTaskFailed(String),

    /// 不支持的任务类型
    #[error("不支持的任务类型")]
    UnsupportedTaskType,

    /// 任务不存在
    #[error("任务不存在: {0}")]
    TaskNotFound(String),

    /// 计划不存在
    #[error("执行计划不存在")]
    PlanNotFound,

    /// 执行被取消
    #[error("任务执行被用户取消")]
    ExecutionCancelled,

    /// Shell 执行错误
    #[error("Shell 执行错误: {0}")]
    ShellExecutionError(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    IoError(#[from] std::io::Error),

    /// 其他错误
    #[error("其他错误: {0}")]
    Other(String),
}

/// 任务系统 Result 类型
pub type TaskResult<T> = Result<T, TaskError>;
