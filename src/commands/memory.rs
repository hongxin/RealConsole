//! 记忆管理命令
//!
//! 提供记忆查看、搜索、清空等功能

use crate::command::{Command, CommandRegistry};
use crate::memory::{EntryType, Memory};
use colored::Colorize;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册记忆管理命令
pub fn register_memory_commands(
    registry: &mut CommandRegistry,
    memory: Arc<RwLock<Memory>>,
) {
    // /memory 命令
    let memory_cmd = Command::from_fn(
        "memory",
        "记忆管理: memory [recent|search|clear|dump|save|type]",
        move |arg: &str| handle_memory(arg, Arc::clone(&memory)),
    )
    .with_aliases(vec!["mem".to_string(), "m".to_string()])
    .with_group("memory");
    registry.register(memory_cmd);
}

/// 处理 /memory 命令
fn handle_memory(arg: &str, memory: Arc<RwLock<Memory>>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_memory_status(memory);
    }

    let subcommand = parts[0];
    let rest = parts.get(1..).unwrap_or(&[]).join(" ");

    match subcommand {
        "recent" | "r" => handle_memory_recent(&rest, memory),
        "search" | "s" => handle_memory_search(&rest, memory),
        "clear" | "c" => handle_memory_clear(memory),
        "dump" | "d" => handle_memory_dump(memory),
        "save" => handle_memory_save(&rest, memory),
        "type" | "t" => handle_memory_type(&rest, memory),
        "help" | "h" => memory_help(),
        _ => format!("{} 未知子命令: {}\n使用 /memory help 查看帮助", "错误:".red(), subcommand),
    }
}

/// 显示记忆状态
fn handle_memory_status(memory: Arc<RwLock<Memory>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            let mut lines = vec![
                format!("{}", "记忆系统状态".bold().cyan()),
                format!("  当前条目: {}", mem.len().to_string().green()),
                format!("  最大容量: {}", "100".dimmed()),
            ];

            if !mem.is_empty() {
                lines.push(String::new());
                lines.push(format!("{}", "最近 3 条记忆:".dimmed()));
                for entry in mem.recent(3) {
                    lines.push(format!("  {}", entry.preview()));
                }
            }

            lines.join("\n")
        })
    })
}

/// 查看最近的记忆
fn handle_memory_recent(arg: &str, memory: Arc<RwLock<Memory>>) -> String {
    let n: usize = arg.trim().parse().unwrap_or(10);

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            let entries = mem.recent(n);

            if entries.is_empty() {
                return format!("{}", "暂无记忆".dimmed());
            }

            let mut lines = vec![
                format!("{} {} 条记忆:", "最近".bold().cyan(), entries.len().to_string().green()),
            ];

            for entry in entries {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 搜索记忆
fn handle_memory_search(keyword: &str, memory: Arc<RwLock<Memory>>) -> String {
    if keyword.is_empty() {
        return format!("{} 请提供搜索关键词", "错误:".red());
    }

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            let results = mem.search(keyword);

            if results.is_empty() {
                return format!("{} 未找到包含 {} 的记忆", "提示:".yellow(), keyword.cyan());
            }

            let mut lines = vec![
                format!("{} {} 条结果 (关键词: {}):",
                    "找到".bold().green(),
                    results.len().to_string().green(),
                    keyword.cyan()
                ),
            ];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 清空记忆
fn handle_memory_clear(memory: Arc<RwLock<Memory>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mut mem = memory.write().await;
            let count = mem.len();
            mem.clear();
            format!("{} 已清空 {} 条记忆", "✓".green(), count.to_string().dimmed())
        })
    })
}

/// 导出所有记忆
fn handle_memory_dump(memory: Arc<RwLock<Memory>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            let entries = mem.dump();

            if entries.is_empty() {
                return format!("{}", "暂无记忆".dimmed());
            }

            let mut lines = vec![
                format!("{} {} 条记忆:", "全部".bold().cyan(), entries.len().to_string().green()),
            ];

            for entry in entries {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 保存记忆到文件
fn handle_memory_save(path: &str, memory: Arc<RwLock<Memory>>) -> String {
    let path = if path.is_empty() {
        "memory_export.jsonl"
    } else {
        path
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            match mem.save_to_file(path) {
                Ok(count) => {
                    format!("{} 已保存 {} 条记忆到 {}",
                        "✓".green(),
                        count.to_string().green(),
                        path.cyan()
                    )
                }
                Err(e) => {
                    format!("{} 保存失败: {}", "错误:".red(), e)
                }
            }
        })
    })
}

