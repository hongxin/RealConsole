//! RealConsole - 融合东方哲学智慧的智能 CLI Agent
//!
//! 基于 Rust 的智能 CLI 代理，结合易经智慧与现代 AI

mod advanced_tools;
mod agent;
mod builtin_tools;
mod command;
mod command_router;  // ✨ Phase 10.1: 智能命令路由系统
mod commands;
mod config;
mod conversation;  // ✨ Phase 8 Week 2: 多轮对话支持
mod display;
mod dsl;
mod error;
mod error_fixer;  // ✨ Phase 9.2: 错误自动修复
mod execution_logger;
mod git_assistant;  // ✨ Phase 6: Git 智能助手
mod history;        // ✨ Phase 8: 命令历史记录管理
mod llm;
mod llm_manager;
mod log_analyzer;  // ✨ Phase 6: 日志分析工具
mod memory;
mod project_context;  // ✨ Phase 6: 项目上下文感知
mod repl;
mod shell_executor;
mod spinner;
mod stats;  // ✨ Phase 9: 统计与可视化
mod system_monitor;  // ✨ Phase 6: 系统监控工具
mod task;  // ✨ Phase 10: 任务分解与规划系统
mod tool;
mod tool_cache;  // ✨ Phase 5.3 Week 3 Day 2
mod tool_executor;
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
            let model = provider_config
                .model
                .as_deref()
                .unwrap_or("qwen2.5:latest");
            let endpoint = provider_config
                .endpoint
                .as_deref()
                .unwrap_or("http://localhost:11434");

            llm::OllamaClient::new(endpoint, model)
                .map(|client| Arc::new(client) as Arc<dyn llm::LlmClient>)
                .map_err(|e| format!("Ollama 客户端创建失败: {}", e))
        }
        "deepseek" => {
            let api_key = provider_config
                .api_key
                .as_ref()
                .ok_or_else(|| "Deepseek 需要 api_key".to_string())?;
            let model = provider_config
                .model
                .as_deref()
                .unwrap_or("deepseek-chat");
            let endpoint = provider_config
                .endpoint
                .as_deref()
                .unwrap_or("https://api.deepseek.com/v1");

            llm::DeepseekClient::new(api_key, model, endpoint)
                .map(|client| Arc::new(client) as Arc<dyn llm::LlmClient>)
                .map_err(|e| format!("Deepseek 客户端创建失败: {}", e))
        }
        other => Err(format!("未知的 LLM provider: {}", other)),
    }
}

