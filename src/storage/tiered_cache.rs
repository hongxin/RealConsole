//! 三层 LRU 缓存
//!
//! v1.59.0: v2.0 探路期 - 分层缓存优化
//!
//! ## 设计理念
//!
//! 基于"一分为三"哲学的三层缓存架构：
//! - **Hot (热层)**: 最常访问，容量小，命中最快
//! - **Warm (温层)**: 较常访问，容量中等
//! - **Cold (冷层)**: 不常访问，容量大，命中较慢
//!
//! ## 升降级策略
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │                    访问模式                          │
//! └─────────────────────────────────────────────────────┘
//!
//!     命中 Hot ─────────► 保持在 Hot
//!          │
//!     命中 Warm ────────► 提升到 Hot（如果访问频次达标）
//!          │
//!     命中 Cold ────────► 提升到 Warm
//!          │
//!     未命中 ───────────► 插入 Cold
//!
//! ┌─────────────────────────────────────────────────────┐
//! │                    淘汰策略                          │
//! └─────────────────────────────────────────────────────┘
//!
//!     Hot 满 ───────────► Hot LRU 降级到 Warm
//!          │
//!     Warm 满 ──────────► Warm LRU 降级到 Cold
//!          │
//!     Cold 满 ──────────► Cold LRU 淘汰
//! ```
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::TieredCache;
//!
//! let cache: TieredCache<String> = TieredCache::new(100, 500, 2000);
//!
//! // 插入数据
//! cache.insert("key1".to_string(), "value1".to_string());
//!
//! // 获取数据（自动升级）
//! if let Some(value) = cache.get(&"key1".to_string()) {
//!     println!("Found: {}", value);
//! }
//!
//! // 查看统计
//! let stats = cache.stats();
//! println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
//! ```

use lru::LruCache;
use std::hash::Hash;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// 三层 LRU 缓存
///
/// 分层缓存结构，自动管理数据在层级间的升降级
pub struct TieredCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// 热层：最常访问，容量最小
    hot: RwLock<LruCache<K, CacheEntry<V>>>,
    /// 温层：较常访问，容量中等
    warm: RwLock<LruCache<K, CacheEntry<V>>>,
    /// 冷层：不常访问，容量最大
    cold: RwLock<LruCache<K, CacheEntry<V>>>,
    /// 缓存配置
    config: TieredCacheConfig,
    /// 统计信息
    stats: CacheStats,
}

/// 缓存条目
#[derive(Clone)]
struct CacheEntry<V> {
    /// 数据值
    value: V,
    /// 访问次数
    access_count: u32,
}

impl<V> CacheEntry<V> {
    fn new(value: V) -> Self {
        Self {
            value,
            access_count: 1,
        }
    }

    fn increment_access(&mut self) {
        self.access_count = self.access_count.saturating_add(1);
    }
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct TieredCacheConfig {
    /// 热层容量
    pub hot_capacity: usize,
    /// 温层容量
    pub warm_capacity: usize,
    /// 冷层容量
    pub cold_capacity: usize,
    /// 提升到热层所需的最小访问次数
    pub promotion_threshold: u32,
}

impl Default for TieredCacheConfig {
    fn default() -> Self {
        Self {
            hot_capacity: 100,
            warm_capacity: 500,
            cold_capacity: 2000,
            promotion_threshold: 3,
        }
    }
}

/// 缓存统计信息
#[derive(Debug, Default)]
pub struct CacheStats {
    /// 热层命中
    hot_hits: AtomicU64,
    /// 温层命中
    warm_hits: AtomicU64,
    /// 冷层命中
    cold_hits: AtomicU64,
    /// 未命中
    misses: AtomicU64,
    /// 插入次数
    inserts: AtomicU64,
    /// 提升次数（cold→warm, warm→hot）
    promotions: AtomicU64,
    /// 降级次数（hot→warm, warm→cold）
    demotions: AtomicU64,
    /// 淘汰次数
    evictions: AtomicU64,
}

impl CacheStats {
    /// 总命中数
    pub fn total_hits(&self) -> u64 {
        self.hot_hits.load(Ordering::Relaxed)
            + self.warm_hits.load(Ordering::Relaxed)
            + self.cold_hits.load(Ordering::Relaxed)
    }

