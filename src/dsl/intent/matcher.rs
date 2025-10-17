//! Intent 匹配引擎
//!
//! 负责将用户的自然语言输入匹配到预定义的意图。

use crate::dsl::intent::extractor::EntityExtractor;
use crate::dsl::intent::types::{Intent, IntentMatch};
use lru::LruCache;
use regex::Regex;
use std::cmp::min;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

/// 计算两个字符串之间的 Levenshtein 距离（编辑距离）
///
/// Levenshtein 距离是指将一个字符串转换成另一个字符串所需的
/// 最少单字符编辑操作次数（插入、删除或替换）。
///
/// # 参数
///
/// * `s1` - 第一个字符串
/// * `s2` - 第二个字符串
///
/// # 返回
///
/// 返回两个字符串之间的编辑距离
///
/// # 示例
///
/// ```
/// use simpleconsole::dsl::intent::matcher::levenshtein_distance;
///
/// assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
/// assert_eq!(levenshtein_distance("统计", "统计"), 0);
/// assert_eq!(levenshtein_distance("统计", "统记"), 1);
/// ```
///
/// # 算法复杂度
///
/// - 时间复杂度: O(m * n)，其中 m 和 n 是两个字符串的长度
/// - 空间复杂度: O(min(m, n))，使用了空间优化的实现
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    // 优化：如果其中一个字符串为空，距离就是另一个字符串的长度
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // 优化：始终让 s1 是较短的字符串，以减少空间使用
    if len1 > len2 {
        return levenshtein_distance(s2, s1);
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    // 使用两行数组进行空间优化（只需 O(min(m,n)) 空间）
    let mut prev_row: Vec<usize> = (0..=len1).collect();
    let mut curr_row: Vec<usize> = vec![0; len1 + 1];

    for i in 1..=len2 {
        curr_row[0] = i;

        for j in 1..=len1 {
            let cost = if s2_chars[i - 1] == s1_chars[j - 1] {
                0
            } else {
                1
            };

            curr_row[j] = min(
                min(
                    curr_row[j - 1] + 1,  // 插入
                    prev_row[j] + 1,      // 删除
                ),
                prev_row[j - 1] + cost, // 替换
            );
        }

        std::mem::swap(&mut prev_row, &mut curr_row);
    }

    prev_row[len1]
}

/// 计算两个字符串之间的相似度（0.0 到 1.0）
///
/// 基于 Levenshtein 距离计算相似度，距离越小相似度越高。
///
/// # 参数
///
/// * `s1` - 第一个字符串
/// * `s2` - 第二个字符串
///
/// # 返回
///
/// 返回 0.0 到 1.0 之间的相似度分数，1.0 表示完全相同
///
/// # 示例
///
/// ```
/// use simpleconsole::dsl::intent::matcher::string_similarity;
///
/// assert_eq!(string_similarity("统计", "统计"), 1.0);
/// assert!(string_similarity("统计", "统记") > 0.5);
/// assert!(string_similarity("hello", "world") < 0.5);
/// ```
pub fn string_similarity(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2);
    let max_len = s1.chars().count().max(s2.chars().count());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

/// 意图匹配器
///
/// IntentMatcher 负责识别用户输入中的意图，通过关键词匹配和正则模式匹配
/// 来计算置信度分数，并提取结构化实体信息。
///
/// # 示例
///
/// ```rust
/// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
///
/// let mut matcher = IntentMatcher::new();
///
/// // 注册意图
/// let intent = Intent::new(
///     "count_lines",
///     IntentDomain::FileOps,
///     vec!["统计".to_string(), "行数".to_string()],
///     vec![r"统计.*行数".to_string()],
///     0.5,
/// );
/// matcher.register(intent);
///
/// // 匹配用户输入（自动提取实体）
/// let matches = matcher.match_intent("统计 Python 代码行数");
/// assert!(!matches.is_empty());
/// ```
#[derive(Debug)]
pub struct IntentMatcher {
    /// 已注册的意图列表
    intents: Vec<Intent>,

    /// 正则表达式缓存（模式 -> 编译后的正则）
    regex_cache: HashMap<String, Regex>,

    /// 实体提取器 (Phase 3 Week 3)
    extractor: EntityExtractor,

    /// LRU 查询缓存 (Optimization)
    /// 缓存最近的查询结果以提升性能
    query_cache: Arc<RwLock<LruCache<String, Vec<IntentMatch>>>>,

    /// 缓存命中统计
    cache_hits: Arc<RwLock<usize>>,

    /// 缓存未命中统计
    cache_misses: Arc<RwLock<usize>>,

    /// 模糊匹配配置 (Fuzzy Matching)
    fuzzy_config: FuzzyConfig,
}

/// 模糊匹配配置
///
/// 控制关键词模糊匹配的行为。
#[derive(Debug, Clone)]
pub struct FuzzyConfig {
    /// 是否启用模糊匹配
    pub enabled: bool,

    /// 相似度阈值（0.0 到 1.0）
    /// 只有相似度 >= 该阈值的关键词才会被认为匹配
    /// 默认值：0.8
    pub similarity_threshold: f64,

