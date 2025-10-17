//! LLM Client 抽象和实现
//!
//! 设计参考 Python 版本的成功经验，结合 Rust 优势：
//! - trait 系统替代 Protocol
//! - 类型安全的错误处理
//! - 异步 async/await
//! - 线程安全的统计
//!
//! 支持的提供商：
//! - Ollama (本地)
//! - Deepseek (远程 API)
//! - OpenAI (兼容 API)

mod ollama;
mod deepseek;
pub mod http_base;

pub use ollama::OllamaClient;
pub use deepseek::DeepseekClient;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

// ============================================================================
// 错误类型
// ============================================================================

/// LLM 客户端错误
#[derive(Debug, Clone, Error)]
pub enum LlmError {
    /// 网络错误
    #[error("Network error: {0}")]
    Network(String),

    /// HTTP 错误
    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    /// 速率限制
    #[error("Rate limit exceeded")]
    RateLimit,

    /// 超时
    #[error("Timeout")]
    Timeout,

    /// 解析错误
    #[error("Parse error: {0}")]
    Parse(String),

    /// 配置错误
    #[error("Config error: {0}")]
    Config(String),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for LlmError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            LlmError::Timeout
        } else if let Some(status) = e.status() {
            let status_code = status.as_u16();
            if status_code == 429 {
                LlmError::RateLimit
            } else {
                LlmError::Http {
                    status: status_code,
                    message: e.to_string(),
                }
            }
        } else {
            LlmError::Network(e.to_string())
        }
    }
}

// ============================================================================
// 消息结构
// ============================================================================

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// 工具调用信息（在 assistant 消息中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String, // 通常是 "function"
    pub function: FunctionCall,
}

/// 函数调用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON 字符串
}

/// LLM 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// 创建系统消息
    #[allow(dead_code)]
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// 创建助手消息
    #[allow(dead_code)]
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// 创建带工具调用的助手消息
    #[allow(dead_code)]
    pub fn assistant_with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }

    /// 创建工具结果消息
    #[allow(dead_code)]
    pub fn tool_result(tool_call_id: String, content: String) -> Self {
        Self {
            role: MessageRole::Tool,
            content: Some(content),
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
}

// ============================================================================
// 统计系统
// ============================================================================

/// 客户端统计信息（线程安全）
#[derive(Debug, Clone)]
pub struct ClientStats {
    total_calls: Arc<AtomicU64>,
    total_retries: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
    total_success: Arc<AtomicU64>,
}

impl ClientStats {
    pub fn new() -> Self {
        Self {
            total_calls: Arc::new(AtomicU64::new(0)),
            total_retries: Arc::new(AtomicU64::new(0)),
            total_errors: Arc::new(AtomicU64::new(0)),
            total_success: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_call(&self) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry(&self) {
        self.total_retries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_success(&self) {
        self.total_success.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)]
    pub fn total_calls(&self) -> u64 {
        self.total_calls.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn total_retries(&self) -> u64 {
        self.total_retries.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn total_errors(&self) -> u64 {
        self.total_errors.load(Ordering::Relaxed)
    }

    #[allow(dead_code)]
    pub fn total_success(&self) -> u64 {
        self.total_success.load(Ordering::Relaxed)
    }
}

impl Default for ClientStats {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Chat 响应结构
// ============================================================================

/// LLM Chat 响应（支持工具调用）
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// 文本响应内容
    pub content: Option<String>,

    /// 工具调用列表
    pub tool_calls: Vec<ToolCall>,

    /// 是否为最终响应（没有工具调用）
    pub is_final: bool,
}

impl ChatResponse {
    /// 创建纯文本响应
    pub fn text(content: String) -> Self {
        Self {
            content: Some(content),
            tool_calls: Vec::new(),
            is_final: true,
        }
    }

    /// 创建工具调用响应
    pub fn with_tools(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: None,
            tool_calls,
            is_final: false,
        }
    }
}

// ============================================================================
// LLM Client Trait
// ============================================================================

/// LLM 客户端统一接口
///
/// 参考 Python 版本的 LLMClient Protocol，提供：
/// - 异步聊天接口
/// - Function Calling 支持
/// - 模型信息
/// - 统计数据
/// - 诊断功能
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 核心聊天接口
    ///
    /// # 参数
    /// - `messages`: 对话消息列表
    ///
    /// # 返回
    /// - `Ok(String)`: LLM 响应
    /// - `Err(LlmError)`: 错误
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;

    /// 带工具的聊天接口（Function Calling）
    ///
    /// # 参数
    /// - `messages`: 对话消息列表
    /// - `tools`: 工具 schema 列表（OpenAI Function Schema 格式）
    ///
    /// # 返回
    /// - `Ok(ChatResponse)`: 可能包含文本或工具调用
    /// - `Err(LlmError)`: 错误
    ///
    /// # 默认实现
    /// 不支持工具调用的客户端会降级为普通 chat
    async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        _tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse, LlmError> {
        // 默认实现：降级为普通 chat
        let content = self.chat(messages).await?;
        Ok(ChatResponse::text(content))
    }

    /// 获取模型名称
    fn model(&self) -> &str;

    /// 获取统计信息
    #[allow(dead_code)]
    fn stats(&self) -> ClientStats;

    /// 诊断连接状态
    ///
    /// 返回人类可读的诊断信息
    async fn diagnose(&self) -> String;
}

// ============================================================================
// 重试策略
// ============================================================================

/// 重试策略配置
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// 最大尝试次数
    pub max_attempts: u32,
    /// 初始退避时间（毫秒）
    pub initial_backoff_ms: u64,
    /// 最大退避时间（毫秒）
    pub max_backoff_ms: u64,
    /// 退避倍数
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 800,
            max_backoff_ms: 8000,
            backoff_multiplier: 1.8,
        }
    }
}

