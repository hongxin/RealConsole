//! Google Gemini 客户端实现
//!
//! API 格式特点：
//! - Query 参数认证 (?key=xxx)
//! - contents/parts 消息结构（非 OpenAI 格式）
//! - candidates 响应结构
//! - functionDeclarations 工具调用
//!
//! 参考文档：
//! - https://ai.google.dev/gemini-api/docs/text-generation
//! - https://ai.google.dev/api/generate-content

use super::http_base::HttpClientBase;
use super::{
    async_trait, ChatResponse, ClientStats, LlmClient, LlmError, Message, MessageRole,
};
use serde_json::{json, Value};

/// Google Gemini 客户端
pub struct GeminiClient {
    /// HTTP 客户端基础层
    base: HttpClientBase,

    /// 模型名称（如 "gemini-pro", "gemini-1.5-flash"）
    model: String,

    /// API Key
    api_key: String,
}

impl GeminiClient {
    /// 创建新的 Gemini 客户端
    ///
    /// # 参数
    /// - `api_key`: Gemini API key
    /// - `model`: 模型名称（如 "gemini-pro"）
    /// - `endpoint`: API 端点（默认 "https://generativelanguage.googleapis.com"）
    ///
    /// # 返回
    /// - `Ok(GeminiClient)`: 成功创建
    /// - `Err(LlmError)`: API key 为空或配置错误
    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Result<Self, LlmError> {
        let api_key = api_key.into();
        if api_key.is_empty() {
            return Err(LlmError::Config("API key is required".to_string()));
        }

        let base = HttpClientBase::new(endpoint, 60)?;

        Ok(Self {
            base,
            model: model.into(),
            api_key,
        })
    }

    /// 使用默认配置
    pub fn with_defaults(api_key: impl Into<String>) -> Result<Self, LlmError> {
        Self::new(
            api_key,
            "gemini-pro",
            "https://generativelanguage.googleapis.com",
        )
    }
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
        todo!("Implement chat()")
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn stats(&self) -> ClientStats {
        self.base.stats.clone()
    }

    async fn diagnose(&self) -> String {
        format!(
            "端点: {}\n模型: {}\n状态: 待实现",
            self.base.endpoint, self.model
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_creation() {
        let client = GeminiClient::new(
            "test-key",
            "gemini-pro",
            "https://generativelanguage.googleapis.com",
        );
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.model(), "gemini-pro");
    }

    #[test]
    fn test_empty_api_key() {
        let client = GeminiClient::new("", "gemini-pro", "https://example.com");
        assert!(client.is_err());
        if let Err(e) = client {
            assert!(matches!(e, LlmError::Config(_)));
        }
    }

    #[test]
    fn test_with_defaults() {
        let client = GeminiClient::with_defaults("test-key");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.model(), "gemini-pro");
    }

    #[test]
    fn test_stats_initial() {
        let client = GeminiClient::with_defaults("test-key").unwrap();
        let stats = client.stats();
        assert_eq!(stats.total_calls(), 0);
    }
}
