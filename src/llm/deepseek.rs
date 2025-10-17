//! Deepseek 客户端实现
//!
//! 特色功能：
//! - Bearer Token 认证
//! - 标准 OpenAI compatible API
//! - 速率限制处理
//! - 自动重试机制（通过 HttpClientBase）
//! - 流式输出支持 (SSE)

use super::{async_trait, ChatResponse, ClientStats, FunctionCall, LlmClient, LlmError, Message, ToolCall};
use super::http_base::HttpClientBase;
use futures::StreamExt;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

/// Deepseek 客户端
pub struct DeepseekClient {
    /// HTTP 客户端基础层（提供通用功能）
    base: HttpClientBase,

    /// 模型名称
    model: String,

    /// API Key（用于 Bearer 认证）
    api_key: String,
}

impl DeepseekClient {
    /// 创建新的 Deepseek 客户端
    ///
    /// # 参数
    /// - `api_key`: Deepseek API key（必需）
    /// - `model`: 模型名称（如 "deepseek-chat"）
    /// - `endpoint`: API 端点 URL
    ///
    /// # 返回
    /// - `Ok(DeepseekClient)`: 成功创建
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

        // 使用 HttpClientBase 创建 HTTP 客户端（60秒超时）
        let base = HttpClientBase::new(endpoint, 60)?;

