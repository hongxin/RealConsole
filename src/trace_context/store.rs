//! Trace 存储
//!
//! 负责存储和查询 Trace 和 Span

use super::types::{CompleteTrace, ExecutionSpan};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Trace 存储
///
/// 线程安全的存储，支持并发读写
pub struct TraceStore {
    /// Trace 存储（按 trace_id 索引）
    traces: Arc<RwLock<HashMap<Uuid, CompleteTrace>>>,

    /// 最近的 trace_id 列表（时间排序，最新的在前）
    recent_traces: Arc<RwLock<VecDeque<Uuid>>>,

    /// 最大保留数量
    max_traces: usize,
}

impl TraceStore {
    /// 创建新的 TraceStore
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            recent_traces: Arc::new(RwLock::new(VecDeque::new())),
            max_traces,
        }
    }

    /// 开始一个新的 Trace
    pub async fn start_trace(&self, trace_id: Uuid, user_input: String) -> Result<()> {
        let trace = CompleteTrace::new(trace_id, user_input);

        let mut traces = self.traces.write().await;
        traces.insert(trace_id, trace);

        let mut recent = self.recent_traces.write().await;
        recent.push_front(trace_id);

        // 限制数量
        if recent.len() > self.max_traces {
            if let Some(old_id) = recent.pop_back() {
                traces.remove(&old_id);
            }
        }

        Ok(())
    }

    /// 记录 Span
    pub async fn record_span(&self, span: ExecutionSpan) -> Result<()> {
        let trace_id = span.trace_id;

        let mut traces = self.traces.write().await;

        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.add_span(span);
        } else {
            // 如果 Trace 不存在，创建一个新的
            let mut trace = CompleteTrace::new(trace_id, String::new());
            trace.add_span(span);
            traces.insert(trace_id, trace);

            // 同时更新 recent 列表
            drop(traces); // 释放写锁
            let mut recent = self.recent_traces.write().await;
            if !recent.contains(&trace_id) {
                recent.push_front(trace_id);
                if recent.len() > self.max_traces {
                    recent.pop_back();
                }
            }
        }

        Ok(())
    }

    /// 完成一个 Trace
    pub async fn finish_trace(&self, trace_id: Uuid) -> Result<()> {
        let mut traces = self.traces.write().await;

        if let Some(trace) = traces.get_mut(&trace_id) {
            trace.finish();
        }

        Ok(())
    }

    /// 获取完整的 Trace
    pub async fn get_trace(&self, trace_id: Uuid) -> Option<CompleteTrace> {
        let traces = self.traces.read().await;
        traces.get(&trace_id).cloned()
    }

    /// 获取最近的 N 个 Trace ID
    pub async fn get_recent_trace_ids(&self, limit: usize) -> Vec<Uuid> {
        let recent = self.recent_traces.read().await;
        recent.iter().take(limit).copied().collect()
    }

    /// 获取最近的 N 个完整 Trace
    pub async fn get_recent_traces(&self, limit: usize) -> Vec<CompleteTrace> {
        let trace_ids = self.get_recent_trace_ids(limit).await;
        let traces = self.traces.read().await;

        trace_ids
            .into_iter()
            .filter_map(|id| traces.get(&id).cloned())
            .collect()
    }

    /// 搜索 Trace（通过用户输入关键词）
    pub async fn search_traces(&self, keyword: &str) -> Vec<CompleteTrace> {
        let traces = self.traces.read().await;
        let keyword_lower = keyword.to_lowercase();

        traces
            .values()
            .filter(|trace| trace.user_input.to_lowercase().contains(&keyword_lower))
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> TraceStoreStats {
        let traces = self.traces.read().await;

        let total_traces = traces.len();
        let success_traces = traces.values().filter(|t| t.is_success()).count();
        let failed_traces = total_traces - success_traces;

        let avg_duration_ms = if total_traces > 0 {
            let total_ms: u64 = traces
                .values()
                .filter_map(|t| t.total_duration_ms())
                .sum();
            total_ms / total_traces as u64
        } else {
            0
        };

        TraceStoreStats {
            total_traces,
            success_traces,
            failed_traces,
            avg_duration_ms,
        }
    }

    /// 清空所有 Trace
    pub async fn clear(&self) {
        let mut traces = self.traces.write().await;
        let mut recent = self.recent_traces.write().await;

        traces.clear();
        recent.clear();
    }

    /// 获取当前存储的 Trace 数量
    pub async fn len(&self) -> usize {
        let traces = self.traces.read().await;
        traces.len()
    }

    /// 判断是否为空
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }
}

