//! 显示模式控制
//!
//! 提供三种显示模式：
//! - Minimal（默认）：极简模式，只显示必要信息
//! - Standard：标准模式，显示适中信息
//! - Debug：调试模式，显示所有细节

use colored::Colorize;
use serde::{Deserialize, Serialize};

/// 显示模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DisplayMode {
    /// 极简模式（默认）
    /// - 不显示启动信息
    /// - 不显示 Intent 识别过程
    /// - 不显示执行命令
    /// - 不显示 fallback 警告
    /// - 仅显示最终输出
    Minimal,

    /// 标准模式
    /// - 简化启动信息
    /// - 显示 Intent 名称
    /// - 简化执行命令
    /// - 简化 fallback 信息
    /// - 显示执行耗时
    Standard,

    /// 调试模式
    /// - 显示所有启动信息
    /// - 显示 Intent 详情
    /// - 显示完整命令
    /// - 显示详细错误
    /// - 显示内部状态
    Debug,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Minimal
    }
}

impl DisplayMode {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" | "min" => Some(Self::Minimal),
            "standard" | "std" => Some(Self::Standard),
            "debug" | "dbg" => Some(Self::Debug),
            _ => None,
        }
    }

    /// 是否显示启动信息
    pub fn show_startup(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }

    /// 是否显示 Intent 识别信息
    pub fn show_intent(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }

    /// 是否显示执行命令
    pub fn show_command(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }

    /// 是否显示 fallback 信息
    pub fn show_fallback(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }

    /// 是否显示执行耗时
    pub fn show_timing(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }

    /// 是否显示调试信息
    pub fn show_debug(self) -> bool {
        matches!(self, Self::Debug)
    }

    /// 是否显示 LLM 生成提示
    pub fn show_llm_hint(self) -> bool {
        matches!(self, Self::Standard | Self::Debug)
    }
}

/// 显示辅助函数
pub struct Display;

impl Display {
    /// 启动信息（记忆加载）
    pub fn startup_memory(mode: DisplayMode, count: usize) {
        if mode.show_startup() {
            println!("{} {} 条记忆 (最近)", "✓ 已加载".dimmed(), count.to_string().dimmed());
        }
    }

    /// 启动信息（LLM 配置）
    pub fn startup_llm(mode: DisplayMode, llm_type: &str, model: &str, provider: &str) {
        if mode.show_debug() {
            println!(
                "{} {} ({})",
                format!("✓ {} LLM:", llm_type).green(),
                model,
                provider.dimmed()
            );
        }
    }

    /// 启动信息（LLM Pipeline）
    pub fn startup_llm_pipeline(mode: DisplayMode) {
        if mode.show_startup() {
            println!("{}", "✓ LLM Pipeline 生成器已启用".dimmed());
        }
    }

    /// 启动信息（Workflow Intent 系统）✨ Phase 8
    pub fn startup_workflow(mode: DisplayMode, workflow_count: usize) {
        if mode.show_startup() {
            println!(
                "{} {} 个工作流模板",
                "✓ Workflow Intent 系统已启用".dimmed(),
                workflow_count.to_string().dimmed()
            );
        }
    }

    /// Intent 识别信息
    pub fn intent_match(mode: DisplayMode, intent_name: &str, confidence: f64) {
        if mode.show_intent() {
            if mode.show_debug() {
                println!(
                    "{} {} (置信度: {:.2})",
                    "✨ Intent:".dimmed(),
                    intent_name.dimmed(),
                    confidence
                );
            } else {
                println!("{} {}", "✨".dimmed(), intent_name.dimmed());
            }
        }
    }

    /// LLM 生成提示
    pub fn llm_generation(mode: DisplayMode) {
        if mode.show_llm_hint() {
            println!("{}", "🤖 LLM 生成".dimmed());
        }
    }

    /// Workflow 匹配信息 ✨ Phase 8
    pub fn workflow_match(mode: DisplayMode, workflow_name: &str, confidence: f64) {
        if mode.show_intent() {
            if mode.show_debug() {
                println!(
                    "{} {} (置信度: {:.2})",
                    "⚡ Workflow:".cyan(),
                    workflow_name.cyan(),
                    confidence
                );
            } else {
                println!("{} {}", "⚡".cyan(), workflow_name.cyan());
            }
        }
    }

