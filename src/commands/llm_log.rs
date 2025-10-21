//! /llm-log 命令实现
//!
//! 用于查看和管理 LLM 交互日志
//!
//! 用法：
//! - `/llm-log status` - 显示日志状态
//! - `/llm-log recent [n]` - 查看最近 N 条日志
//! - `/llm-log enable` - 启用日志记录
//! - `/llm-log disable` - 禁用日志记录

use crate::command::{Command, CommandRegistry};
use crate::llm::LlmLogger;
use colored::Colorize;
use std::fs;
use std::sync::Arc;

/// 注册 LLM 日志命令
pub fn register_llm_log_commands(
    registry: &mut CommandRegistry,
    logger: Option<Arc<LlmLogger>>,
) {
    let cmd = Command::from_fn(
        "llm-log",
        "LLM 交互日志管理",
        move |args| handle_llm_log(args, logger.as_ref().map(Arc::clone)),
    )
    .with_group("log");

    registry.register(cmd);
}

/// 处理 /llm-log 命令
fn handle_llm_log(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() {
        return show_help();
    }

    let subcommand = parts[0];
    let rest = parts.get(1..).unwrap_or(&[]).join(" ");

    match subcommand {
        "status" => handle_status(logger),
        "recent" => handle_recent(&rest, logger),
        "search" => handle_search(&rest, logger),
        "stats" => handle_stats(&rest, logger),
        "clean" => handle_clean(&rest, logger),
        "replay" => handle_replay(&rest, logger),
        "sessions" => handle_sessions(&rest, logger),
        "enable" => handle_enable(logger),
        "disable" => handle_disable(logger),
        "help" | "h" => show_help(),
        _ => format!(
            "{} 未知子命令: {}\n使用 /llm-log help 查看帮助",
            "错误:".red(),
            subcommand
        ),
    }
}

/// 显示帮助信息
fn show_help() -> String {
    format!(
        r#"{title}

{subtitle}
  /llm-log status          - 显示日志状态和统计信息
  /llm-log recent [n]      - 查看最近 N 条日志（默认 10）
  /llm-log search <keyword> [--days N] - 搜索日志（支持时间范围）
  /llm-log stats [--days N] - 显示统计报告（默认全部）
  /llm-log clean <days>    - 清理 N 天前的日志
  /llm-log sessions [n]    - 列出最近的会话（默认 20）
  /llm-log replay <session_id> - 回放指定会话的完整交互
  /llm-log enable          - 启用日志记录（暂不可用）
  /llm-log disable         - 禁用日志记录（暂不可用）

{examples}
  /llm-log status
  /llm-log recent 20
  /llm-log search "错误" --days 7
  /llm-log stats --days 7
  /llm-log clean 30
  /llm-log sessions 10
  /llm-log replay 06dcbee2-4cf1-49d7-9788-a2a4843cfa28

{note}
  LLM 日志功能需要在配置文件中启用：
  llm:
    logging:
      enabled: true
"#,
        title = "LLM 交互日志管理".bold().cyan(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        note = "注意:".dimmed()
    )
}

/// 显示日志状态
fn handle_status(logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            let log_dir = logger.log_dir();

            // 统计日志文件
            let (file_count, total_size, entry_count) = count_log_files(log_dir);

            let mut lines = vec![
                format!("{}", "LLM 日志状态".bold().cyan()),
                String::new(),
                format!("{} {}", "状态:".dimmed(), "启用".green()),
                format!("{} {}", "日志目录:".dimmed(), log_dir.display().to_string().cyan()),
                String::new(),
                format!("{}", "统计信息:".bold()),
                format!("  文件数量: {}", file_count.to_string().green()),
                format!("  总大小: {} KB", (total_size / 1024).to_string().yellow()),
                format!("  总条目: {} 条（估算）", entry_count.to_string().green()),
            ];

            // 显示最新的日志文件
            if let Some(latest_file) = get_latest_log_file(log_dir) {
                lines.push(String::new());
                lines.push(format!("{}", "最新日志文件:".dimmed()));
                lines.push(format!("  {}", latest_file.display().to_string().cyan()));
            }

            lines.join("\n")
        }
        None => {
            format!(
                "{} LLM 日志功能未启用\n\n{}\n  {}",
                "ℹ️".cyan(),
                "要启用此功能，请在配置文件中设置:".dimmed(),
                "llm.logging.enabled: true".cyan()
            )
        }
    }
}

