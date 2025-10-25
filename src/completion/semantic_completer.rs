//! 语义补全器
//!
//! 基于 Intent DSL 和模糊匹配的语义补全：
//! - Intent 意图识别补全
//! - 模糊匹配（Levenshtein 距离）
//! - 上下文感知补全
//!
//! # 特性
//!
//! - **响应时间**: 10-50ms
//! - **确定性**: 0.4-0.8
//! - **智能程度**: 理解语义，提供相关建议

use super::types::{Candidate, CompletionSource};
use crate::command::CommandRegistry;
use crate::dsl::intent::matcher::{levenshtein_distance, string_similarity};
use crate::dsl::intent::{IntentMatch, IntentMatcher};
use crate::history::{HistoryManager, SortStrategy};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 语义补全器
///
/// # 补全策略
///
/// 1. **Intent 意图匹配**: 基于用户输入识别意图，推荐相关命令
/// 2. **模糊匹配**: 使用 Levenshtein 距离查找相似命令
/// 3. **上下文感知**: 结合历史记录和当前上下文
pub struct SemanticCompleter {
    /// 命令注册表
    command_registry: Arc<CommandRegistry>,

    /// 历史管理器
    history: Arc<RwLock<HistoryManager>>,

    /// Intent 匹配器
    intent_matcher: Arc<IntentMatcher>,

    /// 模糊匹配相似度阈值（0.0 - 1.0）
    fuzzy_threshold: f64,
}

impl SemanticCompleter {
    /// 创建新的语义补全器
    pub fn new(
        command_registry: Arc<CommandRegistry>,
        history: Arc<RwLock<HistoryManager>>,
        intent_matcher: Arc<IntentMatcher>,
    ) -> Self {
        Self {
            command_registry,
            history,
            intent_matcher,
            fuzzy_threshold: 0.6, // 默认相似度阈值
        }
    }