    /// Workflow 执行统计 ✨ Phase 8
    pub fn workflow_stats(
        mode: DisplayMode,
        duration_ms: u64,
        llm_calls: usize,
        tool_calls: usize,
        from_cache: bool,
    ) {
        if mode.show_timing() {
            let duration_sec = duration_ms as f64 / 1000.0;
            if mode.show_debug() {
                println!(
                    "{} {:.2}s | LLM: {} | 工具: {} | 缓存: {}",
                    "ⓘ".dimmed(),
                    duration_sec.to_string().dimmed(),
                    llm_calls.to_string().dimmed(),
                    tool_calls.to_string().dimmed(),
                    if from_cache { "命中" } else { "未命中" }
                );
            } else {
                // Standard 模式：简化显示
                if from_cache {
                    println!("{} {:.2}s (缓存)", "ⓘ".dimmed(), duration_sec.to_string().green().dimmed());
                } else {
                    println!("{} {:.2}s", "ⓘ".dimmed(), duration_sec.to_string().dimmed());
                }
            }
        }
    }

    /// 执行命令提示
    pub fn command_execution(mode: DisplayMode, command: &str) {
        if mode.show_command() {
            if mode.show_debug() {
                println!("{} {}", "→ 执行:".dimmed(), command.dimmed());
            } else {
                // Standard 模式：简化显示（最多50字符）
                let short_cmd = if command.len() > 50 {
                    format!("{}...", &command[..47])
                } else {
                    command.to_string()
                };
                println!("{} {}", "→".dimmed(), short_cmd.dimmed());
            }
        }
    }

    /// Fallback 警告
    pub fn fallback_warning(mode: DisplayMode, reason: &str) {
        if mode.show_fallback() {
            if mode.show_debug() {
                println!("{} {}", "⚠️  LLM 生成失败，降级到规则匹配:".yellow(), reason);
            } else {
                // Standard 模式：简化信息
                println!("{}", "⚠️  降级到规则匹配".yellow().dimmed());
            }
        }
    }

    /// 执行耗时
    pub fn execution_timing(mode: DisplayMode, seconds: f64) {
        if mode.show_timing() {
            println!("{} {:.1}s", "ⓘ".dimmed(), seconds.to_string().dimmed());
        }
    }

    /// 调试信息（任意消息）
    pub fn debug_info(mode: DisplayMode, message: &str) {
        if mode.show_debug() {
            println!("{} {}", "[DEBUG]".blue().dimmed(), message.dimmed());
        }
    }

    /// 错误信息（总是显示，但详细程度不同）
    pub fn error(mode: DisplayMode, error: &str) {
        if mode.show_debug() {
            eprintln!("{} {}", "❌ 错误:".red(), error);
        } else {
            // Minimal/Standard: 简化错误信息
            eprintln!("{} {}", "❌".red(), error);
        }
    }

    /// 配置加载信息
    pub fn config_loaded(mode: DisplayMode, path: &str) {
        if mode.show_debug() {
            println!("{} {}", "已加载配置:".dimmed(), path.dimmed());
        }
    }

    /// .env 加载信息
    pub fn env_loaded(mode: DisplayMode, path: &str) {
        if mode.show_debug() {
            println!("{} {}", "✓ 已加载 .env:".dimmed(), path.dimmed());
        }
    }

    /// 任务编排启动信息
    pub fn task_execution_start(mode: DisplayMode, goal: &str, total_stages: usize, total_tasks: usize) {
        if mode.show_startup() {
            println!(
                "{} {} · {} 阶段 · {} 任务",
                "🚀 开始执行:".cyan(),
                goal.bold(),
                total_stages.to_string().cyan(),
                total_tasks.to_string().cyan()
            );
        }
    }

    /// 阶段执行信息
    pub fn stage_execution(mode: DisplayMode, stage_num: usize, total_stages: usize, execution_mode: &str) {
        if mode.show_intent() {
            let mode_icon = match execution_mode {
                "Sequential" => "→",
                "Parallel" => "⇉",
                _ => "•",
            };
            println!(
                "{} {} {} ({}/{})",
                "▸".dimmed(),
                mode_icon.cyan(),
                format!("阶段 {}", stage_num + 1).dimmed(),
                stage_num + 1,
                total_stages
            );
        }
    }

