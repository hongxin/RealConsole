//! 两仪演化系统可视化命令
//!
//! 提供 /liangyyi 命令来展示系统状态

use crate::command::{Command, CommandRegistry};
use crate::liangyyi::{LearningPhase, Liangyyi, Sixiang, StateTracker, StateTrend};
use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;

/// 注册两仪演化系统命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `tracker`: 状态追踪器实例
pub fn register_liangyyi_commands(
    registry: &mut CommandRegistry,
    tracker: Option<Arc<StateTracker>>,
) {
    let cmd = Command::from_fn("liangyyi", "两仪演化系统状态查询", move |args| {
        handle_liangyyi(args, tracker.clone())
    })
    .with_group("liangyyi");

    registry.register(cmd);
}

/// 处理 /liangyyi 命令
fn handle_liangyyi(args: &str, tracker: Option<Arc<StateTracker>>) -> String {
    let args_str = args.trim();

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            if tracker.is_none() {
                return format!(
                    "{} 两仪追踪器未启用",
                    "⚠".yellow()
                );
            }

            let tracker = tracker.unwrap();
            let command = LiangyiCommand::new(tracker);

            let subcommand = if args_str.is_empty() || args_str == "status" {
                None
            } else {
                Some(args_str)
            };

            match command.execute(subcommand).await {
                Ok(output) => output,
                Err(e) => format!("{} 执行失败: {}", "❌".red(), e),
            }
        })
    })
}

/// 两仪命令处理器
pub struct LiangyiCommand {
    tracker: Arc<StateTracker>,
}

impl LiangyiCommand {
    /// 创建命令处理器
    pub fn new(tracker: Arc<StateTracker>) -> Self {
        Self { tracker }
    }

    /// 执行命令
    pub async fn execute(&self, subcommand: Option<&str>) -> Result<String> {
        match subcommand {
            None | Some("status") => self.show_status().await,
            Some("stats") => self.show_stats().await,
            Some("history") => self.show_history().await,
            Some("trend") => self.show_trend().await,
            Some(unknown) => Ok(format!(
                "{} {}\n{}\n  - status (默认)\n  - stats\n  - history\n  - trend",
                "❌ 未知子命令:".red(),
                unknown,
                "可用子命令:".yellow()
            )),
        }
    }

