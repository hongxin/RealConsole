//! 显示辅助工具
//!
//! 提供便捷的符号和格式化函数，根据配置自动选择 emoji 或纯文本

use crate::config::DisplayConfig;
use colored::Colorize;

/// 显示符号辅助器
pub struct DisplayHelper {
    use_emoji: bool,
}

impl Default for DisplayHelper {
    fn default() -> Self {
        Self { use_emoji: false }
    }
}

impl DisplayHelper {
    /// 从配置创建
    pub fn from_config(config: &DisplayConfig) -> Self {
        Self {
            use_emoji: config.use_emoji,
        }
    }

    /// 创建一个启用 emoji 的 helper
    pub fn with_emoji() -> Self {
        Self { use_emoji: true }
    }

    // === 状态符号 ===

    /// 成功标记
    pub fn ok(&self) -> &str {
        if self.use_emoji {
            "✓"
        } else {
            "[OK]"
        }
    }

    /// 错误标记
    pub fn error(&self) -> &str {
        if self.use_emoji {
            "❌"
        } else {
            "[ERROR]"
        }
    }

    /// 警告标记
    pub fn warning(&self) -> &str {
        if self.use_emoji {
            "⚠️"
        } else {
            "[!]"
        }
    }

    /// 信息标记
    pub fn info(&self) -> &str {
        if self.use_emoji {
            "ℹ️"
        } else {
            "[i]"
        }
    }

    /// 提示标记
    pub fn tip(&self) -> &str {
        if self.use_emoji {
            "💡"
        } else {
            "[TIP]"
        }
    }

    // === 功能符号 ===

    /// 火箭（启动/部署）
    pub fn rocket(&self) -> &str {
        if self.use_emoji {
            "🚀"
        } else {
            "[>>]"
        }
    }

    /// 闪电（快速）
    pub fn lightning(&self) -> &str {
        if self.use_emoji {
            "⚡"
        } else {
            "[>]"
        }
    }

    /// 工具（配置/修复）
    pub fn tool(&self) -> &str {
        if self.use_emoji {
            "🔧"
        } else {
            "[TOOL]"
        }
    }

    /// 图表（统计）
    pub fn chart(&self) -> &str {
        if self.use_emoji {
            "📊"
        } else {
            "[STATS]"
        }
    }

    /// 日志
    pub fn log(&self) -> &str {
        if self.use_emoji {
            "📝"
        } else {
            "[LOG]"
        }
    }

    // === 用户符号 ===

    /// 用户
    pub fn user(&self) -> &str {
        if self.use_emoji {
            "👤"
        } else {
            "[User]"
        }
    }

    /// AI/机器人
    pub fn ai(&self) -> &str {
        if self.use_emoji {
            "🤖"
        } else {
            "[AI]"
        }
    }

    // === 状态符号 ===

    /// 开启状态
    pub fn on(&self) -> &str {
        if self.use_emoji {
            "🟢"
        } else {
            "[ON]"
        }
    }

    /// 关闭状态
    pub fn off(&self) -> &str {
        if self.use_emoji {
            "🔴"
        } else {
            "[OFF]"
        }
    }

    // === 重要性符号 ===

    /// 重要（一星）
    pub fn important(&self) -> &str {
        if self.use_emoji {
            "⭐"
        } else {
            "[*]"
        }
    }

    /// 非常重要（两星）
    pub fn critical(&self) -> &str {
        if self.use_emoji {
            "⭐⭐"
        } else {
            "[**]"
        }
    }

    // === 格式化辅助函数 ===

    /// 格式化成功消息
    pub fn success(&self, msg: &str) -> String {
        format!("{} {}", self.ok().green(), msg)
    }

    /// 格式化错误消息
    pub fn error_msg(&self, msg: &str) -> String {
        format!("{} {}", self.error().red(), msg)
    }

    /// 格式化警告消息
    pub fn warning_msg(&self, msg: &str) -> String {
        format!("{} {}", self.warning().yellow(), msg)
    }

    /// 格式化信息消息
    pub fn info_msg(&self, msg: &str) -> String {
        format!("{} {}", self.info().cyan(), msg)
    }

    /// 格式化提示消息
    pub fn tip_msg(&self, msg: &str) -> String {
        format!("{} {}", self.tip().cyan(), msg)
    }
}

// 提供全局便捷函数（使用默认配置 - 不使用 emoji）
use once_cell::sync::Lazy;

static DEFAULT_HELPER: Lazy<DisplayHelper> = Lazy::new(DisplayHelper::default);

/// 获取默认的 DisplayHelper
pub fn default_helper() -> &'static DisplayHelper {
    &DEFAULT_HELPER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_without_emoji() {
        let helper = DisplayHelper::default();
        assert_eq!(helper.ok(), "[OK]");
        assert_eq!(helper.error(), "[ERROR]");
        assert_eq!(helper.warning(), "[!]");
        assert_eq!(helper.info(), "[i]");
        assert_eq!(helper.user(), "[User]");
        assert_eq!(helper.ai(), "[AI]");
    }

    #[test]
    fn test_with_emoji() {
        let helper = DisplayHelper::with_emoji();
        assert_eq!(helper.ok(), "✓");
        assert_eq!(helper.error(), "❌");
        assert_eq!(helper.warning(), "⚠️");
        assert_eq!(helper.info(), "ℹ️");
        assert_eq!(helper.user(), "👤");
        assert_eq!(helper.ai(), "🤖");
    }

    #[test]
    fn test_formatted_messages() {
        let helper = DisplayHelper::default();
        let msg = helper.success("操作成功");
        assert!(msg.contains("[OK]"));
        assert!(msg.contains("操作成功"));
    }
}
