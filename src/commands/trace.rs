//! 统一追踪命令
//!
//! 提供 /trace 命令，聚合四个观测维度的统一视图

use crate::command::{Command, CommandRegistry};
use crate::trace_context::TraceStore;
use crate::tracer::{Dashboard, DashboardConfig, Dimension, TraceEntry, TraceStats, UnifiedTracer};
use colored::Colorize;
use std::sync::Arc;
use uuid::Uuid;

/// 注册 trace 命令
pub fn register_trace_commands(
    registry: &mut CommandRegistry,
    tracer: Arc<UnifiedTracer>,
    trace_store: Arc<TraceStore>,
) {
    let trace_cmd = Command::from_fn(
        "trace",
        "统一追踪: trace [all|history|log|llm|context|search|stats|dashboard|detail|tree] [options]",
        move |arg: &str| handle_trace(arg, Arc::clone(&tracer), Arc::clone(&trace_store)),
    )
    .with_aliases(vec!["t".to_string()])
    .with_group("debug");

    registry.register(trace_cmd);
}

/// 处理 /trace 命令
fn handle_trace(arg: &str, tracer: Arc<UnifiedTracer>, trace_store: Arc<TraceStore>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_trace_default(tracer);
    }

    match parts[0] {
        "all" | "a" => handle_trace_all(&parts[1..], tracer),
        "recent" | "r" => handle_trace_recent(&parts[1..], trace_store),
        "history" | "h" => handle_trace_history(&parts[1..], tracer),
        "log" | "l" => handle_trace_log(&parts[1..], tracer),
        "llm" => handle_trace_llm(&parts[1..], tracer),
        "context" | "ctx" | "c" => handle_trace_context(&parts[1..], tracer),
        "search" | "s" => handle_trace_search(&parts[1..], tracer),
        "stats" => handle_trace_stats(tracer),
        "dashboard" | "dash" => handle_trace_dashboard(tracer),
        "detail" | "d" => handle_trace_detail(&parts[1..], trace_store),
        "tree" => handle_trace_tree(&parts[1..], trace_store),
        "help" => trace_help(),
        _ => format!(
            "{} 未知子命令: {}\n使用 {} 查看帮助",
            "错误:".red(),
            parts[0],
            "/trace help".cyan()
        ),
    }
}

/// 默认视图（最近 10 条 Trace，关联视图）
/// ✨ v1.5.1: 改进 - 显示 TraceStore 中的完整追踪，而非四维堆叠
fn handle_trace_default(tracer: Arc<UnifiedTracer>) -> String {
    // 先尝试从旧的四维查询（向后兼容）
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

/// ✨ v1.5.1: 显示最近的完整 Trace（关联视图）
fn handle_trace_recent(args: &[&str], trace_store: Arc<TraceStore>) -> String {
    let limit = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let traces = trace_store.get_recent_traces(limit).await;

            if traces.is_empty() {
                return format!("{}", "暂无完整追踪记录\n提示: 使用 /trace 查看传统四维聚合视图".dimmed());
            }

            let mut lines = vec![
                format!("{}", "最近的完整追踪".bold().cyan()),
                format!("{} {} 条 Trace", "━━".dimmed(), traces.len().to_string().green()),
                String::new(),
            ];

            for (i, trace) in traces.iter().enumerate() {
                if i > 0 {
                    lines.push(format!("{}", "━━━━━━━━━━━━━━━━━━━━━━".dimmed()));
                }

                let trace_id_short = trace.trace_id.to_string()[..8].to_string();
                let status = if trace.is_success() {
                    "✓".green()
                } else {
                    "✗".red()
                };

                lines.push(format!(
                    "{} {} [{}] \"{}\"",
                    status,
                    "Trace".bold(),
                    trace_id_short.dimmed(),
                    trace.user_input.yellow()
                ));

                // 显示 Span 统计
                if !trace.spans.is_empty() {
                    let span_types: std::collections::HashMap<_, _> = trace.spans.iter()
                        .map(|s| s.span_type)
                        .fold(std::collections::HashMap::new(), |mut acc, t| {
                            *acc.entry(t).or_insert(0) += 1;
                            acc
                        });

                    let mut type_strs = Vec::new();
                    for (span_type, count) in span_types {
                        type_strs.push(format!("{} {}", span_type.icon(), count));
                    }

                    lines.push(format!("  {} {}", "Spans:".dimmed(), type_strs.join(", ")));
                }

                // 显示总时长
                if let Some(duration_ms) = trace.total_duration_ms() {
                    lines.push(format!("  {} {}ms", "耗时:".dimmed(), duration_ms.to_string().cyan()));
                }

                // 提示如何查看详情
                lines.push(format!(
                    "  {} {} | {}",
                    "💡".dimmed(),
                    format!("/trace detail {}", trace_id_short).cyan(),
                    format!("/trace tree {}", trace_id_short).cyan()
                ));
            }

            lines.join("\n")
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

/// ✨ v1.6.0: Dashboard 四象分区视图
fn handle_trace_dashboard(tracer: Arc<UnifiedTracer>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let dashboard = Dashboard::with_defaults(tracer);
            match dashboard.render().await {
                Ok(output) => output,
                Err(e) => format!("{} Dashboard 渲染失败: {}", "错误:".red(), e),
            }
        })
    })
}

