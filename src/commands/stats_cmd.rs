//! /dashboard 和 /stats 命令实现
//!
//! 用法：
//! - `/dashboard` - 显示完整系统仪表板
//! - `/stats` - 显示紧凑统计摘要

use crate::command::{Command, CommandRegistry};
use crate::stats::{Dashboard, StatsCollector};
use std::sync::Arc;

/// 注册统计命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `stats_collector`: 共享的统计收集器
pub fn register_stats_commands(
    registry: &mut CommandRegistry,
    stats_collector: Arc<StatsCollector>,
) {
    // 注册 /dashboard 命令
    {
        let collector = Arc::clone(&stats_collector);
        let cmd = Command::from_fn("dashboard", "显示系统仪表板", move |_args| {
            handle_dashboard(Arc::clone(&collector))
        })
        .with_group("stats");

        registry.register(cmd);
    }

    // 注册 /stats 命令
    {
        let collector = Arc::clone(&stats_collector);
        let cmd = Command::from_fn("stats", "显示统计摘要", move |_args| {
            handle_stats(Arc::clone(&collector))
        })
        .with_group("stats");

        registry.register(cmd);
    }
}

/// 处理 /dashboard 命令
fn handle_dashboard(stats_collector: Arc<StatsCollector>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let dashboard = Dashboard::new(stats_collector);
            dashboard.render().await
        })
    })
}

/// 处理 /stats 命令
fn handle_stats(stats_collector: Arc<StatsCollector>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let dashboard = Dashboard::new(stats_collector);
            dashboard.render_compact().await
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::StatEvent;
    use std::time::Duration;

    fn create_test_collector() -> Arc<StatsCollector> {
        let collector = Arc::new(StatsCollector::new());

        // 添加一些测试数据
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // LLM 调用
                collector
                    .record(StatEvent::LlmCall {
                        success: true,
                        duration: Duration::from_millis(800),
                        tokens: 100,
                    })
                    .await;

                collector
                    .record(StatEvent::LlmCall {
                        success: true,
                        duration: Duration::from_millis(1200),
                        tokens: 150,
                    })
                    .await;

                // 工具调用
                collector
                    .record(StatEvent::ToolCall {
                        tool_name: "calculator".to_string(),
                        success: true,
                        duration: Duration::from_millis(50),
                    })
                    .await;

                collector
                    .record(StatEvent::ToolCall {
                        tool_name: "shell_execute".to_string(),
                        success: true,
                        duration: Duration::from_millis(200),
                    })
                    .await;

                // 命令执行
                collector
                    .record(StatEvent::CommandExecution {
                        command: "test command 1".to_string(),
                        success: true,
                        duration: Duration::from_millis(1000),
                    })
                    .await;

                collector
                    .record(StatEvent::CommandExecution {
                        command: "test command 2".to_string(),
                        success: true,
                        duration: Duration::from_millis(500),
                    })
                    .await;
            })
        });

        collector
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_dashboard() {
        let collector = create_test_collector();
        let result = handle_dashboard(collector);

        // 验证输出包含关键信息
        assert!(result.contains("RealConsole System Dashboard"));
        assert!(result.contains("会话统计"));
        assert!(result.contains("LLM 统计"));
        assert!(result.contains("工具使用"));
        assert!(result.contains("性能指标"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_stats() {
        let collector = create_test_collector();
        let result = handle_stats(collector);

        // 验证输出包含紧凑统计信息
        assert!(result.contains("Stats"));
        assert!(result.contains("LLM"));
        assert!(result.contains("Tools"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_dashboard_with_empty_data() {
        let collector = Arc::new(StatsCollector::new());
        let result = handle_dashboard(collector);

        // 即使没有数据也应该能正常渲染
        assert!(result.contains("RealConsole System Dashboard"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_stats_with_empty_data() {
        let collector = Arc::new(StatsCollector::new());
        let result = handle_stats(collector);

        // 即使没有数据也应该能正常渲染
        assert!(result.contains("Stats"));
    }
}