    /// 模糊匹配的分数权重（相对于精确匹配）
    /// 精确匹配贡献 0.3 分，模糊匹配贡献 0.3 * fuzzy_weight 分
    /// 默认值：0.7（即模糊匹配贡献 0.21 分）
    pub fuzzy_weight: f64,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            similarity_threshold: 0.8,
            fuzzy_weight: 0.7,
        }
    }
}

impl FuzzyConfig {
    /// 创建一个启用模糊匹配的配置
    ///
    /// # 参数
    ///
    /// * `similarity_threshold` - 相似度阈值（0.0 到 1.0）
    /// * `fuzzy_weight` - 模糊匹配的分数权重（0.0 到 1.0）
    pub fn enabled(similarity_threshold: f64, fuzzy_weight: f64) -> Self {
        Self {
            enabled: true,
            similarity_threshold,
            fuzzy_weight,
        }
    }

    /// 创建一个禁用模糊匹配的配置
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }
}

impl IntentMatcher {
    /// 创建一个新的意图匹配器
    ///
    /// 默认缓存容量为 100 个查询结果。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::IntentMatcher;
    ///
    /// let matcher = IntentMatcher::new();
    /// ```
    pub fn new() -> Self {
        Self::with_cache_capacity(100)
    }

    /// 创建一个带有指定缓存容量的意图匹配器
    ///
    /// # 参数
    ///
    /// * `capacity` - LRU 缓存的容量（最多缓存多少个查询结果）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::IntentMatcher;
    ///
    /// // 创建一个缓存容量为 50 的匹配器
    /// let matcher = IntentMatcher::with_cache_capacity(50);
    /// ```
    pub fn with_cache_capacity(capacity: usize) -> Self {
        Self::with_config(capacity, FuzzyConfig::default())
    }

    /// 创建一个带有自定义配置的意图匹配器
    ///
    /// # 参数
    ///
    /// * `cache_capacity` - LRU 缓存的容量
    /// * `fuzzy_config` - 模糊匹配配置
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{IntentMatcher, FuzzyConfig};
    ///
    /// // 创建一个启用模糊匹配的匹配器
    /// let fuzzy_config = FuzzyConfig::enabled(0.8, 0.7);
    /// let matcher = IntentMatcher::with_config(100, fuzzy_config);
    /// ```
    pub fn with_config(cache_capacity: usize, fuzzy_config: FuzzyConfig) -> Self {
        let cache_capacity = NonZeroUsize::new(cache_capacity).unwrap_or(NonZeroUsize::new(100).unwrap());

        Self {
            intents: Vec::new(),
            regex_cache: HashMap::new(),
            extractor: EntityExtractor::new(),
            query_cache: Arc::new(RwLock::new(LruCache::new(cache_capacity))),
            cache_hits: Arc::new(RwLock::new(0)),
            cache_misses: Arc::new(RwLock::new(0)),
            fuzzy_config,
        }
    }

    /// 启用模糊匹配
    ///
    /// # 参数
    ///
    /// * `similarity_threshold` - 相似度阈值（0.0 到 1.0）
    /// * `fuzzy_weight` - 模糊匹配的分数权重（0.0 到 1.0）
    pub fn enable_fuzzy_matching(&mut self, similarity_threshold: f64, fuzzy_weight: f64) {
        self.fuzzy_config = FuzzyConfig::enabled(similarity_threshold, fuzzy_weight);
        // 清空缓存，因为模糊匹配会影响匹配结果
        self.clear_cache();
    }

    /// 禁用模糊匹配
    pub fn disable_fuzzy_matching(&mut self) {
        self.fuzzy_config = FuzzyConfig::disabled();
        // 清空缓存，因为禁用模糊匹配会影响匹配结果
        self.clear_cache();
    }

    /// 注册一个意图
    ///
    /// 注册时会预编译所有正则表达式模式以提高匹配性能。
    /// 注册新意图会清空查询缓存，因为新意图会影响匹配结果。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "count_lines",
    ///     IntentDomain::FileOps,
    ///     vec!["统计".to_string()],
    ///     vec![r"统计.*行数".to_string()],
    ///     0.5,
    /// );
    ///
    /// matcher.register(intent);
    /// ```
    pub fn register(&mut self, intent: Intent) {
        // 预编译正则表达式
        for pattern in &intent.patterns {
            if !self.regex_cache.contains_key(pattern) {
                if let Ok(regex) = Regex::new(pattern) {
                    self.regex_cache.insert(pattern.clone(), regex);
                } else {
                    eprintln!("警告: 无效的正则表达式模式: {}", pattern);
                }
            }
        }

        self.intents.push(intent);

        // 清空查询缓存，因为新意图会影响匹配结果
        if let Ok(mut cache) = self.query_cache.write() {
            cache.clear();
        }
    }

