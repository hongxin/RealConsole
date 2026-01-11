//! v1.88.0: LLM 响应缓存系统
//!
//! 提供 LLM 响应的智能缓存，减少重复 API 调用：
//! - **查询哈希**: 基于消息内容生成唯一缓存键
//! - **TTL 过期**: 可配置的生存时间
//! - **LRU 淘汰**: 超出容量时淘汰最少使用的条目
//! - **统计监控**: 命中率、缓存大小等指标
//!
//! # 设计原则
//!
//! 1. **透明缓存**: 作为 LlmClient 的包装器，对调用者透明
//! 2. **智能过期**: 支持滑动过期和固定过期
//! 3. **安全排除**: 不缓存工具调用、错误响应

use super::{ChatResponse, LlmClient, LlmError, Message, ClientStats};
use async_trait::async_trait;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的响应内容
    response: String,

    /// 创建时间
    created_at: Instant,

    /// 最后访问时间（用于滑动过期）
    last_accessed: Instant,

    /// TTL（生存时间）
    ttl: Duration,

    /// 是否使用滑动过期
    sliding_expiration: bool,
}

impl CacheEntry {
    /// 创建新的缓存条目
    fn new(response: String, ttl: Duration, sliding_expiration: bool) -> Self {
        let now = Instant::now();
        Self {
            response,
            created_at: now,
            last_accessed: now,
            ttl,
            sliding_expiration,
        }
    }

    /// 检查是否过期
    fn is_expired(&self) -> bool {
        if self.sliding_expiration {
            self.last_accessed.elapsed() > self.ttl
        } else {
            self.created_at.elapsed() > self.ttl
        }
    }

    /// 更新访问时间
    fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

/// 缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    /// 模型名称
    model: String,
    /// 消息内容哈希
    messages_hash: u64,
}

impl CacheKey {
    /// 从消息列表创建缓存键
    fn from_messages(model: &str, messages: &[Message]) -> Self {
        let mut hasher = DefaultHasher::new();

        // 哈希模型名称
        model.hash(&mut hasher);

        // 哈希每条消息的内容
        for msg in messages {
            // 角色
            format!("{:?}", msg.role).hash(&mut hasher);

            // 内容
            if let Some(ref content) = msg.content {
                content.hash(&mut hasher);
            }

            // 不包含 tool_calls 和 tool_call_id，因为这些不应该被缓存
        }

        Self {
            model: model.to_string(),
            messages_hash: hasher.finish(),
        }
    }
}

/// 缓存统计（线程安全）
#[derive(Debug, Clone)]
pub struct LlmCacheStats {
    /// 总请求数
    total_requests: Arc<AtomicU64>,
    /// 缓存命中数
    hits: Arc<AtomicU64>,
    /// 缓存未命中数
    misses: Arc<AtomicU64>,
    /// 过期条目数
    expired: Arc<AtomicU64>,
    /// 缓存写入数
    writes: Arc<AtomicU64>,
}

impl Default for LlmCacheStats {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmCacheStats {
    /// 创建新的统计实例
    pub fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicU64::new(0)),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            expired: Arc::new(AtomicU64::new(0)),
            writes: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 记录命中
    fn record_hit(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录未命中
    fn record_miss(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录过期
    fn record_expired(&self) {
        self.expired.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录写入
    fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取总请求数
    pub fn total_requests(&self) -> u64 {
        self.total_requests.load(Ordering::Relaxed)
    }

    /// 获取命中数
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// 获取未命中数
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// 获取过期数
    pub fn expired(&self) -> u64 {
        self.expired.load(Ordering::Relaxed)
    }

    /// 获取写入数
    pub fn writes(&self) -> u64 {
        self.writes.load(Ordering::Relaxed)
    }

    /// 计算命中率（百分比）
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            (self.hits() as f64 / total as f64) * 100.0
        }
    }
}

/// LLM 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCacheConfig {
    /// 是否启用缓存
    pub enabled: bool,

    /// 缓存容量（条目数）
    pub capacity: usize,

    /// 默认 TTL（秒）
    pub default_ttl_secs: u64,

    /// 是否使用滑动过期
    pub sliding_expiration: bool,

    /// 最小消息长度（太短的消息可能不值得缓存）
    pub min_message_length: usize,

    /// 最大响应长度（太长的响应可能占用过多内存）
    pub max_response_length: usize,
}

impl Default for LlmCacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            capacity: 500,                    // 缓存 500 条
            default_ttl_secs: 3600,           // 1 小时过期
            sliding_expiration: false,        // 固定过期
            min_message_length: 5,            // 最少 5 字符
            max_response_length: 50000,       // 最大 50KB
        }
    }
}

