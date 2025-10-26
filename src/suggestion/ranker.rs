//! 建议排序器
//!
//! 负责合并、排序和去重来自多个来源的建议

use super::types::{Suggestion, SuggestionCategory, SuggestionSource};
use std::collections::HashMap;

/// 建议排序器
///
/// 使用多维度评分系统对建议进行排序
pub struct SuggestionRanker {
    /// 最大建议数
    max_suggestions: usize,

    /// 最小分数阈值
    min_score: f64,

    /// 多样性因子（0.0-1.0，越高越注重多样性）
    diversity_factor: f64,

    /// 来源权重
    source_weights: HashMap<SuggestionSource, f64>,

    /// 类别权重
    category_weights: HashMap<SuggestionCategory, f64>,
}

impl SuggestionRanker {
    /// 创建新的排序器
    pub fn new() -> Self {
        let mut source_weights = HashMap::new();
        source_weights.insert(SuggestionSource::Context, 1.2); // 上下文建议最可靠
        source_weights.insert(SuggestionSource::History, 1.1); // 历史建议次可靠
        source_weights.insert(SuggestionSource::Llm, 1.0); // LLM 建议基准
        source_weights.insert(SuggestionSource::Rule, 1.15);

        let mut category_weights = HashMap::new();
        category_weights.insert(SuggestionCategory::Diagnostic, 1.1); // 诊断类优先
        category_weights.insert(SuggestionCategory::Git, 1.05);
        category_weights.insert(SuggestionCategory::Testing, 1.05);
        category_weights.insert(SuggestionCategory::Building, 1.0);
        category_weights.insert(SuggestionCategory::Project, 1.0);
        category_weights.insert(SuggestionCategory::Deployment, 0.95);
        category_weights.insert(SuggestionCategory::General, 0.9);

        Self {
            max_suggestions: 5,
            min_score: 0.3,
            diversity_factor: 0.3, // 30% 多样性权重
            source_weights,
            category_weights,
        }
    }

