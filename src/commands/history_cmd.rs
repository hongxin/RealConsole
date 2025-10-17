//! /history 命令实现
//!
//! 用法：
//! - `/history` - 显示最近的历史记录
//! - `/history search <keyword>` - 搜索历史记录
//! - `/history clear` - 清空历史记录
//! - `/history stats` - 显示统计信息

use crate::command::{Command, CommandRegistry};
use crate::history::{HistoryManager, SortStrategy};
use chrono::Local;
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册历史记录命令
///
/// # 参数
/// - `registry`: 命令注册器
/// - `history`: 共享的历史记录管理器
pub fn register_history_commands(
    registry: &mut CommandRegistry,
    history: Arc<RwLock<HistoryManager>>,
) {
    let cmd = Command::from_fn("history", "命令历史管理", move |args| {
        handle_history(args, Arc::clone(&history))
    })
    .with_group("history");

    registry.register(cmd);
}

/// 处理 /history 命令
fn handle_history(args: &str, history: Arc<RwLock<HistoryManager>>) -> String {
    let args_str = args.trim();

    // 使用 tokio runtime 处理异步锁
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            if args_str.is_empty() {
                // 显示最近 20 条历史
                show_recent_history(&history, 20).await
            } else if args_str == "clear" {
                // 清空历史
                clear_history(&history).await
            } else if args_str == "stats" {
                // 显示统计信息
                show_stats(&history).await
            } else if let Some(keyword) = args_str.strip_prefix("search ") {
                // 搜索历史
                search_history(&history, keyword.trim()).await
            } else {
                format!(
                    "{}\n  /history          - 显示最近 20 条历史\n  /history search <keyword> - 搜索历史\n  /history clear    - 清空历史\n  /history stats    - 显示统计信息",
                    "用法:".yellow()
                )
            }
        })
    })
}

/// 显示最近的历史记录
async fn show_recent_history(history: &Arc<RwLock<HistoryManager>>, limit: usize) -> String {
    let history = history.read().await;

    let entries = history.recent(limit, SortStrategy::Smart);

    if entries.is_empty() {
        return "暂无历史记录".dimmed().to_string();
    }

    let mut output = Vec::new();
    output.push(format!("{} {}", "最近的历史记录".bold().cyan(), format!("(智能排序)").dimmed()));
    output.push("".to_string());

    for (index, entry) in entries.iter().enumerate() {
        let time_str = entry
            .last_timestamp
            .with_timezone(&Local)
            .format("%m-%d %H:%M")
            .to_string();

        // 格式：序号 | 命令 | 次数 | 时间
        let line = format!(
            "{:>3}. {} {}{}",
            (index + 1).to_string().dimmed(),
            entry.command.cyan(),
            if entry.count > 1 {
                format!("({}x) ", entry.count).yellow().to_string()
            } else {
                "".to_string()
            },
            time_str.dimmed()
        );
        output.push(line);
    }

    output.join("\n")
}

/// 搜索历史记录
async fn search_history(history: &Arc<RwLock<HistoryManager>>, keyword: &str) -> String {
    let history = history.read().await;

    let results = history.search(keyword, SortStrategy::Smart);

    if results.is_empty() {
        return format!("{} '{}'", "未找到匹配的历史记录:".yellow(), keyword);
    }

    let mut output = Vec::new();
    output.push(format!(
        "{} '{}' {} {}",
        "搜索结果:".bold().cyan(),
        keyword.green(),
        format!("(找到 {} 条)", results.len()).dimmed(),
        "(智能排序)".dimmed()
    ));
    output.push("".to_string());

    for (index, entry) in results.iter().enumerate() {
        let time_str = entry
            .last_timestamp
            .with_timezone(&Local)
            .format("%m-%d %H:%M")
            .to_string();

        // 高亮关键词
        let highlighted = highlight_keyword(&entry.command, keyword);

        let line = format!(
            "{:>3}. {} {}{}",
            (index + 1).to_string().dimmed(),
            highlighted,
            if entry.count > 1 {
                format!("({}x) ", entry.count).yellow().to_string()
            } else {
                "".to_string()
            },
            time_str.dimmed()
        );
        output.push(line);
    }

    output.join("\n")
}

