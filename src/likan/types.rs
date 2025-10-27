//! 炼化炉核心类型
//!
//! 极简设计，只保留本质

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 通知模式
///
/// 控制炼化炉如何向用户报告状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationMode {
    /// 最小模式：仅在循环完成时输出一行简洁通知
    Minimal,
    /// 提示符模式：在命令行提示符中显示状态
    Prompt,
    /// 静默模式：完全不主动通知，仅通过命令查询
    None,
}

impl Default for NotificationMode {
    fn default() -> Self {
        Self::Minimal
    }
}

impl fmt::Display for NotificationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Prompt => write!(f, "prompt"),
            Self::None => write!(f, "none"),
        }
    }
}

impl NotificationMode {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" => Some(Self::Minimal),
            "prompt" => Some(Self::Prompt),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FurnaceConfig {
    /// 是否启用炼化炉
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// 循环间隔（秒）
    #[serde(default = "default_cycle_interval")]
    pub cycle_interval_secs: u64,

    /// 通知模式
    #[serde(default)]
    pub notification_mode: NotificationMode,

    /// 是否在提示符中显示状态
    #[serde(default)]
    pub show_in_prompt: bool,

    /// 最小置信度阈值
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,

    /// 最小频率阈值（命令至少出现几次）
    #[serde(default = "default_min_frequency")]
    pub min_frequency: usize,

    /// 最大模式数量（防止过载）
    #[serde(default = "default_max_patterns")]
    pub max_patterns: usize,
}

// Serde 默认值函数
fn default_enabled() -> bool {
    true
}

fn default_cycle_interval() -> u64 {
    300 // 5分钟
}

fn default_min_confidence() -> f64 {
    0.6
}

fn default_min_frequency() -> usize {
    3
}

fn default_max_patterns() -> usize {
    50
}

impl Default for FurnaceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cycle_interval_secs: 300, // 5分钟（测试用，正式环境建议3600）
            notification_mode: NotificationMode::Minimal,
            show_in_prompt: false,
            min_confidence: 0.6, // 60%置信度
            min_frequency: 3,    // 至少3次
            max_patterns: 50,    // 最多50个模式
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
        assert_eq!(config.cycle_interval_secs, 300); // 5分钟（已从1小时优化）
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.min_frequency, 3);
    }
}
