//! 工具响应缓存系统
//!
//! Phase 5.3 Week 3 Day 2: 性能优化
//!
//! 功能：
//! - LRU 缓存工具执行结果
//! - 仅缓存幂等工具（可配置）
//! - 自动过期机制（TTL）
//! - 缓存统计（命中率、大小等）

use lru::LruCache;
use serde_json::Value as JsonValue;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的结果
    result: String,

    /// 创建时间
    created_at: Instant,

    /// TTL（生存时间）
    ttl: Duration,
}

impl CacheEntry {
    /// 检查是否过期
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

/// 缓存键（工具名 + 参数的哈希）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    tool_name: String,
    args_hash: String,
}

impl CacheKey {
    fn new(tool_name: String, arguments: &JsonValue) -> Self {
        // 使用 JSON 序列化作为参数的哈希键
        let args_hash = serde_json::to_string(arguments).unwrap_or_default();
        Self {
            tool_name,
            args_hash,
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 总请求数
    pub total_requests: u64,

    /// 缓存命中数
    pub hits: u64,

    /// 缓存未命中数
    pub misses: u64,

    /// 过期条目数
    pub expired: u64,
}

impl CacheStats {
    /// 计算命中率（百分比）
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.hits as f64 / self.total_requests as f64) * 100.0
        }
    }
}

/// 工具缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 缓存容量（条目数）
    pub capacity: usize,

    /// 默认 TTL
    pub default_ttl: Duration,

    /// 启用缓存的工具列表（None 表示全部启用）
    pub enabled_tools: Option<Vec<String>>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            capacity: 100,                           // 默认缓存 100 条
            default_ttl: Duration::from_secs(300),   // 默认 5 分钟过期
            enabled_tools: None,                      // 默认全部启用
        }
    }
}

/// 工具响应缓存
pub struct ToolCache {
    /// LRU 缓存
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,

    /// 配置
    config: CacheConfig,

    /// 统计信息
    stats: Arc<RwLock<CacheStats>>,
}

impl ToolCache {
    /// 创建新的工具缓存
    pub fn new(config: CacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.capacity).unwrap_or(NonZeroUsize::new(100).unwrap());

        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// 检查工具是否启用缓存
    fn is_tool_cacheable(&self, tool_name: &str) -> bool {
        match &self.config.enabled_tools {
            None => true, // 默认全部启用
            Some(tools) => tools.contains(&tool_name.to_string()),
        }
    }

    /// 获取缓存（如果存在且未过期）
    pub async fn get(&self, tool_name: &str, arguments: &JsonValue) -> Option<String> {
        // 检查工具是否启用缓存
        if !self.is_tool_cacheable(tool_name) {
            return None;
        }

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;

        // 查找缓存
        let key = CacheKey::new(tool_name.to_string(), arguments);
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get(&key) {
            // 检查是否过期
            if entry.is_expired() {
                stats.expired += 1;
                stats.misses += 1;
                cache.pop(&key); // 删除过期条目
                None
            } else {
                stats.hits += 1;
                Some(entry.result.clone())
            }
        } else {
            stats.misses += 1;
            None
        }
    }

    /// 设置缓存
    pub async fn set(&self, tool_name: &str, arguments: &JsonValue, result: String) {
        // 检查工具是否启用缓存
        if !self.is_tool_cacheable(tool_name) {
            return;
        }

        let key = CacheKey::new(tool_name.to_string(), arguments);
        let entry = CacheEntry {
            result,
            created_at: Instant::now(),
            ttl: self.config.default_ttl,
        };

        let mut cache = self.cache.write().await;
        cache.put(key, entry);
    }