/// 查看最近的日志
fn handle_recent(arg: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            let n: usize = arg.trim().parse().unwrap_or(10);
            let log_dir = logger.log_dir();

            // 获取最新的日志文件
            let latest_file = match get_latest_log_file(log_dir) {
                Some(f) => f,
                None => {
                    return format!("{} 未找到日志文件", "提示:".yellow());
                }
            };

            // 读取最后 N 行
            match read_last_n_lines(&latest_file, n) {
                Ok(lines) => {
                    if lines.is_empty() {
                        return format!("{} 日志文件为空", "提示:".yellow());
                    }

                    let mut output = vec![
                        format!(
                            "{} {} 条日志:",
                            "最近".bold().cyan(),
                            lines.len().to_string().green()
                        ),
                        String::new(),
                    ];

                    for (idx, line) in lines.iter().enumerate() {
                        // 尝试解析 JSON
                        if let Ok(log) = serde_json::from_str::<serde_json::Value>(line) {
                            output.push(format!("[{}] {}", idx + 1, format_log_entry(&log)));
                        } else {
                            output.push(format!("[{}] {}", idx + 1, line.dimmed()));
                        }
                        output.push(String::new());
                    }

                    output.join("\n")
                }
                Err(e) => format!("{} 读取日志失败: {}", "错误:".red(), e),
            }
        }
        None => format!(
            "{} LLM 日志功能未启用",
            "ℹ️".cyan()
        ),
    }
}

/// 启用日志（占位符）
fn handle_enable(_logger: Option<Arc<LlmLogger>>) -> String {
    format!(
        "{} 动态启用功能暂未实现\n{} 请在配置文件中设置 llm.logging.enabled: true 并重启",
        "⚠️".yellow(),
        "提示:".dimmed()
    )
}

/// 禁用日志（占位符）
fn handle_disable(_logger: Option<Arc<LlmLogger>>) -> String {
    format!(
        "{} 动态禁用功能暂未实现\n{} 请在配置文件中设置 llm.logging.enabled: false 并重启",
        "⚠️".yellow(),
        "提示:".dimmed()
    )
}

/// 搜索日志
fn handle_search(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            // 解析参数：keyword [--days N]
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.is_empty() {
                return format!("{} 请提供搜索关键词", "错误:".red());
            }

            let keyword = parts[0];
            let mut days: Option<u32> = None;

            // 解析 --days 参数
            if let Some(pos) = parts.iter().position(|&x| x == "--days") {
                if let Some(days_str) = parts.get(pos + 1) {
                    days = days_str.parse().ok();
                }
            }

            // 执行搜索
            let results = logger.search_logs(keyword, days);

            if results.is_empty() {
                return format!(
                    "{} 未找到包含 \"{}\" 的日志",
                    "提示:".yellow(),
                    keyword
                );
            }

            let mut output = vec![
                format!(
                    "{} {} 条匹配结果{}:",
                    "找到".bold().green(),
                    results.len(),
                    if let Some(d) = days {
                        format!("（最近 {} 天）", d)
                    } else {
                        String::new()
                    }
                ),
                String::new(),
            ];

            for (idx, log) in results.iter().take(20).enumerate() {
                let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                let status_icon = match log.meta.status.as_str() {
                    "success" => "✓".green(),
                    "error" => "✗".red(),
                    _ => "?".yellow(),
                };

                output.push(format!(
                    "[{}] {} {} | {} | {}",
                    idx + 1,
                    status_icon,
                    time_str.dimmed(),
                    log.model.cyan(),
                    log.request.summary
                ));

                if let Some(ref response) = log.response {
                    output.push(format!("    → {}", response.summary.dimmed()));
                }

                output.push(String::new());
            }

            if results.len() > 20 {
                output.push(format!(
                    "{}（仅显示前 20 条，共 {} 条）",
                    "...".dimmed(),
                    results.len()
                ));
            }

            output.join("\n")
        }
        None => format!("{} LLM 日志功能未启用", "ℹ️".cyan()),
    }
}