    /// 总访问数
    pub fn total_accesses(&self) -> u64 {
        self.total_hits() + self.misses.load(Ordering::Relaxed)
    }

    /// 总体命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 {
            0.0
        } else {
            self.total_hits() as f64 / total as f64
        }
    }

    /// 热层命中率
    pub fn hot_hit_rate(&self) -> f64 {
        let total = self.total_accesses();
        if total == 0 {
            0.0
        } else {
            self.hot_hits.load(Ordering::Relaxed) as f64 / total as f64
        }
    }

    /// 获取详细统计
    pub fn detailed(&self) -> DetailedCacheStats {
        DetailedCacheStats {
            hot_hits: self.hot_hits.load(Ordering::Relaxed),
            warm_hits: self.warm_hits.load(Ordering::Relaxed),
            cold_hits: self.cold_hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            inserts: self.inserts.load(Ordering::Relaxed),
            promotions: self.promotions.load(Ordering::Relaxed),
            demotions: self.demotions.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
        }
    }
}

/// 详细缓存统计（可序列化）
#[derive(Debug, Clone)]
pub struct DetailedCacheStats {
    pub hot_hits: u64,
    pub warm_hits: u64,
    pub cold_hits: u64,
    pub misses: u64,
    pub inserts: u64,
    pub promotions: u64,
    pub demotions: u64,
    pub evictions: u64,
}

impl DetailedCacheStats {
    /// 总命中率
    pub fn hit_rate(&self) -> f64 {
        let total_hits = self.hot_hits + self.warm_hits + self.cold_hits;
        let total = total_hits + self.misses;
        if total == 0 {
            0.0
        } else {
            total_hits as f64 / total as f64
        }
    }
}

/// 缓存层级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheTier {
    Hot,
    Warm,
    Cold,
    None,
}

