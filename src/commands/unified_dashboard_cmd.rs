//! 统一Dashboard命令
//!
//! ✨ v1.15.0 Phase 4: 一键查看三系（Liangyyi/Tracer/Bagua）状态

use crate::bagua::palace::BaguaMemoryPalace;
use crate::command::{Command, CommandRegistry};
use crate::liangyyi::StateTracker;
use crate::tracer::UnifiedTracer;
use anyhow::Result;
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册统一Dashboard命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `tracker`: StateTracker 实例（可选）
/// - `tracer`: UnifiedTracer 实例（可选）
/// - `bagua`: BaguaMemoryPalace 实例（可选）
pub fn register_unified_dashboard_command(
    registry: &mut CommandRegistry,
    tracker: Option<Arc<StateTracker>>,
    tracer: Option<Arc<UnifiedTracer>>,
    bagua: Option<Arc<RwLock<BaguaMemoryPalace>>>,
) {
    let cmd = Command::from_fn("system", "系统状态总览（三系协同）", move |args| {
        handle_system_command(args, tracker.clone(), tracer.clone(), bagua.clone())
    })
    .with_group("system");

    registry.register(cmd);
}

/// 处理 /system 命令
fn handle_system_command(
    args: &str,
    tracker: Option<Arc<StateTracker>>,
    tracer: Option<Arc<UnifiedTracer>>,
    bagua: Option<Arc<RwLock<BaguaMemoryPalace>>>,
) -> String {
    let args_str = args.trim();

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let dashboard = UnifiedDashboard::new(tracker, tracer, bagua);

            let subcommand = if args_str.is_empty() || args_str == "status" {
                "status"
            } else {
                args_str
            };

            match dashboard.execute(subcommand).await {
                Ok(output) => output,
                Err(e) => format!("{} 执行失败: {}", "❌".red(), e),
            }
        })
    })
}

/// 统一Dashboard处理器
pub struct UnifiedDashboard {
    tracker: Option<Arc<StateTracker>>,
    tracer: Option<Arc<UnifiedTracer>>,
    bagua: Option<Arc<RwLock<BaguaMemoryPalace>>>,
}

impl UnifiedDashboard {
    /// 创建Dashboard处理器
    pub fn new(
        tracker: Option<Arc<StateTracker>>,
        tracer: Option<Arc<UnifiedTracer>>,
        bagua: Option<Arc<RwLock<BaguaMemoryPalace>>>,
    ) -> Self {
        Self {
            tracker,
            tracer,
            bagua,
        }
    }

    /// 执行命令
    pub async fn execute(&self, subcommand: &str) -> Result<String> {
        match subcommand {
            "status" => self.show_status().await,
            "dashboard" => self.show_dashboard().await,
            "help" | "-h" | "--help" => Ok(self.show_help()),
            unknown => Ok(format!(
                "{} {}\n{}\n  - status (默认) - 简洁状态一览\n  - dashboard - 详细Dashboard\n  - help - 显示帮助",
                "❌ 未知子命令:".red(),
                unknown,
                "可用子命令:".yellow()
            )),
        }
    }