/// ✨ v1.5.1: 详细调用链
fn handle_trace_detail(args: &[&str], trace_store: Arc<TraceStore>) -> String {
    if args.is_empty() {
        return format!(
            "{} 请提供 trace_id\n用法: {} <trace_id>",
            "错误:".red(),
            "/trace detail".cyan()
        );
    }

    let trace_id_str = args[0];

    // 支持短格式（前8位）或完整 UUID
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // 首先尝试完整 UUID
            if let Ok(trace_id) = Uuid::parse_str(trace_id_str) {
                if let Some(trace) = trace_store.get_trace(trace_id).await {
                    return format_trace_detail(trace);
                }
            }

            // 如果不是完整 UUID 或未找到，尝试短格式匹配
            if trace_id_str.len() >= 8 {
                let trace_ids = trace_store.get_recent_trace_ids(100).await;
                for id in trace_ids {
                    let id_str = id.to_string();
                    if id_str.starts_with(trace_id_str) {
                        if let Some(trace) = trace_store.get_trace(id).await {
                            return format_trace_detail(trace);
                        }
                    }
                }
            }

            format!(
                "{} 未找到 trace_id: {}\n提示: 使用 {} 查看最近的 trace",
                "错误:".red(),
                trace_id_str.yellow(),
                "/trace recent".cyan()
            )
        })
    })
}

