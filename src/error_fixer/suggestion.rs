//! 多维度建议系统 - Phase 2 (v1.16.0)
//!
//! 提供基于多个维度的智能建议排序：
//! - 相关性（Relevance）: 与当前错误的相关度
//! - 可行性（Feasibility）: 成功执行的概率
//! - 安全性（Safety）: 操作的风险评估
//! - 学习价值（Learning Value）: 是否帮助用户学习

use serde::{Deserialize, Serialize};

/// 多维度建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// 建议的命令
    pub command: String,

    /// 建议描述
    pub description: String,

    /// 相关性评分 (0.0-1.0)
    /// 与当前错误的相关程度
    pub relevance: f64,

    /// 可行性评分 (0.0-1.0)
    /// 成功执行的概率
    pub feasibility: f64,

    /// 安全性评分 (0.0-1.0)
    /// 1.0 = 完全安全, 0.0 = 高风险
    pub safety: f64,

    /// 学习价值评分 (0.0-1.0)
    /// 是否帮助用户理解和学习
    pub learning_value: f64,

    /// 是否为推荐选项
    pub recommended: bool,
}

impl Suggestion {
    /// 创建新建议
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            description: description.into(),
            relevance: 0.5,
            feasibility: 0.5,
            safety: 0.5,
            learning_value: 0.5,
            recommended: false,
        }
    }

    /// 设置相关性
    pub fn with_relevance(mut self, relevance: f64) -> Self {
        self.relevance = relevance.clamp(0.0, 1.0);
        self
    }

    /// 设置可行性
    pub fn with_feasibility(mut self, feasibility: f64) -> Self {
        self.feasibility = feasibility.clamp(0.0, 1.0);
        self
    }

    /// 设置安全性
    pub fn with_safety(mut self, safety: f64) -> Self {
        self.safety = safety.clamp(0.0, 1.0);
        self
    }

    /// 设置学习价值
    pub fn with_learning_value(mut self, learning_value: f64) -> Self {
        self.learning_value = learning_value.clamp(0.0, 1.0);
        self
    }

    /// 标记为推荐
    pub fn mark_recommended(mut self) -> Self {
        self.recommended = true;
        self
    }

    /// 计算综合评分
    ///
    /// 权重分配：
    /// - 相关性: 40% (最重要)
    /// - 可行性: 30%
    /// - 安全性: 20%
    /// - 学习价值: 10%
    pub fn score(&self) -> f64 {
        self.relevance * 0.4
            + self.feasibility * 0.3
            + self.safety * 0.2
            + self.learning_value * 0.1
    }

    /// 获取评分等级
    pub fn score_level(&self) -> ScoreLevel {
        let score = self.score();
        match score {
            s if s >= 0.8 => ScoreLevel::Excellent,
            s if s >= 0.6 => ScoreLevel::Good,
            s if s >= 0.4 => ScoreLevel::Fair,
            _ => ScoreLevel::Poor,
        }
    }
}

/// 评分等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreLevel {
    /// 优秀 (>= 0.8)
    Excellent,
    /// 良好 (>= 0.6)
    Good,
    /// 一般 (>= 0.4)
    Fair,
    /// 较差 (< 0.4)
    Poor,
}

impl ScoreLevel {
    /// 获取显示符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Excellent => "⭐⭐⭐",
            Self::Good => "⭐⭐",
            Self::Fair => "⭐",
            Self::Poor => "·",
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Excellent => "强烈推荐",
            Self::Good => "推荐",
            Self::Fair => "可选",
            Self::Poor => "不推荐",
        }
    }
}

/// 建议列表
#[derive(Debug, Clone, Default)]
pub struct SuggestionList {
    suggestions: Vec<Suggestion>,
}

