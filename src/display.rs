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