    /// 清空缓存
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// 获取缓存统计
    pub async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }

    /// 重置统计
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }

    /// 获取当前缓存大小
    pub async fn size(&self) -> usize {
        self.cache.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_basic() {
        let cache = ToolCache::with_defaults();

        let tool_name = "calculator";
        let args = json!({"expression": "2+2"});
        let result = "result: 4".to_string();

        // 首次获取应该未命中
        assert!(cache.get(tool_name, &args).await.is_none());

        // 设置缓存
        cache.set(tool_name, &args, result.clone()).await;

        // 再次获取应该命中
        let cached = cache.get(tool_name, &args).await;
        assert_eq!(cached, Some(result));
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig {
            capacity: 10,
            default_ttl: Duration::from_millis(100), // 100ms 过期
            enabled_tools: None,
        };
        let cache = ToolCache::new(config);

        let tool_name = "test";
        let args = json!({"a": 1});
        let result = "test result".to_string();

        // 设置缓存
        cache.set(tool_name, &args, result.clone()).await;

        // 立即获取应该命中
        assert_eq!(cache.get(tool_name, &args).await, Some(result));

        // 等待过期
        tokio::time::sleep(Duration::from_millis(150)).await;

        // 过期后应该未命中
        assert!(cache.get(tool_name, &args).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = ToolCache::with_defaults();

        let tool_name = "test";
        let args1 = json!({"x": 1});
        let args2 = json!({"x": 2});

        // 设置缓存
        cache.set(tool_name, &args1, "result1".to_string()).await;
        cache.set(tool_name, &args2, "result2".to_string()).await;

        // 命中
        cache.get(tool_name, &args1).await;
        cache.get(tool_name, &args1).await;

        // 未命中
        cache.get(tool_name, &json!({"x": 3})).await;

        let stats = cache.stats().await;
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);

        // 使用近似比较来避免浮点数精度问题
        let expected_rate = 200.0 / 3.0;
        assert!((stats.hit_rate() - expected_rate).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_cache_different_args() {
        let cache = ToolCache::with_defaults();

        let tool_name = "calculator";

        // 设置两个不同参数的缓存
        cache.set(tool_name, &json!({"a": 1}), "result1".to_string()).await;
        cache.set(tool_name, &json!({"a": 2}), "result2".to_string()).await;

        // 应该能正确区分
        assert_eq!(
            cache.get(tool_name, &json!({"a": 1})).await,
            Some("result1".to_string())
        );
        assert_eq!(
            cache.get(tool_name, &json!({"a": 2})).await,
            Some("result2".to_string())
        );
    }

    #[tokio::test]
    async fn test_cache_enabled_tools() {
        let config = CacheConfig {
            capacity: 10,
            default_ttl: Duration::from_secs(60),
            enabled_tools: Some(vec!["calculator".to_string(), "datetime".to_string()]),
        };
        let cache = ToolCache::new(config);

        // 启用缓存的工具
        cache.set("calculator", &json!({"a": 1}), "result1".to_string()).await;
        assert_eq!(
            cache.get("calculator", &json!({"a": 1})).await,
            Some("result1".to_string())
        );

        // 未启用缓存的工具
        cache.set("file_read", &json!({"path": "/tmp"}), "result2".to_string()).await;
        assert!(cache.get("file_read", &json!({"path": "/tmp"})).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_lru_eviction() {
        let config = CacheConfig {
            capacity: 2, // 只能缓存 2 条
            default_ttl: Duration::from_secs(60),
            enabled_tools: None,
        };
        let cache = ToolCache::new(config);

        // 添加 3 条缓存
        cache.set("tool", &json!({"a": 1}), "result1".to_string()).await;
        cache.set("tool", &json!({"a": 2}), "result2".to_string()).await;
        cache.set("tool", &json!({"a": 3}), "result3".to_string()).await; // 应该淘汰第一条

        // 第一条应该被淘汰
        assert!(cache.get("tool", &json!({"a": 1})).await.is_none());

        // 第二、三条应该还在
        assert_eq!(
            cache.get("tool", &json!({"a": 2})).await,
            Some("result2".to_string())
        );
        assert_eq!(
            cache.get("tool", &json!({"a": 3})).await,
            Some("result3".to_string())
        );
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = ToolCache::with_defaults();

        cache.set("tool", &json!({"a": 1}), "result1".to_string()).await;
        cache.set("tool", &json!({"a": 2}), "result2".to_string()).await;

        assert_eq!(cache.size().await, 2);

        cache.clear().await;

        assert_eq!(cache.size().await, 0);
        assert!(cache.get("tool", &json!({"a": 1})).await.is_none());
    }
}
