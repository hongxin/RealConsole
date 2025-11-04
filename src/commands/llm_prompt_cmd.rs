//! /set-prompt 和 /show-prompt 命令实现
//!
//! 用法：
//! - `/set-prompt <prompt>` - 设置运行时系统提示词
//! - `/set-prompt reset` - 重置为配置文件默认值
//! - `/show-prompt` - 显示当前系统提示词

use crate::command::{Command, CommandRegistry};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册 LLM 提示词命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `runtime_system_prompt`: 运行时系统提示词
/// - `config_system_prompt`: 配置文件中的系统提示词
pub fn register_llm_prompt_commands(
    registry: &mut CommandRegistry,
    runtime_system_prompt: Arc<RwLock<Option<String>>>,
    config_system_prompt: Option<String>,
) {
    // 注册 /set-prompt 命令
    {
        let prompt_arc = Arc::clone(&runtime_system_prompt);
        let config_prompt = config_system_prompt.clone();
        let cmd = Command::from_fn("set-prompt", "设置系统提示词", move |args| {
            handle_set_prompt(Arc::clone(&prompt_arc), config_prompt.clone(), args)
        })
        .with_group("llm");

        registry.register(cmd);
    }

    // 注册 /show-prompt 命令
    {
        let prompt_arc = Arc::clone(&runtime_system_prompt);
        let config_prompt = config_system_prompt;
        let cmd = Command::from_fn("show-prompt", "显示当前系统提示词", move |_args| {
            handle_show_prompt(Arc::clone(&prompt_arc), config_prompt.clone())
        })
        .with_group("llm");

        registry.register(cmd);
    }
}

/// 处理 /set-prompt 命令
fn handle_set_prompt(
    runtime_system_prompt: Arc<RwLock<Option<String>>>,
    config_system_prompt: Option<String>,
    args: &str,
) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let args = args.trim();

            if args.is_empty() {
                return format!(
                    "{}\n\n{}\n  {} <prompt>\n  {} reset\n\n{}\n  {}",
                    "用法说明：".bold(),
                    "设置系统提示词：".cyan(),
                    "/set-prompt".yellow(),
                    "/set-prompt".yellow(),
                    "查看当前提示词：".cyan(),
                    "/show-prompt".yellow()
                );
            }

            // 处理 reset 命令
            if args.eq_ignore_ascii_case("reset") {
                let mut prompt = runtime_system_prompt.write().await;
                *prompt = None;

                return if config_system_prompt.is_some() {
                    format!(
                        "{} 已重置为配置文件中的默认值\n使用 {} 查看当前提示词",
                        "✓".green(),
                        "/show-prompt".yellow()
                    )
                } else {
                    format!(
                        "{} 已重置为内置默认值\n使用 {} 查看当前提示词",
                        "✓".green(),
                        "/show-prompt".yellow()
                    )
                };
            }

            // 设置新的系统提示词
            let mut prompt = runtime_system_prompt.write().await;
            *prompt = Some(args.to_string());

            format!(
                "{} 系统提示词已更新\n\n{}\n{}\n\n{}\n  {}",
                "✓".green(),
                "新提示词：".cyan().bold(),
                args.dimmed(),
                "提示：".cyan(),
                "使用 /show-prompt 查看当前提示词，使用 /set-prompt reset 重置".dimmed()
            )
        })
    })
}

/// 处理 /show-prompt 命令
fn handle_show_prompt(
    runtime_system_prompt: Arc<RwLock<Option<String>>>,
    config_system_prompt: Option<String>,
) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let prompt = runtime_system_prompt.read().await;

            match prompt.as_ref() {
                Some(runtime_prompt) => {
                    // 有运行时提示词
                    format!(
                        "{}\n\n{}\n{}\n\n{}\n  {}",
                        "当前系统提示词：".cyan().bold(),
                        "来源：".cyan(),
                        "运行时设置（/set-prompt）".yellow(),
                        "内容：".cyan(),
                        runtime_prompt.dimmed()
                    )
                }
                None => {
                    // 使用默认提示词
                    if let Some(ref config_prompt) = config_system_prompt {
                        format!(
                            "{}\n\n{}\n{}\n\n{}\n{}\n\n{}\n  {}",
                            "当前系统提示词：".cyan().bold(),
                            "来源：".cyan(),
                            "配置文件（realconsole.yaml）".yellow(),
                            "内容：".cyan(),
                            config_prompt.dimmed(),
                            "提示：".cyan(),
                            "使用 /set-prompt <prompt> 可临时覆盖".dimmed()
                        )
                    } else {
                        let default_prompt = "你是一个有用的智能助手。你可以使用提供的工具来帮助用户完成任务。\n\
                            请直接、自然地回答用户的问题，不要过度客套。\n\
                            当用户询问事实性问题时，请提供准确、详细的信息。";

                        format!(
                            "{}\n\n{}\n{}\n\n{}\n{}\n\n{}\n  {}",
                            "当前系统提示词：".cyan().bold(),
                            "来源：".cyan(),
                            "内置默认值".yellow(),
                            "内容：".cyan(),
                            default_prompt.dimmed(),
                            "提示：".cyan(),
                            "使用 /set-prompt <prompt> 可自定义提示词".dimmed()
                        )
                    }
                }
            }
        })
    })
}
