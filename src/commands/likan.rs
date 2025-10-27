//! 离坎炼化炉命令
//!
//! 提供炼化炉状态查询和控制命令
//!
//! 用法：
//! - `/likan` - 显示当前状态
//! - `/likan status` - 显示当前状态
//! - `/likan history` - 显示循环历史
//! - `/likan cycle` - 手动触发一次炼化循环

use crate::command::{Command, CommandRegistry};
use crate::likan::{LiKanFurnace, LiKanStatusBar, LiKanTrigger};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册离坎炼化炉命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `furnace`: 炼化炉实例
/// - `statusbar`: 状态栏实例
/// - `trigger`: 手动触发器实例
pub fn register_likan_commands(
    registry: &mut CommandRegistry,
    furnace: Option<Arc<RwLock<LiKanFurnace>>>,
    statusbar: Option<Arc<LiKanStatusBar>>,
    trigger: Option<Arc<LiKanTrigger>>,
) {
    let cmd = Command::from_fn("likan", "离坎炼化炉状态和控制", move |args| {
        handle_likan(args, furnace.clone(), statusbar.clone(), trigger.clone())
    })
    .with_group("likan");

    registry.register(cmd);
}

/// 处理 /likan 命令
fn handle_likan(
    args: &str,
    furnace: Option<Arc<RwLock<LiKanFurnace>>>,
    statusbar: Option<Arc<LiKanStatusBar>>,
    trigger: Option<Arc<LiKanTrigger>>,
) -> String {
    let args_str = args.trim();

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            if args_str.is_empty() || args_str == "status" || args_str == "s" {
                show_status(&statusbar).await
            } else if args_str == "history" || args_str == "h" {
                show_history(&furnace).await
            } else if args_str == "cycle" || args_str == "c" {
                trigger_cycle(&trigger, &statusbar).await
            } else if args_str == "help" {
                show_help()
            } else {
                format!(
                    "⚠️ 未知子命令: {}\n\n{}",
                    args_str.yellow(),
                    show_help()
                )
            }
        })
    })
}

/// 显示帮助信息
fn show_help() -> String {
    format!(
        "{}\n\n{}\n  {}    - 显示当前状态\n  {}  - 显示循环历史\n  {}    - 手动触发一次炼化循环\n\n{}\n  {}    # 查看状态\n  {}   # 查看历史\n  {}     # 手动触发",
        "离坎炼化炉状态和控制".bold(),
        "子命令:".yellow(),
        "status".cyan(),
        "history".cyan(),
        "cycle".cyan(),
        "示例:".yellow(),
        "/likan status".cyan(),
        "/likan history".cyan(),
        "/likan cycle".cyan()
    )
}

/// 显示状态
async fn show_status(statusbar: &Option<Arc<LiKanStatusBar>>) -> String {
    // 检查炼化炉是否初始化
    let Some(ref statusbar) = statusbar else {
        return "⚠️ 炼化炉未初始化".to_string();
    };

    let status = statusbar.status();
    let s = status.read().await;

    // 构建状态输出
    let mut output = vec![
        "".to_string(),
        "🌊🔥 离坎炼化炉状态".bold().to_string(),
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
    ];

    // 上次循环时间
    match s.last_cycle {
        Some(last) => {
            let elapsed = last.elapsed().as_secs();
            output.push(format!(
                "上次循环: {} {}",
                format_duration(elapsed).cyan(),
                "前".dimmed()
            ));
        }
        None => {
            output.push(format!("上次循环: {}", "等待首次触发".yellow()));
        }
    }

    // 模式总数
    output.push(format!(
        "模式总数: {} 个",
        s.pattern_count.to_string().green().bold()
    ));

    // 高质量模式
    if s.high_confidence_count > 0 {
        output.push(format!(
            "高质量: {} 个 {}",
            s.high_confidence_count.to_string().green().bold(),
            "⭐".yellow()
        ));
    } else {
        output.push(format!("高质量: {}", "0 个".dimmed()));
    }

    // 下次循环
    match s.last_cycle {
        Some(last) => {
            let elapsed = last.elapsed().as_secs();
            let next_in = s.cycle_interval_secs.saturating_sub(elapsed);

            if next_in == 0 {
                output.push(format!("下次循环: {}", "即将触发".green().bold()));
            } else {
                output.push(format!(
                    "下次循环: {} {}",
                    format_duration(next_in).cyan(),
                    "后".dimmed()
                ));
            }
        }
        None => {
            output.push(format!(
                "下次循环: {} {}",
                format_duration(s.cycle_interval_secs).cyan(),
                "后".dimmed()
            ));
        }
    }

    // 循环间隔
    output.push(format!(
        "循环间隔: {}",
        format_duration(s.cycle_interval_secs).dimmed()
    ));

    // 状态栏
    output.push(format!(
        "状态栏: {}",
        if s.enabled {
            "启用".green()
        } else {
            "禁用（通知模式）".yellow()
        }
    ));

    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
    output.push("".to_string());

    output.join("\n")
}

