//! 离坎炼化炉 - 手动触发器
//!
//! 提供手动触发炼化循环的能力

use super::types::CycleReport;
use super::LiKanFurnace;
use crate::conversation::ContextManager;
use crate::execution_logger::ExecutionLogger;
use crate::history::HistoryManager;
use crate::llm::LlmLogger;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 离坎炼化炉触发器
///
/// 封装手动触发所需的所有依赖
pub struct LiKanTrigger {
    furnace: Arc<RwLock<LiKanFurnace>>,
    history: Arc<RwLock<HistoryManager>>,
    exec_logger: Arc<RwLock<ExecutionLogger>>,
    llm_logger: Option<Arc<LlmLogger>>,
    context_manager: Arc<RwLock<ContextManager>>,
}

impl LiKanTrigger {
    /// 创建新的触发器
    pub fn new(
        furnace: Arc<RwLock<LiKanFurnace>>,
        history: Arc<RwLock<HistoryManager>>,
        exec_logger: Arc<RwLock<ExecutionLogger>>,
        llm_logger: Option<Arc<LlmLogger>>,
        context_manager: Arc<RwLock<ContextManager>>,
    ) -> Self {
        Self {
            furnace,
            history,
            exec_logger,
            llm_logger,
            context_manager,
        }
    }

    /// 手动触发一次炼化循环
    ///
    /// 返回循环报告
    pub async fn trigger_once(&self) -> Result<CycleReport> {
        // 创建 UnifiedTracer 获取数据
        let tracer = crate::tracer::UnifiedTracer::new(
            Arc::clone(&self.history),
            Arc::clone(&self.exec_logger),
            self.llm_logger.clone(),
            Arc::clone(&self.context_manager),
        );

        // 查询最近200条记录
        let entries = tracer
            .query_all(200)
            .await
            .context("查询追踪数据失败")?;

        // 暂时使用空的 suggestion stats（Phase 4.4 可集成反馈系统）
        let stats = std::collections::HashMap::new();

        // 执行炼化循环
        let mut f = self.furnace.write().await;
        let report = f
            .cycle_once(&entries, &stats)
            .await
            .context("炼化循环执行失败")?;

        Ok(report)
    }

    /// 检查炼化炉是否应该触发
    pub async fn should_cycle(&self) -> bool {
        let f = self.furnace.read().await;
        f.should_cycle()
    }

    /// 获取距离上次循环的时间（秒）
    pub async fn time_since_last_cycle(&self) -> Option<u64> {
        let f = self.furnace.read().await;
        f.time_since_last_cycle()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::likan::types::FurnaceConfig;

    #[tokio::test]
    async fn test_trigger_creation() {
        let furnace = Arc::new(RwLock::new(LiKanFurnace::new(FurnaceConfig::default())));
        let history = Arc::new(RwLock::new(HistoryManager::new(None)));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(None)));
        let llm_logger = Some(Arc::new(LlmLogger::new(None)));
        let context = Arc::new(RwLock::new(ContextManager::new()));

        let trigger = LiKanTrigger::new(furnace, history, exec_logger, llm_logger, context);

        // 应该可以触发第一次循环
        assert!(trigger.should_cycle().await);
    }
}
