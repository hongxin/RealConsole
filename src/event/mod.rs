//! 统一事件系统
//!
//! v1.106.0 新增：发布-订阅事件总线
//!
//! # 功能特性
//! - 事件发布与订阅
//! - 事件过滤与路由
//! - 异步事件处理
//! - 事件历史记录
//!
//! # 架构设计
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              EventBus                    │
//! ├─────────────────────────────────────────┤
//! │  Publishers ──► Topics ──► Subscribers  │
//! │                   │                      │
//! │              ┌────▼────┐                │
//! │              │ Filters │                │
//! │              └────┬────┘                │
//! │                   │                      │
//! │              ┌────▼────┐                │
//! │              │Handlers │                │
//! │              └─────────┘                │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # 使用示例
//! ```ignore
//! use crate::event::{EventBus, Event, EventHandler};
//!
//! let mut bus = EventBus::new();
//!
//! // 订阅事件
//! bus.subscribe("command.*", handler);
//!
//! // 发布事件
//! bus.publish(Event::new("command.executed", data)).await;
//! ```

mod bus;
mod handler;

pub use bus::{EventBus, EventBusConfig, Subscription, SubscriptionId};
pub use handler::{EventHandler, HandlerResult, AsyncEventHandler};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use uuid::Uuid;

/// 事件 ID
pub type EventId = String;

/// 事件主题
pub type Topic = String;

/// 事件优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventPriority {
    /// 低优先级
    Low = 0,
    /// 普通优先级
    #[default]
    Normal = 1,
    /// 高优先级
    High = 2,
    /// 紧急
    Critical = 3,
}

/// 事件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    /// 待处理
    #[default]
    Pending,
    /// 处理中
    Processing,
    /// 已处理
    Processed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
}

/// 事件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// 事件 ID
    pub id: EventId,
    /// 主题
    pub topic: Topic,
    /// 优先级
    pub priority: EventPriority,
    /// 状态
    pub status: EventStatus,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 处理时间
    pub processed_at: Option<DateTime<Utc>>,
    /// 来源
    pub source: Option<String>,
    /// 关联 ID（用于追踪）
    pub correlation_id: Option<String>,
    /// 自定义属性
    pub attributes: HashMap<String, String>,
}

impl EventMetadata {
    /// 创建新的元数据
    pub fn new(topic: impl Into<String>) -> Self {
        Self {
            id: format!("evt-{}", Uuid::new_v4()),
            topic: topic.into(),
            priority: EventPriority::Normal,
            status: EventStatus::Pending,
            created_at: Utc::now(),
            processed_at: None,
            source: None,
            correlation_id: None,
            attributes: HashMap::new(),
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置来源
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 设置关联 ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// 添加属性
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// 标记为处理中
    pub fn mark_processing(&mut self) {
        self.status = EventStatus::Processing;
    }

    /// 标记为已处理
    pub fn mark_processed(&mut self) {
        self.status = EventStatus::Processed;
        self.processed_at = Some(Utc::now());
    }

    /// 标记为失败
    pub fn mark_failed(&mut self) {
        self.status = EventStatus::Failed;
        self.processed_at = Some(Utc::now());
    }
}

/// 事件数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventData {
    /// 空数据
    #[default]
    None,
    /// 字符串
    String(String),
    /// 整数
    Integer(i64),
    /// 浮点数
    Float(f64),
    /// 布尔值
    Boolean(bool),
    /// JSON 对象
    Json(serde_json::Value),
    /// 字节数组
    Bytes(Vec<u8>),
}

impl EventData {
    /// 从 JSON 值创建
    pub fn from_json<T: Serialize>(value: &T) -> Self {
        serde_json::to_value(value)
            .map(EventData::Json)
            .unwrap_or(EventData::None)
    }

    /// 转换为 JSON 值
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            EventData::Json(v) => Some(v),
            _ => None,
        }
    }

    /// 转换为字符串
    pub fn as_string(&self) -> Option<&str> {
        match self {
            EventData::String(s) => Some(s),
            _ => None,
        }
    }

    /// 转换为整数
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            EventData::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// 是否为空
    pub fn is_none(&self) -> bool {
        matches!(self, EventData::None)
    }
}

/// 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// 元数据
    pub metadata: EventMetadata,
    /// 数据
    pub data: EventData,
}

