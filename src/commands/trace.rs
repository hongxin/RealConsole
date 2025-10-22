//! 统一追踪命令
//!
//! 提供 /trace 命令，聚合四个观测维度的统一视图

use crate::command::{Command, CommandRegistry};
use crate::tracer::{Dimension, TraceEntry, TraceStats, UnifiedTracer};
use colored::Colorize;
use std::sync::Arc;

/// 注册 trace 命令
pub fn register_trace_commands(registry: &mut CommandRegistry, tracer: Arc<UnifiedTracer>) {
    let trace_cmd = Command::from_fn(
        "trace",
        "统一追踪: trace [all|history|log|llm|context|search|stats] [options]",
        move |arg: &str| handle_trace(arg, Arc::clone(&tracer)),
    )
    .with_aliases(vec!["t".to_string()])
    .with_group("debug");

    registry.register(trace_cmd);
}

/// 处理 /trace 命令
fn handle_trace(arg: &str, tracer: Arc<UnifiedTracer>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_trace_default(tracer);
    }

    match parts[0] {
        "all" | "a" => handle_trace_all(&parts[1..], tracer),
        "history" | "h" => handle_trace_history(&parts[1..], tracer),
        "log" | "l" => handle_trace_log(&parts[1..], tracer),
        "llm" => handle_trace_llm(&parts[1..], tracer),
        "context" | "ctx" | "c" => handle_trace_context(&parts[1..], tracer),
        "search" | "s" => handle_trace_search(&parts[1..], tracer),
        "stats" => handle_trace_stats(tracer),
        "help" => trace_help(),
        _ => format!(
            "{} 未知子命令: {}\n使用 {} 查看帮助",
            "错误:".red(),
            parts[0],
            "/trace help".cyan()
        ),
    }
}