/// 运行配置向导
async fn run_wizard(quick: bool) {
    use wizard::{ConfigWizard, WizardMode};

    println!("\n{}", "=== RealConsole 配置向导 ===\n".cyan().bold());

    let mode = if quick {
        println!("{}", "模式: 快速配置（使用推荐默认值）\n".dimmed());
        WizardMode::Quick
    } else {
        println!("{}", "模式: 完整配置（可自定义所有选项）\n".dimmed());
        WizardMode::Complete
    };

    let wizard = ConfigWizard::new(mode);

    match wizard.run().await {
        Ok(config) => {
            if let Err(e) = wizard.generate_and_save(&config) {
                eprintln!("\n{} {}", "✗ 保存配置失败:".red(), e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("\n{} {}", "✗ 配置向导失败:".red(), e);
            process::exit(1);
        }
    }
}

/// 显示当前配置
fn show_config(config_path: &str, show_path: bool) {
    if show_path {
        // 显示配置文件路径
        let abs_path = std::fs::canonicalize(config_path)
            .unwrap_or_else(|_| PathBuf::from(config_path));
        println!("{}", abs_path.display());
        return;
    }

    // 显示配置内容
    if !std::path::Path::new(config_path).exists() {
        eprintln!("{} {}", "配置文件不存在:".red(), config_path);
        eprintln!("{}", "请运行 'realconsole wizard' 创建配置".cyan());
        process::exit(1);
    }

    match std::fs::read_to_string(config_path) {
        Ok(content) => {
            println!("\n{} {}\n", "配置文件:".green().bold(), config_path);
            println!("{}", content);
        }
        Err(e) => {
            eprintln!("{} {}", "读取配置文件失败:".red(), e);
            process::exit(1);
        }
    }
}

/// 尝试从配置文件所在目录加载 .env 文件
fn load_env_file(config_path: &str) {
    // 获取配置文件所在目录
    let config_path = PathBuf::from(config_path);
    let config_dir = config_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    let env_path = config_dir.join(".env");

    if env_path.exists() {
        match dotenvy::from_path(&env_path) {
            Ok(_) => {
                // 只在 RUST_LOG 环境变量存在时显示（相当于 debug 模式）
                if std::env::var("RUST_LOG").is_ok() {
                    Display::env_loaded(display::DisplayMode::Debug, &env_path.display().to_string());
                }
            }
            Err(e) => {
                // 只在调试模式下显示错误，不影响主流程
                if std::env::var("RUST_LOG").is_ok() {
                    eprintln!("{} {}", "⚠ .env 加载失败:".yellow(), e);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

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
    if !std::path::Path::new(&args.config).exists()
        && !std::path::Path::new(".env").exists() {
        println!("\n{}", "欢迎使用 RealConsole！".green().bold());
        println!("\n{}", "未检测到配置文件，首次使用需要进行配置。".yellow());
        println!("\n请选择以下方式之一：\n");
        println!("  1. {} 运行配置向导（推荐）", "realconsole wizard".cyan());
        println!("  2. {} 快速配置模式", "realconsole wizard --quick".cyan());
        println!("  3. {} 手动创建 realconsole.yaml 和 .env\n", "参考 config/minimal.yaml".dimmed());
        println!("{}", "提示: 向导将帮助你在 2 分钟内完成配置\n".dimmed());
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
                eprintln!("\n{}", "使用默认配置继续运行...".yellow());
                config::Config::default()
            }
        }
    } else {
        eprintln!("{} {}", "配置文件不存在:".yellow(), args.config);
        eprintln!("{}", "请运行 'realconsole wizard' 创建配置\n".cyan());
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

    // 注册统计命令（Phase 9） - 需要 stats_collector
    let stats_collector = agent.stats_collector();
    commands::register_stats_commands(&mut agent.registry, stats_collector);

    // 初始化 LLM 客户端
    {
        let mut manager = agent.llm_manager.write().await;

        // 初始化 primary LLM
        if let Some(ref primary_cfg) = config.llm.primary {
            match create_llm_client(primary_cfg) {
                Ok(client) => {
                    // 使用显示模式控制输出
                    Display::startup_llm(config.display.mode, "Primary", client.model(), &primary_cfg.provider);
                    manager.set_primary(client.clone());

                    // 如果是 Deepseek，同时设置 deepseek_client 用于流式输出
                    if primary_cfg.provider == "deepseek" {
                        if let Some(api_key) = &primary_cfg.api_key {
                            let model = primary_cfg.model.as_deref().unwrap_or("deepseek-chat");
                            let endpoint = primary_cfg.endpoint.as_deref().unwrap_or("https://api.deepseek.com/v1");
                            if let Ok(deepseek_client) = llm::DeepseekClient::new(api_key, model, endpoint) {
                                manager.set_deepseek(Arc::new(deepseek_client));
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "⚠ Primary LLM 初始化失败:".yellow(), e);
                }
            }
        }

        // 初始化 fallback LLM
        if let Some(ref fallback_cfg) = config.llm.fallback {
            match create_llm_client(fallback_cfg) {
                Ok(client) => {
                    // 使用显示模式控制输出
                    Display::startup_llm(config.display.mode, "Fallback", client.model(), &fallback_cfg.provider);
                    manager.set_fallback(client);
                }
                Err(e) => {
                    eprintln!("{} {}", "⚠ Fallback LLM 初始化失败:".yellow(), e);
                }
            }
        }
    }

    // ✨ Phase 7: 配置 LLM Pipeline 生成器（如果启用）
    agent.configure_llm_bridge();

    // 注册 LLM 命令（需要访问 agent 的 llm_manager）
    let llm_manager = agent.llm_manager();
    commands::register_llm_commands(&mut agent.registry, llm_manager);

    // 注册记忆管理命令（需要访问 agent 的 memory）
    let memory = agent.memory();
    commands::register_memory_commands(&mut agent.registry, memory);

    // 注册执行日志命令（需要访问 agent 的 exec_logger）
    let exec_logger = agent.exec_logger();
    commands::register_log_commands(&mut agent.registry, exec_logger);

    // 注册工具管理命令（需要访问 agent 的 tool_registry）
    let tool_registry = agent.tool_registry();
    commands::register_tool_commands(&mut agent.registry, tool_registry);

    // ✨ Phase 8: 注册历史记录命令（需要访问 agent 的 history）
    let history = agent.history();
    commands::register_history_commands(&mut agent.registry, history);

    // ✨ Phase 10: 注册任务分解与规划命令
    let llm_mgr_for_task = agent.llm_manager();
    let shell_exec_for_task = agent.shell_executor_with_fixer.clone();
    commands::register_task_commands(&mut agent.registry, llm_mgr_for_task, shell_exec_for_task);

    // 运行模式
    if let Some(input) = args.once {
        // 单次执行模式
        repl::run_once(&agent, &input);
    } else {
        // REPL 模式
        if let Err(e) = repl::run(&agent) {
            eprintln!("{} {:?}", "REPL 错误:".red(), e);
            process::exit(1);
        }
    }
}