/// 显示统计报告
fn handle_stats(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            // 解析 --days 参数
            let parts: Vec<&str> = args.split_whitespace().collect();
            let mut days: Option<u32> = None;

            if let Some(pos) = parts.iter().position(|&x| x == "--days") {
                if let Some(days_str) = parts.get(pos + 1) {
                    days = days_str.parse().ok();
                }
            }

            // 获取统计信息
            let stats = logger.get_statistics(days);

            if stats.total_requests == 0 {
                return format!("{} 暂无日志数据", "提示:".yellow());
            }

            let success_rate = if stats.total_requests > 0 {
                (stats.successful_requests as f64 / stats.total_requests as f64 * 100.0) as u32
            } else {
                0
            };

            let mut lines = vec![
                format!(
                    "{} {}",
                    "LLM 交互统计".bold().cyan(),
                    if let Some(d) = days {
                        format!("(最近 {} 天)", d)
                    } else {
                        "(全部)".to_string()
                    }
                ),
                String::new(),
                format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━".dimmed()),
                String::new(),
            ];

            // 请求统计
            lines.push(format!("{}", "请求统计:".bold()));
            lines.push(format!(
                "  总请求:   {} 次",
                stats.total_requests.to_string().green()
            ));
            lines.push(format!(
                "  - 成功:   {} 次 ({}%)",
                stats.successful_requests.to_string().green(),
                success_rate.to_string().green()
            ));
            lines.push(format!(
                "  - 失败:   {} 次",
                stats.failed_requests.to_string().red()
            ));
            lines.push(format!(
                "  - 流式:   {} 次",
                stats.streaming_requests.to_string().cyan()
            ));
            lines.push(String::new());

            // 模型分布
            if !stats.model_usage.is_empty() {
                lines.push(format!("{}", "模型分布:".bold()));
                let mut models: Vec<_> = stats.model_usage.iter().collect();
                models.sort_by(|a, b| b.1.cmp(a.1));

                for (model, count) in models {
                    let percentage = (*count as f64 / stats.total_requests as f64 * 100.0) as u32;
                    let bar = "█".repeat((percentage / 5) as usize);
                    lines.push(format!(
                        "  {:<20} │{} {}%",
                        model.cyan(),
                        bar.green(),
                        percentage
                    ));
                }
                lines.push(String::new());
            }

            // 性能指标
            if stats.avg_latency_ms > 0 {
                lines.push(format!("{}", "性能指标:".bold()));
                lines.push(format!(
                    "  平均延迟: {} ms",
                    stats.avg_latency_ms.to_string().yellow()
                ));
                lines.push(format!(
                    "  最小延迟: {} ms",
                    stats.min_latency_ms.to_string().green()
                ));
                lines.push(format!(
                    "  最大延迟: {} ms",
                    stats.max_latency_ms.to_string().red()
                ));
                lines.push(format!(
                    "  P50 延迟: {} ms",
                    stats.p50_latency_ms.to_string().yellow()
                ));
                lines.push(format!(
                    "  P95 延迟: {} ms",
                    stats.p95_latency_ms.to_string().yellow()
                ));
                lines.push(format!(
                    "  P99 延迟: {} ms",
                    stats.p99_latency_ms.to_string().red()
                ));
                lines.push(String::new());
            }

            // Token 使用量
            if stats.total_tokens > 0 {
                lines.push(format!("{}", "Token 使用:".bold()));
                lines.push(format!(
                    "  总量:     {} tokens",
                    format_number(stats.total_tokens).green()
                ));
                lines.push(format!(
                    "  - Prompt:     {} tokens ({}%)",
                    format_number(stats.total_prompt_tokens).cyan(),
                    (stats.total_prompt_tokens as f64 / stats.total_tokens as f64 * 100.0) as u32
                ));
                lines.push(format!(
                    "  - Completion: {} tokens ({}%)",
                    format_number(stats.total_completion_tokens).cyan(),
                    (stats.total_completion_tokens as f64 / stats.total_tokens as f64 * 100.0)
                        as u32
                ));
            }

            lines.join("\n")
        }
        None => format!("{} LLM 日志功能未启用", "ℹ️".cyan()),
    }
}

/// 清理旧日志
fn handle_clean(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            let days: u32 = match args.trim().parse() {
                Ok(d) => d,
                Err(_) => {
                    return format!(
                        "{} 无效的天数参数: {}\n{} /llm-log clean 30",
                        "错误:".red(),
                        args,
                        "用法:".dimmed()
                    )
                }
            };

            if days == 0 {
                return format!("{} 天数必须大于 0", "错误:".red());
            }

            // 执行清理
            let (deleted_files, freed_bytes) = logger.clean_old_logs(days);

            if deleted_files == 0 {
                format!(
                    "{} 没有需要清理的日志（{} 天前）",
                    "提示:".yellow(),
                    days
                )
            } else {
                format!(
                    "{} 清理完成\n  删除文件: {} 个\n  释放空间: {} KB",
                    "✓".green(),
                    deleted_files.to_string().green(),
                    (freed_bytes / 1024).to_string().yellow()
                )
            }
        }
        None => format!("{} LLM 日志功能未启用", "ℹ️".cyan()),
    }
}

