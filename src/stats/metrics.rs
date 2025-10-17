//! 统计指标数据结构
//!
//! 定义各种统计指标的数据结构

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// LLM 调用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMetrics {
    /// 总调用次数
    pub total_calls: u64,

    /// 成功次数
    pub success_calls: u64,

    /// 失败次数
    pub failed_calls: u64,

    /// 总响应时间（毫秒）
    pub total_response_time_ms: u64,

    /// 预估 token 使用量
    pub estimated_tokens: u64,

    /// 预估成本（美元）
    pub estimated_cost: f64,

    /// 最近更新时间
    pub last_updated: DateTime<Utc>,
}

impl LlmMetrics {
    pub fn new() -> Self {
        Self {
            total_calls: 0,
            success_calls: 0,
            failed_calls: 0,
            total_response_time_ms: 0,
            estimated_tokens: 0,
            estimated_cost: 0.0,
            last_updated: Utc::now(),
        }
    }

    /// 记录一次 LLM 调用
    pub fn record_call(&mut self, success: bool, duration: Duration, tokens: u64) {
        self.total_calls += 1;
        if success {
            self.success_calls += 1;
        } else {
            self.failed_calls += 1;
        }
        self.total_response_time_ms += duration.as_millis() as u64;
        self.estimated_tokens += tokens;
        // 简单估算：$0.001 per 1000 tokens (Deepseek 定价)
        self.estimated_cost += (tokens as f64) * 0.001 / 1000.0;
        self.last_updated = Utc::now();
    }

    /// 成功率
    pub fn success_rate(&self) -> f32 {
        if self.total_calls == 0 {
            return 0.0;
        }
        (self.success_calls as f32) / (self.total_calls as f32)
    }

    /// 平均响应时间（毫秒）
    pub fn avg_response_time_ms(&self) -> u64 {
        if self.total_calls == 0 {
            return 0;
        }
        self.total_response_time_ms / self.total_calls
    }
}

impl Default for LlmMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具调用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetrics {
    /// 按工具名称统计的调用次数
    pub usage_by_tool: HashMap<String, u64>,

    /// 按工具名称统计的成功次数
    pub success_by_tool: HashMap<String, u64>,

    /// 按工具名称统计的总执行时间（毫秒）
    pub time_by_tool: HashMap<String, u64>,

    /// 总调用次数
    pub total_calls: u64,

    /// 最近更新时间
    pub last_updated: DateTime<Utc>,
}

impl ToolMetrics {
    pub fn new() -> Self {
        Self {
            usage_by_tool: HashMap::new(),
            success_by_tool: HashMap::new(),
            time_by_tool: HashMap::new(),
            total_calls: 0,
            last_updated: Utc::now(),
        }
    }

    /// 记录一次工具调用
    pub fn record_call(&mut self, tool_name: &str, success: bool, duration: Duration) {
        self.total_calls += 1;

        // 更新使用次数
        *self.usage_by_tool.entry(tool_name.to_string()).or_insert(0) += 1;

        // 更新成功次数
        if success {
            *self.success_by_tool.entry(tool_name.to_string()).or_insert(0) += 1;
        }

        // 更新执行时间
        *self.time_by_tool.entry(tool_name.to_string()).or_insert(0) += duration.as_millis() as u64;

        self.last_updated = Utc::now();
    }

    /// 获取工具成功率
    pub fn success_rate(&self, tool_name: &str) -> f32 {
        let usage = self.usage_by_tool.get(tool_name).unwrap_or(&0);
        let success = self.success_by_tool.get(tool_name).unwrap_or(&0);

        if *usage == 0 {
            return 0.0;
        }

        (*success as f32) / (*usage as f32)
    }

    /// 获取工具平均执行时间（毫秒）
    pub fn avg_time_ms(&self, tool_name: &str) -> u64 {
        let usage = self.usage_by_tool.get(tool_name).unwrap_or(&0);
        let total_time = self.time_by_tool.get(tool_name).unwrap_or(&0);

        if *usage == 0 {
            return 0;
        }

        total_time / usage
    }

    /// 获取使用次数最多的工具（Top N）
    pub fn top_tools(&self, limit: usize) -> Vec<(String, u64)> {
        let mut tools: Vec<_> = self.usage_by_tool.iter()
            .map(|(name, count)| (name.clone(), *count))
            .collect();

        tools.sort_by(|a, b| b.1.cmp(&a.1));
        tools.truncate(limit);
        tools
    }
}

impl Default for ToolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 命令执行统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetrics {
    /// 总命令数
    pub total_commands: u64,

    /// 成功命令数
    pub success_commands: u64,

    /// 失败命令数
    pub failed_commands: u64,

    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,

    /// 会话开始时间
    pub session_start: DateTime<Utc>,

    /// 最近更新时间
    pub last_updated: DateTime<Utc>,
}

impl CommandMetrics {
    pub fn new() -> Self {
        Self {
            total_commands: 0,
            success_commands: 0,
            failed_commands: 0,
            total_execution_time_ms: 0,
            session_start: Utc::now(),
            last_updated: Utc::now(),
        }
    }

