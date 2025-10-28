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
    /// 4. ✨ v1.9.4: 根据学习阶段调整建议策略
    /// 5. ✨ 通过离增强器优化（炼化）
    pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut all_suggestions = Vec::new();

        // ✨ v1.9.4: 根据学习阶段调整建议权重
        let (weight_context, weight_history, min_score_threshold, max_count) =
            self.get_phase_adjustments(context);

        // 1. 上下文建议（如果启用）
        if self.config.enable_context {
            let mut context_suggestions = self.context_suggester.suggest(context).await;
            // ✨ 应用学习阶段权重
            for suggestion in &mut context_suggestions {
                suggestion.score *= weight_context;
            }
            all_suggestions.extend(context_suggestions);
        }

        // 2. 历史建议（如果启用）
        if self.config.enable_history {
            let mut history_suggestions = self.history_suggester.suggest(context).await;
            // ✨ 应用学习阶段权重
            for suggestion in &mut history_suggestions {
                suggestion.score *= weight_history;
            }
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

        // ✨ v1.9.4: 应用学习阶段的分数阈值过滤
        ranked_suggestions.retain(|s| s.score >= min_score_threshold);

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

            // ✨ v1.9.4: 根据学习阶段限制最终数量
            ranked_suggestions.truncate(max_count);
        }

        ranked_suggestions
    }

    /// ✨ v1.9.4: 根据学习阶段获取调整参数
    ///
    /// 返回：(context_weight, history_weight, min_score_threshold, max_count)
    fn get_phase_adjustments(&self, context: &SuggestionContext) -> (f64, f64, f64, usize) {
        match context.learning_phase.as_deref() {
            // 探索期：鼓励多样性，降低阈值
            Some("Exploration") => {
                (
                    1.2,  // 提升上下文建议权重（探索新命令）
                    0.8,  // 降低历史建议权重（避免重复）
                    0.3,  // 降低分数阈值（允许更多建议）
                    self.config.max_suggestions + 2, // 增加建议数量
                )
            }
            // 稳定期：优先精准性，提高阈值
            Some("Stability") => {
                (
                    0.8,  // 降低上下文建议权重
                    1.2,  // 提升历史建议权重（熟悉的命令）
                    0.6,  // 提高分数阈值（只保留高质量）
                    self.config.max_suggestions.saturating_sub(1).max(3), // 减少建议数量
                )
            }
            // 转变期或未知：使用默认配置
            _ => {
                (
                    1.0,  // 默认上下文权重
                    1.0,  // 默认历史权重
                    self.config.min_score, // 使用配置的最小分数
                    self.config.max_suggestions, // 使用配置的最大数量
                )
            }
        }
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

    /// ✨ v1.8.4 Phase 3: 从八卦记忆宫离维度加载知识
    ///
    /// 读取离维度的知识条目，并解析为建议优化规则
    /// 这些知识由炼化炉的离阶段生成
    pub async fn load_knowledge_from_li(
        &self,
        palace: &crate::bagua::BaguaMemoryPalace,
    ) -> anyhow::Result<usize> {
        use crate::bagua::dimension::BaguaDimension;
        use crate::bagua::entry::MemoryContent;

        // 从离维度读取最近的知识
        let knowledge_entries = palace.retrieve(BaguaDimension::Li, Some(100)).await?;

        let mut loaded_count = 0;

        // 解析知识并应用到 LiEnhancer
        for entry in knowledge_entries {
            if let MemoryContent::Knowledge { fact, confidence, .. } = &entry.content {
                // 解析知识字符串并提取信息
                // 格式示例：
                // "命令 'cargo build' 被频繁使用（15次，置信度85%），应优先推荐"
                // "命令序列 'cargo build' → 'cargo run' 常一起执行（10次，置信度78%）"
                // "错误模式 'type mismatch' 通常用 'cargo check' 修复（成功率90%）"

                if Self::apply_knowledge_to_enhancer(&self.li_enhancer, fact, *confidence).await {
                    loaded_count += 1;
                }
            }
        }

        Ok(loaded_count)
    }

    /// 将单条知识应用到离增强器
    ///
    /// 解析知识字符串并转换为增强器能理解的形式
    async fn apply_knowledge_to_enhancer(
        li_enhancer: &Arc<RwLock<LiEnhancer>>,
        knowledge: &str,
        confidence: f64,
    ) -> bool {
        use crate::likan::types::Pattern;

        // 解析知识字符串
        // 1. 频率模式："命令 'X' 被频繁使用"
        if let Some(command) = Self::extract_frequent_command(knowledge) {
            let pattern = Pattern::Frequency {
                command: command.to_string(),
                count: 10, // 默认计数
                confidence,
            };

            let mut li = li_enhancer.write().await;
            li.update_patterns(vec![pattern]);
            return true;
        }

        // 2. 序列模式："命令序列 'X' → 'Y'"
        if let Some((cmd1, cmd2)) = Self::extract_sequence_pattern(knowledge) {
            let pattern = Pattern::Sequence {
                commands: vec![cmd1.to_string(), cmd2.to_string()],
                occurrences: 5, // 默认出现次数
                confidence,
            };

            let mut li = li_enhancer.write().await;
            li.update_patterns(vec![pattern]);
            return true;
        }

        // 3. 错误修复模式："错误模式 'X' 通常用 'Y' 修复"
        if let Some((error_pattern, fix_cmd)) = Self::extract_error_fix_pattern(knowledge) {
            let pattern = Pattern::ErrorFix {
                error_pattern: error_pattern.to_string(),
                fix_command: fix_cmd.to_string(),
                success_rate: confidence,
            };

            let mut li = li_enhancer.write().await;
            li.update_patterns(vec![pattern]);
            return true;
        }

        false
    }

    /// 提取频繁命令："命令 'cargo build' 被频繁使用"
    fn extract_frequent_command(knowledge: &str) -> Option<&str> {
        if knowledge.contains("被频繁使用") || knowledge.contains("应优先推荐") {
            // 提取单引号中的命令
            Self::extract_quoted_text(knowledge)
        } else {
            None
        }
    }

    /// 提取序列模式："命令序列 'cargo build' → 'cargo run' 常一起执行"
    fn extract_sequence_pattern(knowledge: &str) -> Option<(&str, &str)> {
        if knowledge.contains("命令序列") && knowledge.contains("→") {
            // 提取两个单引号中的命令
            let parts: Vec<&str> = knowledge.split('\'').collect();
            if parts.len() >= 4 {
                return Some((parts[1], parts[3]));
            }
        }
        None
    }

    /// 提取错误修复模式："错误模式 'type mismatch' 通常用 'cargo check' 修复"
    fn extract_error_fix_pattern(knowledge: &str) -> Option<(&str, &str)> {
        if knowledge.contains("错误模式") && knowledge.contains("修复") {
            let parts: Vec<&str> = knowledge.split('\'').collect();
            if parts.len() >= 4 {
                return Some((parts[1], parts[3]));
            }
        }
        None
    }

    /// 从单引号中提取文本
    fn extract_quoted_text(text: &str) -> Option<&str> {
        let parts: Vec<&str> = text.split('\'').collect();
        if parts.len() >= 2 {
            Some(parts[1])
        } else {
            None
        }
    }

    /// ✨ v1.8.4 Phase 3: 周期性从离维度刷新知识
    ///
    /// 返回新增的知识数量
    pub async fn refresh_knowledge_from_bagua(
        &self,
        palace: &crate::bagua::BaguaMemoryPalace,
    ) -> anyhow::Result<usize> {
        self.load_knowledge_from_li(palace).await
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
