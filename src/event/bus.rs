//! 事件总线
//!
//! 实现发布-订阅模式的事件分发

use super::{Event, EventFilter, EventId, EventStats, Topic};
use super::handler::{AsyncEventHandler, EventHandler, HandlerResult, HandlerWrapper};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use thiserror::Error;
use uuid::Uuid;

/// 订阅 ID
pub type SubscriptionId = String;

/// 事件总线错误
#[derive(Error, Debug)]
pub enum EventBusError {
    /// 订阅不存在
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(SubscriptionId),
    /// 发布失败
    #[error("Failed to publish event: {0}")]
    PublishFailed(String),
    /// 处理器错误
    #[error("Handler error: {0}")]
    HandlerError(String),
    /// 总线已关闭
    #[error("Event bus is closed")]
    Closed,
}

/// 事件总线配置
#[derive(Debug, Clone)]
pub struct EventBusConfig {
    /// 最大历史记录
    pub max_history: usize,
    /// 是否启用统计
    pub enable_stats: bool,
    /// 默认超时（毫秒）
    pub default_timeout_ms: u64,
    /// 是否允许并行处理
    pub parallel_handlers: bool,
    /// 最大并行处理器数
    pub max_parallel: usize,
}

impl Default for EventBusConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            enable_stats: true,
            default_timeout_ms: 5000,
            parallel_handlers: false,
            max_parallel: 10,
        }
    }
}

impl EventBusConfig {
    /// 创建高性能配置
    pub fn high_performance() -> Self {
        Self {
            max_history: 100,
            enable_stats: false,
            default_timeout_ms: 1000,
            parallel_handlers: true,
            max_parallel: 50,
        }
    }

    /// 创建调试配置
    pub fn debug() -> Self {
        Self {
            max_history: 10000,
            enable_stats: true,
            default_timeout_ms: 30000,
            parallel_handlers: false,
            max_parallel: 1,
        }
    }
}

/// 订阅信息
#[derive(Debug)]
pub struct Subscription {
    /// 订阅 ID
    pub id: SubscriptionId,
    /// 过滤器
    pub filter: EventFilter,
    /// 处理器
    handler: HandlerWrapper,
    /// 是否启用
    pub enabled: bool,
    /// 处理计数
    processed_count: AtomicU64,
}

impl Subscription {
    /// 创建同步订阅
    pub fn sync(filter: EventFilter, handler: Box<dyn EventHandler>) -> Self {
        Self {
            id: format!("sub-{}", Uuid::new_v4()),
            filter,
            handler: HandlerWrapper::Sync(handler),
            enabled: true,
            processed_count: AtomicU64::new(0),
        }
    }

    /// 创建异步订阅
    pub fn async_sub(filter: EventFilter, handler: Box<dyn AsyncEventHandler>) -> Self {
        Self {
            id: format!("sub-{}", Uuid::new_v4()),
            filter,
            handler: HandlerWrapper::Async(handler),
            enabled: true,
            processed_count: AtomicU64::new(0),
        }
    }

    /// 处理器名称
    pub fn handler_name(&self) -> &str {
        self.handler.name()
    }

    /// 处理器优先级
    pub fn priority(&self) -> i32 {
        self.handler.priority()
    }

    /// 是否为异步处理器
    pub fn is_async(&self) -> bool {
        self.handler.is_async()
    }

    /// 获取处理计数
    pub fn processed_count(&self) -> u64 {
        self.processed_count.load(Ordering::Relaxed)
    }

