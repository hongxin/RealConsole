//! /context 命令实现
//!
//! 用法：
//! - `/context` - 显示帮助信息
//! - `/context start` - 启动上下文（Manual 模式）
//! - `/context stop` - 停止上下文
//! - `/context show` - 显示当前上下文内容
//! - `/context status` - 显示状态信息
//! - `/context clear` - 清除上下文（保持激活状态）

use crate::command::{Command, CommandRegistry};
use crate::config::ContextMode;
use crate::conversation::ContextManager;
use chrono::Local;
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册上下文命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `context_manager`: 共享的上下文管理器
pub fn register_context_commands(
    registry: &mut CommandRegistry,
    context_manager: Arc<RwLock<ContextManager>>,
) {
    let cmd = Command::from_fn("context", "对话上下文管理", move |args| {
        handle_context(args, Arc::clone(&context_manager))
    })
    .with_group("context");

    registry.register(cmd);
}

/// 处理 /context 命令
fn handle_context(args: &str, context_manager: Arc<RwLock<ContextManager>>) -> String {
    let args_str = args.trim();

    // 使用 tokio runtime 处理异步锁
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            if args_str.is_empty() {
                // 显示帮助
                show_help(&context_manager).await
            } else if args_str == "start" {
                // 启动上下文
                start_context(&context_manager).await
            } else if args_str == "stop" {
                // 停止上下文
                stop_context(&context_manager).await
            } else if args_str == "show" {
                // 显示上下文内容
                show_context(&context_manager).await
            } else if args_str == "status" {
                // 显示状态信息
                show_status(&context_manager).await
            } else if args_str == "clear" {
                // 清除上下文
                clear_context(&context_manager).await
            } else {
                // 未知子命令
                format!(
                    "{} {}\n\n{}",
                    "未知子命令:".red(),
                    args_str.yellow(),
                    show_help_text()
                )
            }
        })
    })
}

/// 显示帮助信息
async fn show_help(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let manager = context_manager.read().await;
    let mode = manager.mode();

    let mut output = Vec::new();
    output.push(format!("{}", "对话上下文管理".bold().cyan()));
    output.push("".to_string());

    // 显示当前模式
    output.push(format!(
        "{} {}",
        "当前模式:".dimmed(),
        format!("{:?}", mode).yellow()
    ));
    output.push("".to_string());

    // 显示帮助文本
    output.push(show_help_text());

    output.join("\n")
}

/// 帮助文本
fn show_help_text() -> String {
    format!(
        "{}\n  {} - 显示此帮助\n  {} - 启动上下文（Manual 模式）\n  {} - 停止上下文\n  {} - 显示当前上下文内容\n  {} - 显示状态信息\n  {} - 清除上下文（保持激活）",
        "用法:".yellow(),
        "/context".green(),
        "/context start".green(),
        "/context stop".green(),
        "/context show".green(),
        "/context status".green(),
        "/context clear".green()
    )
}

/// 启动上下文
async fn start_context(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let mut manager = context_manager.write().await;

    let mode = manager.mode();

    // 检查模式
    if mode == ContextMode::Disabled {
        return format!(
            "{} 当前模式为 {}，无法手动启动上下文\n{} 请在配置文件中将 mode 设置为 {} 或 {}",
            "⚠️".yellow(),
            "Disabled".yellow(),
            "提示:".dimmed(),
            "Manual".green(),
            "Auto".green()
        );
    }

    // 检查是否已激活
    if manager.is_active() {
        return format!(
            "{} 上下文已处于激活状态\n{} 当前轮次数: {}",
            "ℹ️".cyan(),
            "状态:".dimmed(),
            manager.turn_count().to_string().yellow()
        );
    }

    // 启动上下文
    manager.start();

    format!(
        "{} 上下文已启动\n{} 模式: {}\n{} 最大轮次: {}",
        "✓".green(),
        "模式:".dimmed(),
        format!("{:?}", mode).yellow(),
        "限制:".dimmed(),
        manager.config().max_turns.to_string().yellow()
    )
}