/// 列出最近的会话
fn handle_sessions(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            let n: usize = args.trim().parse().unwrap_or(20);
            let sessions = logger.list_recent_sessions(n);

            if sessions.is_empty() {
                return format!("{} 暂无会话记录", "提示:".yellow());
            }

            let mut output = vec![
                format!(
                    "{} {} 个会话:",
                    "最近".bold().cyan(),
                    sessions.len().to_string().green()
                ),
                String::new(),
            ];

            for (idx, (session_id, timestamp, model, summary)) in sessions.iter().enumerate() {
                let time_str = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

                output.push(format!(
                    "[{}] {} | {}",
                    idx + 1,
                    time_str.dimmed(),
                    model.cyan()
                ));
                output.push(format!("    {} {}", "ID:".dimmed(), session_id.yellow()));
                output.push(format!("    {}", summary));
                output.push(String::new());
            }

            output.push(format!(
                "{}",
                "提示: 使用 /llm-log replay <session_id> 查看完整交互".dimmed()
            ));

            output.join("\n")
        }
        None => format!("{} LLM 日志功能未启用", "ℹ️".cyan()),
    }
}

/// 回放会话
fn handle_replay(args: &str, logger: Option<Arc<LlmLogger>>) -> String {
    match logger {
        Some(logger) => {
            let session_id = args.trim();
            if session_id.is_empty() {
                return format!(
                    "{} 请提供会话 ID\n{} /llm-log replay <session_id>",
                    "错误:".red(),
                    "用法:".dimmed()
                );
            }

            // 查找日志
            match logger.get_log_by_session_id(session_id) {
                Some(log) => {
                    let mut output = vec![
                        format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()),
                        format!("{}", "会话回放".bold().cyan()),
                        format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()),
                        String::new(),
                    ];

                    // 会话元信息
                    output.push(format!("{}", "会话信息:".bold()));
                    output.push(format!("  {} {}", "ID:".dimmed(), log.session_id.yellow()));
                    output.push(format!(
                        "  {} {}",
                        "时间:".dimmed(),
                        log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().cyan()
                    ));
                    output.push(format!("  {} {}", "模型:".dimmed(), log.model.cyan()));

                    let status_icon = match log.meta.status.as_str() {
                        "success" => "✓ 成功".green(),
                        "error" => "✗ 失败".red(),
                        _ => "? 未知".yellow(),
                    };
                    output.push(format!("  {} {}", "状态:".dimmed(), status_icon));
                    output.push(String::new());

                    // 请求部分
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(format!("{} ({} 条消息)", "请求".bold().green(), log.request.message_count));
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(String::new());

                    if let Some(ref messages) = log.request.messages {
                        for (idx, msg) in messages.iter().enumerate() {
                            let role_str = match msg.role {
                                crate::llm::MessageRole::System => "System".magenta(),
                                crate::llm::MessageRole::User => "User".blue(),
                                crate::llm::MessageRole::Assistant => "Assistant".green(),
                                crate::llm::MessageRole::Tool => "Tool".yellow(),
                            };

                            output.push(format!("[{}] {}", idx + 1, role_str));

                            if let Some(ref content) = msg.content {
                                // 格式化内容（处理长文本）
                                let lines: Vec<&str> = content.lines().collect();
                                if lines.len() > 10 {
                                    for line in &lines[..10] {
                                        output.push(format!("  {}", line));
                                    }
                                    output.push(format!("  {} ({} 行)", "...".dimmed(), lines.len() - 10));
                                } else {
                                    for line in lines {
                                        output.push(format!("  {}", line));
                                    }
                                }
                            }

                            if let Some(ref tool_calls) = msg.tool_calls {
                                output.push(format!("  {} {} 个工具调用", "Tool Calls:".dimmed(), tool_calls.len()));
                                for tc in tool_calls {
                                    output.push(format!("    - {}", tc.function.name.yellow()));
                                }
                            }

                            output.push(String::new());
                        }
                    } else {
                        output.push(format!("  {} {}", "摘要:".dimmed(), log.request.summary));
                        output.push(format!("  {}", "(完整内容未记录)".dimmed()));
                        output.push(String::new());
                    }

                    // 响应部分
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(format!("{}", "响应".bold().cyan()));
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(String::new());

                    if let Some(ref response) = log.response {
                        if let Some(ref content) = response.content {
                            // 格式化响应内容
                            let lines: Vec<&str> = content.lines().collect();
                            if lines.len() > 20 {
                                for line in &lines[..20] {
                                    output.push(format!("  {}", line));
                                }
                                output.push(format!("  {} ({} 行)", "...".dimmed(), lines.len() - 20));
                            } else {
                                for line in lines {
                                    output.push(format!("  {}", line));
                                }
                            }
                        } else {
                            output.push(format!("  {} {}", "摘要:".dimmed(), response.summary));
                            output.push(format!("  {}", "(完整内容未记录)".dimmed()));
                        }

                        output.push(String::new());
                        output.push(format!(
                            "  {} {} 字符",
                            "长度:".dimmed(),
                            response.content_length.to_string().yellow()
                        ));
                        output.push(format!(
                            "  {} {}",
                            "结束原因:".dimmed(),
                            response.finish_reason.cyan()
                        ));

                        if let Some(ref usage) = response.usage {
                            output.push(String::new());
                            output.push(format!("{}", "Token 使用:".bold()));
                            output.push(format!(
                                "  Prompt:     {} tokens",
                                usage.prompt_tokens.to_string().cyan()
                            ));
                            output.push(format!(
                                "  Completion: {} tokens",
                                usage.completion_tokens.to_string().cyan()
                            ));
                            output.push(format!(
                                "  总计:       {} tokens",
                                usage.total_tokens.to_string().green()
                            ));
                        }
                    } else {
                        output.push(format!("  {}", "(请求失败，无响应)".red()));
                    }

                    output.push(String::new());

                    // 上下文信息（如果有）
                    if let Some(ref context) = log.meta.context {
                        output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                        output.push(format!("{}", "执行上下文".bold().magenta()));
                        output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                        output.push(String::new());

                        if let Some(ref user_input) = context.user_input {
                            output.push(format!("{}", "原始输入:".bold()));
                            output.push(format!("  {}", user_input.cyan()));
                            output.push(String::new());
                        }

                        if let Some(ref intent) = context.intent {
                            output.push(format!("  {} {}", "Intent:".dimmed(), intent.yellow()));
                        }

                        if !context.tools_used.is_empty() {
                            output.push(format!("{}", "工具调用:".bold()));
                            for tool in &context.tools_used {
                                output.push(format!("  • {}", tool.green()));
                            }
                            output.push(String::new());
                        }

                        if let Some(ref summary) = context.tool_results_summary {
                            output.push(format!("{}", "工具结果摘要:".bold()));
                            output.push(format!("  {}", summary.dimmed()));
                            output.push(String::new());
                        }
                    }

                    // 元数据
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(format!("{}", "性能数据".bold().yellow()));
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                    output.push(String::new());

                    output.push(format!(
                        "  {} {} ms",
                        "延迟:".dimmed(),
                        log.meta.latency_ms.to_string().yellow()
                    ));
                    output.push(format!(
                        "  {} {}",
                        "流式:".dimmed(),
                        if log.meta.is_streaming { "是".green() } else { "否".dimmed() }
                    ));

                    if let Some(ref started) = log.meta.started_at {
                        output.push(format!(
                            "  {} {}",
                            "开始:".dimmed(),
                            started.format("%H:%M:%S%.3f").to_string().dimmed()
                        ));
                    }

                    if let Some(ref completed) = log.meta.completed_at {
                        output.push(format!(
                            "  {} {}",
                            "完成:".dimmed(),
                            completed.format("%H:%M:%S%.3f").to_string().dimmed()
                        ));
                    }

                    if let Some(ref error) = log.meta.error {
                        output.push(String::new());
                        output.push(format!("{}", "错误信息:".bold().red()));
                        output.push(format!("  {}", error.red()));
                    }

                    output.push(String::new());
                    output.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()));

                    output.join("\n")
                }
                None => format!(
                    "{} 未找到会话 ID: {}\n{} 使用 /llm-log sessions 查看可用会话",
                    "错误:".red(),
                    session_id.yellow(),
                    "提示:".dimmed()
                ),
            }
        }
        None => format!("{} LLM 日志功能未启用", "ℹ️".cyan()),
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 格式化大数字（添加千位分隔符）
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

