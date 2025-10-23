//! 追踪上下文模块
//!
//! 提供完整的请求追踪能力，记录调用链和执行路径
//!
//! # 核心概念
//!
//! - **TraceContext**: 追踪上下文，串联整个请求
//! - **ExecutionSpan**: 执行步骤，记录调用链中的每一步
//! - **TraceStore**: 存储和查询 Trace
//!
//! # 使用示例
//!
//! ```rust
//! use realconsole::trace_context::{TraceContext, TraceStore, SpanType};
//!
//! #[tokio::main]
//! async fn main() {
//!     let store = TraceStore::new(100);
//!
//!     // 创建追踪上下文
//!     let ctx = TraceContext::new("user input");
//!
//!     // 开始追踪
//!     store.start_trace(ctx.trace_id, ctx.user_input.clone()).await.unwrap();
//!
//!     // 创建子 Span
//!     let (child_ctx, mut child_span) = ctx.create_child("llm_call");
//!     child_span.span_type = SpanType::LlmCall;
//!
//!     // 执行并记录
//!     // ... 执行实际操作 ...
//!
//!     child_span.finish();
//!     store.record_span(child_span).await.unwrap();
//!
//!     // 完成追踪
//!     store.finish_trace(ctx.trace_id).await.unwrap();
//!
//!     // 查询
//!     let trace = store.get_trace(ctx.trace_id).await;
//!     println!("Trace: {:?}", trace);
//! }
//! ```

pub mod store;
pub mod types;

// 重新导出核心类型
pub use store::{TraceStore, TraceStoreStats};
pub use types::{
    CompleteTrace, ExecutionSpan, SpanEvent, SpanStatus, SpanType, TraceContext,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 测试核心类型可以正常导入
        let ctx = TraceContext::new("test");
        assert!(!ctx.user_input.is_empty());

        let span = ExecutionSpan::new(
            ctx.span_id,
            ctx.trace_id,
            None,
            "test",
            SpanType::Handler,
        );
        assert_eq!(span.name, "test");

        let store = TraceStore::new(100);
        assert!(tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(store.is_empty()));
    }
}