    /// 匹配用户输入到意图
    ///
    /// 返回所有满足置信度阈值的意图匹配结果，按置信度降序排列。
    /// 使用 LRU 缓存提升重复查询的性能。
    ///
    /// # 匹配算法
    ///
    /// 1. **关键词匹配** - 每个匹配的关键词贡献 0.3 分
    /// 2. **正则模式匹配** - 每个匹配的模式贡献 0.7 分
    /// 3. **置信度归一化** - 总分不超过 1.0
    /// 4. **阈值过滤** - 只返回满足意图阈值的匹配
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "count_python_lines",
    ///     IntentDomain::FileOps,
    ///     vec!["python".to_string(), "行数".to_string()],
    ///     vec![r"统计.*python.*行数".to_string()],
    ///     0.5,
    /// );
    /// matcher.register(intent);
    ///
    /// let matches = matcher.match_intent("统计 Python 代码行数");
    /// assert!(!matches.is_empty());
    /// assert!(matches[0].confidence >= 0.5);
    /// ```
    pub fn match_intent(&self, input: &str) -> Vec<IntentMatch> {
        // 尝试从缓存中获取结果（使用 write lock 因为 LRU get 会修改顺序）
        if let Ok(mut cache) = self.query_cache.write() {
            if let Some(cached_result) = cache.get(input) {
                // 缓存命中
                if let Ok(mut hits) = self.cache_hits.write() {
                    *hits += 1;
                }
                return cached_result.clone();
            }
        }

        // 缓存未命中，执行实际匹配
        if let Ok(mut misses) = self.cache_misses.write() {
            *misses += 1;
        }

        let mut matches = Vec::new();

        // 将输入转换为小写以进行不区分大小写的匹配
        let input_lower = input.to_lowercase();

        for intent in &self.intents {
            let mut score: f64 = 0.0;
            let mut matched_keywords = Vec::new();

            // 1. 关键词匹配（每个关键词 0.3 分，模糊匹配权重更低）
            for keyword in &intent.keywords {
                let keyword_lower = keyword.to_lowercase();
                let mut matched = false;

                // 精确匹配：检查输入中的每个词是否包含关键词
                for input_word in input_lower.split_whitespace() {
                    if input_word.contains(&keyword_lower) {
                        score += 0.3;
                        matched_keywords.push(keyword.clone());
                        matched = true;
                        break;
                    }
                }

                // 模糊匹配：如果精确匹配失败且启用模糊匹配
                if !matched && self.fuzzy_config.enabled {
                    let mut best_similarity = 0.0;
                    let keyword_len = keyword_lower.chars().count();

                    for input_word in input_lower.split_whitespace() {
                        let input_len = input_word.chars().count();

                        // ✨ 长度预筛选优化：如果长度差异太大，跳过计算
                        // 例如：threshold=0.8 时，长度比率必须 >= 0.8
                        let len_ratio = if input_len < keyword_len {
                            input_len as f64 / keyword_len as f64
                        } else {
                            keyword_len as f64 / input_len as f64
                        };

                        // 长度比率低于阈值，相似度不可能达标，跳过
                        if len_ratio < self.fuzzy_config.similarity_threshold {
                            continue;
                        }

                        let similarity = string_similarity(input_word, &keyword_lower);
                        if similarity > best_similarity {
                            best_similarity = similarity;
                        }
                    }

                    // 如果相似度超过阈值，按权重计分
                    if best_similarity >= self.fuzzy_config.similarity_threshold {
                        score += 0.3 * self.fuzzy_config.fuzzy_weight * best_similarity;
                        matched_keywords.push(format!("{}~", keyword)); // 使用 ~ 标记模糊匹配
                    }
                }
            }

            // 2. 正则模式匹配（每个模式 0.7 分）
            for pattern in &intent.patterns {
                if let Some(regex) = self.regex_cache.get(pattern) {
                    if regex.is_match(input) {
                        score += 0.7;
                    }
                }
            }

            // 3. 归一化置信度（不超过 1.0）
            let confidence = score.min(1.0);

            // 4. 只保留满足阈值的匹配
            if confidence >= intent.confidence_threshold {
                // ✨ Phase 3 Week 3: 实体提取
                let extracted_entities = self.extractor.extract(input, &intent.entities);

                let intent_match = IntentMatch {
                    intent: intent.clone(),
                    confidence,
                    matched_keywords,
                    extracted_entities,
                };
                matches.push(intent_match);
            }
        }

        // 按置信度降序排序
        matches.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // 将结果存入缓存
        if let Ok(mut cache) = self.query_cache.write() {
            cache.put(input.to_string(), matches.clone());
        }

        matches
    }

    /// 获取最佳匹配的意图
    ///
    /// 返回置信度最高的意图匹配结果。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "count_lines",
    ///     IntentDomain::FileOps,
    ///     vec!["统计".to_string(), "行数".to_string()],
    ///     vec![],
    ///     0.3,
    /// );
    /// matcher.register(intent);
    ///
    /// if let Some(best_match) = matcher.best_match("统计代码行数") {
    ///     assert_eq!(best_match.intent.name, "count_lines");
    /// }
    /// ```
    pub fn best_match(&self, input: &str) -> Option<IntentMatch> {
        self.match_intent(input).into_iter().next()
    }