/// ✨ v1.5.1: 调用树视图
fn handle_trace_tree(args: &[&str], trace_store: Arc<TraceStore>) -> String {
    if args.is_empty() {
        return format!(
            "{} 请提供 trace_id\n用法: {} <trace_id>",
            "错误:".red(),
            "/trace tree".cyan()
        );
    }

    let trace_id_str = args[0];

    // 支持短格式（前8位）或完整 UUID
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // 首先尝试完整 UUID
            if let Ok(trace_id) = Uuid::parse_str(trace_id_str) {
                if let Some(trace) = trace_store.get_trace(trace_id).await {
                    return format_trace_tree(trace);
                }
            }

            // 如果不是完整 UUID 或未找到，尝试短格式匹配
            if trace_id_str.len() >= 8 {
                let trace_ids = trace_store.get_recent_trace_ids(100).await;
                for id in trace_ids {
                    let id_str = id.to_string();
                    if id_str.starts_with(trace_id_str) {
                        if let Some(trace) = trace_store.get_trace(id).await {
                            return format_trace_tree(trace);
                        }
                    }
                }
            }

            format!(
                "{} 未找到 trace_id: {}\n提示: 使用 {} 查看最近的 trace",
                "错误:".red(),
                trace_id_str.yellow(),
                "/trace recent".cyan()
            )
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

/// ✨ v1.5.1: 格式化详细调用链
fn format_trace_detail(trace: crate::trace_context::CompleteTrace) -> String {
    use crate::trace_context::SpanStatus;

    let mut lines = vec![
        format!(
            "{} [{}]",
            "详细调用链".bold().cyan(),
            trace.trace_id.to_string().dimmed()
        ),
        format!("{} {}", "输入:".bold(), trace.user_input.yellow()),
        String::new(),
    ];

    // 按时间排序的 Spans
    let mut spans = trace.spans.clone();
    spans.sort_by_key(|s| s.start_time);

    lines.push(format!("{}", "调用链:".bold()));

    for span in &spans {
        let indent = match span.parent_span_id {
            None => "",
            Some(_) => "  ",
        };

        let status_icon = span.status.icon();
        let duration_ms = span.duration_ms().unwrap_or(0);
        let span_type_icon = span.span_type.icon();

        lines.push(format!(
            "{}{} {} {} ({}ms)",
            indent,
            status_icon,
            span_type_icon,
            span.name.cyan(),
            duration_ms.to_string().dimmed()
        ));

        // 显示属性
        if !span.attributes.is_empty() {
            for (key, value) in &span.attributes {
                lines.push(format!(
                    "{}    {}: {}",
                    indent,
                    key.dimmed(),
                    value.to_string().yellow()
                ));
            }
        }

        // 显示事件
        if !span.events.is_empty() {
            lines.push(format!("{}    {} 事件:", indent, "📝".dimmed()));
            for event in &span.events {
                lines.push(format!(
                    "{}      - {}",
                    indent,
                    event.name.dimmed()
                ));
            }
        }
    }

    // 总结
    if let Some(duration_ms) = trace.total_duration_ms() {
        lines.push(String::new());
        lines.push(format!(
            "{} {}ms",
            "总耗时:".bold(),
            duration_ms.to_string().green()
        ));
    }

    if let Some(root) = trace.root_span() {
        let status = if root.status.is_success() {
            "成功".green()
        } else {
            "失败".red()
        };
        lines.push(format!("{} {}", "状态:".bold(), status));
    }

    lines.join("\n")
}

/// ✨ v1.5.1: 格式化调用树
fn format_trace_tree(trace: crate::trace_context::CompleteTrace) -> String {
    let mut lines = vec![
        format!(
            "{} [{}]",
            "调用树".bold().cyan(),
            trace.trace_id.to_string().dimmed()
        ),
        format!("{} {}", "输入:".bold(), trace.user_input.yellow()),
        String::new(),
    ];

    // 从根节点开始递归构建树
    if let Some(root) = trace.root_span() {
        build_tree_lines(&trace, root, "", true, &mut lines);
    } else {
        lines.push("  (无根节点)".dimmed().to_string());
    }

    // 总结
    if let Some(duration_ms) = trace.total_duration_ms() {
        lines.push(String::new());
        lines.push(format!(
            "{} {}ms | {} Spans",
            "总计:".bold(),
            duration_ms.to_string().green(),
            trace.spans.len().to_string().cyan()
        ));
    }

    lines.join("\n")
}

/// 递归构建树状结构
fn build_tree_lines(
    trace: &crate::trace_context::CompleteTrace,
    span: &crate::trace_context::ExecutionSpan,
    prefix: &str,
    is_last: bool,
    lines: &mut Vec<String>,
) {
    // 当前节点
    let connector = if is_last { "└─" } else { "├─" };
    let status_icon = span.status.icon();
    let span_type_icon = span.span_type.icon();
    let duration_ms = span.duration_ms().unwrap_or(0);

    lines.push(format!(
        "{}{} {} {} {} ({}ms)",
        prefix,
        connector,
        status_icon,
        span_type_icon,
        span.name.cyan(),
        duration_ms.to_string().dimmed()
    ));

    // 子节点
    let children = trace.children_of(span.span_id);
    let child_count = children.len();

    for (i, child) in children.iter().enumerate() {
        let is_last_child = i == child_count - 1;
        let new_prefix = if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        build_tree_lines(trace, child, &new_prefix, is_last_child, lines);
    }
}

/// 帮助文档
fn trace_help() -> String {
    format!(
        r#"{title}

{desc}

{subtitle}
  /trace                       - 显示最近 20 条记录（四维聚合）
  /trace recent [n]            - ✨ 显示最近 N 个完整 Trace（默认 10）
  /trace all [n]               - 显示最近 N 条记录（默认 20）
  /trace history [n]           - 仅显示 History 维度（统计）
  /trace log [n]               - 仅显示 log 维度（协同）
  /trace llm [n]               - 仅显示 llm-log 维度（黑盒）
  /trace context [n]           - 仅显示 Context 维度（记忆）
  /trace search <关键词>       - 搜索包含关键词的记录
  /trace stats                 - 显示统计信息
  /trace dashboard             - 🎛️ 显示四象分区 Dashboard（v1.6.0 新增）
  /trace detail <trace_id>     - 显示详细调用链（v1.5.1 新增）
  /trace tree <trace_id>       - 显示调用树视图（v1.5.1 新增）

{examples}
  /trace                       # 快速概览（四维聚合）
  /trace recent                # ✨ 查看完整追踪（推荐）
  /trace recent 5              # 查看最近 5 个 Trace
  /trace all 50                # 查看最近 50 条
  /trace history 30            # 查看最近 30 条命令历史
  /trace search "error"        # 搜索包含 "error" 的记录
  /trace stats                 # 查看统计分布
  /trace dashboard             # 🎛️ 系统健康度 Dashboard（四象分区）
  /trace detail <uuid>         # 查看完整调用链
  /trace tree <uuid>           # 查看调用树

{philosophy}
  📊 History   (统计维度) - 命令频率，使用模式
  🔗 log       (协同维度) - 端到端执行追踪
  🤖 llm-log   (黑盒维度) - LLM API 调用详情
  💭 Context   (记忆维度) - 对话上下文状态

{shortcuts}
  trace → t
  recent → r, history → h, log → l, context → c/ctx
  search → s, all → a, detail → d, dashboard → dash

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
        let trace_store = Arc::new(TraceStore::new(100));
        let result = handle_trace("unknown", tracer, trace_store);
        assert!(result.contains("未知子命令"));
        assert!(result.contains("unknown"));
    }
}
