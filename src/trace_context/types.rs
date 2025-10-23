//! 追踪上下文类型定义
//!
//! 提供 TraceContext 和 ExecutionSpan，用于追踪整个请求的执行路径

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// 追踪上下文
///
/// 串联整个请求的执行路径，每个请求对应一个唯一的 trace_id
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// 全局唯一追踪ID（串联整个请求）
    pub trace_id: Uuid,

    /// 当前 Span ID
    pub span_id: Uuid,

    /// 父级 Span ID
    pub parent_span_id: Option<Uuid>,

    /// Span 栈（调用层级）
    pub span_stack: Vec<Uuid>,

    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 用户输入
    pub user_input: String,

    /// 自定义属性
    pub attributes: HashMap<String, serde_json::Value>,
}

impl TraceContext {
    /// 创建新的追踪上下文（根 Span）
    pub fn new(user_input: impl Into<String>) -> Self {
        let trace_id = Uuid::new_v4();
        let span_id = Uuid::new_v4();

        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            span_stack: vec![span_id],
            start_time: Utc::now(),
            user_input: user_input.into(),
            attributes: HashMap::new(),
        }
    }

    /// 创建子 Span 的上下文
    pub fn create_child(&self, name: impl Into<String>) -> (TraceContext, ExecutionSpan) {
        let child_span_id = Uuid::new_v4();

        let child_ctx = TraceContext {
            trace_id: self.trace_id,
            span_id: child_span_id,
            parent_span_id: Some(self.span_id),
            span_stack: {
                let mut stack = self.span_stack.clone();
                stack.push(child_span_id);
                stack
            },
            start_time: Utc::now(),
            user_input: self.user_input.clone(),
            attributes: HashMap::new(),
        };

        let span = ExecutionSpan::new(
            child_span_id,
            self.trace_id,
            Some(self.span_id),
            name,
            SpanType::Internal,
        );

        (child_ctx, span)
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.attributes.insert(key.into(), value);
    }

    /// 获取当前深度
    pub fn depth(&self) -> usize {
        self.span_stack.len()
    }
}

/// 执行 Span
///
/// 记录一个执行步骤的详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSpan {
    /// Span ID
    pub span_id: Uuid,

    /// Trace ID（关联到请求）
    pub trace_id: Uuid,

    /// 父级 Span ID
    pub parent_span_id: Option<Uuid>,

    /// Span 名称
    pub name: String,

    /// Span 类型
    pub span_type: SpanType,

    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// 执行时长
    pub duration: Option<Duration>,

    /// 状态
    pub status: SpanStatus,

    /// 自定义属性
    pub attributes: HashMap<String, serde_json::Value>,

    /// 事件列表
    pub events: Vec<SpanEvent>,
}

impl ExecutionSpan {
    /// 创建新的 Span
    pub fn new(
        span_id: Uuid,
        trace_id: Uuid,
        parent_span_id: Option<Uuid>,
        name: impl Into<String>,
        span_type: SpanType,
    ) -> Self {
        Self {
            span_id,
            trace_id,
            parent_span_id,
            name: name.into(),
            span_type,
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            status: SpanStatus::Running,
            attributes: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// 结束 Span
    pub fn finish(&mut self) {
        let end_time = Utc::now();
        let duration = end_time
            .signed_duration_since(self.start_time)
            .to_std()
            .unwrap_or(Duration::from_secs(0));

        self.end_time = Some(end_time);
        self.duration = Some(duration);

        // 如果还在运行状态，标记为成功
        if matches!(self.status, SpanStatus::Running) {
            self.status = SpanStatus::Success;
        }
    }

    /// 标记为成功
    pub fn set_success(&mut self) {
        self.status = SpanStatus::Success;
        self.finish();
    }

    /// 标记为失败
    pub fn set_failed(&mut self, error: impl Into<String>) {
        self.status = SpanStatus::Failed(error.into());
        self.finish();
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.attributes.insert(key.into(), value);
    }

    /// 添加事件
    pub fn add_event(&mut self, name: impl Into<String>) {
        self.events.push(SpanEvent {
            timestamp: Utc::now(),
            name: name.into(),
            attributes: HashMap::new(),
        });
    }

    /// 添加带属性的事件
    pub fn add_event_with_attrs(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, serde_json::Value>,
    ) {
        self.events.push(SpanEvent {
            timestamp: Utc::now(),
            name: name.into(),
            attributes,
        });
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }

    /// 获取时长（毫秒）
    pub fn duration_ms(&self) -> Option<u64> {
        self.duration.map(|d| d.as_millis() as u64)
    }
}

/// Span 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpanType {
    /// 用户输入（根 Span）
    UserInput,

    /// 路由识别
    Router,

    /// 处理器
    Handler,

    /// LLM 调用
    LlmCall,

    /// 工具调用
    ToolCall,

    /// Shell 执行
    ShellExec,

    /// 系统命令
    SystemCommand,

    /// 内部调用
    Internal,
}

impl SpanType {
    /// 获取图标
    pub fn icon(&self) -> &'static str {
        match self {
            SpanType::UserInput => "👤",
            SpanType::Router => "🔄",
            SpanType::Handler => "⚙️",
            SpanType::LlmCall => "🤖",
            SpanType::ToolCall => "🔧",
            SpanType::ShellExec => "📊",
            SpanType::SystemCommand => "💻",
            SpanType::Internal => "🔹",
        }
    }

    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            SpanType::UserInput => "用户输入",
            SpanType::Router => "路由",
            SpanType::Handler => "处理器",
            SpanType::LlmCall => "LLM调用",
            SpanType::ToolCall => "工具调用",
            SpanType::ShellExec => "Shell执行",
            SpanType::SystemCommand => "系统命令",
            SpanType::Internal => "内部调用",
        }
    }
}

