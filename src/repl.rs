//! REPL 循环实现
//!
//! 使用 rustyline 提供基础的 readline 功能
//! ✨ Phase 8: 集成命令历史记录和 Ctrl+R 搜索
//! ✨ Phase 11: 多语言支持
//! ✨ Phase 1: Tab 补全系统集成

use crate::agent::Agent;
use crate::completion::{CompletionConfig, MultiDimensionalCompleter};
use crate::history::SortStrategy;
use crate::i18n;
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Result as RustyResult};
use std::borrow::Cow;
use std::env;

/// REPL 退出信号
const QUIT_SIGNAL: &str = "__QUIT__";

/// RealConsole Helper - 集成补全、高亮、提示等功能
struct RealConsoleHelper {
    completer: MultiDimensionalCompleter,
}

impl RealConsoleHelper {
    fn new(agent: &Agent) -> Self {
        // 获取 LLM 客户端（用于智能补全）
        let llm_client = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // 从 llm_manager 获取 primary 客户端
                let manager = agent.llm_manager.read().await;
                manager.primary().cloned()
            })
        });

        let completer = MultiDimensionalCompleter::new(
            std::sync::Arc::new(agent.registry.clone()),
            agent.state_manager().history(),
            CompletionConfig::default(),
            llm_client, // 传递 LLM 客户端（可能为 None）
        );

        Self { completer }
    }
}

// 实现 rustyline 的各种 trait
impl rustyline::Helper for RealConsoleHelper {}

impl Completer for RealConsoleHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Highlighter for RealConsoleHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        // TODO: Phase 2 可实现语法高亮
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize, _forced: bool) -> bool {
        false
    }
}

impl Hinter for RealConsoleHelper {
    type Hint = String;

    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<String> {
        // TODO: Phase 3 可实现智能提示
        None
    }
}

impl Validator for RealConsoleHelper {}

/// 运行 REPL 循环
pub fn run(agent: &Agent) -> RustyResult<()> {
    // 创建 Helper（包含补全器）
    let helper = RealConsoleHelper::new(agent);

    // 创建 Editor 并设置 Helper
    let mut rl = Editor::new()?;
    rl.set_helper(Some(helper));

    // ✨ Phase 8: 配置历史记录行为（使用 Configurer trait）
    rl.set_max_history_size(1000)?; // 与 HistoryManager 的容量保持一致
    rl.set_history_ignore_dups(true)?; // 忽略连续重复
    rl.set_auto_add_history(true); // 自动添加历史

    // ✨ Phase 8: 从 HistoryManager 加载历史到 rustyline
    // 注意：rustyline 已经内置了 Ctrl+R 反向搜索功能
    load_history_to_editor(&mut rl, agent);

    // 显示欢迎信息
    print_welcome();

    loop {
        // 每次循环重新构建提示符，以反映当前目录和上下文状态
        let prompt = build_prompt(agent);

        // 读取输入
        let readline = rl.readline(&prompt);

        match readline {
            Ok(line) => {
                // 添加到历史记录
                let _ = rl.add_history_entry(line.as_str());

                // 处理输入
                let response = agent.handle(&line);

                // 检查退出信号
                if response == QUIT_SIGNAL {
                    println!("{}", "Bye 👋".cyan());
                    break;
                }

                // 显示响应（如果非空）
                if !response.is_empty() {
                    println!("{}", response);
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl-C
                println!("{}", i18n::t("command.interrupted").dimmed());
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl-D
                println!("{}", i18n::t("command.bye").cyan());
                break;
            }
            Err(err) => {
                eprintln!("{} {:?}", i18n::t("command.error").red(), err);
                break;
            }
        }
    }

    Ok(())
}

/// 打印欢迎信息
fn print_welcome() {
    let version = env!("CARGO_PKG_VERSION");
    // 极简单行显示：版本 | 用途 | 帮助 | 退出
    println!(
        " {} {} {} {} {} {} {}",
        i18n::t("welcome.app_name").bold().cyan(),
        i18n::t_with_args("welcome.version", &[("version", version)]).dimmed(),
        "|".dimmed(),
        i18n::t("welcome.hint").dimmed(),
        i18n::t("welcome.help").cyan(),
        "|".dimmed(),
        i18n::t("welcome.exit").dimmed()
    );
    // 去掉空行，让体验更接近普通 console
}

/// ✨ Phase 8: 从 HistoryManager 加载历史到 rustyline Editor
///
/// 这样用户可以使用 Ctrl+R 反向搜索历史命令
fn load_history_to_editor(
    rl: &mut Editor<RealConsoleHelper, rustyline::history::DefaultHistory>,
    agent: &Agent,
) {
    // 使用 tokio runtime 访问异步的 HistoryManager
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let history = agent.state_manager().history();
            let history_guard = history.read().await;

            // 获取所有历史记录（按时间排序，从旧到新）
            let entries = history_guard.all(SortStrategy::Time);

            // 将历史记录添加到 rustyline（从旧到新的顺序）
            for entry in entries.iter().rev() {
                // 只添加非空且非系统命令的记录
                if !entry.command.is_empty() && !entry.command.starts_with('/') {
                    let _ = rl.add_history_entry(&entry.command);
                }
            }
        })
    });
}

