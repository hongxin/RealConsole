//! 事件处理器
//!
//! 定义同步和异步事件处理接口

use super::{Event, EventId};
use async_trait::async_trait;
use std::fmt;
use thiserror::Error;

/// 处理器 ID
pub type HandlerId = String;

/// 处理器错误
#[derive(Error, Debug)]
pub enum HandlerError {
    /// 处理失败
    #[error("Handler failed: {0}")]
    Failed(String),
    /// 超时
    #[error("Handler timeout after {0}ms")]
    Timeout(u64),
    /// 处理器不可用
    #[error("Handler unavailable: {0}")]
    Unavailable(String),
    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),
}

/// 处理结果
#[derive(Debug, Clone, Default)]
pub enum HandlerResult {
    /// 成功处理
    #[default]
    Handled,
    /// 成功处理并停止传播
    StopPropagation,
    /// 跳过（让其他处理器处理）
    Skip,
    /// 重试
    Retry { after_ms: u64 },
    /// 失败
    Failed(String),
}

impl HandlerResult {
    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, HandlerResult::Handled | HandlerResult::StopPropagation)
    }

    /// 是否需要停止传播
    pub fn should_stop(&self) -> bool {
        matches!(self, HandlerResult::StopPropagation)
    }

    /// 是否跳过
    pub fn is_skip(&self) -> bool {
        matches!(self, HandlerResult::Skip)
    }

    /// 是否需要重试
    pub fn should_retry(&self) -> Option<u64> {
        if let HandlerResult::Retry { after_ms } = self {
            Some(*after_ms)
        } else {
            None
        }
    }

    /// 是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self, HandlerResult::Failed(_))
    }

    /// 获取错误信息
    pub fn error_message(&self) -> Option<&str> {
        if let HandlerResult::Failed(msg) = self {
            Some(msg)
        } else {
            None
        }
    }
}

/// 同步事件处理器
pub trait EventHandler: Send + Sync {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 处理事件
    fn handle(&self, event: &Event) -> HandlerResult;

    /// 处理器优先级（越大越先执行）
    fn priority(&self) -> i32 {
        0
    }

    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }
}

impl fmt::Debug for dyn EventHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventHandler")
            .field("name", &self.name())
            .field("priority", &self.priority())
            .field("enabled", &self.is_enabled())
            .finish()
    }
}

/// 异步事件处理器
#[async_trait]
pub trait AsyncEventHandler: Send + Sync {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 异步处理事件
    async fn handle(&self, event: &Event) -> HandlerResult;

    /// 处理器优先级（越大越先执行）
    fn priority(&self) -> i32 {
        0
    }

    /// 是否启用
    fn is_enabled(&self) -> bool {
        true
    }

    /// 超时时间（毫秒）
    fn timeout_ms(&self) -> u64 {
        5000
    }
}

impl fmt::Debug for dyn AsyncEventHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AsyncEventHandler")
            .field("name", &self.name())
            .field("priority", &self.priority())
            .field("enabled", &self.is_enabled())
            .field("timeout_ms", &self.timeout_ms())
            .finish()
    }
}

/// 简单闭包处理器
pub struct FnHandler<F>
where
    F: Fn(&Event) -> HandlerResult + Send + Sync,
{
    name: String,
    handler: F,
    priority: i32,
}