impl Event {
    /// 创建新事件
    pub fn new(topic: impl Into<String>, data: EventData) -> Self {
        Self {
            metadata: EventMetadata::new(topic),
            data,
        }
    }

    /// 创建空事件
    pub fn empty(topic: impl Into<String>) -> Self {
        Self::new(topic, EventData::None)
    }

    /// 创建带字符串数据的事件
    pub fn with_string(topic: impl Into<String>, data: impl Into<String>) -> Self {
        Self::new(topic, EventData::String(data.into()))
    }

    /// 创建带 JSON 数据的事件
    pub fn with_json<T: Serialize>(topic: impl Into<String>, data: &T) -> Self {
        Self::new(topic, EventData::from_json(data))
    }

    /// 获取事件 ID
    pub fn id(&self) -> &str {
        &self.metadata.id
    }

    /// 获取主题
    pub fn topic(&self) -> &str {
        &self.metadata.topic
    }

    /// 获取优先级
    pub fn priority(&self) -> EventPriority {
        self.metadata.priority
    }

    /// 设置优先级
    pub fn set_priority(mut self, priority: EventPriority) -> Self {
        self.metadata.priority = priority;
        self
    }

    /// 设置来源
    pub fn set_source(mut self, source: impl Into<String>) -> Self {
        self.metadata.source = Some(source.into());
        self
    }

    /// 设置关联 ID
    pub fn set_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.metadata.correlation_id = Some(id.into());
        self
    }

    /// 添加属性
    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.attributes.insert(key.into(), value.into());
        self
    }
}

/// 事件过滤器
#[derive(Debug, Clone, Default)]
pub enum EventFilter {
    /// 匹配所有
    #[default]
    All,
    /// 主题匹配（支持通配符）
    Topic(String),
    /// 优先级匹配
    Priority(EventPriority),
    /// 来源匹配
    Source(String),
    /// 属性匹配
    Attribute { key: String, value: String },
    /// 组合过滤（AND）
    And(Vec<EventFilter>),
    /// 组合过滤（OR）
    Or(Vec<EventFilter>),
    /// 取反
    Not(Box<EventFilter>),
}

impl EventFilter {
    /// 检查事件是否匹配
    pub fn matches(&self, event: &Event) -> bool {
        match self {
            EventFilter::All => true,
            EventFilter::Topic(pattern) => Self::match_topic(pattern, event.topic()),
            EventFilter::Priority(p) => event.priority() == *p,
            EventFilter::Source(s) => event.metadata.source.as_deref() == Some(s.as_str()),
            EventFilter::Attribute { key, value } => {
                event.metadata.attributes.get(key).map(|v| v == value).unwrap_or(false)
            }
            EventFilter::And(filters) => filters.iter().all(|f| f.matches(event)),
            EventFilter::Or(filters) => filters.iter().any(|f| f.matches(event)),
            EventFilter::Not(f) => !f.matches(event),
        }
    }

    /// 主题匹配（支持 * 和 ** 通配符）
    fn match_topic(pattern: &str, topic: &str) -> bool {
        if pattern == "*" || pattern == "**" {
            return true;
        }

        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let topic_parts: Vec<&str> = topic.split('.').collect();

        Self::match_parts(&pattern_parts, &topic_parts)
    }

    fn match_parts(pattern: &[&str], topic: &[&str]) -> bool {
        if pattern.is_empty() {
            return topic.is_empty();
        }

        if topic.is_empty() {
            return pattern.iter().all(|p| *p == "*" || *p == "**");
        }

        match pattern[0] {
            "**" => {
                // ** 匹配零个或多个部分
                Self::match_parts(&pattern[1..], topic) ||
                Self::match_parts(pattern, &topic[1..])
            }
            "*" => {
                // * 匹配单个部分
                Self::match_parts(&pattern[1..], &topic[1..])
            }
            p => {
                if p == topic[0] {
                    Self::match_parts(&pattern[1..], &topic[1..])
                } else {
                    false
                }
            }
        }
    }
}

/// 预定义事件主题
pub mod topics {
    /// 命令相关
    pub const COMMAND_RECEIVED: &str = "command.received";
    pub const COMMAND_EXECUTED: &str = "command.executed";
    pub const COMMAND_FAILED: &str = "command.failed";

