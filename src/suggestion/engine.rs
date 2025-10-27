//! 建议引擎
//!
//! 统一入口，协调所有建议生成器

use super::context_suggester::ContextSuggester;
use super::history_suggester::HistorySuggester;
use super::llm_suggester::LlmSuggester;
use super::ranker::SuggestionRanker;
use super::types::{
    Suggestion, SuggestionConfig, SuggestionContext, SuggestionTrigger,
};
use crate::history::HistoryManager;
use crate::likan::LiEnhancer; // ✨ 引入离增强器
use crate::llm::LlmClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 建议引擎
///
/// "一分为三"架构的统一入口：
/// - Context：基于项目类型和上下文
/// - History：基于命令历史
/// - LLM：基于 AI 推理
/// - Li（离）：基于模式学习的增强（✨ 新增）
pub struct SuggestionEngine {
    /// 上下文建议生成器
    context_suggester: ContextSuggester,

    /// 历史建议生成器
    history_suggester: HistorySuggester,

    /// LLM 建议生成器（可选）
    llm_suggester: Option<LlmSuggester>,

    /// 建议排序器
    ranker: SuggestionRanker,

    /// 离增强器（✨ 炼化炉的离端）
    li_enhancer: Arc<RwLock<LiEnhancer>>,

    /// 配置
    config: SuggestionConfig,
}

impl SuggestionEngine {
    /// 创建新的建议引擎
    pub fn new(history: Arc<RwLock<HistoryManager>>, config: SuggestionConfig) -> Self {
        let ranker = SuggestionRanker::new()
            .with_max_suggestions(config.max_suggestions)
            .with_min_score(config.min_score);

        Self {
            context_suggester: ContextSuggester::new(),
            history_suggester: HistorySuggester::new(history),
            llm_suggester: None,
            ranker,
            li_enhancer: Arc::new(RwLock::new(LiEnhancer::new())), // ✨ 初始化离增强器
            config,
        }
    }

    /// 设置离增强器
    ///
    /// 用于与炼化炉共享同一个增强器实例
    pub fn with_li_enhancer(mut self, li_enhancer: Arc<RwLock<LiEnhancer>>) -> Self {
        self.li_enhancer = li_enhancer;
        self
    }

    /// 获取离增强器引用
    ///
    /// 供炼化炉使用
    pub fn li_enhancer(&self) -> Arc<RwLock<LiEnhancer>> {
        Arc::clone(&self.li_enhancer)
    }

    /// 设置 LLM 客户端
    pub fn with_llm(mut self, llm_client: Arc<dyn LlmClient>) -> Self {
        self.llm_suggester = Some(
            LlmSuggester::new(llm_client).with_timeout(self.config.llm_timeout_ms),
        );
        self
    }

    /// 生成建议
    ///
    /// 采用"一分为三"的融合策略：
    /// 1. 并行调用三个建议生成器
    /// 2. 收集所有建议
    /// 3. 通过排序器融合和排序
    /// 4. ✨ 通过离增强器优化（炼化）
    pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut all_suggestions = Vec::new();

        // 1. 上下文建议（如果启用）
        if self.config.enable_context {
            let context_suggestions = self.context_suggester.suggest(context).await;
            all_suggestions.extend(context_suggestions);
        }

        // 2. 历史建议（如果启用）
        if self.config.enable_history {
            let history_suggestions = self.history_suggester.suggest(context).await;
            all_suggestions.extend(history_suggestions);
        }

        // 3. LLM 建议（如果启用且可用）
        if self.config.enable_llm {
            if let Some(ref llm_suggester) = self.llm_suggester {
                let llm_suggestions = llm_suggester.suggest(context).await;
                all_suggestions.extend(llm_suggestions);
            }
        }

        // 4. 排序和融合
        let mut ranked_suggestions = self.ranker.rank(all_suggestions);

