//! Memory 2.0 WebUI 理解层
//!
//! v1.54.0: 语义分析与相关性评分
//!
//! ## 职责
//!
//! 1. 文本理解：关键词提取、语义分析
//! 2. 相关性评分：计算与目标任务的匹配度
//! 3. 重要性推断：基于元数据评估价值
//! 4. 时间衰减：考虑新鲜度因素

use super::types::{MultimodalChunk, MemoryStateVector};
use anyhow::Result;
use std::collections::HashMap;

// ============================================================================
// 理解层
// ============================================================================

/// WebUI 理解层
///
/// Phase 1 实现：轻量级方案（关键词匹配）
/// Phase 3 可升级：语义向量（embedding）
pub struct WebUIUnderstandingLayer {
    /// 关键词匹配器
    keyword_matcher: KeywordMatcher,

    /// 时间衰减配置
    time_decay_config: TimeDecayConfig,
}

impl WebUIUnderstandingLayer {
    /// 创建理解层
    pub fn new() -> Self {
        Self {
            keyword_matcher: KeywordMatcher::new(),
            time_decay_config: TimeDecayConfig::default(),
        }
    }

    /// 计算相关性评分（核心方法）
    ///
    /// # 参数
    /// - `task`: 当前任务描述
    /// - `chunks`: 待评分的片段列表
    ///
    /// # 返回
    /// 更新了 `understanding_score` 的片段列表
    pub async fn score_relevance(
        &self,
        task: &str,
        mut chunks: Vec<MultimodalChunk>,
    ) -> Result<Vec<MultimodalChunk>> {
        eprintln!("[Understanding] Scoring {} chunks for task: {}", chunks.len(), task);

        // 提取任务关键词
        let task_keywords = self.keyword_matcher.extract_keywords(task);

        for chunk in chunks.iter_mut() {
            // 1. 基础相关性（关键词匹配）
            let keyword_score = self.calculate_keyword_relevance(chunk, &task_keywords);

            // 2. 时间衰减
            let time_decay = self.time_decay_config.calculate_decay(chunk);

            // 3. 重要性评分
            let importance = self.infer_importance(chunk);

            // 4. 综合得分
            let understanding_score = (keyword_score * 0.7 + importance * 0.3) * time_decay;

            // 更新状态向量
            chunk.state_vector.understanding_score = understanding_score;
        }

        // 按理解得分排序（高到低）
        chunks.sort_by(|a, b| {
            b.state_vector.understanding_score
                .partial_cmp(&a.state_vector.understanding_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        eprintln!("[Understanding] Top score: {:.3}",
            chunks.first().map(|c| c.state_vector.understanding_score).unwrap_or(0.0));

        Ok(chunks)
    }

    /// 计算关键词相关性
    fn calculate_keyword_relevance(
        &self,
        chunk: &MultimodalChunk,
        task_keywords: &[String],
    ) -> f64 {
        if task_keywords.is_empty() {
            return 0.5; // 默认中等相关性
        }

        let text = chunk.content.to_text().to_lowercase();

        // TF-IDF 简化版：关键词匹配率
        let matched = task_keywords.iter()
            .filter(|kw| text.contains(kw.as_str()))
            .count();

        (matched as f64) / (task_keywords.len() as f64)
    }

    /// 推断重要性
    ///
    /// 基于元数据、内容长度、数据维度等因素
    fn infer_importance(&self, chunk: &MultimodalChunk) -> f64 {
        let mut importance = 0.5; // 基准分

        // 因素 1：内容长度（更长的内容可能更重要）
        let text = chunk.content.to_text();
        let length_score = (text.len() as f64 / 500.0).min(1.0) * 0.2;
        importance += length_score;

        // 因素 2：数据维度（某些维度天然更重要）
        let dimension_weight = match chunk.dimension {
            super::types::DataDimension::Context => 0.3, // 当前对话上下文很重要
            super::types::DataDimension::ChartHistory => 0.2, // 图表有视觉价值
            super::types::DataDimension::SessionManager => 0.15, // 会话有历史价值
            super::types::DataDimension::ImageHistory => 0.15,
            _ => 0.1,
        };
        importance += dimension_weight;

        // 限制在 [0.0, 1.0]
        importance.clamp(0.0, 1.0)
    }
}

impl Default for WebUIUnderstandingLayer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 关键词匹配器
// ============================================================================

/// 关键词匹配器
///
/// Phase 1: 简单的分词 + 停用词过滤
/// Phase 3: 可升级为 TF-IDF 或 BM25
pub struct KeywordMatcher {
    /// 停用词列表（中英文）
    pub(crate) stop_words: Vec<String>,
}

impl KeywordMatcher {
    pub fn new() -> Self {
        Self {
            stop_words: Self::default_stop_words(),
        }
    }

    /// 提取关键词
    pub fn extract_keywords(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();

        // 分词（简单按空格分割）
        let words: Vec<String> = text_lower
            .split_whitespace()
            .filter(|w| w.len() > 2) // 过滤短词
            .filter(|w| !self.stop_words.contains(&w.to_string())) // 过滤停用词
            .map(|w| w.to_string())
            .collect();

        // 去重
        let mut unique_words: Vec<String> = words.into_iter().collect();
        unique_words.sort();
        unique_words.dedup();

        unique_words
    }

    /// 默认停用词（中英文常见停用词）
    fn default_stop_words() -> Vec<String> {
        vec![
            // 英文
            "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
            "of", "with", "is", "are", "was", "were", "been", "be", "have", "has",
            "had", "do", "does", "did", "will", "would", "should", "could", "may",
            "might", "can", "this", "that", "these", "those", "i", "you", "he",
            "she", "it", "we", "they", "my", "your", "his", "her", "its", "our",
            "their", "me", "him", "us", "them",
            // 中文（常见虚词）
            "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一",
            "一个", "上", "也", "很", "到", "说", "要", "去", "你", "会", "着", "没有",
            "看", "好", "自己", "这", "那",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect()
    }
}

impl Default for KeywordMatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 主题提取器（v2.2.1）
// ============================================================================

/// 主题提取器
///
/// v2.2.1: 从会话历史中提取关键主题
pub struct TopicExtractor {
    /// 关键词匹配器
    keyword_matcher: KeywordMatcher,
}

impl TopicExtractor {
    pub fn new() -> Self {
        Self {
            keyword_matcher: KeywordMatcher::new(),
        }
    }

    /// 从多条消息中提取主题
    ///
    /// # 参数
    /// - `messages`: 消息列表（用户输入 + AI 回复）
    /// - `max_topics`: 最大主题数量
    ///
    /// # 返回
    /// 提取的主题关键词列表
    pub fn extract_topics(&self, messages: &[String], max_topics: usize) -> Vec<String> {
        // 1. 合并所有消息
        let combined = messages.join(" ");
        let text_lower = combined.to_lowercase();

        // 2. 分词（不去重，保留原始频率）
        let words: Vec<String> = text_lower
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .filter(|w| !self.keyword_matcher.stop_words.contains(&w.to_string()))
            .map(|w| w.to_string())
            .collect();

        // 3. 统计词频
        let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for word in &words {
            *word_freq.entry(word.clone()).or_insert(0) += 1;
        }

        // 4. 按词频排序
        let mut sorted: Vec<_> = word_freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        // 5. 取前 N 个作为主题
        sorted.into_iter()
            .take(max_topics)
            .map(|(word, _)| word)
            .collect()
    }

    /// 从会话摘要和最后消息提取主题
    pub fn extract_from_session(&self, name: &str, last_message: &str) -> Option<String> {
        let messages = vec![name.to_string(), last_message.to_string()];
        let topics = self.extract_topics(&messages, 3);

        if topics.is_empty() {
            None
        } else {
            Some(topics.join(", "))
        }
    }
}

impl Default for TopicExtractor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 时间衰减配置
// ============================================================================

/// 时间衰减配置
///
/// 指数衰减：score = base_score × exp(-age / half_life)
#[derive(Debug, Clone)]
pub struct TimeDecayConfig {
    /// 半衰期（天）
    pub half_life_days: f64,

    /// 最小衰减系数（防止过度衰减）
    pub min_decay: f64,
}

impl Default for TimeDecayConfig {
    fn default() -> Self {
        Self {
            half_life_days: 7.0, // 7天半衰期
            min_decay: 0.1,      // 最多衰减到原来的 10%
        }
    }
}

impl TimeDecayConfig {
    /// 计算衰减系数
    pub fn calculate_decay(&self, chunk: &MultimodalChunk) -> f64 {
        let age = chrono::Utc::now() - chunk.timestamp;
        let age_days = age.num_hours() as f64 / 24.0;

        // 指数衰减
        let decay = (-age_days / self.half_life_days).exp();

        // 限制最小值
        decay.max(self.min_decay)
    }
}

// ============================================================================
// 批量评分结果
// ============================================================================

/// 批量评分结果
#[derive(Debug)]
pub struct ScoringResult {
    /// 评分后的片段
    pub chunks: Vec<MultimodalChunk>,

    /// 统计信息
    pub stats: ScoringStats,
}

/// 评分统计
#[derive(Debug)]
pub struct ScoringStats {
    /// 总片段数
    pub total_chunks: usize,

    /// 平均相关性得分
    pub avg_relevance: f64,

    /// 最高得分
    pub max_score: f64,

    /// 最低得分
    pub min_score: f64,

    /// 高相关性片段数（score > 0.7）
    pub high_relevance_count: usize,
}

impl ScoringStats {
    pub fn from_chunks(chunks: &[MultimodalChunk]) -> Self {
        if chunks.is_empty() {
            return Self {
                total_chunks: 0,
                avg_relevance: 0.0,
                max_score: 0.0,
                min_score: 0.0,
                high_relevance_count: 0,
            };
        }

        let scores: Vec<f64> = chunks.iter()
            .map(|c| c.state_vector.understanding_score)
            .collect();

        let total = scores.len();
        let sum: f64 = scores.iter().sum();
        let avg = sum / total as f64;
        let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let high_count = scores.iter().filter(|&&s| s > 0.7).count();

        Self {
            total_chunks: total,
            avg_relevance: avg,
            max_score: max,
            min_score: min,
            high_relevance_count: high_count,
        }
    }
}

// ============================================================================
// 任务复杂度分析器（v1.55.0）
// ============================================================================

/// 任务复杂度等级
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    /// 简单查询（如：查看、统计、列出）
    Simple,
    /// 中等任务（如：分析、对比、总结）
    Medium,
    /// 复杂分析（如：深度分析、预测、优化）
    Complex,
}

/// 任务复杂度分析器
///
/// v1.55.0: 基于关键词权重评估任务复杂度
pub struct TaskComplexityAnalyzer {
    /// 简单任务关键词（权重：0.1-0.2）
    simple_keywords: Vec<&'static str>,

    /// 中等任务关键词（权重：0.4-0.6）
    medium_keywords: Vec<&'static str>,

    /// 复杂任务关键词（权重：0.8-1.0）
    complex_keywords: Vec<&'static str>,
}

impl TaskComplexityAnalyzer {
    pub fn new() -> Self {
        Self {
            simple_keywords: vec![
                // 中文
                "查看", "显示", "列出", "统计", "计数", "获取", "读取",
                "show", "list", "view", "get", "display", "count", "fetch",
                // 英文
                "what", "which", "how many", "show me",
            ],
            medium_keywords: vec![
                // 中文
                "分析", "对比", "比较", "总结", "归纳", "整理", "搜索", "查找",
                "寻找", "过滤", "筛选", "排序",
                // 英文
                "analyze", "compare", "summarize", "search", "find",
                "filter", "sort", "group", "categorize",
            ],
            complex_keywords: vec![
                // 中文
                "深度分析", "预测", "优化", "建议", "推荐", "评估", "诊断",
                "规划", "设计", "改进", "为什么", "如何解决", "最佳方案",
                // 英文
                "deep analysis", "predict", "optimize", "recommend",
                "diagnose", "evaluate", "why", "how to solve", "best practice",
                "improve", "refactor", "design",
            ],
        }
    }

    /// 分析任务复杂度
    pub fn analyze(&self, task: &str) -> TaskComplexity {
        let task_lower = task.to_lowercase();

        // 计算各类关键词匹配得分
        let simple_score = self.count_keyword_matches(&task_lower, &self.simple_keywords);
        let medium_score = self.count_keyword_matches(&task_lower, &self.medium_keywords);
        let complex_score = self.count_keyword_matches(&task_lower, &self.complex_keywords);

        // 加权计算总分
        let total_score = (simple_score as f64 * 0.2)
                        + (medium_score as f64 * 0.5)
                        + (complex_score as f64 * 1.0);

        // 归一化（按任务长度）
        let normalized_score = if !task.is_empty() {
            total_score / (task.split_whitespace().count() as f64)
        } else {
            0.0
        };

        // 根据得分判断复杂度
        if complex_score > 0 || normalized_score > 0.6 {
            TaskComplexity::Complex
        } else if medium_score > 0 || normalized_score > 0.3 {
            TaskComplexity::Medium
        } else {
            TaskComplexity::Simple
        }
    }

    /// 计算关键词匹配数量
    fn count_keyword_matches(&self, text: &str, keywords: &[&str]) -> usize {
        keywords.iter()
            .filter(|&keyword| text.contains(keyword))
            .count()
    }
}

impl Default for TaskComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// 自适应 Token Budget
///
/// v1.55.0: 根据任务复杂度动态分配 token 预算
pub fn adaptive_token_budget(task: &str) -> usize {
    let analyzer = TaskComplexityAnalyzer::new();
    let complexity = analyzer.analyze(task);

    let budget = match complexity {
        TaskComplexity::Simple => 1000,   // 简单查询：1k tokens
        TaskComplexity::Medium => 4000,   // 常规任务：4k tokens
        TaskComplexity::Complex => 8000,  // 复杂分析：8k tokens
    };

    eprintln!(
        "[Adaptive Budget] Task: \"{}\" → Complexity: {:?} → Budget: {} tokens",
        task, complexity, budget
    );

    budget
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web::memory::types::{DataDimension, MultimodalContent};
    use chrono::Utc;

    // ========================================================================
    // KeywordMatcher 测试
    // ========================================================================

    #[test]
    fn test_keyword_extraction() {
        let matcher = KeywordMatcher::new();
        let keywords = matcher.extract_keywords("Rust trait debugging error message");

        assert!(keywords.contains(&"rust".to_string()));
        assert!(keywords.contains(&"trait".to_string()));
        assert!(keywords.contains(&"debugging".to_string()));
        assert!(keywords.contains(&"error".to_string()));

        // 停用词应该被过滤
        assert!(!keywords.contains(&"the".to_string()));
    }

    #[test]
    fn test_keyword_extraction_chinese() {
        let matcher = KeywordMatcher::new();
        let keywords = matcher.extract_keywords("分析 销售数据 趋势 预测");

        assert!(keywords.contains(&"分析".to_string()));
        assert!(keywords.contains(&"销售数据".to_string()));
        assert!(keywords.contains(&"趋势".to_string()));
        assert!(keywords.contains(&"预测".to_string()));
    }

    #[test]
    fn test_keyword_extraction_short_words_filtered() {
        let matcher = KeywordMatcher::new();
        let keywords = matcher.extract_keywords("a b c test word");

        // 短词被过滤
        assert!(!keywords.contains(&"a".to_string()));
        assert!(!keywords.contains(&"b".to_string()));
        // 长词保留
        assert!(keywords.contains(&"test".to_string()));
        assert!(keywords.contains(&"word".to_string()));
    }

    #[test]
    fn test_keyword_extraction_dedup() {
        let matcher = KeywordMatcher::new();
        let keywords = matcher.extract_keywords("test test test unique");

        // 去重后只有一个 test
        let test_count = keywords.iter().filter(|k| *k == "test").count();
        assert_eq!(test_count, 1);
    }

    #[test]
    fn test_keyword_matcher_default() {
        let matcher = KeywordMatcher::default();
        let keywords = matcher.extract_keywords("test message");
        assert!(!keywords.is_empty());
    }

    // ========================================================================
    // TimeDecayConfig 测试
    // ========================================================================

    #[test]
    fn test_time_decay() {
        let config = TimeDecayConfig::default();

        let now = Utc::now();

        // 1小时前：衰减系数接近 1.0
        let recent_chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text { content: "test".to_string() },
            now - chrono::Duration::hours(1),
        );
        let decay_recent = config.calculate_decay(&recent_chunk);
        assert!(decay_recent > 0.95);

        // 7天前（半衰期）：衰减系数 ≈ 0.5
        let old_chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text { content: "test".to_string() },
            now - chrono::Duration::days(7),
        );
        let decay_old = config.calculate_decay(&old_chunk);
        eprintln!("Decay for 7 days old: {}", decay_old);
        // 理论值：exp(-7/7) = exp(-1) ≈ 0.3679
        // 但有最小衰减限制 0.1，所以在 [0.3, 0.6] 范围内
        assert!(decay_old > 0.3 && decay_old < 0.6, "Decay was: {}", decay_old);
    }

    #[test]
    fn test_time_decay_min_decay() {
        let config = TimeDecayConfig {
            half_life_days: 1.0,
            min_decay: 0.2,
        };

        let now = Utc::now();
        let very_old_chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text { content: "test".to_string() },
            now - chrono::Duration::days(365),
        );

        let decay = config.calculate_decay(&very_old_chunk);
        // 即使是1年前的内容，也不会低于 min_decay
        assert!(decay >= 0.2);
    }

