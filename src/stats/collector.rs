//! 统计收集器
//!
//! 负责收集和管理所有统计数据

use super::metrics::{CommandMetrics, LlmMetrics, PerformanceMetrics, ToolMetrics};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 统计事件
#[derive(Debug, Clone)]
pub enum StatEvent {
    /// LLM 调用
    LlmCall {
        success: bool,
        duration: Duration,
        tokens: u64,
    },

    /// 工具调用
    ToolCall {
        tool_name: String,
        success: bool,
        duration: Duration,
    },

    /// 命令执行
    CommandExecution {
        command: String,
        success: bool,
        duration: Duration,
    },
}

/// 统计收集器
pub struct StatsCollector {
    /// LLM 统计
    pub llm_metrics: Arc<RwLock<LlmMetrics>>,

    /// 工具统计
    pub tool_metrics: Arc<RwLock<ToolMetrics>>,

    /// 命令统计
    pub command_metrics: Arc<RwLock<CommandMetrics>>,

    /// 性能统计
    pub performance_metrics: Arc<RwLock<PerformanceMetrics>>,
}

impl StatsCollector {
    /// 创建新的统计收集器
    pub fn new() -> Self {
        Self {
            llm_metrics: Arc::new(RwLock::new(LlmMetrics::new())),
            tool_metrics: Arc::new(RwLock::new(ToolMetrics::new())),
            command_metrics: Arc::new(RwLock::new(CommandMetrics::new())),
            performance_metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
        }
    }

    /// 记录事件
    pub async fn record(&self, event: StatEvent) {
        match event {
            StatEvent::LlmCall {
                success,
                duration,
                tokens,
            } => {
                let mut metrics = self.llm_metrics.write().await;
                metrics.record_call(success, duration, tokens);
            }

            StatEvent::ToolCall {
                tool_name,
                success,
                duration,
            } => {
                let mut metrics = self.tool_metrics.write().await;
                metrics.record_call(&tool_name, success, duration);
            }

            StatEvent::CommandExecution {
                command,
                success,
                duration,
            } => {
                // 更新命令统计
                {
                    let mut metrics = self.command_metrics.write().await;
                    metrics.record_command(success, duration);
                }

                // 更新性能统计
                {
                    let mut metrics = self.performance_metrics.write().await;
                    metrics.record_response(duration, &command);
                }
            }
        }
    }

    /// 获取 LLM 统计的克隆
    pub async fn get_llm_metrics(&self) -> LlmMetrics {
        self.llm_metrics.read().await.clone()
    }

    /// 获取工具统计的克隆
    pub async fn get_tool_metrics(&self) -> ToolMetrics {
        self.tool_metrics.read().await.clone()
    }

    /// 获取命令统计的克隆
    pub async fn get_command_metrics(&self) -> CommandMetrics {
        self.command_metrics.read().await.clone()
    }

    /// 获取性能统计的克隆
    pub async fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_metrics.read().await.clone()
    }

    /// 重置所有统计
    pub async fn reset(&self) {
        *self.llm_metrics.write().await = LlmMetrics::new();
        *self.tool_metrics.write().await = ToolMetrics::new();
        *self.command_metrics.write().await = CommandMetrics::new();
        *self.performance_metrics.write().await = PerformanceMetrics::new();
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stats_collector() {
        let collector = StatsCollector::new();

        // 记录 LLM 调用
        collector
            .record(StatEvent::LlmCall {
                success: true,
                duration: Duration::from_millis(800),
                tokens: 100,
            })
            .await;

        // 记录工具调用
        collector
            .record(StatEvent::ToolCall {
                tool_name: "calculator".to_string(),
                success: true,
                duration: Duration::from_millis(50),
            })
            .await;

        // 记录命令执行
        collector
            .record(StatEvent::CommandExecution {
                command: "test command".to_string(),
                success: true,
                duration: Duration::from_millis(1000),
            })
            .await;

        // 验证统计
        let llm_metrics = collector.get_llm_metrics().await;
        assert_eq!(llm_metrics.total_calls, 1);
        assert_eq!(llm_metrics.success_calls, 1);

        let tool_metrics = collector.get_tool_metrics().await;
        assert_eq!(tool_metrics.total_calls, 1);

        let command_metrics = collector.get_command_metrics().await;
        assert_eq!(command_metrics.total_commands, 1);
        assert_eq!(command_metrics.success_commands, 1);

        let perf_metrics = collector.get_performance_metrics().await;
        assert_eq!(perf_metrics.response_times.len(), 1);
        assert_eq!(perf_metrics.response_times[0], 1000);
    }

    #[tokio::test]
    async fn test_reset() {
        let collector = StatsCollector::new();

        collector
            .record(StatEvent::LlmCall {
                success: true,
                duration: Duration::from_millis(800),
                tokens: 100,
            })
            .await;

        // 验证有数据
        let llm_metrics = collector.get_llm_metrics().await;
        assert_eq!(llm_metrics.total_calls, 1);

        // 重置
        collector.reset().await;

        // 验证已清空
        let llm_metrics = collector.get_llm_metrics().await;
        assert_eq!(llm_metrics.total_calls, 0);
    }
}