impl LlmCacheConfig {
    /// 创建禁用缓存的配置
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// 创建高命中率配置（更长 TTL，更大容量）
    pub fn high_hit_rate() -> Self {
        Self {
            enabled: true,
            capacity: 1000,
            default_ttl_secs: 86400,  // 24 小时
            sliding_expiration: true,
            min_message_length: 3,
            max_response_length: 100000,
        }
    }

    /// 创建低内存配置（更短 TTL，更小容量）
    pub fn low_memory() -> Self {
        Self {
            enabled: true,
            capacity: 100,
            default_ttl_secs: 900,  // 15 分钟
            sliding_expiration: false,
            min_message_length: 10,
            max_response_length: 10000,
        }
    }
}

/// 缓存包装的 LLM 客户端
///
/// 包装任意 LlmClient，添加响应缓存功能。
/// 透明地缓存 chat() 调用的响应，不缓存 chat_with_tools()。
pub struct CachedLlmClient<C: LlmClient> {
    /// 内部客户端
    inner: Arc<C>,

    /// 缓存存储
    cache: Arc<RwLock<LruCache<CacheKey, CacheEntry>>>,

    /// 缓存配置
    config: LlmCacheConfig,

    /// 缓存统计
    stats: LlmCacheStats,
}

impl<C: LlmClient> CachedLlmClient<C> {
    /// 创建新的缓存客户端
    pub fn new(client: C, config: LlmCacheConfig) -> Self {
        let capacity = NonZeroUsize::new(config.capacity).unwrap_or(NonZeroUsize::new(100).unwrap());

        Self {
            inner: Arc::new(client),
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            config,
            stats: LlmCacheStats::new(),
        }
    }

    /// 使用默认配置创建
    pub fn with_defaults(client: C) -> Self {
        Self::new(client, LlmCacheConfig::default())
    }

    /// 获取缓存统计
    pub fn cache_stats(&self) -> &LlmCacheStats {
        &self.stats
    }

    /// 获取缓存大小
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }

    /// 清空缓存
    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }

    /// 检查消息是否应该被缓存
    fn should_cache(&self, messages: &[Message]) -> bool {
        if !self.config.enabled {
            return false;
        }

        // 检查消息列表不为空
        if messages.is_empty() {
            return false;
        }

        // 检查最后一条消息是否包含工具调用相关内容
        if let Some(last_msg) = messages.last() {
            // 如果是工具结果消息，不缓存
            if last_msg.tool_call_id.is_some() {
                return false;
            }
            // 如果消息包含工具调用，不缓存
            if last_msg.tool_calls.is_some() {
                return false;
            }
        }

        // 检查消息长度
        let total_length: usize = messages
            .iter()
            .filter_map(|m| m.content.as_ref())
            .map(|c| c.len())
            .sum();

        total_length >= self.config.min_message_length
    }

    /// 检查响应是否应该被缓存
    fn should_cache_response(&self, response: &str) -> bool {
        !response.is_empty() && response.len() <= self.config.max_response_length
    }
}

#[async_trait]
impl<C: LlmClient + 'static> LlmClient for CachedLlmClient<C> {
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
        // 检查是否应该使用缓存
        if !self.should_cache(&messages) {
            return self.inner.chat(messages).await;
        }

        let cache_key = CacheKey::from_messages(self.inner.model(), &messages);

        // 尝试从缓存获取
        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(&cache_key) {
                if entry.is_expired() {
                    // 过期，移除
                    cache.pop(&cache_key);
                    self.stats.record_expired();
                } else {
                    // 命中
                    entry.touch();
                    self.stats.record_hit();
                    return Ok(entry.response.clone());
                }
            }
        }

        // 缓存未命中，调用实际客户端
        self.stats.record_miss();
        let response = self.inner.chat(messages).await?;

        // 缓存响应
        if self.should_cache_response(&response) {
            let entry = CacheEntry::new(
                response.clone(),
                Duration::from_secs(self.config.default_ttl_secs),
                self.config.sliding_expiration,
            );

            let mut cache = self.cache.write().await;
            cache.put(cache_key, entry);
            self.stats.record_write();
        }

        Ok(response)
    }

    async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse, LlmError> {
        // 工具调用不缓存，因为结果可能依赖于实时状态
        self.inner.chat_with_tools(messages, tools).await
    }

    fn model(&self) -> &str {
        self.inner.model()
    }

    fn stats(&self) -> ClientStats {
        self.inner.stats()
    }

    async fn diagnose(&self) -> String {
        let inner_diag = self.inner.diagnose().await;
        let cache_info = format!(
            "\n[Cache] Size: {}, Hit Rate: {:.1}%, Hits: {}, Misses: {}",
            self.cache.read().await.len(),
            self.stats.hit_rate(),
            self.stats.hits(),
            self.stats.misses()
        );
        format!("{}{}", inner_diag, cache_info)
    }
}

