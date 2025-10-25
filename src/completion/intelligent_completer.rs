//! 智能补全器
//!
//! 基于 LLM 的智能预测补全：
//! - Deepseek 集成
//! - Ollama 支持（预留）
//! - 上下文感知
//! - 智能预测
//!
//! # 特性
//!
//! - **响应时间**: 50-300ms（设置超时）
//! - **确定性**: 0.0-0.4（低确定性，高预测性）
//! - **智能程度**: 基于上下文的 AI 预测

use super::types::{Candidate, CompletionSource};
use crate::history::{HistoryManager, SortStrategy};
use crate::llm::{LlmClient, Message};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// 智能补全器
///
/// # 补全策略
///
/// 1. **上下文构建**: 收集历史记录、当前目录等上下文
/// 2. **LLM 预测**: 使用 LLM 基于上下文预测可能的命令
/// 3. **结果解析**: 解析 LLM 输出，生成候选列表
pub struct IntelligentCompleter {
    /// LLM 客户端（Deepseek 或 Ollama）
    llm_client: Arc<dyn LlmClient>,

    /// 历史管理器
    history: Arc<RwLock<HistoryManager>>,

    /// LLM 调用超时时间（毫秒）
    timeout_ms: u64,

    /// 最大返回候选数
    max_candidates: usize,

    /// 是否启用上下文增强
    enable_context: bool,
}

impl IntelligentCompleter {
    /// 创建新的智能补全器
    ///
    /// # 参数
    ///
    /// * `llm_client` - LLM 客户端（Deepseek 或 Ollama）
    /// * `history` - 历史管理器
    pub fn new(
        llm_client: Arc<dyn LlmClient>,
        history: Arc<RwLock<HistoryManager>>,
    ) -> Self {
        Self {
            llm_client,
            history,
            timeout_ms: 2000, // 默认 2 秒超时
            max_candidates: 3, // 最多 3 个 LLM 建议
            enable_context: true,
        }
    }

    /// 设置 LLM 调用超时（毫秒）
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 设置最大候选数
    pub fn with_max_candidates(mut self, max: usize) -> Self {
        self.max_candidates = max;
        self
    }

    /// 设置是否启用上下文增强
    pub fn with_context(mut self, enable: bool) -> Self {
        self.enable_context = enable;
        self
    }

    /// 统一补全入口
    ///
    /// # 参数
    ///
    /// * `input` - 用户输入
    ///
    /// # 返回
    ///
    /// LLM 预测的补全候选列表
    pub async fn complete(&self, input: &str) -> Vec<Candidate> {
        // 输入太短，不调用 LLM
        if input.trim().is_empty() || input.len() < 2 {
            return Vec::new();
        }

        // 构建上下文
        let context = if self.enable_context {
            self.build_context().await
        } else {
            String::new()
        };

        // 构建 prompt
        let prompt = self.build_prompt(input, &context);

        // 调用 LLM（带超时）
        let llm_response = match self.call_llm_with_timeout(&prompt).await {
            Ok(response) => response,
            Err(_) => return Vec::new(), // 超时或错误，返回空列表
        };

        // 解析 LLM 响应
        self.parse_llm_response(&llm_response, input)
    }

    /// 构建上下文信息
    ///
    /// 收集：
    /// - 最近的历史命令（前 5 条）
    /// - 当前工作目录
    async fn build_context(&self) -> String {
        let mut context_parts = Vec::new();

        // 1. 当前目录
        if let Ok(cwd) = std::env::current_dir() {
            context_parts.push(format!(
                "Current directory: {}",
                cwd.to_string_lossy()
            ));
        }

        // 2. 最近的历史命令
        if let Ok(history) = self.history.try_read() {
            let recent = history.all(SortStrategy::Time);
            if !recent.is_empty() {
                let commands: Vec<String> = recent
                    .iter()
                    .take(5)
                    .map(|e| e.command.clone())
                    .collect();
                context_parts.push(format!("Recent commands:\n- {}", commands.join("\n- ")));
            }
        }

        context_parts.join("\n\n")
    }

    /// 构建 LLM prompt
    ///
    /// # 设计原则
    ///
    /// - 清晰的任务描述
    /// - 提供足够的上下文
    /// - 要求结构化输出（每行一个建议）
    fn build_prompt(&self, input: &str, context: &str) -> String {
        format!(
            r#"You are a shell command completion assistant.

{context}

User is typing: {input}

Suggest {max} most likely shell commands to complete this input.
Each suggestion should be on a new line.
Only output the commands, no explanations or numbering.

Suggestions:"#,
            context = if context.is_empty() {
                "No context available."
            } else {
                context
            },
            input = input,
            max = self.max_candidates
        )
    }

