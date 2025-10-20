//! Agent 核心逻辑
//!
//! 负责：
//! - 解析用户输入
//! - 路由到命令系统
//! - 智能命令路由（常见Shell命令自动识别） ✨ Phase 10.1
//! - 处理特殊前缀（!, /）
//! - Intent DSL 意图识别 ✨ Phase 3

use crate::command::CommandRegistry;
use crate::command_router::{CommandRouter, CommandType as RouterCommandType};
use crate::config::Config;
use crate::display::Display;
use crate::dsl::intent::{
    BuiltinIntents, CommandValidator, EntityExtractor, ExecutionPlan, IntentMatcher,
    IntentToPipeline, LlmToPipeline, TemplateEngine, ValidationResult,
};
use crate::execution_logger::{CommandType, ExecutionLogger};
use crate::history::HistoryManager;
use crate::llm_manager::LlmManager;
use crate::memory::{EntryType, Memory};
use crate::spinner::Spinner;
use crate::tool::ToolRegistry;
use crate::tool_executor::ToolExecutor;
use colored::Colorize;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

// ✨ Phase 8 Week 2: 多轮对话支持
use crate::conversation::{
    clear_current_conversation, get_current_conversation, has_active_conversation,
    set_current_conversation, ConversationManager, ParameterSpec, ParameterType, ParameterValue,
    Response,
};

// ✨ Phase 9: 统计与可视化支持
use crate::stats::{StatEvent, StatsCollector};

// ✨ Phase 9.1: 上下文追踪支持
use crate::memory::ContextTracker;

// ✨ Phase 9.2: 错误自动修复支持
use crate::error_fixer::{FeedbackLearner, FeedbackRecord, FeedbackType, FixOutcome};
use crate::shell_executor::ShellExecutorWithFixer;

// ✨ Phase 8 (Workflow): Workflow Intent 支持
use crate::dsl::intent::{WorkflowExecutor, WorkflowIntent};

/// Agent 核心
pub struct Agent {
    pub config: Config,
    pub registry: CommandRegistry,
    pub llm_manager: Arc<RwLock<LlmManager>>,
    pub memory: Arc<RwLock<Memory>>,
    pub exec_logger: Arc<RwLock<ExecutionLogger>>,
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    pub tool_executor: Arc<ToolExecutor>,
    // ✨ Intent DSL 支持 (Phase 3)
    pub intent_matcher: IntentMatcher,
    pub template_engine: TemplateEngine,
    // ✨ Pipeline DSL 支持 (Phase 6.3)
    pub pipeline_converter: IntentToPipeline,
    // ✨ LLM-driven Pipeline 支持 (Phase 7)
    pub llm_bridge: Option<Arc<LlmToPipeline>>,
    // ✨ Phase 8: 命令历史记录管理
    pub history: Arc<RwLock<HistoryManager>>,
    // ✨ Phase 8 Week 2: 多轮对话管理
    pub conversation_manager: Arc<RwLock<ConversationManager>>,
    // ✨ Phase 9: 统计收集器
    pub stats_collector: Arc<StatsCollector>,
    // ✨ Phase 9.1: 上下文追踪器
    pub context_tracker: Arc<RwLock<ContextTracker>>,
    // ✨ Phase 9.2: Shell执行器（带错误修复）
    pub shell_executor_with_fixer: Arc<ShellExecutorWithFixer>,
    // 最后失败的命令（用于/fix命令）
    pub last_failed_command: Arc<RwLock<Option<String>>>,
    // ✨ Phase 10.1: 智能命令路由器
    pub command_router: CommandRouter,
    // ✨ Phase 8 (Workflow): Workflow Intent 系统
    pub workflow_intents: Vec<WorkflowIntent>,
    pub workflow_executor: Option<Arc<WorkflowExecutor>>,
}

impl Agent {
    /// 规范化文件路径：
    /// - 将 ~ 展开为用户主目录
    /// - 将相对路径转换为基于用户数据目录的绝对路径
    /// - 绝对路径保持不变
    fn normalize_path(path: &str) -> String {
        use std::env;
        use std::path::PathBuf;

        let path = path.trim();

        // 处理 ~ 开头的路径
        if path.starts_with('~') {
            if let Some(home) = dirs::home_dir() {
                return path.replacen('~', &home.display().to_string(), 1);
            }
        }

        // 检查是否是绝对路径
        let path_buf = PathBuf::from(path);
        if path_buf.is_absolute() {
            return path.to_string();
        }

        // 相对路径：转换为用户数据目录下的路径
        // 使用 ~/.realconsole/ 作为基础目录
        if let Some(home) = dirs::home_dir() {
            let base_dir = home.join(".realconsole");
            return base_dir.join(path).display().to_string();
        }

        // 降级：如果无法获取主目录，尝试使用当前目录
        env::current_dir()
            .ok()
            .map(|d| d.join(path).display().to_string())
            .unwrap_or_else(|| path.to_string())
    }

