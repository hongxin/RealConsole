//! 显示模式控制
//!
//! 提供三种显示模式：
//! - Minimal（默认）：极简模式，只显示必要信息
//! - Standard：标准模式，显示适中信息
//! - Debug：调试模式，显示所有细节

use crate::task::{ExecutionPlan, ExecutionResult, TaskStatus};
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

impl std::str::FromStr for DisplayMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "minimal" | "min" => Ok(Self::Minimal),
            "standard" | "std" => Ok(Self::Standard),
            "debug" | "dbg" => Ok(Self::Debug),
            _ => Err(format!("Unknown display mode: {}", s)),
        }
    }
}

impl DisplayMode {
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
            println!(
                "{} {} 条记忆 (最近)",
                "✓ 已加载".dimmed(),
                count.to_string().dimmed()
            );
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
                    println!(
                        "{} {:.2}s (缓存)",
                        "ⓘ".dimmed(),
                        duration_sec.to_string().green().dimmed()
                    );
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
                use crate::utils::string::truncate_safe;
                let short_cmd = truncate_safe(command, 47);
                println!("{} {}", "→".dimmed(), short_cmd.dimmed());
            }
        }
    }

    /// Fallback 警告
    pub fn fallback_warning(mode: DisplayMode, reason: &str) {
        if mode.show_fallback() {
            if mode.show_debug() {
                println!(
                    "{} {}",
                    "⚠️  LLM 生成失败，降级到规则匹配:".yellow(),
                    reason
                );
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
    pub fn task_execution_start(
        mode: DisplayMode,
        goal: &str,
        total_stages: usize,
        total_tasks: usize,
    ) {
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
    pub fn stage_execution(
        mode: DisplayMode,
        stage_num: usize,
        total_stages: usize,
        execution_mode: &str,
    ) {
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
            println!("{} {} {}", "→".dimmed(), task_name, percentage.dimmed());
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
    pub fn progress_bar(
        mode: DisplayMode,
        completed: usize,
        total: usize,
        elapsed: u32,
        remaining: u32,
    ) {
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

            println!("{} {:.0}% {}", bar, percentage, time_info.dimmed());
        }
    }

    /// 上下文溢出错误显示
    pub fn context_overflow_error(mode: DisplayMode, requested: usize, limit: usize) {
        eprintln!("{} 上下文长度超限", "❌".red());

        if mode.show_debug() || mode.show_timing() {
            eprintln!("  请求: {} tokens", requested.to_string().red());
            eprintln!("  限制: {} tokens", limit.to_string().yellow());
            eprintln!(
                "  超出: {} tokens",
                (requested.saturating_sub(limit)).to_string().red().bold()
            );

            eprintln!("\n{}", "💡 优化建议:".cyan());
            eprintln!("  1. 减少记忆容量: /memory config --capacity 50");
            eprintln!("  2. 清理历史记忆: /memory clear");
            eprintln!("  3. 使用更简洁的提问");
            eprintln!("  4. 减少工具数量（当前14+个工具）");
        } else {
            // Minimal 模式：只显示简要信息
            eprintln!(
                "  超出 {} tokens，使用 /help 查看优化建议",
                (requested.saturating_sub(limit)).to_string().red()
            );
        }
    }

    /// 上下文统计显示（仅 debug 模式）
    pub fn context_stats(
        mode: DisplayMode,
        memory_tokens: usize,
        tool_tokens: usize,
        system_tokens: usize,
        user_tokens: usize,
        max_tokens: usize,
    ) {
        if mode.show_debug() {
            let total = memory_tokens + tool_tokens + system_tokens + user_tokens;
            let percentage = (total as f64 / max_tokens as f64) * 100.0;

            println!("\n{}", "📊 上下文统计".cyan().bold());
            println!("  Memory: {} tokens", memory_tokens.to_string().yellow());
            println!("  Tools: {} tokens", tool_tokens.to_string().yellow());
            println!("  System: {} tokens", system_tokens.to_string().dimmed());
            println!("  User: {} tokens", user_tokens.to_string().dimmed());
            println!(
                "  Total: {}/{} tokens ({:.1}%)",
                total.to_string().bold(),
                max_tokens.to_string().dimmed(),
                percentage
            );

            if percentage > 90.0 {
                println!("\n{} 上下文使用率过高，建议清理", "⚠️".yellow());
            }
        }
    }

    /// 任务执行统计
    pub fn task_statistics(
        mode: DisplayMode,
        completed: usize,
        failed: usize,
        skipped: usize,
        total: usize,
        total_time: u32,
    ) {
        if mode.show_timing() {
            let success_rate = if total > 0 {
                ((completed - failed) as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            let status = if failed == 0 {
                "✓".green()
            } else {
                "⚠".yellow()
            };

            println!(
                "{} {} · {} · {:.0}% · {}s",
                status,
                format!("{}/{}", completed, total).bold(),
                format!("{}✓ {}✗ {}⊘", completed - failed, failed, skipped).dimmed(),
                success_rate,
                total_time
            );
        }
    }

    /// LLM 对话轮次调试信息（仅 debug 模式）
    /// 用于分析上下文使用和 workflow 套路重用
    pub fn conversation_rounds_debug(mode: DisplayMode, rounds: &[ConversationRoundInfo]) {
        if !mode.show_debug() {
            return;
        }

        println!("\n{}", "🔍 LLM 对话轮次详情".cyan().bold());
        println!("{}", "━".repeat(60).dimmed());

        for round in rounds {
            println!(
                "\n{} {} {} {}ms",
                format!("第 {} 轮", round.round).cyan().bold(),
                "│".dimmed(),
                format!("{} 条消息", round.message_count).yellow(),
                round.duration_ms.to_string().dimmed()
            );

            // 输入摘要
            if !round.input_summary.is_empty() {
                println!(
                    "  {} {}",
                    "输入:".dimmed(),
                    Self::truncate(&round.input_summary, 80)
                );
            }

            // LLM 响应
            if let Some(ref response) = round.assistant_response {
                println!("  {} {}", "响应:".green(), Self::truncate(response, 80));
            }

            // 工具调用
            if !round.tool_calls.is_empty() {
                println!("  {} {} 个工具", "工具:".yellow(), round.tool_calls.len());
                for (idx, tool) in round.tool_calls.iter().enumerate() {
                    println!(
                        "    {}. {} {}",
                        idx + 1,
                        tool.name.cyan(),
                        Self::truncate(&tool.arguments, 60).dimmed()
                    );
                }
            }

            // 工具结果
            if !round.tool_results.is_empty() {
                println!("  {} {} 个结果", "结果:".dimmed(), round.tool_results.len());
                for (idx, result) in round.tool_results.iter().enumerate() {
                    println!("    {}. {}", idx + 1, Self::truncate(result, 70).dimmed());
                }
            }
        }

        println!("\n{}", "━".repeat(60).dimmed());
        println!(
            "{} 总计 {} 轮对话，{} 条消息",
            "📊".dimmed(),
            rounds.len().to_string().cyan(),
            rounds
                .iter()
                .map(|r| r.message_count)
                .sum::<usize>()
                .to_string()
                .yellow()
        );
        println!("{} 便于分析和 workflow 套路重用", "💡".cyan());
    }

    /// 任务执行结果显示（/execute 命令）
    ///
    /// 根据 DisplayMode 显示不同详细程度的执行结果：
    /// - Minimal: 一行摘要
    /// - Standard: 摘要 + 失败任务列表
    /// - Debug: 完整执行计划结构 + 所有任务详情 + 输出内容 + 时间统计
    pub fn task_execution_result(
        mode: DisplayMode,
        result: &ExecutionResult,
        plan: Option<&ExecutionPlan>,
    ) {
        // 状态图标
        let status_icon = if result.is_success() {
            "✓".green()
        } else {
            "✗".red()
        };

        // === Minimal 模式：仅一行摘要 ===
        if !mode.show_timing() {
            println!(
                "{} {} · {:.0}%",
                status_icon,
                format!("{}/{}", result.completed_tasks, result.total_tasks).bold(),
                result.success_rate() * 100.0
            );

            // 失败时显示失败任务名称
            if result.failed_tasks > 0 {
                for task_result in &result.task_results {
                    if matches!(task_result.status, TaskStatus::Failed) {
                        println!("  {} {}", "✗".red(), task_result.task.name);
                    }
                }
            }
            return;
        }

        // === Standard 模式：摘要 + 所有任务输出 ===
        if !mode.show_debug() {
            println!(
                "\n{} {} · {} · {}秒",
                status_icon,
                format!("{}/{}", result.completed_tasks, result.total_tasks).bold(),
                format!("{:.0}%", result.success_rate() * 100.0).dimmed(),
                result.total_time
            );

            // ✨ v1.19.0: 显示所有任务的输出（不只是失败的）
            if !result.task_results.is_empty() {
                println!(); // 空行分隔

                for task_result in &result.task_results {
                    // 任务状态图标
                    let task_icon = match task_result.status {
                        TaskStatus::Success => "✓".green(),
                        TaskStatus::Failed => "✗".red(),
                        TaskStatus::Skipped => "⊘".yellow(),
                        TaskStatus::Cancelled => "⊗".dimmed(),
                        _ => "•".dimmed(),
                    };

                    // 任务名称
                    println!("  {} {}", task_icon, task_result.task.name);

                    // 显示输出内容（限制 50 行）
                    let max_lines = 50;
                    if !task_result.output.trim().is_empty() {
                        let lines: Vec<&str> = task_result.output.lines().collect();
                        let display_lines = lines.iter().take(max_lines);

                        for line in display_lines {
                            println!("    {}", line.dimmed());
                        }

                        // 如果输出超过限制，显示提示
                        if lines.len() > max_lines {
                            println!(
                                "    {} (省略 {} 行)",
                                "...".dimmed(),
                                lines.len() - max_lines
                            );
                        }
                    }

                    // 失败时显示错误信息
                    if let Some(error) = &task_result.error {
                        println!("    {} {}", "错误:".red(), error.red());
                    }
                }
            }
            return;
        }

        // === Debug 模式：完整信息 ===
        println!("\n{}", "═══ 任务执行结果 ═══".cyan().bold());

        // 1. 执行计划概览
        if let Some(plan) = plan {
            println!("\n{}", "📋 执行计划".cyan().bold());
            println!("  目标: {}", plan.goal.bold());
            println!(
                "  阶段: {} 个（{} 个并行）",
                plan.stages.len(),
                plan.parallel_stages
            );
            println!("  预估时间: {}秒", plan.total_estimated_time);

            // 显示阶段结构
            for (idx, stage) in plan.stages.iter().enumerate() {
                let mode_icon = match stage.execution_mode {
                    crate::task::ExecutionMode::Sequential => "→",
                    crate::task::ExecutionMode::Parallel => "⇉",
                };
                println!(
                    "    {} Stage {} {} {} 个任务 · {}秒",
                    mode_icon,
                    idx + 1,
                    "│".dimmed(),
                    stage.tasks.len(),
                    stage.estimated_time
                );
            }
        }

        // 2. 执行摘要
        println!("\n{}", "📊 执行摘要".cyan().bold());
        println!(
            "  {} {} · 成功率 {:.0}% · 总耗时 {}秒",
            status_icon,
            format!("{}/{}", result.completed_tasks, result.total_tasks).bold(),
            result.success_rate() * 100.0,
            result.total_time
        );

        if result.failed_tasks > 0 {
            println!("  失败: {} 个任务", result.failed_tasks);
        }
        if result.skipped_tasks > 0 {
            println!("  跳过: {} 个任务", result.skipped_tasks);
        }

        // 3. 所有任务的详细执行信息
        println!("\n{}", "🔍 任务详情".cyan().bold());

        for (idx, task_result) in result.task_results.iter().enumerate() {
            // 状态图标
            let task_icon = match task_result.status {
                TaskStatus::Success => "✓".green(),
                TaskStatus::Failed => "✗".red(),
                TaskStatus::Skipped => "⊘".yellow(),
                TaskStatus::Cancelled => "⊗".dimmed(),
                _ => "•".dimmed(),
            };

            // 任务信息行
            println!(
                "\n  {} {} {}",
                task_icon,
                format!("[{}/{}]", idx + 1, result.total_tasks).dimmed(),
                task_result.task.name.bold()
            );

            // 命令
            println!(
                "     {} {}",
                "命令:".dimmed(),
                task_result.task.command.dimmed()
            );

            // 时间对比
            println!(
                "     {} {}秒（预估 {}秒）",
                "耗时:".dimmed(),
                task_result.duration,
                task_result.task.estimated_time
            );

            // 任务类型
            if task_result.task.task_type != crate::task::TaskType::Shell {
                println!(
                    "     {} {}",
                    "类型:".dimmed(),
                    task_result.task.task_type.to_string().dimmed()
                );
            }

            // 输出内容（关键！）
            if !task_result.output.is_empty() {
                println!("     {}", "输出:".bold());
                for line in task_result.output.lines().take(20) {
                    // 限制显示行数
                    println!("       {}", line.dimmed());
                }
                if task_result.output.lines().count() > 20 {
                    println!(
                        "       {}",
                        format!("... ({} 行被截断)", task_result.output.lines().count() - 20)
                            .dimmed()
                    );
                }
            }

            // 错误信息
            if let Some(error) = &task_result.error {
                println!("     {}", "错误:".bold());
                for line in error.lines() {
                    println!("       {}", line.red());
                }
            }
        }

        // 4. 时间统计
        if let Some(plan) = plan {
            let time_saved = plan.total_estimated_time.saturating_sub(result.total_time);
            println!("\n{}", "⏱️  时间统计".cyan().bold());
            println!("  预估时间: {}秒", plan.total_estimated_time);
            println!("  实际时间: {}秒", result.total_time);
            if time_saved > 0 {
                println!("  节省时间: {}秒 {}", time_saved, "✓".green());
            } else if result.total_time > plan.total_estimated_time {
                let overtime = result.total_time - plan.total_estimated_time;
                println!("  超时: {}秒", overtime);
            }
        }

        println!("\n{}", "═".repeat(30).dimmed());
    }

    /// 截断长文本（按字符数而非字节数）
    fn truncate(text: &str, max_len: usize) -> String {
        let char_count = text.chars().count();
        if char_count <= max_len {
            text.to_string()
        } else {
            // 使用 chars().take() 保证在字符边界截断
            let truncated: String = text.chars().take(max_len.saturating_sub(3)).collect();
            format!("{}...", truncated)
        }
    }
}

/// 对话轮次信息（用于 Display）
#[derive(Debug, Clone)]
pub struct ConversationRoundInfo {
    pub round: usize,
    pub input_summary: String,
    pub assistant_response: Option<String>,
    pub tool_calls: Vec<ToolCallInfo>,
    pub tool_results: Vec<String>,
    pub message_count: usize,
    pub duration_ms: u64,
}

/// 工具调用信息（用于 Display）
#[derive(Debug, Clone)]
pub struct ToolCallInfo {
    pub name: String,
    pub arguments: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_mode_from_str() {
        assert_eq!("minimal".parse::<DisplayMode>(), Ok(DisplayMode::Minimal));
        assert_eq!("min".parse::<DisplayMode>(), Ok(DisplayMode::Minimal));
        assert_eq!("standard".parse::<DisplayMode>(), Ok(DisplayMode::Standard));
        assert_eq!("std".parse::<DisplayMode>(), Ok(DisplayMode::Standard));
        assert_eq!("debug".parse::<DisplayMode>(), Ok(DisplayMode::Debug));
        assert_eq!("dbg".parse::<DisplayMode>(), Ok(DisplayMode::Debug));
        assert!("unknown".parse::<DisplayMode>().is_err());
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

    #[test]
    fn test_truncate_chinese_chars() {
        // 测试中文字符截断（UTF-8边界问题）
        let text = "📅 公历 2025年10月20日\n农历 二零二五 乙巳、蛇年 八月 廿九";
        let result = Display::truncate(text, 80);
        // 应该能正常截断，不会panic
        assert!(result.len() > 0);

        // 测试短文本
        let short_text = "今天农历几号";
        let result = Display::truncate(short_text, 80);
        assert_eq!(result, short_text);

        // 测试需要截断的情况
        let long_text = "这是一个非常长的中文字符串，需要被截断处理";
        let result = Display::truncate(long_text, 10);
        assert!(result.ends_with("..."));
        assert!(result.chars().count() <= 10);
    }
}