/// 默认视图（最近 20 条，四维聚合）
fn handle_trace_default(tracer: Arc<UnifiedTracer>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.query_all(20).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{}", "暂无追踪记录".dimmed())
                    } else {
                        format_trace_entries(entries, "统一追踪 - 最近 20 条")
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 查询所有维度
fn handle_trace_all(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.query_all(limit).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{}", "暂无追踪记录".dimmed())
                    } else {
                        format_trace_entries(entries, &format!("统一追踪 - 最近 {} 条", limit))
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 查询 History 维度
fn handle_trace_history(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer
                .query_by_dimension(Dimension::Statistics, limit)
                .await
            {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{} 暂无 History 记录", "提示:".yellow())
                    } else {
                        format_trace_entries(
                            entries,
                            &format!("📊 History (统计维度) - {} 条", limit),
                        )
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 查询 log 维度
fn handle_trace_log(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer
                .query_by_dimension(Dimension::Coordination, limit)
                .await
            {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{} 暂无 log 记录", "提示:".yellow())
                    } else {
                        format_trace_entries(
                            entries,
                            &format!("🔗 log (协同维度) - {} 条", limit),
                        )
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 查询 llm-log 维度
fn handle_trace_llm(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.query_by_dimension(Dimension::BlackBox, limit).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!(
                            "{} 暂无 llm-log 记录（LlmLogger 可能未启用）",
                            "提示:".yellow()
                        )
                    } else {
                        format_trace_entries(
                            entries,
                            &format!("🤖 llm-log (黑盒维度) - {} 条", limit),
                        )
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 查询 Context 维度
fn handle_trace_context(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.query_by_dimension(Dimension::Memory, limit).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{} 暂无 Context 记录", "提示:".yellow())
                    } else {
                        format_trace_entries(
                            entries,
                            &format!("💭 Context (记忆维度) - {} 条", limit),
                        )
                    }
                }
                Err(e) => format!("{} 查询失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 关键词搜索
fn handle_trace_search(args: &[&str], tracer: Arc<UnifiedTracer>) -> String {
    if args.is_empty() {
        return format!(
            "{} 请提供搜索关键词\n用法: {} <关键词>",
            "错误:".red(),
            "/trace search".cyan()
        );
    }

    let keyword = args.join(" ");

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.search(&keyword).await {
                Ok(entries) => {
                    if entries.is_empty() {
                        format!("{} 未找到包含 '{}' 的记录", "提示:".yellow(), keyword.cyan())
                    } else {
                        format_trace_entries(
                            entries,
                            &format!("🔍 搜索结果: '{}'", keyword.cyan()),
                        )
                    }
                }
                Err(e) => format!("{} 搜索失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// 统计信息
fn handle_trace_stats(tracer: Arc<UnifiedTracer>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match tracer.stats().await {
                Ok(stats) => format_trace_stats(stats),
                Err(e) => format!("{} 获取统计失败: {}", "错误:".red(), e),
            }
        })
    })
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// 格式化输出
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// 格式化追踪条目列表
fn format_trace_entries(entries: Vec<TraceEntry>, title: &str) -> String {
    let mut lines = vec![
        format!("{}", title.bold().cyan()),
        format!(
            "{} {} 条记录",
            "━━".dimmed(),
            entries.len().to_string().green()
        ),
        String::new(),
    ];

    for entry in entries {
        lines.push(entry.preview());
    }

    lines.join("\n")
}

/// 格式化统计信息
fn format_trace_stats(stats: TraceStats) -> String {
    if stats.total_entries == 0 {
        return format!("{}", "暂无追踪数据".dimmed());
    }

    let mut lines = vec![
        format!("{}", "统一追踪 - 统计信息".bold().cyan()),
        String::new(),
        format!(
            "{} {}",
            "总条目数:".bold(),
            stats.total_entries.to_string().green()
        ),
    ];

    // 时间范围
    if let Some((earliest, latest)) = stats.time_range {
        let duration = latest.signed_duration_since(earliest);
        let time_span = if duration.num_days() > 0 {
            format!("{} 天", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} 小时", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} 分钟", duration.num_minutes())
        } else {
            format!("{} 秒", duration.num_seconds())
        };

        lines.push(format!("{} {}", "时间跨度:".bold(), time_span.cyan()));
        lines.push(format!(
            "{} {}",
            "平均条目/小时:".bold(),
            format!("{:.1}", stats.avg_entries_per_hour).cyan()
        ));
    }

    // 按维度分布
    lines.push(String::new());
    lines.push(format!("{}", "按维度分布:".bold()));

    let mut dim_vec: Vec<_> = stats.by_dimension.iter().collect();
    dim_vec.sort_by(|a, b| b.1.cmp(a.1)); // 按数量降序

    for (dim, count) in dim_vec {
        let percentage = (*count as f64 / stats.total_entries as f64 * 100.0) as usize;
        let bar = "█".repeat((percentage / 5).max(1)); // 每5%一个方块

        lines.push(format!(
            "  {} {:15} {} {:3}% ({})",
            dim.icon(),
            dim.chinese_name().yellow(),
            bar.green(),
            percentage,
            count.to_string().dimmed()
        ));
    }

    // 按状态分布
    if !stats.by_status.is_empty() {
        lines.push(String::new());
        lines.push(format!("{}", "按状态分布:".bold()));

        let mut status_vec: Vec<_> = stats.by_status.iter().collect();
        status_vec.sort_by(|a, b| b.1.cmp(a.1));

        for (status, count) in status_vec {
            let percentage = (*count as f64 / stats.total_entries as f64 * 100.0) as usize;
            let icon = match status.as_str() {
                "Success" => "✓".green(),
                "Failed" => "✗".red(),
                "Running" => "⟳".yellow(),
                "Cancelled" => "⊘".dimmed(),
                _ => "?".dimmed(),
            };

            lines.push(format!(
                "  {} {:10} {:3}% ({})",
                icon,
                status.yellow(),
                percentage,
                count.to_string().dimmed()
            ));
        }
    }

    lines.join("\n")
}

/// 帮助文档
fn trace_help() -> String {
    format!(
        r#"{title}

{desc}

{subtitle}
  /trace                       - 显示最近 20 条记录（四维聚合）
  /trace all [n]               - 显示最近 N 条记录（默认 20）
  /trace history [n]           - 仅显示 History 维度（统计）
  /trace log [n]               - 仅显示 log 维度（协同）
  /trace llm [n]               - 仅显示 llm-log 维度（黑盒）
  /trace context [n]           - 仅显示 Context 维度（记忆）
  /trace search <关键词>       - 搜索包含关键词的记录
  /trace stats                 - 显示统计信息

{examples}
  /trace                       # 快速概览
  /trace all 50                # 查看最近 50 条
  /trace history 30            # 查看最近 30 条命令历史
  /trace search "error"        # 搜索包含 "error" 的记录
  /trace stats                 # 查看统计分布

{philosophy}
  📊 History   (统计维度) - 命令频率，使用模式
  🔗 log       (协同维度) - 端到端执行追踪
  🤖 llm-log   (黑盒维度) - LLM API 调用详情
  💭 Context   (记忆维度) - 对话上下文状态

{shortcuts}
  trace → t
  history → h, log → l, context → c/ctx
  search → s, all → a

{footer}
  详见: docs/04-reports/trace-command-design.md
"#,
        title = "统一追踪".bold().cyan(),
        desc = "/trace 提供四个维度的统一视图，降低记忆负担".dimmed(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        philosophy = "四维哲学:".bold(),
        shortcuts = "快捷命令:".dimmed(),
        footer = "文档:".dimmed()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConversationConfig;
    use crate::conversation::context_manager::ContextManager;
    use crate::execution_logger::ExecutionLogger;
    use crate::history::HistoryManager;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_tracer() -> Arc<UnifiedTracer> {
        let test_path = PathBuf::from("/tmp/realconsole_test_trace_history.json");
        let mut history = HistoryManager::new(test_path, 100);
        history.add("ls -la".to_string(), true);
        history.add("pwd".to_string(), true);

        let exec_logger = ExecutionLogger::new(100);
        let config = ConversationConfig::default();
        let context = ContextManager::new(config);

        Arc::new(UnifiedTracer::new(
            Arc::new(RwLock::new(history)),
            Arc::new(RwLock::new(exec_logger)),
            None,
            Arc::new(RwLock::new(context)),
        ))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_trace_default() {
        let tracer = create_test_tracer();
        let result = handle_trace_default(tracer);
        assert!(result.contains("统一追踪") || result.contains("暂无"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_trace_all() {
        let tracer = create_test_tracer();
        let result = handle_trace_all(&["10"], tracer);
        assert!(result.contains("统一追踪") || result.contains("暂无"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_trace_history() {
        let tracer = create_test_tracer();
        let result = handle_trace_history(&["5"], tracer);
        assert!(result.contains("History") || result.contains("暂无"));
    }

    #[test]
    fn test_handle_trace_search_no_keyword() {
        let tracer = create_test_tracer();
        let result = handle_trace_search(&[], tracer);
        assert!(result.contains("错误"));
        assert!(result.contains("关键词"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_trace_search() {
        let tracer = create_test_tracer();
        let result = handle_trace_search(&["ls"], tracer);
        assert!(result.contains("搜索结果") || result.contains("未找到"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_trace_stats() {
        let tracer = create_test_tracer();
        let result = handle_trace_stats(tracer);
        assert!(result.contains("统计") || result.contains("暂无"));
    }

    #[test]
    fn test_trace_help() {
        let result = trace_help();
        assert!(result.contains("统一追踪"));
        assert!(result.contains("用法"));
        assert!(result.contains("示例"));
        assert!(result.contains("四维哲学"));
    }

    #[test]
    fn test_handle_trace_unknown_subcommand() {
        let tracer = create_test_tracer();
        let result = handle_trace("unknown", tracer);
        assert!(result.contains("未知子命令"));
        assert!(result.contains("unknown"));
    }
}