/// 停止上下文
async fn stop_context(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let mut manager = context_manager.write().await;

    let mode = manager.mode();

    // 检查模式
    if mode == ContextMode::Disabled {
        return format!(
            "{} 当前模式为 {}，无上下文运行",
            "ℹ️".cyan(),
            "Disabled".yellow()
        );
    }

    // 检查是否已停止
    if !manager.is_active() {
        return format!("{} 上下文已处于停止状态", "ℹ️".cyan());
    }

    // 记录停止前的统计
    let turn_count = manager.turn_count();
    let context_length = manager.context_length();

    // 停止上下文
    manager.stop();

    format!(
        "{} 上下文已停止\n{} 已清除 {} 轮对话（{} 字符）",
        "✓".green(),
        "统计:".dimmed(),
        turn_count.to_string().yellow(),
        context_length.to_string().yellow()
    )
}

/// 显示上下文内容
async fn show_context(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let manager = context_manager.read().await;

    let turns = manager.turns();

    if turns.is_empty() {
        return format!("{} 当前无上下文", "ℹ️".cyan());
    }

    let mut output = Vec::new();
    output.push(format!(
        "{} ({} 轮)",
        "当前上下文".bold().cyan(),
        turns.len().to_string().yellow()
    ));
    output.push("".to_string());

    for (index, turn) in turns.iter().enumerate() {
        output.push(format!(
            "{} {}",
            format!("[轮次 {}]", index + 1).bold().blue(),
            Local::now().format("%H:%M:%S").to_string().dimmed()
        ));

        // 用户输入
        let user_preview = if turn.user_input.len() > 60 {
            format!("{}...", &turn.user_input[..60])
        } else {
            turn.user_input.clone()
        };
        output.push(format!("  {} {}", "👤".to_string(), user_preview.white()));

        // AI 响应
        let assistant_preview = if turn.assistant_response.len() > 60 {
            format!("{}...", &turn.assistant_response[..60])
        } else {
            turn.assistant_response.clone()
        };
        output.push(format!(
            "  {} {}",
            "🤖".to_string(),
            assistant_preview.dimmed()
        ));

        output.push("".to_string());
    }

    output.join("\n")
}

/// 显示状态信息
async fn show_status(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let manager = context_manager.read().await;

    let mode = manager.mode();
    let is_active = manager.is_active();
    let turn_count = manager.turn_count();
    let context_length = manager.context_length();
    let idle_seconds = manager.idle_seconds();

    let mut output = Vec::new();
    output.push(format!("{}", "上下文状态".bold().cyan()));
    output.push("".to_string());

    // 模式
    output.push(format!(
        "{} {}",
        "模式:".dimmed(),
        format!("{:?}", mode).yellow()
    ));

    // 激活状态
    let status_icon = if is_active { "🟢" } else { "🔴" };
    let status_text = if is_active {
        "激活".green()
    } else {
        "未激活".red()
    };
    output.push(format!(
        "{} {} {}",
        "状态:".dimmed(),
        status_icon,
        status_text
    ));

    // 轮次数
    output.push(format!(
        "{} {} / {}",
        "轮次:".dimmed(),
        turn_count.to_string().yellow(),
        manager.config().max_turns.to_string().dimmed()
    ));

    // 上下文长度
    output.push(format!(
        "{} {} / {} 字符",
        "长度:".dimmed(),
        context_length.to_string().yellow(),
        manager.config().max_context_length.to_string().dimmed()
    ));

    // 空闲时间
    if is_active && turn_count > 0 {
        let idle_minutes = idle_seconds / 60;
        let idle_display = if idle_minutes > 0 {
            format!("{} 分钟前", idle_minutes)
        } else {
            format!("{} 秒前", idle_seconds)
        };

        output.push(format!(
            "{} {}",
            "最后活动:".dimmed(),
            idle_display.yellow()
        ));

        // 空闲警告
        if manager.is_near_timeout() {
            let timeout = manager.config().auto_clear.idle_timeout / 60;
            output.push("".to_string());
            output.push(format!(
                "{} 上下文即将超时（{} 分钟未活动将自动清除）",
                "⚠️".yellow(),
                timeout.to_string().yellow()
            ));
        }
    }

    output.join("\n")
}

