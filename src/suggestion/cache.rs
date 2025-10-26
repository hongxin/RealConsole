//! 建议缓存模块
//!
//! 为快速执行建议提供缓存支持，包含：
//! - 时间戳管理
//! - 过期检查
//! - 自动清理
//!
//! ## 设计理念
//!
//! 遵循"一分为三"哲学：
//! - **新鲜** (< 2分钟)：高置信度，可直接使用
//! - **陈旧** (2-5分钟)：中等置信度，提示用户
//! - **过期** (> 5分钟)：低置信度，自动清除
//!
//! ## 使用示例
//!
//! ```rust
//! use realconsole::suggestion::{SuggestionCache, Suggestion, SuggestionSource};
//! use std::time::Duration;
//!
//! let mut cache = SuggestionCache::new(Duration::from_secs(300));
//!
//! // 添加建议
//! let suggestions = vec![
//!     Suggestion::new("cargo build", "Build project", 0.9, SuggestionSource::Context),
//! ];
//! cache.update(suggestions);
//!
//! // 获取建议（如果未过期）
//! if let Some(cached) = cache.get() {
//!     println!("Found {} cached suggestions", cached.len());
//! }
//! ```

use super::types::Suggestion;
use std::time::{Duration, Instant};

/// 建议缓存
///
/// 存储最近显示的建议，支持过期检查和自动清理
pub struct SuggestionCache {
    /// 缓存的建议列表
    suggestions: Vec<Suggestion>,

    /// 缓存创建时间
    timestamp: Option<Instant>,

    /// 最大缓存时间（超过此时间视为过期）
    max_age: Duration,

    /// 警告时间（超过此时间给出陈旧警告）
    warn_age: Duration,
}

impl SuggestionCache {
    /// 创建新的建议缓存
    ///
    /// # 参数
    /// - `max_age`: 最大缓存时间（默认 5 分钟）
    pub fn new(max_age: Duration) -> Self {
        Self {
            suggestions: Vec::new(),
            timestamp: None,
            max_age,
            warn_age: max_age / 2, // 警告时间为最大时间的一半
        }
    }

    /// 使用默认配置创建缓存（5分钟过期）
    pub fn with_default_config() -> Self {
        Self::new(Duration::from_secs(300)) // 5 分钟
    }

    /// 更新缓存
    ///
    /// 保存新的建议列表并更新时间戳
    pub fn update(&mut self, suggestions: Vec<Suggestion>) {
        self.suggestions = suggestions;
        self.timestamp = Some(Instant::now());
    }

    /// 获取缓存的建议
    ///
    /// 如果缓存已过期，返回 None 并清空缓存
    pub fn get(&mut self) -> Option<&[Suggestion]> {
        if self.is_expired() {
            self.clear();
            None
        } else if self.suggestions.is_empty() {
            None
        } else {
            Some(&self.suggestions)
        }
    }

    /// 获取指定索引的建议
    ///
    /// # 返回
    /// - `Some(&Suggestion)`: 如果索引有效且缓存未过期
    /// - `None`: 如果索引无效或缓存已过期
    pub fn get_by_index(&mut self, index: usize) -> Option<&Suggestion> {
        self.get()?.get(index)
    }

    /// 检查缓存是否已过期
    pub fn is_expired(&self) -> bool {
        match self.timestamp {
            Some(ts) => ts.elapsed() > self.max_age,
            None => true, // 没有时间戳视为已过期
        }
    }

    /// 检查缓存是否陈旧（接近过期）
    ///
    /// 用于向用户发出警告
    pub fn is_stale(&self) -> bool {
        match self.timestamp {
            Some(ts) => {
                let age = ts.elapsed();
                age > self.warn_age && age <= self.max_age
            }
            None => false,
        }
    }

    /// 获取缓存年龄
    pub fn age(&self) -> Option<Duration> {
        self.timestamp.map(|ts| ts.elapsed())
    }

    /// 获取缓存的建议数量
    pub fn len(&self) -> usize {
        self.suggestions.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.suggestions.is_empty()
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.suggestions.clear();
        self.timestamp = None;
    }

    /// 获取缓存状态的人类可读描述
    pub fn status(&self) -> CacheStatus {
        if self.is_empty() {
            CacheStatus::Empty
        } else if self.is_expired() {
            CacheStatus::Expired {
                age: self.age().unwrap(),
            }
        } else if self.is_stale() {
            CacheStatus::Stale {
                age: self.age().unwrap(),
            }
        } else {
            CacheStatus::Fresh {
                age: self.age().unwrap(),
                count: self.len(),
            }
        }
    }
}

impl Default for SuggestionCache {
    fn default() -> Self {
        Self::with_default_config()
    }
}

/// 缓存状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    /// 缓存为空
    Empty,

    /// 缓存新鲜（可安全使用）
    Fresh { age: Duration, count: usize },

    /// 缓存陈旧（接近过期，建议提示用户）
    Stale { age: Duration },

    /// 缓存已过期
    Expired { age: Duration },
}