impl RetryPolicy {
    /// 判断错误是否可重试
    pub fn is_retryable(&self, error: &LlmError) -> bool {
        matches!(
            error,
            LlmError::Network(_)
                | LlmError::Timeout
                | LlmError::RateLimit
                | LlmError::Http { status: 500..=599, .. }
        )
    }

    /// 计算退避时间（含抖动）
    pub fn backoff_duration(&self, attempt: u32) -> std::time::Duration {
        let base = self.initial_backoff_ms as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);
        let backoff_ms = base.min(self.max_backoff_ms as f64) as u64;

        // 添加 10% 抖动
        let jitter = (rand::random::<f64>() * 0.1 * backoff_ms as f64) as u64;

        std::time::Duration::from_millis(backoff_ms + jitter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::user("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, Some("Hello".to_string()));
        assert!(msg.tool_calls.is_none());

        let sys_msg = Message::system("You are a helpful assistant");
        assert_eq!(sys_msg.role, MessageRole::System);
    }

    #[test]
    fn test_message_with_tools() {
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "test_tool".to_string(),
                arguments: r#"{"arg": "value"}"#.to_string(),
            },
        };

        let msg = Message::assistant_with_tools(vec![tool_call]);
        assert_eq!(msg.role, MessageRole::Assistant);
        assert!(msg.content.is_none());
        assert!(msg.tool_calls.is_some());
        assert_eq!(msg.tool_calls.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_tool_result_message() {
        let msg = Message::tool_result("call_123".to_string(), "result".to_string());
        assert_eq!(msg.role, MessageRole::Tool);
        assert_eq!(msg.content, Some("result".to_string()));
        assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
    }

    #[test]
    fn test_client_stats() {
        let stats = ClientStats::new();
        assert_eq!(stats.total_calls(), 0);

        stats.record_call();
        assert_eq!(stats.total_calls(), 1);

        stats.record_retry();
        stats.record_retry();
        assert_eq!(stats.total_retries(), 2);
    }

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_backoff_ms, 800);
    }

    #[test]
    fn test_retry_policy_retryable() {
        let policy = RetryPolicy::default();

        assert!(policy.is_retryable(&LlmError::Timeout));
        assert!(policy.is_retryable(&LlmError::RateLimit));
        assert!(policy.is_retryable(&LlmError::Network("error".into())));
        assert!(policy.is_retryable(&LlmError::Http {
            status: 503,
            message: "Service Unavailable".into()
        }));

        assert!(!policy.is_retryable(&LlmError::Parse("error".into())));
        assert!(!policy.is_retryable(&LlmError::Config("error".into())));
    }

    #[test]
    fn test_retry_backoff() {
        let policy = RetryPolicy::default();

        let d1 = policy.backoff_duration(1);
        let d2 = policy.backoff_duration(2);
        let d3 = policy.backoff_duration(3);

        // 应该递增
        assert!(d2 > d1);
        assert!(d3 > d2);

        // 不应该超过最大值
        let d10 = policy.backoff_duration(10);
        assert!(d10.as_millis() <= policy.max_backoff_ms as u128 * 11 / 10); // 允许10%抖动
    }
}