    /// 设置模糊匹配阈值
    pub fn with_fuzzy_threshold(mut self, threshold: f64) -> Self {
        self.fuzzy_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// 统一补全入口
    ///
    /// # 补全优先级
    ///
    /// 1. Intent 意图匹配（高置信度）
    /// 2. 模糊命令匹配
    /// 3. 模糊历史匹配
    pub fn complete(&self, input: &str) -> Vec<Candidate> {
        let mut candidates = Vec::new();

        // 1. Intent 意图匹配
        candidates.extend(self.complete_by_intent(input));

        // 2. 模糊命令匹配
        candidates.extend(self.complete_by_fuzzy_command(input));

        // 3. 模糊历史匹配
        candidates.extend(self.complete_by_fuzzy_history(input));

        // 去重（基于 text）
        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.dedup_by(|a, b| a.text == b.text);

        // 最多返回 10 个候选
        candidates.truncate(10);

        candidates
    }

    /// 基于 Intent 意图匹配补全
    ///
    /// # 示例
    ///
    /// ```text
    /// 输入: 统计 python 行数
    /// 输出: [find . -name "*.py" | xargs wc -l]
    /// ```
    fn complete_by_intent(&self, input: &str) -> Vec<Candidate> {
        // 使用 IntentMatcher 识别意图
        let matches: Vec<IntentMatch> = self.intent_matcher.match_intent(input);

        matches
            .iter()
            .filter(|m| m.confidence >= 0.4) // 语义补全的最低阈值
            .map(|m| {
                // 根据意图生成候选命令建议
                let suggestion = self.intent_to_command_suggestion(&m.intent.name);
                let description = format!("Intent: {} ({})", m.intent.name, m.confidence);

                // 置信度映射到 0.4-0.8 范围（语义补全的特征）
                let score = 0.4 + (m.confidence * 0.4);

                Candidate::with_score(suggestion, description, score, CompletionSource::Semantic)
            })
            .collect()
    }

    /// 基于模糊匹配补全系统命令
    ///
    /// # 示例
    ///
    /// ```text
    /// 输入: hlep  (拼写错误)
    /// 输出: [/help (similarity: 0.8)]
    /// ```
    fn complete_by_fuzzy_command(&self, input: &str) -> Vec<Candidate> {
        // 去掉可能的 / 前缀
        let query = input.trim_start_matches('/');

        self.command_registry
            .list()
            .iter()
            .filter_map(|cmd| {
                let similarity = string_similarity(query, &cmd.name);

                if similarity >= self.fuzzy_threshold {
                    let description = format!("{} (similarity: {:.2})", cmd.desc, similarity);
                    let score = 0.4 + (similarity * 0.4); // 映射到 0.4-0.8

                    Some(Candidate::with_score(
                        format!("/{}", cmd.name),
                        description,
                        score,
                        CompletionSource::Semantic,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    /// 基于模糊匹配补全历史命令
    ///
    /// # 示例
    ///
    /// ```text
    /// 输入: git statsu  (拼写错误)
    /// 输出: [git status (similarity: 0.91)]
    /// ```
    fn complete_by_fuzzy_history(&self, input: &str) -> Vec<Candidate> {
        // 使用 try_read 避免阻塞
        let history = match self.history.try_read() {
            Ok(h) => h,
            Err(_) => return Vec::new(),
        };

        let entries = history.all(SortStrategy::Frequency);

        entries
            .iter()
            .filter(|entry| !entry.command.is_empty())
            .filter(|entry| !entry.command.starts_with('/')) // 过滤系统命令
            .filter_map(|entry| {
                let similarity = string_similarity(input, &entry.command);

                if similarity >= self.fuzzy_threshold {
                    let description = if entry.count > 1 {
                        format!("used {} times (similarity: {:.2})", entry.count, similarity)
                    } else {
                        format!("history (similarity: {:.2})", similarity)
                    };

                    let score = 0.4 + (similarity * 0.4); // 映射到 0.4-0.8

                    Some(Candidate::with_score(
                        entry.command.clone(),
                        description,
                        score,
                        CompletionSource::Semantic,
                    ))
                } else {
                    None
                }
            })
            .take(5) // 最多 5 个模糊历史匹配
            .collect()
    }

    /// 将 Intent 名称映射为命令建议
    ///
    /// 这是一个简化版本，实际应该根据提取的实体生成更精确的命令
    fn intent_to_command_suggestion(&self, intent_name: &str) -> String {
        match intent_name {
            "count_python_lines" => "find . -name '*.py' | xargs wc -l".to_string(),
            "count_rust_lines" => "find . -name '*.rs' | xargs wc -l".to_string(),
            "find_large_files" => "find . -type f -size +10M".to_string(),
            "find_error_logs" => "grep -r 'ERROR' . --include='*.log'".to_string(),
            "git_status" => "git status".to_string(),
            "git_log" => "git log --oneline -10".to_string(),
            "disk_usage" => "du -sh * | sort -hr".to_string(),
            "process_list" => "ps aux | head -20".to_string(),
            _ => format!("# Intent: {}", intent_name),
        }
    }

    /// 计算两个字符串的 Levenshtein 距离（复用现有实现）
    #[allow(dead_code)]
    fn levenshtein(&self, s1: &str, s2: &str) -> usize {
        levenshtein_distance(s1, s2)
    }

    /// 计算字符串相似度（复用现有实现）
    #[allow(dead_code)]
    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        string_similarity(s1, s2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::dsl::intent::{Intent, IntentDomain};

    fn create_test_registry() -> Arc<CommandRegistry> {
        let mut registry = CommandRegistry::new();
        registry.register(Command::from_fn("help", "Show help", |_| String::new()));
        registry.register(Command::from_fn("history", "Show history", |_| {
            String::new()
        }));
        registry.register(Command::from_fn("config", "Show config", |_| String::new()));
        Arc::new(registry)
    }

    fn create_test_history() -> Arc<RwLock<HistoryManager>> {
        let mut history = HistoryManager::new("test_history.json", 100);

        history.add("git status".to_string(), true);
        history.add("git status".to_string(), true); // 使 count = 2
        history.add("git commit -m 'test'".to_string(), true);
        history.add("cargo test".to_string(), true);

        Arc::new(RwLock::new(history))
    }

    fn create_test_intent_matcher() -> Arc<IntentMatcher> {
        let mut matcher = IntentMatcher::new();

        // 添加一些测试 Intent
        matcher.register(Intent::new(
            "git_status",
            IntentDomain::SystemOps,
            vec!["git".to_string(), "状态".to_string()],
            vec![r"git.*status".to_string()],
            0.5,
        ));

        Arc::new(matcher)
    }

    #[test]
    fn test_fuzzy_command_matching() {
        let completer = SemanticCompleter::new(
            create_test_registry(),
            create_test_history(),
            create_test_intent_matcher(),
        )
        .with_fuzzy_threshold(0.5); // 降低阈值以匹配更多拼写错误

        // 测试拼写错误的命令
        let candidates = completer.complete_by_fuzzy_command("hel"); // help 少打一个字母
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text.contains("help")));
    }

    #[test]
    fn test_fuzzy_history_matching() {
        let completer = SemanticCompleter::new(
            create_test_registry(),
            create_test_history(),
            create_test_intent_matcher(),
        );

        // 测试拼写错误的历史命令
        let candidates = completer.complete_by_fuzzy_history("git statsu"); // status 拼错
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|c| c.text.contains("git status")));
    }

    #[test]
    fn test_semantic_score_range() {
        let completer = SemanticCompleter::new(
            create_test_registry(),
            create_test_history(),
            create_test_intent_matcher(),
        );

        let candidates = completer.complete("help");

        // 语义补全的分数应该在 0.4-0.8 范围内
        for candidate in candidates {
            assert!(
                candidate.score >= 0.4 && candidate.score <= 0.8,
                "Score {} out of range [0.4, 0.8]",
                candidate.score
            );
        }
    }

    #[test]
    fn test_complete_unified() {
        let completer = SemanticCompleter::new(
            create_test_registry(),
            create_test_history(),
            create_test_intent_matcher(),
        )
        .with_fuzzy_threshold(0.5); // 降低阈值

        // 统一接口应该整合所有补全源
        let candidates = completer.complete("hel"); // 使用会产生模糊匹配的输入
        assert!(!candidates.is_empty());

        // 应该按分数降序排序
        for i in 1..candidates.len() {
            assert!(candidates[i - 1].score >= candidates[i].score);
        }
    }

    #[test]
    fn test_fuzzy_threshold() {
        let completer = SemanticCompleter::new(
            create_test_registry(),
            create_test_history(),
            create_test_intent_matcher(),
        )
        .with_fuzzy_threshold(0.9); // 高阈值

        // 高阈值应该过滤掉大部分模糊匹配
        let candidates = completer.complete_by_fuzzy_command("hlp"); // 差异较大
        assert!(candidates.is_empty() || candidates.len() < 2);
    }
}
