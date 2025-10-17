//! 执行日志管理命令
//!
//! 提供执行日志查看、搜索、统计等功能

use crate::command::{Command, CommandRegistry};
use crate::execution_logger::{CommandType, ExecutionLogger};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册日志管理命令
pub fn register_log_commands(registry: &mut CommandRegistry, logger: Arc<RwLock<ExecutionLogger>>) {
    // /log 命令
    let log_cmd = Command::from_fn(
        "log",
        "执行日志: log [recent|search|stats|type|failed|clear]",
        move |arg: &str| handle_log(arg, Arc::clone(&logger)),
    )
    .with_group("log");
    registry.register(log_cmd);
}

/// 处理 /log 命令
fn handle_log(arg: &str, logger: Arc<RwLock<ExecutionLogger>>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_log_recent("10", logger);
    }

    let subcommand = parts[0];
    let rest = parts.get(1..).unwrap_or(&[]).join(" ");

    match subcommand {
        "recent" | "r" => handle_log_recent(&rest, logger),
        "search" | "s" => handle_log_search(&rest, logger),
        "stats" | "st" => handle_log_stats(&rest, logger),
        "type" | "t" => handle_log_type(&rest, logger),
        "failed" | "f" => handle_log_failed(logger),
        "success" | "ok" => handle_log_success(logger),
        "clear" | "c" => handle_log_clear(logger),
        "help" | "h" => log_help(),
        _ => format!(
            "{} 未知子命令: {}\n使用 /log help 查看帮助",
            "错误:".red(),
            subcommand
        ),
    }
}

/// 查看最近的执行日志
fn handle_log_recent(arg: &str, logger: Arc<RwLock<ExecutionLogger>>) -> String {
    let n: usize = if arg.is_empty() {
        10
    } else {
        arg.trim().parse().unwrap_or(10)
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;
            let entries = log.recent(n);

            if entries.is_empty() {
                return format!("{}", "暂无执行日志".dimmed());
            }

            let mut lines = vec![format!(
                "{} {} 条执行日志:",
                "最近".bold().cyan(),
                entries.len().to_string().green()
            )];

            for entry in entries {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 搜索执行日志
fn handle_log_search(keyword: &str, logger: Arc<RwLock<ExecutionLogger>>) -> String {
    if keyword.is_empty() {
        return format!("{} 请提供搜索关键词", "错误:".red());
    }

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;
            let results = log.search(keyword);

            if results.is_empty() {
                return format!(
                    "{} 未找到包含 {} 的日志",
                    "提示:".yellow(),
                    keyword.cyan()
                );
            }

            let mut lines = vec![format!(
                "{} {} 条结果 (关键词: {}):",
                "找到".bold().green(),
                results.len().to_string().green(),
                keyword.cyan()
            )];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 查看执行统计
fn handle_log_stats(arg: &str, logger: Arc<RwLock<ExecutionLogger>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;

            if arg.is_empty() {
                // 全局统计
                let stats = log.stats();
                format!("{}\n{}", "执行统计".bold().cyan(), stats.format())
            } else {
                // 按类型统计
                let command_type = match arg.to_lowercase().as_str() {
                    "command" | "cmd" => CommandType::Command,
                    "shell" | "sh" => CommandType::Shell,
                    "text" => CommandType::Text,
                    _ => {
                        return format!(
                            "{} 未知类型: {}\n支持的类型: command, shell, text",
                            "错误:".red(),
                            arg
                        );
                    }
                };

                let stats = log.stats_by_type(command_type);
                format!(
                    "{} {} 执行统计\n{}",
                    command_type.to_string().bold().cyan(),
                    "类型".dimmed(),
                    stats.format()
                )
            }
        })
    })
}

/// 按类型过滤日志
fn handle_log_type(type_str: &str, logger: Arc<RwLock<ExecutionLogger>>) -> String {
    if type_str.is_empty() {
        return format!(
            "{} 请指定类型\n支持的类型: command, shell, text",
            "错误:".red()
        );
    }

    let command_type = match type_str.to_lowercase().as_str() {
        "command" | "cmd" | "c" => CommandType::Command,
        "shell" | "sh" | "s" => CommandType::Shell,
        "text" | "t" => CommandType::Text,
        _ => {
            return format!(
                "{} 未知类型: {}\n支持的类型: command, shell, text",
                "错误:".red(),
                type_str
            );
        }
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;
            let results = log.filter_by_type(command_type);

            if results.is_empty() {
                return format!(
                    "{} 未找到类型为 {} 的日志",
                    "提示:".yellow(),
                    command_type
                );
            }

            let mut lines = vec![format!(
                "{} {} 条 {} 日志:",
                "找到".bold().green(),
                results.len().to_string().green(),
                command_type
            )];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 查看失败的执行日志
fn handle_log_failed(logger: Arc<RwLock<ExecutionLogger>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;
            let results = log.failed();

            if results.is_empty() {
                return format!("{} 没有失败的执行记录", "✓".green());
            }

            let mut lines = vec![format!(
                "{} {} 条失败记录:",
                "失败".bold().red(),
                results.len().to_string().red()
            )];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 查看成功的执行日志
fn handle_log_success(logger: Arc<RwLock<ExecutionLogger>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let log = logger.read().await;
            let results = log.successful();

            if results.is_empty() {
                return format!("{}", "暂无成功记录".dimmed());
            }

            let mut lines = vec![format!(
                "{} {} 条成功记录:",
                "成功".bold().green(),
                results.len().to_string().green()
            )];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 清空执行日志
fn handle_log_clear(logger: Arc<RwLock<ExecutionLogger>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut log = logger.write().await;
            let count = log.len();
            log.clear();
            format!("{} 已清空 {} 条执行日志", "✓".green(), count.to_string().dimmed())
        })
    })
}

