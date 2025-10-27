//! 炼化炉核心类型
//!
//! 极简设计，只保留本质

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 模式（Pattern）- 从坎中提取的规律
///
/// 一分为三：频率、序列、关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    /// 频率模式：命令使用频率
    ///
    /// 例如："cargo check" 被使用了 15 次
    Frequency {
        command: String,
        count: usize,
        confidence: f64, // 0.0-1.0
    },

    /// 序列模式：命令执行序列
    ///
    /// 例如："cargo build" 后常跟 "cargo run"
    Sequence {
        commands: Vec<String>,
        occurrences: usize,
        confidence: f64,
    },

    /// 错误修复模式：错误后的有效命令
    ///
    /// 例如：编译错误后执行 "cargo check"
    ErrorFix {
        error_pattern: String,
        fix_command: String,
        success_rate: f64,
    },
}

impl Pattern {
    /// 获取模式的置信度
    pub fn confidence(&self) -> f64 {
        match self {
            Pattern::Frequency { confidence, .. } => *confidence,
            Pattern::Sequence { confidence, .. } => *confidence,
            Pattern::ErrorFix { success_rate, .. } => *success_rate,
        }
    }

    /// 获取模式的核心命令
    pub fn command(&self) -> Option<&str> {
        match self {
            Pattern::Frequency { command, .. } => Some(command),
            Pattern::Sequence { commands, .. } => commands.first().map(|s| s.as_str()),
            Pattern::ErrorFix { fix_command, .. } => Some(fix_command),
        }
    }

    /// 判断是否为高置信度模式
    pub fn is_high_confidence(&self) -> bool {
        self.confidence() >= 0.7
    }
}

/// 炼化报告（Cycle Report）
///
/// 记录一次炼化循环的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleReport {
    /// 发现的模式数量
    pub patterns_found: usize,

    /// 高置信度模式数量
    pub high_confidence_patterns: usize,

    /// 循环开始时间
    pub started_at: DateTime<Utc>,

    /// 循环结束时间
    pub completed_at: DateTime<Utc>,

    /// 循环耗时（毫秒）
    pub duration_ms: u64,
}

impl CycleReport {
    /// 创建新的报告
    pub fn new(patterns: &[Pattern], started_at: DateTime<Utc>) -> Self {
        let completed_at = Utc::now();
        let duration_ms = (completed_at - started_at).num_milliseconds().max(0) as u64;

        Self {
            patterns_found: patterns.len(),
            high_confidence_patterns: patterns
                .iter()
                .filter(|p| p.is_high_confidence())
                .count(),
            started_at,
            completed_at,
            duration_ms,
        }
    }
}

/// 炼化配置
///
/// 极简配置，只保留关键参数
#[derive(Debug, Clone)]
pub struct FurnaceConfig {
    /// 循环间隔（秒）
    pub cycle_interval_secs: u64,

    /// 最小置信度阈值
    pub min_confidence: f64,

    /// 最小频率阈值（命令至少出现几次）
    pub min_frequency: usize,

    /// 最大模式数量（防止过载）
    pub max_patterns: usize,
}

impl Default for FurnaceConfig {
    fn default() -> Self {
        Self {
            cycle_interval_secs: 300, // 5分钟（测试用，正式环境建议3600）
            min_confidence: 0.6,       // 60%置信度
            min_frequency: 3,          // 至少3次
            max_patterns: 50,          // 最多50个模式
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_confidence() {
        let pattern = Pattern::Frequency {
            command: "cargo build".to_string(),
            count: 10,
            confidence: 0.85,
        };
        assert_eq!(pattern.confidence(), 0.85);
        assert!(pattern.is_high_confidence());
    }

    #[test]
    fn test_pattern_command() {
        let pattern = Pattern::Frequency {
            command: "cargo build".to_string(),
            count: 10,
            confidence: 0.85,
        };
        assert_eq!(pattern.command(), Some("cargo build"));
    }

    #[test]
    fn test_cycle_report() {
        let patterns = vec![
            Pattern::Frequency {
                command: "test".to_string(),
                count: 5,
                confidence: 0.8,
            },
            Pattern::Frequency {
                command: "test2".to_string(),
                count: 3,
                confidence: 0.5,
            },
        ];

        let started_at = Utc::now();
        let report = CycleReport::new(&patterns, started_at);

        assert_eq!(report.patterns_found, 2);
        assert_eq!(report.high_confidence_patterns, 1);
    }

    #[test]
    fn test_furnace_config_default() {
        let config = FurnaceConfig::default();
        assert_eq!(config.cycle_interval_secs, 3600);
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.min_frequency, 3);
    }
}
