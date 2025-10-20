//! 状态管理器
//!
//! 统一管理 Agent 的所有状态，包括：
//! - Memory（记忆）
//! - History（历史）
//! - ContextTracker（上下文追踪）
//! - StatsCollector（统计收集）
//! - ExecutionLogger（执行日志）
//! - ContextManager（对话上下文）

use crate::conversation::ContextManager;
use crate::execution_logger::ExecutionLogger;
use crate::history::HistoryManager;
use crate::memory::{ContextTracker, Memory};
use crate::stats::StatsCollector;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 状态管理器
///
/// 集中管理所有状态相关的组件，提供统一的访问接口
pub struct StateManager {
    /// 记忆系统
    memory: Arc<RwLock<Memory>>,
    /// 历史管理
    history: Arc<RwLock<HistoryManager>>,
    /// 上下文追踪
    context_tracker: Arc<RwLock<ContextTracker>>,
    /// 统计收集器
    stats_collector: Arc<StatsCollector>,
    /// 执行日志
    exec_logger: Arc<RwLock<ExecutionLogger>>,
    /// 对话上下文管理器
    conversation_context: Arc<RwLock<ContextManager>>,
}

impl StateManager {
    /// 创建新的状态管理器
    pub fn new(
        memory: Arc<RwLock<Memory>>,
        history: Arc<RwLock<HistoryManager>>,
        context_tracker: Arc<RwLock<ContextTracker>>,
        stats_collector: Arc<StatsCollector>,
        exec_logger: Arc<RwLock<ExecutionLogger>>,
        conversation_context: Arc<RwLock<ContextManager>>,
    ) -> Self {
        Self {
            memory,
            history,
            context_tracker,
            stats_collector,
            exec_logger,
            conversation_context,
        }
    }

    // ========== Accessors ==========

    pub fn memory(&self) -> Arc<RwLock<Memory>> {
        Arc::clone(&self.memory)
    }

    pub fn history(&self) -> Arc<RwLock<HistoryManager>> {
        Arc::clone(&self.history)
    }

    pub fn context_tracker(&self) -> Arc<RwLock<ContextTracker>> {
        Arc::clone(&self.context_tracker)
    }

    pub fn stats_collector(&self) -> Arc<StatsCollector> {
        Arc::clone(&self.stats_collector)
    }

    pub fn exec_logger(&self) -> Arc<RwLock<ExecutionLogger>> {
        Arc::clone(&self.exec_logger)
    }

    pub fn conversation_context(&self) -> Arc<RwLock<ContextManager>> {
        Arc::clone(&self.conversation_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        use crate::config::ConversationConfig;

        let memory = Arc::new(RwLock::new(Memory::new(100)));
        let history = Arc::new(RwLock::new(HistoryManager::new("test_history.jsonl", 100)));
        let context_tracker = Arc::new(RwLock::new(ContextTracker::new()));
        let stats_collector = Arc::new(StatsCollector::new());
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let conversation_context =
            Arc::new(RwLock::new(ContextManager::new(ConversationConfig::default())));

        let state_manager = StateManager::new(
            memory,
            history,
            context_tracker,
            stats_collector,
            exec_logger,
            conversation_context,
        );

        // 验证可以访问各个组件
        assert!(Arc::strong_count(&state_manager.memory()) > 0);
        assert!(Arc::strong_count(&state_manager.history()) > 0);
        assert!(Arc::strong_count(&state_manager.conversation_context()) > 0);
    }
}