    /// 设置最大建议数
    pub fn with_max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }

    /// 设置最小分数阈值
    pub fn with_min_score(mut self, min: f64) -> Self {
        self.min_score = min;
        self
    }

    /// 排序和融合建议
    ///
    /// 采用"一分为三"的融合策略：
    /// 1. 去重（合一）
    /// 2. 评分（分化）
    /// 3. 融合（和谐）
    pub fn rank(&self, mut suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        if suggestions.is_empty() {
            return Vec::new();
        }

        // 1. 去重：相同命令只保留最高分的
        suggestions = self.deduplicate(suggestions);

        // 2. 计算最终分数
        for suggestion in &mut suggestions {
            let final_score = self.calculate_final_score(suggestion);
            suggestion.score = final_score;
        }

        // 3. 按分数排序
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 4. 应用多样性过滤
        suggestions = self.apply_diversity_filter(suggestions);

        // 5. 过滤低分建议
        suggestions.retain(|s| s.score >= self.min_score);

        // 6. 限制数量
        suggestions.truncate(self.max_suggestions);

        suggestions
    }

    /// 去重：相同命令只保留最高分的
    fn deduplicate(&self, suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        let mut best_suggestions: HashMap<String, Suggestion> = HashMap::new();

        for suggestion in suggestions {
            let cmd = suggestion.command.clone();

            if let Some(existing) = best_suggestions.get(&cmd) {
                // 如果已存在，比较分数，保留更高的
                if suggestion.score > existing.score {
                    best_suggestions.insert(cmd, suggestion);
                }
            } else {
                best_suggestions.insert(cmd, suggestion);
            }
        }

        best_suggestions.into_values().collect()
    }

    /// 计算最终分数
    ///
    /// 最终分数 = 基础分数 × 来源权重 × 类别权重
    fn calculate_final_score(&self, suggestion: &Suggestion) -> f64 {
        let base_score = suggestion.score;

        // 来源权重
        let source_weight = self
            .source_weights
            .get(&suggestion.source)
            .copied()
            .unwrap_or(1.0);

        // 类别权重
        let category_weight = self
            .category_weights
            .get(&suggestion.category)
            .copied()
            .unwrap_or(1.0);

        // 最终分数 = 基础分数 × 来源权重 × 类别权重
        // 限制在 0.0-1.0 之间
        (base_score * source_weight * category_weight).min(1.0)
    }

    /// 应用多样性过滤
    ///
    /// 避免返回过多相似的建议
    fn apply_diversity_filter(&self, suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        if suggestions.len() <= 1 {
            return suggestions;
        }

        let mut filtered = Vec::new();
        filtered.push(suggestions[0].clone());

        for suggestion in suggestions.into_iter().skip(1) {
            // 检查与已选建议的相似度
            let max_similarity = filtered
                .iter()
                .map(|s| self.calculate_similarity(s, &suggestion))
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            // 如果相似度低于阈值，或者分数足够高，则保留
            let similarity_threshold = 1.0 - self.diversity_factor;

            if max_similarity < similarity_threshold || suggestion.score > 0.85 {
                filtered.push(suggestion);
            }
        }

        filtered
    }

    /// 计算两个建议的相似度 (0.0-1.0)
    ///
    /// 基于命令前缀和编辑距离
    fn calculate_similarity(&self, s1: &Suggestion, s2: &Suggestion) -> f64 {
        let cmd1 = &s1.command;
        let cmd2 = &s2.command;

        // 1. 如果命令完全相同，相似度 100%
        if cmd1 == cmd2 {
            return 1.0;
        }

        // 2. 提取命令前缀（第一个单词）
        let prefix1 = cmd1.split_whitespace().next().unwrap_or("");
        let prefix2 = cmd2.split_whitespace().next().unwrap_or("");

        // 3. 如果前缀相同（如都是 git），相似度较高
        if prefix1 == prefix2 && !prefix1.is_empty() {
            return 0.7;
        }

        // 4. 如果类别相同，相似度中等
        if s1.category == s2.category {
            return 0.5;
        }

        // 5. 否则相似度较低
        0.0
    }
}