    /// 显示简洁的全局状态
    async fn show_status(&self) -> Result<String> {
        let mut output = String::new();

        // 标题
        output.push_str(&format!(
            "\n{}\n",
            "📊 系统状态总览".bright_cyan().bold()
        ));
        output.push_str(&format!(
            "{}\n\n",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                .dimmed()
        ));

        // 1. Liangyyi 自适应系统
        if let Some(tracker) = &self.tracker {
            let vector = tracker.to_state_vector().await;
            let is_adaptive = tracker.is_adaptive_enabled();

            output.push_str(&format!("{}\n", "🧘 自适应系统 (Liangyyi)".green().bold()));
            output.push_str(&format!(
                "   效率: {:.2} | 活动: {:.2} | 负载: {:.2} | 上下文: {:.2}\n",
                vector.get("efficiency").unwrap_or(0.0),
                vector.get("activity").unwrap_or(0.0),
                vector.get("load").unwrap_or(0.0),
                vector.get("context").unwrap_or(0.0)
            ));

            if is_adaptive {
                output.push_str(&format!("   状态: {} 自动优化已启用\n", "✓".green()));
            } else {
                output.push_str(&format!("   状态: {} 自动优化未启用\n", "○".dimmed()));
            }
        } else {
            output.push_str(&format!(
                "{}  {}\n",
                "🧘 自适应系统 (Liangyyi)".dimmed(),
                "未启用".yellow()
            ));
        }

        // 2. Tracer 观测系统
        output.push_str("\n");
        if let Some(tracer) = &self.tracer {
            let stats = tracer.stats().await?;

            output.push_str(&format!("{}\n", "🔍 观测系统 (Tracer)".green().bold()));
            output.push_str(&format!("   总条目: {}\n", stats.total_entries));
            output.push_str(&format!(
                "   各维度: Statistics({}) | Coordination({}) | BlackBox({}) | Memory({})\n",
                stats.by_dimension.get(&crate::tracer::Dimension::Statistics).unwrap_or(&0),
                stats.by_dimension.get(&crate::tracer::Dimension::Coordination).unwrap_or(&0),
                stats.by_dimension.get(&crate::tracer::Dimension::BlackBox).unwrap_or(&0),
                stats.by_dimension.get(&crate::tracer::Dimension::Memory).unwrap_or(&0)
            ));
        } else {
            output.push_str(&format!(
                "{}  {}\n",
                "🔍 观测系统 (Tracer)".dimmed(),
                "未启用".yellow()
            ));
        }

        // 3. Bagua 炼化系统
        output.push_str("\n");
        if let Some(bagua_lock) = &self.bagua {
            let bagua = bagua_lock.read().await;
            let tracer_enabled = bagua.is_tracer_enabled();

            output.push_str(&format!("{}\n", "🌊 炼化系统 (Bagua)".green().bold()));

            // 获取离坎平衡状态
            let likan_balance = bagua.check_likan_balance().await;
            output.push_str(&format!(
                "   离坎平衡: 离({}) / 坎({}) | 平衡度: {:.2}\n",
                likan_balance.li_count,
                likan_balance.kan_count,
                likan_balance.balance
            ));

            if tracer_enabled {
                output.push_str(&format!("   状态: {} Tracer 集成已启用\n", "✓".green()));
            } else {
                output.push_str(&format!("   状态: {} Tracer 集成未启用\n", "○".dimmed()));
            }
        } else {
            output.push_str(&format!(
                "{}  {}\n",
                "🌊 炼化系统 (Bagua)".dimmed(),
                "未启用".yellow()
            ));
        }

        // 系统协同状态
        output.push_str("\n");
        output.push_str(&format!("{}\n", "🔗 系统协同".cyan().bold()));
        let integration_count = [
            self.tracker.is_some(),
            self.tracer.is_some(),
            self.bagua.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        match integration_count {
            3 => output.push_str(&format!("   {} 三系完整集成\n", "✓".green())),
            2 => output.push_str(&format!("   {} 部分系统集成\n", "⚠".yellow())),
            1 => output.push_str(&format!("   {} 单系统运行\n", "⚠".yellow())),
            _ => output.push_str(&format!("   {} 无系统启用\n", "✗".red())),
        }

        // 快捷命令提示
        output.push_str(&format!(
            "\n{}\n",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                .dimmed()
        ));
        output.push_str(&format!("💡 {} 查看详细Dashboard\n", "/system dashboard".cyan()));
        output.push_str(&format!("💡 {} 查看自适应优化历史\n", "/liangyyi adaptive".cyan()));
        output.push_str(&format!("💡 {} 查看追踪数据\n", "/trace".cyan()));
        output.push_str(&format!("💡 {} 查看炼化状态\n\n", "/likan status".cyan()));

        Ok(output)
    }

    /// 显示详细Dashboard
    async fn show_dashboard(&self) -> Result<String> {
        let mut output = String::new();

        // 标题
        output.push_str(&format!(
            "\n{}\n",
            "╔═══════════════════════════════════════════════════════════╗"
                .bright_cyan()
        ));
        output.push_str(&format!(
            "{}\n",
            "║          系统Dashboard - 三系协同状态全览                ║"
                .bright_cyan()
                .bold()
        ));
        output.push_str(&format!(
            "{}\n\n",
            "╚═══════════════════════════════════════════════════════════╝"
                .bright_cyan()
        ));

        // === 1. Liangyyi 自适应系统详情 ===
        if let Some(tracker) = &self.tracker {
            output.push_str(&format!(
                "{}\n",
                "┌─────────────────────────────────────────────────────────┐"
                    .blue()
            ));
            output.push_str(&format!(
                "│ {}                                      │\n",
                "🧘 自适应系统 (Liangyyi)".green().bold()
            ));
            output.push_str(&format!(
                "{}\n",
                "└─────────────────────────────────────────────────────────┘"
                    .blue()
            ));

            let vector = tracker.to_state_vector().await;

            output.push_str(&format!(
                "   📐 {}:\n",
                "状态向量".bright_white().bold()
            ));
            output.push_str(&format!("      效率: {:.3}\n", vector.get("efficiency").unwrap_or(0.0)));
            output.push_str(&format!("      活动: {:.3}\n", vector.get("activity").unwrap_or(0.0)));
            output.push_str(&format!("      负载: {:.3}\n", vector.get("load").unwrap_or(0.0)));
            output.push_str(&format!("      上下文: {:.3}\n", vector.get("context").unwrap_or(0.0)));

            if tracker.is_adaptive_enabled() {
                let stats = tracker.get_optimization_stats().await;
                output.push_str(&format!(
                    "\n   🎯 {}:\n",
                    "自动优化统计".bright_white().bold()
                ));
                output.push_str(&format!("      优化次数: {}\n", stats.total_optimizations));
                output.push_str(&format!("      成功次数: {}\n", stats.successful_optimizations));
                output.push_str(&format!(
                    "      失败次数: {}\n",
                    stats.failed_optimizations
                ));
                output.push_str(&format!(
                    "      平均耗时: {} ms\n",
                    stats.avg_duration_ms
                ));
                output.push_str(&format!(
                    "      平均建议数: {}\n",
                    stats.avg_recommendations_per_run
                ));
            }

            output.push_str("\n");
        }

        // === 2. Tracer 观测系统详情 ===
        if let Some(tracer) = &self.tracer {
            output.push_str(&format!(
                "{}\n",
                "┌─────────────────────────────────────────────────────────┐"
                    .blue()
            ));
            output.push_str(&format!(
                "│ {}                                         │\n",
                "🔍 观测系统 (Tracer)".green().bold()
            ));
            output.push_str(&format!(
                "{}\n",
                "└─────────────────────────────────────────────────────────┘"
                    .blue()
            ));

            let stats = tracer.stats().await?;

            output.push_str(&format!(
                "   📊 {}:\n",
                "统计概览".bright_white().bold()
            ));
            output.push_str(&format!("      总条目数: {}\n", stats.total_entries));

            output.push_str(&format!(
                "\n   🔢 {}:\n",
                "各维度统计".bright_white().bold()
            ));
            output.push_str(&format!(
                "      📊 Statistics: {}\n",
                stats
                    .by_dimension
                    .get(&crate::tracer::Dimension::Statistics)
                    .unwrap_or(&0)
            ));
            output.push_str(&format!(
                "      🔗 Coordination: {}\n",
                stats
                    .by_dimension
                    .get(&crate::tracer::Dimension::Coordination)
                    .unwrap_or(&0)
            ));
            output.push_str(&format!(
                "      🤖 BlackBox: {}\n",
                stats
                    .by_dimension
                    .get(&crate::tracer::Dimension::BlackBox)
                    .unwrap_or(&0)
            ));
            output.push_str(&format!(
                "      💭 Memory: {}\n",
                stats
                    .by_dimension
                    .get(&crate::tracer::Dimension::Memory)
                    .unwrap_or(&0)
            ));

            output.push_str(&format!(
                "\n   ⚡ {}:\n",
                "自定义事件统计".bright_white().bold()
            ));
            output.push_str(&format!(
                "      事件总数: {}\n",
                tracer.custom_entries_count().await
            ));

            output.push_str("\n");
        }

        // === 3. Bagua 炼化系统详情 ===
        if let Some(bagua_lock) = &self.bagua {
            let bagua = bagua_lock.read().await;

            output.push_str(&format!(
                "{}\n",
                "┌─────────────────────────────────────────────────────────┐"
                    .blue()
            ));
            output.push_str(&format!(
                "│ {}                                         │\n",
                "🌊 炼化系统 (Bagua)".green().bold()
            ));
            output.push_str(&format!(
                "{}\n",
                "└─────────────────────────────────────────────────────────┘"
                    .blue()
            ));

            let likan_balance = bagua.check_likan_balance().await;

            output.push_str(&format!(
                "   ☯️  {}:\n",
                "离坎能量平衡".bright_white().bold()
            ));
            output.push_str(&format!(
                "      离能量: {:.2} ({}条)\n",
                likan_balance.li_energy, likan_balance.li_count
            ));
            output.push_str(&format!(
                "      坎能量: {:.2} ({}条)\n",
                likan_balance.kan_energy, likan_balance.kan_count
            ));

            let balance_str = if likan_balance.balance > 0.1 {
                format!("{:.2} (离强)", likan_balance.balance).red()
            } else if likan_balance.balance < -0.1 {
                format!("{:.2} (坎强)", likan_balance.balance).cyan()
            } else {
                format!("{:.2} (平衡)", likan_balance.balance).green()
            };
            output.push_str(&format!("      平衡度: {}\n", balance_str));

            if bagua.is_tracer_enabled() {
                output.push_str(&format!(
                    "\n   🔗 {}:  {}\n",
                    "Tracer集成".bright_white().bold(),
                    "已启用".green()
                ));
            }

            output.push_str("\n");
        }

        // 系统协同总结
        output.push_str(&format!(
            "{}\n",
            "┌─────────────────────────────────────────────────────────┐"
                .yellow()
        ));
        output.push_str(&format!(
            "│ {}                                                 │\n",
            "🔗 系统协同状态".cyan().bold()
        ));
        output.push_str(&format!(
            "{}\n",
            "└─────────────────────────────────────────────────────────┘"
                .yellow()
        ));

        let tracker_enabled = self.tracker.is_some();
        let tracer_enabled = self.tracer.is_some();
        let bagua_enabled = self.bagua.is_some();

        output.push_str(&format!(
            "   Liangyyi → Tracer: {}\n",
            if tracker_enabled && tracer_enabled && self.tracker.as_ref().unwrap().is_tracer_enabled() {
                "✓ 已集成".green()
            } else {
                "○ 未集成".dimmed()
            }
        ));

        let bagua_tracer_integrated = if bagua_enabled && tracer_enabled {
            self.bagua.as_ref().unwrap().read().await.is_tracer_enabled()
        } else {
            false
        };

        output.push_str(&format!(
            "   Bagua → Tracer:    {}\n",
            if bagua_tracer_integrated {
                "✓ 已集成".green()
            } else {
                "○ 未集成".dimmed()
            }
        ));

        output.push_str(&format!(
            "   三系协同:          {}\n",
            if tracker_enabled && tracer_enabled && bagua_enabled {
                "✓ 完整".green()
            } else if tracker_enabled || tracer_enabled || bagua_enabled {
                "⚠ 部分".yellow()
            } else {
                "✗ 无".red()
            }
        ));

        output.push_str(&format!(
            "\n{}\n",
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
                .dimmed()
        ));
        output.push_str(&format!(
            "💡 使用 {} 查看简洁状态\n\n",
            "/system status".cyan()
        ));

        Ok(output)
    }

    /// 显示帮助信息
    fn show_help(&self) -> String {
        format!(
            r#"
{}
{}

{}:
  /system status     - 简洁的系统状态一览
  /system dashboard  - 详细的三系Dashboard
  /system help       - 显示此帮助

{}:
  Liangyyi (🧘)    - 自适应配置优化系统
  Tracer (🔍)      - 四维观测追踪系统
  Bagua (🌊)       - 记忆炼化处理系统

{}:
  /liangyyi        - 查看Liangyyi详情
  /trace           - 查看Tracer追踪数据
  /likan status    - 查看Bagua炼化状态

"#,
            "📊 统一系统Dashboard".bright_cyan().bold(),
            "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed(),
            "用法".yellow().bold(),
            "三系说明".yellow().bold(),
            "相关命令".yellow().bold()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_dashboard_no_systems() {
        let dashboard = UnifiedDashboard::new(None, None, None);
        let output = dashboard.show_status().await.unwrap();
        assert!(output.contains("未启用"));
    }

    #[tokio::test]
    async fn test_unified_dashboard_help() {
        let dashboard = UnifiedDashboard::new(None, None, None);
        let output = dashboard.show_help();
        assert!(output.contains("统一系统Dashboard"));
        assert!(output.contains("Liangyyi"));
        assert!(output.contains("Tracer"));
        assert!(output.contains("Bagua"));
    }
}
