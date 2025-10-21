//! 语音播报命令
//!
//! 提供语音播报控制功能

use crate::command::{Command, CommandRegistry};
use crate::voice::VoiceBroadcaster;
use colored::Colorize;
use std::sync::Arc;

/// 注册语音播报命令
pub fn register_voice_commands(
    registry: &mut CommandRegistry,
    voice_broadcaster: Option<Arc<VoiceBroadcaster>>,
) {
    // /voice 命令
    let voice_cmd = Command::from_fn(
        "voice",
        "语音播报控制: voice [on|off|status|test]",
        move |arg: &str| handle_voice(arg, voice_broadcaster.clone()),
    )
    .with_aliases(vec!["v".to_string()])
    .with_group("system");
    registry.register(voice_cmd);
}

/// 处理 /voice 命令
fn handle_voice(arg: &str, voice_broadcaster: Option<Arc<VoiceBroadcaster>>) -> String {
    let Some(broadcaster) = voice_broadcaster else {
        return format!(
            "{} 语音播报未启用\n提示: 在配置文件中设置 voice.enabled = true",
            "提示:".yellow()
        );
    };

    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_voice_status(&broadcaster);
    }

    let subcommand = parts[0];
    let rest = parts.get(1..).unwrap_or(&[]).join(" ");

    match subcommand {
        "on" | "enable" => handle_voice_enable(&broadcaster),
        "off" | "disable" => handle_voice_disable(&broadcaster),
        "status" | "s" => handle_voice_status(&broadcaster),
        "test" | "t" => handle_voice_test(&broadcaster, &rest),
        "help" | "h" => voice_help(),
        _ => format!(
            "{} 未知子命令: {}\n使用 /voice help 查看帮助",
            "错误:".red(),
            subcommand
        ),
    }
}

/// 启用语音播报
fn handle_voice_enable(broadcaster: &Arc<VoiceBroadcaster>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            broadcaster.enable().await;
            format!("{} 语音播报已启用", "✓".green())
        })
    })
}

/// 禁用语音播报
fn handle_voice_disable(broadcaster: &Arc<VoiceBroadcaster>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            broadcaster.disable().await;
            format!("{} 语音播报已禁用", "✓".green())
        })
    })
}

/// 显示语音播报状态
fn handle_voice_status(broadcaster: &Arc<VoiceBroadcaster>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let enabled = broadcaster.is_enabled().await;
            let status = if enabled {
                "已启用".green()
            } else {
                "已禁用".dimmed()
            };

            format!(
                "{}\n  状态: {}",
                "语音播报".bold().cyan(),
                status
            )
        })
    })
}

/// 测试语音播报
fn handle_voice_test(broadcaster: &Arc<VoiceBroadcaster>, text: &str) -> String {
    let test_text = if text.is_empty() {
        "这是一个语音播报测试"
    } else {
        text
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            match broadcaster.speak(test_text).await {
                Ok(_) => {
                    format!("{} 正在播报: {}", "✓".green(), test_text.cyan())
                }
                Err(e) => {
                    format!("{} 播报失败: {}", "错误:".red(), e)
                }
            }
        })
    })
}

/// 语音命令帮助
fn voice_help() -> String {
    format!(
        r#"{title}

{subtitle}
  /voice              - 显示语音播报状态
  /voice on           - 启用语音播报
  /voice off          - 禁用语音播报
  /voice status       - 显示状态
  /voice test [文本]  - 测试语音播报（默认测试文本）

{examples}
  /voice on
  /voice test "你好，世界"
  /voice status

{shortcuts}
  on → enable, off → disable, status → s, test → t

{note}
  提示: macOS 支持使用中文语音（如 Ting-Ting）
       可在配置文件中设置: voice.voice = "Ting-Ting"
"#,
        title = "语音播报控制".bold().cyan(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        shortcuts = "快捷命令:".dimmed(),
        note = "说明:".bold()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice::BroadcastConfig;

    fn create_test_broadcaster() -> Arc<VoiceBroadcaster> {
        let config = BroadcastConfig {
            enabled: false,
            voice: None,
            max_queue_size: 10,
        };
        Arc::new(VoiceBroadcaster::new(config))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_voice_status() {
        let broadcaster = create_test_broadcaster();
        let result = handle_voice_status(&broadcaster);
        assert!(result.contains("语音播报"));
        assert!(result.contains("已禁用"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_voice_enable() {
        let broadcaster = create_test_broadcaster();
        let result = handle_voice_enable(&broadcaster);
        assert!(result.contains("已启用"));

        let enabled = broadcaster.is_enabled().await;
        assert!(enabled);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_voice_disable() {
        let broadcaster = create_test_broadcaster();
        broadcaster.enable().await;

        let result = handle_voice_disable(&broadcaster);
        assert!(result.contains("已禁用"));

        let enabled = broadcaster.is_enabled().await;
        assert!(!enabled);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_voice_test_default_text() {
        let broadcaster = create_test_broadcaster();
        broadcaster.enable().await;

        let result = handle_voice_test(&broadcaster, "");
        assert!(result.contains("正在播报") || result.contains("这是一个语音播报测试"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_voice_test_custom_text() {
        let broadcaster = create_test_broadcaster();
        broadcaster.enable().await;

        let result = handle_voice_test(&broadcaster, "测试文本");
        assert!(result.contains("测试文本"));
    }

    #[test]
    fn test_voice_help() {
        let result = voice_help();
        assert!(result.contains("语音播报控制"));
        assert!(result.contains("/voice on"));
        assert!(result.contains("/voice test"));
    }

    #[test]
    fn test_handle_voice_no_broadcaster() {
        let result = handle_voice("on", None);
        assert!(result.contains("未启用"));
    }
}
