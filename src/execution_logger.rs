//! 执行日志系统
//!
//! 记录命令执行的时间、结果、耗时等信息，用于：
//! - 审计追踪
//! - 性能分析
//! - 错误调试
//! - 统计报告

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::Duration;

/// 执行日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 命令内容
    pub command: String,
    /// 命令类型（command, shell, text）
    pub command_type: CommandType,
    /// 是否成功
    pub success: bool,
    /// 执行耗时（毫秒）
    pub duration_ms: u64,
    /// 结果预览（前 100 字符）
    pub result_preview: String,
}

/// 命令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommandType {
    /// 命令（/开头）
    Command,
    /// Shell（!开头）
    Shell,
    /// 文本对话
    Text,
}

impl std::fmt::Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::Command => write!(f, "CMD"),
            CommandType::Shell => write!(f, "SHELL"),
            CommandType::Text => write!(f, "TEXT"),
        }
    }
}

impl ExecutionLog {
    /// 创建新的执行日志
    pub fn new(
        command: String,
        command_type: CommandType,
        success: bool,
        duration: Duration,
        result: &str,
    ) -> Self {
        let duration_ms = duration.as_millis() as u64;

        // 截取结果的前 100 字符作为预览（考虑 UTF-8 边界）
        let result_preview = if result.len() > 100 {
            // 找到安全的截断位置（UTF-8 字符边界）
            let mut cutoff = 100.min(result.len());
            while cutoff > 0 && !result.is_char_boundary(cutoff) {
                cutoff -= 1;
            }
            format!("{}...", &result[..cutoff])
        } else {
            result.to_string()
        };

        Self {
            timestamp: Utc::now(),
            command,
            command_type,
            success,
            duration_ms,
            result_preview,
        }
    }

    /// 格式化输出
    pub fn format(&self) -> String {
        let status = if self.success { "✓" } else { "✗" };
        format!(
            "[{}] {} {:7} | {:>5}ms | {}",
            self.timestamp.format("%H:%M:%S"),
            status,
            self.command_type,
            self.duration_ms,
            self.command
        )
    }

    /// 详细输出
    pub fn format_detailed(&self) -> String {
        let status = if self.success { "✓ 成功" } else { "✗ 失败" };
        format!(
            "[{}] {} - {}\n  类型: {} | 耗时: {}ms\n  命令: {}\n  结果: {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            status,
            self.command_type,
            self.command_type,
            self.duration_ms,
            self.command,
            self.result_preview
        )
    }
}

/// 执行统计
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// 总执行次数
    pub total: usize,
    /// 成功次数
    pub success: usize,
    /// 失败次数
    pub failed: usize,
    /// 平均耗时（毫秒）
    pub avg_duration_ms: f64,
    /// 最大耗时（毫秒）
    pub max_duration_ms: u64,
    /// 最小耗时（毫秒）
    pub min_duration_ms: u64,
}

impl ExecutionStats {
    /// 计算成功率（百分比）
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.success as f64 / self.total as f64) * 100.0
        }
    }

    /// 格式化输出
    pub fn format(&self) -> String {
        if self.total == 0 {
            return "暂无执行记录".to_string();
        }

        format!(
            "执行统计:\n\
             ├─ 总执行: {} 次\n\
             ├─ 成功: {} 次 ({:.1}%)\n\
             ├─ 失败: {} 次 ({:.1}%)\n\
             ├─ 平均耗时: {:.1}ms\n\
             ├─ 最快: {}ms\n\
             └─ 最慢: {}ms",
            self.total,
            self.success,
            self.success_rate(),
            self.failed,
            100.0 - self.success_rate(),
            self.avg_duration_ms,
            self.min_duration_ms,
            self.max_duration_ms
        )
    }
}

/// 执行日志系统
pub struct ExecutionLogger {
    /// 日志队列（Ring Buffer）
    logs: VecDeque<ExecutionLog>,
    /// 最大日志数量
    max_logs: usize,
}