/// TraceStore 统计信息
#[derive(Debug, Clone)]
pub struct TraceStoreStats {
    /// 总 Trace 数量
    pub total_traces: usize,

    /// 成功的 Trace 数量
    pub success_traces: usize,

    /// 失败的 Trace 数量
    pub failed_traces: usize,

    /// 平均时长（毫秒）
    pub avg_duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace_context::types::{SpanType, SpanStatus};

    #[tokio::test]
    async fn test_trace_store_basic() {
        let store = TraceStore::new(100);

        let trace_id = Uuid::new_v4();
        store
            .start_trace(trace_id, "test input".to_string())
            .await
            .unwrap();

        let trace = store.get_trace(trace_id).await;
        assert!(trace.is_some());
        assert_eq!(trace.unwrap().user_input, "test input");
    }

    #[tokio::test]
    async fn test_record_span() {
        let store = TraceStore::new(100);

        let trace_id = Uuid::new_v4();
        store
            .start_trace(trace_id, "test".to_string())
            .await
            .unwrap();

        let span = ExecutionSpan::new(
            Uuid::new_v4(),
            trace_id,
            None,
            "test_span",
            SpanType::Handler,
        );

        store.record_span(span).await.unwrap();

        let trace = store.get_trace(trace_id).await.unwrap();
        assert_eq!(trace.spans.len(), 1);
    }

    #[tokio::test]
    async fn test_recent_traces() {
        let store = TraceStore::new(100);

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let id3 = Uuid::new_v4();

        store.start_trace(id1, "first".to_string()).await.unwrap();
        store.start_trace(id2, "second".to_string()).await.unwrap();
        store.start_trace(id3, "third".to_string()).await.unwrap();

        let recent = store.get_recent_trace_ids(2).await;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0], id3); // 最新的在前
        assert_eq!(recent[1], id2);
    }

    #[tokio::test]
    async fn test_search_traces() {
        let store = TraceStore::new(100);

        store
            .start_trace(Uuid::new_v4(), "create project".to_string())
            .await
            .unwrap();
        store
            .start_trace(Uuid::new_v4(), "list files".to_string())
            .await
            .unwrap();
        store
            .start_trace(Uuid::new_v4(), "create file".to_string())
            .await
            .unwrap();

        let results = store.search_traces("create").await;
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_max_traces_limit() {
        let store = TraceStore::new(3);

        for i in 0..5 {
            store
                .start_trace(Uuid::new_v4(), format!("trace {}", i))
                .await
                .unwrap();
        }

        let len = store.len().await;
        assert_eq!(len, 3); // 只保留最近的 3 个
    }

    #[tokio::test]
    async fn test_get_stats() {
        let store = TraceStore::new(100);

        let trace_id = Uuid::new_v4();
        store
            .start_trace(trace_id, "test".to_string())
            .await
            .unwrap();

        let mut span = ExecutionSpan::new(
            Uuid::new_v4(),
            trace_id,
            None,
            "root",
            SpanType::UserInput,
        );
        span.set_success();

        store.record_span(span).await.unwrap();
        store.finish_trace(trace_id).await.unwrap();

        let stats = store.get_stats().await;
        assert_eq!(stats.total_traces, 1);
        assert_eq!(stats.success_traces, 1);
        assert_eq!(stats.failed_traces, 0);
    }
}