    /// 增加处理计数
    fn increment_count(&self) {
        self.processed_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 检查是否匹配事件
    pub fn matches(&self, event: &Event) -> bool {
        self.enabled && self.filter.matches(event)
    }

    /// 同步处理事件
    pub fn handle_sync(&self, event: &Event) -> Option<HandlerResult> {
        if let HandlerWrapper::Sync(h) = &self.handler {
            if h.is_enabled() {
                self.increment_count();
                return Some(h.handle(event));
            }
        }
        None
    }

    /// 异步处理事件
    pub async fn handle_async(&self, event: &Event) -> Option<HandlerResult> {
        if let HandlerWrapper::Async(h) = &self.handler {
            if h.is_enabled() {
                self.increment_count();
                return Some(h.handle(event).await);
            }
        }
        None
    }
}

/// 事件总线
pub struct EventBus {
    /// 配置
    config: EventBusConfig,
    /// 订阅列表
    subscriptions: Arc<RwLock<Vec<Subscription>>>,
    /// 事件历史
    history: Arc<RwLock<Vec<Event>>>,
    /// 统计
    stats: Arc<RwLock<EventStats>>,
    /// 是否关闭
    closed: Arc<RwLock<bool>>,
}

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        Self::with_config(EventBusConfig::default())
    }

    /// 使用配置创建
    pub fn with_config(config: EventBusConfig) -> Self {
        Self {
            config,
            subscriptions: Arc::new(RwLock::new(Vec::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(EventStats::default())),
            closed: Arc::new(RwLock::new(false)),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &EventBusConfig {
        &self.config
    }

    /// 订阅主题（简化接口）
    pub fn subscribe(
        &self,
        topic_pattern: &str,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, EventBusError> {
        self.subscribe_with_filter(EventFilter::Topic(topic_pattern.to_string()), handler)
    }

    /// 订阅所有事件
    pub fn subscribe_all(
        &self,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, EventBusError> {
        self.subscribe_with_filter(EventFilter::All, handler)
    }

    /// 使用过滤器订阅
    pub fn subscribe_with_filter(
        &self,
        filter: EventFilter,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, EventBusError> {
        self.check_closed()?;

        let subscription = Subscription::sync(filter, handler);
        let id = subscription.id.clone();

        let count = {
            let mut subs = self.subscriptions.write().unwrap();
            subs.push(subscription);
            // 按优先级排序（高优先级在前）
            subs.sort_by(|a, b| b.priority().cmp(&a.priority()));
            subs.len()
        };

        self.update_subscriber_count(count);
        Ok(id)
    }

    /// 异步订阅
    pub fn subscribe_async(
        &self,
        topic_pattern: &str,
        handler: Box<dyn AsyncEventHandler>,
    ) -> Result<SubscriptionId, EventBusError> {
        self.subscribe_async_with_filter(
            EventFilter::Topic(topic_pattern.to_string()),
            handler,
        )
    }

    /// 使用过滤器异步订阅
    pub fn subscribe_async_with_filter(
        &self,
        filter: EventFilter,
        handler: Box<dyn AsyncEventHandler>,
    ) -> Result<SubscriptionId, EventBusError> {
        self.check_closed()?;

        let subscription = Subscription::async_sub(filter, handler);
        let id = subscription.id.clone();

        let count = {
            let mut subs = self.subscriptions.write().unwrap();
            subs.push(subscription);
            subs.sort_by(|a, b| b.priority().cmp(&a.priority()));
            subs.len()
        };

        self.update_subscriber_count(count);
        Ok(id)
    }

    /// 取消订阅
    pub fn unsubscribe(&self, id: &SubscriptionId) -> Result<(), EventBusError> {
        let (found, count) = {
            let mut subs = self.subscriptions.write().unwrap();
            let initial_len = subs.len();
            subs.retain(|s| &s.id != id);
            (subs.len() != initial_len, subs.len())
        };

        if !found {
            Err(EventBusError::SubscriptionNotFound(id.clone()))
        } else {
            self.update_subscriber_count(count);
            Ok(())
        }
    }

    /// 发布事件（同步）
    pub fn publish(&self, event: Event) -> Result<Vec<HandlerResult>, EventBusError> {
        self.check_closed()?;

        // 记录统计
        if self.config.enable_stats {
            let mut stats = self.stats.write().unwrap();
            stats.record_publish(event.topic());
        }

        // 记录历史
        self.record_history(&event);

        // 获取匹配的订阅
        let results = self.dispatch_sync(&event);

        Ok(results)
    }

    /// 发布事件（异步）
    pub async fn publish_async(&self, event: Event) -> Result<Vec<HandlerResult>, EventBusError> {
        self.check_closed()?;

        // 记录统计
        if self.config.enable_stats {
            let mut stats = self.stats.write().unwrap();
            stats.record_publish(event.topic());
        }

        // 记录历史
        self.record_history(&event);

        // 获取匹配的订阅
        let results = self.dispatch_async(&event).await;

        Ok(results)
    }

    /// 同步分发
    fn dispatch_sync(&self, event: &Event) -> Vec<HandlerResult> {
        let subs = self.subscriptions.read().unwrap();
        let mut results = Vec::new();

        for sub in subs.iter() {
            if sub.matches(event) {
                if let Some(result) = sub.handle_sync(event) {
                    let should_stop = result.should_stop();
                    self.record_result(&result);
                    results.push(result);
                    if should_stop {
                        break;
                    }
                }
            }
        }

        results
    }

    /// 异步分发
    async fn dispatch_async(&self, event: &Event) -> Vec<HandlerResult> {
        // 先处理同步处理器
        let sync_results = self.dispatch_sync(event);

        // 检查是否需要停止
        if sync_results.iter().any(|r| r.should_stop()) {
            return sync_results;
        }

        let mut results = sync_results;

        // 处理异步处理器
        let subs = self.subscriptions.read().unwrap();
        for sub in subs.iter() {
            if sub.matches(event) && sub.is_async() {
                if let Some(result) = sub.handle_async(event).await {
                    let should_stop = result.should_stop();
                    self.record_result(&result);
                    results.push(result);
                    if should_stop {
                        break;
                    }
                }
            }
        }

        results
    }

    /// 记录历史
    fn record_history(&self, event: &Event) {
        let mut history = self.history.write().unwrap();
        history.push(event.clone());

        // 保持历史记录在限制内
        while history.len() > self.config.max_history {
            history.remove(0);
        }
    }

    /// 记录结果统计
    fn record_result(&self, result: &HandlerResult) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().unwrap();
            if result.is_success() {
                stats.record_processed();
            } else if result.is_failed() {
                stats.record_failed();
            }
        }
    }

    /// 更新订阅者数量（传入计数避免死锁）
    fn update_subscriber_count(&self, count: usize) {
        if self.config.enable_stats {
            let mut stats = self.stats.write().unwrap();
            stats.subscribers = count;
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> EventStats {
        self.stats.read().unwrap().clone()
    }

    /// 获取订阅数量
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.read().unwrap().len()
    }

    /// 获取历史事件
    pub fn history(&self) -> Vec<Event> {
        self.history.read().unwrap().clone()
    }

    /// 获取最近的事件
    pub fn recent_events(&self, count: usize) -> Vec<Event> {
        let history = self.history.read().unwrap();
        history.iter().rev().take(count).cloned().collect()
    }

    /// 按主题查询历史
    pub fn events_by_topic(&self, topic_pattern: &str) -> Vec<Event> {
        let filter = EventFilter::Topic(topic_pattern.to_string());
        let history = self.history.read().unwrap();
        history.iter().filter(|e| filter.matches(e)).cloned().collect()
    }

    /// 清空历史
    pub fn clear_history(&self) {
        let mut history = self.history.write().unwrap();
        history.clear();
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = EventStats::default();
        stats.subscribers = self.subscription_count();
    }

    /// 禁用订阅
    pub fn disable_subscription(&self, id: &SubscriptionId) -> Result<(), EventBusError> {
        let mut subs = self.subscriptions.write().unwrap();
        for sub in subs.iter_mut() {
            if &sub.id == id {
                sub.enabled = false;
                return Ok(());
            }
        }
        Err(EventBusError::SubscriptionNotFound(id.clone()))
    }

    /// 启用订阅
    pub fn enable_subscription(&self, id: &SubscriptionId) -> Result<(), EventBusError> {
        let mut subs = self.subscriptions.write().unwrap();
        for sub in subs.iter_mut() {
            if &sub.id == id {
                sub.enabled = true;
                return Ok(());
            }
        }
        Err(EventBusError::SubscriptionNotFound(id.clone()))
    }

    /// 关闭事件总线
    pub fn close(&self) {
        let mut closed = self.closed.write().unwrap();
        *closed = true;
    }

    /// 是否已关闭
    pub fn is_closed(&self) -> bool {
        *self.closed.read().unwrap()
    }

    /// 检查是否已关闭
    fn check_closed(&self) -> Result<(), EventBusError> {
        if self.is_closed() {
            Err(EventBusError::Closed)
        } else {
            Ok(())
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            subscriptions: Arc::clone(&self.subscriptions),
            history: Arc::clone(&self.history),
            stats: Arc::clone(&self.stats),
            closed: Arc::clone(&self.closed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::handler::FnHandler;
    use crate::event::{EventData, EventPriority};

    fn create_test_handler(name: &str) -> Box<dyn EventHandler> {
        Box::new(FnHandler::new(name.to_string(), |_| HandlerResult::Handled))
    }

    fn create_counting_handler(name: &str, counter: Arc<AtomicU64>) -> Box<dyn EventHandler> {
        Box::new(FnHandler::new(name.to_string(), move |_| {
            counter.fetch_add(1, Ordering::Relaxed);
            HandlerResult::Handled
        }))
    }

    #[test]
    fn test_event_bus_new() {
        let bus = EventBus::new();
        assert_eq!(bus.subscription_count(), 0);
        assert!(!bus.is_closed());
    }

    #[test]
    fn test_event_bus_config() {
        let config = EventBusConfig::high_performance();
        assert!(config.parallel_handlers);
        assert_eq!(config.max_parallel, 50);

        let debug_config = EventBusConfig::debug();
        assert!(debug_config.enable_stats);
        assert_eq!(debug_config.max_history, 10000);
    }

    #[test]
    fn test_subscribe() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");

        let id = bus.subscribe("test.*", handler).unwrap();
        assert!(id.starts_with("sub-"));
        assert_eq!(bus.subscription_count(), 1);
    }

    #[test]
    fn test_subscribe_all() {
        let bus = EventBus::new();
        let handler = create_test_handler("all");

        let id = bus.subscribe_all(handler).unwrap();
        assert!(id.starts_with("sub-"));
    }

    #[test]
    fn test_unsubscribe() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");

        let id = bus.subscribe("test.*", handler).unwrap();
        assert_eq!(bus.subscription_count(), 1);

        bus.unsubscribe(&id).unwrap();
        assert_eq!(bus.subscription_count(), 0);
    }

    #[test]
    fn test_unsubscribe_not_found() {
        let bus = EventBus::new();
        let result = bus.unsubscribe(&"non-existent".to_string());
        assert!(matches!(result, Err(EventBusError::SubscriptionNotFound(_))));
    }

    #[test]
    fn test_publish() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));
        let handler = create_counting_handler("counter", counter.clone());

        bus.subscribe("test.*", handler).unwrap();

        let event = Event::empty("test.topic");
        let results = bus.publish(event).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_publish_multiple_handlers() {
        let bus = EventBus::new();
        let counter1 = Arc::new(AtomicU64::new(0));
        let counter2 = Arc::new(AtomicU64::new(0));

        bus.subscribe("test.*", create_counting_handler("h1", counter1.clone())).unwrap();
        bus.subscribe("test.*", create_counting_handler("h2", counter2.clone())).unwrap();

        let event = Event::empty("test.topic");
        let results = bus.publish(event).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(counter1.load(Ordering::Relaxed), 1);
        assert_eq!(counter2.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_publish_stop_propagation() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));

        // 高优先级处理器停止传播
        let stop_handler = Box::new(
            FnHandler::new("stopper", |_| HandlerResult::StopPropagation).with_priority(10)
        );
        bus.subscribe("test.*", stop_handler).unwrap();

        // 低优先级处理器不应被调用
        bus.subscribe("test.*", create_counting_handler("counter", counter.clone())).unwrap();

        let event = Event::empty("test.topic");
        let results = bus.publish(event).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].should_stop());
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_publish_filter_mismatch() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));

        bus.subscribe("test.*", create_counting_handler("counter", counter.clone())).unwrap();

        let event = Event::empty("other.topic");
        let results = bus.publish(event).unwrap();

        assert_eq!(results.len(), 0);
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_history() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        bus.publish(Event::empty("topic.1")).unwrap();
        bus.publish(Event::empty("topic.2")).unwrap();
        bus.publish(Event::empty("topic.3")).unwrap();

        let history = bus.history();
        assert_eq!(history.len(), 3);

        let recent = bus.recent_events(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].topic(), "topic.3");
        assert_eq!(recent[1].topic(), "topic.2");
    }

    #[test]
    fn test_events_by_topic() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        bus.publish(Event::empty("test.1")).unwrap();
        bus.publish(Event::empty("other.1")).unwrap();
        bus.publish(Event::empty("test.2")).unwrap();

        let events = bus.events_by_topic("test.*");
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_clear_history() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        bus.publish(Event::empty("topic.1")).unwrap();
        bus.publish(Event::empty("topic.2")).unwrap();

        assert_eq!(bus.history().len(), 2);
        bus.clear_history();
        assert_eq!(bus.history().len(), 0);
    }

    #[test]
    fn test_stats() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe("test.*", handler).unwrap();

        bus.publish(Event::empty("test.1")).unwrap();
        bus.publish(Event::empty("test.2")).unwrap();
        bus.publish(Event::empty("other.1")).unwrap(); // 不匹配

        let stats = bus.stats();
        assert_eq!(stats.published, 3);
        assert_eq!(stats.processed, 2);
        assert_eq!(stats.subscribers, 1);
    }

    #[test]
    fn test_reset_stats() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        bus.publish(Event::empty("topic")).unwrap();

        let stats = bus.stats();
        assert_eq!(stats.published, 1);

        bus.reset_stats();
        let stats = bus.stats();
        assert_eq!(stats.published, 0);
        assert_eq!(stats.subscribers, 1); // 订阅者数量保留
    }

    #[test]
    fn test_disable_enable_subscription() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));
        let handler = create_counting_handler("counter", counter.clone());

        let id = bus.subscribe("test.*", handler).unwrap();

        bus.publish(Event::empty("test.1")).unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // 禁用
        bus.disable_subscription(&id).unwrap();
        bus.publish(Event::empty("test.2")).unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 1); // 不增加

        // 启用
        bus.enable_subscription(&id).unwrap();
        bus.publish(Event::empty("test.3")).unwrap();
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_close() {
        let bus = EventBus::new();
        assert!(!bus.is_closed());

        bus.close();
        assert!(bus.is_closed());

        let result = bus.publish(Event::empty("test"));
        assert!(matches!(result, Err(EventBusError::Closed)));
    }

    #[test]
    fn test_clone() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        let bus2 = bus.clone();

        // 共享状态
        bus.publish(Event::empty("topic")).unwrap();

        assert_eq!(bus.history().len(), 1);
        assert_eq!(bus2.history().len(), 1);
    }

    #[test]
    fn test_subscription_priority() {
        let bus = EventBus::new();
        let order = Arc::new(RwLock::new(Vec::new()));

        let order1 = order.clone();
        let h1 = Box::new(FnHandler::new("low", move |_| {
            order1.write().unwrap().push("low");
            HandlerResult::Handled
        }).with_priority(0));

        let order2 = order.clone();
        let h2 = Box::new(FnHandler::new("high", move |_| {
            order2.write().unwrap().push("high");
            HandlerResult::Handled
        }).with_priority(10));

        // 先添加低优先级
        bus.subscribe_all(h1).unwrap();
        // 再添加高优先级
        bus.subscribe_all(h2).unwrap();

        bus.publish(Event::empty("test")).unwrap();

        let order = order.read().unwrap();
        assert_eq!(order[0], "high"); // 高优先级先执行
        assert_eq!(order[1], "low");
    }

    #[tokio::test]
    async fn test_publish_async() {
        let bus = EventBus::new();
        let counter = Arc::new(AtomicU64::new(0));
        let handler = create_counting_handler("counter", counter.clone());

        bus.subscribe("test.*", handler).unwrap();

        let event = Event::empty("test.topic");
        let results = bus.publish_async(event).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_subscription_processed_count() {
        let bus = EventBus::new();
        let handler = create_test_handler("test");
        let id = bus.subscribe_all(handler).unwrap();

        bus.publish(Event::empty("topic.1")).unwrap();
        bus.publish(Event::empty("topic.2")).unwrap();

        let subs = bus.subscriptions.read().unwrap();
        let sub = subs.iter().find(|s| s.id == id).unwrap();
        assert_eq!(sub.processed_count(), 2);
    }

    #[test]
    fn test_max_history() {
        let config = EventBusConfig {
            max_history: 3,
            ..Default::default()
        };
        let bus = EventBus::with_config(config);
        let handler = create_test_handler("test");
        bus.subscribe_all(handler).unwrap();

        for i in 0..5 {
            bus.publish(Event::empty(format!("topic.{}", i))).unwrap();
        }

        let history = bus.history();
        assert_eq!(history.len(), 3);
        assert_eq!(history[0].topic(), "topic.2");
        assert_eq!(history[2].topic(), "topic.4");
    }
}