    /// 调用 LLM（带超时控制）
    async fn call_llm_with_timeout(&self, prompt: &str) -> Result<String, String> {
        let messages = vec![Message::user(prompt)];

        let llm_future = self.llm_client.chat(messages);

        match timeout(Duration::from_millis(self.timeout_ms), llm_future).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(format!("LLM error: {}", e)),
            Err(_) => Err("LLM timeout".to_string()),
        }
    }

    /// 解析 LLM 响应
    ///
    /// 期望格式：每行一个命令建议
    ///
    /// # 评分策略
    ///
    /// - 第一个建议: 0.35-0.4
    /// - 第二个建议: 0.25-0.35
    /// - 第三个建议: 0.15-0.25
    fn parse_llm_response(&self, response: &str, _input: &str) -> Vec<Candidate> {
        response
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#')) // 过滤注释
            .filter(|line| !line.starts_with("```")) // 过滤 markdown 代码块
            .take(self.max_candidates)
            .enumerate()
            .map(|(idx, suggestion)| {
                // 递减评分（第一个建议最高）
                let base_score = match idx {
                    0 => 0.35,
                    1 => 0.25,
                    2 => 0.15,
                    _ => 0.1,
                };

                // 随机微调（避免完全相同的分数）
                let score = (base_score + (idx as f64 * 0.001)).min(0.4);

                Candidate::with_score(
                    suggestion.to_string(),
                    "AI suggestion".to_string(),
                    score,
                    CompletionSource::Intelligent,
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatResponse, LlmError};
    use async_trait::async_trait;

    /// Mock LLM 客户端（用于测试）
    struct MockLlmClient {
        response: String,
        should_fail: bool,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
            if self.should_fail {
                Err(LlmError::Other("Mock error".to_string()))
            } else {
                Ok(self.response.clone())
            }
        }

        async fn chat_with_tools(
            &self,
            _messages: Vec<Message>,
            _tools: Vec<serde_json::Value>,
        ) -> Result<ChatResponse, LlmError> {
            Ok(ChatResponse::text("mock".to_string()))
        }

        fn model(&self) -> &str {
            "mock-model"
        }

        fn stats(&self) -> crate::llm::ClientStats {
            use crate::llm::ClientStats;
            ClientStats::new()
        }

        async fn diagnose(&self) -> String {
            "Mock client OK".to_string()
        }
    }

    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let mut history = HistoryManager::new("test_history.json", 100);
        history.add("git status".to_string(), true);
        history.add("cargo test".to_string(), true);
        Arc::new(RwLock::new(history))
    }

    #[tokio::test]
    async fn test_parse_llm_response() {
        let mock_client = Arc::new(MockLlmClient {
            response: "git status\ncargo build\ncargo test".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let response = "git status\ncargo build\ncargo test";
        let candidates = completer.parse_llm_response(response, "git");

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].text, "git status");
        assert_eq!(candidates[1].text, "cargo build");
        assert_eq!(candidates[2].text, "cargo test");

        // 检查评分递减
        assert!(candidates[0].score > candidates[1].score);
        assert!(candidates[1].score > candidates[2].score);

        // 检查评分范围（0.0-0.4）
        for candidate in &candidates {
            assert!(candidate.score >= 0.0 && candidate.score <= 0.4);
        }
    }

    #[tokio::test]
    async fn test_parse_filters_empty_lines() {
        let mock_client = Arc::new(MockLlmClient {
            response: "".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let response = "git status\n\n\ncargo build\n";
        let candidates = completer.parse_llm_response(response, "");

        assert_eq!(candidates.len(), 2); // 只有非空行
    }

    #[tokio::test]
    async fn test_parse_filters_comments() {
        let mock_client = Arc::new(MockLlmClient {
            response: "".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let response = "git status\n# This is a comment\ncargo build";
        let candidates = completer.parse_llm_response(response, "");

        assert_eq!(candidates.len(), 2);
        assert!(!candidates.iter().any(|c| c.text.starts_with('#')));
    }

    #[tokio::test]
    async fn test_complete_empty_input() {
        let mock_client = Arc::new(MockLlmClient {
            response: "git status".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let candidates = completer.complete("").await;
        assert!(candidates.is_empty()); // 空输入不调用 LLM
    }

    #[tokio::test]
    async fn test_complete_with_mock_llm() {
        let mock_client = Arc::new(MockLlmClient {
            response: "git commit -m 'test'\ngit push origin main".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history())
            .with_context(false); // 禁用上下文以简化测试

        let candidates = completer.complete("git").await;

        assert!(!candidates.is_empty());
        assert!(candidates[0].text.contains("git"));
    }

    #[tokio::test]
    async fn test_llm_error_handling() {
        let mock_client = Arc::new(MockLlmClient {
            response: "".to_string(),
            should_fail: true,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let candidates = completer.complete("test").await;
        assert!(candidates.is_empty()); // 错误时返回空列表
    }

    #[tokio::test]
    async fn test_max_candidates_limit() {
        let mock_client = Arc::new(MockLlmClient {
            response: "cmd1\ncmd2\ncmd3\ncmd4\ncmd5".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history())
            .with_max_candidates(2)
            .with_context(false);

        let candidates = completer.complete("test").await;

        assert_eq!(candidates.len(), 2); // 最多返回 2 个
    }

    #[tokio::test]
    async fn test_intelligent_score_range() {
        let mock_client = Arc::new(MockLlmClient {
            response: "git status\ncargo test".to_string(),
            should_fail: false,
        });

        let completer = IntelligentCompleter::new(mock_client, create_test_history());

        let response = "git status\ncargo test";
        let candidates = completer.parse_llm_response(response, "");

        // 智能补全的分数应该在 0.0-0.4 范围内
        for candidate in candidates {
            assert!(
                candidate.score >= 0.0 && candidate.score <= 0.4,
                "Score {} out of range [0.0, 0.4]",
                candidate.score
            );
            assert_eq!(candidate.source, CompletionSource::Intelligent);
        }
    }
}
