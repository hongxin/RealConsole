//! 补全缓存系统
//!
//! 使用 LRU 缓存优化补全性能

use lru::LruCache;
use rustyline::completion::Pair;
use std::num::NonZeroUsize;

/// 补全缓存
///
/// # 缓存策略
///
/// - 容量：1000 条（平衡内存使用和命中率）
/// - 算法：LRU（Least Recently Used）
/// - 场景：相同输入频繁补全（如反复输入 "/he"）
pub struct CompletionCache {
    /// LRU 缓存
    cache: LruCache<String, Vec<Pair>>,
}

impl CompletionCache {
    /// 创建新的补全缓存
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// 创建指定容量的缓存
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
        }
    }

    /// 获取缓存的补全结果
    pub fn get(&mut self, input: &str) -> Option<&Vec<Pair>> {
        self.cache.get(input)
    }

    /// 缓存补全结果
    pub fn put(&mut self, input: String, candidates: Vec<Pair>) {
        self.cache.put(input, candidates);
    }

    /// 清空缓存
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 获取缓存大小
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 检查缓存是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.cache.len() == 0
    }
}

impl Default for CompletionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pairs() -> Vec<Pair> {
        vec![
            Pair {
                display: "test1".to_string(),
                replacement: "test1".to_string(),
            },
            Pair {
                display: "test2".to_string(),
                replacement: "test2".to_string(),
            },
        ]
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = CompletionCache::new();
        let pairs = create_test_pairs();

        cache.put("test".to_string(), pairs.clone());

        let cached = cache.get("test");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().len(), 2);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = CompletionCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = CompletionCache::new();
        cache.put("test".to_string(), create_test_pairs());

        assert_eq!(cache.len(), 1);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = CompletionCache::with_capacity(2);

        cache.put("key1".to_string(), create_test_pairs());
        cache.put("key2".to_string(), create_test_pairs());
        cache.put("key3".to_string(), create_test_pairs()); // 应该淘汰 key1

        assert!(cache.get("key1").is_none()); // key1 已被淘汰
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_update_access_order() {
        let mut cache = CompletionCache::with_capacity(2);

        cache.put("key1".to_string(), create_test_pairs());
        cache.put("key2".to_string(), create_test_pairs());

        // 访问 key1，更新其访问时间
        let _ = cache.get("key1");

        // 插入 key3，应该淘汰 key2（而不是 key1）
        cache.put("key3".to_string(), create_test_pairs());

        assert!(cache.get("key1").is_some()); // key1 应该还在
        assert!(cache.get("key2").is_none()); // key2 被淘汰
        assert!(cache.get("key3").is_some());
    }
}