    /// 显示当前状态
    async fn show_status(&self) -> Result<String> {
        let current_snapshot = self.tracker.current_state().await;
        let current_taiji = &current_snapshot.taiji;
        let liangyyi = current_snapshot.liangyyi;
        let sixiang = current_snapshot.sixiang;
        let (learning_phase, volatility, change_rate) =
            self.tracker.detect_learning_phase().await;
        let trend = self.tracker.analyze_trend().await;
        let history = self.tracker.history().await;

        let mut output = String::new();

        // 标题
        output.push_str(&format!(
            "{}\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));
        output.push_str(&format!(
            "  {}\n",
            "两仪演化系统 - 当前状态".bright_cyan().bold()
        ));
        output.push_str(&format!(
            "{}\n\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));

        // 太极 - 阴阳能量
        output.push_str(&format!("{}\n", "【太极】阴阳能量".cyan().bold()));
        output.push_str(&format!(
            "  阴 {} {:.2}  (静、收、聚、藏)\n",
            self.energy_bar(current_taiji.yin_energy, 10).cyan(),
            current_taiji.yin_energy
        ));
        output.push_str(&format!(
            "  阳 {} {:.2}  (动、放、散、发)\n",
            self.energy_bar(current_taiji.yang_energy, 10).red(),
            current_taiji.yang_energy
        ));
        output.push_str(&format!(
            "  平衡度: {:.2}  |  能量强度: {:.2}\n\n",
            current_taiji.balance(),
            current_taiji.intensity()
        ));

        // 两仪 - 二元状态
        output.push_str(&format!("{}\n", "【两仪】二元状态".magenta().bold()));
        output.push_str(&format!(
            "  当前: {} ({})\n\n",
            liangyyi.symbol(),
            match liangyyi {
                Liangyyi::Taiyin => "阴主导".cyan(),
                Liangyyi::Taiyang => "阳主导".red(),
            }
        ));

        // 四象 - 四态循环
        output.push_str(&format!("{}\n", "【四象】四态循环".yellow().bold()));
        output.push_str(&format!("  当前: {} {}\n", sixiang.symbol(), {
            let desc = sixiang.description();
            match sixiang {
                Sixiang::LaoYin => format!("({})", desc).cyan(),
                Sixiang::ShaoYin => format!("({})", desc).bright_cyan(),
                Sixiang::ShaoYang => format!("({})", desc).bright_red(),
                Sixiang::LaoYang => format!("({})", desc).red(),
            }
        }));
        output.push_str(&format!(
            "  活动等级: {}/4\n\n",
            sixiang.activity_level()
        ));

        // 学习阶段
        output.push_str(&format!("{}\n", "【学习阶段】".green().bold()));
        output.push_str(&format!(
            "  阶段: {} {}\n",
            match learning_phase {
                LearningPhase::Exploration => "探索期".red(),
                LearningPhase::Stability => "稳定期".green(),
                LearningPhase::Transition => "转变期".yellow(),
            },
            match learning_phase {
                LearningPhase::Exploration => "🔴",
                LearningPhase::Stability => "🟢",
                LearningPhase::Transition => "🟡",
            }
        ));
        output.push_str(&format!(
            "  波动性: {:.3} {}\n",
            volatility,
            if volatility > 0.12 {
                "(高)".red()
            } else if volatility < 0.06 {
                "(低)".green()
            } else {
                "(中)".yellow()
            }
        ));
        output.push_str(&format!(
            "  变化率: {:.3} {}\n",
            change_rate,
            if change_rate > 0.4 {
                "(高)".red()
            } else if change_rate < 0.2 {
                "(低)".green()
            } else {
                "(中)".yellow()
            }
        ));
        output.push_str(&format!(
            "  趋势: {} {}\n\n",
            match trend {
                StateTrend::TowardYin => "趋向阴".cyan(),
                StateTrend::TowardYang => "趋向阳".red(),
                StateTrend::Stable => "稳定".green(),
            },
            match trend {
                StateTrend::TowardYin => "↓",
                StateTrend::TowardYang => "↑",
                StateTrend::Stable => "→",
            }
        ));

        // 时间信息
        output.push_str(&format!("{}\n", "【时间信息】".bright_black().bold()));
        output.push_str(&format!(
            "  快照时间: {}\n",
            current_taiji.timestamp.format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!("  历史记录: {} 个快照\n\n", history.len()));

        // 分隔线和提示
        output.push_str(&format!(
            "{}\n",
            "───────────────────────────────────────────────────────────".bright_black()
        ));
        output.push_str(&format!(
            "{}\n",
            "提示：使用 /liangyyi stats 查看详细统计".bright_black()
        ));

        Ok(output)
    }

    /// 显示统计信息
    async fn show_stats(&self) -> Result<String> {
        let stats = self.tracker.stats().await;

        let mut output = String::new();

        // 标题
        output.push_str(&format!(
            "{}\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));
        output.push_str(&format!(
            "  {}\n",
            "两仪演化系统 - 统计信息".bright_cyan().bold()
        ));
        output.push_str(&format!(
            "{}\n\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));

        // 基本统计
        output.push_str(&format!("{}\n", "【基本统计】".cyan().bold()));
        output.push_str(&format!("  总快照数: {}\n\n", stats.total_snapshots));

        // 能量统计
        output.push_str(&format!("{}\n", "【能量统计】".magenta().bold()));
        output.push_str(&format!(
            "  当前阴能量: {:.3}\n",
            stats.current_yin_energy
        ));
        output.push_str(&format!(
            "  当前阳能量: {:.3}\n",
            stats.current_yang_energy
        ));
        output.push_str(&format!("  平均平衡度: {:.3}\n\n", stats.avg_balance));

        // 状态分布
        output.push_str(&format!("{}\n", "【状态分布】".yellow().bold()));
        output.push_str("  四象分布:\n".to_string().as_str());

        let total = stats.total_snapshots as f64;
        for sixiang in Sixiang::all() {
            let count = stats.sixiang_counts.get(&sixiang).unwrap_or(&0);
            let percentage = if total > 0.0 {
                (*count as f64 / total) * 100.0
            } else {
                0.0
            };
            let bar_len = (percentage / 5.0) as usize; // 每5%一个字符
            let bar = "█".repeat(bar_len);

            output.push_str(&format!(
                "    {:<15}: {} {} 次 ({:.1}%)\n",
                format!("{} {}", sixiang.symbol(), sixiang.description()),
                bar,
                count,
                percentage
            ));
        }
        output.push('\n');

        // 学习阶段
        output.push_str(&format!("{}\n", "【学习阶段】".green().bold()));
        output.push_str(&format!(
            "  当前阶段: {} {}\n",
            match stats.learning_phase {
                LearningPhase::Exploration => "探索期".red(),
                LearningPhase::Stability => "稳定期".green(),
                LearningPhase::Transition => "转变期".yellow(),
            },
            match stats.learning_phase {
                LearningPhase::Exploration => "🔴",
                LearningPhase::Stability => "🟢",
                LearningPhase::Transition => "🟡",
            }
        ));
        output.push_str(&format!("  波动性: {:.3}\n", stats.volatility));
        output.push_str(&format!(
            "  变化率: {:.3}\n\n",
            stats.sixiang_change_rate
        ));

        // 分隔线
        output.push_str(&format!(
            "{}\n",
            "───────────────────────────────────────────────────────────".bright_black()
        ));

        Ok(output)
    }

    /// 显示历史（简化版）
    async fn show_history(&self) -> Result<String> {
        let history = self.tracker.recent_states(20).await;

        let mut output = String::new();

        output.push_str(&format!(
            "{}\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));
        output.push_str(&format!(
            "  {}\n",
            "两仪演化系统 - 最近历史".bright_cyan().bold()
        ));
        output.push_str(&format!(
            "{}\n\n",
            "═══════════════════════════════════════════════════════════".bright_blue()
        ));

        output.push_str(&format!(
            "{:<12} {:<4} {:<15} {}\n",
            "时间", "两仪", "四象", "能量平衡"
        ));
        output.push_str(&format!(
            "{}\n",
            "─────────────────────────────────────────────────────────".bright_black()
        ));

        for snapshot in history.iter().rev().take(10) {
            let balance_bar = self.energy_bar(snapshot.taiji.balance(), 10);
            output.push_str(&format!(
                "{} {} {:<15} [{}]\n",
                snapshot.timestamp.format("%H:%M:%S"),
                snapshot.liangyyi.symbol(),
                format!("{} {}", snapshot.sixiang.symbol(), snapshot.sixiang.description()),
                balance_bar
            ));
        }

        output.push('\n');
        output.push_str(&format!(
            "{}\n",
            "提示：使用 /liangyyi trend 查看趋势分析".bright_black()
        ));

        Ok(output)
    }

    /// 显示趋势（占位实现）
    async fn show_trend(&self) -> Result<String> {
        Ok(format!(
            "{}\n功能开发中，敬请期待...",
            "【趋势分析】".yellow().bold()
        ))
    }

    /// 生成能量条
    fn energy_bar(&self, value: f64, width: usize) -> String {
        let filled = (value * width as f64) as usize;
        let empty = width - filled;
        format!("{}{}", "▰".repeat(filled), "▱".repeat(empty))
    }

    /// 辅助方法：计算活动水平
    async fn calculate_activity_level(&self) -> f64 {
        let history = self.tracker.history().await;
        if history.is_empty() {
            return 0.5;
        }

        let recent_yang: f64 = history
            .iter()
            .rev()
            .take(10)
            .map(|s| s.taiji.yang_energy)
            .sum();

        let count = history.len().min(10) as f64;
        (recent_yang / count).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liangyyi::{Event, StateTrackerConfig};

    #[tokio::test]
    async fn test_liangyyi_status_display() {
        let config = StateTrackerConfig::default();
        let tracker = Arc::new(StateTracker::new(config));

        // 添加一些测试数据
        for _ in 0..10 {
            tracker.update_from_event(Event::UserRead).await;
        }

        let command = LiangyiCommand::new(tracker);
        let result = command.execute(None).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("两仪演化系统"));
        assert!(output.contains("【太极】"));
        assert!(output.contains("【两仪】"));
        assert!(output.contains("【四象】"));
        assert!(output.contains("【学习阶段】"));
    }

    #[tokio::test]
    async fn test_liangyyi_stats_display() {
        let config = StateTrackerConfig::default();
        let tracker = Arc::new(StateTracker::new(config));

        for _ in 0..10 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let command = LiangyiCommand::new(tracker);
        let result = command.execute(Some("stats")).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("统计信息"));
        assert!(output.contains("【基本统计】"));
        assert!(output.contains("【能量统计】"));
        assert!(output.contains("【状态分布】"));
    }

    #[tokio::test]
    async fn test_unknown_subcommand() {
        let config = StateTrackerConfig::default();
        let tracker = Arc::new(StateTracker::new(config));
        let command = LiangyiCommand::new(tracker);

        let result = command.execute(Some("unknown")).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("未知子命令"));
        assert!(output.contains("unknown"));
    }
}