impl SuggestionList {
    /// 创建新的建议列表
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
        }
    }

    /// 添加建议
    pub fn add(&mut self, suggestion: Suggestion) {
        self.suggestions.push(suggestion);
    }

    /// 批量添加建议
    pub fn extend(&mut self, suggestions: impl IntoIterator<Item = Suggestion>) {
        self.suggestions.extend(suggestions);
    }

    /// 按综合评分排序
    pub fn sort_by_score(&mut self) {
        self.suggestions
            .sort_by(|a, b| b.score().partial_cmp(&a.score()).unwrap());
    }

    /// 按特定维度排序
    pub fn sort_by_dimension(&mut self, dimension: SortDimension) {
        self.suggestions.sort_by(|a, b| {
            let score_a = match dimension {
                SortDimension::Relevance => a.relevance,
                SortDimension::Feasibility => a.feasibility,
                SortDimension::Safety => a.safety,
                SortDimension::LearningValue => a.learning_value,
                SortDimension::Overall => a.score(),
            };
            let score_b = match dimension {
                SortDimension::Relevance => b.relevance,
                SortDimension::Feasibility => b.feasibility,
                SortDimension::Safety => b.safety,
                SortDimension::LearningValue => b.learning_value,
                SortDimension::Overall => b.score(),
            };
            score_b.partial_cmp(&score_a).unwrap()
        });
    }

    /// 获取前N个建议
    pub fn top_n(&self, n: usize) -> Vec<&Suggestion> {
        self.suggestions.iter().take(n).collect()
    }

    /// 获取所有建议
    pub fn all(&self) -> &[Suggestion] {
        &self.suggestions
    }

    /// 建议数量
    pub fn len(&self) -> usize {
        self.suggestions.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }

    /// 标记最佳建议为推荐
    pub fn mark_best_as_recommended(&mut self) {
        if let Some(best) = self.suggestions.first_mut() {
            best.recommended = true;
        }
    }
}

/// 排序维度
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDimension {
    /// 相关性
    Relevance,
    /// 可行性
    Feasibility,
    /// 安全性
    Safety,
    /// 学习价值
    LearningValue,
    /// 综合评分
    Overall,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let suggestion = Suggestion::new("cargo build", "Build the project")
            .with_relevance(0.9)
            .with_feasibility(0.8)
            .with_safety(1.0)
            .with_learning_value(0.6);

        assert_eq!(suggestion.command, "cargo build");
        assert_eq!(suggestion.relevance, 0.9);
        assert_eq!(suggestion.feasibility, 0.8);
        assert_eq!(suggestion.safety, 1.0);
        assert_eq!(suggestion.learning_value, 0.6);
    }

    #[test]
    fn test_suggestion_score() {
        let suggestion = Suggestion::new("test", "test")
            .with_relevance(0.9)       // 40% * 0.9 = 0.36
            .with_feasibility(0.8)     // 30% * 0.8 = 0.24
            .with_safety(1.0)          // 20% * 1.0 = 0.20
            .with_learning_value(0.5); // 10% * 0.5 = 0.05
                                        // Total = 0.85

        let score = suggestion.score();
        assert!((score - 0.85).abs() < 0.01);
    }

    #[test]
    fn test_score_level() {
        let excellent = Suggestion::new("test", "test")
            .with_relevance(1.0)
            .with_feasibility(1.0)
            .with_safety(1.0)
            .with_learning_value(1.0);
        assert_eq!(excellent.score_level(), ScoreLevel::Excellent);

        let good = Suggestion::new("test", "test")
            .with_relevance(0.7)
            .with_feasibility(0.7)
            .with_safety(0.7)
            .with_learning_value(0.7);
        assert_eq!(good.score_level(), ScoreLevel::Good);

        let fair = Suggestion::new("test", "test")
            .with_relevance(0.5)
            .with_feasibility(0.5)
            .with_safety(0.5)
            .with_learning_value(0.5);
        assert_eq!(fair.score_level(), ScoreLevel::Fair);

        let poor = Suggestion::new("test", "test")
            .with_relevance(0.2)
            .with_feasibility(0.2)
            .with_safety(0.2)
            .with_learning_value(0.2);
        assert_eq!(poor.score_level(), ScoreLevel::Poor);
    }

    #[test]
    fn test_suggestion_list_sorting() {
        let mut list = SuggestionList::new();

        list.add(
            Suggestion::new("cmd1", "desc1")
                .with_relevance(0.5)
                .with_feasibility(0.5)
                .with_safety(0.5)
                .with_learning_value(0.5),
        );

        list.add(
            Suggestion::new("cmd2", "desc2")
                .with_relevance(0.9)
                .with_feasibility(0.9)
                .with_safety(0.9)
                .with_learning_value(0.9),
        );

        list.add(
            Suggestion::new("cmd3", "desc3")
                .with_relevance(0.7)
                .with_feasibility(0.7)
                .with_safety(0.7)
                .with_learning_value(0.7),
        );

        list.sort_by_score();

        let top = list.top_n(1);
        assert_eq!(top[0].command, "cmd2");
    }

    #[test]
    fn test_score_clamping() {
        let suggestion = Suggestion::new("test", "test")
            .with_relevance(1.5) // 应该被限制在1.0
            .with_feasibility(-0.5) // 应该被限制在0.0
            .with_safety(0.5)
            .with_learning_value(2.0); // 应该被限制在1.0

        assert_eq!(suggestion.relevance, 1.0);
        assert_eq!(suggestion.feasibility, 0.0);
        assert_eq!(suggestion.learning_value, 1.0);
    }
}
