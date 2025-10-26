//! 基于历史的建议生成器
//!
//! 根据用户的历史命令生成建议

use super::types::{Suggestion, SuggestionCategory, SuggestionContext, SuggestionSource};
use crate::history::{HistoryManager, SortStrategy};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 基于历史的建议生成器
pub struct HistorySuggester {
    /// 历史管理器
    history: Arc<RwLock<HistoryManager>>,

    /// 最小使用次数阈值（低于此次数的命令不建议）
    min_usage_count: usize,

    /// 最大建议数
    max_suggestions: usize,
}

impl HistorySuggester {
    /// 创建新的历史建议生成器
    pub fn new(history: Arc<RwLock<HistoryManager>>) -> Self {
        Self {
            history,
            min_usage_count: 2, // 至少使用过 2 次
            max_suggestions: 5,
        }
    }

    /// 生成建议
    pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 1. 基于频率的建议（全局高频命令）
        suggestions.extend(self.suggest_frequent_commands().await);

        // 2. 基于上下文的历史建议（在当前目录下常用的命令）
        suggestions.extend(self.suggest_contextual_commands(context).await);

        // 去重
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        suggestions.dedup_by(|a, b| a.command == b.command);

        // 限制数量
        suggestions.truncate(self.max_suggestions);

        suggestions
    }

    /// 基于频率的建议
    async fn suggest_frequent_commands(&self) -> Vec<Suggestion> {
        let history = match self.history.try_read() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

        let entries = history.all(SortStrategy::Frequency);

        entries
            .iter()
            .filter(|entry| entry.count >= self.min_usage_count as u32)
            .filter(|entry| !entry.command.is_empty())
            .filter(|entry| !entry.command.starts_with('/')) // 过滤系统命令
            .take(self.max_suggestions)
            .map(|entry| {
                // 根据使用频率计算分数
                let base_score = 0.6;
                let frequency_bonus = (entry.count as f64).ln() * 0.05;
                let score = (base_score + frequency_bonus).min(0.9);

                let description = if entry.count > 10 {
                    format!("Frequently used ({} times)", entry.count)
                } else {
                    format!("Used {} times", entry.count)
                };

                Suggestion::new(entry.command.clone(), description, score, SuggestionSource::History)
                    .with_category(self.categorize_command(&entry.command))
            })
            .collect()
    }

    /// 基于上下文的历史建议
    async fn suggest_contextual_commands(&self, context: &SuggestionContext) -> Vec<Suggestion> {
        // 如果最近有命令，找相似的历史模式
        if context.recent_commands.is_empty() {
            return Vec::new();
        }

        let history = match self.history.try_read() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

        let entries = history.all(SortStrategy::Frequency);
        let last_cmd = &context.recent_commands[0];

        // 找出与最近命令相关的历史命令
        entries
            .iter()
            .filter(|entry| entry.count >= self.min_usage_count as u32)
            .filter(|entry| !entry.command.is_empty())
            .filter(|entry| self.is_related_command(&entry.command, last_cmd))
            .take(3)
            .map(|entry| {
                let score = 0.7; // 上下文相关的命令给较高分数

                Suggestion::new(
                    entry.command.clone(),
                    format!("Often follows '{}'", last_cmd),
                    score,
                    SuggestionSource::History,
                )
                .with_category(self.categorize_command(&entry.command))
            })
            .collect()
    }

    /// 判断两个命令是否相关
    fn is_related_command(&self, cmd1: &str, cmd2: &str) -> bool {
        // 简单的相关性判断：是否有相同的前缀（如 git, cargo）
        let prefix1 = cmd1.split_whitespace().next().unwrap_or("");
        let prefix2 = cmd2.split_whitespace().next().unwrap_or("");

        prefix1 == prefix2 && cmd1 != cmd2
    }

    /// 为命令分类
    fn categorize_command(&self, command: &str) -> SuggestionCategory {
        let cmd = command.to_lowercase();

        if cmd.starts_with("git") {
            SuggestionCategory::Git
        } else if cmd.contains("test") || cmd.contains("jest") || cmd.contains("pytest") {
            SuggestionCategory::Testing
        } else if cmd.contains("build") || cmd.contains("compile") {
            SuggestionCategory::Building
        } else if cmd.contains("deploy") {
            SuggestionCategory::Deployment
        } else if cmd.contains("cargo") || cmd.contains("npm") || cmd.contains("pip") {
            SuggestionCategory::Project
        } else {
            SuggestionCategory::General
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let mut history = HistoryManager::new("test_history.json", 100);

        // 添加一些测试命令
        for _ in 0..10 {
            history.add("git status".to_string(), true);
        }
        for _ in 0..5 {
            history.add("cargo test".to_string(), true);
        }
        for _ in 0..3 {
            history.add("git commit -m 'test'".to_string(), true);
        }
        history.add("ls -la".to_string(), true);

        Arc::new(RwLock::new(history))
    }

    #[tokio::test]
    async fn test_suggest_frequent_commands() {
        let history = create_test_history();
        let suggester = HistorySuggester::new(history);

        let suggestions = suggester.suggest_frequent_commands().await;

        assert!(!suggestions.is_empty());

        // git status 应该有最高的分数（使用次数最多）
        assert_eq!(suggestions[0].command, "git status");
        assert!(suggestions[0].score > 0.6);
    }

    #[tokio::test]
    async fn test_suggest_with_context() {
        let history = create_test_history();
        let suggester = HistorySuggester::new(history);

        let mut context = SuggestionContext::from_env();
        context.recent_commands.push("git status".to_string());

        let suggestions = suggester.suggest(&context).await;

        assert!(!suggestions.is_empty());

        // 应该包含与 git 相关的建议
        assert!(suggestions.iter().any(|s| s.command.contains("git")));
    }

    #[test]
    fn test_is_related_command() {
        let suggester = HistorySuggester::new(Arc::new(RwLock::new(
            HistoryManager::new("test.json", 100),
        )));

        assert!(suggester.is_related_command("git status", "git commit"));
        assert!(suggester.is_related_command("cargo build", "cargo test"));
        assert!(!suggester.is_related_command("git status", "npm test"));
        assert!(!suggester.is_related_command("git status", "git status")); // 相同命令不相关
    }

    #[test]
    fn test_categorize_command() {
        let suggester = HistorySuggester::new(Arc::new(RwLock::new(
            HistoryManager::new("test.json", 100),
        )));

        assert_eq!(
            suggester.categorize_command("git status"),
            SuggestionCategory::Git
        );
        assert_eq!(
            suggester.categorize_command("cargo test"),
            SuggestionCategory::Testing
        );
        assert_eq!(
            suggester.categorize_command("npm run build"),
            SuggestionCategory::Building
        );
    }

    #[tokio::test]
    async fn test_min_usage_count_filter() {
        // Delete the test file if it exists
        let test_file = "test_min_usage_filter.json";
        let _ = std::fs::remove_file(test_file);

        // Create a fresh history for this test
        let mut history_mgr = HistoryManager::new(test_file, 100);

        // Add test data
        for _ in 0..10 {
            history_mgr.add("git status".to_string(), true);
        }
        for _ in 0..5 {
            history_mgr.add("cargo test".to_string(), true);
        }
        for _ in 0..3 {
            history_mgr.add("git commit -m 'test'".to_string(), true);
        }

        let history = Arc::new(RwLock::new(history_mgr));
        let mut suggester = HistorySuggester::new(history);
        suggester.min_usage_count = 5; // 设置较高的阈值

        let suggestions = suggester.suggest_frequent_commands().await;

        // 应该有建议
        assert!(!suggestions.is_empty());

        // 只有使用次数 >= 5 的命令会被建议
        for suggestion in &suggestions {
            // git status (10次) 和 cargo test (5次) 应该在列表中
            assert!(
                suggestion.command == "git status" || suggestion.command == "cargo test",
                "Unexpected command: {}",
                suggestion.command
            );
        }

        // 应该包含 git status（最高频）
        assert!(suggestions.iter().any(|s| s.command == "git status"));

        // Cleanup
        let _ = std::fs::remove_file(test_file);
    }
}