/// 显示循环历史
async fn show_history(furnace: &Option<Arc<RwLock<LiKanFurnace>>>) -> String {
    let Some(ref furnace) = furnace else {
        return "⚠️ 炼化炉未初始化".to_string();
    };

    let f = furnace.read().await;
    let history = f.cycle_history();

    if history.is_empty() {
        return "📜 暂无循环历史".to_string();
    }

    let mut output = vec![
        "".to_string(),
        "🌊🔥 离坎炼化炉 - 循环历史".bold().to_string(),
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
        "".to_string(),
    ];

    // 显示最近10次循环（倒序）
    for (i, report) in history.iter().rev().enumerate() {
        let index = history.len() - i;
        let high_conf_str = if report.high_confidence_patterns > 0 {
            format!("({} ⭐)", report.high_confidence_patterns).yellow().to_string()
        } else {
            String::new()
        };

        output.push(format!(
            "{}. {} - {} 模式 {} - 耗时 {}ms",
            index.to_string().cyan(),
            report.started_at.format("%H:%M:%S").to_string().dimmed(),
            report.patterns_found.to_string().green(),
            high_conf_str,
            report.duration_ms
        ));
    }

    output.push("".to_string());
    output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
    output.push("".to_string());

    output.join("\n")
}

/// 手动触发循环
async fn trigger_cycle(
    trigger: &Option<Arc<LiKanTrigger>>,
    statusbar: &Option<Arc<LiKanStatusBar>>,
) -> String {
    // 检查触发器是否初始化
    let Some(ref trigger) = trigger else {
        return "⚠️ 炼化炉未初始化".to_string();
    };

    // 触发炼化循环
    match trigger.trigger_once().await {
        Ok(report) => {
            // 更新状态栏
            if let Some(ref statusbar) = statusbar {
                let status = statusbar.status();
                let mut s = status.write().await;
                s.last_cycle = Some(std::time::Instant::now());
                s.pattern_count = report.patterns_found;
                s.high_confidence_count = report.high_confidence_patterns;
                drop(s);
                statusbar.update().await;
            }

            // 格式化输出
            let mut output = vec![
                "".to_string(),
                "🌊🔥 手动触发炼化循环".bold().green().to_string(),
                "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
            ];

            output.push(format!(
                "循环时间: {}",
                report.started_at.format("%H:%M:%S").to_string().cyan()
            ));

            output.push(format!(
                "发现模式: {} 个",
                report.patterns_found.to_string().green().bold()
            ));

            if report.high_confidence_patterns > 0 {
                output.push(format!(
                    "高质量: {} 个 {}",
                    report.high_confidence_patterns.to_string().green().bold(),
                    "⭐".yellow()
                ));
            }

            output.push(format!(
                "执行耗时: {} ms",
                report.duration_ms.to_string().dimmed()
            ));

            output.push("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string());
            output.push("".to_string());
            output.push(format!(
                "{}：使用 {} 查看更新后的状态",
                "提示".yellow(),
                "/likan status".cyan()
            ));
            output.push("".to_string());

            output.join("\n")
        }
        Err(e) => {
            format!(
                "{}\\n\\n错误: {}\\n\\n提示：检查系统是否有足够的追踪数据\\n",
                "⚠️  炼化循环失败".bold().red(),
                e.to_string().yellow()
            )
        }
    }
}

/// 格式化持续时间为人类可读格式
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}秒", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let secs = secs % 60;
        if secs == 0 {
            format!("{}分钟", mins)
        } else {
            format!("{}分{}秒", mins, secs)
        }
    } else {
        let hours = secs / 3600;
        let mins = (secs % 3600) / 60;
        if mins == 0 {
            format!("{}小时", hours)
        } else {
            format!("{}小时{}分钟", hours, mins)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30秒");
        assert_eq!(format_duration(60), "1分钟");
        assert_eq!(format_duration(90), "1分30秒");
        assert_eq!(format_duration(3600), "1小时");
        assert_eq!(format_duration(3660), "1小时1分钟");
    }
}