impl ExecutionLogger {
    /// 创建新的执行日志系统
    ///
    /// # 参数
    /// - `max_logs`: 最大保存的日志数量
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: VecDeque::with_capacity(max_logs),
            max_logs,
        }
    }

    /// 记录执行日志
    ///
    /// # 参数
    /// - `command`: 执行的命令
    /// - `command_type`: 命令类型
    /// - `success`: 是否成功
    /// - `duration`: 执行耗时
    /// - `result`: 执行结果
    pub fn log(
        &mut self,
        command: String,
        command_type: CommandType,
        success: bool,
        duration: Duration,
        result: &str,
    ) {
        let log = ExecutionLog::new(command, command_type, success, duration, result);

        // Ring Buffer: 超过容量则移除最旧的
        if self.logs.len() >= self.max_logs {
            self.logs.pop_front();
        }

        self.logs.push_back(log);
    }

    /// 获取最近的 N 条日志
    pub fn recent(&self, n: usize) -> Vec<&ExecutionLog> {
        let count = n.min(self.logs.len());
        self.logs.iter().rev().take(count).collect()
    }

    /// 搜索包含关键词的日志
    pub fn search(&self, keyword: &str) -> Vec<&ExecutionLog> {
        let keyword_lower = keyword.to_lowercase();
        self.logs
            .iter()
            .filter(|log| {
                log.command.to_lowercase().contains(&keyword_lower)
                    || log.result_preview.to_lowercase().contains(&keyword_lower)
            })
            .collect()
    }

    /// 按类型过滤日志
    pub fn filter_by_type(&self, command_type: CommandType) -> Vec<&ExecutionLog> {
        self.logs
            .iter()
            .filter(|log| log.command_type == command_type)
            .collect()
    }

    /// 只获取成功的日志
    pub fn successful(&self) -> Vec<&ExecutionLog> {
        self.logs.iter().filter(|log| log.success).collect()
    }

    /// 只获取失败的日志
    pub fn failed(&self) -> Vec<&ExecutionLog> {
        self.logs.iter().filter(|log| !log.success).collect()
    }

    /// 获取所有日志
    pub fn all(&self) -> Vec<&ExecutionLog> {
        self.logs.iter().collect()
    }

    /// 计算统计信息
    pub fn stats(&self) -> ExecutionStats {
        if self.logs.is_empty() {
            return ExecutionStats::default();
        }

        let total = self.logs.len();
        let success = self.logs.iter().filter(|log| log.success).count();
        let failed = total - success;

        let total_duration: u64 = self.logs.iter().map(|log| log.duration_ms).sum();
        let avg_duration_ms = total_duration as f64 / total as f64;

        let max_duration_ms = self
            .logs
            .iter()
            .map(|log| log.duration_ms)
            .max()
            .unwrap_or(0);

        let min_duration_ms = self
            .logs
            .iter()
            .map(|log| log.duration_ms)
            .min()
            .unwrap_or(0);

        ExecutionStats {
            total,
            success,
            failed,
            avg_duration_ms,
            max_duration_ms,
            min_duration_ms,
        }
    }

    /// 按类型统计
    pub fn stats_by_type(&self, command_type: CommandType) -> ExecutionStats {
        let filtered: Vec<_> = self.filter_by_type(command_type);

        if filtered.is_empty() {
            return ExecutionStats::default();
        }

        let total = filtered.len();
        let success = filtered.iter().filter(|log| log.success).count();
        let failed = total - success;

        let total_duration: u64 = filtered.iter().map(|log| log.duration_ms).sum();
        let avg_duration_ms = total_duration as f64 / total as f64;

        let max_duration_ms = filtered
            .iter()
            .map(|log| log.duration_ms)
            .max()
            .unwrap_or(0);

        let min_duration_ms = filtered
            .iter()
            .map(|log| log.duration_ms)
            .min()
            .unwrap_or(0);

        ExecutionStats {
            total,
            success,
            failed,
            avg_duration_ms,
            max_duration_ms,
            min_duration_ms,
        }
    }

    /// 清空所有日志
    pub fn clear(&mut self) {
        self.logs.clear();
    }

    /// 获取日志数量
    pub fn len(&self) -> usize {
        self.logs.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }
}

impl Default for ExecutionLogger {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_logger_creation() {
        let logger = ExecutionLogger::new(100);
        assert_eq!(logger.len(), 0);
        assert!(logger.is_empty());
        assert_eq!(logger.max_logs, 100);
    }