    /// 任务执行信息
    pub fn task_execution(mode: DisplayMode, task_name: &str, task_idx: usize, total_tasks: usize) {
        if mode.show_command() {
            let percentage = if total_tasks > 0 {
                format!("({:.0}%)", (task_idx as f64 / total_tasks as f64) * 100.0)
            } else {
                String::new()
            };
            println!(
                "{} {} {}",
                "→".dimmed(),
                task_name,
                percentage.dimmed()
            );
        }
    }

    /// 任务完成状态
    pub fn task_completion(mode: DisplayMode, task_name: &str, status: &str, duration: u32) {
        if mode.show_timing() {
            let (icon, color) = match status {
                "Success" => ("✓", "green"),
                "Failed" => ("✗", "red"),
                "Skipped" => ("⊘", "yellow"),
                _ => ("•", "dimmed"),
            };

            let colored_icon = match color {
                "green" => icon.green(),
                "red" => icon.red(),
                "yellow" => icon.yellow(),
                _ => icon.dimmed(),
            };

            println!(
                "{} {} ({})",
                colored_icon,
                task_name.dimmed(),
                format!("{}s", duration).dimmed()
            );
        }
    }

    /// 进度条显示
    pub fn progress_bar(mode: DisplayMode, completed: usize, total: usize, elapsed: u32, remaining: u32) {
        if mode.show_timing() && total > 0 {
            let percentage = (completed as f64 / total as f64) * 100.0;
            let bar_width = 20;
            let filled = (percentage / 100.0 * bar_width as f64).round() as usize;
            let empty = bar_width - filled;

            let bar = format!(
                "[{}{}]",
                "█".repeat(filled).green(),
                "░".repeat(empty).dimmed()
            );

            let time_info = if remaining > 0 {
                format!("({}s/{}s)", elapsed, remaining)
            } else {
                format!("({}s)", elapsed)
            };

            println!(
                "{} {:.1}% {}",
                bar,
                percentage,
                time_info.dimmed()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_from_str() {
        assert_eq!(DisplayMode::from_str("minimal"), Some(DisplayMode::Minimal));
        assert_eq!(DisplayMode::from_str("min"), Some(DisplayMode::Minimal));
        assert_eq!(DisplayMode::from_str("standard"), Some(DisplayMode::Standard));
        assert_eq!(DisplayMode::from_str("std"), Some(DisplayMode::Standard));
        assert_eq!(DisplayMode::from_str("debug"), Some(DisplayMode::Debug));
        assert_eq!(DisplayMode::from_str("dbg"), Some(DisplayMode::Debug));
        assert_eq!(DisplayMode::from_str("unknown"), None);
    }

    #[test]
    fn test_minimal_mode() {
        let mode = DisplayMode::Minimal;
        assert!(!mode.show_startup());
        assert!(!mode.show_intent());
        assert!(!mode.show_command());
        assert!(!mode.show_fallback());
        assert!(!mode.show_timing());
        assert!(!mode.show_debug());
        assert!(!mode.show_llm_hint());
    }

    #[test]
    fn test_standard_mode() {
        let mode = DisplayMode::Standard;
        assert!(mode.show_startup());
        assert!(mode.show_intent());
        assert!(mode.show_command());
        assert!(mode.show_fallback());
        assert!(mode.show_timing());
        assert!(!mode.show_debug());
        assert!(mode.show_llm_hint());
    }

    #[test]
    fn test_debug_mode() {
        let mode = DisplayMode::Debug;
        assert!(mode.show_startup());
        assert!(mode.show_intent());
        assert!(mode.show_command());
        assert!(mode.show_fallback());
        assert!(mode.show_timing());
        assert!(mode.show_debug());
        assert!(mode.show_llm_hint());
    }

    #[test]
    fn test_default_mode() {
        let mode = DisplayMode::default();
        assert_eq!(mode, DisplayMode::Minimal);
    }
}