        // 5. ✨ 离（☲火）增强：应用学习到的模式优化建议
        {
            let li = self.li_enhancer.read().await;

            // 应用评分增强
            ranked_suggestions = li.enhance(ranked_suggestions);

            // 添加上下文相关的额外建议（基于序列和错误修复模式）
            let last_command = context.recent_commands.last().map(|s| s.as_str());
            let last_error = if context.last_command_failed {
                context.last_command_output.as_deref()
            } else {
                None
            };

            let additional = li.add_contextual_suggestions(last_command, last_error);
            ranked_suggestions.extend(additional);

            // 重新排序（因为新增了建议）
            ranked_suggestions.sort_by(|a, b| {
                b.score
                    .partial_cmp(&a.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // 限制最终数量
            ranked_suggestions.truncate(self.config.max_suggestions);
        }

        ranked_suggestions
    }

    /// 基于触发器生成建议
    ///
    /// 根据不同的触发事件，构建上下文并生成建议
    pub async fn suggest_on_trigger(&self, trigger: SuggestionTrigger) -> Vec<Suggestion> {
        // 从触发器构建上下文
        let context = self.build_context_from_trigger(trigger).await;

        // 生成建议
        self.suggest(&context).await
    }

    /// 从触发器构建建议上下文
    async fn build_context_from_trigger(&self, trigger: SuggestionTrigger) -> SuggestionContext {
        match trigger {
            SuggestionTrigger::DirectoryChange(dir) => {
                // 目录变化：使用新目录作为上下文
                SuggestionContext::new(dir)
            }

            SuggestionTrigger::CommandFailed {
                command,
                exit_code: _,
                error: _,
            } => {
                // 命令失败：标记失败状态，添加失败的命令
                let mut context = SuggestionContext::from_env();
                context.last_command_failed = true;
                context.recent_commands.push(command);
                context
            }

            SuggestionTrigger::CommandSuccess { command } => {
                // 命令成功：添加到最近命令
                let mut context = SuggestionContext::from_env();
                context.recent_commands.push(command);
                context
            }

            SuggestionTrigger::FileDetected(file_type) => {
                // 文件检测：设置项目类型
                let mut context = SuggestionContext::from_env();
                context.project_type = Some(file_type);
                context
            }

            SuggestionTrigger::Explicit | SuggestionTrigger::Startup => {
                // 显式请求或启动：使用当前环境
                SuggestionContext::from_env()
            }

            SuggestionTrigger::Idle(_duration) => {
                // 闲置：可以基于历史生成建议
                SuggestionContext::from_env()
            }
        }
    }

    /// 检查是否应该自动触发建议
    pub fn should_auto_trigger(&self, trigger: &SuggestionTrigger) -> bool {
        match trigger {
            // Explicit 总是触发（即使 auto_trigger 为 false）
            SuggestionTrigger::Explicit => true,

            // 其他事件根据配置决定
            _ if !self.config.auto_trigger => false,

            // 这些事件总是触发（当 auto_trigger 为 true 时）
            SuggestionTrigger::Startup => true,

            // 命令失败时触发
            SuggestionTrigger::CommandFailed { .. } => true,

            // 其他事件可配置
            SuggestionTrigger::DirectoryChange(_) => true,
            SuggestionTrigger::FileDetected(_) => true,

            // 闲置和命令成功默认不触发
            SuggestionTrigger::Idle(_) => false,
            SuggestionTrigger::CommandSuccess { .. } => false,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &SuggestionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::{ChatResponse, LlmError, Message};
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

    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let mut history = HistoryManager::new("test_suggestion_engine_history.json", 100);

        // 添加一些测试命令
        for _ in 0..5 {
            history.add("git status".to_string(), true);
        }
        for _ in 0..3 {
            history.add("cargo test".to_string(), true);
        }

        Arc::new(RwLock::new(history))
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let history = create_test_history();
        let config = SuggestionConfig::default();

        let engine = SuggestionEngine::new(history, config);

        assert!(engine.llm_suggester.is_none()); // LLM 默认未启用
    }

    #[tokio::test]
    async fn test_engine_with_llm() {
        let history = create_test_history();
        let config = SuggestionConfig::default();
        let mock_llm = Arc::new(MockLlmClient {
            response: "cargo build | Build project\n".to_string(),
        });

        let engine = SuggestionEngine::new(history, config).with_llm(mock_llm);

        assert!(engine.llm_suggester.is_some());
    }

    #[tokio::test]
    async fn test_suggest_basic() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.enable_llm = false; // 禁用 LLM 以简化测试

        let engine = SuggestionEngine::new(history, config);

        let context = SuggestionContext::from_env();
        let suggestions = engine.suggest(&context).await;

        // 应该有来自上下文和历史的建议
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_with_all_sources() {
        let history = create_test_history();
        let config = SuggestionConfig::default();
        let mock_llm = Arc::new(MockLlmClient {
            response: "docker ps | List containers\n".to_string(),
        });

        let engine = SuggestionEngine::new(history, config).with_llm(mock_llm);

        let context = SuggestionContext::from_env();
        let suggestions = engine.suggest(&context).await;

        // 应该有来自三个来源的建议
        assert!(!suggestions.is_empty());

        // 应该包含不同来源的建议
        // 注意：具体内容取决于当前项目类型
    }

    #[tokio::test]
    async fn test_suggest_on_trigger_command_failed() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.enable_llm = false;

        let engine = SuggestionEngine::new(history, config);

        let trigger = SuggestionTrigger::CommandFailed {
            command: "cargo build".to_string(),
            exit_code: 1,
            error: "compilation error".to_string(),
        };

        let suggestions = engine.suggest_on_trigger(trigger).await;

        // 应该有建议（如 cargo build --help）
        assert!(!suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_suggest_on_trigger_directory_change() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.enable_llm = false;

        let engine = SuggestionEngine::new(history, config);

        let trigger =
            SuggestionTrigger::DirectoryChange(std::env::current_dir().unwrap());

        let suggestions = engine.suggest_on_trigger(trigger).await;

        // 应该根据新目录生成建议
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_should_auto_trigger() {
        let history = create_test_history();
        let config = SuggestionConfig::default();
        let engine = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { SuggestionEngine::new(history, config) });

        // 显式请求总是触发
        assert!(engine.should_auto_trigger(&SuggestionTrigger::Explicit));

        // 命令失败时触发
        assert!(engine.should_auto_trigger(&SuggestionTrigger::CommandFailed {
            command: "test".to_string(),
            exit_code: 1,
            error: "error".to_string(),
        }));

        // 闲置默认不触发
        assert!(!engine.should_auto_trigger(&SuggestionTrigger::Idle(Duration::from_secs(5))));
    }

    #[test]
    fn test_config_disable_auto_trigger() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.auto_trigger = false;

        let engine = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { SuggestionEngine::new(history, config) });

        // 禁用自动触发后，除了 Explicit，其他都不应触发
        assert!(engine.should_auto_trigger(&SuggestionTrigger::Explicit));
        assert!(!engine.should_auto_trigger(&SuggestionTrigger::CommandFailed {
            command: "test".to_string(),
            exit_code: 1,
            error: "error".to_string(),
        }));
    }

    #[tokio::test]
    async fn test_max_suggestions_limit() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.max_suggestions = 3;
        config.enable_llm = false;

        let engine = SuggestionEngine::new(history, config);

        let context = SuggestionContext::from_env();
        let suggestions = engine.suggest(&context).await;

        // 不应超过配置的最大数量
        assert!(suggestions.len() <= 3);
    }

    #[tokio::test]
    async fn test_min_score_filter() {
        let history = create_test_history();
        let mut config = SuggestionConfig::default();
        config.min_score = 0.6; // 设置较高的阈值
        config.enable_llm = false;

        let engine = SuggestionEngine::new(history, config);

        let context = SuggestionContext::from_env();
        let suggestions = engine.suggest(&context).await;

        // 所有建议的分数应该 >= 0.6
        for suggestion in &suggestions {
            assert!(suggestion.score >= 0.6);
        }
    }
}