    #[test]
    fn test_log_entry() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(50),
            "Help message",
        );

        assert_eq!(logger.len(), 1);

        let logs = logger.recent(1);
        assert_eq!(logs[0].command, "/help");
        assert_eq!(logs[0].command_type, CommandType::Command);
        assert!(logs[0].success);
        assert_eq!(logs[0].duration_ms, 50);
    }

    #[test]
    fn test_ring_buffer() {
        let mut logger = ExecutionLogger::new(5);

        // 添加 10 条日志
        for i in 0..10 {
            logger.log(
                format!("command-{}", i),
                CommandType::Command,
                true,
                Duration::from_millis(10),
                "result",
            );
        }

        // 应该只保留最新的 5 条
        assert_eq!(logger.len(), 5);

        let logs = logger.all();
        assert!(logs[0].command.contains("command-5"));
        assert!(logs[4].command.contains("command-9"));
    }

    #[test]
    fn test_search() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "Help",
        );
        logger.log(
            "!ls".to_string(),
            CommandType::Shell,
            true,
            Duration::from_millis(20),
            "files",
        );
        logger.log(
            "/memory".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(15),
            "Memory",
        );

        let results = logger.search("help");
        assert_eq!(results.len(), 1);

        let results = logger.search("memory");
        assert_eq!(results.len(), 1);

        let results = logger.search("notfound");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_filter_by_type() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "Help",
        );
        logger.log(
            "!ls".to_string(),
            CommandType::Shell,
            true,
            Duration::from_millis(20),
            "files",
        );
        logger.log(
            "你好".to_string(),
            CommandType::Text,
            true,
            Duration::from_millis(500),
            "你好！",
        );

        let commands = logger.filter_by_type(CommandType::Command);
        assert_eq!(commands.len(), 1);

        let shells = logger.filter_by_type(CommandType::Shell);
        assert_eq!(shells.len(), 1);

        let texts = logger.filter_by_type(CommandType::Text);
        assert_eq!(texts.len(), 1);
    }

    #[test]
    fn test_success_and_failed() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "cmd1".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "ok",
        );
        logger.log(
            "cmd2".to_string(),
            CommandType::Command,
            false,
            Duration::from_millis(15),
            "error",
        );
        logger.log(
            "cmd3".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(12),
            "ok",
        );

        let successful = logger.successful();
        assert_eq!(successful.len(), 2);

        let failed = logger.failed();
        assert_eq!(failed.len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "cmd1".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "ok",
        );
        logger.log(
            "cmd2".to_string(),
            CommandType::Command,
            false,
            Duration::from_millis(20),
            "error",
        );
        logger.log(
            "cmd3".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(30),
            "ok",
        );

        let stats = logger.stats();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.success, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.avg_duration_ms, 20.0);
        assert_eq!(stats.max_duration_ms, 30);
        assert_eq!(stats.min_duration_ms, 10);
        // 使用近似比较（浮点数精度问题）
        assert!((stats.success_rate() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_stats_by_type() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "ok",
        );
        logger.log(
            "!ls".to_string(),
            CommandType::Shell,
            true,
            Duration::from_millis(50),
            "files",
        );
        logger.log(
            "/memory".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(20),
            "ok",
        );

        let cmd_stats = logger.stats_by_type(CommandType::Command);
        assert_eq!(cmd_stats.total, 2);
        assert_eq!(cmd_stats.avg_duration_ms, 15.0);

        let shell_stats = logger.stats_by_type(CommandType::Shell);
        assert_eq!(shell_stats.total, 1);
        assert_eq!(shell_stats.avg_duration_ms, 50.0);
    }

    #[test]
    fn test_clear() {
        let mut logger = ExecutionLogger::new(100);

        logger.log(
            "cmd".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            "ok",
        );

        assert_eq!(logger.len(), 1);

        logger.clear();
        assert_eq!(logger.len(), 0);
        assert!(logger.is_empty());
    }

    #[test]
    fn test_execution_log_format() {
        let log = ExecutionLog::new(
            "/help".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(50),
            "Help message",
        );

        let formatted = log.format();
        assert!(formatted.contains("✓"));
        assert!(formatted.contains("CMD"));
        assert!(formatted.contains("50ms"));
        assert!(formatted.contains("/help"));
    }

    #[test]
    fn test_result_preview_truncation() {
        let long_result = "a".repeat(200);
        let log = ExecutionLog::new(
            "cmd".to_string(),
            CommandType::Command,
            true,
            Duration::from_millis(10),
            &long_result,
        );

        assert_eq!(log.result_preview.len(), 103); // 100 + "..."
        assert!(log.result_preview.ends_with("..."));
    }
}