/// 构建标准的 shell 提示符
fn build_prompt(agent: &Agent) -> String {
    // 获取版本号并提取主版本号
    let version = env!("CARGO_PKG_VERSION");
    let major_version = version.split('.').next().unwrap_or("1");

    // 获取用户名
    let username = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    // 获取当前目录名（不是完整路径，只是目录名）
    let current_dir = env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "~".to_string());

    // ✨ Phase 对话上下文: 获取上下文状态
    let context_indicator = build_context_indicator(agent);

    // ✨ Phase 4.3: 获取离坎炼化炉提示符前缀
    let likan_prefix = agent
        .get_likan_prompt_prefix()
        .map(|prefix| format!("{} | ", prefix))
        .unwrap_or_default();

    // 构建提示符：[🌊🔥 8 | ](RealConsole v1) Username Pathname [上下文] %
    // 样式与欢迎信息保持一致：RealConsole 粗体青色，版本号灰色
    format!(
        "{}({} {}) {} {}{} % ",
        likan_prefix, // 离坎前缀（如果有）
        "RealConsole".bold().cyan(), // 粗体青色 RealConsole（与首行一致）
        format!("v{}", major_version).dimmed(), // 灰色版本号（与首行一致）
        username.truecolor(255, 165, 0), // 橙色用户名
        current_dir.truecolor(255, 165, 0), // 橙色目录名
        context_indicator // 上下文指示器
    )
}

/// ✨ Phase 对话上下文: 构建上下文状态指示器（优化版 - 避免死锁）
fn build_context_indicator(agent: &Agent) -> String {
    // 使用 block_in_place 来访问异步的 ContextManager
    // 但添加了 try_read 保护，避免在高负载时死锁
    let snapshot_opt = tokio::task::block_in_place(|| {
        let ctx_arc = agent.state_manager().conversation_context();

        tokio::runtime::Handle::current().block_on(async move {
            // 使用 try_read 而不是 read().await
            // 如果锁被占用（比如正在处理命令），直接返回 None
            // 这样可以避免 REPL 循环被阻塞
            match ctx_arc.try_read() {
                Ok(manager) => Some(manager.snapshot()),
                Err(_) => None,
            }
        })
    });

    // 如果无法获取锁，返回空字符串（安全降级）
    // 下一次循环会重新尝试获取状态
    let snapshot = match snapshot_opt {
        Some(s) => s,
        None => return String::new(),
    };

    // 检查是否激活
    if !snapshot.is_active || snapshot.turn_count == 0 {
        return String::new(); // 未激活或无轮次，不显示
    }

    // 构建指示器（使用快照数据）
    let idle_minutes = snapshot.idle_seconds / 60;

    if snapshot.is_near_timeout {
        // 即将超时：显示警告
        format!(
            " {}",
            format!("[上下文: {}轮 | {}分钟前]", snapshot.turn_count, idle_minutes).yellow()
        )
    } else if snapshot.idle_seconds > 60 {
        // 空闲超过 1 分钟：显示空闲时间
        format!(
            " {}",
            format!("[上下文: {}轮 | {}分钟前]", snapshot.turn_count, idle_minutes).dimmed()
        )
    } else {
        // 正常激活：只显示轮次
        format!(
            " {}",
            format!("[上下文: {}轮]", snapshot.turn_count).green()
        )
    }
}

/// 单次执行模式（--once）
pub fn run_once(agent: &Agent, input: &str) {
    let response = agent.handle(input);
    if !response.is_empty() && response != QUIT_SIGNAL {
        println!("{}", response);
    }
}