impl<K, V> TieredCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// 创建新的三层缓存
    pub fn new(hot_capacity: usize, warm_capacity: usize, cold_capacity: usize) -> Self {
        Self::with_config(TieredCacheConfig {
            hot_capacity,
            warm_capacity,
            cold_capacity,
            ..Default::default()
        })
    }

    /// 使用配置创建缓存
    pub fn with_config(config: TieredCacheConfig) -> Self {
        Self {
            hot: RwLock::new(LruCache::new(
                NonZeroUsize::new(config.hot_capacity).unwrap_or(NonZeroUsize::MIN),
            )),
            warm: RwLock::new(LruCache::new(
                NonZeroUsize::new(config.warm_capacity).unwrap_or(NonZeroUsize::MIN),
            )),
            cold: RwLock::new(LruCache::new(
                NonZeroUsize::new(config.cold_capacity).unwrap_or(NonZeroUsize::MIN),
            )),
            config,
            stats: CacheStats::default(),
        }
    }

    /// 使用默认配置创建缓存
    pub fn with_defaults() -> Self {
        Self::with_config(TieredCacheConfig::default())
    }

    /// 获取数据
    ///
    /// 自动处理升级逻辑
    pub fn get(&self, key: &K) -> Option<V> {
        // 先检查热层
        {
            let mut hot = self.hot.write().unwrap();
            if let Some(entry) = hot.get_mut(key) {
                entry.increment_access();
                self.stats.hot_hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.value.clone());
            }
        }

        // 检查温层
        {
            let mut warm = self.warm.write().unwrap();
            if let Some(entry) = warm.pop(key) {
                self.stats.warm_hits.fetch_add(1, Ordering::Relaxed);

                // 检查是否应该提升到热层
                let mut new_entry = entry.clone();
                new_entry.increment_access();

                if new_entry.access_count >= self.config.promotion_threshold {
                    // 提升到热层
                    self.promote_to_hot(key.clone(), new_entry.clone());
                } else {
                    // 保持在温层
                    warm.put(key.clone(), new_entry.clone());
                }

                return Some(entry.value);
            }
        }

        // 检查冷层
        {
            let mut cold = self.cold.write().unwrap();
            if let Some(entry) = cold.pop(key) {
                self.stats.cold_hits.fetch_add(1, Ordering::Relaxed);

                // 提升到温层
                let mut new_entry = entry.clone();
                new_entry.increment_access();
                self.promote_to_warm(key.clone(), new_entry);
                self.stats.promotions.fetch_add(1, Ordering::Relaxed);

                return Some(entry.value);
            }
        }

        // 未命中
        self.stats.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// 插入数据（插入到冷层）
    pub fn insert(&self, key: K, value: V) {
        self.stats.inserts.fetch_add(1, Ordering::Relaxed);

        // 检查是否已存在于任何层
        if self.contains(&key) {
            // 更新现有条目
            self.update_existing(key, value);
            return;
        }

        // 新条目插入冷层
        let entry = CacheEntry::new(value);
        let mut cold = self.cold.write().unwrap();

        if cold.len() >= self.config.cold_capacity {
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }

        cold.put(key, entry);
    }

    /// 直接插入到热层（用于已知重要的数据）
    pub fn insert_hot(&self, key: K, value: V) {
        self.stats.inserts.fetch_add(1, Ordering::Relaxed);

        let entry = CacheEntry {
            value,
            access_count: self.config.promotion_threshold,
        };

        self.promote_to_hot(key, entry);
    }

    /// 检查键是否存在
    pub fn contains(&self, key: &K) -> bool {
        self.tier_of(key) != CacheTier::None
    }

    /// 获取键所在的层级
    pub fn tier_of(&self, key: &K) -> CacheTier {
        if self.hot.read().unwrap().contains(key) {
            CacheTier::Hot
        } else if self.warm.read().unwrap().contains(key) {
            CacheTier::Warm
        } else if self.cold.read().unwrap().contains(key) {
            CacheTier::Cold
        } else {
            CacheTier::None
        }
    }

    /// 删除数据
    pub fn remove(&self, key: &K) -> Option<V> {
        // 尝试从各层删除
        if let Some(entry) = self.hot.write().unwrap().pop(key) {
            return Some(entry.value);
        }
        if let Some(entry) = self.warm.write().unwrap().pop(key) {
            return Some(entry.value);
        }
        if let Some(entry) = self.cold.write().unwrap().pop(key) {
            return Some(entry.value);
        }
        None
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        self.hot.write().unwrap().clear();
        self.warm.write().unwrap().clear();
        self.cold.write().unwrap().clear();
    }

    /// 获取总条目数
    pub fn len(&self) -> usize {
        self.hot.read().unwrap().len()
            + self.warm.read().unwrap().len()
            + self.cold.read().unwrap().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 获取各层大小
    pub fn tier_sizes(&self) -> (usize, usize, usize) {
        (
            self.hot.read().unwrap().len(),
            self.warm.read().unwrap().len(),
            self.cold.read().unwrap().len(),
        )
    }

    /// 获取统计信息
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 获取配置
    pub fn config(&self) -> &TieredCacheConfig {
        &self.config
    }

    // ========================================================================
    // 内部方法
    // ========================================================================

    /// 提升到热层
    fn promote_to_hot(&self, key: K, entry: CacheEntry<V>) {
        let mut hot = self.hot.write().unwrap();

        // 如果热层满了，降级最老的到温层
        if hot.len() >= self.config.hot_capacity {
            if let Some((old_key, old_entry)) = hot.pop_lru() {
                self.demote_to_warm(old_key, old_entry);
                self.stats.demotions.fetch_add(1, Ordering::Relaxed);
            }
        }

        hot.put(key, entry);
        self.stats.promotions.fetch_add(1, Ordering::Relaxed);
    }

    /// 提升到温层
    fn promote_to_warm(&self, key: K, entry: CacheEntry<V>) {
        let mut warm = self.warm.write().unwrap();

        // 如果温层满了，降级最老的到冷层
        if warm.len() >= self.config.warm_capacity {
            if let Some((old_key, old_entry)) = warm.pop_lru() {
                self.demote_to_cold(old_key, old_entry);
                self.stats.demotions.fetch_add(1, Ordering::Relaxed);
            }
        }

        warm.put(key, entry);
    }

    /// 降级到温层
    fn demote_to_warm(&self, key: K, entry: CacheEntry<V>) {
        let mut warm = self.warm.write().unwrap();

        if warm.len() >= self.config.warm_capacity {
            if let Some((old_key, old_entry)) = warm.pop_lru() {
                self.demote_to_cold(old_key, old_entry);
            }
        }

        warm.put(key, entry);
    }

    /// 降级到冷层
    fn demote_to_cold(&self, key: K, entry: CacheEntry<V>) {
        let mut cold = self.cold.write().unwrap();

        if cold.len() >= self.config.cold_capacity {
            self.stats.evictions.fetch_add(1, Ordering::Relaxed);
        }

        cold.put(key, entry);
    }

    /// 更新已存在的条目
    fn update_existing(&self, key: K, value: V) {
        // 检查热层
        {
            let mut hot = self.hot.write().unwrap();
            if let Some(entry) = hot.get_mut(&key) {
                entry.value = value;
                entry.increment_access();
                return;
            }
        }

        // 检查温层
        {
            let mut warm = self.warm.write().unwrap();
            if let Some(entry) = warm.get_mut(&key) {
                entry.value = value;
                entry.increment_access();
                return;
            }
        }

        // 检查冷层
        {
            let mut cold = self.cold.write().unwrap();
            if let Some(entry) = cold.get_mut(&key) {
                entry.value = value;
                entry.increment_access();
            }
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());

        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some("value1".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        let value = cache.get(&"nonexistent".to_string());
        assert_eq!(value, None);

        let stats = cache.stats().detailed();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_insert_to_cold() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());

        // 新插入的数据应该在冷层
        assert_eq!(cache.tier_of(&"key1".to_string()), CacheTier::Cold);
    }

    #[test]
    fn test_cache_promotion_cold_to_warm() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());

        // 第一次访问，从冷层提升到温层
        cache.get(&"key1".to_string());

        assert_eq!(cache.tier_of(&"key1".to_string()), CacheTier::Warm);
    }

    #[test]
    fn test_cache_promotion_warm_to_hot() {
        let config = TieredCacheConfig {
            hot_capacity: 10,
            warm_capacity: 50,
            cold_capacity: 200,
            promotion_threshold: 3,
        };
        let cache: TieredCache<String, String> = TieredCache::with_config(config);

        cache.insert("key1".to_string(), "value1".to_string());

        // 访问 3 次应该提升到热层
        // 第 1 次：cold → warm (access_count = 2)
        cache.get(&"key1".to_string());
        assert_eq!(cache.tier_of(&"key1".to_string()), CacheTier::Warm);

        // 第 2 次：warm 中 (access_count = 3，达到阈值)
        cache.get(&"key1".to_string());

        // 第 3 次后应该在热层
        assert_eq!(cache.tier_of(&"key1".to_string()), CacheTier::Hot);
    }

    #[test]
    fn test_cache_insert_hot() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert_hot("key1".to_string(), "value1".to_string());

        assert_eq!(cache.tier_of(&"key1".to_string()), CacheTier::Hot);
    }

    #[test]
    fn test_cache_remove() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());
        assert!(cache.contains(&"key1".to_string()));

        let removed = cache.remove(&"key1".to_string());
        assert_eq!(removed, Some("value1".to_string()));
        assert!(!cache.contains(&"key1".to_string()));
    }

    #[test]
    fn test_cache_clear() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert("key2".to_string(), "value2".to_string());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_tier_sizes() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.insert_hot("key2".to_string(), "value2".to_string());

        let (hot, warm, cold) = cache.tier_sizes();
        assert_eq!(hot, 1);
        assert_eq!(warm, 0);
        assert_eq!(cold, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());

        // 4 hits
        for _ in 0..4 {
            cache.get(&"key1".to_string());
        }

        // 1 miss
        cache.get(&"nonexistent".to_string());

        let hit_rate = cache.stats().hit_rate();
        assert!((hit_rate - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_cache_eviction() {
        // 非常小的缓存来测试淘汰
        let cache: TieredCache<i32, String> = TieredCache::new(2, 3, 5);

        // 插入超过冷层容量
        for i in 0..10 {
            cache.insert(i, format!("value_{}", i));
        }

        // 冷层应该只有 5 个
        let (_, _, cold) = cache.tier_sizes();
        assert_eq!(cold, 5);

        // 应该有淘汰
        let stats = cache.stats().detailed();
        assert!(stats.evictions > 0);
    }

    #[test]
    fn test_cache_demotion() {
        // 小缓存测试降级
        let cache: TieredCache<i32, String> = TieredCache::new(2, 3, 10);

        // 插入并提升到热层
        for i in 0..5 {
            cache.insert_hot(i, format!("value_{}", i));
        }

        // 热层应该只有 2 个，其余降级
        let (hot, warm, cold) = cache.tier_sizes();
        assert_eq!(hot, 2);
        assert!(warm > 0 || cold > 0);

        let stats = cache.stats().detailed();
        assert!(stats.demotions > 0);
    }

    #[test]
    fn test_cache_update_existing() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "original".to_string());
        cache.insert("key1".to_string(), "updated".to_string());

        let value = cache.get(&"key1".to_string());
        assert_eq!(value, Some("updated".to_string()));
    }

    #[test]
    fn test_cache_contains() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        assert!(!cache.contains(&"key1".to_string()));

        cache.insert("key1".to_string(), "value1".to_string());
        assert!(cache.contains(&"key1".to_string()));
    }

    #[test]
    fn test_cache_config() {
        let config = TieredCacheConfig {
            hot_capacity: 100,
            warm_capacity: 500,
            cold_capacity: 2000,
            promotion_threshold: 5,
        };

        let cache: TieredCache<String, String> = TieredCache::with_config(config.clone());

        assert_eq!(cache.config().hot_capacity, 100);
        assert_eq!(cache.config().warm_capacity, 500);
        assert_eq!(cache.config().cold_capacity, 2000);
        assert_eq!(cache.config().promotion_threshold, 5);
    }

    #[test]
    fn test_cache_detailed_stats() {
        let cache: TieredCache<String, String> = TieredCache::new(10, 50, 200);

        cache.insert("key1".to_string(), "value1".to_string());
        cache.get(&"key1".to_string()); // cold hit, promotes to warm
        cache.get(&"key1".to_string()); // warm hit
        cache.get(&"nonexistent".to_string()); // miss

        let stats = cache.stats().detailed();
        assert_eq!(stats.inserts, 1);
        assert_eq!(stats.cold_hits, 1);
        assert_eq!(stats.warm_hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.promotions >= 1);
    }

    #[test]
    fn test_cache_default_config() {
        let config = TieredCacheConfig::default();

        assert_eq!(config.hot_capacity, 100);
        assert_eq!(config.warm_capacity, 500);
        assert_eq!(config.cold_capacity, 2000);
        assert_eq!(config.promotion_threshold, 3);
    }

    #[test]
    fn test_cache_with_defaults() {
        let cache: TieredCache<String, String> = TieredCache::with_defaults();

        assert_eq!(cache.config().hot_capacity, 100);
        assert_eq!(cache.config().warm_capacity, 500);
        assert_eq!(cache.config().cold_capacity, 2000);
    }
}
