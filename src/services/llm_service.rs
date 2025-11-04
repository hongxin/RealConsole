//! LLM 服务
//!
//! 负责处理 LLM 相关的所有操作，包括：
//! - 普通对话
//! - 流式对话
//! - 工具调用（Function Calling）
//! - Primary/Fallback 机制
//!
//! ## 设计原则
//! 1. **统一接口** - 所有 LLM 调用都通过 LlmService
//! 2. **模式隔离** - 普通/流式/工具调用分别处理
//! 3. **错误恢复** - Primary 失败时自动 Fallback

use crate::llm::LlmClient;
use crate::llm_manager::LlmManager;
use crate::services::{Service, ServiceResponse};
use crate::tool::ToolRegistry;
use crate::tool_executor::ToolExecutor;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// LLM 处理模式
#[derive(Debug, Clone, PartialEq)]
pub enum LlmMode {
    /// 普通模式 - 一次性返回完整响应
    Normal,
    /// 流式模式 - 实时输出 token
    Streaming,
    /// 工具调用模式 - Function Calling
    WithTools,
}

/// LLM 服务请求
#[derive(Debug, Clone)]
pub struct LlmRequest {
    /// 用户输入文本
    pub text: String,
    /// 处理模式
    pub mode: LlmMode,
    /// 流式输出回调（仅在 Streaming 模式下有效）
    /// 注意：因为闭包不能 Clone，这里使用 Option
    #[allow(dead_code)]
    pub stream_callback: Option<String>, // 占位符，实际使用时传递函数指针
    /// 历史对话消息（用于上下文支持）
    /// 如果为 None，则只使用 text 字段
    pub messages: Option<Vec<crate::llm::Message>>,
}

impl LlmRequest {
    /// 创建普通模式请求
    pub fn normal(text: String) -> Self {
        Self {
            text,
            mode: LlmMode::Normal,
            stream_callback: None,
            messages: None,
        }
    }

    /// 创建流式模式请求
    pub fn streaming(text: String) -> Self {
        Self {
            text,
            mode: LlmMode::Streaming,
            stream_callback: None,
            messages: None,
        }
    }

    /// 创建工具调用模式请求
    pub fn with_tools(text: String) -> Self {
        Self {
            text,
            mode: LlmMode::WithTools,
            stream_callback: None,
            messages: None,
        }
    }

    /// 创建带上下文的工具调用模式请求
    pub fn with_tools_and_context(messages: Vec<crate::llm::Message>) -> Self {
        // 从 messages 中提取最后一条用户消息的文本（用于兼容性）
        let text = messages
            .last()
            .and_then(|m| m.content.clone())
            .unwrap_or_default();

        Self {
            text,
            mode: LlmMode::WithTools,
            stream_callback: None,
            messages: Some(messages),
        }
    }
}

/// LLM 服务响应
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// 响应文本
    pub text: String,
    /// 是否使用了 Fallback
    pub used_fallback: bool,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 使用的模型名称
    pub model_name: String,
}

/// LLM 服务错误
#[derive(Debug, Clone)]
pub enum LlmError {
    /// 未配置 LLM 客户端
    NoClient,
    /// LLM 调用失败
    CallFailed(String),
    /// 工具调用失败
    ToolCallFailed(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::NoClient => write!(f, "未配置 LLM 客户端"),
            LlmError::CallFailed(e) => write!(f, "LLM 调用失败: {}", e),
            LlmError::ToolCallFailed(e) => write!(f, "工具调用失败: {}", e),
        }
    }
}

impl std::error::Error for LlmError {}

/// LLM 服务
///
/// 统一管理所有 LLM 交互
pub struct LlmService {
    /// LLM Manager（管理 Primary 和 Fallback）
    llm_manager: Arc<RwLock<LlmManager>>,
    /// Tool Registry（用于工具调用模式）
    tool_registry: Arc<RwLock<ToolRegistry>>,
    /// Tool Executor（用于工具调用模式）
    tool_executor: Arc<ToolExecutor>,
    /// 配置文件中的系统提示词
    config_system_prompt: Option<String>,
    /// 运行时系统提示词（可通过 /set-prompt 动态修改）
    runtime_system_prompt: Arc<RwLock<Option<String>>>,
}

