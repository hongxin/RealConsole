//! Google Gemini 客户端实现
//!
//! API 格式特点：
//! - Query 参数认证 (?key=xxx)
//! - contents/parts 消息结构（非 OpenAI 格式）
//! - candidates 响应结构
//! - functionDeclarations 工具调用
//!
//! 参考文档：
//! - <https://ai.google.dev/gemini-api/docs/text-generation>
//! - <https://ai.google.dev/api/generate-content>

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
    /// - `endpoint`: API 端点（默认 `https://generativelanguage.googleapis.com`）
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

    /// 转换消息格式：OpenAI → Gemini
    ///
    /// 核心转换：
    /// - `messages` → `contents`
    /// - `{role, content}` → `{role, parts: [{text}]}`
    /// - `assistant` → `model`
    /// - 提取 `system` 消息到 `systemInstruction`
    ///
    /// # 返回
    /// - `contents`: Gemini 格式的对话内容
    /// - `system_instruction`: 系统提示词（可选）
    fn convert_messages(messages: &[Message]) -> (Vec<Value>, Option<String>) {
        let mut system_instruction = None;
        let mut contents = Vec::new();

        for msg in messages {
            match msg.role {
                MessageRole::System => {
                    // 提取系统消息
                    if let Some(content) = &msg.content {
                        system_instruction = Some(content.clone());
                    }
                }
                MessageRole::User | MessageRole::Assistant => {
                    // 转换用户/助手消息
                    let role = if msg.role == MessageRole::Assistant {
                        "model" // OpenAI 的 assistant → Gemini 的 model
                    } else {
                        "user"
                    };

                    if let Some(content) = &msg.content {
                        contents.push(json!({
                            "role": role,
                            "parts": [{"text": content}]
                        }));
                    }
                }
                MessageRole::Tool => {
                    // 工具结果消息，暂时跳过（chat_with_tools 中处理）
                    continue;
                }
            }
        }

        (contents, system_instruction)
    }

    /// 构建完整的请求 URL（包含 API key）
    fn build_url(&self) -> String {
        format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base.endpoint, self.model, self.api_key
        )
    }

    /// 执行一次 chat 请求（无重试）
    async fn chat_once(&self, messages: &[Message]) -> Result<String, LlmError> {
        let url = self.build_url();

        // 转换消息格式
        let (contents, system_instruction) = Self::convert_messages(messages);

        // 构建请求 payload
        let mut payload = json!({
            "contents": contents,
        });

        // 添加系统提示词（如果有）
        if let Some(sys_inst) = system_instruction {
            payload["systemInstruction"] = json!({
                "parts": {"text": sys_inst}
            });
        }

        // 发送请求（Gemini 使用 query param 认证，不需要 header）
        let resp = self.base.post_json(&url, payload, None).await?;

        // 处理响应
        let data = HttpClientBase::handle_response(resp).await?;

        // 提取响应内容：candidates[0].content.parts[0].text
        if let Some(candidates) = data["candidates"].as_array() {
            if let Some(first) = candidates.first() {
                if let Some(parts) = first["content"]["parts"].as_array() {
                    if let Some(first_part) = parts.first() {
                        if let Some(text) = first_part["text"].as_str() {
                            return Ok(text.to_string());
                        }
                    }
                }
            }
        }

        // 兜底：返回解析错误
        Err(LlmError::Parse(format!(
            "无法解析 Gemini 响应: {}",
            data
        )))
    }
}

#[async_trait]
impl LlmClient for GeminiClient {
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
        // 使用 HttpClientBase 的组合方法：重试 + 统计
        self.base
            .with_retry_and_stats(|| {
                let msgs = messages.clone();
                async move { self.chat_once(&msgs).await }
            })
            .await
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

    // ========== Message Conversion Tests ==========

    #[test]
    fn test_convert_simple_user_message() {
        let messages = vec![Message::user("Hello")];
        let (contents, system) = GeminiClient::convert_messages(&messages);

        assert_eq!(contents.len(), 1);
        assert!(system.is_none());
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[0]["parts"][0]["text"], "Hello");
    }

    #[test]
    fn test_convert_system_message() {
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("Hello"),
        ];
        let (contents, system) = GeminiClient::convert_messages(&messages);

        assert_eq!(contents.len(), 1); // system 不在 contents 中
        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(contents[0]["role"], "user");
    }

    #[test]
    fn test_convert_assistant_to_model() {
        let messages = vec![
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];
        let (contents, _) = GeminiClient::convert_messages(&messages);

        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[1]["role"], "model"); // assistant → model
        assert_eq!(contents[1]["parts"][0]["text"], "Hi there!");
    }

    #[test]
    fn test_convert_multi_turn_conversation() {
        let messages = vec![
            Message::system("You are helpful"),
            Message::user("What is 2+2?"),
            Message::assistant("It's 4"),
            Message::user("What about 3+3?"),
        ];
        let (contents, system) = GeminiClient::convert_messages(&messages);

        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(contents.len(), 3); // user, model, user
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[1]["role"], "model");
        assert_eq!(contents[2]["role"], "user");
    }

    #[test]
    fn test_build_url() {
        let client = GeminiClient::new(
            "test-api-key",
            "gemini-pro",
            "https://generativelanguage.googleapis.com",
        )
        .unwrap();

        let url = client.build_url();
        assert!(url.contains("/v1beta/models/gemini-pro:generateContent"));
        assert!(url.contains("?key=test-api-key"));
    }

    // ========== Integration Tests (require real API key) ==========

    #[tokio::test]
    #[ignore] // Requires GEMINI_API_KEY
    async fn test_chat_real() {
        let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return;
        }

        let client = GeminiClient::with_defaults(api_key).unwrap();
        let messages = vec![Message::user("Say 'test successful' if you can read this")];

        let result = client.chat(messages).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.is_empty());
        println!("Gemini response: {}", response);
    }
}