/// 日志命令帮助
fn log_help() -> String {
    format!(
        r#"{title}

{subtitle}
  /log                 - 查看最近 10 条执行日志
  /log recent <n>      - 查看最近 N 条日志（默认 10）
  /log search <关键词>  - 搜索包含关键词的日志
  /log type <类型>      - 按类型过滤（command/shell/text）
  /log stats [类型]     - 查看统计信息（可选按类型）
  /log failed          - 查看失败的执行记录
  /log success         - 查看成功的执行记录
  /log clear           - 清空所有日志

{examples}
  /log recent 20
  /log search rust
  /log type shell
  /log stats command
  /log failed

{shortcuts}
  recent → r, search → s, stats → st, type → t, failed → f, success → ok, clear → c
"#,
        title = "执行日志管理".bold().cyan(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        shortcuts = "快捷命令:".dimmed()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_logger::ExecutionLogger;
    use std::time::Duration;

    fn create_test_logger() -> Arc<RwLock<ExecutionLogger>> {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(50),
            "Help message",
        );
        logger.log(
            "!ls".to_string(),
            CommandType::Shell,
            false,
            Duration::from_millis(100),
            "error",
        );
        logger.log(
            "你好".to_string(),
            CommandType::Text,
            true,
            Duration::from_millis(500),
            "你好！",
        );

        Arc::new(RwLock::new(logger))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_recent() {
        let logger = create_test_logger();
        let result = handle_log_recent("2", logger);
        assert!(result.contains("最近"));
        assert!(result.contains("你好"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_search() {
        let logger = create_test_logger();
        let result = handle_log_search("help", logger);
        assert!(result.contains("找到"));
        assert!(result.contains("/help"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_failed() {
        let logger = create_test_logger();
        let result = handle_log_failed(logger);
        assert!(result.contains("失败"));
        assert!(result.contains("!ls"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_clear() {
        let logger = create_test_logger();
        let result = handle_log_clear(Arc::clone(&logger));
        assert!(result.contains("已清空 3 条"));

        let log = logger.read().await;
        assert_eq!(log.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_stats() {
        let logger = create_test_logger();
        let result = handle_log_stats("", logger);
        assert!(result.contains("执行统计"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_stats_by_type() {
        let logger = create_test_logger();
        let result = handle_log_stats("shell", logger);
        assert!(result.contains("Shell") || result.contains("执行统计"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_stats_invalid_type() {
        let logger = create_test_logger();
        let result = handle_log_stats("invalid", logger);
        assert!(result.contains("未知类型"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_type_command() {
        let logger = create_test_logger();
        let result = handle_log_type("command", logger);
        assert!(result.contains("找到") || result.contains("Command"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_type_empty() {
        let logger = create_test_logger();
        let result = handle_log_type("", logger);
        assert!(result.contains("请指定类型"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_type_invalid() {
        let logger = create_test_logger();
        let result = handle_log_type("invalid", logger);
        assert!(result.contains("未知类型"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_success() {
        let logger = create_test_logger();
        let result = handle_log_success(logger);
        assert!(result.contains("成功"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_search_no_keyword() {
        let logger = create_test_logger();
        let result = handle_log_search("", logger);
        assert!(result.contains("错误"));
        assert!(result.contains("关键词"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_log_search_no_results() {
        let logger = create_test_logger();
        let result = handle_log_search("nonexistent", logger);
        assert!(result.contains("未找到"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_log_with_empty_args() {
        let logger = create_test_logger();
        let result = handle_log("", logger);
        // 应该返回最近 10 条日志
        assert!(result.contains("最近") || result.contains("执行日志"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_log_unknown_subcommand() {
        let logger = create_test_logger();
        let result = handle_log("unknown", logger);
        assert!(result.contains("未知子命令"));
    }

    #[test]
    fn test_log_help() {
        let result = log_help();
        assert!(result.contains("执行日志管理"));
        assert!(result.contains("recent"));
        assert!(result.contains("search"));
        assert!(result.contains("stats"));
    }
}