    /// LLM 相关
    pub const LLM_REQUEST: &str = "llm.request";
    pub const LLM_RESPONSE: &str = "llm.response";
    pub const LLM_ERROR: &str = "llm.error";
    pub const LLM_STREAM_START: &str = "llm.stream.start";
    pub const LLM_STREAM_TOKEN: &str = "llm.stream.token";
    pub const LLM_STREAM_END: &str = "llm.stream.end";

    /// 工具相关
    pub const TOOL_CALL: &str = "tool.call";
    pub const TOOL_RESULT: &str = "tool.result";
    pub const TOOL_ERROR: &str = "tool.error";

    /// 会话相关
    pub const SESSION_START: &str = "session.start";
    pub const SESSION_END: &str = "session.end";
    pub const SESSION_SAVE: &str = "session.save";
    pub const SESSION_LOAD: &str = "session.load";

    /// 系统相关
    pub const SYSTEM_STARTUP: &str = "system.startup";
    pub const SYSTEM_SHUTDOWN: &str = "system.shutdown";
    pub const SYSTEM_ERROR: &str = "system.error";

    /// 插件相关
    pub const PLUGIN_LOADED: &str = "plugin.loaded";
    pub const PLUGIN_ENABLED: &str = "plugin.enabled";
    pub const PLUGIN_DISABLED: &str = "plugin.disabled";
    pub const PLUGIN_ERROR: &str = "plugin.error";
}

/// 事件统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventStats {
    /// 发布的事件数
    pub published: u64,
    /// 处理的事件数
    pub processed: u64,
    /// 失败的事件数
    pub failed: u64,
    /// 订阅者数量
    pub subscribers: usize,
    /// 按主题统计
    pub by_topic: HashMap<String, u64>,
}

impl EventStats {
    /// 记录发布
    pub fn record_publish(&mut self, topic: &str) {
        self.published += 1;
        *self.by_topic.entry(topic.to_string()).or_insert(0) += 1;
    }

    /// 记录处理
    pub fn record_processed(&mut self) {
        self.processed += 1;
    }

    /// 记录失败
    pub fn record_failed(&mut self) {
        self.failed += 1;
    }