/// 清除上下文
async fn clear_context(context_manager: &Arc<RwLock<ContextManager>>) -> String {
    let mut manager = context_manager.write().await;

    let mode = manager.mode();

    // 检查模式
    if mode == ContextMode::Disabled {
        return format!(
            "{} 当前模式为 {}，无上下文运行",
            "ℹ️".cyan(),
            "Disabled".yellow()
        );
    }

    // 记录清除前的统计
    let turn_count = manager.turn_count();
    let context_length = manager.context_length();

    if turn_count == 0 {
        return format!("{} 当前无上下文可清除", "ℹ️".cyan());
    }

    // 清除上下文（但保持激活状态）
    manager.clear();

    format!(
        "{} 上下文已清除\n{} 已清除 {} 轮对话（{} 字符）\n{} 上下文仍处于 {} 状态",
        "✓".green(),
        "统计:".dimmed(),
        turn_count.to_string().yellow(),
        context_length.to_string().yellow(),
        "提示:".dimmed(),
        if manager.is_active() {
            "激活".green()
        } else {
            "未激活".red()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AutoClearConfig, ContextIncludeConfig, ConversationConfig};
    use crate::conversation::Turn;

    fn create_test_manager() -> Arc<RwLock<ContextManager>> {
        let config = ConversationConfig {
            mode: ContextMode::Manual,
            max_turns: 5,
            max_context_length: 1000,
            auto_clear: AutoClearConfig {
                enabled: true,
                idle_timeout: 300,
                on_task_complete: false,
            },
            include: ContextIncludeConfig {
                tool_calls: false,
                shell_output: false,
                errors: true,
            },
        };

        Arc::new(RwLock::new(ContextManager::new(config)))
    }

    #[tokio::test]
    async fn test_context_start() {
        let manager = create_test_manager();
        let result = start_context(&manager).await;

        assert!(result.contains("上下文已启动"));
        assert!(manager.read().await.is_active());
    }

    #[tokio::test]
    async fn test_context_stop() {
        let manager = create_test_manager();

        // 先启动
        start_context(&manager).await;

        // 再停止
        let result = stop_context(&manager).await;

        assert!(result.contains("上下文已停止"));
        assert!(!manager.read().await.is_active());
    }

    #[tokio::test]
    async fn test_context_clear() {
        let manager = create_test_manager();

        // 启动并添加轮次
        {
            let mut mgr = manager.write().await;
            mgr.start();
            mgr.add_turn(Turn::new("hello".to_string(), "hi".to_string()));
        }

        let result = clear_context(&manager).await;

        assert!(result.contains("上下文已清除"));
        assert_eq!(manager.read().await.turn_count(), 0);
        // Manual 模式下，clear 后仍然激活
        assert!(manager.read().await.is_active());
    }

    #[tokio::test]
    async fn test_context_show_empty() {
        let manager = create_test_manager();
        let result = show_context(&manager).await;

        assert!(result.contains("当前无上下文"));
    }

    #[tokio::test]
    async fn test_context_show_with_turns() {
        let manager = create_test_manager();

        // 添加轮次
        {
            let mut mgr = manager.write().await;
            mgr.start();
            mgr.add_turn(Turn::new("hello".to_string(), "hi there".to_string()));
        }

        let result = show_context(&manager).await;

        assert!(result.contains("当前上下文"));
        assert!(result.contains("1 轮"));
    }

    #[tokio::test]
    async fn test_context_status() {
        let manager = create_test_manager();

        let result = show_status(&manager).await;

        assert!(result.contains("上下文状态"));
        assert!(result.contains("模式"));
        assert!(result.contains("状态"));
        assert!(result.contains("轮次"));
    }
}