impl LlmService {
    /// 创建新的 LLM 服务
    pub fn new(
        llm_manager: Arc<RwLock<LlmManager>>,
        tool_registry: Arc<RwLock<ToolRegistry>>,
        tool_executor: Arc<ToolExecutor>,
        config_system_prompt: Option<String>,
        runtime_system_prompt: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            llm_manager,
            tool_registry,
            tool_executor,
            config_system_prompt,
            runtime_system_prompt,
        }
    }

    /// 获取可用的 LLM 客户端（Primary 或 Fallback）
    async fn get_llm_client(&self) -> Result<(Arc<dyn LlmClient>, bool), LlmError> {
        let manager = self.llm_manager.read().await;

        // 优先使用 Primary
        if let Some(primary) = manager.primary() {
            return Ok((Arc::clone(primary), false));
        }

        // Fallback
        if let Some(fallback) = manager.fallback() {
            return Ok((Arc::clone(fallback), true));
        }

        Err(LlmError::NoClient)
    }

    /// 普通模式 - 一次性返回完整响应
    async fn process_normal(&self, text: &str) -> Result<LlmResponse, LlmError> {
        let start = std::time::Instant::now();
        let (llm, used_fallback) = self.get_llm_client().await?;

        // 获取模型名称
        let model_name = llm.model().to_string();

        let manager = self.llm_manager.read().await;
        let response = manager
            .chat(text)
            .await
            .map_err(|e| LlmError::CallFailed(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(LlmResponse {
            text: response,
            used_fallback,
            duration_ms,
            model_name,
        })
    }

    /// 流式模式 - 实时输出 token
    ///
    /// 注意：流式输出直接打印到 stdout，返回的 text 为空
    async fn process_streaming<F>(&self, text: &str, callback: F) -> Result<LlmResponse, LlmError>
    where
        F: FnMut(&str),
    {
        let start = std::time::Instant::now();
        let (llm_client, used_fallback) = self.get_llm_client().await?;

        // 获取模型名称
        let model_name = llm_client.model().to_string();

        let manager = self.llm_manager.read().await;
        manager
            .chat_stream(text, callback)
            .await
            .map_err(|e| LlmError::CallFailed(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(LlmResponse {
            text: String::new(), // 流式模式不返回文本
            used_fallback,
            duration_ms,
            model_name,
        })
    }

    /// 工具调用模式 - Function Calling
    async fn process_with_tools(
        &self,
        text: &str,
        messages: Option<Vec<crate::llm::Message>>,
    ) -> Result<LlmResponse, LlmError> {
        let start = std::time::Instant::now();
        let (llm_client, used_fallback) = self.get_llm_client().await?;

        // 获取模型名称
        let model_name = llm_client.model().to_string();

        // 获取工具 schemas
        let registry = self.tool_registry.read().await;
        let tool_schemas = registry.get_function_schemas();
        drop(registry);

        // 如果没有工具，回退到普通模式
        if tool_schemas.is_empty() {
            return self.process_normal(text).await;
        }

        // 构建消息列表（如果有历史上下文则使用，否则创建新的）
        let msgs = messages.unwrap_or_else(|| {
            // 内置默认提示词
            let default_system_prompt = "你是一个有用的智能助手。你可以使用提供的工具来帮助用户完成任务。\n\
                请直接、自然地回答用户的问题，不要过度客套。\n\
                当用户询问事实性问题时，请提供准确、详细的信息。";

            // 系统提示词优先级：运行时 > 配置文件 > 内置默认
            // 注意：这里使用阻塞调用是安全的，因为我们已经在 tokio 运行时中
            let runtime_prompt = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.runtime_system_prompt.read().await.clone()
                })
            });

            let system_prompt = runtime_prompt
                .as_deref()
                .or(self.config_system_prompt.as_deref())
                .unwrap_or(default_system_prompt);

            vec![
                crate::llm::Message::system(system_prompt),
                crate::llm::Message::user(text),
            ]
        });

        // 使用工具执行引擎
        let response = self
            .tool_executor
            .execute_iterative(llm_client.as_ref(), msgs, tool_schemas)
            .await
            .map_err(LlmError::ToolCallFailed)?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(LlmResponse {
            text: response,
            used_fallback,
            duration_ms,
            model_name,
        })
    }
}

#[async_trait]
impl Service for LlmService {
    type Request = LlmRequest;
    type Response = LlmResponse;
    type Error = LlmError;

    async fn process(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        match request.mode {
            LlmMode::Normal => self.process_normal(&request.text).await,
            LlmMode::Streaming => {
                // 流式模式需要回调函数，这里使用默认的 stdout 输出
                use std::io::Write;
                self.process_streaming(&request.text, |token| {
                    print!("{}", token);
                    let _ = std::io::stdout().flush();
                })
                .await
            }
            LlmMode::WithTools => self.process_with_tools(&request.text, request.messages).await,
        }
    }

    fn name(&self) -> &str {
        "LlmService"
    }

    async fn health_check(&self) -> bool {
        let manager = self.llm_manager.read().await;
        manager.primary().is_some() || manager.fallback().is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolRegistry;
    use crate::tool_executor::ToolExecutor;

    #[tokio::test]
    async fn test_llm_service_no_client() {
        let llm_manager = Arc::new(RwLock::new(LlmManager::new()));
        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        let tool_executor = Arc::new(ToolExecutor::with_defaults(tool_registry.clone()));
        let runtime_prompt = Arc::new(RwLock::new(None));

        let service = LlmService::new(
            llm_manager,
            tool_registry,
            tool_executor,
            None,
            runtime_prompt,
        );

        let request = LlmRequest::normal("Hello".to_string());
        let result = service.process(request).await;

        assert!(result.is_err());
        match result {
            Err(LlmError::NoClient) => {} // 预期错误
            _ => panic!("Expected NoClient error"),
        }
    }

    #[tokio::test]
    async fn test_llm_service_health_check() {
        let llm_manager = Arc::new(RwLock::new(LlmManager::new()));
        let tool_registry = Arc::new(RwLock::new(ToolRegistry::new()));
        let tool_executor = Arc::new(ToolExecutor::with_defaults(tool_registry.clone()));
        let runtime_prompt = Arc::new(RwLock::new(None));

        let service = LlmService::new(
            llm_manager,
            tool_registry,
            tool_executor,
            None,
            runtime_prompt,
        );

        // 没有配置 LLM 时，health_check 应该返回 false
        assert!(!service.health_check().await);
    }

    #[test]
    fn test_llm_request_creation() {
        let req = LlmRequest::normal("test".to_string());
        assert_eq!(req.mode, LlmMode::Normal);

        let req = LlmRequest::streaming("test".to_string());
        assert_eq!(req.mode, LlmMode::Streaming);

        let req = LlmRequest::with_tools("test".to_string());
        assert_eq!(req.mode, LlmMode::WithTools);
    }
}