    /// 成功率
    pub fn success_rate(&self) -> f64 {
        if self.processed + self.failed == 0 {
            1.0
        } else {
            self.processed as f64 / (self.processed + self.failed) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_priority() {
        assert!(EventPriority::Critical > EventPriority::High);
        assert!(EventPriority::High > EventPriority::Normal);
        assert!(EventPriority::Normal > EventPriority::Low);
    }

    #[test]
    fn test_event_metadata() {
        let meta = EventMetadata::new("test.topic")
            .with_priority(EventPriority::High)
            .with_source("test-source")
            .with_correlation_id("corr-123")
            .with_attribute("key", "value");

        assert!(meta.id.starts_with("evt-"));
        assert_eq!(meta.topic, "test.topic");
        assert_eq!(meta.priority, EventPriority::High);
        assert_eq!(meta.source, Some("test-source".to_string()));
        assert_eq!(meta.correlation_id, Some("corr-123".to_string()));
        assert_eq!(meta.attributes.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_event_metadata_status() {
        let mut meta = EventMetadata::new("test");
        assert_eq!(meta.status, EventStatus::Pending);

        meta.mark_processing();
        assert_eq!(meta.status, EventStatus::Processing);

        meta.mark_processed();
        assert_eq!(meta.status, EventStatus::Processed);
        assert!(meta.processed_at.is_some());
    }

    #[test]
    fn test_event_data() {
        let none = EventData::None;
        assert!(none.is_none());

        let string = EventData::String("test".to_string());
        assert_eq!(string.as_string(), Some("test"));

        let int = EventData::Integer(42);
        assert_eq!(int.as_integer(), Some(42));

        let json = EventData::from_json(&serde_json::json!({"key": "value"}));
        assert!(json.as_json().is_some());
    }

    #[test]
    fn test_event_new() {
        let event = Event::new("test.topic", EventData::String("data".to_string()));

        assert!(event.id().starts_with("evt-"));
        assert_eq!(event.topic(), "test.topic");
        assert_eq!(event.data.as_string(), Some("data"));
    }

    #[test]
    fn test_event_empty() {
        let event = Event::empty("test.topic");
        assert!(event.data.is_none());
    }

    #[test]
    fn test_event_with_string() {
        let event = Event::with_string("test.topic", "hello");
        assert_eq!(event.data.as_string(), Some("hello"));
    }

    #[test]
    fn test_event_with_json() {
        let data = serde_json::json!({"name": "test"});
        let event = Event::with_json("test.topic", &data);
        assert!(event.data.as_json().is_some());
    }

    #[test]
    fn test_event_builder() {
        let event = Event::empty("test")
            .set_priority(EventPriority::High)
            .set_source("test-source")
            .set_correlation_id("corr-123")
            .add_attribute("key", "value");

        assert_eq!(event.priority(), EventPriority::High);
        assert_eq!(event.metadata.source, Some("test-source".to_string()));
    }

    #[test]
    fn test_event_filter_all() {
        let filter = EventFilter::All;
        let event = Event::empty("any.topic");
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_event_filter_topic_exact() {
        let filter = EventFilter::Topic("test.topic".to_string());
        assert!(filter.matches(&Event::empty("test.topic")));
        assert!(!filter.matches(&Event::empty("other.topic")));
    }

    #[test]
    fn test_event_filter_topic_wildcard() {
        let filter = EventFilter::Topic("test.*".to_string());
        assert!(filter.matches(&Event::empty("test.one")));
        assert!(filter.matches(&Event::empty("test.two")));
        assert!(!filter.matches(&Event::empty("other.one")));
    }

    #[test]
    fn test_event_filter_topic_double_wildcard() {
        let filter = EventFilter::Topic("test.**".to_string());
        assert!(filter.matches(&Event::empty("test.one")));
        assert!(filter.matches(&Event::empty("test.one.two")));
        assert!(filter.matches(&Event::empty("test.one.two.three")));
        assert!(!filter.matches(&Event::empty("other.one")));
    }

    #[test]
    fn test_event_filter_priority() {
        let filter = EventFilter::Priority(EventPriority::High);
        let high_event = Event::empty("test").set_priority(EventPriority::High);
        let low_event = Event::empty("test").set_priority(EventPriority::Low);

        assert!(filter.matches(&high_event));
        assert!(!filter.matches(&low_event));
    }

    #[test]
    fn test_event_filter_source() {
        let filter = EventFilter::Source("my-source".to_string());
        let event = Event::empty("test").set_source("my-source");
        let other = Event::empty("test").set_source("other-source");

        assert!(filter.matches(&event));
        assert!(!filter.matches(&other));
    }

    #[test]
    fn test_event_filter_and() {
        let filter = EventFilter::And(vec![
            EventFilter::Topic("test.*".to_string()),
            EventFilter::Priority(EventPriority::High),
        ]);

        let matching = Event::empty("test.one").set_priority(EventPriority::High);
        let wrong_topic = Event::empty("other.one").set_priority(EventPriority::High);
        let wrong_priority = Event::empty("test.one").set_priority(EventPriority::Low);

        assert!(filter.matches(&matching));
        assert!(!filter.matches(&wrong_topic));
        assert!(!filter.matches(&wrong_priority));
    }

    #[test]
    fn test_event_filter_or() {
        let filter = EventFilter::Or(vec![
            EventFilter::Topic("test.*".to_string()),
            EventFilter::Priority(EventPriority::Critical),
        ]);

        let topic_match = Event::empty("test.one");
        let priority_match = Event::empty("other.one").set_priority(EventPriority::Critical);
        let neither = Event::empty("other.one").set_priority(EventPriority::Low);

        assert!(filter.matches(&topic_match));
        assert!(filter.matches(&priority_match));
        assert!(!filter.matches(&neither));
    }

    #[test]
    fn test_event_filter_not() {
        let filter = EventFilter::Not(Box::new(EventFilter::Topic("test.*".to_string())));

        assert!(!filter.matches(&Event::empty("test.one")));
        assert!(filter.matches(&Event::empty("other.one")));
    }

    #[test]
    fn test_event_stats() {
        let mut stats = EventStats::default();

        stats.record_publish("test.topic");
        stats.record_publish("test.topic");
        stats.record_publish("other.topic");
        stats.record_processed();
        stats.record_processed();
        stats.record_failed();

        assert_eq!(stats.published, 3);
        assert_eq!(stats.processed, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.by_topic.get("test.topic"), Some(&2));
        assert!((stats.success_rate() - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_predefined_topics() {
        assert_eq!(topics::COMMAND_EXECUTED, "command.executed");
        assert_eq!(topics::LLM_RESPONSE, "llm.response");
        assert_eq!(topics::SESSION_START, "session.start");
    }
}