    /// 获取已注册的意图数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    /// assert_eq!(matcher.len(), 0);
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec![],
    ///     vec![],
    ///     0.5,
    /// );
    /// matcher.register(intent);
    /// assert_eq!(matcher.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.intents.len()
    }

    /// 检查是否没有注册任何意图
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::IntentMatcher;
    ///
    /// let matcher = IntentMatcher::new();
    /// assert!(matcher.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.intents.is_empty()
    }

    /// 清空所有已注册的意图
    ///
    /// 同时会清空正则缓存、查询缓存和统计数据。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec![],
    ///     vec![],
    ///     0.5,
    /// );
    /// matcher.register(intent);
    /// assert!(!matcher.is_empty());
    ///
    /// matcher.clear();
    /// assert!(matcher.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.intents.clear();
        self.regex_cache.clear();
        self.clear_cache();
    }

    /// 清空查询缓存和统计数据
    ///
    /// 保留已注册的意图，只清空缓存的查询结果和统计信息。
    /// 当你想重置缓存统计或释放缓存内存时可以使用此方法。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec!["test".to_string()],
    ///     vec![],
    ///     0.3,
    /// );
    /// matcher.register(intent);
    ///
    /// // 执行一些查询
    /// matcher.match_intent("test query");
    /// matcher.match_intent("test query"); // 第二次查询会命中缓存
    ///
    /// assert!(matcher.cache_hits() > 0);
    ///
    /// // 清空缓存
    /// matcher.clear_cache();
    /// assert_eq!(matcher.cache_hits(), 0);
    /// assert_eq!(matcher.cache_misses(), 0);
    /// assert!(!matcher.is_empty()); // 意图仍然保留
    /// ```
    pub fn clear_cache(&mut self) {
        if let Ok(mut cache) = self.query_cache.write() {
            cache.clear();
        }
        if let Ok(mut hits) = self.cache_hits.write() {
            *hits = 0;
        }
        if let Ok(mut misses) = self.cache_misses.write() {
            *misses = 0;
        }
    }

    /// 获取缓存命中次数
    ///
    /// 返回查询缓存命中的次数。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec!["test".to_string()],
    ///     vec![],
    ///     0.3,
    /// );
    /// matcher.register(intent);
    ///
    /// assert_eq!(matcher.cache_hits(), 0);
    ///
    /// // 第一次查询会缓存未命中
    /// matcher.match_intent("test query");
    /// assert_eq!(matcher.cache_hits(), 0);
    ///
    /// // 第二次相同查询会命中缓存
    /// matcher.match_intent("test query");
    /// assert_eq!(matcher.cache_hits(), 1);
    /// ```
    pub fn cache_hits(&self) -> usize {
        self.cache_hits.read().map(|hits| *hits).unwrap_or(0)
    }

    /// 获取缓存未命中次数
    ///
    /// 返回查询缓存未命中的次数（即执行了实际匹配的次数）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec!["test".to_string()],
    ///     vec![],
    ///     0.3,
    /// );
    /// matcher.register(intent);
    ///
    /// assert_eq!(matcher.cache_misses(), 0);
    ///
    /// // 每次不同的查询都会缓存未命中
    /// matcher.match_intent("test query 1");
    /// assert_eq!(matcher.cache_misses(), 1);
    ///
    /// matcher.match_intent("test query 2");
    /// assert_eq!(matcher.cache_misses(), 2);
    /// ```
    pub fn cache_misses(&self) -> usize {
        self.cache_misses.read().map(|misses| *misses).unwrap_or(0)
    }

    /// 获取缓存命中率
    ///
    /// 返回缓存命中率（0.0 到 1.0 之间）。
    /// 如果还没有任何查询，返回 0.0。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use simpleconsole::dsl::intent::{Intent, IntentDomain, IntentMatcher};
    ///
    /// let mut matcher = IntentMatcher::new();
    ///
    /// let intent = Intent::new(
    ///     "test",
    ///     IntentDomain::FileOps,
    ///     vec!["test".to_string()],
    ///     vec![],
    ///     0.3,
    /// );
    /// matcher.register(intent);
    ///
    /// // 没有查询时命中率为 0.0
    /// assert_eq!(matcher.cache_hit_rate(), 0.0);
    ///
    /// // 执行一些查询
    /// matcher.match_intent("test query");  // miss
    /// matcher.match_intent("test query");  // hit
    /// matcher.match_intent("test query");  // hit
    ///
    /// // 命中率 = 2 / (1 + 2) = 0.666...
    /// let hit_rate = matcher.cache_hit_rate();
    /// assert!(hit_rate > 0.6 && hit_rate < 0.7);
    /// ```
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits();
        let misses = self.cache_misses();
        let total = hits + misses;

        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// 获取所有已注册的意图的引用
    pub fn intents(&self) -> &[Intent] {
        &self.intents
    }
}