    pub fn new(config: Config, registry: CommandRegistry) -> Self {
        // ✨ Phase 10.1: 初始化智能命令路由器
        let command_router = CommandRouter::new(config.prefix.clone());

        // ✨ Phase 8 (Workflow): 初始化 Workflow Intent 系统
        let (workflow_intents, workflow_executor) =
            if config.features.workflow_enabled.unwrap_or(false) {
                use crate::dsl::intent::register_builtin_workflows;
                let intents = register_builtin_workflows();
                (intents, None) // executor 在配置 LLM 后再初始化
            } else {
                (Vec::new(), None)
            };

        // 初始化记忆系统
        let memory_capacity = config
            .memory
            .as_ref()
            .and_then(|m| m.capacity)
            .unwrap_or(100);

        let memory = Memory::new(memory_capacity);

        // 如果配置了持久化文件，尝试加载历史记忆
        let memory = if let Some(ref mem_config) = config.memory {
            if let Some(ref path) = mem_config.persistent_file {
                // 规范化路径，避免跟随工作目录改变
                let normalized_path = Self::normalize_path(path);
                match Memory::load_from_file(&normalized_path, memory_capacity) {
                    Ok(loaded) => {
                        if !loaded.is_empty() {
                            // 说明：由于环形缓冲区的容量限制，只保留最近的 N 条记忆
                            Display::startup_memory(config.display.mode, loaded.len());
                        }
                        loaded
                    }
                    Err(e) => {
                        eprintln!("{} {}", "⚠ 记忆加载失败:".yellow(), e);
                        memory
                    }
                }
            } else {
                memory
            }
        } else {
            memory
        };

        // 初始化执行日志系统
        let exec_logger = ExecutionLogger::new(1000);

        // ✨ Phase 8: 初始化命令历史记录管理器
        let history = HistoryManager::default();

        // ✨ Phase 8 Week 2: 初始化多轮对话管理器
        let conversation_manager = ConversationManager::new(300); // 5分钟超时

        // ✨ Phase 9: 初始化统计收集器
        let stats_collector = Arc::new(StatsCollector::new());

        // ✨ Phase 9.1: 初始化上下文追踪器
        let context_tracker = ContextTracker::new();

        // 初始化工具注册表并注册内置工具
        let mut tool_registry = ToolRegistry::new();
        crate::builtin_tools::register_builtin_tools(&mut tool_registry);
        // ✨ Phase 5: 注册高级工具（HTTP、JSON、文本、系统信息）
        crate::advanced_tools::register_advanced_tools(&mut tool_registry);
        let tool_registry = Arc::new(RwLock::new(tool_registry));

        // 初始化工具执行引擎（使用配置值）
        let tool_executor = ToolExecutor::new(
            Arc::clone(&tool_registry),
            config.features.max_tool_iterations,
            config.features.max_tools_per_round,
        );

        // 初始化 Intent DSL 系统（使用内置意图库）
        let builtin = BuiltinIntents::new();
        let intent_matcher = builtin.create_matcher();
        let template_engine = builtin.create_engine();

        // ✨ Phase 6.3: 初始化 Pipeline DSL 转换器
        let pipeline_converter = IntentToPipeline::new();

        // ✨ Phase 7: LLM Bridge 初始化为 None，在配置 LLM 后再设置
        // 这个在 main.rs 中调用 configure_llm() 后会被设置
        let llm_bridge = None;

        // ✨ Phase 9.2: 初始化错误修复系统
        let feedback_learner = Arc::new(FeedbackLearner::new());
        // 如果配置了持久化路径，设置存储路径
        if let Some(ref config_dir) = dirs::config_dir() {
            let storage_path = config_dir.join("realconsole").join("feedback.json");
            let learner_with_storage = FeedbackLearner::new().with_storage(storage_path);

            // 在测试环境中跳过磁盘加载以避免阻塞问题
            #[cfg(not(test))]
            {
                // 尝试从磁盘加载历史反馈
                let _ = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { learner_with_storage.load_from_disk().await })
                });
            }

            let feedback_learner = Arc::new(learner_with_storage);

            let shell_executor_with_fixer =
                Arc::new(ShellExecutorWithFixer::new().with_feedback_learner(feedback_learner));

            let last_failed_command = Arc::new(RwLock::new(None));

            return Self {
                config,
                registry,
                llm_manager: Arc::new(RwLock::new(LlmManager::new())),
                memory: Arc::new(RwLock::new(memory)),
                exec_logger: Arc::new(RwLock::new(exec_logger)),
                tool_registry,
                tool_executor: Arc::new(tool_executor),
                intent_matcher,
                template_engine,
                pipeline_converter,
                llm_bridge,
                history: Arc::new(RwLock::new(history)),
                conversation_manager: Arc::new(RwLock::new(conversation_manager)),
                stats_collector,
                context_tracker: Arc::new(RwLock::new(context_tracker)),
                shell_executor_with_fixer,
                last_failed_command,
                command_router,
                workflow_intents: workflow_intents.clone(),
                workflow_executor: workflow_executor.clone(),
            };
        }

        // Fallback: 无持久化
        let shell_executor_with_fixer =
            Arc::new(ShellExecutorWithFixer::new().with_feedback_learner(feedback_learner));
        let last_failed_command = Arc::new(RwLock::new(None));

        Self {
            config,
            registry,
            llm_manager: Arc::new(RwLock::new(LlmManager::new())),
            memory: Arc::new(RwLock::new(memory)),
            exec_logger: Arc::new(RwLock::new(exec_logger)),
            tool_registry,
            tool_executor: Arc::new(tool_executor),
            intent_matcher,
            template_engine,
            pipeline_converter,
            llm_bridge,
            history: Arc::new(RwLock::new(history)),
            conversation_manager: Arc::new(RwLock::new(conversation_manager)),
            stats_collector,
            context_tracker: Arc::new(RwLock::new(context_tracker)),
            shell_executor_with_fixer,
            last_failed_command,
            command_router,
            workflow_intents,
            workflow_executor,
        }
    }

    /// 获取 LLM 管理器的引用
    pub fn llm_manager(&self) -> Arc<RwLock<LlmManager>> {
        Arc::clone(&self.llm_manager)
    }

    /// 获取记忆系统的引用
    pub fn memory(&self) -> Arc<RwLock<Memory>> {
        Arc::clone(&self.memory)
    }

    /// 获取执行日志系统的引用
    pub fn exec_logger(&self) -> Arc<RwLock<ExecutionLogger>> {
        Arc::clone(&self.exec_logger)
    }

    /// 获取工具注册表的引用
    pub fn tool_registry(&self) -> Arc<RwLock<ToolRegistry>> {
        Arc::clone(&self.tool_registry)
    }

    /// 获取历史记录管理器的引用
    pub fn history(&self) -> Arc<RwLock<HistoryManager>> {
        Arc::clone(&self.history)
    }

    /// 获取对话管理器的引用
    pub fn conversation_manager(&self) -> Arc<RwLock<ConversationManager>> {
        Arc::clone(&self.conversation_manager)
    }

    /// 获取统计收集器的引用
    pub fn stats_collector(&self) -> Arc<StatsCollector> {
        Arc::clone(&self.stats_collector)
    }

    /// 获取上下文追踪器的引用
    pub fn context_tracker(&self) -> Arc<RwLock<ContextTracker>> {
        Arc::clone(&self.context_tracker)
    }

    /// 配置 LLM Bridge（Phase 7）
    ///
    /// 在配置 LLM 客户端后调用，初始化 LLM 驱动的 Pipeline 生成器
    pub fn configure_llm_bridge(&mut self) {
        // 只在启用 LLM 生成且有 LLM 客户端时初始化
        if !self.config.intent.llm_generation_enabled.unwrap_or(false) {
            return;
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;
                if let Some(llm) = manager.primary().or(manager.fallback()) {
                    self.llm_bridge = Some(Arc::new(LlmToPipeline::new(llm.clone())));
                    Display::startup_llm_pipeline(self.config.display.mode);
                }
            })
        });
    }

    /// 配置 Workflow Executor（Phase 8）
    ///
    /// 在配置 LLM 客户端后调用，初始化 Workflow 执行器
    pub fn configure_workflow_executor(&mut self) {
        // 只在启用 Workflow 时初始化
        if !self.config.features.workflow_enabled.unwrap_or(false) {
            return;
        }

        // 创建 Workflow 执行器
        let executor = WorkflowExecutor::new(
            Arc::clone(&self.tool_registry),
            Some(Arc::clone(&self.llm_manager)),
        );

        self.workflow_executor = Some(Arc::new(executor));

        // 显示启动信息
        Display::startup_workflow(self.config.display.mode, self.workflow_intents.len());
    }

    /// 处理用户输入
    pub fn handle(&self, line: &str) -> String {
        let line = line.trim();

        if line.is_empty() {
            return String::new();
        }

        // 开始计时
        let start = Instant::now();

        // 记录用户输入
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut memory = self.memory.write().await;
                memory.add(line.to_string(), EntryType::User);
            })
        });

        // ✨ Phase 9.1: 提取实体并更新上下文追踪器
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut tracker = self.context_tracker.write().await;
                let entities = tracker.extract_entities(line);
                for entity in entities {
                    tracker.record_entity(entity);
                }
            })
        });

        // 特殊处理：exit 命令直接退出
        if line.trim().to_lowercase() == "exit" {
            return "__QUIT__".to_string();
        }

        // ✨ Phase 10.1: 使用智能命令路由器识别命令类型
        let router_result = self.command_router.route(line);

        let (command_type, response) = match router_result {
            RouterCommandType::CommonShell(cmd) => {
                // 常见Shell命令，直接执行
                (CommandType::Shell, self.handle_shell(&cmd))
            }
            RouterCommandType::ForcedShell(cmd) => {
                // 强制Shell执行（!前缀）
                (CommandType::Shell, self.handle_shell(&cmd))
            }
            RouterCommandType::SystemCommand(cmd_name, arg) => {
                // 系统命令（/前缀）
                let input = if arg.is_empty() {
                    cmd_name
                } else {
                    format!("{} {}", cmd_name, arg)
                };
                (CommandType::Command, self.handle_command(&input))
            }
            RouterCommandType::NaturalLanguage(text) => {
                // 自然语言，交给LLM处理
                (CommandType::Text, self.handle_text(&text))
            }
        };

        // 计算耗时
        let duration = start.elapsed();

        // 记录响应和执行日志
        if !response.is_empty() {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // 判断是否成功（简单检测：不包含错误关键词）
                    let success = !response.contains("错误")
                        && !response.contains("失败")
                        && !response.to_lowercase().contains("error")
                        && !response.to_lowercase().contains("failed");

                    // 记录到执行日志
                    {
                        let mut logger = self.exec_logger.write().await;
                        logger.log(line.to_string(), command_type, success, duration, &response);
                    }

                    // ✨ Phase 8: 记录到命令历史
                    {
                        let mut history = self.history.write().await;
                        history.add(line, success);
                    }

                    // ✨ Phase 9: 记录到统计收集器
                    {
                        self.stats_collector
                            .record(StatEvent::CommandExecution {
                                command: line.to_string(),
                                success,
                                duration,
                            })
                            .await;
                    }

                    // 记录到记忆
                    {
                        let mut memory = self.memory.write().await;
                        // 简化响应内容（最多保存前200个字符，考虑 UTF-8 边界）
                        let content = if response.len() > 200 {
                            // 找到安全的截断位置（UTF-8 字符边界）
                            let mut cutoff = 200.min(response.len());
                            while cutoff > 0 && !response.is_char_boundary(cutoff) {
                                cutoff -= 1;
                            }
                            format!("{}...", &response[..cutoff])
                        } else {
                            response.clone()
                        };
                        memory.add(content, EntryType::Assistant);

                        // 如果启用了自动保存，追加到文件
                        if let Some(ref mem_config) = self.config.memory {
                            if mem_config.auto_save.unwrap_or(false) {
                                if let Some(ref path) = mem_config.persistent_file {
                                    // 规范化路径，避免跟随工作目录改变
                                    let normalized_path = Self::normalize_path(path);
                                    let entries = memory.recent(1);
                                    if let Some(entry) = entries.first() {
                                        let _ = Memory::append_to_file(&normalized_path, entry);
                                    }
                                }
                            }
                        }
                    }

                    // ✨ Phase 9.1: 更新工作上下文（如果命令成功）
                    if success {
                        let mut tracker = self.context_tracker.write().await;
                        use crate::memory::WorkingContextUpdate;

                        // 更新当前目录
                        if let Ok(current_dir) = std::env::current_dir() {
                            tracker.update_working_context(WorkingContextUpdate::CurrentDirectory(
                                current_dir,
                            ));
                        }

                        // 更新最后执行的命令
                        tracker.update_working_context(WorkingContextUpdate::LastCommand(
                            line.to_string(),
                        ));
                    }
                })
            });
        }

        response
    }

    /// 处理 Shell 命令
    /// ✨ Phase 9.2: 集成错误自动修复系统
    fn handle_shell(&self, cmd: &str) -> String {
        if !self.config.features.shell_enabled {
            return format!("{}", "Shell 执行已禁用".red());
        }

        // 特殊处理：cd 命令需要在主进程中生效
        let cmd_trimmed = cmd.trim();
        if cmd_trimmed.starts_with("cd ") || cmd_trimmed == "cd" {
            return self.handle_cd_command(cmd_trimmed);
        }

        // ✨ Phase 9.2: 使用 ShellExecutorWithFixer 执行命令（带错误分析）
        let execution_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.shell_executor_with_fixer
                    .execute_with_analysis(cmd)
                    .await
            })
        });

        // 如果执行失败且有修复策略，保存失败的命令并显示交互式修复流程
        if !execution_result.success && !execution_result.fix_strategies.is_empty() {
            // 保存失败的命令（用于 /fix 命令）
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut last_cmd = self.last_failed_command.write().await;
                    *last_cmd = Some(cmd.to_string());
                })
            });

            // 显示交互式修复建议
            self.display_fix_suggestions(&execution_result)
        } else {
            // 正常输出或没有修复建议的错误
            execution_result.output
        }
    }

    /// 处理 cd 命令（在主进程中改变目录）
    fn handle_cd_command(&self, cmd: &str) -> String {
        use std::env;
        use std::path::Path;

        // 解析目标目录
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let target = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            // cd 无参数，进入 HOME 目录
            match env::var("HOME") {
                Ok(home) => home,
                Err(_) => return format!("{}", "无法获取 HOME 环境变量".red()),
            }
        };

        // 展开 ~ 为 HOME 目录
        let target = if target.starts_with('~') {
            match env::var("HOME") {
                Ok(home) => target.replacen('~', &home, 1),
                Err(_) => return format!("{}", "无法获取 HOME 环境变量".red()),
            }
        } else {
            target
        };

        // 改变目录
        match env::set_current_dir(Path::new(&target)) {
            Ok(_) => {
                // 成功，获取新的绝对路径
                match env::current_dir() {
                    Ok(new_dir) => format!("{}", new_dir.display().to_string().dimmed()),
                    Err(_) => format!("{}", "✓ 目录已切换".green()),
                }
            }
            Err(e) => format!("{} {}", "切换目录失败:".red(), e),
        }
    }

    /// ✨ Phase 9.2: 显示交互式修复建议
    ///
    /// 展示错误分析和修复建议，允许用户选择并执行修复策略
    fn display_fix_suggestions(&self, result: &crate::shell_executor::ExecutionResult) -> String {
        let mut output = String::new();

        // 1. 显示原始错误输出
        output.push_str(&format!(
            "\n{}\n{}\n",
            "❌ 命令执行失败".red().bold(),
            result.output
        ));

        // 2. 显示错误分析（如果有）
        if let Some(analysis) = &result.error_analysis {
            output.push_str(&format!("\n{}\n", "🔍 错误分析".cyan().bold()));
            output.push_str(&format!(
                "  {}: {}\n",
                "类别".dimmed(),
                analysis.category.to_string().yellow()
            ));

            // 显示严重程度
            let severity_str = match analysis.severity {
                crate::error_fixer::ErrorSeverity::Low => "低",
                crate::error_fixer::ErrorSeverity::Medium => "中",
                crate::error_fixer::ErrorSeverity::High => "高",
                crate::error_fixer::ErrorSeverity::Critical => "严重",
            };
            output.push_str(&format!(
                "  {}: {}\n",
                "严重程度".dimmed(),
                severity_str.red()
            ));

            if !analysis.possible_causes.is_empty() {
                output.push_str(&format!("\n  {}:\n", "可能原因".dimmed()));
                for cause in &analysis.possible_causes {
                    output.push_str(&format!("    • {}\n", cause));
                }
            }

            if !analysis.suggested_fixes.is_empty() {
                output.push_str(&format!("\n  {}:\n", "建议修复".dimmed()));
                for fix in &analysis.suggested_fixes {
                    output.push_str(&format!("    • {}\n", fix));
                }
            }
        }

        // 3. 显示修复策略列表
        if result.fix_strategies.is_empty() {
            output.push_str(&format!("\n{}\n", "暂无自动修复策略".yellow()));
            return output;
        }

        output.push_str(&format!(
            "\n{}\n",
            "💡 修复策略 (按推荐度排序)".green().bold()
        ));

        for (i, strategy) in result.fix_strategies.iter().enumerate() {
            // 风险指示器: 🟢 低 < 5, 🟡 中 5-7, 🔴 高 >= 8
            let risk_indicator = match strategy.risk_level {
                r if r < 5 => "🟢",
                r if r < 8 => "🟡",
                _ => "🔴",
            };

            output.push_str(&format!(
                "\n  {}. {} {} (风险: {}/10)\n",
                (i + 1).to_string().cyan().bold(),
                risk_indicator,
                strategy.description.bold(),
                strategy.risk_level
            ));

            // 显示策略名称和命令
            output.push_str(&format!(
                "     {}: {}\n",
                "策略".dimmed(),
                strategy.name.cyan()
            ));
            output.push_str(&format!(
                "     {}: {}\n",
                "修复命令".dimmed(),
                strategy.command.green()
            ));
            output.push_str(&format!(
                "     {}: {}\n",
                "预期效果".dimmed(),
                strategy.expected_outcome.dimmed()
            ));
        }

        // 4. 提示用户选择
        output.push_str(&format!("\n{}\n", "请选择:".yellow().bold()));
        output.push_str(&format!("  • {} - 选择对应编号执行修复\n", "1-N".cyan()));
        output.push_str(&format!("  • {} - 跳过，不执行修复\n", "s/skip".dimmed()));
        output.push_str(&format!("  • {} - 取消\n", "c/cancel".dimmed()));

        print!("\n{} ", "您的选择:".yellow());
        let _ = io::stdout().flush();

        // 5. 读取用户输入
        let mut user_input = String::new();
        if io::stdin().read_line(&mut user_input).is_err() {
            return format!("{}", "\n读取输入失败".red());
        }

        let choice = user_input.trim().to_lowercase();

        // 6. 处理用户选择
        if choice == "s" || choice == "skip" {
            return format!("{}\n{}", output, "✓ 已跳过修复".yellow());
        }

        if choice == "c" || choice == "cancel" {
            return format!("{}\n{}", output, "✓ 已取消".yellow());
        }

        // 解析数字选择
        let selected_index: usize = match choice.parse::<usize>() {
            Ok(n) if n > 0 && n <= result.fix_strategies.len() => n - 1,
            _ => {
                return format!("{}\n{}", output, format!("❌ 无效选择: {}", choice).red());
            }
        };

        let selected_strategy = &result.fix_strategies[selected_index];

        // 7. 执行选中的修复策略
        output.push_str(&format!(
            "\n{} {}\n",
            "🔧 执行修复:".cyan().bold(),
            selected_strategy.command.green()
        ));

        let fix_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                crate::shell_executor::execute_shell(&selected_strategy.command).await
            })
        });

        let (success, fix_output) = match fix_result {
            Ok(out) => (true, out),
            Err(e) => (false, e.format_user_friendly()),
        };

        output.push_str(&format!(
            "\n{}\n{}\n",
            if success {
                "✓ 修复执行成功".green().bold()
            } else {
                "✗ 修复执行失败".red().bold()
            },
            fix_output
        ));

        // 8. 记录反馈
        self.record_fix_feedback(result, selected_index, success);

        output
    }

    /// 记录修复反馈（用于学习）
    fn record_fix_feedback(
        &self,
        result: &crate::shell_executor::ExecutionResult,
        strategy_index: usize,
        success: bool,
    ) {
        if let Some(error_analysis) = &result.error_analysis {
            if strategy_index < result.fix_strategies.len() {
                let strategy = &result.fix_strategies[strategy_index];

                let feedback_type = if success {
                    FeedbackType::Accepted
                } else {
                    FeedbackType::Rejected
                };
                let outcome = if success {
                    FixOutcome::Success
                } else {
                    FixOutcome::Failure
                };

                // 创建反馈记录
                let record = FeedbackRecord::new(error_analysis, strategy, feedback_type, outcome);

                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = self
                            .shell_executor_with_fixer
                            .feedback_learner()
                            .record_feedback(record)
                            .await;
                    })
                });
            }
        }
    }

    /// 处理命令
    /// ✨ Phase 9.2: 添加 /fix 命令支持
    fn handle_command(&self, input: &str) -> String {
        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd_name = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        // ✨ Phase 9.2: 特殊处理 /fix 命令
        if cmd_name == "fix" {
            return self.handle_fix_command();
        }

        match self.registry.execute(cmd_name, arg) {
            Ok(output) => output,
            Err(err) => format!("{}", err.red()),
        }
    }

    /// ✨ Phase 9.2: 处理 /fix 命令 - 重试上次失败的命令
    fn handle_fix_command(&self) -> String {
        let last_cmd = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let cmd_guard = self.last_failed_command.read().await;
                cmd_guard.clone()
            })
        });

        match last_cmd {
            Some(cmd) => {
                println!("{} {}", "🔄 重试命令:".cyan().bold(), cmd.cyan());
                self.handle_shell(&cmd)
            }
            None => {
                format!(
                    "{}\n{}",
                    "❌ 没有可重试的失败命令".red(),
                    "提示: 执行一个失败的命令后再使用 /fix".dimmed()
                )
            }
        }
    }

    /// 处理自由文本（Intent 识别 → LLM 对话）
    fn handle_text(&self, text: &str) -> String {
        // ✨ Phase 8 Week 2: 优先检查多轮对话（一分为三：对话态、意图态、LLM态）
        // 1️⃣ 对话态：如果有活跃对话，继续对话流程
        if has_active_conversation() {
            return self.handle_conversation_input(text);
        }

        // 2️⃣ 检测是否需要启动新对话（特定意图需要参数收集）
        if let Some(response) = self.try_start_conversation(text) {
            return response;
        }

        // 🔧 优先使用 LLM 工具调用（如果启用且可用）
        let use_tools = self.config.features.tool_calling_enabled.unwrap_or(false);

        if use_tools {
            // 使用 LLM 工具调用模式（更智能，支持 count_code_lines 等工具）
            return self.handle_text_with_tools(text);
        }

        // ✨ Phase 8: 尝试匹配 Workflow Intent（套路化复用）
        // 优先于传统 Intent，因为 Workflow 性能更优
        if let Some(response) = self.try_match_workflow(text) {
            return response;
        }

        // ✨ Phase 3: 回退到 Intent 识别（道法自然 - 先识别意图，未匹配则回退到流式LLM）
        if let Some(plan) = self.try_match_intent(text) {
            return self.execute_intent(&plan);
        }

        // 最后回退：使用传统流式输出模式
        self.handle_text_streaming(text)
    }

    /// 使用工具调用处理文本
    fn handle_text_with_tools(&self, text: &str) -> String {
        // 启动 spinner
        let spinner = Spinner::new();

        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // 获取 LLM 客户端
                let manager = self.llm_manager.read().await;
                let llm = manager
                    .primary()
                    .or(manager.fallback())
                    .ok_or_else(|| "未配置 LLM 客户端".to_string())?;

                // 获取工具 schemas
                let registry = self.tool_registry.read().await;
                let tool_schemas = registry.get_function_schemas();
                drop(registry); // 提前释放锁

                // 如果没有工具，回退到普通对话
                if tool_schemas.is_empty() {
                    let response: Result<String, String> =
                        manager.chat(text).await.map_err(|e| e.to_string());
                    return response;
                }

                // 使用工具执行引擎
                self.tool_executor
                    .execute_iterative(llm.as_ref(), text, tool_schemas)
                    .await
            })
        }) {
            Ok(response) => {
                // 停止 spinner
                spinner.stop();

                // ✨ 解析并显示对话轮次调试信息（成功时，仅当配置启用且为 debug 模式）
                if self.config.display.show_conversation_rounds
                    && self.config.display.mode.show_debug()
                {
                    if let Some(rounds) =
                        crate::tool_executor::ToolExecutor::decode_debug_info(&response)
                    {
                        let round_infos: Vec<crate::display::ConversationRoundInfo> = rounds
                            .iter()
                            .map(|r| crate::display::ConversationRoundInfo {
                                round: r.round,
                                input_summary: r.input_summary.clone(),
                                assistant_response: r.assistant_response.clone(),
                                tool_calls: r
                                    .tool_calls
                                    .iter()
                                    .map(|tc| crate::display::ToolCallInfo {
                                        name: tc.name.clone(),
                                        arguments: tc.arguments.clone(),
                                    })
                                    .collect(),
                                tool_results: r.tool_results.clone(),
                                message_count: r.message_count,
                                duration_ms: r.duration_ms,
                            })
                            .collect();

                        Display::conversation_rounds_debug(self.config.display.mode, &round_infos);
                    }
                }

                // 提取实际响应内容（移除调试信息）
                if let Some(pos) = response.find("__DEBUG__") {
                    response[..pos].to_string()
                } else {
                    response
                }
            }
            Err(e) => {
                // 停止 spinner
                spinner.stop();

                // ✨ 解析并显示对话轮次调试信息（仅 debug 模式）
                if let Some(rounds) = crate::tool_executor::ToolExecutor::decode_debug_info(&e) {
                    let round_infos: Vec<crate::display::ConversationRoundInfo> = rounds
                        .iter()
                        .map(|r| crate::display::ConversationRoundInfo {
                            round: r.round,
                            input_summary: r.input_summary.clone(),
                            assistant_response: r.assistant_response.clone(),
                            tool_calls: r
                                .tool_calls
                                .iter()
                                .map(|tc| crate::display::ToolCallInfo {
                                    name: tc.name.clone(),
                                    arguments: tc.arguments.clone(),
                                })
                                .collect(),
                            tool_results: r.tool_results.clone(),
                            message_count: r.message_count,
                            duration_ms: r.duration_ms,
                        })
                        .collect();

                    Display::conversation_rounds_debug(self.config.display.mode, &round_infos);
                }

                // 提取错误主消息（移除调试信息）
                let error_msg = if let Some(pos) = e.find("__DEBUG__") {
                    &e[..pos]
                } else {
                    &e
                };

                // 解析上下文长度错误
                if let Some((requested, limit)) = parse_context_length_error(error_msg) {
                    Display::context_overflow_error(self.config.display.mode, requested, limit);
                    return format!(
                        "\n{} 使用 {}help 查看帮助",
                        "提示:".dimmed(),
                        self.config.prefix.dimmed()
                    );
                }

                // 其他错误
                format!(
                    "{} {}\n{} {}help",
                    "处理失败:".red(),
                    error_msg,
                    "提示: 使用".dimmed(),
                    self.config.prefix.dimmed()
                )
            }
        }
    }

    /// 使用流式输出处理文本（传统模式）
    fn handle_text_streaming(&self, text: &str) -> String {
        // 不显示 "AI:" 前缀，让输出更接近普通 console
        // 显示 spinner 等待 LLM 响应

        // 开始计时
        let start = Instant::now();

        // 启动 spinner
        let spinner = Spinner::new();

        // 使用 block_in_place 在同步上下文中调用异步代码
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;
                // 使用流式输出，实时显示每个 token
                manager
                    .chat_stream(text, |token| {
                        print!("{}", token);
                        let _ = io::stdout().flush();
                    })
                    .await
            })
        }) {
            Ok(_response) => {
                // 停止 spinner
                spinner.stop();

                // 计算耗时
                let elapsed = start.elapsed();

                // 流式输出已经完成，不需要额外换行
                // 因为流式输出已经包含了完整的输出内容
                Display::execution_timing(self.config.display.mode, elapsed.as_secs_f64());

                // 返回空字符串，因为内容已通过流式输出显示
                String::new()
            }
            Err(e) => {
                // 停止 spinner
                spinner.stop();

                // LLM 调用失败，显示友好的错误信息
                format!(
                    "\n{} {}\n{} {}help",
                    "LLM 调用失败:".red(),
                    e,
                    "提示: 使用".dimmed(),
                    self.config.prefix.dimmed()
                )
            }
        }
    }

    // ========== Intent DSL 支持方法 (Phase 3) ==========

    /// 尝试匹配用户输入到意图
    ///
    /// 使用 IntentMatcher 查找最佳匹配的意图，如果匹配成功且置信度足够，
    /// 则使用 TemplateEngine 生成执行计划。
    ///
    /// Phase 2 & 3 增强：
    /// - 支持 LLM 智能参数提取
    /// - 支持 LLM 命令验证
    ///
    /// Phase 6.3 增强：
    /// - 优先使用 Pipeline DSL 生成命令（如果支持）
    /// - 回退到传统模板引擎
    ///
    /// Phase 7 增强：
    /// - 优先使用 LLM 驱动的 Pipeline 生成（如果启用）
    /// - Fallback 到规则匹配
    ///
    /// # 返回
    /// - `Some(ExecutionPlan)`: 匹配成功，返回可执行计划
    /// - `None`: 没有匹配的意图，应回退到 LLM 处理
    fn try_match_intent(&self, text: &str) -> Option<ExecutionPlan> {
        // 0. Phase 7: 优先尝试 LLM 驱动的 Pipeline 生成（如果启用）
        if self.config.intent.llm_generation_enabled.unwrap_or(false) {
            if let Some(llm_bridge) = &self.llm_bridge {
                match tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { llm_bridge.understand_and_generate(text).await })
                }) {
                    Ok(pipeline_plan) => {
                        // LLM 成功生成 ExecutionPlan
                        let command = pipeline_plan.to_shell_command();

                        Display::llm_generation(self.config.display.mode);

                        return Some(ExecutionPlan {
                            command,
                            template_name: "llm_generated".to_string(),
                            bindings: std::collections::HashMap::new(),
                        });
                    }
                    Err(e) => {
                        // LLM 失败，根据配置决定是否 fallback
                        if self.config.intent.llm_generation_fallback.unwrap_or(true) {
                            Display::fallback_warning(self.config.display.mode, &e);
                        } else {
                            Display::error(
                                self.config.display.mode,
                                &format!("LLM 生成失败: {}", e),
                            );
                            return None;
                        }
                    }
                }
            }
        }

        // 1. 使用 IntentMatcher 匹配最佳意图
        let mut intent_match = self.intent_matcher.best_match(text)?;

        // 2. Phase 2: 使用 LLM 智能补充参数提取（如果启用）
        if self.config.intent.llm_extraction_enabled {
            intent_match = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { self.try_llm_extraction(text, intent_match).await })
            });
        }

        // 3. Phase 6.3: 优先尝试使用 Pipeline DSL 生成执行计划
        let plan = if let Some(pipeline_plan) = self
            .pipeline_converter
            .convert(&intent_match, &intent_match.extracted_entities)
        {
            // Pipeline DSL 成功生成 ExecutionPlan
            // 将 Pipeline ExecutionPlan 转换为 Template ExecutionPlan
            let command = pipeline_plan.to_shell_command();

            // 将实体转换为字符串绑定
            let mut bindings = std::collections::HashMap::new();
            for (key, entity) in &intent_match.extracted_entities {
                let value = match entity {
                    crate::dsl::intent::EntityType::Path(p) => p.clone(),
                    crate::dsl::intent::EntityType::FileType(ft) => ft.clone(),
                    crate::dsl::intent::EntityType::Number(n) => n.to_string(),
                    crate::dsl::intent::EntityType::Custom(_, v) => v.clone(),
                    crate::dsl::intent::EntityType::Operation(op) => op.clone(),
                    crate::dsl::intent::EntityType::Date(d) => d.clone(),
                };
                bindings.insert(key.clone(), value);
            }

            ExecutionPlan {
                command,
                template_name: intent_match.intent.name.clone(),
                bindings,
            }
        } else {
            // 回退到传统模板引擎
            match self.template_engine.generate_from_intent(&intent_match) {
                Ok(plan) => plan,
                Err(e) => {
                    // 生成执行计划失败，记录错误但不中断流程
                    eprintln!("{} {}", "⚠ 执行计划生成失败:".yellow(), e);
                    return None;
                }
            }
        };

        // 显示意图识别结果
        Display::intent_match(
            self.config.display.mode,
            &intent_match.intent.name,
            intent_match.confidence,
        );

        // 4. Phase 3: 使用 LLM 验证命令（如果启用）
        if self.config.intent.llm_validation_enabled {
            let intent_name = intent_match.intent.name.clone();
            let validation = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { self.try_llm_validation(text, &plan, &intent_name).await })
            });

            // 如果验证失败或置信度低，警告用户
            if let Some(validation) = validation {
                if validation.should_warn(self.config.intent.validation_threshold) {
                    self.display_validation_warning(&validation);

                    // 如果需要用户确认
                    if self.config.intent.require_confirmation && !self.ask_user_confirmation() {
                        return None; // 用户拒绝执行
                    }
                }
            }
        }

        Some(plan)
    }

    /// Phase 8: 尝试匹配 Workflow Intent
    ///
    /// 使用 Workflow Intent 系统匹配用户输入，如果匹配成功则执行工作流
    ///
    /// # 返回
    /// - `Some(String)`: 匹配成功并执行，返回执行结果
    /// - `None`: 没有匹配的工作流，应回退到传统 Intent 或 LLM
    fn try_match_workflow(&self, text: &str) -> Option<String> {
        // 如果 Workflow 未启用，直接返回 None
        if !self.config.features.workflow_enabled.unwrap_or(false) {
            return None;
        }

        // 如果没有 executor，返回 None
        let executor = self.workflow_executor.as_ref()?;

        // 遍历所有 workflow intents，找到最佳匹配
        let mut best_match: Option<(usize, crate::dsl::intent::IntentMatch)> = None;
        let mut best_confidence = 0.0;

        for (idx, workflow_intent) in self.workflow_intents.iter().enumerate() {
            // 为每个 workflow 创建临时 matcher 并匹配
            let mut temp_matcher = IntentMatcher::new();
            temp_matcher.register(workflow_intent.base_intent.clone());

            if let Some(intent_match) = temp_matcher.best_match(text) {
                if intent_match.confidence > best_confidence {
                    best_confidence = intent_match.confidence;
                    best_match = Some((idx, intent_match));
                }
            }
        }

        // 如果没有找到匹配，返回 None
        let (workflow_idx, intent_match) = best_match?;
        let workflow = &self.workflow_intents[workflow_idx];

        // 显示 workflow 匹配信息
        Display::workflow_match(
            self.config.display.mode,
            &workflow.base_intent.name,
            intent_match.confidence,
        );

        // 执行 workflow
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { executor.execute(workflow, &intent_match).await })
        }) {
            Ok(result) => {
                // 显示 workflow 执行统计（包含缓存状态）
                let from_cache = result.duration_ms < 100; // 简单判断：< 100ms 可能是缓存
                Display::workflow_stats(
                    self.config.display.mode,
                    result.duration_ms,
                    result.llm_calls,
                    result.tool_calls,
                    from_cache,
                );

                // 返回执行结果
                Some(result.output)
            }
            Err(e) => {
                // Workflow 执行失败，返回 None 以回退到传统流程
                eprintln!("{} {}", "⚠ Workflow 执行失败:".yellow(), e);
                None
            }
        }
    }

    /// Phase 2: 尝试使用 LLM 补充提取实体
    async fn try_llm_extraction(
        &self,
        text: &str,
        mut intent_match: crate::dsl::intent::types::IntentMatch,
    ) -> crate::dsl::intent::types::IntentMatch {
        // 检查是否有缺失的实体
        let expected_count = intent_match.intent.entities.len();
        let extracted_count = intent_match.extracted_entities.len();

        if extracted_count < expected_count {
            // 有缺失实体，使用 LLM 补充
            let manager = self.llm_manager.read().await;
            if let Some(llm) = manager.primary().or(manager.fallback()) {
                let extractor = EntityExtractor::new();
                match extractor
                    .extract_with_llm(text, &intent_match.intent.entities, llm.as_ref())
                    .await
                {
                    entities if !entities.is_empty() => {
                        Display::debug_info(self.config.display.mode, "LLM 参数提取成功");
                        intent_match.extracted_entities = entities;
                    }
                    _ => {}
                }
            }
        }

        intent_match
    }

    /// Phase 3: 尝试使用 LLM 验证命令
    async fn try_llm_validation(
        &self,
        text: &str,
        plan: &ExecutionPlan,
        intent_name: &str,
    ) -> Option<ValidationResult> {
        let manager = self.llm_manager.read().await;
        if let Some(llm) = manager.primary().or(manager.fallback()) {
            let validator = CommandValidator::new();
            match validator
                .validate(text, plan, intent_name, llm.as_ref())
                .await
            {
                Ok(result) => Some(result),
                Err(e) => {
                    eprintln!("{} {}", "⚠ LLM 验证失败:".yellow(), e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// 显示验证警告
    fn display_validation_warning(&self, validation: &ValidationResult) {
        println!("\n{}", "⚠️ 命令验证警告:".yellow().bold());
        println!("  {}: {:.2}", "置信度".dimmed(), validation.confidence);
        println!("  {}: {}", "原因".dimmed(), validation.reason);

        if !validation.suggestions.is_empty() {
            println!("\n  {}:", "建议".dimmed());
            for suggestion in &validation.suggestions {
                println!("    - {}", suggestion);
            }
        }
        println!();
    }

    /// 询问用户确认
    fn ask_user_confirmation(&self) -> bool {
        print!("是否继续执行? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let answer = input.trim().to_lowercase();
            matches!(answer.as_str(), "y" | "yes")
        } else {
            false
        }
    }

    /// 执行意图对应的命令
    ///
    /// 将 ExecutionPlan 中的命令作为 Shell 命令执行。
    ///
    /// # 设计原则（道法自然）
    /// - Intent DSL 生成的命令都是标准 Shell 命令
    /// - 直接复用现有的 shell_executor 基础设施
    /// - 不引入额外的复杂性
    fn execute_intent(&self, plan: &ExecutionPlan) -> String {
        // 显示将要执行的命令
        Display::command_execution(self.config.display.mode, &plan.command);

        // 使用 shell_executor 执行命令
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { crate::shell_executor::execute_shell(&plan.command).await })
        }) {
            Ok(output) => output,
            Err(e) => {
                // 使用用户友好的错误格式
                e.format_user_friendly()
            }
        }
    }

    // ========== 多轮对话支持方法 (Phase 8 Week 2) ==========

    /// 尝试启动多轮对话
    ///
    /// 检测用户输入是否匹配需要参数收集的意图，如果是则启动对话流程
    /// ✨ Phase 8 Week 2 增强：使用 LLM 智能提取参数
    fn try_start_conversation(&self, text: &str) -> Option<String> {
        // 检测特定关键词，判断是否需要启动对话
        let intent = self.detect_conversation_intent(text)?;

        // 启动对话
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut manager = self.conversation_manager.write().await;
                manager.start_conversation(&intent)
            })
        }) {
            Ok(conversation_id) => {
                // 设置当前对话
                set_current_conversation(Some(conversation_id.clone()));

                // 获取参数规格
                let params = self.get_parameter_specs_for_intent(&intent);

                // 添加参数到对话
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let mut manager = self.conversation_manager.write().await;
                        for param in params {
                            let _ = manager.add_parameter_spec(&conversation_id, param);
                        }
                    })
                });

                // ✨ 新增：尝试使用 LLM 从用户输入中提取参数
                let extracted_params = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let llm_manager = self.llm_manager.read().await;
                        if let Some(llm) = llm_manager.primary().or(llm_manager.fallback()) {
                            let mut manager = self.conversation_manager.write().await;
                            manager
                                .extract_parameters_with_llm(&conversation_id, text, llm.as_ref())
                                .await
                                .unwrap_or_default()
                        } else {
                            Vec::new()
                        }
                    })
                });

                // 自动收集提取到的参数
                for (param_name, param_value) in extracted_params {
                    let _ = tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let mut manager = self.conversation_manager.write().await;
                            manager.collect_parameter(&conversation_id, &param_name, param_value)
                        })
                    });
                }

                // 显示对话开始提示
                let mut response = format!(
                    "{} {}\n{}",
                    "▶".cyan().bold(),
                    "启动多轮对话".cyan(),
                    "输入 'cancel' 或 'exit' 可以随时取消对话".dimmed()
                );

                // 检查是否还有待收集的参数
                match tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let manager = self.conversation_manager.read().await;
                        manager.detect_missing_parameters(&conversation_id).ok()
                    })
                }) {
                    Some(missing) if !missing.is_empty() => {
                        // ✨ 使用 LLM 生成智能提问
                        let smart_question = tokio::task::block_in_place(|| {
                            tokio::runtime::Handle::current().block_on(async {
                                let llm_manager = self.llm_manager.read().await;
                                if let Some(llm) = llm_manager.primary().or(llm_manager.fallback())
                                {
                                    let manager = self.conversation_manager.read().await;
                                    manager
                                        .generate_smart_question(&conversation_id, llm.as_ref())
                                        .await
                                        .ok()
                                } else {
                                    None
                                }
                            })
                        });

                        if let Some(question) = smart_question {
                            response.push_str(&format!("\n{} {}", "❓".yellow(), question));
                        } else {
                            // 回退到标准提问
                            let next_param = &missing[0];
                            response.push_str(&format!(
                                "\n{} {}\n  {}\n{}\n{}",
                                "●".yellow(),
                                next_param.name.bold(),
                                next_param.description.dimmed(),
                                next_param
                                    .hint
                                    .as_ref()
                                    .map(|h| format!("  💡 {}", h.dimmed()))
                                    .unwrap_or_default(),
                                next_param
                                    .example
                                    .as_ref()
                                    .map(|e| format!("  📝 例如: {}", e.cyan()))
                                    .unwrap_or_default(),
                            ));
                        }
                    }
                    _ => {
                        // 没有缺失参数，准备执行
                        response.push_str("\n所有参数已收集完成，准备执行...");
                    }
                }

                Some(response)
            }
            Err(e) => Some(format!("{} {}", "对话启动失败:".red(), e)),
        }
    }

    /// 检测对话意图
    ///
    /// 根据关键词判断用户是否想要执行需要多轮对话的操作
    fn detect_conversation_intent(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        // 日志分析意图
        if text_lower.contains("分析日志") || text_lower.contains("查看日志") {
            return Some("analyze_logs".to_string());
        }

        // 文件操作意图
        if (text_lower.contains("删除")
            || text_lower.contains("移动")
            || text_lower.contains("复制"))
            && (text_lower.contains("文件") || text_lower.contains("目录"))
        {
            return Some("file_operation".to_string());
        }

        None
    }

    /// 获取意图对应的参数规格
    fn get_parameter_specs_for_intent(&self, intent: &str) -> Vec<ParameterSpec> {
        match intent {
            "analyze_logs" => vec![
                ParameterSpec::new("file_path", ParameterType::Path, "日志文件路径")
                    .with_hint("支持绝对路径或相对路径")
                    .with_example("/var/log/app.log"),
                ParameterSpec::new("keyword", ParameterType::String, "要搜索的关键词")
                    .with_hint("支持正则表达式")
                    .with_example("ERROR|WARN"),
                ParameterSpec::new("time_range", ParameterType::String, "时间范围（可选）")
                    .optional()
                    .with_hint("格式: YYYY-MM-DD 或 '最近24小时'")
                    .with_example("2025-01-15"),
            ],
            "file_operation" => vec![
                ParameterSpec::new("operation", ParameterType::String, "操作类型")
                    .with_hint("delete, move, copy")
                    .with_example("delete"),
                ParameterSpec::new("source", ParameterType::Path, "源文件/目录路径")
                    .with_example("/path/to/file.txt"),
                ParameterSpec::new(
                    "destination",
                    ParameterType::Path,
                    "目标路径（移动/复制时需要）",
                )
                .optional()
                .with_example("/path/to/dest/"),
            ],
            _ => vec![],
        }
    }

    /// 处理对话输入
    /// ✨ Phase 8 Week 2 增强：使用 LLM 智能参数收集和智能提问
    fn handle_conversation_input(&self, text: &str) -> String {
        // 检查是否是取消命令
        let text_lower = text.trim().to_lowercase();
        if text_lower == "cancel" || text_lower == "exit" || text_lower == "quit" {
            return self.cancel_current_conversation();
        }

        // 检查是否是确认命令（y/yes）
        if text_lower == "y" || text_lower == "yes" {
            return self.handle_conversation_confirmation(true);
        }

        // 检查是否是拒绝命令（n/no）
        if text_lower == "n" || text_lower == "no" {
            return self.handle_conversation_confirmation(false);
        }

        // 获取当前对话 ID
        let conversation_id: String = match get_current_conversation() {
            Some(id) => id,
            None => return "没有活跃的对话".to_string(),
        };

        // 获取当前待收集的参数
        let param_name: String = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.conversation_manager.read().await;
                let context = manager.get_context(&conversation_id).ok()?;
                context.next_pending_parameter().map(|p| p.name.clone())
            })
        }) {
            Some(name) => name,
            None => return "对话状态异常".red().to_string(),
        };

        // 解析参数值
        let param_value = self.parse_parameter_value(text, &param_name);

        // ✨ 使用智能参数收集（带 LLM 验证和智能提问）
        let use_smart_collection = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let llm_manager = self.llm_manager.read().await;
                llm_manager.primary().or(llm_manager.fallback()).is_some()
            })
        });

        if use_smart_collection {
            // 使用智能收集
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let llm_manager = self.llm_manager.read().await;
                    if let Some(llm) = llm_manager.primary().or(llm_manager.fallback()) {
                        let mut manager = self.conversation_manager.write().await;
                        manager
                            .collect_parameter_smart(
                                &conversation_id,
                                &param_name,
                                param_value,
                                llm.as_ref(),
                            )
                            .await
                    } else {
                        // 回退到普通收集
                        let mut manager = self.conversation_manager.write().await;
                        manager.collect_parameter(&conversation_id, &param_name, param_value)
                    }
                })
            }) {
                Ok(Response::AskForParameter {
                    name: _,
                    description,
                    ..
                }) => {
                    // 继续询问下一个参数（description 已包含 LLM 生成的智能提问）
                    format!("{} 已记录\n{} {}", "✓".green(), "❓".yellow(), description)
                }
                Ok(Response::AllParametersCollected) => {
                    // 所有参数收集完成，询问确认
                    self.confirm_conversation_execution(&conversation_id)
                }
                Ok(Response::ReadyToExecute) => {
                    // 执行对话意图
                    self.execute_conversation(&conversation_id)
                }
                Ok(Response::ExecutionResult { success, output }) => {
                    // 清理对话
                    clear_current_conversation();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let mut manager = self.conversation_manager.write().await;
                            manager.cleanup_completed();
                        })
                    });

                    if success {
                        format!("{}\n{}", "✓ 执行成功".green().bold(), output)
                    } else {
                        format!("{}\n{}", "✗ 执行失败".red().bold(), output)
                    }
                }
                Ok(Response::Cancelled) => {
                    clear_current_conversation();
                    "对话已取消".yellow().to_string()
                }
                Err(e) => format!("{} {}", "参数收集失败:".red(), e),
            }
        } else {
            // 回退到普通收集
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut manager = self.conversation_manager.write().await;
                    manager.collect_parameter(&conversation_id, &param_name, param_value)
                })
            }) {
                Ok(Response::AskForParameter {
                    name,
                    description,
                    hint,
                    default,
                }) => {
                    // 继续询问下一个参数
                    format!(
                        "{} 已记录\n{} {}\n  {}\n{}\n{}",
                        "✓".green(),
                        "●".yellow(),
                        name.bold(),
                        description.dimmed(),
                        hint.map(|h| format!("  💡 {}", h.dimmed()))
                            .unwrap_or_default(),
                        default
                            .map(|d| format!("  🔹 默认值: {:?}", d))
                            .unwrap_or_default(),
                    )
                }
                Ok(Response::AllParametersCollected) => {
                    // 所有参数收集完成，询问确认
                    self.confirm_conversation_execution(&conversation_id)
                }
                Ok(Response::ReadyToExecute) => {
                    // 执行对话意图
                    self.execute_conversation(&conversation_id)
                }
                Ok(Response::ExecutionResult { success, output }) => {
                    // 清理对话
                    clear_current_conversation();
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let mut manager = self.conversation_manager.write().await;
                            manager.cleanup_completed();
                        })
                    });

                    if success {
                        format!("{}\n{}", "✓ 执行成功".green().bold(), output)
                    } else {
                        format!("{}\n{}", "✗ 执行失败".red().bold(), output)
                    }
                }
                Ok(Response::Cancelled) => {
                    clear_current_conversation();
                    "对话已取消".yellow().to_string()
                }
                Err(e) => format!("{} {}", "参数收集失败:".red(), e),
            }
        }
    }

    /// 处理对话确认
    fn handle_conversation_confirmation(&self, confirmed: bool) -> String {
        let conversation_id = match get_current_conversation() {
            Some(id) => id,
            None => return "没有活跃的对话".to_string(),
        };

        if confirmed {
            // 用户确认，执行对话
            self.execute_conversation(&conversation_id)
        } else {
            // 用户拒绝，取消对话
            self.cancel_current_conversation()
        }
    }

    /// 解析参数值
    fn parse_parameter_value(&self, text: &str, _param_name: &str) -> ParameterValue {
        // 简单实现：统一解析为字符串
        // TODO: 根据参数类型智能解析
        ParameterValue::String(text.to_string())
    }

    /// 确认对话执行
    fn confirm_conversation_execution(&self, conversation_id: &str) -> String {
        // 获取已收集的参数
        let params = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.conversation_manager.read().await;
                let context = manager.get_context(conversation_id).ok()?;
                Some(context.parameters.clone())
            })
        }) {
            Some(p) => p,
            None => return "无法获取对话上下文".red().to_string(),
        };

        // 显示参数摘要
        let mut summary = String::from("\n📋 参数摘要:\n");
        for (name, value) in &params {
            summary.push_str(&format!("  {} = {:?}\n", name.cyan(), value));
        }

        format!(
            "{}\n{}\n{}",
            summary,
            "确认执行？[y/N]:".yellow().bold(),
            "输入 y 确认，其他键取消".dimmed()
        )
    }

    /// 执行对话
    fn execute_conversation(&self, conversation_id: &str) -> String {
        // 获取意图和参数
        let (intent, params): (String, std::collections::HashMap<String, ParameterValue>) =
            match tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let manager = self.conversation_manager.read().await;
                    let context = manager.get_context(conversation_id).ok()?;
                    Some((context.intent.clone(), context.parameters.clone()))
                })
            }) {
                Some(data) => data,
                None => return "无法获取对话上下文".red().to_string(),
            };

        // 根据意图构建命令
        let command = self.build_command_from_conversation(&intent, &params);

        // 执行命令
        let result = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { crate::shell_executor::execute_shell(&command).await })
        }) {
            Ok(output) => (true, output),
            Err(e) => (false, e.format_user_friendly()),
        };

        // 记录执行结果
        let _ = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut manager = self.conversation_manager.write().await;
                manager.complete_execution(conversation_id, result.0, result.1.clone())
            })
        });

        if result.0 {
            format!("{}\n{}", "✓ 执行成功".green().bold(), result.1)
        } else {
            format!("{}\n{}", "✗ 执行失败".red().bold(), result.1)
        }
    }

    /// 从对话构建命令
    fn build_command_from_conversation(
        &self,
        intent: &str,
        params: &std::collections::HashMap<String, ParameterValue>,
    ) -> String {
        match intent {
            "analyze_logs" => {
                let file_path = params
                    .get("file_path")
                    .and_then(|v| {
                        if let ParameterValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");
                let keyword = params
                    .get("keyword")
                    .and_then(|v| {
                        if let ParameterValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");

                format!("grep -i '{}' {} | tail -50", keyword, file_path)
            }
            "file_operation" => {
                let operation = params
                    .get("operation")
                    .and_then(|v| {
                        if let ParameterValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("ls");
                let source = params
                    .get("source")
                    .and_then(|v| {
                        if let ParameterValue::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");

                match operation {
                    "delete" => format!("rm -i {}", source),
                    "move" => {
                        let dest = params
                            .get("destination")
                            .and_then(|v| {
                                if let ParameterValue::String(s) = v {
                                    Some(s.as_str())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or("");
                        format!("mv {} {}", source, dest)
                    }
                    "copy" => {
                        let dest = params
                            .get("destination")
                            .and_then(|v| {
                                if let ParameterValue::String(s) = v {
                                    Some(s.as_str())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or("");
                        format!("cp {} {}", source, dest)
                    }
                    _ => format!("ls -l {}", source),
                }
            }
            _ => "echo 'Unknown intent'".to_string(),
        }
    }

    /// 取消当前对话
    fn cancel_current_conversation(&self) -> String {
        if let Some(conversation_id) = get_current_conversation() {
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let mut manager = self.conversation_manager.write().await;
                    manager.cancel_conversation(&conversation_id, "用户取消")
                })
            });

            clear_current_conversation();
            format!("{} 对话已取消", "✓".yellow())
        } else {
            "没有活跃的对话".to_string()
        }
    }
}

/// 解析上下文长度错误，提取请求的 tokens 和限制
///
/// 示例错误信息：
/// "This model's maximum context length is 131072 tokens. However, you requested 133770 tokens"
fn parse_context_length_error(error_msg: &str) -> Option<(usize, usize)> {
    // 尝试匹配 "requested X tokens" 和 "maximum context length is Y tokens"
    let requested_pattern = regex::Regex::new(r"requested (\d+) tokens").ok()?;
    let limit_pattern = regex::Regex::new(r"maximum context length is (\d+) tokens").ok()?;

    let requested = requested_pattern
        .captures(error_msg)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<usize>().ok())?;

    let limit = limit_pattern
        .captures(error_msg)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<usize>().ok())?;

    Some((requested, limit))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    #[test]
    fn test_parse_context_length_error() {
        let error_msg = "This model's maximum context length is 131072 tokens. However, you requested 133770 tokens (133770 in the messages, 0 in the completion).";
        let result = parse_context_length_error(error_msg);
        assert_eq!(result, Some((133770, 131072)));

        let invalid_msg = "Some other error";
        let result = parse_context_length_error(invalid_msg);
        assert_eq!(result, None);
    }

    fn test_handler(_arg: &str) -> String {
        "test output".to_string()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_command_handling() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();
        registry.register(Command::from_fn("test", "Test", test_handler));

        let agent = Agent::new(config, registry);
        let result = agent.handle("/test");
        assert_eq!(result, "test output");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_empty_input() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        assert_eq!(agent.handle(""), "");
        assert_eq!(agent.handle("  "), "");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_shell_command_enabled() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试基本 shell 命令
        let result = agent.handle("!echo 'test'");
        assert!(result.contains("test") || result.contains("Shell"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_shell_command_disabled() {
        let mut config = Config::default();
        config.features.shell_enabled = false;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // Shell 命令应该被禁用
        let result = agent.handle("!echo 'test'");
        assert!(result.contains("禁用") || result.contains("disabled"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_system_command() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();

        // 注册一个测试命令
        registry.register(Command::from_fn("testcmd", "Test command", |_| {
            "command output".to_string()
        }));

        let agent = Agent::new(config, registry);

        // 测试系统命令（使用默认前缀 "/"）
        let result = agent.handle("/testcmd arg");
        assert_eq!(result, "command output");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_unknown_system_command() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试未知命令
        let result = agent.handle("/unknowncmd");
        // 应该返回错误信息（包含错误关键词）
        assert!(!result.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_memory_tracking() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 执行一个命令
        agent.handle("/nonexistent");

        // 检查记忆系统是否记录了输入
        let memory = agent.memory();
        let memory_guard = memory.read().await;
        assert!(!memory_guard.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_execution_logging() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();
        registry.register(Command::from_fn("test", "Test", |_| "ok".to_string()));

        let agent = Agent::new(config, registry);

        // 执行命令
        agent.handle("/test");

        // 检查执行日志
        let logger = agent.exec_logger();
        let logger_guard = logger.read().await;
        let stats = logger_guard.stats();

        assert_eq!(stats.total, 1);
    }

    // ========== handle_cd_command 测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_cd_to_tmp() {
        use std::env;

        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 保存当前目录
        let original_dir = env::current_dir().unwrap();

        // 测试 cd 到 /tmp
        let result = agent.handle("!cd /tmp");

        // 验证结果包含路径或成功消息
        assert!(!result.contains("失败") && !result.contains("错误"));

        // 验证目录确实改变了
        let current = env::current_dir().unwrap();
        assert!(current.to_string_lossy().contains("tmp"));

        // 恢复原始目录
        let _ = env::set_current_dir(&original_dir);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_cd_invalid_path() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试 cd 到不存在的目录
        let result = agent.handle("!cd /nonexistent_directory_12345");

        // 应该返回错误信息
        assert!(result.contains("失败") || result.contains("错误"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_cd_home() {
        use std::env;

        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 保存当前目录
        let original_dir = env::current_dir().unwrap();

        // 测试 cd 无参数（应该进入 HOME）
        let result = agent.handle("!cd");

        // 不应该包含错误
        assert!(!result.contains("失败"));

        // 恢复原始目录
        let _ = env::set_current_dir(&original_dir);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_cd_tilde_expansion() {
        use std::env;

        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 保存当前目录
        let original_dir = env::current_dir().unwrap();

        // 测试 cd ~/（应该展开为 HOME 目录）
        let result = agent.handle("!cd ~");

        // 不应该包含错误
        assert!(!result.contains("失败"));

        // 恢复原始目录
        let _ = env::set_current_dir(&original_dir);
    }

    // ========== handle_shell 危险命令测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_shell_dangerous_rm() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试危险的 rm -rf / 命令
        let result = agent.handle("!rm -rf /");

        // 应该被阻止
        assert!(result.contains("禁止") || result.contains("危险"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_shell_dangerous_sudo() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试 sudo 命令
        let result = agent.handle("!sudo whoami");

        // 应该被阻止
        assert!(result.contains("禁止") || result.contains("危险"));
    }

    // ========== handle_text 相关测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_text_without_llm() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试文本处理（没有配置 LLM）
        let result = agent.handle("你好");

        // 应该返回错误或提示信息
        assert!(!result.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_text_tool_calling_disabled() {
        let mut config = Config::default();
        config.features.tool_calling_enabled = Some(false);
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试工具调用被禁用的情况
        let result = agent.handle("计算 2+2");

        // 应该有响应（即使失败也应该有错误消息）
        assert!(!result.is_empty());
    }

    // ========== Intent DSL 测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_intent_matching_basic() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试基础意图匹配（列出文件）
        let result = agent.try_match_intent("列出所有rs文件");

        // 应该能够匹配到意图或返回 None
        // 这里我们只是测试不会 panic
        let _ = result;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_intent_matching_no_match() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试无法匹配的输入
        let result = agent.try_match_intent("这是一个随机的句子，不应该匹配任何意图");

        // 应该返回 None
        assert!(result.is_none());
    }

    // ========== 错误处理测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_command_with_error() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试执行不存在的命令
        let result = agent.handle("/nonexistent_command_xyz");

        // 应该返回非空的错误消息
        assert!(!result.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_long_response_truncation() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();

        // 注册一个返回很长响应的命令
        registry.register(Command::from_fn("longtest", "Long test", |_| {
            "x".repeat(300) // 超过 200 字符
        }));

        let agent = Agent::new(config, registry);

        // 执行命令
        agent.handle("/longtest");

        // 检查记忆系统中的内容是否被截断
        let memory = agent.memory();
        let memory_guard = memory.read().await;
        let recent = memory_guard.recent(1);

        // 最近的记忆应该被截断到 ~203 字符（200 + "..."）
        if let Some(entry) = recent.first() {
            assert!(entry.content.len() <= 210); // 留一些余地
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_multiple_commands_execution() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();
        registry.register(Command::from_fn("test1", "Test 1", |_| {
            "output1".to_string()
        }));
        registry.register(Command::from_fn("test2", "Test 2", |_| {
            "output2".to_string()
        }));

        let agent = Agent::new(config, registry);

        // 执行多个命令
        let result1 = agent.handle("/test1");
        let result2 = agent.handle("/test2");

        assert_eq!(result1, "output1");
        assert_eq!(result2, "output2");

        // 检查执行日志记录了两次
        let logger = agent.exec_logger();
        let logger_guard = logger.read().await;
        let stats = logger_guard.stats();

        assert_eq!(stats.total, 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tool_registry_access() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试工具注册表访问
        let tool_registry = agent.tool_registry();
        let registry_guard = tool_registry.read().await;

        // 应该有内置工具被注册
        assert!(!registry_guard.list_tools().is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_llm_manager_access() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试 LLM 管理器访问
        let llm_manager = agent.llm_manager();
        let manager_guard = llm_manager.read().await;

        // 默认情况下应该没有配置 LLM
        assert!(manager_guard.primary().is_none());
    }

    // ========== configure_llm_bridge 测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_configure_llm_bridge_disabled() {
        let mut config = Config::default();
        config.intent.llm_generation_enabled = Some(false);
        let registry = CommandRegistry::new();
        let mut agent = Agent::new(config, registry);

        // LLM 生成被禁用，bridge 应该保持 None
        agent.configure_llm_bridge();
        assert!(agent.llm_bridge.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_configure_llm_bridge_no_llm() {
        let mut config = Config::default();
        config.intent.llm_generation_enabled = Some(true);
        let registry = CommandRegistry::new();
        let mut agent = Agent::new(config, registry);

        // 没有配置 LLM 客户端，bridge 应该保持 None
        agent.configure_llm_bridge();
        assert!(agent.llm_bridge.is_none());
    }

    // ========== execute_intent 测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_intent_basic() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 创建一个简单的执行计划
        let plan = ExecutionPlan {
            command: "echo 'test'".to_string(),
            template_name: "test_template".to_string(),
            bindings: std::collections::HashMap::new(),
        };

        // 执行 Intent
        let result = agent.execute_intent(&plan);

        // 应该包含执行结果或命令
        assert!(!result.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_intent_with_error() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 创建一个会失败的执行计划（不存在的命令）
        let plan = ExecutionPlan {
            command: "nonexistent_command_xyz_123".to_string(),
            template_name: "test_template".to_string(),
            bindings: std::collections::HashMap::new(),
        };

        // 执行 Intent
        let result = agent.execute_intent(&plan);

        // 应该包含错误信息
        assert!(!result.is_empty());
        // 可能包含 "not found" 或类似的错误消息
    }

    // ========== handle_text 路径测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_text_with_tools_no_tools() {
        let mut config = Config::default();
        config.features.tool_calling_enabled = Some(true);
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试工具调用模式（但没有配置 LLM）
        let result = agent.handle("测试文本");

        // 应该返回错误或提示
        assert!(!result.is_empty());
    }

    // ========== 边界情况测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_with_only_whitespace() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试只包含空格和 Tab 的输入
        assert_eq!(agent.handle("   \t  \n  "), "");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_command_with_args() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();

        // 注册一个接收参数的命令
        registry.register(Command::from_fn("echo_arg", "Echo argument", |arg| {
            format!("arg: {}", arg)
        }));

        let agent = Agent::new(config, registry);

        // 测试带参数的命令
        let result = agent.handle("/echo_arg hello world");
        assert_eq!(result, "arg: hello world");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_command_no_args() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();

        // 注册一个不需要参数的命令
        registry.register(Command::from_fn("noarg", "No argument command", |arg| {
            if arg.is_empty() {
                "no args".to_string()
            } else {
                format!("got: {}", arg)
            }
        }));

        let agent = Agent::new(config, registry);

        // 测试不带参数的命令
        let result = agent.handle("/noarg");
        assert_eq!(result, "no args");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cd_with_trailing_slash() {
        use std::env;

        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 保存当前目录
        let original_dir = env::current_dir().unwrap();

        // 测试 cd 带尾部斜杠
        let result = agent.handle("!cd /tmp/");

        // 不应该包含错误
        assert!(!result.contains("失败"));

        // 恢复原始目录
        let _ = env::set_current_dir(&original_dir);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_persistence_config() {
        let mut config = Config::default();
        // 不配置持久化文件
        config.memory = None;

        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 执行一个命令
        agent.handle("/help");

        // 记忆应该正常工作（即使没有持久化）
        let memory = agent.memory();
        let memory_guard = memory.read().await;
        assert!(!memory_guard.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_success_detection_in_logging() {
        let config = Config::default();
        let mut registry = CommandRegistry::new();

        // 注册成功和失败的命令
        registry.register(Command::from_fn("success_cmd", "Success", |_| {
            "操作成功完成".to_string()
        }));
        registry.register(Command::from_fn("error_cmd", "Error", |_| {
            "错误: 操作失败".to_string()
        }));

        let agent = Agent::new(config, registry);

        // 执行成功命令
        agent.handle("/success_cmd");

        // 执行失败命令
        agent.handle("/error_cmd");

        // 检查执行日志统计
        let logger = agent.exec_logger();
        let logger_guard = logger.read().await;
        let stats = logger_guard.stats();

        assert_eq!(stats.total, 2);
        assert!(stats.success >= 1);
        assert!(stats.failed >= 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_shell_with_output_limit() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 测试生成大量输出的命令
        let result = agent.handle("!echo 'line1'; echo 'line2'; echo 'line3'");

        // 应该有输出（可能被限制）
        assert!(!result.is_empty());
    }

    // ========== Phase 8: Workflow Intent 兼容性测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_disabled_by_default() {
        // 验证默认配置下 workflow 是禁用的
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config.clone(), registry);

        // 验证配置默认值
        assert_eq!(config.features.workflow_enabled, Some(false));

        // 验证 workflow 相关字段为空
        assert!(agent.workflow_intents.is_empty());
        assert!(agent.workflow_executor.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_disabled_no_impact() {
        // 验证 workflow 禁用时对现有功能无影响
        let mut config = Config::default();
        config.features.workflow_enabled = Some(false); // 显式禁用

        let mut registry = CommandRegistry::new();
        registry.register(Command::from_fn("test", "Test", |_| {
            "test output".to_string()
        }));

        let agent = Agent::new(config, registry);

        // 测试系统命令正常工作
        let result = agent.handle("/test");
        assert_eq!(result, "test output");

        // 测试 handle_text 不会调用 workflow
        // （由于没有 LLM 配置，会返回错误，但不应该因为 workflow 而崩溃）
        let result = agent.handle("随机文本");
        assert!(!result.is_empty()); // 应该有响应（即使是错误）
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_enabled_initializes_templates() {
        // 验证启用 workflow 时正确初始化模板
        let mut config = Config::default();
        config.features.workflow_enabled = Some(true); // 启用

        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 验证 workflow 模板已加载
        assert!(!agent.workflow_intents.is_empty());
        assert!(agent.workflow_intents.len() >= 4); // 至少有 4 个内置模板

        // 验证模板名称
        let template_names: Vec<String> = agent
            .workflow_intents
            .iter()
            .map(|w| w.base_intent.name.clone())
            .collect();

        assert!(template_names.contains(&"crypto_analysis".to_string()));
        assert!(template_names.contains(&"stock_analysis".to_string()));
        assert!(template_names.contains(&"weather_analysis".to_string()));
        assert!(template_names.contains(&"website_summary".to_string()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_try_match_returns_none_when_disabled() {
        // 验证 workflow 禁用时 try_match_workflow 返回 None
        let mut config = Config::default();
        config.features.workflow_enabled = Some(false);

        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 调用 try_match_workflow 应该立即返回 None
        let result = agent.try_match_workflow("分析 BNB 的走势");
        assert!(result.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_backward_compatible_config() {
        // 测试旧配置文件（没有 workflow 字段）可以正常解析
        let yaml = r#"
prefix: "/"
features:
  shell_enabled: true
  shell_timeout: 10
"#;
        let config: Config = serde_yml::from_str(yaml).unwrap();

        // 验证新字段有默认值
        assert_eq!(config.features.workflow_enabled, Some(false));
        assert_eq!(config.features.workflow_cache_enabled, Some(true));
        assert_eq!(config.features.workflow_cache_ttl_default, Some(300));

        // 创建 Agent 应该成功
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // workflow 应该是禁用状态
        assert!(agent.workflow_intents.is_empty());
        assert!(agent.workflow_executor.is_none());
    }
}