impl<F> FnHandler<F>
where
    F: Fn(&Event) -> HandlerResult + Send + Sync,
{
    /// 创建新处理器
    pub fn new(name: impl Into<String>, handler: F) -> Self {
        Self {
            name: name.into(),
            handler,
            priority: 0,
        }
    }

    /// 设置优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl<F> EventHandler for FnHandler<F>
where
    F: Fn(&Event) -> HandlerResult + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, event: &Event) -> HandlerResult {
        (self.handler)(event)
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

/// 日志处理器（用于调试）
pub struct LoggingHandler {
    name: String,
    prefix: String,
}

impl LoggingHandler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prefix: "[EVENT]".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

impl EventHandler for LoggingHandler {
    fn name(&self) -> &str {
        &self.name
    }

    fn handle(&self, event: &Event) -> HandlerResult {
        println!(
            "{} {} - topic: {}, priority: {:?}",
            self.prefix,
            event.id(),
            event.topic(),
            event.priority()
        );
        HandlerResult::Handled
    }

    fn priority(&self) -> i32 {
        -100 // 低优先级，最后执行
    }
}

/// 处理器包装（用于存储）
pub enum HandlerWrapper {
    Sync(Box<dyn EventHandler>),
    Async(Box<dyn AsyncEventHandler>),
}

impl HandlerWrapper {
    pub fn name(&self) -> &str {
        match self {
            HandlerWrapper::Sync(h) => h.name(),
            HandlerWrapper::Async(h) => h.name(),
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            HandlerWrapper::Sync(h) => h.priority(),
            HandlerWrapper::Async(h) => h.priority(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        match self {
            HandlerWrapper::Sync(h) => h.is_enabled(),
            HandlerWrapper::Async(h) => h.is_enabled(),
        }
    }

    pub fn is_async(&self) -> bool {
        matches!(self, HandlerWrapper::Async(_))
    }
}

impl fmt::Debug for HandlerWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerWrapper::Sync(h) => {
                f.debug_tuple("Sync").field(h).finish()
            }
            HandlerWrapper::Async(h) => {
                f.debug_tuple("Async").field(h).finish()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventData;

    #[test]
    fn test_handler_result_success() {
        assert!(HandlerResult::Handled.is_success());
        assert!(HandlerResult::StopPropagation.is_success());
        assert!(!HandlerResult::Skip.is_success());
        assert!(!HandlerResult::Failed("error".to_string()).is_success());
    }

    #[test]
    fn test_handler_result_stop() {
        assert!(!HandlerResult::Handled.should_stop());
        assert!(HandlerResult::StopPropagation.should_stop());
    }

    #[test]
    fn test_handler_result_skip() {
        assert!(HandlerResult::Skip.is_skip());
        assert!(!HandlerResult::Handled.is_skip());
    }

    #[test]
    fn test_handler_result_retry() {
        assert_eq!(HandlerResult::Retry { after_ms: 100 }.should_retry(), Some(100));
        assert_eq!(HandlerResult::Handled.should_retry(), None);
    }

    #[test]
    fn test_handler_result_failed() {
        assert!(HandlerResult::Failed("error".to_string()).is_failed());
        assert_eq!(
            HandlerResult::Failed("test error".to_string()).error_message(),
            Some("test error")
        );
        assert!(!HandlerResult::Handled.is_failed());
    }

    #[test]
    fn test_fn_handler() {
        let handler = FnHandler::new("test", |event| {
            if event.topic() == "test.topic" {
                HandlerResult::Handled
            } else {
                HandlerResult::Skip
            }
        })
        .with_priority(10);

        assert_eq!(handler.name(), "test");
        assert_eq!(handler.priority(), 10);

        let event = Event::new("test.topic", EventData::None);
        assert!(handler.handle(&event).is_success());

        let other = Event::new("other.topic", EventData::None);
        assert!(handler.handle(&other).is_skip());
    }

    #[test]
    fn test_logging_handler() {
        let handler = LoggingHandler::new("logger").with_prefix("[DEBUG]");

        assert_eq!(handler.name(), "logger");
        assert_eq!(handler.priority(), -100);

        let event = Event::new("test.topic", EventData::None);
        let result = handler.handle(&event);
        assert!(result.is_success());
    }

    #[test]
    fn test_handler_wrapper() {
        let sync_handler: Box<dyn EventHandler> = Box::new(
            FnHandler::new("sync", |_| HandlerResult::Handled).with_priority(5)
        );

        let wrapper = HandlerWrapper::Sync(sync_handler);

        assert_eq!(wrapper.name(), "sync");
        assert_eq!(wrapper.priority(), 5);
        assert!(wrapper.is_enabled());
        assert!(!wrapper.is_async());
    }

    struct TestAsyncHandler {
        name: String,
    }

    #[async_trait]
    impl AsyncEventHandler for TestAsyncHandler {
        fn name(&self) -> &str {
            &self.name
        }

        async fn handle(&self, _event: &Event) -> HandlerResult {
            HandlerResult::Handled
        }

        fn priority(&self) -> i32 {
            20
        }

        fn timeout_ms(&self) -> u64 {
            3000
        }
    }

    #[test]
    fn test_async_handler_wrapper() {
        let async_handler = TestAsyncHandler {
            name: "async_test".to_string(),
        };

        let wrapper = HandlerWrapper::Async(Box::new(async_handler));

        assert_eq!(wrapper.name(), "async_test");
        assert_eq!(wrapper.priority(), 20);
        assert!(wrapper.is_async());
    }

    #[test]
    fn test_handler_error() {
        let failed = HandlerError::Failed("test failure".to_string());
        assert!(failed.to_string().contains("test failure"));

        let timeout = HandlerError::Timeout(5000);
        assert!(timeout.to_string().contains("5000"));
    }
}