/// 按类型过滤记忆
fn handle_memory_type(type_str: &str, memory: Arc<RwLock<Memory>>) -> String {
    let entry_type = match type_str.to_lowercase().as_str() {
        "user" | "u" => EntryType::User,
        "assistant" | "a" => EntryType::Assistant,
        "system" | "s" => EntryType::System,
        "shell" | "sh" => EntryType::Shell,
        "tool" | "t" => EntryType::Tool,
        _ => {
            return format!("{} 未知类型: {}\n支持的类型: user, assistant, system, shell, tool",
                "错误:".red(),
                type_str
            );
        }
    };

    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let mem = memory.read().await;
            let results = mem.filter_by_type(entry_type);

            if results.is_empty() {
                return format!("{} 未找到类型为 {} 的记忆", "提示:".yellow(), entry_type);
            }

            let mut lines = vec![
                format!("{} {} 条 {} 记忆:",
                    "找到".bold().green(),
                    results.len().to_string().green(),
                    entry_type
                ),
            ];

            for entry in results {
                lines.push(entry.format());
            }

            lines.join("\n")
        })
    })
}

/// 记忆命令帮助
fn memory_help() -> String {
    format!(
        r#"{title}

{subtitle}
  /memory              - 显示记忆状态和最近记忆
  /memory recent <n>   - 查看最近 N 条记忆（默认 10）
  /memory search <关键词> - 搜索包含关键词的记忆
  /memory type <类型>   - 按类型过滤（user/assistant/system/shell/tool）
  /memory clear        - 清空所有记忆
  /memory dump         - 导出所有记忆
  /memory save [路径]  - 保存记忆到文件（默认 memory_export.jsonl）

{examples}
  /memory recent 20
  /memory search "rust"
  /memory type user
  /memory save my_memory.jsonl

{shortcuts}
  recent → r, search → s, clear → c, dump → d, type → t
"#,
        title = "记忆管理".bold().cyan(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        shortcuts = "快捷命令:".dimmed()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::Memory;

    fn create_test_memory() -> Arc<RwLock<Memory>> {
        let mut mem = Memory::new(100);
        mem.add("Hello world".to_string(), EntryType::User);
        mem.add("Hi there".to_string(), EntryType::Assistant);
        mem.add("Test command".to_string(), EntryType::Shell);
        Arc::new(RwLock::new(mem))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_status() {
        let memory = create_test_memory();
        let result = handle_memory_status(memory);
        assert!(result.contains("当前条目: 3"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_search() {
        let memory = create_test_memory();
        let result = handle_memory_search("hello", memory);
        assert!(result.contains("找到"));
        assert!(result.contains("Hello"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_clear() {
        let memory = create_test_memory();
        let result = handle_memory_clear(Arc::clone(&memory));
        assert!(result.contains("已清空 3 条记忆"));

        let mem = memory.read().await;
        assert_eq!(mem.len(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_recent() {
        let memory = create_test_memory();
        let result = handle_memory_recent("2", memory);
        assert!(result.contains("最近"));
        assert!(result.contains("2 条记忆"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_dump() {
        let memory = create_test_memory();
        let result = handle_memory_dump(memory);
        assert!(result.contains("全部"));
        assert!(result.contains("3 条记忆"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_search_no_keyword() {
        let memory = create_test_memory();
        let result = handle_memory_search("", memory);
        assert!(result.contains("错误"));
        assert!(result.contains("关键词"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_search_no_results() {
        let memory = create_test_memory();
        let result = handle_memory_search("nonexistent", memory);
        assert!(result.contains("未找到"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_type_user() {
        let memory = create_test_memory();
        let result = handle_memory_type("user", memory);
        assert!(result.contains("找到") || result.contains("记忆"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_memory_type_invalid() {
        let memory = create_test_memory();
        let result = handle_memory_type("invalid", memory);
        assert!(result.contains("未知类型"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_memory_with_empty_args() {
        let memory = create_test_memory();
        let result = handle_memory("", memory);
        assert!(result.contains("记忆系统状态"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_handle_memory_unknown_subcommand() {
        let memory = create_test_memory();
        let result = handle_memory("unknown", memory);
        assert!(result.contains("未知子命令"));
    }

    #[test]
    fn test_memory_help() {
        let result = memory_help();
        assert!(result.contains("记忆管理"));
        assert!(result.contains("recent"));
        assert!(result.contains("search"));
    }
}