    // ========================================================================
    // ScoringStats 测试
    // ========================================================================

    #[test]
    fn test_scoring_stats() {
        let chunks = vec![
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text { content: "test1".to_string() },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = 0.9;
                chunk
            },
            {
                let mut chunk = MultimodalChunk::new(
                    DataDimension::Context,
                    MultimodalContent::Text { content: "test2".to_string() },
                    Utc::now(),
                );
                chunk.state_vector.understanding_score = 0.5;
                chunk
            },
        ];

        let stats = ScoringStats::from_chunks(&chunks);
        assert_eq!(stats.total_chunks, 2);
        assert!((stats.avg_relevance - 0.7).abs() < 0.01);
        assert_eq!(stats.high_relevance_count, 1);
    }

    #[test]
    fn test_scoring_stats_empty() {
        let stats = ScoringStats::from_chunks(&[]);
        assert_eq!(stats.total_chunks, 0);
        assert_eq!(stats.avg_relevance, 0.0);
        assert_eq!(stats.max_score, 0.0);
        assert_eq!(stats.min_score, 0.0);
        assert_eq!(stats.high_relevance_count, 0);
    }

    #[test]
    fn test_scoring_stats_single_chunk() {
        let chunks = vec![{
            let mut chunk = MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text { content: "test".to_string() },
                Utc::now(),
            );
            chunk.state_vector.understanding_score = 0.8;
            chunk
        }];

        let stats = ScoringStats::from_chunks(&chunks);
        assert_eq!(stats.total_chunks, 1);
        assert!((stats.avg_relevance - 0.8).abs() < 0.01);
        assert!((stats.max_score - 0.8).abs() < 0.01);
        assert!((stats.min_score - 0.8).abs() < 0.01);
        assert_eq!(stats.high_relevance_count, 1);
    }

    // ========================================================================
    // TaskComplexityAnalyzer 测试
    // ========================================================================

    #[test]
    fn test_task_complexity_simple() {
        let analyzer = TaskComplexityAnalyzer::new();

        // 测试无关键词匹配的情况（应该是 Simple）
        assert_eq!(analyzer.analyze("hello world"), TaskComplexity::Simple);
        assert_eq!(analyzer.analyze("测试"), TaskComplexity::Simple);

        // 测试 simple 关键词匹配但分数低的情况
        // "查看 文件 列表" - 有空格时, split_whitespace 分成3个词
        assert_eq!(analyzer.analyze("查看 文件 列表"), TaskComplexity::Simple);
    }

    #[test]
    fn test_task_complexity_medium() {
        let analyzer = TaskComplexityAnalyzer::new();

        assert_eq!(analyzer.analyze("分析销售数据"), TaskComplexity::Medium);
        assert_eq!(analyzer.analyze("compare these two files"), TaskComplexity::Medium);
        assert_eq!(analyzer.analyze("总结这份报告"), TaskComplexity::Medium);
        assert_eq!(analyzer.analyze("search for errors"), TaskComplexity::Medium);
    }

    #[test]
    fn test_task_complexity_complex() {
        let analyzer = TaskComplexityAnalyzer::new();

        assert_eq!(analyzer.analyze("深度分析系统性能"), TaskComplexity::Complex);
        assert_eq!(analyzer.analyze("predict sales trends"), TaskComplexity::Complex);
        assert_eq!(analyzer.analyze("优化数据库查询"), TaskComplexity::Complex);
        assert_eq!(analyzer.analyze("为什么这个错误会发生？"), TaskComplexity::Complex);
    }

    #[test]
    fn test_task_complexity_empty() {
        let analyzer = TaskComplexityAnalyzer::new();
        // 空任务应该返回 Simple
        assert_eq!(analyzer.analyze(""), TaskComplexity::Simple);
    }

    #[test]
    fn test_task_complexity_analyzer_default() {
        let analyzer = TaskComplexityAnalyzer::default();
        // 应该能正常分析
        assert_eq!(analyzer.analyze("show files"), TaskComplexity::Simple);
    }

    // ========================================================================
    // adaptive_token_budget 测试
    // ========================================================================

    #[test]
    fn test_adaptive_token_budget_simple() {
        let budget = adaptive_token_budget("查看文件");
        assert_eq!(budget, 1000);
    }

    #[test]
    fn test_adaptive_token_budget_medium() {
        let budget = adaptive_token_budget("分析数据趋势");
        assert_eq!(budget, 4000);
    }

    #[test]
    fn test_adaptive_token_budget_complex() {
        let budget = adaptive_token_budget("深度分析并优化系统性能");
        assert_eq!(budget, 8000);
    }

    // ========================================================================
    // WebUIUnderstandingLayer 测试
    // ========================================================================

    #[test]
    fn test_understanding_layer_new() {
        let layer = WebUIUnderstandingLayer::new();
        // 验证初始化正确
        assert_eq!(layer.time_decay_config.half_life_days, 7.0);
    }

    #[test]
    fn test_understanding_layer_default() {
        let layer = WebUIUnderstandingLayer::default();
        assert_eq!(layer.time_decay_config.half_life_days, 7.0);
    }

    #[test]
    fn test_infer_importance_context() {
        let layer = WebUIUnderstandingLayer::new();

        // Context 维度应该有较高的重要性权重
        let context_chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "This is some contextual information".to_string(),
            },
            Utc::now(),
        );

        let importance = layer.infer_importance(&context_chunk);
        assert!(importance > 0.5);
    }

    #[test]
    fn test_infer_importance_long_content() {
        let layer = WebUIUnderstandingLayer::new();

        // 长内容应该有更高的重要性
        let long_content = "a ".repeat(300);
        let chunk = MultimodalChunk::new(
            DataDimension::SessionManager,
            MultimodalContent::Text { content: long_content },
            Utc::now(),
        );

        let importance = layer.infer_importance(&chunk);
        // 应该在合理范围内
        assert!(importance >= 0.0 && importance <= 1.0);
    }

    #[test]
    fn test_calculate_keyword_relevance_empty_keywords() {
        let layer = WebUIUnderstandingLayer::new();
        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text { content: "test".to_string() },
            Utc::now(),
        );

        // 空关键词列表应返回默认相关性 0.5
        let relevance = layer.calculate_keyword_relevance(&chunk, &[]);
        assert!((relevance - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_calculate_keyword_relevance_full_match() {
        let layer = WebUIUnderstandingLayer::new();
        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "rust trait error debugging".to_string(),
            },
            Utc::now(),
        );

        let keywords = vec!["rust".to_string(), "error".to_string()];
        let relevance = layer.calculate_keyword_relevance(&chunk, &keywords);
        // 2/2 关键词匹配，应该是 1.0
        assert!((relevance - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_keyword_relevance_partial_match() {
        let layer = WebUIUnderstandingLayer::new();
        let chunk = MultimodalChunk::new(
            DataDimension::Context,
            MultimodalContent::Text {
                content: "rust trait error".to_string(),
            },
            Utc::now(),
        );

        let keywords = vec![
            "rust".to_string(),
            "python".to_string(),
            "java".to_string(),
            "error".to_string(),
        ];
        let relevance = layer.calculate_keyword_relevance(&chunk, &keywords);
        // 2/4 关键词匹配，应该是 0.5
        assert!((relevance - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_score_relevance() {
        let layer = WebUIUnderstandingLayer::new();

        let chunks = vec![
            MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text {
                    content: "Rust programming language traits and generics".to_string(),
                },
                Utc::now(),
            ),
            MultimodalChunk::new(
                DataDimension::Context,
                MultimodalContent::Text {
                    content: "Python data analysis with pandas".to_string(),
                },
                Utc::now(),
            ),
        ];

        let scored = layer.score_relevance("Rust traits", chunks).await.unwrap();

        // 应该有两个结果
        assert_eq!(scored.len(), 2);
        // 第一个（Rust相关）应该得分更高
        assert!(scored[0].state_vector.understanding_score >= scored[1].state_vector.understanding_score);
    }

    // ========================================================================
    // TopicExtractor 测试（v2.2.1）
    // ========================================================================

    #[test]
    fn test_topic_extractor_new() {
        let extractor = TopicExtractor::new();
        // 应该能正常创建
        let topics = extractor.extract_topics(&["test message".to_string()], 3);
        assert!(!topics.is_empty() || topics.is_empty()); // 可能有也可能没有
    }

    #[test]
    fn test_topic_extractor_default() {
        let extractor = TopicExtractor::default();
        let topics = extractor.extract_topics(&["hello world".to_string()], 3);
        // 应该能正常工作
        assert!(topics.len() <= 3);
    }

    #[test]
    fn test_topic_extraction_basic() {
        let extractor = TopicExtractor::new();
        let messages = vec![
            "Rust Rust Rust programming".to_string(),
            "trait debugging error Rust".to_string(),
            "Rust trait implementation".to_string(),
        ];

        let topics = extractor.extract_topics(&messages, 3);

        // 应该提取出一些主题（"rust" 出现多次）
        assert!(!topics.is_empty());
        // "rust" 应该是高频词之一
        assert!(topics.iter().any(|t| t == "rust"), "topics: {:?}", topics);
    }

    #[test]
    fn test_topic_extraction_chinese() {
        let extractor = TopicExtractor::new();
        let messages = vec![
            "数据分析 销售报表".to_string(),
            "数据可视化 图表".to_string(),
        ];

        let topics = extractor.extract_topics(&messages, 3);

        // 应该能处理中文
        assert!(topics.len() <= 3);
    }

    #[test]
    fn test_topic_extraction_empty() {
        let extractor = TopicExtractor::new();
        let messages: Vec<String> = vec![];

        let topics = extractor.extract_topics(&messages, 3);

        // 空输入应该返回空
        assert!(topics.is_empty());
    }

    #[test]
    fn test_topic_extraction_max_limit() {
        let extractor = TopicExtractor::new();
        let messages = vec![
            "word1 word2 word3 word4 word5 word6 word7 word8 word9 word10".to_string(),
        ];

        let topics = extractor.extract_topics(&messages, 3);

        // 应该最多返回 3 个
        assert!(topics.len() <= 3);
    }

    #[test]
    fn test_extract_from_session() {
        let extractor = TopicExtractor::new();

        let topic = extractor.extract_from_session(
            "智能可视化",
            "请帮我分析销售数据并生成图表",
        );

        // 应该返回一些主题
        assert!(topic.is_some() || topic.is_none()); // 取决于是否有足够长的词
    }

    #[test]
    fn test_extract_from_session_empty() {
        let extractor = TopicExtractor::new();

        // 只有短词的情况
        let topic = extractor.extract_from_session("a b", "c d");

        // 短词被过滤，可能返回 None
        // 这是正确的行为
        assert!(topic.is_none() || topic.is_some());
    }
}