impl Default for IntentMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::intent::types::IntentDomain;

    #[test]
    fn test_matcher_creation() {
        let matcher = IntentMatcher::new();
        assert_eq!(matcher.len(), 0);
        assert!(matcher.is_empty());
    }

    #[test]
    fn test_register_intent() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test_intent",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            vec![r"test.*".to_string()],
            0.5,
        );

        matcher.register(intent);

        assert_eq!(matcher.len(), 1);
        assert!(!matcher.is_empty());
        assert_eq!(matcher.intents()[0].name, "test_intent");
    }

    #[test]
    fn test_keyword_matching() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec!["统计".to_string(), "文件".to_string()],
            Vec::new(),
            0.3, // 需要至少 0.3 分（1 个关键词）
        );

        matcher.register(intent);

        // 匹配包含关键词的输入
        let matches = matcher.match_intent("统计 Python 文件数量");
        assert!(!matches.is_empty());
        assert!(matches[0].confidence >= 0.3);
        assert!(matches[0].matched_keywords.contains(&"统计".to_string()));
        assert!(matches[0].matched_keywords.contains(&"文件".to_string()));
    }

    #[test]
    fn test_pattern_matching() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_lines",
            IntentDomain::FileOps,
            Vec::new(),
            vec![r"统计.*行数".to_string()],
            0.5, // 需要至少 0.5 分
        );

        matcher.register(intent);

        // 匹配符合模式的输入
        let matches = matcher.match_intent("统计 Python 代码行数");
        assert!(!matches.is_empty());
        assert!(matches[0].confidence >= 0.7); // 模式匹配贡献 0.7 分
    }

    #[test]
    fn test_combined_matching() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_python_lines",
            IntentDomain::FileOps,
            vec!["python".to_string(), "行数".to_string()],
            vec![r"(?i)统计.*python.*行数".to_string()], // 使用 (?i) 使正则表达式不区分大小写
            0.7, // 较高的阈值
        );

        matcher.register(intent);

        // 同时匹配关键词和模式
        let matches = matcher.match_intent("统计 Python 代码行数");
        assert!(!matches.is_empty());
        assert!(matches[0].confidence >= 0.7);
    }

    #[test]
    fn test_threshold_filtering() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_lines",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.5, // 阈值 0.5
        );

        matcher.register(intent);

        // 只有一个关键词，置信度 0.3 < 0.5，不应返回
        let matches = matcher.match_intent("统计");
        assert!(matches.is_empty());

        // 多个关键词或模式匹配可以超过阈值
        let intent2 = Intent::new(
            "count_lines2",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            vec![r"统计".to_string()],
            0.5,
        );

        matcher.register(intent2);
        let matches = matcher.match_intent("统计行数");
        assert!(!matches.is_empty()); // 关键词 0.3 + 模式 0.7 = 1.0 > 0.5
    }

    #[test]
    fn test_case_insensitive_matching() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["Python".to_string(), "COUNT".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 不区分大小写的匹配
        let matches = matcher.match_intent("python count");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].matched_keywords.len(), 2);
    }

    #[test]
    fn test_best_match() {
        let mut matcher = IntentMatcher::new();

        // 注册多个意图
        let intent1 = Intent::new(
            "low_confidence",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.3,
        );

        let intent2 = Intent::new(
            "high_confidence",
            IntentDomain::FileOps,
            vec!["统计".to_string(), "行数".to_string()],
            vec![r"统计.*行数".to_string()],
            0.3,
        );

        matcher.register(intent1);
        matcher.register(intent2);

        // best_match 应该返回置信度最高的
        let best = matcher.best_match("统计代码行数");
        assert!(best.is_some());
        assert_eq!(best.unwrap().intent.name, "high_confidence");
    }

    #[test]
    fn test_multiple_intents_sorting() {
        let mut matcher = IntentMatcher::new();

        let intent1 = Intent::new(
            "intent_a",
            IntentDomain::FileOps,
            vec!["a".to_string()],
            Vec::new(),
            0.3,
        );

        let intent2 = Intent::new(
            "intent_b",
            IntentDomain::FileOps,
            vec!["b".to_string(), "c".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent1);
        matcher.register(intent2);

        let matches = matcher.match_intent("a b c");
        assert_eq!(matches.len(), 2);
        // intent_b 应该排在前面（置信度更高）
        assert_eq!(matches[0].intent.name, "intent_b");
        assert_eq!(matches[1].intent.name, "intent_a");
    }

    #[test]
    fn test_clear() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.5,
        );

        matcher.register(intent);
        assert!(!matcher.is_empty());

        matcher.clear();
        assert!(matcher.is_empty());
        assert_eq!(matcher.len(), 0);
    }

    #[test]
    fn test_invalid_regex_pattern() {
        let mut matcher = IntentMatcher::new();

        // 无效的正则表达式
        let intent = Intent::new(
            "bad_pattern",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            vec!["[invalid(".to_string()], // 无效的正则
            0.3, // 降低阈值以匹配单个关键词
        );

        // 应该不会 panic，只是不会匹配模式
        matcher.register(intent);

        let matches = matcher.match_intent("test something");
        assert!(!matches.is_empty()); // 关键词仍然可以匹配
        assert_eq!(matches[0].confidence, 0.3); // 只有关键词匹配
    }

    #[test]
    fn test_confidence_normalization() {
        let mut matcher = IntentMatcher::new();

        // 很多关键词和模式，总分可能超过 1.0
        let intent = Intent::new(
            "many_matches",
            IntentDomain::FileOps,
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
            vec![r"a.*b.*c.*d".to_string()],
            0.5,
        );

        matcher.register(intent);

        let matches = matcher.match_intent("a b c d");
        assert!(!matches.is_empty());
        // 置信度应该被归一化到 1.0
        assert_eq!(matches[0].confidence, 1.0);
    }

    #[test]
    fn test_no_matches() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["python".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 完全不相关的输入
        let matches = matcher.match_intent("rust is great");
        assert!(matches.is_empty());
    }

    // ==================== 缓存测试 ====================

    #[test]
    fn test_cache_hits_and_misses() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 初始状态
        assert_eq!(matcher.cache_hits(), 0);
        assert_eq!(matcher.cache_misses(), 0);
        assert_eq!(matcher.cache_hit_rate(), 0.0);

        // 第一次查询 - 缓存未命中
        matcher.match_intent("test query");
        assert_eq!(matcher.cache_hits(), 0);
        assert_eq!(matcher.cache_misses(), 1);

        // 第二次相同查询 - 缓存命中
        matcher.match_intent("test query");
        assert_eq!(matcher.cache_hits(), 1);
        assert_eq!(matcher.cache_misses(), 1);

        // 第三次相同查询 - 再次缓存命中
        matcher.match_intent("test query");
        assert_eq!(matcher.cache_hits(), 2);
        assert_eq!(matcher.cache_misses(), 1);

        // 不同的查询 - 缓存未命中
        matcher.match_intent("another query");
        assert_eq!(matcher.cache_hits(), 2);
        assert_eq!(matcher.cache_misses(), 2);

        // 验证命中率
        let hit_rate = matcher.cache_hit_rate();
        assert!((hit_rate - 0.5).abs() < 0.01); // 2 / (2 + 2) = 0.5
    }

    #[test]
    fn test_cache_with_custom_capacity() {
        let mut matcher = IntentMatcher::with_cache_capacity(2); // 只能缓存 2 个查询

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 添加 2 个不同的查询
        matcher.match_intent("query 1"); // miss
        matcher.match_intent("query 2"); // miss

        assert_eq!(matcher.cache_misses(), 2);

        // 再次查询应该命中缓存
        matcher.match_intent("query 1"); // hit
        matcher.match_intent("query 2"); // hit
        assert_eq!(matcher.cache_hits(), 2);

        // 添加第 3 个查询会淘汰最久未使用的（在这个场景下是 query 1，因为 query 2 更新）
        matcher.match_intent("query 3"); // miss
        assert_eq!(matcher.cache_misses(), 3);

        // query 2 和 query 3 应该仍在缓存中
        matcher.match_intent("query 2"); // hit
        matcher.match_intent("query 3"); // hit
        assert_eq!(matcher.cache_hits(), 4);

        // query 1 应该已经被淘汰
        matcher.match_intent("query 1"); // miss
        assert_eq!(matcher.cache_misses(), 4);
    }

    #[test]
    fn test_clear_cache() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 执行一些查询
        matcher.match_intent("test query");
        matcher.match_intent("test query"); // hit
        matcher.match_intent("another query");

        assert_eq!(matcher.cache_hits(), 1);
        assert_eq!(matcher.cache_misses(), 2);

        // 清空缓存
        matcher.clear_cache();

        // 统计应该被重置
        assert_eq!(matcher.cache_hits(), 0);
        assert_eq!(matcher.cache_misses(), 0);
        assert_eq!(matcher.cache_hit_rate(), 0.0);

        // 意图应该仍然保留
        assert_eq!(matcher.len(), 1);

        // 之前缓存的查询现在应该会 miss
        matcher.match_intent("test query");
        assert_eq!(matcher.cache_misses(), 1);
        assert_eq!(matcher.cache_hits(), 0);
    }

    #[test]
    fn test_register_clears_cache() {
        let mut matcher = IntentMatcher::new();

        let intent1 = Intent::new(
            "test1",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent1);

        // 执行一些查询
        matcher.match_intent("test query");
        matcher.match_intent("test query"); // hit
        assert_eq!(matcher.cache_hits(), 1);

        // 注册新意图应该清空缓存（但不清空统计）
        let intent2 = Intent::new(
            "test2",
            IntentDomain::FileOps,
            vec!["another".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent2);

        // 统计应该保留
        assert_eq!(matcher.cache_hits(), 1);
        assert_eq!(matcher.cache_misses(), 1);

        // 但是之前缓存的查询应该不再有效（会重新匹配）
        // 因为新意图可能会改变匹配结果
        let prev_misses = matcher.cache_misses();
        matcher.match_intent("test query");
        // 这次查询应该是 miss（因为缓存被清空了）
        assert_eq!(matcher.cache_misses(), prev_misses + 1);
    }

    #[test]
    fn test_clear_also_clears_cache() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["test".to_string()],
            Vec::new(),
            0.3,
        );

        matcher.register(intent);

        // 执行一些查询
        matcher.match_intent("test query");
        matcher.match_intent("test query"); // hit

        assert_eq!(matcher.cache_hits(), 1);
        assert_eq!(matcher.cache_misses(), 1);

        // clear() 应该清空所有内容
        matcher.clear();

        // 意图、缓存和统计都应该被清空
        assert!(matcher.is_empty());
        assert_eq!(matcher.cache_hits(), 0);
        assert_eq!(matcher.cache_misses(), 0);
    }

    #[test]
    fn test_cache_returns_same_results() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec!["统计".to_string(), "文件".to_string()],
            vec![r"统计.*文件".to_string()],
            0.5,
        );

        matcher.register(intent);

        // 第一次查询（缓存未命中）
        let matches1 = matcher.match_intent("统计 Python 文件数量");
        assert_eq!(matcher.cache_misses(), 1);

        // 第二次相同查询（缓存命中）
        let matches2 = matcher.match_intent("统计 Python 文件数量");
        assert_eq!(matcher.cache_hits(), 1);

        // 两次查询结果应该完全相同
        assert_eq!(matches1.len(), matches2.len());
        if !matches1.is_empty() {
            assert_eq!(matches1[0].intent.name, matches2[0].intent.name);
            assert_eq!(matches1[0].confidence, matches2[0].confidence);
            assert_eq!(
                matches1[0].matched_keywords,
                matches2[0].matched_keywords
            );
        }
    }

    // ==================== 模糊匹配测试 ====================

    #[test]
    fn test_fuzzy_matching_simple() {
        // 最小化测试：单个词的模糊匹配
        let fuzzy_config = FuzzyConfig::enabled(0.45, 0.7);
        let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.05,
        );

        matcher.register(intent);

        // 测试单个词的输入
        let matches = matcher.match_intent("统记");
        if matches.is_empty() {
            eprintln!("Failed to match '统记' with fuzzy matching");
            eprintln!("Config: threshold={}, weight={}",
                      matcher.fuzzy_config.similarity_threshold,
                      matcher.fuzzy_config.fuzzy_weight);

            // 计算期望的相似度和分数
            let sim = string_similarity("统记", "统计");
            let expected_score = 0.3 * 0.7 * sim;
            eprintln!("Similarity: {}, Expected score: {}", sim, expected_score);
        }

        assert!(!matches.is_empty(), "Should match '统记' with fuzzy matching");
    }

    #[test]
    fn test_levenshtein_distance() {
        // 完全相同
        assert_eq!(levenshtein_distance("kitten", "kitten"), 0);
        assert_eq!(levenshtein_distance("统计", "统计"), 0);

        // 单字符差异
        assert_eq!(levenshtein_distance("kitten", "sitten"), 1); // 替换
        assert_eq!(levenshtein_distance("统计", "统记"), 1); // 替换

        // 经典示例
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);

        // 空字符串
        assert_eq!(levenshtein_distance("", "hello"), 5);
        assert_eq!(levenshtein_distance("hello", ""), 5);
        assert_eq!(levenshtein_distance("", ""), 0);

        // 中文示例
        assert_eq!(levenshtein_distance("统计文件", "统计文档"), 1);
        assert_eq!(levenshtein_distance("统计行数", "统计代码"), 2);
    }

    #[test]
    fn test_string_similarity() {
        // 完全相同 -> 1.0
        assert_eq!(string_similarity("统计", "统计"), 1.0);
        assert_eq!(string_similarity("hello", "hello"), 1.0);

        // 完全不同 -> 较低的相似度
        assert!(string_similarity("统计", "文件") < 0.5);
        assert!(string_similarity("hello", "world") < 0.5);

        // 一个字符差异
        let sim = string_similarity("统计", "统记");
        assert!((0.5..1.0).contains(&sim)); // 应该在 0.5 到 1.0 之间
        assert_eq!(sim, 0.5); // 编辑距离 1，最大长度 2，相似度 = 1 - 1/2 = 0.5

        // 空字符串
        assert_eq!(string_similarity("", ""), 1.0);

        // 相似的词
        let sim_count = string_similarity("count", "counts");
        assert!(sim_count > 0.8); // 只差一个字符，应该很相似
    }

    #[test]
    fn test_fuzzy_matching_disabled_by_default() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.2, // 低阈值
        );

        matcher.register(intent);

        // 默认模糊匹配是禁用的，所以 "统记" 不应该匹配 "统计"
        let matches = matcher.match_intent("统记文件");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_fuzzy_matching_enabled() {
        let fuzzy_config = FuzzyConfig::enabled(0.45, 0.7); // 相似度阈值 0.45（低于 0.5）
        let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

        let intent = Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec!["统计".to_string()], // "统计"
            Vec::new(),
            0.05, // 更低的阈值，确保模糊匹配可以通过
        );

        matcher.register(intent);

        // 测试1：单个词的模糊匹配
        // "统记" 与 "统计" 相似度 = 1 - 1/2 = 0.5（Levenshtein 距离为 1，最大长度为 2）
        // 分数 = 0.3 * 0.7 * 0.5 = 0.105
        let matches = matcher.match_intent("统记");
        assert!(!matches.is_empty(), "Should match '统记' with fuzzy matching");
        assert_eq!(matches[0].intent.name, "count_files");
        assert!(matches[0].matched_keywords.iter().any(|k| k.contains('~')));

        // 测试2：包含模糊匹配词的句子
        let matches = matcher.match_intent("统记 python 文件");
        assert!(!matches.is_empty(), "Should match '统记 python 文件' with fuzzy matching");
        assert!(matches[0].matched_keywords.iter().any(|k| k.contains('~')));
    }

    #[test]
    fn test_fuzzy_matching_threshold() {
        let fuzzy_config = FuzzyConfig::enabled(0.9, 0.7); // 高相似度阈值
        let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

        let intent = Intent::new(
            "count_files",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.1,
        );

        matcher.register(intent);

        // "文件" 与 "统计" 相似度很低，不应该匹配
        let matches = matcher.match_intent("文件数量");
        assert!(matches.is_empty());

        // "统计" 精确匹配
        let matches = matcher.match_intent("统计文件");
        assert!(!matches.is_empty());
    }

    #[test]
    fn test_fuzzy_matching_confidence_score() {
        let fuzzy_config = FuzzyConfig::enabled(0.5, 0.7);
        let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["统计".to_string()], // 使用中文避免子字符串匹配问题
            Vec::new(),
            0.05,
        );

        matcher.register(intent);

        // 精确匹配应该得到 0.3 分
        let exact_matches = matcher.match_intent("统计 文件");
        assert!(!exact_matches.is_empty());
        assert_eq!(exact_matches[0].confidence, 0.3);

        // 模糊匹配应该得到较低的分数
        // "统记" 与 "统计" 相似度 = 0.5
        // 分数 = 0.3 * 0.7 * 0.5 = 0.105
        let fuzzy_matches = matcher.match_intent("统记 文件");
        assert!(!fuzzy_matches.is_empty());
        let fuzzy_score = fuzzy_matches[0].confidence;
        // 模糊匹配分数应该低于精确匹配
        assert!(fuzzy_score < 0.3, "Fuzzy score {} should be less than 0.3", fuzzy_score);
        assert!(fuzzy_score > 0.05, "Fuzzy score {} should be greater than 0.05", fuzzy_score);
    }

    #[test]
    fn test_enable_disable_fuzzy_matching() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.05,
        );

        matcher.register(intent);

        // 默认禁用模糊匹配
        let matches = matcher.match_intent("统记");
        assert!(matches.is_empty());

        // 启用模糊匹配
        matcher.enable_fuzzy_matching(0.45, 0.7);
        let matches = matcher.match_intent("统记");
        assert!(!matches.is_empty());

        // 禁用模糊匹配
        matcher.disable_fuzzy_matching();
        let matches = matcher.match_intent("统记");
        assert!(matches.is_empty());
    }

    #[test]
    fn test_fuzzy_matching_exact_takes_precedence() {
        let fuzzy_config = FuzzyConfig::enabled(0.5, 0.7);
        let mut matcher = IntentMatcher::with_config(100, fuzzy_config);

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["count".to_string()],
            Vec::new(),
            0.1,
        );

        matcher.register(intent);

        // 精确匹配应该优先于模糊匹配
        let matches = matcher.match_intent("count files");
        assert!(!matches.is_empty());
        assert_eq!(matches[0].confidence, 0.3); // 精确匹配的分数

        // matched_keywords 不应该包含模糊匹配标记
        assert!(!matches[0]
            .matched_keywords
            .iter()
            .any(|k| k.contains('~')));
    }

    #[test]
    fn test_fuzzy_matching_clears_cache() {
        let mut matcher = IntentMatcher::new();

        let intent = Intent::new(
            "test",
            IntentDomain::FileOps,
            vec!["统计".to_string()],
            Vec::new(),
            0.05,
        );

        matcher.register(intent);

        // 执行查询并缓存（不会匹配，因为模糊匹配未启用）
        matcher.match_intent("统记");
        matcher.match_intent("统记"); // hit
        assert_eq!(matcher.cache_hits(), 1);

        // 启用模糊匹配应该清空缓存
        matcher.enable_fuzzy_matching(0.45, 0.7); // 阈值 0.45，低于 0.5
        assert_eq!(matcher.cache_hits(), 0);
        assert_eq!(matcher.cache_misses(), 0);

        // 再次查询，现在应该能匹配到（模糊匹配）
        let matches = matcher.match_intent("统记");
        assert!(!matches.is_empty(), "Should fuzzy match after enabling");
        assert!(matches[0].matched_keywords.iter().any(|k| k.contains('~')));
    }
}
