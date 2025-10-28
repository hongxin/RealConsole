//! RealConsole - 融合东方哲学智慧的智能 CLI Agent
//!
//! 基于 Rust 的智能 CLI 代理，结合易经智慧与现代 AI

// Allow unused code for future use and module exports
#![allow(dead_code)]
#![allow(unused_imports)]

mod advanced_tools;
mod agent;
mod bagua; // ✨ v1.8.4: 八卦记忆宫（多维记忆系统）
mod builtin_tools;
mod command;
mod command_router; // ✨ Phase 10.1: 智能命令路由系统
mod commands;
mod completion; // ✨ Phase 1: Tab 补全系统（静态补全）
mod config;
mod conversation; // ✨ Phase 8 Week 2: 多轮对话支持
mod display;
mod dsl;
mod error;
mod error_fixer; // ✨ Phase 9.2: 错误自动修复
mod execution_logger;
mod git_assistant; // ✨ Phase 6: Git 智能助手
mod history; // ✨ Phase 8: 命令历史记录管理
mod i18n; // ✨ Phase 11: 多语言支持
mod likan; // ✨ Phase 4.3: 离坎炼化炉（自主学习循环）
mod liangyyi; // ✨ v1.9.0: 两仪演化系统（先天八卦·竖看·体）
mod llm;
mod llm_manager;
mod log_analyzer; // ✨ Phase 6: 日志分析工具
mod lunar_tool; // ✨ 农历工具：公历/农历转换、节气、干支生肖
mod memory;
mod path_resolver; // ✨ UX 改进：统一的配置文件路径搜索
mod project_context; // ✨ Phase 6: 项目上下文感知
mod repl; // REPL 循环（包含 Tab 补全集成）
mod services; // ✨ Phase 2: 服务层架构（v1.3.0）
mod shell_executor;
mod spinner;
mod stats; // ✨ Phase 9: 统计与可视化
mod suggestion; // ✨ Phase 4.1: 主动建议系统（三源融合）
mod system_monitor; // ✨ Phase 6: 系统监控工具
mod task; // ✨ Phase 10: 任务分解与规划系统
mod tool;
mod tool_cache; // ✨ Phase 5.3 Week 3 Day 2
mod tool_executor;
mod trace_context; // ✨ v1.5.1: 追踪上下文（TraceContext, ExecutionSpan）
mod tracer; // ✨ Phase 2 (Memory Redesign): 统一追踪系统
mod utils; // ✨ Phase 2: 软阈值工具（连续场重构）
mod voice; // ✨ 语音播报系统
mod wizard;

use clap::{Parser, Subcommand};
use colored::Colorize;
use display::Display;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

/// CLI 参数
#[derive(Parser, Debug)]
#[command(name = "realconsole")]
#[command(about = "融合东方哲学智慧的智能 CLI Agent", long_about = None)]
#[command(version)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "realconsole.yaml", global = true)]
    config: String,

    /// 单次执行模式
    #[arg(long)]
    once: Option<String>,

    /// 界面语言 (zh-CN, en-US)
    #[arg(short, long)]
    lang: Option<String>,

    /// 子命令
    #[command(subcommand)]
    command: Option<Commands>,
}

/// 子命令定义
#[derive(Subcommand, Debug)]
enum Commands {
    /// 运行配置向导（交互式配置生成）
    #[command(alias = "init")]
    Wizard {
        /// 快速配置模式（最小提问）
        #[arg(short, long)]
        quick: bool,
    },

    /// 显示当前配置
    Config {
        /// 显示配置路径
        #[arg(short, long)]
        path: bool,
    },
}

/// 根据配置创建 LLM 客户端
fn create_llm_client(
    provider_config: &config::LlmProvider,
) -> Result<Arc<dyn llm::LlmClient>, String> {
    match provider_config.provider.as_str() {
        "ollama" => {
            let model = provider_config.model.as_deref().unwrap_or("qwen2.5:latest");
            let endpoint = provider_config
                .endpoint
                .as_deref()
                .unwrap_or("http://localhost:11434");

            llm::OllamaClient::new(model, endpoint)
                .map(|client| Arc::new(client) as Arc<dyn llm::LlmClient>)
                .map_err(|e| {
                    format!(
                        "{}: {}",
                        i18n::t_with_args("llm.client_failed", &[("provider", "Ollama")]),
                        e
                    )
                })
        }
        "deepseek" => {
            let api_key = provider_config.api_key.as_ref().ok_or_else(|| {
                i18n::t_with_args("llm.need_api_key", &[("provider", "Deepseek")])
            })?;
            let model = provider_config.model.as_deref().unwrap_or("deepseek-chat");
            let endpoint = provider_config
                .endpoint
                .as_deref()
                .unwrap_or("https://api.deepseek.com/v1");

            llm::DeepseekClient::new(api_key, model, endpoint)
                .map(|client| Arc::new(client) as Arc<dyn llm::LlmClient>)
                .map_err(|e| {
                    format!(
                        "{}: {}",
                        i18n::t_with_args("llm.client_failed", &[("provider", "Deepseek")]),
                        e
                    )
                })
        }
        other => Err(format!("{} {}", i18n::t("llm.unknown_provider"), other)),
    }
}