/// Span 状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 运行中
    Running,

    /// 成功
    Success,

    /// 失败
    Failed(String),

    /// 取消
    Cancelled,
}

impl SpanStatus {
    /// 获取图标
    pub fn icon(&self) -> &'static str {
        match self {
            SpanStatus::Running => "⟳",
            SpanStatus::Success => "✓",
            SpanStatus::Failed(_) => "✗",
            SpanStatus::Cancelled => "⊘",
        }
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        matches!(self, SpanStatus::Success)
    }

    /// 是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self, SpanStatus::Failed(_))
    }
}

/// Span 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// 事件时间戳
    pub timestamp: DateTime<Utc>,

    /// 事件名称
    pub name: String,

    /// 事件属性
    pub attributes: HashMap<String, serde_json::Value>,
}

/// 完整的 Trace（包含所有 Span）
#[derive(Debug, Clone)]
pub struct CompleteTrace {
    /// Trace ID
    pub trace_id: Uuid,

    /// 用户输入
    pub user_input: String,

    /// 所有 Span（按时间排序）
    pub spans: Vec<ExecutionSpan>,

    /// 根 Span ID
    pub root_span_id: Option<Uuid>,

    /// 开始时间
    pub start_time: DateTime<Utc>,

    /// 结束时间
    pub end_time: Option<DateTime<Utc>>,

    /// 总时长
    pub total_duration: Option<Duration>,
}

impl CompleteTrace {
    /// 创建新的完整 Trace
    pub fn new(trace_id: Uuid, user_input: String) -> Self {
        Self {
            trace_id,
            user_input,
            spans: Vec::new(),
            root_span_id: None,
            start_time: Utc::now(),
            end_time: None,
            total_duration: None,
        }
    }

    /// 添加 Span
    pub fn add_span(&mut self, span: ExecutionSpan) {
        // 如果是根 Span（无父级），记录为 root_span_id
        if span.parent_span_id.is_none() {
            self.root_span_id = Some(span.span_id);
            self.start_time = span.start_time;
        }

        self.spans.push(span);
    }

    /// 完成 Trace
    pub fn finish(&mut self) {
        self.end_time = Some(Utc::now());
        self.total_duration = self
            .end_time
            .and_then(|end| {
                end.signed_duration_since(self.start_time)
                    .to_std()
                    .ok()
            });
    }

    /// 获取根 Span
    pub fn root_span(&self) -> Option<&ExecutionSpan> {
        self.root_span_id
            .and_then(|id| self.spans.iter().find(|s| s.span_id == id))
    }

    /// 获取子 Span
    pub fn children_of(&self, parent_id: Uuid) -> Vec<&ExecutionSpan> {
        self.spans
            .iter()
            .filter(|s| s.parent_span_id == Some(parent_id))
            .collect()
    }

    /// 获取总时长（毫秒）
    pub fn total_duration_ms(&self) -> Option<u64> {
        self.total_duration.map(|d| d.as_millis() as u64)
    }

    /// 是否成功
    pub fn is_success(&self) -> bool {
        self.root_span()
            .map(|s| s.status.is_success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_context_new() {
        let ctx = TraceContext::new("test input");

        assert_eq!(ctx.user_input, "test input");
        assert_eq!(ctx.span_stack.len(), 1);
        assert_eq!(ctx.parent_span_id, None);
        assert_eq!(ctx.depth(), 1);
    }

    #[test]
    fn test_create_child() {
        let parent = TraceContext::new("test");
        let (child_ctx, child_span) = parent.create_child("child_span");

        assert_eq!(child_ctx.trace_id, parent.trace_id);
        assert_eq!(child_ctx.parent_span_id, Some(parent.span_id));
        assert_eq!(child_ctx.depth(), 2);
        assert_eq!(child_span.parent_span_id, Some(parent.span_id));
        assert_eq!(child_span.name, "child_span");
    }

    #[test]
    fn test_execution_span_lifecycle() {
        let mut span = ExecutionSpan::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "test_span",
            SpanType::Handler,
        );

        assert_eq!(span.status, SpanStatus::Running);
        assert!(!span.is_finished());

        span.finish();

        assert!(span.is_finished());
        assert!(span.duration.is_some());
        assert_eq!(span.status, SpanStatus::Success);
    }

    #[test]
    fn test_span_set_failed() {
        let mut span = ExecutionSpan::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            "test",
            SpanType::ShellExec,
        );

        span.set_failed("command not found");

        assert!(span.is_finished());
        assert!(matches!(span.status, SpanStatus::Failed(_)));
    }

    #[test]
    fn test_complete_trace() {
        let trace_id = Uuid::new_v4();
        let mut trace = CompleteTrace::new(trace_id, "test input".to_string());

        let root_span = ExecutionSpan::new(
            Uuid::new_v4(),
            trace_id,
            None,
            "root",
            SpanType::UserInput,
        );

        trace.add_span(root_span.clone());

        assert_eq!(trace.root_span_id, Some(root_span.span_id));
        assert_eq!(trace.spans.len(), 1);
    }
}