impl Default for SuggestionRanker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_suggestion(cmd: &str, score: f64, source: SuggestionSource) -> Suggestion {
        Suggestion::new(cmd, "Test suggestion", score, source)
    }

    #[test]
    fn test_deduplicate() {
        let ranker = SuggestionRanker::new();

        let suggestions = vec![
            create_suggestion("git status", 0.8, SuggestionSource::Context),
            create_suggestion("git status", 0.6, SuggestionSource::History),
            create_suggestion("cargo test", 0.7, SuggestionSource::Context),
        ];

        let deduped = ranker.deduplicate(suggestions);

        assert_eq!(deduped.len(), 2);

        // git status 应该保留 0.8 分的版本
        let git_status = deduped.iter().find(|s| s.command == "git status").unwrap();
        assert_eq!(git_status.score, 0.8);
    }

    #[test]
    fn test_calculate_final_score() {
        let ranker = SuggestionRanker::new();

        // Context 来源的建议应该有更高的权重
        let context_suggestion = create_suggestion("git status", 0.8, SuggestionSource::Context)
            .with_category(SuggestionCategory::Git);

        let final_score = ranker.calculate_final_score(&context_suggestion);

        // 0.8 * 1.2 (context weight) * 1.05 (git weight) = 1.008, capped at 1.0
        assert_eq!(final_score, 1.0);
    }

    #[test]
    fn test_calculate_similarity() {
        let ranker = SuggestionRanker::new();

        let s1 = create_suggestion("git status", 0.8, SuggestionSource::Context)
            .with_category(SuggestionCategory::Git);
        let s2 = create_suggestion("git commit", 0.8, SuggestionSource::Context)
            .with_category(SuggestionCategory::Git);
        let s3 = create_suggestion("cargo test", 0.8, SuggestionSource::Context)
            .with_category(SuggestionCategory::Testing);

        // 相同前缀（git）应该有高相似度
        assert_eq!(ranker.calculate_similarity(&s1, &s2), 0.7);

        // 不同前缀但相同类别应该有中等相似度
        let s4 = create_suggestion("npm test", 0.8, SuggestionSource::Context)
            .with_category(SuggestionCategory::Testing);
        assert_eq!(ranker.calculate_similarity(&s3, &s4), 0.5);

        // 完全不同应该有低相似度
        assert_eq!(ranker.calculate_similarity(&s1, &s3), 0.0);
    }

    #[test]
    fn test_rank_basic() {
        let ranker = SuggestionRanker::new().with_max_suggestions(3);

        let suggestions = vec![
            create_suggestion("git status", 0.8, SuggestionSource::Context),
            create_suggestion("cargo test", 0.7, SuggestionSource::History),
            create_suggestion("npm install", 0.6, SuggestionSource::Llm),
            create_suggestion("docker build", 0.5, SuggestionSource::Context),
        ];

        let ranked = ranker.rank(suggestions);

        // 应该返回最多 3 个建议
        assert!(ranked.len() <= 3);

        // 应该按分数排序
        for i in 0..ranked.len() - 1 {
            assert!(ranked[i].score >= ranked[i + 1].score);
        }
    }

    #[test]
    fn test_rank_with_duplicates() {
        let ranker = SuggestionRanker::new();

        let suggestions = vec![
            create_suggestion("git status", 0.8, SuggestionSource::Context),
            create_suggestion("git status", 0.6, SuggestionSource::History),
            create_suggestion("git status", 0.5, SuggestionSource::Llm),
        ];

        let ranked = ranker.rank(suggestions);

        // 应该只返回一个 git status
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].command, "git status");

        // 应该保留最高分的
        // 0.8 * 1.2 (context weight) * 0.9 (general category weight) = 0.864
        assert!(ranked[0].score > 0.85);
    }

    #[test]
    fn test_min_score_filter() {
        let ranker = SuggestionRanker::new().with_min_score(0.5);

        let suggestions = vec![
            create_suggestion("git status", 0.8, SuggestionSource::Context),
            create_suggestion("cargo test", 0.3, SuggestionSource::Llm), // 低于阈值
            create_suggestion("npm install", 0.6, SuggestionSource::History),
        ];

        let ranked = ranker.rank(suggestions);

        // cargo test 应该被过滤掉（最终分数可能低于 0.5）
        assert!(ranked.iter().all(|s| s.score >= 0.5));
    }

    #[test]
    fn test_diversity_filter() {
        let ranker = SuggestionRanker::new()
            .with_max_suggestions(10)
            .with_min_score(0.0);

        let suggestions = vec![
            create_suggestion("git status", 0.9, SuggestionSource::Context)
                .with_category(SuggestionCategory::Git),
            create_suggestion("git commit", 0.8, SuggestionSource::Context)
                .with_category(SuggestionCategory::Git),
            create_suggestion("git pull", 0.75, SuggestionSource::Context)
                .with_category(SuggestionCategory::Git),
            create_suggestion("cargo test", 0.7, SuggestionSource::Context)
                .with_category(SuggestionCategory::Testing),
        ];

        let ranked = ranker.rank(suggestions);

        // 由于多样性过滤，不应该所有 git 命令都被保留
        // 但高分的应该保留
        assert!(ranked.iter().any(|s| s.command.contains("git")));
        assert!(ranked.iter().any(|s| s.command == "git status")); // 最高分
    }

    #[test]
    fn test_source_weight_priority() {
        let ranker = SuggestionRanker::new();

        let suggestions = vec![
            create_suggestion("cmd1", 0.7, SuggestionSource::Context),
            create_suggestion("cmd2", 0.7, SuggestionSource::History),
            create_suggestion("cmd3", 0.7, SuggestionSource::Llm),
        ];

        let ranked = ranker.rank(suggestions);

        // 分数相同时，Context 应该排在前面
        assert_eq!(ranked[0].source, SuggestionSource::Context);
    }
}