/// 运行配置向导
async fn run_wizard(quick: bool) {
    use wizard::{ConfigWizard, WizardMode};

    println!("\n{}\n", i18n::t("config.wizard_title").cyan().bold());

    let mode = if quick {
        println!("{}\n", i18n::t("config.mode_quick").dimmed());
        WizardMode::Quick
    } else {
        println!("{}\n", i18n::t("config.mode_complete").dimmed());
        WizardMode::Complete
    };

    let wizard = ConfigWizard::new(mode);

    match wizard.run().await {
        Ok(config) => {
            if let Err(e) = wizard.generate_and_save(&config) {
                eprintln!("\n{} {}", i18n::t("config.save_failed").red(), e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\n{} {}", i18n::t("config.wizard_failed").red(), e);
            process::exit(1);
        }
    }
}

/// 显示当前配置
fn show_config(config_path: &str, show_path: bool) {
    if show_path {
        // 显示配置文件路径
        let abs_path =
            std::fs::canonicalize(config_path).unwrap_or_else(|_| PathBuf::from(config_path));
        println!("{}", abs_path.display());
        return;
    }

    // 显示配置内容
    if !std::path::Path::new(config_path).exists() {
        eprintln!("{} {}", i18n::t("config.not_found").red(), config_path);
        eprintln!("{}", i18n::t("config.run_wizard").cyan());
        process::exit(1);
    }

    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            println!(
                "\n{} {}\n",
                i18n::t("config.file_label").green().bold(),
                config_path
            );
            println!("{}", content);
        }
        Err(e) => {
            eprintln!("{} {}", i18n::t("config.read_failed").red(), e);
            process::exit(1);
        }
    }
}

/// 加载 .env 文件（支持多路径搜索）
///
/// # 搜索策略
/// 1. 配置文件所在目录
/// 2. 当前工作目录
/// 3. 用户配置目录 ~/.realconsole/
fn load_env_file(config_path: &str) {
    use path_resolver::PathResolver;

    // 策略 1: 配置文件所在目录（保持原有行为）
    let config_path_buf = PathBuf::from(config_path);
    let config_dir = config_path_buf
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let env_in_config_dir = config_dir.join(".env");

    if env_in_config_dir.exists() {
        if try_load_dotenv(&env_in_config_dir) {
            return;
        }
    }

    // 策略 2 & 3: 使用 PathResolver 搜索 .env 文件
    if let Some(env_path) = PathResolver::resolve(".env") {
        try_load_dotenv(&env_path);
    }
}

