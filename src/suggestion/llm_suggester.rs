//! 基于 LLM 的建议生成器
//!
//! 使用 LLM（Deepseek/Ollama）生成智能建议

use super::types::{Suggestion, SuggestionCategory, SuggestionContext, SuggestionSource};
use crate::llm::{LlmClient, Message};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// 基于 LLM 的建议生成器
pub struct LlmSuggester {
    /// LLM 客户端
    llm_client: Arc<dyn LlmClient>,

    /// 超时时间（毫秒）
    timeout_ms: u64,

    /// 最大建议数
    max_suggestions: usize,
}

impl LlmSuggester {
    /// 创建新的 LLM 建议生成器
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self {
            llm_client,
            timeout_ms: 2000, // 2 秒超时
            max_suggestions: 3,
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// 生成建议
    pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        // 构建 Prompt
        let prompt = self.build_prompt(context);

        // 调用 LLM（带超时）
        let llm_response = match self.call_llm_with_timeout(&prompt).await {
            Ok(response) => response,
            Err(_) => return Vec::new(), // 超时或错误，返回空列表
        };

        // 解析响应
        self.parse_llm_response(&llm_response)
    }

    /// 构建 LLM Prompt
    fn build_prompt(&self, context: &SuggestionContext) -> String {
        let mut prompt = String::from("You are a helpful shell command assistant.\n\n");

        // 1. 添加当前目录信息
        prompt.push_str(&format!(
            "Current directory: {}\n",
            context.current_dir.display()
        ));

        // 2. 添加项目类型信息
        if let Some(ref project_type) = context.project_type {
            prompt.push_str(&format!("Project type: {:?}\n", project_type));
        }

        // 3. 添加最近命令信息
        if !context.recent_commands.is_empty() {
            prompt.push_str("\nRecent commands:\n");
            for (i, cmd) in context.recent_commands.iter().take(3).enumerate() {
                prompt.push_str(&format!("  {}. {}\n", i + 1, cmd));
            }
        }

        // 4. 添加任务描述
        prompt.push_str("\nSuggest ");
        prompt.push_str(&self.max_suggestions.to_string());
        prompt.push_str(" useful shell commands that the user might want to run next.\n");

        // 5. 如果上一次命令失败，特别说明
        if context.last_command_failed {
            prompt.push_str("\nNote: The last command failed. Consider suggesting diagnostic or fix commands.\n");
        }

        // 6. 输出格式说明
        prompt.push_str("\nOutput format (one command per line):\n");
        prompt.push_str("command | description\n\n");
        prompt.push_str("Example:\n");
        prompt.push_str("cargo test | Run all tests\n");
        prompt.push_str("git status | Check repository status\n\n");
        prompt.push_str("Suggestions:\n");

        prompt
    }

    /// 调用 LLM（带超时）
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
    /// 期望格式：
    /// ```
    /// command | description
    /// another command | description
    /// ```
    fn parse_llm_response(&self, response: &str) -> Vec<Suggestion> {
        response
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter(|line| !line.starts_with('#')) // 过滤注释
            .filter(|line| !line.to_lowercase().contains("suggestion")) // 过滤标题行
            .filter_map(|line| {
                // 分割命令和描述
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let command = parts[0].trim().to_string();
                    let description = parts[1].trim().to_string();

                    // 过滤掉空的或明显错误的建议
                    if command.is_empty() || command.len() > 200 {
                        return None;
                    }

                    Some(Suggestion::new(
                        command.clone(),
                        description,
                        0.65, // LLM 建议给中等分数
                        SuggestionSource::Llm,
                    ).with_category(self.categorize_command(&command)))
                } else {
                    // 如果没有 |，尝试整行作为命令
                    let command = line.trim();
                    if !command.is_empty() && command.len() < 200 {
                        Some(Suggestion::new(
                            command,
                            "AI suggestion",
                            0.60,
                            SuggestionSource::Llm,
                        ))
                    } else {
                        None
                    }
                }
            })
            .take(self.max_suggestions)
            .collect()
    }

    /// 为命令分类
    fn categorize_command(&self, command: &str) -> SuggestionCategory {
        let cmd = command.to_lowercase();

        if cmd.starts_with("git") {
            SuggestionCategory::Git
        } else if cmd.contains("test") {
            SuggestionCategory::Testing
        } else if cmd.contains("build") {
            SuggestionCategory::Building
        } else if cmd.contains("deploy") {
            SuggestionCategory::Deployment
        } else {
            SuggestionCategory::General
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatResponse, LlmError};
    use async_trait::async_trait;

    /// Mock LLM 客户端
    struct MockLlmClient {
        response: String,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
            Ok(self.response.clone())
        }

        async fn chat_with_tools(
            &self,
            _messages: Vec<Message>,
            _tools: Vec<serde_json::Value>,
        ) -> Result<ChatResponse, LlmError> {
            Ok(ChatResponse::text(self.response.clone()))
        }

        fn model(&self) -> &str {
            "mock"
        }

        fn stats(&self) -> crate::llm::ClientStats {
            crate::llm::ClientStats::new()
        }

        async fn diagnose(&self) -> String {
            "OK".to_string()
        }
    }

    #[tokio::test]
    async fn test_parse_llm_response() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "cargo test | Run all tests\ngit status | Check repository status\n".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        let response = "cargo test | Run all tests\ngit status | Check repository status\n";
        let suggestions = suggester.parse_llm_response(response);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].command, "cargo test");
        assert_eq!(suggestions[0].description, "Run all tests");
        assert_eq!(suggestions[1].command, "git status");
    }

    #[tokio::test]
    async fn test_parse_without_description() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        let response = "cargo build\ngit status\n";
        let suggestions = suggester.parse_llm_response(response);

        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].command, "cargo build");
        assert_eq!(suggestions[0].description, "AI suggestion");
    }

    #[tokio::test]
    async fn test_suggest_with_context() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "cargo test | Run tests\ngit commit | Commit changes\n".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        let mut context = SuggestionContext::from_env();
        context.recent_commands.push("cargo build".to_string());

        let suggestions = suggester.suggest(&context).await;

        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].source, SuggestionSource::Llm);
    }

    #[test]
    fn test_build_prompt() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        let mut context = SuggestionContext::from_env();
        context.recent_commands.push("git add .".to_string());
        context.last_command_failed = false;

        let prompt = suggester.build_prompt(&context);

        assert!(prompt.contains("Current directory"));
        assert!(prompt.contains("Recent commands"));
        assert!(prompt.contains("git add ."));
    }

    #[test]
    fn test_prompt_with_failure() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        let mut context = SuggestionContext::from_env();
        context.last_command_failed = true;

        let prompt = suggester.build_prompt(&context);

        assert!(prompt.contains("last command failed"));
        assert!(prompt.contains("diagnostic"));
    }

    #[test]
    fn test_categorize_command() {
        let mock_llm = Arc::new(MockLlmClient {
            response: "".to_string(),
        });

        let suggester = LlmSuggester::new(mock_llm);

        assert_eq!(
            suggester.categorize_command("git status"),
            SuggestionCategory::Git
        );
        assert_eq!(
            suggester.categorize_command("npm test"),
            SuggestionCategory::Testing
        );
        assert_eq!(
            suggester.categorize_command("cargo build"),
            SuggestionCategory::Building
        );
    }
}