/// 统计日志文件
fn count_log_files(log_dir: &std::path::Path) -> (usize, u64, usize) {
    let mut file_count = 0;
    let mut total_size = 0;
    let mut entry_count = 0;

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "jsonl" {
                            file_count += 1;
                            total_size += metadata.len();

                            // 估算条目数（每行一个条目）
                            if let Ok(content) = fs::read_to_string(&path) {
                                entry_count += content.lines().count();
                            }
                        }
                    }
                }
            }
        }
    }

    (file_count, total_size, entry_count)
}

/// 获取最新的日志文件
fn get_latest_log_file(log_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut latest: Option<(std::path::PathBuf, std::time::SystemTime)> = None;

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "jsonl" {
                            if let Ok(modified) = metadata.modified() {
                                match latest {
                                    None => latest = Some((path, modified)),
                                    Some((_, ref latest_time)) => {
                                        if modified > *latest_time {
                                            latest = Some((path, modified));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    latest.map(|(path, _)| path)
}

/// 读取文件的最后 N 行
fn read_last_n_lines(path: &std::path::Path, n: usize) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let start = if lines.len() > n {
        lines.len() - n
    } else {
        0
    };

    Ok(lines[start..].to_vec())
}

/// 格式化日志条目
fn format_log_entry(log: &serde_json::Value) -> String {
    let timestamp = log["timestamp"]
        .as_str()
        .unwrap_or("unknown")
        .split('T')
        .nth(1)
        .and_then(|t| t.split('.').next())
        .unwrap_or("unknown");

    let model = log["model"].as_str().unwrap_or("unknown");

    let summary = log["request"]["summary"]
        .as_str()
        .unwrap_or("[无内容]");

    let status = log["meta"]["status"].as_str().unwrap_or("unknown");
    let latency = log["meta"]["latency_ms"].as_u64().unwrap_or(0);

    let status_icon = match status {
        "success" => "✓".green(),
        "error" => "✗".red(),
        _ => "?".yellow(),
    };

    format!(
        "{} {} | {} | {} | {}ms",
        status_icon,
        timestamp.dimmed(),
        model.cyan(),
        summary,
        latency.to_string().yellow()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_log_help() {
        let result = show_help();
        assert!(result.contains("LLM 交互日志管理"));
        assert!(result.contains("status"));
        assert!(result.contains("recent"));
    }

    #[test]
    fn test_handle_status_without_logger() {
        let result = handle_status(None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_recent_without_logger() {
        let result = handle_recent("10", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_enable() {
        let result = handle_enable(None);
        assert!(result.contains("暂未实现"));
    }

    #[test]
    fn test_handle_disable() {
        let result = handle_disable(None);
        assert!(result.contains("暂未实现"));
    }

    #[test]
    fn test_handle_search_without_logger() {
        let result = handle_search("test", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_search_no_keyword() {
        use crate::llm::LlmLoggerConfig;
        use std::sync::Arc;

        let config = LlmLoggerConfig::default();
        let logger = Arc::new(crate::llm::LlmLogger::new(config));
        let result = handle_search("", Some(logger));
        assert!(result.contains("请提供搜索关键词"));
    }

    #[test]
    fn test_handle_stats_without_logger() {
        let result = handle_stats("", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_clean_without_logger() {
        let result = handle_clean("30", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_clean_invalid_days() {
        use crate::llm::LlmLoggerConfig;
        use std::sync::Arc;

        let config = LlmLoggerConfig::default();
        let logger = Arc::new(crate::llm::LlmLogger::new(config));
        let result = handle_clean("invalid", Some(logger));
        assert!(result.contains("无效的天数参数"));
    }

    #[test]
    fn test_handle_clean_zero_days() {
        use crate::llm::LlmLoggerConfig;
        use std::sync::Arc;

        let config = LlmLoggerConfig::default();
        let logger = Arc::new(crate::llm::LlmLogger::new(config));
        let result = handle_clean("0", Some(logger));
        assert!(result.contains("天数必须大于 0"));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(123), "123");
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(1234567890), "1,234,567,890");
    }

    #[test]
    fn test_handle_sessions_without_logger() {
        let result = handle_sessions("", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_replay_without_logger() {
        let result = handle_replay("test-id", None);
        assert!(result.contains("未启用"));
    }

    #[test]
    fn test_handle_replay_no_session_id() {
        use crate::llm::LlmLoggerConfig;
        use std::sync::Arc;

        let config = LlmLoggerConfig::default();
        let logger = Arc::new(crate::llm::LlmLogger::new(config));
        let result = handle_replay("", Some(logger));
        assert!(result.contains("请提供会话 ID"));
    }
}
