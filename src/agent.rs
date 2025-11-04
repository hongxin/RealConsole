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
use crate::llm::Message;
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
    set_current_conversation, ContextManager, ConversationManager, ParameterSpec, ParameterType,
    ParameterValue, Response, Turn,
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

// ✨ v1.25.0: Markdown 渲染支持
use crate::markdown_renderer::MarkdownRenderer;

// ✨ Phase 2 (v1.3.0): 服务层架构
use crate::services::{
    IntentRequest, IntentService, LlmRequest, LlmService, Service, ShellService, StateManager,
};

// ✨ 语音播报系统
use crate::voice::{BroadcastConfig, VoiceBroadcaster};

// ✨ v1.5.1: 追踪上下文系统
use crate::trace_context::{ExecutionSpan, SpanType, TraceContext, TraceStore};

// ✨ Phase 4.1: 主动建议系统
use crate::suggestion::{
    Suggestion, SuggestionCache, SuggestionConfig, SuggestionContext, SuggestionEngine,
};

// ✨ Phase 4.3: 离坎炼化炉（自主学习循环）
use crate::likan::{FurnaceConfig, FurnaceStatus, LiKanFurnace, LiKanStatusBar, LiKanTrigger};

// ✨ v1.8.4: 八卦记忆宫（多维记忆系统）
use crate::bagua::BaguaMemoryPalace;

/// Agent 核心
///
/// ✨ Phase 2 (v1.3.0): 服务层架构重构
/// - 新增服务层字段（state_manager, intent_service, llm_service, shell_service）
/// - 保留原有字段以维持向后兼容
/// - 逐步迁移方法实现到服务调用
pub struct Agent {
    // === 核心配置 ===
    pub config: Config,
    pub registry: CommandRegistry,

    // === 服务层（v1.3.0 新增）===
    state_manager: Arc<StateManager>,
    intent_service: Arc<IntentService>,
    llm_service: Arc<LlmService>,
    shell_service: Arc<ShellService>,

    // === 运行时状态 ===
    /// 运行时系统提示词（可通过 /set-prompt 动态修改）
    runtime_system_prompt: Arc<RwLock<Option<String>>>,

    // === 原有字段（保留，向后兼容）===
    pub llm_manager: Arc<RwLock<LlmManager>>,

    /// ⚠️ **v2.0.0 将改为 private** - 请使用 `state_manager().memory()` 代替
    pub memory: Arc<RwLock<Memory>>,

    /// ⚠️ **v2.0.0 将改为 private** - 请使用 `state_manager().exec_logger()` 代替
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

    /// ⚠️ **v2.0.0 将改为 private** - 请使用 `state_manager().history()` 代替
    // ✨ Phase 8: 命令历史记录管理
    pub history: Arc<RwLock<HistoryManager>>,

    // ✨ Phase 8 Week 2: 多轮对话管理
    pub conversation_manager: Arc<RwLock<ConversationManager>>,

    /// ⚠️ **v2.0.0 将改为 private** - 请使用 `state_manager().stats_collector()` 代替
    // ✨ Phase 9: 统计收集器
    pub stats_collector: Arc<StatsCollector>,

    /// ⚠️ **v2.0.0 将改为 private** - 请使用 `state_manager().context_tracker()` 代替
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
    // ✨ LLM 交互日志系统
    llm_logger: Option<Arc<crate::llm::LlmLogger>>,
    // ✨ 语音播报系统
    pub voice_broadcaster: Option<Arc<VoiceBroadcaster>>,
    // ✨ v1.5.1: 追踪上下文存储（保留最近1000个trace）
    pub trace_store: Arc<TraceStore>,
    // ✨ Phase 4.1: 主动建议系统（三源融合）
    pub suggestion_engine: Option<Arc<SuggestionEngine>>,
    // ✨ Phase 4.2 P1: 建议缓存（带过期机制）
    pub last_suggestions: Arc<RwLock<SuggestionCache>>,
    // ✨ Phase 4.3: 离坎炼化炉（自主学习循环）
    pub likan_furnace: Option<Arc<RwLock<LiKanFurnace>>>,
    // ✨ Phase 4.3: 炼化炉后台任务句柄
    likan_task_handle: Option<tokio::task::JoinHandle<()>>,
    // ✨ Phase 4.3: 炼化炉状态栏（底部状态显示）
    pub likan_statusbar: Option<Arc<LiKanStatusBar>>,
    // ✨ Phase 4.3: 炼化炉手动触发器（用于 /likan cycle 命令）
    pub likan_trigger: Option<Arc<LiKanTrigger>>,
    // ✨ v1.8.4: 八卦记忆宫（多维记忆系统）
    pub bagua_palace: Option<Arc<RwLock<BaguaMemoryPalace>>>,
    // ✨ v1.9.0: 两仪状态追踪器（时间维度演化）
    pub state_tracker: Option<Arc<crate::liangyyi::StateTracker>>,
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

        // ✨ v1.5.1: 初始化追踪存储（保留最近1000个trace）
        let trace_store = Arc::new(TraceStore::new(1000));

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

        // 预先创建 Arc 包装的组件（在分支之前）
        let llm_manager = Arc::new(RwLock::new(LlmManager::new()));
        let memory_arc = Arc::new(RwLock::new(memory));
        let exec_logger_arc = Arc::new(RwLock::new(exec_logger));
        let history_arc = Arc::new(RwLock::new(history));
        let conversation_manager_arc = Arc::new(RwLock::new(conversation_manager));
        let context_tracker_arc = Arc::new(RwLock::new(context_tracker));
        let tool_executor_arc = Arc::new(tool_executor);

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

            // 初始化对话上下文管理器
            let conversation_context = Arc::new(RwLock::new(ContextManager::new(
                config.conversation.clone(),
            )));

            // ✨ Phase 2 (v1.3.0): 初始化服务层
            let state_manager = Arc::new(StateManager::new(
                Arc::clone(&memory_arc),
                Arc::clone(&history_arc),
                Arc::clone(&context_tracker_arc),
                Arc::clone(&stats_collector),
                Arc::clone(&exec_logger_arc),
                Arc::clone(&conversation_context),
            ));

            let intent_service = Arc::new(IntentService::new(
                intent_matcher.clone(),
                template_engine.clone(),
                pipeline_converter.clone(),
                llm_bridge.clone(),
                workflow_intents.clone(),
                workflow_executor.clone(),
                Arc::clone(&llm_manager),
            ));

            // 创建运行时系统提示词
            let runtime_system_prompt = Arc::new(RwLock::new(None));

            let llm_service = Arc::new(LlmService::new(
                Arc::clone(&llm_manager),
                Arc::clone(&tool_registry),
                Arc::clone(&tool_executor_arc),
                config.llm.system_prompt.clone(),
                Arc::clone(&runtime_system_prompt),
            ));

            let shell_service = Arc::new(ShellService::new(Arc::clone(&shell_executor_with_fixer)));

            // ✨ 初始化 LLM 日志系统
            let llm_logger = Self::create_llm_logger(&config);

            // ✨ 初始化语音播报系统
            let voice_broadcaster = Self::create_voice_broadcaster(&config);