impl CacheStatus {
    /// 获取状态的描述
    pub fn description(&self) -> String {
        match self {
            CacheStatus::Empty => "No cached suggestions".to_string(),
            CacheStatus::Fresh { age, count } => {
                format!("{} suggestions ({}s ago)", count, age.as_secs())
            }
            CacheStatus::Stale { age } => {
                format!("Suggestions may be outdated ({}s ago)", age.as_secs())
            }
            CacheStatus::Expired { age } => {
                format!("Suggestions expired ({}s ago)", age.as_secs())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::suggestion::SuggestionSource;
    use std::thread::sleep;

    #[test]
    fn test_cache_creation() {
        let cache = SuggestionCache::new(Duration::from_secs(60));
        assert!(cache.is_empty());
        assert!(cache.is_expired()); // 没有时间戳视为已过期
    }

    #[test]
    fn test_cache_update_and_get() {
        let mut cache = SuggestionCache::new(Duration::from_secs(60));

        let suggestions = vec![
            Suggestion::new("cargo build", "Build project", 0.9, SuggestionSource::Context),
            Suggestion::new("cargo test", "Run tests", 0.8, SuggestionSource::Context),
        ];

        cache.update(suggestions.clone());

        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert!(!cache.is_expired());

        let cached = cache.get().unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].command, "cargo build");
    }

    #[test]
    fn test_cache_get_by_index() {
        let mut cache = SuggestionCache::new(Duration::from_secs(60));

        let suggestions = vec![
            Suggestion::new("cmd1", "First", 0.9, SuggestionSource::Context),
            Suggestion::new("cmd2", "Second", 0.8, SuggestionSource::Context),
        ];

        cache.update(suggestions);

        assert_eq!(cache.get_by_index(0).unwrap().command, "cmd1");
        assert_eq!(cache.get_by_index(1).unwrap().command, "cmd2");
        assert!(cache.get_by_index(2).is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = SuggestionCache::new(Duration::from_millis(100));

        let suggestions = vec![
            Suggestion::new("test", "Test", 0.9, SuggestionSource::Context),
        ];

        cache.update(suggestions);
        assert!(!cache.is_expired());

        // 等待过期
        sleep(Duration::from_millis(150));

        assert!(cache.is_expired());
        assert!(cache.get().is_none()); // 过期后返回 None
        assert!(cache.is_empty()); // 并且自动清空
    }

    #[test]
    fn test_cache_staleness() {
        let mut cache = SuggestionCache::new(Duration::from_secs(10));

        let suggestions = vec![
            Suggestion::new("test", "Test", 0.9, SuggestionSource::Context),
        ];

        cache.update(suggestions);
        assert!(!cache.is_stale());

        // 修改时间戳以模拟陈旧状态（超过警告时间但未过期）
        cache.timestamp = Some(Instant::now() - Duration::from_secs(6));

        assert!(cache.is_stale());
        assert!(!cache.is_expired());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = SuggestionCache::new(Duration::from_secs(60));

        let suggestions = vec![
            Suggestion::new("test", "Test", 0.9, SuggestionSource::Context),
        ];

        cache.update(suggestions);
        assert!(!cache.is_empty());

        cache.clear();
        assert!(cache.is_empty());
        assert!(cache.is_expired());
    }

    #[test]
    fn test_cache_status() {
        let mut cache = SuggestionCache::new(Duration::from_secs(10));

        // 空缓存
        assert!(matches!(cache.status(), CacheStatus::Empty));

        // 新鲜缓存
        let suggestions = vec![
            Suggestion::new("test", "Test", 0.9, SuggestionSource::Context),
        ];
        cache.update(suggestions);
        assert!(matches!(
            cache.status(),
            CacheStatus::Fresh { .. }
        ));

        // 陈旧缓存
        cache.timestamp = Some(Instant::now() - Duration::from_secs(6));
        assert!(matches!(cache.status(), CacheStatus::Stale { .. }));

        // 过期缓存
        cache.timestamp = Some(Instant::now() - Duration::from_secs(11));
        assert!(matches!(cache.status(), CacheStatus::Expired { .. }));
    }

    #[test]
    fn test_cache_age() {
        let mut cache = SuggestionCache::new(Duration::from_secs(60));

        assert!(cache.age().is_none());

        let suggestions = vec![
            Suggestion::new("test", "Test", 0.9, SuggestionSource::Context),
        ];
        cache.update(suggestions);

        let age = cache.age().unwrap();
        assert!(age < Duration::from_secs(1)); // 刚创建，应该很短
    }

    #[test]
    fn test_cache_status_description() {
        let status = CacheStatus::Empty;
        assert!(status.description().contains("No cached"));

        let status = CacheStatus::Fresh {
            age: Duration::from_secs(30),
            count: 3,
        };
        assert!(status.description().contains("3 suggestions"));
        assert!(status.description().contains("30s"));

        let status = CacheStatus::Stale {
            age: Duration::from_secs(120),
        };
        assert!(status.description().contains("outdated"));

        let status = CacheStatus::Expired {
            age: Duration::from_secs(600),
        };
        assert!(status.description().contains("expired"));
    }
}
