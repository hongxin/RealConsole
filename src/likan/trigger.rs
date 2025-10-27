//! 离坎炼化炉 - 手动触发器
//!
//! 提供手动触发炼化循环的能力

use super::types::CycleReport;
use super::LiKanFurnace;
use crate::conversation::ContextManager;
use crate::execution_logger::ExecutionLogger;
use crate::history::HistoryManager;
use crate::llm::LlmLogger;
use crate::suggestion::feedback::FeedbackStorage; // ✨ Phase 4.4: 反馈系统集成
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
    feedback_storage: Option<Arc<RwLock<FeedbackStorage>>>, // ✨ Phase 4.4: 反馈统计
    bagua_palace: Option<Arc<RwLock<crate::bagua::BaguaMemoryPalace>>>, // ✨ v1.8.4: 八卦记忆宫
}

impl LiKanTrigger {
    /// 创建新的触发器
    pub fn new(
        furnace: Arc<RwLock<LiKanFurnace>>,
        history: Arc<RwLock<HistoryManager>>,
        exec_logger: Arc<RwLock<ExecutionLogger>>,
        llm_logger: Option<Arc<LlmLogger>>,
        context_manager: Arc<RwLock<ContextManager>>,
        feedback_storage: Option<Arc<RwLock<FeedbackStorage>>>, // ✨ Phase 4.4: 新增参数
        bagua_palace: Option<Arc<RwLock<crate::bagua::BaguaMemoryPalace>>>, // ✨ v1.8.4: 八卦记忆宫
    ) -> Self {
        Self {
            furnace,
            history,
            exec_logger,
            llm_logger,
            context_manager,
            feedback_storage, // ✨ Phase 4.4: 存储反馈统计
            bagua_palace,     // ✨ v1.8.4: 存储八卦记忆宫
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

        // ✨ Phase 4.4: 从 FeedbackStorage 获取反馈统计
        let stats = if let Some(ref storage) = self.feedback_storage {
            match storage.read().await.load_stats().await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("⚠️ 无法加载反馈统计: {}", e);
                    std::collections::HashMap::new()
                }
            }
        } else {
            std::collections::HashMap::new() // 降级到空统计
        };

        // ✨ v1.8.4: 执行炼化循环（带八卦记忆宫）
        let report = if let Some(ref palace) = self.bagua_palace {
            // 先锁定八卦记忆宫
            let palace_guard = palace.read().await;

            // 再锁定炼化炉并执行
            let mut f = self.furnace.write().await;
            f.cycle_once(&entries, &stats, Some(&*palace_guard))
                .await
                .context("炼化循环执行失败")?
        } else {
            // 不使用八卦记忆宫
            let mut f = self.furnace.write().await;
            f.cycle_once(&entries, &stats, None)
                .await
                .context("炼化循环执行失败")?
        };

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
        use crate::config::ConversationConfig;
        use std::path::PathBuf;

        let furnace = Arc::new(RwLock::new(LiKanFurnace::new(FurnaceConfig::default())));
        let history = Arc::new(RwLock::new(HistoryManager::new(
            PathBuf::from("/tmp/test"),
            100,
        )));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let llm_logger = None; // 测试时不需要 LLM logger
        let context = Arc::new(RwLock::new(ContextManager::new(
            ConversationConfig::default(),
        )));

        let feedback_storage = None; // 测试时不需要反馈存储
        let bagua_palace = None; // ✨ v1.8.4: 测试时不需要八卦记忆宫
        let trigger = LiKanTrigger::new(
            furnace,
            history,
            exec_logger,
            llm_logger,
            context,
            feedback_storage,
            bagua_palace, // ✨ v1.8.4
        );

        // 应该可以触发第一次循环
        assert!(trigger.should_cycle().await);
    }
}
