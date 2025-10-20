//! REPL 循环实现
//!
//! 使用 rustyline 提供基础的 readline 功能
//! ✨ Phase 8: 集成命令历史记录和 Ctrl+R 搜索
//! ✨ Phase 11: 多语言支持

use crate::agent::Agent;
use crate::history::SortStrategy;
use crate::i18n;
use colored::Colorize;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustyResult};
use std::env;

/// REPL 退出信号
const QUIT_SIGNAL: &str = "__QUIT__";

/// 运行 REPL 循环
pub fn run(agent: &Agent) -> RustyResult<()> {
    let mut rl = DefaultEditor::new()?;

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
        // 每次循环重新构建提示符，以反映当前目录
        let prompt = build_prompt();

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
fn load_history_to_editor(rl: &mut DefaultEditor, agent: &Agent) {
    // 使用 tokio runtime 访问异步的 HistoryManager
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let history = agent.history();
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
fn build_prompt() -> String {
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

    // 构建提示符：(RealConsole v1) Username Pathname %
    // 样式与欢迎信息保持一致：RealConsole 粗体青色，版本号灰色
    format!(
        "({} {}) {} {} % ",
        "RealConsole".bold().cyan(), // 粗体青色 RealConsole（与首行一致）
        format!("v{}", major_version).dimmed(), // 灰色版本号（与首行一致）
        username.truecolor(255, 165, 0), // 橙色用户名
        current_dir.truecolor(255, 165, 0)  // 橙色目录名
    )
}

/// 单次执行模式（--once）
pub fn run_once(agent: &Agent, input: &str) {
    let response = agent.handle(input);
    if !response.is_empty() && response != QUIT_SIGNAL {
        println!("{}", response);
    }
}