// 为 Arc<CachedLlmClient<C>> 实现 LlmClient
#[async_trait]
impl<C: LlmClient + 'static> LlmClient for Arc<CachedLlmClient<C>> {
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
        (**self).chat(messages).await
    }

    async fn chat_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
    ) -> Result<ChatResponse, LlmError> {
        (**self).chat_with_tools(messages, tools).await
    }

    fn model(&self) -> &str {
        (**self).model()
    }

    fn stats(&self) -> ClientStats {
        (**self).stats()
    }

    async fn diagnose(&self) -> String {
        (**self).diagnose().await
    }
}

/// 缓存预热工具
///
/// 用于预加载常见查询的缓存
pub struct CacheWarmer<C: LlmClient> {
    client: Arc<CachedLlmClient<C>>,
}

impl<C: LlmClient + 'static> CacheWarmer<C> {
    /// 创建缓存预热器
    pub fn new(client: Arc<CachedLlmClient<C>>) -> Self {
        Self { client }
    }

    /// 预热常见查询
    ///
    /// 传入常见的查询列表，提前填充缓存
    pub async fn warm_up(&self, queries: &[&str]) -> usize {
        let mut warmed = 0;

        for query in queries {
            let messages = vec![Message::user(*query)];
            if let Ok(_) = self.client.chat(messages).await {
                warmed += 1;
            }
        }

        warmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    // 模拟 LLM 客户端用于测试
    struct MockLlmClient {
        model_name: String,
        call_count: Arc<AtomicUsize>,
        response: String,
    }

    impl MockLlmClient {
        fn new(model: &str, response: &str) -> Self {
            Self {
                model_name: model.to_string(),
                call_count: Arc::new(AtomicUsize::new(0)),
                response: response.to_string(),
            }
        }

        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn chat(&self, _messages: Vec<Message>) -> Result<String, LlmError> {
            self.call_count.fetch_add(1, Ordering::Relaxed);
            Ok(self.response.clone())
        }

        fn model(&self) -> &str {
            &self.model_name
        }

        fn stats(&self) -> ClientStats {
            ClientStats::new()
        }

        async fn diagnose(&self) -> String {
            "Mock client OK".to_string()
        }
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let mock = MockLlmClient::new("test-model", "Hello!");
        let cached = CachedLlmClient::with_defaults(mock);

        // 第一次调用 - 未命中（使用足够长的消息以满足 min_message_length）
        let messages = vec![Message::user("Hello, how are you today?")];
        let result1 = cached.chat(messages.clone()).await.unwrap();
        assert_eq!(result1, "Hello!");
        assert_eq!(cached.cache_stats().hits(), 0);
        assert_eq!(cached.cache_stats().misses(), 1);

        // 第二次调用 - 命中
        let result2 = cached.chat(messages.clone()).await.unwrap();
        assert_eq!(result2, "Hello!");
        assert_eq!(cached.cache_stats().hits(), 1);
        assert_eq!(cached.cache_stats().misses(), 1);

        // 内部客户端只被调用一次
        assert_eq!(cached.inner.call_count(), 1);
    }

    #[tokio::test]
    async fn test_cache_key_different_messages() {
        let mock = MockLlmClient::new("test-model", "Response");
        let cached = CachedLlmClient::with_defaults(mock);

        // 不同的消息应该有不同的缓存键（使用足够长的消息）
        let msg1 = vec![Message::user("Hello, this is message one")];
        let msg2 = vec![Message::user("World, this is message two")];

        cached.chat(msg1.clone()).await.unwrap();
        cached.chat(msg2.clone()).await.unwrap();

        // 两次都是未命中
        assert_eq!(cached.cache_stats().misses(), 2);

        // 再次查询应该命中
        cached.chat(msg1).await.unwrap();
        cached.chat(msg2).await.unwrap();
        assert_eq!(cached.cache_stats().hits(), 2);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let mock = MockLlmClient::new("test-model", "Response");
        let config = LlmCacheConfig {
            enabled: true,
            capacity: 100,
            default_ttl_secs: 0, // 立即过期
            sliding_expiration: false,
            min_message_length: 1,
            max_response_length: 10000,
        };
        let cached = CachedLlmClient::new(mock, config);

        let messages = vec![Message::user("Test message for expiration")];

        // 第一次调用
        cached.chat(messages.clone()).await.unwrap();

        // 等待一小段时间确保过期
        tokio::time::sleep(Duration::from_millis(10)).await;

        // 第二次调用 - 应该过期
        cached.chat(messages).await.unwrap();

        // 应该有 1 个过期记录
        assert_eq!(cached.cache_stats().expired(), 1);
    }

    #[tokio::test]
    async fn test_disabled_cache() {
        let mock = MockLlmClient::new("test-model", "Response");
        let config = LlmCacheConfig::disabled();
        let cached = CachedLlmClient::new(mock, config);

        let messages = vec![Message::user("Test")];

        cached.chat(messages.clone()).await.unwrap();
        cached.chat(messages).await.unwrap();

        // 缓存禁用时，每次都应该调用内部客户端
        assert_eq!(cached.inner.call_count(), 2);
        assert_eq!(cached.cache_stats().total_requests(), 0);
    }

    #[tokio::test]
    async fn test_skip_short_messages() {
        let mock = MockLlmClient::new("test-model", "Response");
        let config = LlmCacheConfig {
            min_message_length: 100, // 需要至少 100 字符
            ..Default::default()
        };
        let cached = CachedLlmClient::new(mock, config);

        // 短消息不应该被缓存
        let messages = vec![Message::user("Hi")];
        cached.chat(messages.clone()).await.unwrap();
        cached.chat(messages).await.unwrap();

        // 每次都调用内部客户端
        assert_eq!(cached.inner.call_count(), 2);
    }

    #[tokio::test]
    async fn test_skip_tool_messages() {
        let mock = MockLlmClient::new("test-model", "Response");
        let cached = CachedLlmClient::with_defaults(mock);

        // 带工具调用 ID 的消息不应该被缓存
        let messages = vec![Message {
            role: super::super::MessageRole::Tool,
            content: Some("Tool result".to_string()),
            tool_calls: None,
            tool_call_id: Some("call_123".to_string()),
        }];

        cached.chat(messages.clone()).await.unwrap();
        cached.chat(messages).await.unwrap();

        // 每次都调用内部客户端
        assert_eq!(cached.inner.call_count(), 2);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let mock = MockLlmClient::new("test-model", "Response");
        let cached = CachedLlmClient::with_defaults(mock);

        let messages = vec![Message::user("Test message")];
        cached.chat(messages.clone()).await.unwrap();
        assert_eq!(cached.cache_size().await, 1);

        cached.clear_cache().await;
        assert_eq!(cached.cache_size().await, 0);
    }

    #[tokio::test]
    async fn test_hit_rate() {
        let stats = LlmCacheStats::new();

        assert_eq!(stats.hit_rate(), 0.0);

        // 1 命中，3 未命中 = 25% 命中率
        stats.record_hit();
        stats.record_miss();
        stats.record_miss();
        stats.record_miss();

        assert!((stats.hit_rate() - 25.0).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_cache_key_consistency() {
        let key1 = CacheKey::from_messages("model", &[Message::user("Hello")]);
        let key2 = CacheKey::from_messages("model", &[Message::user("Hello")]);
        let key3 = CacheKey::from_messages("model", &[Message::user("World")]);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_diagnose_includes_cache_info() {
        let mock = MockLlmClient::new("test-model", "Response");
        let cached = CachedLlmClient::with_defaults(mock);

        // 产生一些缓存活动
        let messages = vec![Message::user("Test")];
        cached.chat(messages.clone()).await.unwrap();
        cached.chat(messages).await.unwrap();

        let diag = cached.diagnose().await;
        assert!(diag.contains("[Cache]"));
        assert!(diag.contains("Hit Rate"));
    }

    #[test]
    fn test_config_presets() {
        let default = LlmCacheConfig::default();
        assert!(default.enabled);
        assert_eq!(default.capacity, 500);

        let disabled = LlmCacheConfig::disabled();
        assert!(!disabled.enabled);

        let high_hit = LlmCacheConfig::high_hit_rate();
        assert!(high_hit.sliding_expiration);
        assert_eq!(high_hit.capacity, 1000);

        let low_mem = LlmCacheConfig::low_memory();
        assert_eq!(low_mem.capacity, 100);
    }
}
