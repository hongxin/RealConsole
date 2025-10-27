//! 坎 ☵ - 模式提取器
//!
//! ## 哲学
//!
//! 坎为水，向下流动，汇聚于低处，形成深渊。
//! 在系统中，坎从大量数据中沉淀出深层模式。
//!
//! ## 实现
//!
//! 极简策略，只提取最明显的三种模式：
//! 1. 频率模式（Frequency）- 命令使用频率
//! 2. 序列模式（Sequence）- 命令执行序列
//! 3. 错误修复模式（ErrorFix）- 错误后的有效命令

use super::types::{FurnaceConfig, Pattern};
use crate::suggestion::feedback::SuggestionStats; // 使用公开的 re-export
use crate::tracer::entry::TraceEntry;
use crate::tracer::types::{Dimension, EntryType, Status};
use std::collections::HashMap;

/// 坎：模式提取器
///
/// 从 Tracer 和 Feedback 中提取深层模式
pub struct KanExtractor {
    config: FurnaceConfig,
}

impl KanExtractor {
    /// 创建新的提取器
    pub fn new(config: FurnaceConfig) -> Self {
        Self { config }
    }

    /// 提取所有模式
    ///
    /// 一分为三：频率、序列、错误修复
    pub fn extract_patterns(
        &self,
        trace_entries: &[TraceEntry],
        suggestion_stats: &HashMap<String, SuggestionStats>,
    ) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // 1. 提取频率模式
        patterns.extend(self.extract_frequency_patterns(trace_entries));

        // 2. 提取序列模式
        patterns.extend(self.extract_sequence_patterns(trace_entries));

        // 3. 提取错误修复模式
        patterns.extend(self.extract_error_fix_patterns(trace_entries));

        // 4. 从 Feedback 增强
        patterns.extend(self.extract_from_feedback(suggestion_stats));