/// 尝试加载 .env 文件
fn try_load_dotenv(env_path: &std::path::Path) -> bool {
    match dotenvy::from_path(env_path) {
        Ok(_) => {
            // 只在 RUST_LOG 环境变量存在时显示（相当于 debug 模式）
            if std::env::var("RUST_LOG").is_ok() {
                Display::env_loaded(
                    display::DisplayMode::Debug,
                    &env_path.display().to_string(),
                );
            }
            true
        }
        Err(e) => {
            // 只在调试模式下显示错误，不影响主流程
            if std::env::var("RUST_LOG").is_ok() {
                eprintln!("{} {}", "⚠ .env 加载失败:".yellow(), e);
            }
            false
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // ✨ Phase 11: 初始化 i18n 系统
    // 语言选择优先级：命令行 > 配置文件 > 环境变量 > 系统语言 > 默认中文
    let selected_lang = if let Some(ref lang_str) = args.lang {
        lang_str.parse::<i18n::Language>().unwrap_or_else(|_| {
            eprintln!("⚠ 未知语言: {}，使用默认中文", lang_str);
            i18n::Language::ZhCn
        })
    } else if let Ok(lang_env) = std::env::var("REALCONSOLE_LANG") {
        lang_env.parse::<i18n::Language>().unwrap_or_else(|_| i18n::Language::from_system())
    } else {
        i18n::Language::from_system()
    };
    i18n::init(selected_lang);

    // 处理子命令
    if let Some(command) = args.command {
        match command {
            Commands::Wizard { quick } => {
                run_wizard(quick).await;
                return;
            }
            Commands::Config { path } => {
                show_config(&args.config, path);
                return;
            }
        }
    }

    // 首次运行检测：如果配置文件不存在，提示运行 wizard
    if !std::path::Path::new(&args.config).exists() && !std::path::Path::new(".env").exists() {
        println!("\n{}", i18n::t("first_run.welcome").green().bold());
        println!("\n{}", i18n::t("first_run.no_config").yellow());
        println!("\n{}\n", i18n::t("first_run.choose_one"));
        println!(
            "  1. {} {}",
            i18n::t("first_run.option1_cmd").cyan(),
            i18n::t("first_run.option1")
        );
        println!(
            "  2. {} {}",
            i18n::t("first_run.option2_cmd").cyan(),
            i18n::t("first_run.option2")
        );
        println!(
            "  3. {} {}\n",
            i18n::t("first_run.option3_hint").dimmed(),
            i18n::t("first_run.option3")
        );
        println!("{}\n", i18n::t("first_run.hint").dimmed());
        process::exit(0);
    }

    // 加载 .env 文件（如果存在）
    load_env_file(&args.config);

    // 加载配置
    let config = if std::path::Path::new(&args.config).exists() {
        match config::Config::from_file(&args.config) {
            Ok(cfg) => {
                // 使用配置中的显示模式
                Display::config_loaded(cfg.display.mode, &args.config);
                cfg
            }
            Err(e) => {
                // 使用用户友好的错误格式显示详细信息
                eprintln!("{}", e.format_user_friendly());
                eprintln!("\n{}", i18n::t("error.use_default_config").yellow());
                config::Config::default()
            }
        }
    } else {
        eprintln!("{} {}", i18n::t("config.not_found").yellow(), args.config);
        eprintln!("{}\n", i18n::t("config.run_wizard").cyan());
        process::exit(1);
    };

    // 创建命令注册表
    let mut registry = command::CommandRegistry::new();

    // 注册核心命令
    commands::register_core_commands(&mut registry);

    // 注册项目上下文命令（Phase 6）
    commands::register_project_commands(&mut registry);

    // 注册 Git 智能助手命令（Phase 6）
    commands::register_git_commands(&mut registry);

    // 注册日志文件分析命令（Phase 6）
    commands::register_log_analysis_commands(&mut registry);

    // 注册系统监控命令（Phase 6）
    commands::register_system_commands(&mut registry);

    // 创建 Agent
    let mut agent = agent::Agent::new(config.clone(), registry);

    // ✨ Phase 3 (v1.3.0-beta): 预先获取 StateManager 的所有引用
    // 避免后续与 agent.registry 的可变借用冲突
    let state_manager = agent.state_manager();
    let stats_collector = state_manager.stats_collector();
    let memory = state_manager.memory();
    let exec_logger = state_manager.exec_logger();
    let history = state_manager.history();

    // 注册统计命令（Phase 9） - 需要 stats_collector
    commands::register_stats_commands(&mut agent.registry, stats_collector);

    // 初始化 LLM 客户端
    {
        let mut manager = agent.llm_manager.write().await;

        // 初始化 primary LLM
        if let Some(ref primary_cfg) = config.llm.primary {
            match create_llm_client(primary_cfg) {
                Ok(client) => {
                    // 使用显示模式控制输出
                    Display::startup_llm(
                        config.display.mode,
                        "Primary",
                        client.model(),
                        &primary_cfg.provider,
                    );
                    manager.set_primary(client.clone());

                    // 如果是 Deepseek，同时设置 deepseek_client 用于流式输出
                    if primary_cfg.provider == "deepseek" {
                        if let Some(api_key) = &primary_cfg.api_key {
                            let model = primary_cfg.model.as_deref().unwrap_or("deepseek-chat");
                            let endpoint = primary_cfg
                                .endpoint
                                .as_deref()
                                .unwrap_or("https://api.deepseek.com/v1");
                            if let Ok(deepseek_client) =
                                llm::DeepseekClient::new(api_key, model, endpoint)
                            {
                                manager.set_deepseek(Arc::new(deepseek_client));
                            }
                        }
                    }
                }
                Err(e) => {
                    let msg = i18n::t_with_args(
                        "llm.init_failed",
                        &[("type", &i18n::t("llm.type_primary"))],
                    );
                    eprintln!("{} {}", msg.yellow(), e);
                }
            }
        }

        // 初始化 fallback LLM
        if let Some(ref fallback_cfg) = config.llm.fallback {
            match create_llm_client(fallback_cfg) {
                Ok(client) => {
                    // 使用显示模式控制输出
                    Display::startup_llm(
                        config.display.mode,
                        "Fallback",
                        client.model(),
                        &fallback_cfg.provider,
                    );
                    manager.set_fallback(client);
                }
                Err(e) => {
                    let msg = i18n::t_with_args(
                        "llm.init_failed",
                        &[("type", &i18n::t("llm.type_fallback"))],
                    );
                    eprintln!("{} {}", msg.yellow(), e);
                }
            }
        }
    }

    // ✨ Phase 7: 配置 LLM Pipeline 生成器（如果启用）
    agent.configure_llm_bridge();

    // ✨ Phase 8 (Workflow): 配置 Workflow 执行器（如果启用）
    agent.configure_workflow_executor();

    // ✨ Phase 4.1: 配置主动建议系统
    agent.configure_suggestion_engine();

    // ✨ Phase 4.3: 启动离坎炼化炉后台循环
    agent.start_likan_background_cycle();

    // ✨ Phase 4.3: 注册离坎炼化炉命令
    commands::register_likan_commands(
        &mut agent.registry,
        agent.likan_furnace.clone(),
        agent.likan_statusbar.clone(),
        agent.likan_trigger.clone(),
    );

    // 注册 LLM 命令（需要访问 agent 的 llm_manager）
    let llm_manager = agent.llm_manager();
    commands::register_llm_commands(&mut agent.registry, llm_manager);

    // 注册记忆管理命令（使用预先获取的 memory 引用）
    commands::register_memory_commands(&mut agent.registry, memory);

    // 注册执行日志命令（使用预先获取的 exec_logger 引用）
    commands::register_log_commands(&mut agent.registry, exec_logger.clone());

    // 注册 LLM 交互日志命令
    let llm_logger = agent.llm_logger();
    commands::register_llm_log_commands(&mut agent.registry, llm_logger.clone());

    // 注册工具管理命令（需要访问 agent 的 tool_registry）
    let tool_registry = agent.tool_registry();
    commands::register_tool_commands(&mut agent.registry, tool_registry);

    // ✨ Phase 8: 注册历史记录命令（使用预先获取的 history 引用）
    commands::register_history_commands(&mut agent.registry, history.clone());

    // ✨ Phase 对话上下文: 注册上下文管理命令
    let conversation_context = agent.state_manager().conversation_context();
    commands::register_context_commands(&mut agent.registry, conversation_context.clone());

    // ✨ Phase 2 (Memory Redesign): 注册统一追踪命令
    let unified_tracer = Arc::new(tracer::UnifiedTracer::new(
        history.clone(),
        exec_logger.clone(),
        llm_logger.clone(),
        conversation_context.clone(),
    ));
    // ✨ v1.5.1: 传递 trace_store 到 trace 命令
    let trace_store_clone = Arc::clone(&agent.trace_store);
    commands::register_trace_commands(&mut agent.registry, unified_tracer, trace_store_clone);

    // ✨ 注册语音播报命令
    let voice_broadcaster = agent.voice_broadcaster.clone();
    commands::register_voice_commands(&mut agent.registry, voice_broadcaster);

    // ✨ Phase 10: 注册任务分解与规划命令
    let llm_mgr_for_task = agent.llm_manager();
    let shell_exec_for_task = agent.shell_executor_with_fixer.clone();
    commands::register_task_commands(
        &mut agent.registry,
        llm_mgr_for_task,
        shell_exec_for_task,
        agent.config.clone(),
    );

    // 运行模式
    if let Some(input) = args.once {
        // 单次执行模式
        repl::run_once(&agent, &input);
    } else {
        // REPL 模式
        if let Err(e) = repl::run(&agent) {
            eprintln!("{} {:?}", i18n::t("error.repl_error").red(), e);
            process::exit(1);
        }
    }
}