/// 清空历史记录
async fn clear_history(history: &Arc<RwLock<HistoryManager>>) -> String {
    let mut history = history.write().await;

    match history.clear() {
        Ok(_) => "✓ 历史记录已清空".green().to_string(),
        Err(e) => format!("{} {}", "清空历史记录失败:".red(), e),
    }
}

/// 显示统计信息
async fn show_stats(history: &Arc<RwLock<HistoryManager>>) -> String {
    let history = history.read().await;

    let stats = history.stats();

    let mut output = Vec::new();
    output.push(format!("{}", "历史记录统计".bold().cyan()));
    output.push("".to_string());
    output.push(format!("  总记录数:     {}", stats.total_entries.to_string().yellow()));
    output.push(format!("  总执行次数:   {}", stats.total_executions.to_string().yellow()));
    output.push(format!("  唯一命令数:   {}", stats.unique_commands.to_string().yellow()));

    if stats.total_entries > 0 {
        let avg_executions = stats.total_executions as f64 / stats.unique_commands as f64;
        output.push(format!("  平均执行次数: {}", format!("{:.1}", avg_executions).yellow()));
    }

    output.join("\n")
}

/// 高亮关键词
fn highlight_keyword(text: &str, keyword: &str) -> String {
    let keyword_lower = keyword.to_lowercase();
    let mut result = String::new();
    let mut last_pos = 0;

    for (pos, _) in text.to_lowercase().match_indices(&keyword_lower) {
        // 添加关键词之前的文本
        result.push_str(&text[last_pos..pos].cyan().to_string());

        // 添加高亮的关键词
        result.push_str(&text[pos..pos + keyword.len()].yellow().bold().to_string());

        last_pos = pos + keyword.len();
    }

    // 添加剩余的文本
    result.push_str(&text[last_pos..].cyan().to_string());

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::HistoryManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let temp_file = std::env::temp_dir().join(format!("test_history_{}.json", rand::random::<u32>()));
        let mut manager = HistoryManager::new(temp_file, 100);

        manager.add("git status", true);
        manager.add("git log", true);
        manager.add("ls -la", true);
        manager.add("echo hello", true);

        Arc::new(RwLock::new(manager))
    }

    #[tokio::test]
    async fn test_show_recent_history() {
        let history = create_test_history();
        let result = show_recent_history(&history, 10).await;

        assert!(result.contains("最近的历史记录"));
        assert!(result.contains("git status") || result.contains("git log"));
    }

    #[tokio::test]
    async fn test_search_history() {
        let history = create_test_history();
        let result = search_history(&history, "git").await;

        assert!(result.contains("搜索结果"));
        assert!(result.contains("找到 2 条"));
    }

    #[tokio::test]
    async fn test_search_history_no_results() {
        let history = create_test_history();
        let result = search_history(&history, "nonexistent").await;

        assert!(result.contains("未找到匹配的历史记录"));
    }

    #[tokio::test]
    async fn test_clear_history() {
        let history = create_test_history();

        // 清空前应该有记录
        {
            let h = history.read().await;
            assert!(h.stats().total_entries > 0);
        }

        let result = clear_history(&history).await;
        assert!(result.contains("已清空"));

        // 清空后应该没有记录
        {
            let h = history.read().await;
            assert_eq!(h.stats().total_entries, 0);
        }
    }

    #[tokio::test]
    async fn test_show_stats() {
        let history = create_test_history();
        let result = show_stats(&history).await;

        assert!(result.contains("历史记录统计"));
        assert!(result.contains("总记录数"));
        assert!(result.contains("总执行次数"));
    }

    #[test]
    fn test_highlight_keyword() {
        let text = "git status";
        let result = highlight_keyword(text, "git");

        // 结果应该包含原始文本（虽然带有颜色代码）
        assert!(result.contains("git"));
        assert!(result.contains("status"));
    }
}
