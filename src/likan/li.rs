//! 离 ☲ - 建议增强器
//!
//! ## 哲学
//!
//! 离为火，向上燃烧，挥发输出，照亮前路。
//! 在系统中，离将沉淀的模式转化为主动建议。
//!
//! ## 实现
//!
//! 极简策略，三种增强方式：
//! 1. 提升高频命令的优先级
//! 2. 根据序列添加后续建议
//! 3. 在错误时提供修复建议

use super::types::Pattern;
use crate::suggestion::{Suggestion, SuggestionCategory, SuggestionSource}; // 使用公开的 re-export
use std::collections::HashMap;

/// 离：建议增强器
///
/// 应用模式来优化建议质量
pub struct LiEnhancer {
    /// 存储的模式
    patterns: Vec<Pattern>,

    /// 命令权重映射（用于快速查找）
    command_weights: HashMap<String, f64>,
}

impl LiEnhancer {
    /// 创建新的增强器
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            command_weights: HashMap::new(),
        }
    }

    /// 更新模式
    ///
    /// 每次炼化循环后调用
    pub fn update_patterns(&mut self, patterns: Vec<Pattern>) {
        self.patterns = patterns;
        self.rebuild_weights();
    }

    /// 重建权重映射
    ///
    /// 从模式中提取命令权重
    fn rebuild_weights(&mut self) {
        self.command_weights.clear();

        for pattern in &self.patterns {
            if let Some(command) = pattern.command() {
                let confidence = pattern.confidence();

                // 更新权重（取最高置信度）
                self.command_weights
                    .entry(command.to_string())
                    .and_modify(|w| *w = w.max(confidence))
                    .or_insert(confidence);
            }
        }
    }

    /// 增强建议
    ///
    /// 根据模式调整建议的评分和顺序
    pub fn enhance(&self, mut suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        // 1. 调整评分
        for suggestion in &mut suggestions {
            if let Some(&weight) = self.command_weights.get(&suggestion.command) {
                // 根据模式权重调整评分
                // 使用混合策略：原评分 70% + 模式权重 30%
                suggestion.score = suggestion.score * 0.7 + weight * 0.3;
            }
        }

        // 2. 重新排序
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 3. 限制数量（保持前N个）
        suggestions.truncate(10);

        suggestions
    }

    /// 为当前上下文添加额外建议
    ///
    /// 基于序列模式和错误修复模式
    pub fn add_contextual_suggestions(
        &self,
        last_command: Option<&str>,
        last_error: Option<&str>,
    ) -> Vec<Suggestion> {
        let mut additional = Vec::new();

        // 1. 基于序列模式
        if let Some(last_cmd) = last_command {
            for pattern in &self.patterns {
                if let Pattern::Sequence {
                    commands,
                    confidence,
                    ..
                } = pattern
                {
                    // 如果上一个命令匹配序列的第一个，添加第二个
                    if commands.len() >= 2 && commands[0] == last_cmd {
                        additional.push(Suggestion::new(
                            commands[1].clone(),
                            format!("常在 {} 后执行", last_cmd),
                            *confidence * 0.9, // 略微降低评分
                            SuggestionSource::Rule, // 使用规则来源
                        ));
                    }
                }
            }
        }

        // 2. 基于错误修复模式
        if let Some(error) = last_error {
            for pattern in &self.patterns {
                if let Pattern::ErrorFix {
                    error_pattern,
                    fix_command,
                    success_rate,
                } = pattern
                {
                    // 简单的错误匹配（包含子串）
                    if error.contains(error_pattern) {
                        additional.push(Suggestion::new(
                            fix_command.clone(),
                            format!("此命令通常可修复该错误（成功率 {:.0}%）", success_rate * 100.0),
                            *success_rate,
                            SuggestionSource::Rule, // 使用规则来源
                        ));
                    }
                }
            }
        }

        // 去重和排序
        additional.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        additional
    }

    /// 获取当前模式数量
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// 获取当前高置信度模式数量
    pub fn high_confidence_count(&self) -> usize {
        self.patterns
            .iter()
            .filter(|p| p.is_high_confidence())
            .count()
    }
}

impl Default for LiEnhancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhance_suggestions() {
        let mut enhancer = LiEnhancer::new();

        // 设置模式
        let patterns = vec![
            Pattern::Frequency {
                command: "cargo build".to_string(),
                count: 10,
                confidence: 0.9,
            },
            Pattern::Frequency {
                command: "cargo test".to_string(),
                count: 5,
                confidence: 0.6,
            },
        ];
        enhancer.update_patterns(patterns);

        // 原始建议
        let suggestions = vec![
            Suggestion::new(
                "cargo test".to_string(),
                "运行测试".to_string(),
                0.5,
                SuggestionSource::History,
            ),
            Suggestion::new(
                "cargo build".to_string(),
                "构建项目".to_string(),
                0.5,
                SuggestionSource::History,
            ),
        ];

        // 增强
        let enhanced = enhancer.enhance(suggestions);

        // cargo build 应该排在前面（因为置信度更高）
        assert_eq!(enhanced[0].command, "cargo build");
        assert!(enhanced[0].score > 0.5); // 评分应该被提升
    }

    #[test]
    fn test_add_contextual_suggestions_sequence() {
        let mut enhancer = LiEnhancer::new();

        let patterns = vec![Pattern::Sequence {
            commands: vec!["cargo build".to_string(), "cargo run".to_string()],
            occurrences: 5,
            confidence: 0.8,
        }];
        enhancer.update_patterns(patterns);

        // 上一个命令是 cargo build
        let additional = enhancer.add_contextual_suggestions(Some("cargo build"), None);

        // 应该建议 cargo run
        assert!(!additional.is_empty());
        assert_eq!(additional[0].command, "cargo run");
    }

    #[test]
    fn test_add_contextual_suggestions_error_fix() {
        let mut enhancer = LiEnhancer::new();

        let patterns = vec![Pattern::ErrorFix {
            error_pattern: "type mismatch".to_string(),
            fix_command: "cargo check".to_string(),
            success_rate: 0.85,
        }];
        enhancer.update_patterns(patterns);

        // 上一个命令有错误
        let additional = enhancer.add_contextual_suggestions(
            None,
            Some("error: type mismatch in function foo"),
        );

        // 应该建议 cargo check
        assert!(!additional.is_empty());
        assert_eq!(additional[0].command, "cargo check");
        assert!(additional[0].score >= 0.85);
    }

    #[test]
    fn test_pattern_counts() {
        let mut enhancer = LiEnhancer::new();

        let patterns = vec![
            Pattern::Frequency {
                command: "high".to_string(),
                count: 10,
                confidence: 0.9, // 高置信度
            },
            Pattern::Frequency {
                command: "low".to_string(),
                count: 3,
                confidence: 0.5, // 低置信度
            },
        ];
        enhancer.update_patterns(patterns);

        assert_eq!(enhancer.pattern_count(), 2);
        assert_eq!(enhancer.high_confidence_count(), 1);
    }
}
