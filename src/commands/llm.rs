//! LLM 相关命令
//!
//! 提供 LLM 交互命令：
//! - /ask - 向 LLM 提问
//! - /llm - LLM 管理和诊断

use crate::command::{Command, CommandRegistry};
use crate::llm_manager::LlmManager;
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册 LLM 命令
///
/// 需要传入 LlmManager 的引用以便命令可以访问
pub fn register_llm_commands(
    registry: &mut CommandRegistry,
    llm_manager: Arc<RwLock<LlmManager>>,
) {
    // /ask 命令
    let ask_manager = Arc::clone(&llm_manager);
    let ask_cmd =
        Command::from_fn("ask", "向 LLM 提问 (使用 fallback LLM)", move |arg: &str| {
            cmd_ask(arg, Arc::clone(&ask_manager))
        })
        .with_group("llm");
    registry.register(ask_cmd);

    // /llm 命令
    let llm_cmd_manager = Arc::clone(&llm_manager);
    let llm_cmd = Command::from_fn(
        "llm",
        "LLM 管理: llm [diag <primary|fallback>]",
        move |arg: &str| cmd_llm(arg, Arc::clone(&llm_cmd_manager)),
    )
    .with_group("llm");
    registry.register(llm_cmd);
}

/// /ask 命令处理器
fn cmd_ask(arg: &str, manager: Arc<RwLock<LlmManager>>) -> String {
    let query = arg.trim();
    if query.is_empty() {
        return format!("{}\n{}", "用法: /ask <问题>".yellow(), "示例: /ask 你好".dimmed());
    }

    // 使用 block_in_place 在同步上下文中调用异步代码
    match tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let manager = manager.read().await;
            manager.chat(query).await
        })
    }) {
        Ok(response) => response,
        Err(e) => format!("{} {}", "错误:".red(), e),
    }
}

/// /llm 命令处理器
fn cmd_llm(arg: &str, manager: Arc<RwLock<LlmManager>>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        // 显示状态
        return cmd_llm_status(manager);
    }

    let subcmd = parts[0].to_lowercase();
    match subcmd.as_str() {
        "diag" | "diagnose" => {
            let target = if parts.len() > 1 {
                parts[1].to_lowercase()
            } else {
                "primary".to_string()
            };
            cmd_llm_diag(&target, manager)
        }
        _ => format!(
            "{} {}\n{}",
            "未知子命令:".red(),
            subcmd,
            "用法: /llm [diag <primary|fallback>]".dimmed()
        ),
    }
}

/// 显示 LLM 状态
fn cmd_llm_status(manager: Arc<RwLock<LlmManager>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let manager = manager.read().await;

            let mut lines = vec!["LLM 状态:".bold().to_string()];

            // Primary LLM
            if let Some(client) = manager.primary() {
                lines.push(format!("  {} {}", "Primary:".cyan(), client.model()));
            } else {
                lines.push(format!("  {} {}", "Primary:".dimmed(), "(未配置)".dimmed()));
            }

            // Fallback LLM
            if let Some(client) = manager.fallback() {
                lines.push(format!("  {} {}", "Fallback:".cyan(), client.model()));
            } else {
                lines.push(format!(
                    "  {} {}",
                    "Fallback:".dimmed(),
                    "(未配置)".dimmed()
                ));
            }

            lines.push("".to_string());
            lines.push(format!("{}", "提示: /llm diag <primary|fallback> 诊断连接".dimmed()));

            lines.join("\n")
        })
    })
}

/// 诊断 LLM 连接
fn cmd_llm_diag(target: &str, manager: Arc<RwLock<LlmManager>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let manager = manager.read().await;

            match target {
                "primary" => {
                    let diag = manager.diagnose_primary().await;
                    format!("{}\n{}", "Primary LLM 诊断:".bold(), diag)
                }
                "fallback" => {
                    let diag = manager.diagnose_fallback().await;
                    format!("{}\n{}", "Fallback LLM 诊断:".bold(), diag)
                }
                _ => format!(
                    "{} {}\n{}",
                    "未知目标:".red(),
                    target,
                    "使用: primary 或 fallback".dimmed()
                ),
            }
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_llm_commands() {
        let mut registry = CommandRegistry::new();
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        register_llm_commands(&mut registry, manager);

        assert!(registry.get("ask").is_some());
        assert!(registry.get("llm").is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_ask_empty_query() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_ask("", manager);
        assert!(result.contains("用法"));
        assert!(result.contains("/ask"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_ask_with_no_llm_configured() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_ask("test question", manager);
        // 应该返回错误，因为没有配置 LLM
        assert!(result.contains("错误") || result.contains("未配置"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_llm_status_no_llm() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_llm_status(manager);
        assert!(result.contains("LLM 状态"));
        assert!(result.contains("未配置"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_llm_diag_primary() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_llm_diag("primary", manager);
        assert!(result.contains("Primary LLM 诊断"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_llm_diag_fallback() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_llm_diag("fallback", manager);
        assert!(result.contains("Fallback LLM 诊断"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_llm_diag_unknown_target() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_llm_diag("unknown", manager);
        assert!(result.contains("未知目标"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_cmd_llm_unknown_subcommand() {
        let manager = Arc::new(RwLock::new(LlmManager::new()));
        let result = cmd_llm("unknown", manager);
        assert!(result.contains("未知子命令"));
    }
}