        // 5. 过滤和排序
        self.filter_and_sort(patterns)
    }

    /// 提取频率模式
    ///
    /// 统计命令使用频率，频率越高置信度越高
    fn extract_frequency_patterns(&self, entries: &[TraceEntry]) -> Vec<Pattern> {
        let mut command_counts: HashMap<String, usize> = HashMap::new();

        // 统计 Shell 命令频率
        for entry in entries {
            if entry.dimension == Dimension::Statistics
                && entry.entry_type == EntryType::ShellCommand
            {
                // 使用 content 字段而不是 command
                *command_counts.entry(entry.content.clone()).or_insert(0) += 1;
            }
        }

        // 转换为模式
        command_counts
            .into_iter()
            .filter(|(_, count)| *count >= self.config.min_frequency)
            .map(|(command, count)| {
                // 置信度 = count / total，但不超过 1.0
                let confidence = (count as f64 / entries.len() as f64).min(1.0);
                Pattern::Frequency {
                    command,
                    count,
                    confidence,
                }
            })
            .collect()
    }

    /// 提取序列模式
    ///
    /// 识别常见的命令执行序列
    fn extract_sequence_patterns(&self, entries: &[TraceEntry]) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // 提取所有成功的 Shell 命令序列
        let commands: Vec<String> = entries
            .iter()
            .filter(|e| {
                e.dimension == Dimension::Statistics
                    && e.entry_type == EntryType::ShellCommand
                    && e.status == Status::Success
            })
            .map(|e| e.content.clone()) // 使用 content 字段
            .collect();

        // 简单的 2-gram 序列识别
        if commands.len() >= 2 {
            let mut sequence_counts: HashMap<Vec<String>, usize> = HashMap::new();

            for window in commands.windows(2) {
                let seq = window.to_vec();
                *sequence_counts.entry(seq).or_insert(0) += 1;
            }

            // 转换为模式
            for (commands, occurrences) in sequence_counts {
                if occurrences >= self.config.min_frequency {
                    let confidence = (occurrences as f64 / entries.len() as f64).min(1.0);
                    patterns.push(Pattern::Sequence {
                        commands,
                        occurrences,
                        confidence,
                    });
                }
            }
        }

        patterns
    }

    /// 提取错误修复模式
    ///
    /// 识别错误后执行的有效命令
    fn extract_error_fix_patterns(&self, entries: &[TraceEntry]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        let mut error_fix_pairs: HashMap<(String, String), (usize, usize)> = HashMap::new();

        // 查找 失败 -> 成功 的配对
        for i in 0..entries.len().saturating_sub(1) {
            let current = &entries[i];
            let next = &entries[i + 1];

            // 当前命令失败
            if let Status::Failed(error) = &current.status {
                // 下一个命令成功
                if next.status == Status::Success {
                    // 使用 content 字段
                    let fix_cmd = &next.content;

                    // 简化错误信息（取前50字符）
                    let error_pattern = error.chars().take(50).collect::<String>();

                    let key = (error_pattern, fix_cmd.clone());
                    let (success, total) = error_fix_pairs.entry(key).or_insert((0, 0));
                    *success += 1;
                    *total += 1;
                }
            }
        }

        // 转换为模式
        for ((error_pattern, fix_command), (success, total)) in error_fix_pairs {
            if total >= self.config.min_frequency {
                let success_rate = success as f64 / total as f64;
                if success_rate >= self.config.min_confidence {
                    patterns.push(Pattern::ErrorFix {
                        error_pattern,
                        fix_command,
                        success_rate,
                    });
                }
            }
        }

        patterns
    }

    /// 从 Feedback 提取模式
    ///
    /// 利用现有的反馈学习系统
    fn extract_from_feedback(
        &self,
        suggestion_stats: &HashMap<String, SuggestionStats>,
    ) -> Vec<Pattern> {
        suggestion_stats
            .iter()
            .filter(|(_, stats)| stats.shown_count >= self.config.min_frequency)
            .map(|(command, stats)| Pattern::Frequency {
                command: command.clone(),
                count: stats.accepted_count,
                confidence: stats.quality_score(),
            })
            .collect()
    }

    /// 过滤和排序模式
    ///
    /// 只保留高置信度的，按置信度排序
    fn filter_and_sort(&self, mut patterns: Vec<Pattern>) -> Vec<Pattern> {
        // 过滤低置信度
        patterns.retain(|p| p.confidence() >= self.config.min_confidence);

        // 按置信度排序（降序）
        patterns.sort_by(|a, b| {
            b.confidence()
                .partial_cmp(&a.confidence())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 限制数量
        patterns.truncate(self.config.max_patterns);

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_entry(command: &str, status: Status) -> TraceEntry {
        use uuid::Uuid;
        TraceEntry {
            id: Uuid::new_v4(),
            dimension: Dimension::Statistics,
            entry_type: EntryType::ShellCommand,
            timestamp: Utc::now(),
            content: command.to_string(), // 使用 content 字段
            status,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_extract_frequency_patterns() {
        let extractor = KanExtractor::new(FurnaceConfig::default());

        let entries = vec![
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo test", Status::Success),
        ];

        let patterns = extractor.extract_frequency_patterns(&entries);
        assert!(!patterns.is_empty());

        // 应该有 cargo build 的模式
        assert!(patterns
            .iter()
            .any(|p| matches!(p, Pattern::Frequency { command, count, .. } if command == "cargo build" && *count == 3)));
    }

    #[test]
    fn test_extract_sequence_patterns() {
        let extractor = KanExtractor::new(FurnaceConfig::default());

        let entries = vec![
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo run", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo run", Status::Success),
            create_test_entry("cargo build", Status::Success),
            create_test_entry("cargo run", Status::Success),
        ];

        let patterns = extractor.extract_sequence_patterns(&entries);
        assert!(!patterns.is_empty());

        // 应该有 build -> run 的序列
        assert!(patterns.iter().any(|p| matches!(
            p,
            Pattern::Sequence { commands, occurrences, .. }
            if commands.len() == 2 && commands[0] == "cargo build" && commands[1] == "cargo run" && *occurrences >= 3
        )));
    }

    #[test]
    fn test_filter_and_sort() {
        let extractor = KanExtractor::new(FurnaceConfig {
            min_confidence: 0.6,
            ..Default::default()
        });

        let patterns = vec![
            Pattern::Frequency {
                command: "low".to_string(),
                count: 1,
                confidence: 0.3, // 低于阈值
            },
            Pattern::Frequency {
                command: "high".to_string(),
                count: 10,
                confidence: 0.9,
            },
            Pattern::Frequency {
                command: "medium".to_string(),
                count: 5,
                confidence: 0.7,
            },
        ];

        let filtered = extractor.filter_and_sort(patterns);

        // 应该过滤掉低置信度
        assert_eq!(filtered.len(), 2);

        // 应该按置信度降序
        assert!(filtered[0].confidence() > filtered[1].confidence());
    }
}