    /// 记录一次命令执行
    pub fn record_command(&mut self, success: bool, duration: Duration) {
        self.total_commands += 1;
        if success {
            self.success_commands += 1;
        } else {
            self.failed_commands += 1;
        }
        self.total_execution_time_ms += duration.as_millis() as u64;
        self.last_updated = Utc::now();
    }

    /// 成功率
    pub fn success_rate(&self) -> f32 {
        if self.total_commands == 0 {
            return 0.0;
        }
        (self.success_commands as f32) / (self.total_commands as f32)
    }

    /// 平均执行时间（毫秒）
    pub fn avg_execution_time_ms(&self) -> u64 {
        if self.total_commands == 0 {
            return 0;
        }
        self.total_execution_time_ms / self.total_commands
    }

    /// 会话运行时长
    pub fn session_duration(&self) -> Duration {
        let now = Utc::now();
        (now - self.session_start).to_std().unwrap_or(Duration::from_secs(0))
    }
}

impl Default for CommandMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// 响应时间样本（最近 100 个，毫秒）
    pub response_times: Vec<u64>,

    /// 最快响应时间
    pub min_response_ms: u64,

    /// 最慢响应时间
    pub max_response_ms: u64,

    /// 最慢的命令
    pub slowest_command: Option<String>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            response_times: Vec::new(),
            min_response_ms: u64::MAX,
            max_response_ms: 0,
            slowest_command: None,
        }
    }

    /// 记录一次响应时间
    pub fn record_response(&mut self, duration: Duration, command: &str) {
        let ms = duration.as_millis() as u64;

        // 更新最小值
        if ms < self.min_response_ms {
            self.min_response_ms = ms;
        }

        // 更新最大值
        if ms > self.max_response_ms {
            self.max_response_ms = ms;
            self.slowest_command = Some(command.to_string());
        }

        // 保存样本（最多 100 个）
        self.response_times.push(ms);
        if self.response_times.len() > 100 {
            self.response_times.remove(0);
        }
    }

    /// 计算百分位数
    pub fn percentile(&self, p: f32) -> u64 {
        if self.response_times.is_empty() {
            return 0;
        }

        let mut sorted = self.response_times.clone();
        sorted.sort_unstable();

        // 使用标准百分位数计算：(n-1) * p / 100
        let index = ((sorted.len() - 1) as f32 * p / 100.0) as usize;

        sorted[index]
    }

    /// P50（中位数）
    pub fn p50(&self) -> u64 {
        self.percentile(50.0)
    }

    /// P95
    pub fn p95(&self) -> u64 {
        self.percentile(95.0)
    }

    /// P99
    pub fn p99(&self) -> u64 {
        self.percentile(99.0)
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_metrics() {
        let mut metrics = LlmMetrics::new();

        metrics.record_call(true, Duration::from_millis(800), 100);
        metrics.record_call(true, Duration::from_millis(1200), 150);
        metrics.record_call(false, Duration::from_millis(500), 50);

        assert_eq!(metrics.total_calls, 3);
        assert_eq!(metrics.success_calls, 2);
        assert_eq!(metrics.failed_calls, 1);
        assert_eq!(metrics.success_rate(), 2.0 / 3.0);
        assert_eq!(metrics.avg_response_time_ms(), 833); // (800+1200+500)/3
        assert_eq!(metrics.estimated_tokens, 300);
    }

    #[test]
    fn test_tool_metrics() {
        let mut metrics = ToolMetrics::new();

        metrics.record_call("calculator", true, Duration::from_millis(100));
        metrics.record_call("calculator", true, Duration::from_millis(150));
        metrics.record_call("shell_execute", true, Duration::from_millis(500));
        metrics.record_call("calculator", false, Duration::from_millis(80));

        assert_eq!(metrics.total_calls, 4);
        assert_eq!(metrics.usage_by_tool.get("calculator"), Some(&3));
        assert_eq!(metrics.success_by_tool.get("calculator"), Some(&2));
        assert!((metrics.success_rate("calculator") - 2.0/3.0).abs() < 0.01);
        assert_eq!(metrics.avg_time_ms("calculator"), 110); // (100+150+80)/3

        let top = metrics.top_tools(2);
        assert_eq!(top[0].0, "calculator");
        assert_eq!(top[0].1, 3);
    }

    #[test]
    fn test_command_metrics() {
        let mut metrics = CommandMetrics::new();

        metrics.record_command(true, Duration::from_millis(1000));
        metrics.record_command(true, Duration::from_millis(2000));
        metrics.record_command(false, Duration::from_millis(500));

        assert_eq!(metrics.total_commands, 3);
        assert_eq!(metrics.success_commands, 2);
        assert_eq!(metrics.failed_commands, 1);
        assert!((metrics.success_rate() - 2.0/3.0).abs() < 0.01);
        assert_eq!(metrics.avg_execution_time_ms(), 1166); // (1000+2000+500)/3
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        for i in 1..=100 {
            metrics.record_response(Duration::from_millis(i * 10), "test");
        }

        assert_eq!(metrics.response_times.len(), 100);
        assert_eq!(metrics.min_response_ms, 10);
        assert_eq!(metrics.max_response_ms, 1000);
        assert_eq!(metrics.p50(), 500);
        assert_eq!(metrics.p95(), 950);
    }
}