            return Self {
                // 核心配置
                config,
                registry,
                // 服务层
                state_manager,
                intent_service,
                llm_service,
                shell_service,
                // 运行时状态
                runtime_system_prompt: Arc::new(RwLock::new(None)),
                // 原有字段
                llm_manager: Arc::clone(&llm_manager),
                memory: Arc::clone(&memory_arc),
                exec_logger: Arc::clone(&exec_logger_arc),
                tool_registry,
                tool_executor: Arc::clone(&tool_executor_arc),
                intent_matcher,
                template_engine,
                pipeline_converter,
                llm_bridge,
                history: Arc::clone(&history_arc),
                conversation_manager: Arc::clone(&conversation_manager_arc),
                stats_collector,
                context_tracker: Arc::clone(&context_tracker_arc),
                shell_executor_with_fixer,
                last_failed_command,
                command_router,
                workflow_intents: workflow_intents.clone(),
                workflow_executor: workflow_executor.clone(),
                llm_logger,
                voice_broadcaster,
                trace_store: Arc::clone(&trace_store),
                suggestion_engine: None, // ✨ Phase 4.1: 在配置 LLM 后初始化
                last_suggestions: Arc::new(RwLock::new(SuggestionCache::with_default_config())), // ✨ Phase 4.2 P1: 建议缓存（5分钟过期）
                likan_furnace: None, // ✨ Phase 4.3: 在配置建议引擎后初始化
                likan_task_handle: None,
                likan_statusbar: None, // ✨ Phase 4.3: 在启动后台循环时初始化
                likan_trigger: None, // ✨ Phase 4.3: 在启动后台循环时初始化
                bagua_palace: None, // ✨ v1.8.4: 八卦记忆宫，稍后初始化
                state_tracker: None, // ✨ v1.9.0: 两仪状态追踪器，稍后初始化
            };
        }

        // Fallback: 无持久化
        let shell_executor_with_fixer =
            Arc::new(ShellExecutorWithFixer::new().with_feedback_learner(feedback_learner));
        let last_failed_command = Arc::new(RwLock::new(None));

        // 初始化对话上下文管理器
        let conversation_context = Arc::new(RwLock::new(ContextManager::new(
            config.conversation.clone(),
        )));

        // ✨ Phase 2 (v1.3.0): 初始化服务层（Fallback 分支）
        let state_manager = Arc::new(StateManager::new(
            Arc::clone(&memory_arc),
            Arc::clone(&history_arc),
            Arc::clone(&context_tracker_arc),
            Arc::clone(&stats_collector),
            Arc::clone(&exec_logger_arc),
            Arc::clone(&conversation_context),
        ));

        let intent_service = Arc::new(IntentService::new(
            intent_matcher.clone(),
            template_engine.clone(),
            pipeline_converter.clone(),
            llm_bridge.clone(),
            workflow_intents.clone(),
            workflow_executor.clone(),
            Arc::clone(&llm_manager),
        ));

        // 创建运行时系统提示词
        let runtime_system_prompt = Arc::new(RwLock::new(None));

        let llm_service = Arc::new(LlmService::new(
            Arc::clone(&llm_manager),
            Arc::clone(&tool_registry),
            Arc::clone(&tool_executor_arc),
            config.llm.system_prompt.clone(),
            Arc::clone(&runtime_system_prompt),
        ));

        let shell_service = Arc::new(ShellService::new(Arc::clone(&shell_executor_with_fixer)));

        // ✨ 初始化 LLM 日志系统
        let llm_logger = Self::create_llm_logger(&config);

        // ✨ 初始化语音播报系统
        let voice_broadcaster = Self::create_voice_broadcaster(&config);

        Self {
            // 核心配置
            config,
            registry,
            // 服务层
            state_manager,
            intent_service,
            llm_service,
            shell_service,
            // 运行时状态
            runtime_system_prompt,
            // 原有字段
            llm_manager: Arc::clone(&llm_manager),
            memory: Arc::clone(&memory_arc),
            exec_logger: Arc::clone(&exec_logger_arc),
            tool_registry,
            tool_executor: Arc::clone(&tool_executor_arc),
            intent_matcher,
            template_engine,
            pipeline_converter,
            llm_bridge,
            history: Arc::clone(&history_arc),
            conversation_manager: Arc::clone(&conversation_manager_arc),
            stats_collector,
            context_tracker: Arc::clone(&context_tracker_arc),
            shell_executor_with_fixer,
            last_failed_command,
            command_router,
            workflow_intents,
            workflow_executor,
            llm_logger,
            voice_broadcaster,
            trace_store: Arc::clone(&trace_store),
            suggestion_engine: None, // ✨ Phase 4.1: 在配置 LLM 后初始化
            last_suggestions: Arc::new(RwLock::new(SuggestionCache::with_default_config())), // ✨ Phase 4.2 P1: 建议缓存（5分钟过期）
            likan_furnace: None, // ✨ Phase 4.3: 在配置建议引擎后初始化
            likan_task_handle: None,
            likan_statusbar: None, // ✨ Phase 4.3: 在启动后台循环时初始化
            likan_trigger: None, // ✨ Phase 4.3: 在启动后台循环时初始化
            bagua_palace: None, // ✨ v1.8.4: 八卦记忆宫，稍后初始化
            state_tracker: None, // ✨ v1.9.0: 两仪状态追踪器，稍后初始化
        }
    }

    /// 创建 LLM 日志记录器
    fn create_llm_logger(config: &Config) -> Option<Arc<crate::llm::LlmLogger>> {
        use crate::llm::{LlmLogger, LlmLoggerConfig};
        use std::path::PathBuf;

        if !config.llm.logging.enabled {
            return None;
        }

        // 构建 logger 配置
        let log_dir = if let Some(ref dir) = config.llm.logging.log_dir {
            PathBuf::from(Self::normalize_path(dir))
        } else {
            // 使用默认目录
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".realconsole")
                .join("llm_logs")
        };

        let logger_config = LlmLoggerConfig {
            enabled: true,
            log_dir,
            include_content: config.llm.logging.include_content,
            sensitive_patterns: vec![], // TODO: 从配置中读取
            retention_days: config.llm.logging.retention_days,
            max_size_mb: config.llm.logging.max_size_mb,
        };

        Some(Arc::new(LlmLogger::new(logger_config)))
    }

    /// 创建语音播报器
    fn create_voice_broadcaster(config: &Config) -> Option<Arc<VoiceBroadcaster>> {
        if !config.voice.enabled {
            return None;
        }

        // 检查平台支持
        if !VoiceBroadcaster::is_platform_supported() {
            eprintln!("⚠ 当前平台不支持语音播报功能");
            return None;
        }

        // 启动时强制关闭语音播报，需要用户主动开启（/voice on）
        // 这样可以避免用户忘记配置而导致意外播报，提升用户体验
        let broadcast_config = BroadcastConfig {
            enabled: false, // 强制关闭，忽略配置文件中的设置
            voice: config.voice.voice.clone(),
            max_queue_size: config.voice.max_queue_size,
        };

        Some(Arc::new(VoiceBroadcaster::new(broadcast_config)))
    }

    /// 获取 LLM 管理器的引用
    pub fn llm_manager(&self) -> Arc<RwLock<LlmManager>> {
        Arc::clone(&self.llm_manager)
    }

    /// 获取 LLM 日志记录器的引用
    ///
    /// # 返回
    /// - `Some(logger)`: 如果日志功能已启用
    /// - `None`: 如果日志功能未启用
    pub fn llm_logger(&self) -> Option<Arc<crate::llm::LlmLogger>> {
        self.llm_logger.as_ref().map(Arc::clone)
    }

    /// 尝试播报响应内容
    ///
    /// 根据配置自动播报 LLM 响应，会进行内容过滤和截断
    async fn try_broadcast_response(&self, response: &str) {
        // 检查是否启用语音播报
        let Some(ref broadcaster) = self.voice_broadcaster else {
            return;
        };

        // 检查是否启用自动播报
        if !self.config.voice.auto_broadcast {
            return;
        }

        // 检查播报器是否启用
        if !broadcaster.is_enabled().await {
            return;
        }

        // 过滤和处理内容
        use crate::voice::{filter_for_voice, FilterConfig};

        let filter_config = FilterConfig {
            filter_code_blocks: self.config.voice.filter_code_blocks,
            max_length: self.config.voice.max_broadcast_length,
        };

        // 过滤内容
        let filtered = match filter_for_voice(response, &filter_config) {
            Some(text) => text,
            None => return, // 过滤后为空，不播报
        };

        // 异步播报（不等待完成）
        let _ = broadcaster.speak(filtered).await;
    }

    /// 获取记忆系统的引用
    ///
    /// ⚠️ **已废弃**: 请使用 `state_manager().memory()` 代替
    #[deprecated(
        since = "1.3.0",
        note = "Use `state_manager().memory()` instead for better encapsulation"
    )]
    pub fn memory(&self) -> Arc<RwLock<Memory>> {
        Arc::clone(&self.memory)
    }

    /// 获取执行日志系统的引用
    ///
    /// ⚠️ **已废弃**: 请使用 `state_manager().exec_logger()` 代替
    #[deprecated(
        since = "1.3.0",
        note = "Use `state_manager().exec_logger()` instead for better encapsulation"
    )]
    pub fn exec_logger(&self) -> Arc<RwLock<ExecutionLogger>> {
        Arc::clone(&self.exec_logger)
    }

    /// 获取工具注册表的引用
    pub fn tool_registry(&self) -> Arc<RwLock<ToolRegistry>> {
        Arc::clone(&self.tool_registry)
    }

    /// 获取历史记录管理器的引用
    ///
    /// ⚠️ **已废弃**: 请使用 `state_manager().history()` 代替
    #[deprecated(
        since = "1.3.0",
        note = "Use `state_manager().history()` instead for better encapsulation"
    )]
    pub fn history(&self) -> Arc<RwLock<HistoryManager>> {
        Arc::clone(&self.history)
    }

    // ✨ Phase 2.2 (v1.3.0): 服务层访问器（Phase 2.4 改为 public）

    /// 获取 Intent 服务的引用
    pub fn intent_service(&self) -> &IntentService {
        &self.intent_service
    }

    /// 获取 LLM 服务的引用
    pub fn llm_service(&self) -> &LlmService {
        &self.llm_service
    }

    /// 获取 Shell 服务的引用
    pub fn shell_service(&self) -> &ShellService {
        &self.shell_service
    }

    /// 获取状态管理器的引用
    pub fn state_manager(&self) -> &StateManager {
        &self.state_manager
    }

    /// 获取运行时系统提示词的引用
    pub fn runtime_system_prompt(&self) -> Arc<RwLock<Option<String>>> {
        Arc::clone(&self.runtime_system_prompt)
    }

    /// 获取对话管理器的引用
    pub fn conversation_manager(&self) -> Arc<RwLock<ConversationManager>> {
        Arc::clone(&self.conversation_manager)
    }

    /// 获取统计收集器的引用
    ///
    /// ⚠️ **已废弃**: 请使用 `state_manager().stats_collector()` 代替
    #[deprecated(
        since = "1.3.0",
        note = "Use `state_manager().stats_collector()` instead for better encapsulation"
    )]
    pub fn stats_collector(&self) -> Arc<StatsCollector> {
        Arc::clone(&self.stats_collector)
    }

    /// 获取上下文追踪器的引用
    ///
    /// ⚠️ **已废弃**: 请使用 `state_manager().context_tracker()` 代替
    #[deprecated(
        since = "1.3.0",
        note = "Use `state_manager().context_tracker()` instead for better encapsulation"
    )]
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

    /// 配置建议引擎（Phase 4.1）
    ///
    /// 在配置 LLM 客户端后调用，初始化主动建议系统
    ///
    /// ✨ Phase 4.3: 同时初始化离坎炼化炉，共享离增强器
    pub fn configure_suggestion_engine(&mut self) {
        // 创建建议配置（使用默认配置）
        let config = SuggestionConfig::default();

        // 获取 LLM 客户端（如果可用）
        let llm_client = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;
                manager.primary().or(manager.fallback()).cloned()
            })
        });

        // ✨ Phase 4.3: 创建离坎炼化炉
        // 从配置文件加载或使用默认配置
        let furnace_config = self.config.likan.clone().unwrap_or_default();
        let furnace = LiKanFurnace::new(furnace_config);

        // 获取离增强器的共享引用
        let li_enhancer = furnace.li_enhancer();

        // 创建建议引擎，注入共享的离增强器
        let engine = if let Some(llm) = llm_client {
            SuggestionEngine::new(Arc::clone(&self.history), config)
                .with_llm(llm)
                .with_li_enhancer(li_enhancer)
        } else {
            SuggestionEngine::new(Arc::clone(&self.history), config)
                .with_li_enhancer(li_enhancer)
        };

        self.suggestion_engine = Some(Arc::new(engine));
        self.likan_furnace = Some(Arc::new(RwLock::new(furnace)));

        // ✨ v1.8.4: 初始化八卦记忆宫
        if let Some(ref bagua_config) = self.config.bagua {
            if bagua_config.enabled {
                // ✨ Phase 4: 创建持久化存储
                let storage = if let Some(ref path) = bagua_config.storage_path {
                    match crate::bagua::BaguaStorage::from_config(path) {
                        Ok(s) => Some(s),
                        Err(e) => {
                            eprintln!("⚠️ 八卦存储初始化失败: {}，使用内存模式", e);
                            None
                        }
                    }
                } else {
                    match crate::bagua::BaguaStorage::from_default_location() {
                        Ok(s) => Some(s),
                        Err(e) => {
                            eprintln!("⚠️ 八卦存储初始化失败: {}，使用内存模式", e);
                            None
                        }
                    }
                };

                // 创建宫殿配置
                let palace_config = crate::bagua::palace::PalaceConfig {
                    max_entries_per_dimension: bagua_config.dimension_capacity,
                    energy_decay_rate: 0.95,
                    relevance_threshold: 0.1,
                };

                // 创建记忆宫殿
                let palace = if let Some(storage) = storage {
                    let p = crate::bagua::BaguaMemoryPalace::with_storage(palace_config, storage);

                    // ✨ Phase 4: 启动时加载数据
                    match tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            p.load_from_storage().await
                        })
                    }) {
                        Ok(count) if count > 0 => {
                            println!("✨ 八卦记忆宫已启动（加载 {} 条记忆）", count);
                        }
                        Ok(_) => {
                            println!("✨ 八卦记忆宫已启动（新建宫殿）");
                        }
                        Err(e) => {
                            eprintln!("⚠️ 八卦记忆加载失败: {}，从空宫殿开始", e);
                        }
                    }

                    p
                } else {
                    println!("✨ 八卦记忆宫已启动（内存模式）");
                    crate::bagua::BaguaMemoryPalace::with_config(palace_config)
                };

                self.bagua_palace = Some(Arc::new(RwLock::new(palace)));
            }
        }

        // ✨ v1.9.0+v1.9.1: 初始化两仪状态追踪器
        let liangyyi_config = self.config.liangyyi.as_ref();

        if liangyyi_config.map(|c| c.enabled).unwrap_or(true) {
            let tracker_config = liangyyi_config
                .map(|c| c.state_tracker.clone())
                .unwrap_or_default();

            let state_tracker = crate::liangyyi::StateTracker::new(tracker_config);
            self.state_tracker = Some(Arc::new(state_tracker));
            println!("✨ 两仪状态追踪器已启动（时间维度）");
        } else {
            println!("ℹ️  两仪状态追踪器已禁用");
        }
    }

    /// 启动离坎炼化炉后台循环（Phase 4.3）
    ///
    /// 在配置建议引擎后调用，启动自主学习循环
    ///
    /// 循环策略：
    /// - 每1分钟检查一次是否需要循环
    /// - 炼化炉自己决定何时触发（基于配置的间隔）
    /// - 从 UnifiedTracer 获取最近的追踪数据
    /// - 执行坎（提取）→ 离（更新）循环
    /// - 状态栏实时显示（不干扰用户输入）
    pub fn start_likan_background_cycle(&mut self) {
        use std::time::{Duration, Instant};

        // 确保炼化炉已初始化
        let Some(furnace) = self.likan_furnace.as_ref() else {
            eprintln!("⚠️ 离坎炼化炉未初始化，无法启动后台循环");
            return;
        };

        let furnace = Arc::clone(furnace);
        let history = Arc::clone(&self.history);
        let exec_logger = Arc::clone(&self.exec_logger);
        let llm_logger = self.llm_logger.clone();
        let conversation_context = self.state_manager.conversation_context();
        let bagua_palace = self.bagua_palace.as_ref().map(Arc::clone); // ✨ v1.8.4: 克隆八卦记忆宫
        let suggestion_engine = self.suggestion_engine.as_ref().map(Arc::clone); // ✨ v1.8.4 Phase 3: 克隆建议引擎

        // 创建状态栏
        let statusbar = Arc::new(LiKanStatusBar::new());
        let status = statusbar.status();
        self.likan_statusbar = Some(Arc::clone(&statusbar));

        // ✨ Phase 4.4: 尝试加载反馈存储
        let feedback_storage = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                use crate::suggestion::feedback::FeedbackStorage;
                match FeedbackStorage::from_default_location().await {
                    Ok(storage) => Some(Arc::new(RwLock::new(storage))),
                    Err(e) => {
                        eprintln!("⚠️ 无法加载反馈存储: {}", e);
                        None
                    }
                }
            })
        });

        // 创建手动触发器（用于 /likan cycle 命令）
        let trigger = Arc::new(LiKanTrigger::new(
            Arc::clone(&furnace),
            Arc::clone(&history),
            Arc::clone(&exec_logger),
            llm_logger.clone(),
            Arc::clone(&conversation_context),
            feedback_storage,                         // ✨ Phase 4.4: 传递反馈存储
            self.bagua_palace.as_ref().map(Arc::clone), // ✨ v1.8.4: 传递八卦记忆宫
        ));
        self.likan_trigger = Some(Arc::clone(&trigger));

        // 获取炼化配置
        let furnace_config = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let f = furnace.read().await;
                f.config().clone()
            })
        });
        let cycle_interval_secs = furnace_config.cycle_interval_secs;

        // 初始化状态
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut s = status.write().await;
                s.cycle_interval_secs = cycle_interval_secs;
                drop(s);
                statusbar.update().await;
            })
        });

        // 启动后台任务
        let notification_mode = furnace_config.notification_mode;
        let handle = tokio::spawn(async move {
            loop {
                // 每1分钟检查一次（测试用）
                tokio::time::sleep(Duration::from_secs(60)).await;

                // 更新状态栏
                statusbar.update().await;

                // 检查是否应该触发循环
                let should_cycle = {
                    let f = furnace.read().await;
                    f.should_cycle()
                };

                if !should_cycle {
                    continue;
                }

                // 创建 UnifiedTracer 获取数据
                let tracer = crate::tracer::UnifiedTracer::new(
                    Arc::clone(&history),
                    Arc::clone(&exec_logger),
                    llm_logger.clone(),
                    Arc::clone(&conversation_context),
                );

                // 查询最近200条记录
                match tracer.query_all(200).await {
                    Ok(entries) => {
                        // 暂时使用空的 suggestion stats（Phase 4.4 可集成反馈系统）
                        let stats = std::collections::HashMap::new();

                        // ✨ v1.8.4: 准备八卦记忆宫引用
                        let palace_guard = if let Some(ref palace) = bagua_palace {
                            Some(palace.read().await)
                        } else {
                            None
                        };

                        // 执行炼化循环
                        let mut f = furnace.write().await;
                        match f
                            .cycle_once(&entries, &stats, palace_guard.as_deref())
                            .await
                        {
                            Ok(report) => {
                                // 更新状态
                                {
                                    let mut s = status.write().await;
                                    s.last_cycle = Some(Instant::now());
                                    s.pattern_count = report.patterns_found;
                                    s.high_confidence_count = report.high_confidence_patterns;
                                }

                                // ✨ v1.8.4 Phase 3: 炼化完成后刷新建议引擎知识
                                if let (Some(ref engine), Some(ref palace)) = (&suggestion_engine, &bagua_palace) {
                                    let palace_guard = palace.read().await;
                                    match engine.refresh_knowledge_from_bagua(&palace_guard).await {
                                        Ok(count) if count > 0 => {
                                            if notification_mode == crate::likan::NotificationMode::Minimal {
                                                eprintln!("✨ 建议引擎更新: {} 条新知识", count);
                                            }
                                        }
                                        Ok(_) => {
                                            // 没有新知识，不输出
                                        }
                                        Err(e) => {
                                            eprintln!("⚠️ 建议引擎刷新失败: {}", e);
                                        }
                                    }
                                }

                                // 根据 notification_mode 决定如何通知
                                use crate::likan::NotificationMode;
                                match notification_mode {
                                    NotificationMode::Minimal => {
                                        // 最小模式：只在有炼化结果时输出
                                        if report.patterns_found > 0 {
                                            eprintln!(
                                                "🌊🔥 炼化完成: {} 模式{}",
                                                report.patterns_found,
                                                if report.high_confidence_patterns > 0 {
                                                    format!(" ({} ⭐)", report.high_confidence_patterns)
                                                } else {
                                                    String::new()
                                                }
                                            );
                                        }
                                    }
                                    NotificationMode::Prompt => {
                                        // 提示符模式：更新状态栏（未来实现）
                                        statusbar.update().await;
                                    }
                                    NotificationMode::None => {
                                        // 静默模式：不输出任何通知
                                        // 用户可以通过 /likan status 查询
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️ 炼化炉循环失败: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ 无法获取追踪数据: {}", e);
                    }
                }
            }
        });

        self.likan_task_handle = Some(handle);

        // 不再 println，状态栏会自动显示
        // println!("✨ 离坎炼化炉后台循环已启动（每1分钟检查，5分钟触发）");
    }

    /// ✨ Phase 4.3: 获取离坎炼化炉提示符前缀
    ///
    /// 用于 `notification_mode: prompt` 模式，在命令行提示符中显示炼化炉状态
    ///
    /// # 返回
    /// - 有模式时：`🌊🔥 8` 或 `🌊🔥 8 (3 ⭐)`
    /// - 无模式时：`None`
    ///
    /// # 示例
    /// ```
    /// // 默认提示符
    /// (RealConsole v1) user %
    ///
    /// // 集成炼化炉状态后
    /// 🌊🔥 8 | (RealConsole v1) user %
    /// ```
    pub fn get_likan_prompt_prefix(&self) -> Option<String> {
        // 检查是否启用了炼化炉并且配置了 show_in_prompt
        let config = self.config.likan.as_ref()?;
        if !config.show_in_prompt {
            return None;
        }

        // 获取状态栏引用
        let statusbar = self.likan_statusbar.as_ref()?;

        // 使用 try_read 避免阻塞
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                match statusbar.status().try_read() {
                    Ok(status) => {
                        // 无模式时不显示
                        if status.pattern_count == 0 {
                            return None;
                        }

                        // 格式化前缀
                        Some(if status.high_confidence_count > 0 {
                            format!("🌊🔥 {} ({} ⭐)", status.pattern_count, status.high_confidence_count)
                        } else {
                            format!("🌊🔥 {}", status.pattern_count)
                        })
                    }
                    Err(_) => None, // 锁被占用，安全降级
                }
            })
        })
    }

    // ========================================
    // ✨ v1.8.4: 八卦记忆宫数据写入接口
    // ========================================

    /// 记录用户意图到乾维度（☰）
    ///
    /// 乾卦代表天、意图、目标
    async fn record_intent(&self, goal: &str, context: Option<String>, priority: f64) {
        if let Some(ref palace) = self.bagua_palace {
            use crate::bagua::{BaguaDimension, MemoryContent, MemoryEntry};

            let content = MemoryContent::Intent {
                goal: goal.to_string(),
                context,
                priority,
            };

            let entry = MemoryEntry::new(BaguaDimension::Qian, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录意图失败: {}", e);
            }
        }
    }

    /// 记录命令执行到震维度（☳）
    ///
    /// 震卦代表雷、行动、触发
    async fn record_action(&self, command: &str, success: bool, duration_ms: u64) {
        if let Some(ref palace) = self.bagua_palace {
            use crate::bagua::{entry::ActionResult, BaguaDimension, MemoryContent, MemoryEntry};

            let result = if success {
                ActionResult::Success
            } else {
                ActionResult::Failure {
                    error: "执行失败".to_string(),
                }
            };

            let content = MemoryContent::Action {
                command: command.to_string(),
                result,
                duration_ms,
            };

            let entry = MemoryEntry::new(BaguaDimension::Zhen, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录动作失败: {}", e);
            }
        }
    }

    // ========================================
    // ✨ v1.9.0: 两仪状态追踪接口
    // ========================================

    /// 根据命令类型和输入判断事件类型
    fn classify_event_from_command(&self, command_type: CommandType, input: &str) -> crate::liangyyi::Event {
        use crate::liangyyi::Event;

        match command_type {
            CommandType::Text => {
                // LLM 对话 → 思考
                Event::UserThink
            }
            CommandType::Shell => {
                // Shell 命令 → 执行
                Event::UserExecute
            }
            CommandType::Command => {
                // 系统命令，根据具体命令判断
                let cmd_lower = input.trim_start_matches('/').to_lowercase();
                let cmd_name = cmd_lower.split_whitespace().next().unwrap_or("");

                match cmd_name {
                    // 查询类命令 → 读取
                    "help" | "history" | "list" | "show" | "get" | "view" | "status"
                    | "trace" | "suggest" | "stats" => Event::UserRead,

                    // 配置类命令 → 写入
                    "config" | "set" | "add" | "remove" | "clear" | "wizard" => Event::UserWrite,

                    // 执行类命令 → 执行
                    "run" | "exec" | "test" | "build" => Event::UserExecute,

                    // 默认：读取
                    _ => Event::UserRead,
                }
            }
        }
    }

    /// 更新状态追踪器
    async fn update_state_tracker(&self, command_type: CommandType, input: &str) {
        if let Some(ref tracker) = self.state_tracker {
            let event = self.classify_event_from_command(command_type, input);
            tracker.update_from_event(event).await;

            // ✨ v1.9.0: 连接 Bagua Memory Palace
            // 每次更新后，记录快照到艮维度
            self.record_state_snapshot().await;

            // 如果有足够历史（>= 5 个快照），记录趋势到巽维度
            let history = tracker.history().await;
            if history.len() >= 5 {
                self.record_state_trend().await;
            }
        }
    }

    /// 记录状态快照到艮维度（☶）
    ///
    /// 艮卦代表山、停止、界限、记录点
    async fn record_state_snapshot(&self) {
        if let (Some(ref tracker), Some(ref palace)) = (&self.state_tracker, &self.bagua_palace) {
            use crate::bagua::{BaguaDimension, MemoryContent, MemoryEntry};

            let state = tracker.current_state().await;

            // 构建状态描述
            let state_desc = format!(
                "{} {} (阴={:.2}, 阳={:.2}, 平衡={:.2})",
                state.liangyyi.symbol(),
                state.sixiang.symbol(),
                state.taiji.yin_energy,
                state.taiji.yang_energy,
                state.taiji.balance()
            );

            // 构建元数据（JSON格式）
            let metadata = serde_json::json!({
                "yin_energy": state.taiji.yin_energy,
                "yang_energy": state.taiji.yang_energy,
                "balance": state.taiji.balance(),
                "liangyyi": format!("{:?}", state.liangyyi),
                "sixiang": format!("{:?}", state.sixiang),
                "timestamp": state.timestamp.to_rfc3339(),
            });

            let content = MemoryContent::Checkpoint {
                state: state_desc,
                snapshot_id: uuid::Uuid::new_v4().to_string(),
                metadata: Some(metadata.to_string()),
            };

            let entry = MemoryEntry::new(BaguaDimension::Gen, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录状态快照失败: {}", e);
            }
        }
    }

    /// 记录状态趋势到巽维度（☴）
    ///
    /// 巽卦代表风、渗透、趋势、渐进
    async fn record_state_trend(&self) {
        if let (Some(ref tracker), Some(ref palace)) = (&self.state_tracker, &self.bagua_palace) {
            use crate::bagua::{BaguaDimension, MemoryContent, MemoryEntry};

            let trend = tracker.analyze_trend().await;
            let stats = tracker.stats().await;

            // 计算变化率（基于最近历史）
            let recent_states = tracker.recent_states(5).await;
            let change_rate = if recent_states.len() >= 2 {
                let first = &recent_states[0];
                let last = &recent_states[recent_states.len() - 1];
                (last.taiji.yang_energy - first.taiji.yang_energy).abs()
            } else {
                0.0
            };

            // 构建趋势描述
            let pattern = match trend {
                crate::liangyyi::StateTrend::TowardYin => {
                    format!(
                        "趋向阴（变静）- 阴能量上升, 当前四象: {:?}",
                        stats.current_sixiang
                    )
                }
                crate::liangyyi::StateTrend::TowardYang => {
                    format!(
                        "趋向阳（变动）- 阳能量上升, 当前四象: {:?}",
                        stats.current_sixiang
                    )
                }
                crate::liangyyi::StateTrend::Stable => {
                    format!("稳定 - 能量平衡, 当前四象: {:?}", stats.current_sixiang)
                }
            };

            let content = MemoryContent::Trend {
                pattern,
                frequency: stats.total_snapshots,
                change_rate,
            };

            let entry = MemoryEntry::new(BaguaDimension::Xun, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录状态趋势失败: {}", e);
            }
        }
    }

    /// 记录对话到坤维度（☷）
    ///
    /// 坤卦代表地、承载、原始数据
    async fn record_conversation(&self, role: &str, message: &str, session_id: Option<String>) {
        if let Some(ref palace) = self.bagua_palace {
            use crate::bagua::{BaguaDimension, MemoryContent, MemoryEntry};

            let content = MemoryContent::Conversation {
                role: role.to_string(),
                message: message.to_string(),
                session_id,
            };

            let entry = MemoryEntry::new(BaguaDimension::Kun, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录对话失败: {}", e);
            }
        }
    }

    /// 记录用户反馈到兑维度（☱）
    ///
    /// 兑卦代表泽、交流、反馈
    async fn record_feedback(&self, action: &str, accepted: bool, score: f64) {
        if let Some(ref palace) = self.bagua_palace {
            use crate::bagua::{entry::FeedbackType, BaguaDimension, MemoryContent, MemoryEntry};

            let feedback_type = if accepted {
                FeedbackType::Accept
            } else {
                FeedbackType::Reject
            };

            let content = MemoryContent::Feedback {
                action: action.to_string(),
                feedback_type,
                score,
            };

            let entry = MemoryEntry::new(BaguaDimension::Dui, content);

            if let Err(e) = palace.write().await.store(entry).await {
                eprintln!("⚠️ 记录反馈失败: {}", e);
            }
        }
    }

    /// 处理用户输入
    pub fn handle(&self, line: &str) -> String {
        let line = line.trim();

        if line.is_empty() {
            return String::new();
        }

        // 开始计时
        let start = Instant::now();

        // ✨ v1.5.1: 创建追踪上下文并启动追踪
        let trace_ctx = TraceContext::new(line);
        let trace_id = trace_ctx.trace_id;

        // 创建根 Span（UserInput）
        let mut root_span = ExecutionSpan::new(
            trace_ctx.span_id,
            trace_ctx.trace_id,
            None,
            "user_input",
            SpanType::UserInput,
        );

        // 启动追踪
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.trace_store.start_trace(trace_id, line.to_string()).await;
            })
        });

        // 记录用户输入
        // NOTE: Memory system FROZEN per Phase 1 of redesign plan
        // See: docs/04-reports/memory-system-redesign.md
        // Memory 2.0 will focus on intelligent context orchestration rather than simple recording
        // Uncomment this when Memory 2.0 is implemented
        // tokio::task::block_in_place(|| {
        //     tokio::runtime::Handle::current().block_on(async {
        //         let mut memory = self.memory.write().await;
        //         memory.add(line.to_string(), EntryType::User);
        //     })
        // });

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

        // ✨ v1.8.4: 记录用户意图到八卦记忆宫（乾维度）
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.record_intent(line, None, 0.8).await;
            })
        });

        // 特殊处理：exit 命令直接退出
        if line.trim().to_lowercase() == "exit" {
            return "__QUIT__".to_string();
        }

        // ✨ Phase 4.2: 尝试快速执行建议（数字输入）
        if let Some(result) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.try_execute_cached_suggestion(line).await
            })
        }) {
            // 如果是错误消息，直接返回
            if result.contains("⚠") {
                return result;
            }
            // 否则，result 是要执行的命令，递归调用 handle
            return self.handle(&result);
        }

        // ✨ Phase 10.1: 使用智能命令路由器识别命令类型
        // ✨ v1.5.1: 创建 Router Span
        let (_router_ctx, mut router_span) = trace_ctx.create_child("command_router");
        router_span.span_type = SpanType::Router;

        let router_result = self.command_router.route(line);

        // 记录路由结果到 span 属性
        router_span.set_attribute(
            "route_type",
            serde_json::json!(format!("{:?}", router_result)),
        );
        router_span.set_success();

        // 记录 Router Span
        let router_span_clone = router_span.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.trace_store.record_span(router_span_clone).await;
            })
        });

        let (command_type, response) = match router_result {
            RouterCommandType::CommonShell(cmd) => {
                // 常见Shell命令，直接执行
                (CommandType::Shell, self.handle_shell(&trace_ctx, &cmd))
            }
            RouterCommandType::ForcedShell(cmd) => {
                // 强制Shell执行（!前缀）
                (CommandType::Shell, self.handle_shell(&trace_ctx, &cmd))
            }
            RouterCommandType::SystemCommand(cmd_name, arg) => {
                // 系统命令（/前缀）
                let input = if arg.is_empty() {
                    cmd_name
                } else {
                    format!("{} {}", cmd_name, arg)
                };
                (CommandType::Command, self.handle_command(&trace_ctx, &input))
            }
            RouterCommandType::NaturalLanguage(text) => {
                // 自然语言，交给LLM处理
                (CommandType::Text, self.handle_text(&trace_ctx, &text))
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

                    // ✨ v1.5.1: 记录到执行日志（带 trace_id）
                    {
                        let mut logger = self.exec_logger.write().await;
                        logger.log_with_trace(line.to_string(), command_type, success, duration, &response, trace_id);
                    }

                    // ✨ v1.8.4: 记录命令执行到八卦记忆宫（震维度）
                    self.record_action(line, success, duration.as_millis() as u64).await;

                    // ✨ Phase 8 + v1.5.1: 记录到命令历史（带 trace_id）
                    {
                        let mut history = self.history.write().await;
                        history.add_with_trace(line, success, trace_id);
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

                    // ✨ v1.9.0: 更新两仪状态追踪器
                    self.update_state_tracker(command_type, line).await;

                    // ✨ 语音播报：自动播报 LLM 响应
                    if command_type == CommandType::Text {
                        self.try_broadcast_response(&response).await;
                    }

                    // ✨ Phase 4.1: 命令失败时自动触发建议
                    if !success && command_type == CommandType::Shell {
                        // 只为Shell命令提供建议（系统命令和LLM对话不需要）
                        if let Some(ref engine) = self.suggestion_engine {
                            // 构建失败上下文
                            let mut ctx = SuggestionContext::from_env();
                            ctx.last_command_failed = true;
                            ctx.recent_commands.push(line.to_string());
                            // ✨ Phase 4.2 P1: 传递错误输出（用于拼写检查）
                            ctx.last_command_output = Some(response.clone());

                            // 获取历史命令
                            let history_guard = self.history.read().await;
                            let recent = history_guard.recent(3, crate::history::SortStrategy::Time);
                            drop(history_guard);

                            for entry in recent.iter().take(2) {
                                if entry.command != line {
                                    ctx.recent_commands.push(entry.command.clone());
                                }
                            }

                            // ✨ v1.9.0: 填充两仪状态信息
                            if let Some(ref tracker) = self.state_tracker {
                                let state = tracker.current_state().await;
                                let trend = tracker.analyze_trend().await;

                                ctx.current_sixiang = Some(format!("{:?}", state.sixiang));
                                ctx.energy_balance = Some(state.taiji.balance());
                                ctx.state_trend = Some(format!("{:?}", trend));
                            }

                            // 生成建议
                            let suggestions = engine.suggest(&ctx).await;

                            // ✨ Phase 4.2 P1: 更新建议缓存（带时间戳和过期检查）
                            {
                                let mut cache = self.last_suggestions.write().await;
                                cache.update(suggestions.clone());
                            }

                            // 显示建议（如果有）
                            if !suggestions.is_empty() && self.config.features.auto_suggest.unwrap_or(true) {
                                println!("\n{}", "💡 建议尝试：".yellow().bold());
                                for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                                    let icon = suggestion.category.icon();
                                    println!(
                                        "  {}. {} {}",
                                        i + 1,
                                        icon,
                                        suggestion.command.cyan()
                                    );
                                }
                                println!("{}\n", "提示: 使用 /suggest 查看更多建议".dimmed());
                            }
                        }
                    }

                    // 记录到记忆
                    // NOTE: Memory system FROZEN per Phase 1 of redesign plan
                    // See: docs/04-reports/memory-system-redesign.md
                    // Memory 2.0 will focus on intelligent context orchestration rather than simple recording
                    // Uncomment this when Memory 2.0 is implemented
                    // {
                    //     let mut memory = self.memory.write().await;
                    //     // 简化响应内容（最多保存前200个字符，考虑 UTF-8 边界）
                    //     let content = if response.len() > 200 {
                    //         // 找到安全的截断位置（UTF-8 字符边界）
                    //         let mut cutoff = 200.min(response.len());
                    //         while cutoff > 0 && !response.is_char_boundary(cutoff) {
                    //             cutoff -= 1;
                    //         }
                    //         format!("{}...", &response[..cutoff])
                    //     } else {
                    //         response.clone()
                    //     };
                    //     memory.add(content, EntryType::Assistant);
                    //
                    //     // 如果启用了自动保存，追加到文件
                    //     if let Some(ref mem_config) = self.config.memory {
                    //         if mem_config.auto_save.unwrap_or(false) {
                    //             if let Some(ref path) = mem_config.persistent_file {
                    //                 // 规范化路径，避免跟随工作目录改变
                    //                 let normalized_path = Self::normalize_path(path);
                    //                 let entries = memory.recent(1);
                    //                 if let Some(entry) = entries.first() {
                    //                     let _ = Memory::append_to_file(&normalized_path, entry);
                    //                 }
                    //             }
                    //         }
                    //     }
                    // }

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

        // ✨ v1.5.1: 完成根 Span 并记录追踪
        root_span.set_success();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // 记录根 Span
                let _ = self.trace_store.record_span(root_span).await;
                // 完成追踪
                let _ = self.trace_store.finish_trace(trace_id).await;
            })
        });

        response
    }

    /// 处理 Shell 命令
    /// ✨ Phase 9.2: 集成错误自动修复系统
    fn handle_shell(&self, ctx: &TraceContext, cmd: &str) -> String {
        // ✨ v1.5.1: 创建 Shell Execution Span
        let (_shell_ctx, mut shell_span) = ctx.create_child("shell_execution");
        shell_span.span_type = SpanType::ShellExec;
        shell_span.set_attribute("command", serde_json::json!(cmd));

        if !self.config.features.shell_enabled {
            shell_span.set_failed("Shell execution disabled");
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self.trace_store.record_span(shell_span).await;
                })
            });
            return format!("{}", "Shell 执行已禁用".red());
        }

        // 特殊处理：cd 命令需要在主进程中生效
        let cmd_trimmed = cmd.trim();
        if cmd_trimmed.starts_with("cd ") || cmd_trimmed == "cd" {
            let result = self.handle_cd_command(cmd_trimmed);
            shell_span.set_success();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self.trace_store.record_span(shell_span).await;
                })
            });
            return result;
        }

        // ✨ Phase 9.2: 使用 ShellExecutorWithFixer 执行命令（带错误分析）
        let execution_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.shell_executor_with_fixer
                    .execute_with_analysis(cmd)
                    .await
            })
        });

        // 记录执行结果到 Span
        shell_span.set_attribute("success", serde_json::json!(execution_result.success));

        // ✨ Phase 4.2 P1: 检查是否为 "command not found" 错误
        // 如果是，跳过自动修复流程，让新的建议系统（带拼写纠错）来处理
        let is_command_not_found = execution_result.output.to_lowercase().contains("command not found")
            || execution_result.output.to_lowercase().contains("not found");

        // 如果执行失败且有修复策略，但不是 "command not found" 错误，显示交互式修复流程
        let result = if !execution_result.success && !execution_result.fix_strategies.is_empty() && !is_command_not_found {
            shell_span.set_attribute("has_fix_strategies", serde_json::json!(true));

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
            // 正常输出或没有修复建议的错误，或者是 "command not found" 错误（留给建议系统处理）
            execution_result.output
        };

        // 完成 Span 并记录
        if execution_result.success {
            shell_span.set_success();
        } else {
            shell_span.set_failed("Command execution failed");
        }

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.trace_store.record_span(shell_span).await;
            })
        });

        result
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
    fn handle_command(&self, ctx: &TraceContext, input: &str) -> String {
        // ✨ v1.5.1: 创建 System Command Span
        let (_cmd_ctx, mut cmd_span) = ctx.create_child("system_command");
        cmd_span.span_type = SpanType::SystemCommand;
        cmd_span.set_attribute("command", serde_json::json!(input));

        let parts: Vec<&str> = input.splitn(2, ' ').collect();
        let cmd_name = parts[0];
        let arg = parts.get(1).copied().unwrap_or("");

        cmd_span.set_attribute("command_name", serde_json::json!(cmd_name));
        cmd_span.set_attribute("argument", serde_json::json!(arg));

        // ✨ Phase 9.2: 特殊处理 /fix 命令
        // ✨ Phase 4.1: 特殊处理 /suggest 命令
        let result = if cmd_name == "fix" {
            self.handle_fix_command()
        } else if cmd_name == "suggest" {
            self.handle_suggest_command()
        } else {
            match self.registry.execute(cmd_name, arg) {
                Ok(output) => {
                    cmd_span.set_success();
                    output
                }
                Err(err) => {
                    cmd_span.set_failed(err.to_string());
                    format!("{}", err.red())
                }
            }
        };

        // 记录 Span
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.trace_store.record_span(cmd_span).await;
            })
        });

        result
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
                // 为重试创建新的追踪上下文
                let retry_ctx = TraceContext::new(format!("/fix: {}", cmd));
                self.handle_shell(&retry_ctx, &cmd)
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

    /// ✨ Phase 4.1: 处理 /suggest 命令 - 生成智能建议
    fn handle_suggest_command(&self) -> String {
        // 检查建议引擎是否已初始化
        let engine = match &self.suggestion_engine {
            Some(engine) => engine,
            None => {
                return format!(
                    "{}\n{}",
                    "⚠ 建议系统未启用".yellow(),
                    "提示: 建议系统需要配置 LLM 客户端".dimmed()
                );
            }
        };

        // 构建建议上下文
        let context = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut ctx = SuggestionContext::from_env();

                // 添加最近的命令（最多3条）
                let history_guard = self.history.read().await;
                let recent = history_guard.recent(3, crate::history::SortStrategy::Time);
                ctx.recent_commands = recent.iter().map(|e| e.command.clone()).collect();

                // 获取最近失败的命令
                let last_failed = self.last_failed_command.read().await;
                ctx.last_command_failed = last_failed.is_some();

                // ✨ v1.9.0: 填充两仪状态信息
                if let Some(ref tracker) = self.state_tracker {
                    let state = tracker.current_state().await;
                    let trend = tracker.analyze_trend().await;

                    ctx.current_sixiang = Some(format!("{:?}", state.sixiang));
                    ctx.energy_balance = Some(state.taiji.balance());
                    ctx.state_trend = Some(format!("{:?}", trend));

                    // ✨ v1.9.4: 填充学习阶段信息
                    let (learning_phase, volatility, change_rate) =
                        tracker.detect_learning_phase().await;
                    ctx.learning_phase = Some(format!("{:?}", learning_phase));
                    ctx.volatility = Some(volatility);
                    ctx.change_rate = Some(change_rate);
                }

                ctx
            })
        });

        // 生成建议
        let suggestions = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                engine.suggest(&context).await
            })
        });

        // ✨ Phase 4.2 P1: 更新建议缓存（带时间戳和过期检查）
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut cache = self.last_suggestions.write().await;
                cache.update(suggestions.clone());
            })
        });

        // 格式化输出
        if suggestions.is_empty() {
            return format!(
                "{}\n{}",
                "💡 暂无建议".dimmed(),
                "提示: 执行一些命令后，系统会学习您的使用模式并提供更好的建议".dimmed()
            );
        }

        let mut output = String::new();

        // 头部信息
        output.push_str(&format!("{}\n", "━━━ 💡 智能建议 ━━━".cyan().bold()));

        // 显示上下文信息
        if let Ok(current_dir) = std::env::current_dir() {
            let dir_name = current_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            output.push_str(&format!("📂 {}\n", dir_name.dimmed()));
        }

        // 按类别分组（可选，暂时不分组，保持简洁）
        output.push('\n');

        for (i, suggestion) in suggestions.iter().enumerate() {
            let icon = suggestion.category.icon();
            let source_badge = format!("[{}]", suggestion.source.display_name()).dimmed();
            let score_badge = if suggestion.is_high_quality() {
                format!("{}%", (suggestion.score * 100.0) as u8).green().bold()
            } else {
                format!("{}%", (suggestion.score * 100.0) as u8).yellow()
            };

            // 主命令行
            output.push_str(&format!(
                "  {} {} {} {}\n",
                format!("[{}]", i + 1).cyan().bold(),
                icon,
                suggestion.command.cyan().bold(),
                score_badge
            ));

            // 描述和来源
            output.push_str(&format!(
                "     {} {}\n",
                suggestion.description.dimmed(),
                source_badge
            ));

            if i < suggestions.len() - 1 {
                output.push('\n');
            }
        }

        // 底部提示
        output.push_str(&format!(
            "\n{}\n{}\n{}",
            "━━━━━━━━━━━━━━━━━━".dimmed(),
            "💡 提示：直接输入数字快速执行建议命令".dimmed(),
            "⚙️  配置：在 realconsole.yaml 中可关闭自动建议 (features.auto_suggest: false)".dimmed()
        ));

        output
    }

    /// ✨ Phase 4.2: 处理数字快速执行建议
    ///
    /// 当用户输入纯数字（如"1"、"2"）时，执行对应索引的建议
    async fn try_execute_cached_suggestion(&self, input: &str) -> Option<String> {
        // 检查是否为纯数字
        let index: usize = match input.trim().parse::<usize>() {
            Ok(n) if n > 0 => n - 1, // 用户输入1-based，转为0-based索引
            _ => return None, // 不是有效数字
        };

        // 获取缓存的建议（带过期检查）
        let mut cache = self.last_suggestions.write().await;

        // 尝试获取建议（如果缓存为空或已过期，get()会返回None并自动清理）
        let result = if let Some(suggestions) = cache.get() {
            // 检查索引是否有效
            if index >= suggestions.len() {
                let count = suggestions.len();
                drop(cache);
                return Some(format!(
                    "{}\n{}",
                    format!("⚠ 无效的建议编号：{}", index + 1).yellow(),
                    format!("提示: 当前有 {} 条建议可用", count).dimmed()
                ));
            }

            // 获取对应的建议并克隆命令
            Ok(suggestions[index].command.clone())
        } else {
            // 缓存为空或已过期
            let status = cache.status();
            Err(status.description())
        };

        // 释放锁
        drop(cache);

        // 处理结果
        let command = match result {
            Ok(cmd) => cmd,
            Err(reason) => {
                return Some(format!(
                    "{}\n{}\n{}",
                    "⚠ 没有可用的建议".yellow(),
                    format!("原因: {}", reason).dimmed(),
                    "提示: 先执行 /suggest 命令查看建议，或等待命令失败时的自动建议".dimmed()
                ));
            }
        };

        // 显示将要执行的命令
        println!(
            "{} {}",
            "⚡ 执行建议:".green().bold(),
            command.cyan()
        );

        // 返回命令让系统重新处理
        Some(command)
    }

    /// 处理自由文本（Intent 识别 → LLM 对话）
    fn handle_text(&self, ctx: &TraceContext, text: &str) -> String {
        // ✨ v1.5.1: 创建 Handler Span
        let (_text_ctx, mut handler_span) = ctx.create_child("text_handler");
        handler_span.span_type = SpanType::Handler;
        handler_span.set_attribute("input", serde_json::json!(text));

        // ✨ Phase 8 Week 2: 优先检查多轮对话（一分为三：对话态、意图态、LLM态）
        // 1️⃣ 对话态：如果有活跃对话，继续对话流程
        if has_active_conversation() {
            handler_span.set_attribute("mode", serde_json::json!("conversation"));
            let result = self.handle_conversation_input(text);
            handler_span.set_success();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self.trace_store.record_span(handler_span).await;
                })
            });
            return result;
        }

        // 2️⃣ 检测是否需要启动新对话（特定意图需要参数收集）
        if let Some(response) = self.try_start_conversation(text) {
            handler_span.set_attribute("mode", serde_json::json!("conversation_start"));
            handler_span.set_success();
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let _ = self.trace_store.record_span(handler_span).await;
                })
            });
            return response;
        }

        // 🔧 优先使用 LLM 工具调用（如果启用且可用）
        let use_tools = self.config.features.tool_calling_enabled.unwrap_or(false);

        let result = if use_tools {
            // 使用 LLM 工具调用模式（更智能，支持 count_code_lines 等工具）
            handler_span.set_attribute("mode", serde_json::json!("llm_with_tools"));
            self.handle_text_with_tools(ctx, text)
        } else if let Some(response) = self.try_match_workflow(text) {
            // ✨ Phase 8: 尝试匹配 Workflow Intent（套路化复用）
            handler_span.set_attribute("mode", serde_json::json!("workflow"));
            response
        } else if let Some(plan) = self.try_match_intent(text) {
            // ✨ Phase 3: 回退到 Intent 识别（道法自然 - 先识别意图，未匹配则回退到流式LLM）
            handler_span.set_attribute("mode", serde_json::json!("intent"));
            self.execute_intent(&plan)
        } else {
            // 最后回退：使用传统流式输出模式
            handler_span.set_attribute("mode", serde_json::json!("llm_streaming"));
            self.handle_text_streaming(ctx, text)
        };

        handler_span.set_success();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _ = self.trace_store.record_span(handler_span).await;
            })
        });

        result
    }

    /// 使用工具调用处理文本
    fn handle_text_with_tools(&self, ctx: &TraceContext, text: &str) -> String {
        // ✨ v1.5.1: 创建 LLM Span（工具调用模式）
        let (_llm_ctx, mut llm_span) = ctx.create_child("llm_with_tools");
        llm_span.span_type = SpanType::LlmCall;
        llm_span.set_attribute("mode", serde_json::json!("with_tools"));
        llm_span.set_attribute("input", serde_json::json!(text));

        // ✨ Phase 3: 集成对话上下文支持（与 handle_text_streaming 对齐）
        // ✨ Phase 2.2 (v1.3.0): 使用 LlmService 处理

        // 🔍 LLM Logger: 初始化日志记录
        let (logger_opt, session_id_opt, start_time) = if let Some(ref logger) = self.llm_logger {
            let (session_id, start) = logger.start_logging("tools");
            (Some(logger.clone()), Some(session_id), start)
        } else {
            use std::time::Instant;
            (None, None, Instant::now())
        };

        // ✨ Phase 3: 检查是否应该使用上下文
        let (should_use_context, messages) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let ctx_arc = self.state_manager().conversation_context();
                let mut ctx_manager = ctx_arc.write().await;

                // 检查是否应该使用上下文
                let should_use = ctx_manager.should_use_context(text);

                // 如果使用上下文，构建消息列表
                let msgs = if should_use {
                    Some(ctx_manager.build_messages(text))
                } else {
                    None
                };

                (should_use, msgs)
            })
        });

        // 先获取模型名称（用于 spinner）
        let model_name = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;
                manager
                    .primary()
                    .or(manager.fallback())
                    .map(|llm| llm.model().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            })
        });

        // 启动 spinner（带模型名称）
        use crate::spinner::simplify_model_name;
        let spinner = Spinner::with_label(&simplify_model_name(&model_name));

        // 🔍 LLM Logger: 保存消息副本用于日志
        let messages_for_log = messages.clone().unwrap_or_else(|| {
            vec![crate::llm::Message::user(text)]
        });

        // 创建 LLM 请求（带上下文支持）
        let request = if let Some(msgs) = messages {
            LlmRequest::with_tools_and_context(msgs)
        } else {
            LlmRequest::with_tools(text.to_string())
        };

        // 调用 LlmService
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.llm_service().process(request).await })
        });

        match result {
            Ok(llm_response) => {
                // 停止 spinner
                spinner.stop();

                let response = llm_response.text;
                // Clone for logging before response is moved
                let full_response_clone = response.clone();

                // ✨ v1.5.1: 记录 LLM 响应到 Span
                llm_span.set_attribute("success", serde_json::json!(true));
                llm_span.set_attribute("response_length", serde_json::json!(response.len()));

                // ✨ 解析并显示对话轮次调试信息（成功时，仅当配置启用且为 debug 模式）
                // ✨ v1.5.1: 同时记录工具调用信息和创建 Tool Span
                if let Some(rounds) =
                    crate::tool_executor::ToolExecutor::decode_debug_info(&response)
                {
                    // 记录工具调用统计到 LLM Span
                    let total_tool_calls: usize = rounds.iter().map(|r| r.tool_calls.len()).sum();
                    llm_span.set_attribute("tool_calls_count", serde_json::json!(total_tool_calls));
                    llm_span.set_attribute("rounds_count", serde_json::json!(rounds.len()));

                    // ✨ v1.5.1: 为每个工具调用创建 Tool Span
                    for round in &rounds {
                        for tool_call in &round.tool_calls {
                            let (_tool_ctx, mut tool_span) = ctx.create_child(format!("tool_{}", tool_call.name));
                            tool_span.span_type = SpanType::ToolCall;
                            tool_span.set_attribute("tool_name", serde_json::json!(&tool_call.name));
                            tool_span.set_attribute("arguments", serde_json::json!(&tool_call.arguments));

                            // 查找对应的工具结果
                            if let Some(result) = round.tool_results.iter()
                                .find(|r| r.contains(&tool_call.name)) {
                                tool_span.set_attribute("result_preview", serde_json::json!(
                                    if result.len() > 100 {
                                        format!("{}...", &result[..100])
                                    } else {
                                        result.clone()
                                    }
                                ));
                            }

                            tool_span.set_success();
                            let tool_span_clone = tool_span.clone();
                            tokio::task::block_in_place(|| {
                                tokio::runtime::Handle::current().block_on(async {
                                    let _ = self.trace_store.record_span(tool_span_clone).await;
                                })
                            });
                        }
                    }

                    // 显示调试信息
                    if self.config.display.show_conversation_rounds
                        && self.config.display.mode.show_debug()
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
                let clean_response = if let Some(pos) = response.find("__DEBUG__") {
                    response[..pos].to_string()
                } else {
                    response
                };

                // ✨ Phase 3: 添加轮次到 ContextManager（仅在使用上下文时）
                if should_use_context {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let ctx_arc = self.state_manager().conversation_context();
                            let mut ctx_manager = ctx_arc.write().await;

                            // 创建新的轮次
                            let turn = Turn::new(text.to_string(), clean_response.clone());
                            ctx_manager.add_turn(turn);
                        })
                    });
                }

                // 🔍 LLM Logger: 记录成功的交互
                if let (Some(logger), Some(session_id)) = (logger_opt.clone(), session_id_opt.clone()) {
                    let response_clone = clean_response.clone();
                    // full_response_clone already created above
                    let messages_clone = messages_for_log.clone();
                    let model_name_clone = model_name.clone();
                    let text_clone = text.to_string();
                    tokio::spawn(async move {
                        // 从调试信息中提取工具调用
                        let (tools_used, tool_results_summary) =
                            if let Some(rounds) = crate::tool_executor::ToolExecutor::decode_debug_info(&full_response_clone) {
                                let mut tools = Vec::new();
                                let mut results = Vec::new();

                                for round in rounds {
                                    for tool_call in &round.tool_calls {
                                        if !tools.contains(&tool_call.name) {
                                            tools.push(tool_call.name.clone());
                                        }
                                    }
                                    results.extend(round.tool_results.clone());
                                }

                                let summary = if !results.is_empty() {
                                    Some(format!("{} 个工具调用，{} 次执行", tools.len(), results.len()))
                                } else {
                                    None
                                };

                                (tools, summary)
                            } else {
                                (vec![], None)
                            };

                        // 构建上下文信息
                        let context = Some(crate::llm::CallContext {
                            user_input: Some(text_clone),
                            intent: None, // TODO: 添加 Intent 识别结果
                            tools_used,
                            tool_results_summary,
                        });

                        logger.log_interaction(crate::llm::logger::LogInteractionParams {
                            session_id,
                            model: model_name_clone,
                            messages: &messages_clone,
                            response_content: Some(response_clone),
                            start_time,
                            is_streaming: false,
                            error: None,
                            context,
                        }).await;
                    });
                }

                // ✨ v1.5.1: 完成并记录 LLM Span
                llm_span.set_success();
                let llm_span_clone = llm_span.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = self.trace_store.record_span(llm_span_clone).await;
                    })
                });

                clean_response
            }
            Err(e) => {
                // 停止 spinner
                spinner.stop();

                let error_msg = e.to_string();

                // ✨ 解析并显示对话轮次调试信息（仅 debug 模式）
                if let Some(rounds) = crate::tool_executor::ToolExecutor::decode_debug_info(&error_msg) {
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
                let error_text = if let Some(pos) = error_msg.find("__DEBUG__") {
                    &error_msg[..pos]
                } else {
                    &error_msg
                };

                // ✨ v1.5.1: 记录 LLM 失败到 Span
                llm_span.set_failed(error_text);
                let llm_span_clone = llm_span.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = self.trace_store.record_span(llm_span_clone).await;
                    })
                });

                // 🔍 LLM Logger: 记录失败的交互
                if let (Some(logger), Some(session_id)) = (logger_opt, session_id_opt) {
                    let error_clone = error_text.to_string();
                    let error_msg_full = error_msg.clone();
                    let messages_clone = messages_for_log.clone();
                    let model_name_clone = model_name.clone();
                    let text_clone = text.to_string();
                    tokio::spawn(async move {
                        // 从调试信息中提取工具调用（即使失败也可能有部分工具调用）
                        let (tools_used, tool_results_summary) =
                            if let Some(rounds) = crate::tool_executor::ToolExecutor::decode_debug_info(&error_msg_full) {
                                let mut tools = Vec::new();
                                let mut _results_count = 0;

                                for round in rounds {
                                    for tool_call in &round.tool_calls {
                                        if !tools.contains(&tool_call.name) {
                                            tools.push(tool_call.name.clone());
                                        }
                                    }
                                    _results_count += round.tool_results.len();
                                }

                                let summary = if !tools.is_empty() {
                                    Some(format!("{} 个工具调用（部分失败）", tools.len()))
                                } else {
                                    None
                                };

                                (tools, summary)
                            } else {
                                (vec![], None)
                            };

                        // 构建上下文信息
                        let context = Some(crate::llm::CallContext {
                            user_input: Some(text_clone),
                            intent: None,
                            tools_used,
                            tool_results_summary,
                        });

                        logger.log_interaction(crate::llm::logger::LogInteractionParams {
                            session_id,
                            model: model_name_clone,
                            messages: &messages_clone,
                            response_content: None,
                            start_time,
                            is_streaming: false,
                            error: Some(error_clone),
                            context,
                        }).await;
                    });
                }

                // 解析上下文长度错误
                if let Some((requested, limit)) = parse_context_length_error(error_text) {
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
                    error_text,
                    "提示: 使用".dimmed(),
                    self.config.prefix.dimmed()
                )
            }
        }
    }

    /// 使用流式输出处理文本（传统模式）
    fn handle_text_streaming(&self, ctx: &TraceContext, text: &str) -> String {
        // ✨ v1.5.1: 创建 LLM Span（流式模式）
        let (_llm_ctx, mut llm_span) = ctx.create_child("llm_streaming");
        llm_span.span_type = SpanType::LlmCall;
        llm_span.set_attribute("mode", serde_json::json!("streaming"));
        llm_span.set_attribute("input", serde_json::json!(text));

        // ✨ Phase 3: 集成对话上下文
        // ✨ Phase 2.4 (v1.3.0): 使用 LlmService 处理流式输出

        // 开始计时
        let start = Instant::now();

        // ✨ LLM 日志: 开始记录（如果启用）
        let (logger_opt, session_id_opt) = if let Some(ref logger) = self.llm_logger {
            let (session_id, _) = logger.start_logging("streaming");
            (Some(logger), Some(session_id))
        } else {
            (None, None)
        };

        // 检查是否应该使用上下文
        let (should_use_context, messages) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let ctx_arc = self.state_manager().conversation_context();
                let mut ctx_manager = ctx_arc.write().await;

                // 检查是否应该使用上下文
                let should_use = ctx_manager.should_use_context(text);

                // 如果使用上下文，构建消息列表
                let msgs = if should_use {
                    ctx_manager.build_messages(text)
                } else {
                    vec![Message::user(text)]
                };

                (should_use, msgs)
            })
        });

        // 先获取模型名称（用于 spinner）
        let model_name = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;
                manager
                    .primary()
                    .or(manager.fallback())
                    .map(|llm| llm.model().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            })
        });

        // 启动 spinner（带模型名称）
        use crate::spinner::simplify_model_name;
        let spinner = Spinner::with_label(&simplify_model_name(&model_name));

        // 调用 LLM（直接使用 LlmManager 以支持多轮上下文）
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let manager = self.llm_manager.read().await;

                // 流式输出（在回调中直接打印）
                manager.chat_stream_with_messages(messages.clone(), |chunk| {
                    print!("{}", chunk);
                    std::io::Write::flush(&mut std::io::stdout()).ok();
                }).await
            })
        });

        match result {
            Ok(response) => {
                // 停止 spinner
                spinner.stop();

                // ✨ v1.5.1: 记录 LLM 响应到 Span
                llm_span.set_attribute("success", serde_json::json!(true));
                llm_span.set_attribute("response_length", serde_json::json!(response.len()));
                llm_span.set_success();

                // ✨ LLM 日志: 记录成功的交互
                if let (Some(logger), Some(session_id)) = (logger_opt, session_id_opt) {
                    let logger_clone = Arc::clone(logger);
                    let session_id_clone = session_id.clone();
                    let model_name_clone = model_name.clone();
                    let messages_clone = messages.clone();
                    let response_clone = response.clone();
                    let start_clone = start;
                    let text_clone = text.to_string();

                    // 异步记录日志，不阻塞主流程
                    tokio::spawn(async move {
                        // 构建上下文信息
                        let context = Some(crate::llm::CallContext {
                            user_input: Some(text_clone),
                            intent: None,
                            tools_used: vec![],
                            tool_results_summary: None,
                        });

                        logger_clone
                            .log_interaction(crate::llm::logger::LogInteractionParams {
                                session_id: session_id_clone,
                                model: model_name_clone,
                                messages: &messages_clone,
                                response_content: Some(response_clone),
                                start_time: start_clone,
                                is_streaming: true,
                                error: None,
                                context,
                            })
                            .await;
                    });
                }

                // ✨ Phase 3: 添加轮次到 ContextManager
                if should_use_context {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(async {
                            let ctx_arc = self.state_manager().conversation_context();
                            let mut ctx_manager = ctx_arc.write().await;

                            // 创建新的轮次
                            let turn = Turn::new(text.to_string(), response.clone());
                            ctx_manager.add_turn(turn);
                        })
                    });
                }

                // 计算耗时
                let elapsed = start.elapsed();

                // 流式输出已经完成，不需要额外换行
                Display::execution_timing(self.config.display.mode, elapsed.as_secs_f64());

                // ✨ v1.5.1: 记录 LLM Span
                let llm_span_clone = llm_span.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = self.trace_store.record_span(llm_span_clone).await;
                    })
                });

                // 返回空字符串，因为内容已通过流式输出显示
                String::new()
            }
            Err(e) => {
                // 停止 spinner
                spinner.stop();

                let error_text = e.to_string();

                // ✨ v1.5.1: 记录 LLM 失败到 Span
                llm_span.set_failed(&error_text);
                let llm_span_clone = llm_span.clone();
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let _ = self.trace_store.record_span(llm_span_clone).await;
                    })
                });

                // ✨ LLM 日志: 记录失败的交互
                if let (Some(logger), Some(session_id)) = (logger_opt, session_id_opt) {
                    let logger_clone = Arc::clone(logger);
                    let session_id_clone = session_id.clone();
                    let model_name_clone = model_name.clone();
                    let messages_clone = messages.clone();
                    let error_msg_clone = error_text.clone();
                    let start_clone = start;
                    let text_clone = text.to_string();

                    // 异步记录错误日志
                    tokio::spawn(async move {
                        // 构建上下文信息
                        let context = Some(crate::llm::CallContext {
                            user_input: Some(text_clone),
                            intent: None,
                            tools_used: vec![],
                            tool_results_summary: None,
                        });

                        logger_clone
                            .log_interaction(crate::llm::logger::LogInteractionParams {
                                session_id: session_id_clone,
                                model: model_name_clone,
                                messages: &messages_clone,
                                response_content: None,
                                start_time: start_clone,
                                is_streaming: true,
                                error: Some(error_msg_clone),
                                context,
                            })
                            .await;
                    });
                }

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
        // ✨ Phase 2.2 (v1.3.0): 使用 IntentService 处理

        // 创建 Intent 请求
        let request = IntentRequest::from_config(text.to_string(), &self.config);

        // 调用 IntentService
        let response = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.intent_service().process(request).await })
        });

        match response {
            Ok(intent_response) => {
                // 显示意图识别结果
                if let Some(intent_name) = &intent_response.intent_name {
                    if intent_name == "llm_generated" {
                        Display::llm_generation(self.config.display.mode);
                    } else if !intent_response.is_workflow {
                        Display::intent_match(
                            self.config.display.mode,
                            intent_name,
                            intent_response.confidence,
                        );
                    }
                }

                // TODO: Phase 2.2 - LLM 验证逻辑需要迁移到 IntentService
                // 当前暂时跳过验证，后续完善

                intent_response.plan
            }
            Err(e) => {
                // IntentService 匹配失败
                if self.config.intent.llm_generation_enabled.unwrap_or(false) {
                    Display::fallback_warning(self.config.display.mode, &e.to_string());
                }
                None
            }
        }
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

    // Phase 2: 尝试使用 LLM 补充提取实体
    // ✨ Phase 2.3 (v1.3.0): 清理未使用的辅助方法
    // 已删除：try_llm_extraction (29 lines)
    // 已删除：try_llm_validation (23 lines)
    // 已删除：display_validation_warning (13 lines)
    // 已删除：ask_user_confirmation (12 lines)
    // 原因：这些方法在迁移到 IntentService 后不再使用

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
    #[ignore] // Phase 1: Memory system frozen, will re-enable when Memory 2.0 is implemented
    async fn test_agent_memory_tracking() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 执行一个命令
        agent.handle("/nonexistent");

        // 检查记忆系统是否记录了输入
        let memory = agent.state_manager().memory();
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
        let logger = agent.state_manager().exec_logger();
        let logger_guard = logger.read().await;
        let stats = logger_guard.stats();

        assert_eq!(stats.total, 1);
    }

    // ========== handle_cd_command 测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
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
    #[serial_test::serial]
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
    #[serial_test::serial]
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
        let memory = agent.state_manager().memory();
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
        let logger = agent.state_manager().exec_logger();
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
    #[serial_test::serial]
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
    #[ignore] // Phase 1: Memory system frozen, will re-enable when Memory 2.0 is implemented
    async fn test_memory_persistence_config() {
        let mut config = Config::default();
        // 不配置持久化文件
        config.memory = None;

        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 执行一个命令
        agent.handle("/help");

        // 记忆应该正常工作（即使没有持久化）
        let memory = agent.state_manager().memory();
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
        let logger = agent.state_manager().exec_logger();
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
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();

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
