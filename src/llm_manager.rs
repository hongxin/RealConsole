//! LLM 客户端管理
//!
//! 负责：
//! - 持有 LLM 客户端实例
//! - 管理主备 LLM（primary/fallback）
//! - 提供统一的调用接口

use crate::llm::{DeepseekClient, LlmClient, LlmError, Message};
use std::sync::Arc;

/// LLM 管理器
pub struct LlmManager {
    /// 主 LLM（通常是远程 API）
    primary: Option<Arc<dyn LlmClient>>,
    /// 备用 LLM（通常是本地 Ollama）
    fallback: Option<Arc<dyn LlmClient>>,
    /// Deepseek 客户端引用（用于流式输出）
    deepseek_client: Option<Arc<DeepseekClient>>,
}

impl LlmManager {
    pub fn new() -> Self {
        Self {
            primary: None,
            fallback: None,
            deepseek_client: None,
        }
    }

    /// 设置主 LLM
    pub fn set_primary(&mut self, client: Arc<dyn LlmClient>) {
        self.primary = Some(client);
    }

    /// 设置备用 LLM
    pub fn set_fallback(&mut self, client: Arc<dyn LlmClient>) {
        self.fallback = Some(client);
    }

    /// 设置 Deepseek 客户端（用于流式输出）
    pub fn set_deepseek(&mut self, client: Arc<DeepseekClient>) {
        self.deepseek_client = Some(client);
    }

    /// 获取主 LLM
    pub fn primary(&self) -> Option<&Arc<dyn LlmClient>> {
        self.primary.as_ref()
    }

    /// 获取备用 LLM
    pub fn fallback(&self) -> Option<&Arc<dyn LlmClient>> {
        self.fallback.as_ref()
    }

    /// 简单 chat（使用 fallback 优先）
    pub async fn chat(&self, query: &str) -> Result<String, LlmError> {
        let client = self
            .fallback
            .as_ref()
            .or(self.primary.as_ref())
            .ok_or_else(|| LlmError::Config("No LLM configured".to_string()))?;

        let messages = vec![Message::user(query)];
        client.chat(messages).await
    }

    /// 流式 chat（实时输出）
    pub async fn chat_stream<F>(&self, query: &str, callback: F) -> Result<String, LlmError>
    where
        F: FnMut(&str),
    {
        let messages = vec![Message::user(query)];

        // 优先使用 Deepseek 流式输出
        if let Some(deepseek_client) = &self.deepseek_client {
            return deepseek_client.chat_stream(&messages, callback).await;
        }

        // 否则降级到普通 chat（一次性输出）
        let client = self
            .fallback
            .as_ref()
            .or(self.primary.as_ref())
            .ok_or_else(|| LlmError::Config("No LLM configured".to_string()))?;

        let response = client.chat(messages).await?;
        Ok(response)
    }

    /// 诊断主 LLM
    pub async fn diagnose_primary(&self) -> String {
        match &self.primary {
            Some(client) => client.diagnose().await,
            None => "(未配置)".to_string(),
        }
    }

    /// 诊断备用 LLM
    pub async fn diagnose_fallback(&self) -> String {
        match &self.fallback {
            Some(client) => client.diagnose().await,
            None => "(未配置)".to_string(),
        }
    }
}

impl Default for LlmManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ClientStats;
    use async_trait::async_trait;

    // Mock LLM client for testing
    struct MockClient {
        name: String,
    }

    #[async_trait]
    impl LlmClient for MockClient {
        async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
            Ok(format!("{}: received {} messages", self.name, messages.len()))
        }

        fn model(&self) -> &str {
            &self.name
        }

        fn stats(&self) -> ClientStats {
            ClientStats::new()
        }

        async fn diagnose(&self) -> String {
            format!("{} is ok", self.name)
        }
    }

    #[tokio::test]
    async fn test_llm_manager_basic() {
        let mut manager = LlmManager::new();
        assert!(manager.primary().is_none());
        assert!(manager.fallback().is_none());

        let primary = Arc::new(MockClient {
            name: "primary".to_string(),
        });
        manager.set_primary(primary);
        assert!(manager.primary().is_some());

        let fallback = Arc::new(MockClient {
            name: "fallback".to_string(),
        });
        manager.set_fallback(fallback);
        assert!(manager.fallback().is_some());
    }

    #[tokio::test]
    async fn test_llm_manager_chat() {
        let mut manager = LlmManager::new();
        let fallback = Arc::new(MockClient {
            name: "fallback".to_string(),
        });
        manager.set_fallback(fallback);

        let result = manager.chat("test").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("fallback"));
    }

    #[tokio::test]
    async fn test_llm_manager_no_client() {
        let manager = LlmManager::new();
        let result = manager.chat("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_llm_manager_diagnose() {
        let mut manager = LlmManager::new();
        let primary = Arc::new(MockClient {
            name: "primary".to_string(),
        });
        manager.set_primary(primary);

        let diag = manager.diagnose_primary().await;
        assert!(diag.contains("primary"));
    }
}