        Ok(Self {
            base,
            model: model.into(),
            api_key,
        })
    }

    /// 使用默认配置（需要从环境变量读取 API key）
    ///
    /// 使用默认的 Deepseek 端点和模型
    #[allow(dead_code)]
    pub fn with_defaults(api_key: impl Into<String>) -> Result<Self, LlmError> {
        Self::new(api_key, "deepseek-chat", "https://api.deepseek.com/v1")
    }

    /// 创建认证 headers
    ///
    /// 返回包含 Bearer token 的 header map
    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_key).parse().unwrap(),
        );
        headers
    }

    /// 执行 chat 请求（单次，无重试）
    ///
    /// # 参数
    /// - `messages`: 对话消息列表
    ///
    /// # 返回
    /// - `Ok(String)`: 响应内容
    /// - `Err(LlmError)`: HTTP 错误或解析错误
    async fn chat_once(&self, messages: &[Message]) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.base.endpoint);

        let payload = json!({
            "model": self.model,
            "messages": messages,
        });

        // 使用 HttpClientBase 发送 POST 请求
        let resp = self.base.post_json(&url, payload, Some(self.auth_headers())).await?;

        // 使用 HttpClientBase 处理响应
        let data = HttpClientBase::handle_response(resp).await?;

        // 提取响应内容
        if let Some(choices) = data["choices"].as_array() {
            if let Some(first) = choices.first() {
                if let Some(content) = first["message"]["content"].as_str() {
                    return Ok(content.to_string());
                }
            }
        }

        // 如果没有 choices，返回整个响应
        Ok(data.to_string())
    }

    /// 流式 chat（实时输出，SSE）
    ///
    /// Deepseek 特有功能：支持服务端推送（SSE）实时流式输出
    ///
    /// # 参数
    /// - `messages`: 对话消息列表
    /// - `callback`: 每次收到内容片段时的回调函数
    ///
    /// # 返回
    /// - `Ok(String)`: 完整的响应内容
    /// - `Err(LlmError)`: 错误
    pub async fn chat_stream<F>(&self, messages: &[Message], mut callback: F) -> Result<String, LlmError>
    where
        F: FnMut(&str),
    {
        let url = format!("{}/chat/completions", self.base.endpoint);

        let payload = json!({
            "model": self.model,
            "messages": messages,
            "stream": true,  // 启用流式输出
        });

        // 发送流式请求
        let resp = self.base.post_json(&url, payload, Some(self.auth_headers())).await?;

        let status = resp.status();
        if !status.is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(LlmError::Http {
                status: status.as_u16(),
                message: error_text,
            });
        }

        // 处理流式响应
        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut full_response = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);

            buffer.push_str(&text);

            // 处理 SSE 格式：data: {...}\n\n
            while let Some(data_start) = buffer.find("data: ") {
                if let Some(newline_pos) = buffer[data_start..].find("\n\n") {
                    // 复制行内容到 String，避免借用冲突
                    let line = buffer[data_start + 6..data_start + newline_pos].to_string();
                    buffer.drain(..data_start + newline_pos + 2);

                    // 跳过 [DONE] 标记
                    if line.trim() == "[DONE]" {
                        break;
                    }

                    // 解析 JSON
                    if let Ok(json) = serde_json::from_str::<Value>(&line) {
                        if let Some(choices) = json["choices"].as_array() {
                            if let Some(first) = choices.first() {
                                if let Some(delta) = first.get("delta") {
                                    if let Some(content) = delta["content"].as_str() {
                                        // 实时输出
                                        callback(content);
                                        full_response.push_str(content);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    break; // 等待更多数据
                }
            }
        }

        Ok(full_response)
    }
}

#[async_trait]
impl LlmClient for DeepseekClient {
    /// 聊天接口（带自动重试和统计）
    ///
    /// 使用 HttpClientBase 提供的重试逻辑和统计记录
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
        // 使用 HttpClientBase 的组合方法：重试 + 统计
        self.base
            .with_retry_and_stats(|| {
                // 捕获 messages 的引用，避免 move
                let msgs = messages.clone();
                async move { self.chat_once(&msgs).await }
            })
            .await
    }

    /// 带工具的聊天接口（Function Calling）
    ///
    /// 支持 OpenAI Function Calling 格式的工具调用
    async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<Value>,
    ) -> Result<ChatResponse, LlmError> {
        // 使用 HttpClientBase 的统计记录包装器
        self.base
            .record_operation(|| async {
                let url = format!("{}/chat/completions", self.base.endpoint);

                // 构建请求 payload
                let mut payload = json!({
                    "model": self.model,
                    "messages": messages,
                });

                // 添加工具定义
                if !tools.is_empty() {
                    payload["tools"] = json!(tools);
                    payload["tool_choice"] = json!("auto"); // 让模型自动决定是否调用工具
                }

                // 发送请求
                let resp = self.base.post_json(&url, payload, Some(self.auth_headers())).await?;

                // 处理响应
                let data = HttpClientBase::handle_response(resp).await?;

                // 解析响应
                if let Some(choices) = data["choices"].as_array() {
                    if let Some(first) = choices.first() {
                        let message = &first["message"];

                        // 检查是否有工具调用
                        if let Some(tool_calls) = message["tool_calls"].as_array() {
                            if !tool_calls.is_empty() {
                                // 解析工具调用
                                let mut parsed_tool_calls = Vec::new();

                                for tc in tool_calls {
                                    if let (Some(id), Some(func)) = (
                                        tc["id"].as_str(),
                                        tc["function"].as_object(),
                                    ) {
                                        if let (Some(name), Some(args)) = (
                                            func.get("name").and_then(|v| v.as_str()),
                                            func.get("arguments").and_then(|v| v.as_str()),
                                        ) {
                                            parsed_tool_calls.push(ToolCall {
                                                id: id.to_string(),
                                                call_type: "function".to_string(),
                                                function: FunctionCall {
                                                    name: name.to_string(),
                                                    arguments: args.to_string(),
                                                },
                                            });
                                        }
                                    }
                                }

                                return Ok(ChatResponse::with_tools(parsed_tool_calls));
                            }
                        }

                        // 没有工具调用，返回文本响应
                        if let Some(content) = message["content"].as_str() {
                            return Ok(ChatResponse::text(content.to_string()));
                        }
                    }
                }

                // 兜底：返回解析错误
                Err(LlmError::Parse(format!("无法解析 LLM 响应: {}", data)))
            })
            .await
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn stats(&self) -> ClientStats {
        // 直接使用 HttpClientBase 的 stats
        self.base.stats.clone()
    }

    async fn diagnose(&self) -> String {
        let mut lines = vec![
            format!("端点: {}", self.base.endpoint),
            format!("模型: {}", self.model),
        ];

        // 简单 ping 测试
        let test_messages = vec![Message::user("ping")];

        match self.chat_once(&test_messages).await {
            Ok(_) => {
                lines.push("✓ API 连接正常".to_string());
            }
            Err(e) => {
                lines.push(format!("✗ API 连接失败: {}", e));
                lines.push("建议: 检查 API key 和网络连接".to_string());
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_creation() {
        let client = DeepseekClient::new("test-key", "deepseek-chat", "https://api.deepseek.com/v1");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.model(), "deepseek-chat");
    }

    #[test]
    fn test_empty_api_key() {
        let client = DeepseekClient::new("", "deepseek-chat", "https://api.deepseek.com/v1");
        assert!(client.is_err());
    }

    #[tokio::test]
    #[ignore] // 需要真实 API key
    async fn test_deepseek_chat() {
        let api_key = std::env::var("DEEPSEEK_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
            return;
        }

        let client = DeepseekClient::with_defaults(api_key).unwrap();
        let messages = vec![Message::user("Hello")];

        let result = client.chat(messages).await;
        println!("{:?}", result);
    }

    // ========== Mock 测试 ==========

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_mockito_basic() {
        // 基础 mockito 测试
        let mut server = mockito::Server::new_async().await;
        let _mock = server
            .mock("GET", "/test")
            .with_status(200)
            .with_body("test response")
            .create_async()
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/test", server.url());
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), 200);
        let body = resp.text().await.unwrap();
        assert_eq!(body, "test response");
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_success() {
        let mut server = mockito::Server::new_async().await;

        // Mock 成功响应
        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "content": "Hello! How can I help you?"
                    }
                }]
            }"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let result = client.chat(vec![Message::user("Hi")]).await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Hello"));
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_http_error_400() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(400)
            .with_body(r#"{"error": "Bad Request"}"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let result = client.chat(vec![Message::user("Hi")]).await;

        assert!(result.is_err());
        if let Err(LlmError::Http { status, .. }) = result {
            assert_eq!(status, 400);
        } else {
            panic!("Expected Http error");
        }
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_http_error_500() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(500)
            .with_body(r#"{"error": "Internal Server Error"}"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let result = client.chat(vec![Message::user("Hi")]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_with_tools_success() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "tool_calls": [{
                            "id": "call_123",
                            "function": {
                                "name": "calculator",
                                "arguments": "{\"expression\": \"2+2\"}"
                            }
                        }]
                    }
                }]
            }"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let tools = vec![json!({
            "type": "function",
            "function": {
                "name": "calculator",
                "description": "Calculate math expressions"
            }
        })];

        let result = client.chat_with_tools(vec![Message::user("Calculate 2+2")], tools).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(!response.tool_calls.is_empty());
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].function.name, "calculator");
        assert!(!response.is_final);
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_with_tools_text_response() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body(r#"{
                "choices": [{
                    "message": {
                        "content": "I don't need tools for this."
                    }
                }]
            }"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let result = client.chat_with_tools(vec![Message::user("Hello")], vec![]).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.tool_calls.is_empty());
        assert!(response.content.is_some());
        assert!(response.is_final);
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_chat_invalid_json() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body("invalid json")
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();
        let result = client.chat(vec![Message::user("Hi")]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore] // TODO: mockito 1.6/1.7 返回 502，待替换为其他 mock 库
    async fn test_stats_tracking() {
        let mut server = mockito::Server::new_async().await;

        let _mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_body(r#"{
                "choices": [{
                    "message": {"content": "Response"}
                }]
            }"#)
            .create_async()
            .await;

        let client = DeepseekClient::new("test-key", "test-model", server.url()).unwrap();

        // Initial stats
        let stats_before = client.stats();
        assert_eq!(stats_before.total_calls(), 0);

        // Make a call
        let _ = client.chat(vec![Message::user("Hi")]).await;

        // Check stats updated
        let stats_after = client.stats();
        assert_eq!(stats_after.total_calls(), 1);
        assert_eq!(stats_after.total_success(), 1);
    }
}
