//! 八卦记忆宫殿核心实现
//!
//! 八维记忆空间的管理和查询

use super::dimension::BaguaDimension;
use super::entry::{MemoryEntry, MemoryContent};
use super::storage::BaguaStorage;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// ✨ v1.15.0 Phase 3: Tracer 集成
use crate::tracer::{Dimension as TracerDimension, EntryType as TracerEntryType, Status, TraceEntry, UnifiedTracer};

/// 八卦记忆宫殿
///
/// 管理八维记忆空间的核心结构
pub struct BaguaMemoryPalace {
    /// 八维存储（每个维度独立存储）
    dimensions: HashMap<BaguaDimension, Arc<RwLock<Vec<MemoryEntry>>>>,

    /// 配置
    config: PalaceConfig,

    /// 持久化存储 (✨ v1.8.4 Phase 4)
    storage: Option<Arc<BaguaStorage>>,

    /// ✨ v1.15.0 Phase 3: 统一追踪器（可选）
    ///
    /// 用于记录记忆存储、炼化过程到 Memory 维度
    tracer: Option<Arc<UnifiedTracer>>,
}

/// 宫殿配置
#[derive(Debug, Clone)]
pub struct PalaceConfig {
    /// 每个维度的最大条目数
    pub max_entries_per_dimension: usize,

    /// 能量衰减率（每天）
    pub energy_decay_rate: f64,

    /// 相关性阈值（低于此值的条目将被清理）
    pub relevance_threshold: f64,
}

impl Default for PalaceConfig {
    fn default() -> Self {
        Self {
            max_entries_per_dimension: 1000,
            energy_decay_rate: 0.95,
            relevance_threshold: 0.1,
        }
    }
}

impl BaguaMemoryPalace {
    /// 创建新的记忆宫殿
    pub fn new() -> Self {
        Self::with_config(PalaceConfig::default())
    }

    /// 使用自定义配置创建
    pub fn with_config(config: PalaceConfig) -> Self {
        let mut dimensions = HashMap::new();

        // 初始化八个维度
        for dim in BaguaDimension::all() {
            dimensions.insert(dim, Arc::new(RwLock::new(Vec::new())));
        }

        Self {
            dimensions,
            config,
            storage: None,
            tracer: None, // ✨ v1.15.0 Phase 3
        }
    }

    /// 使用持久化存储创建 (✨ v1.8.4 Phase 4)
    pub fn with_storage(config: PalaceConfig, storage: BaguaStorage) -> Self {
        let mut dimensions = HashMap::new();

        // 初始化八个维度
        for dim in BaguaDimension::all() {
            dimensions.insert(dim, Arc::new(RwLock::new(Vec::new())));
        }

        Self {
            dimensions,
            config,
            storage: Some(Arc::new(storage)),
            tracer: None, // ✨ v1.15.0 Phase 3
        }
    }

    /// ✨ v1.15.0 Phase 3: 设置 Tracer
    ///
    /// 启用后，所有记忆操作将记录到 Tracer 的 Memory 维度
    pub fn set_tracer(&mut self, tracer: Arc<UnifiedTracer>) {
        self.tracer = Some(tracer);
    }

    /// ✨ v1.15.0 Phase 3: 检查是否启用了 Tracer
    pub fn is_tracer_enabled(&self) -> bool {
        self.tracer.is_some()
    }

    /// 存储记忆条目
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> {
        let dimension = entry.dimension;
        let mut was_pruned = false;

        if let Some(storage) = self.dimensions.get(&dimension) {
            let mut entries = storage.write().await;
            entries.push(entry.clone());

            // 如果超过最大数量，移除最旧的
            if entries.len() > self.config.max_entries_per_dimension {
                entries.remove(0);
                was_pruned = true;
            }
        }

        // ✨ v1.8.4 Phase 4: 同步持久化到磁盘
        self.save_dimension_to_storage(dimension, &entry).await?;

        // ✨ v1.15.0 Phase 3: 记录存储事件到 Tracer
        if let Some(tracer) = &self.tracer {
            let content = format!(
                "记忆存储: {} | {} | 能量={:.2}{}",
                dimension,
                Self::preview_content(&entry.content, 50),
                entry.energy,
                if was_pruned { " (淘汰旧记忆)" } else { "" }
            );

            let trace_entry = TraceEntry::new(
                TracerDimension::Memory,
                TracerEntryType::SystemEvent,
                content,
                Status::Success,
            );

            tracer.add_entry(trace_entry).await;
        }

        Ok(())
    }

    /// ✨ v1.15.0 Phase 3: 辅助方法 - 预览记忆内容
    fn preview_content(content: &MemoryContent, max_len: usize) -> String {
        let text = match content {
            MemoryContent::Intent { goal, .. } => format!("意图: {}", goal),
            MemoryContent::Conversation { role, message, .. } => {
                format!("{}: {}", role, message)
            }
            MemoryContent::Action { command, .. } => format!("命令: {}", command),
            MemoryContent::Trend { pattern, .. } => format!("趋势: {}", pattern),
            MemoryContent::Pattern { pattern_type, .. } => {
                let desc = match pattern_type {
                    super::entry::PatternType::Frequency { command, .. } => {
                        format!("频率模式: {}", command)
                    }
                    super::entry::PatternType::Sequence { commands, .. } => {
                        format!("序列模式: {} 命令", commands.len())
                    }
                    super::entry::PatternType::ErrorFix { error_pattern, .. } => {
                        format!("修复模式: {}", error_pattern)
                    }
                };
                desc
            }
            MemoryContent::Knowledge { fact, .. } => format!("知识: {}", fact),
            MemoryContent::Checkpoint { state, .. } => format!("快照: {}", state),
            MemoryContent::Feedback { action, .. } => format!("反馈: {}", action),
        };

        if text.len() > max_len {
            let truncated = &text[..text.char_indices().nth(max_len).map(|(i, _)| i).unwrap_or(max_len)];
            format!("{}...", truncated)
        } else {
            text
        }
    }

    /// 从指定维度检索记忆
    pub async fn retrieve(
        &self,
        dimension: BaguaDimension,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryEntry>> {
        if let Some(storage) = self.dimensions.get(&dimension) {
            let entries = storage.read().await;
            let limit = limit.unwrap_or(entries.len());

            Ok(entries.iter().rev().take(limit).cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// 获取维度统计信息
    pub async fn dimension_stats(&self, dimension: BaguaDimension) -> Result<DimensionStats> {
        if let Some(storage) = self.dimensions.get(&dimension) {
            let entries = storage.read().await;

            let count = entries.len();
            let total_energy: f64 = entries.iter().map(|e| e.energy).sum();
            let avg_energy = if count > 0 {
                total_energy / count as f64
            } else {
                0.0
            };

            let total_relevance: f64 = entries.iter().map(|e| e.relevance).sum();
            let avg_relevance = if count > 0 {
                total_relevance / count as f64
            } else {
                0.0
            };

            Ok(DimensionStats {
                dimension,
                count,
                avg_energy,
                avg_relevance,
            })
        } else {
            Ok(DimensionStats {
                dimension,
                count: 0,
                avg_energy: 0.0,
                avg_relevance: 0.0,
            })
        }
    }

    /// 分析所有维度的能量分布
    pub async fn analyze_energy(&self) -> HashMap<BaguaDimension, f64> {
        let mut result = HashMap::new();

        for dim in BaguaDimension::all() {
            if let Ok(stats) = self.dimension_stats(dim).await {
                result.insert(dim, stats.avg_energy);
            }
        }

        result
    }

    /// 检查离坎能量平衡
    pub async fn check_likan_balance(&self) -> LiKanBalance {
        let li_stats = self.dimension_stats(BaguaDimension::Li).await.unwrap();
        let kan_stats = self.dimension_stats(BaguaDimension::Kan).await.unwrap();

        LiKanBalance {
            li_energy: li_stats.avg_energy,
            kan_energy: kan_stats.avg_energy,
            li_count: li_stats.count,
            kan_count: kan_stats.count,
            balance: li_stats.avg_energy - kan_stats.avg_energy,
        }
    }

    // ============================================================
    // ✨ v1.8.4 Phase 4: 持久化方法
    // ============================================================

    /// 从存储加载所有维度的数据
    pub async fn load_from_storage(&self) -> Result<usize> {
        if let Some(ref storage) = self.storage {
            let mut total_loaded = 0;

            for dimension in BaguaDimension::all() {
                // 从存储加载数据（不限制数量，使用配置的容量）
                let entries = storage
                    .load_dimension(dimension, Some(self.config.max_entries_per_dimension))
                    .await?;

                if !entries.is_empty() {
                    // 写入内存
                    if let Some(dim_storage) = self.dimensions.get(&dimension) {
                        let mut dim_entries = dim_storage.write().await;
                        *dim_entries = entries.clone();
                        total_loaded += entries.len();
                    }
                }
            }

            Ok(total_loaded)
        } else {
            Ok(0)
        }
    }

    /// 保存所有维度到存储
    pub async fn save_to_storage(&self) -> Result<usize> {
        if let Some(ref storage) = self.storage {
            let mut total_saved = 0;

            for dimension in BaguaDimension::all() {
                if let Some(dim_storage) = self.dimensions.get(&dimension) {
                    let entries = dim_storage.read().await;
                    if !entries.is_empty() {
                        storage
                            .save_dimension(dimension, &entries)
                            .await?;
                        total_saved += entries.len();
                    }
                }
            }

            Ok(total_saved)
        } else {
            Ok(0)
        }
    }

    /// 保存单个维度到存储（追加模式）
    async fn save_dimension_to_storage(
        &self,
        dimension: BaguaDimension,
        entry: &MemoryEntry,
    ) -> Result<()> {
        if let Some(ref storage) = self.storage {
            storage.append_dimension(dimension, &[entry.clone()]).await?;
        }
        Ok(())
    }

    /// 清理过期数据
    pub async fn cleanup_expired(&self, retention_days: u64) -> Result<usize> {
        if let Some(ref storage) = self.storage {
            let mut total_removed = 0;

            for dimension in BaguaDimension::all() {
                let removed = storage.cleanup_expired(dimension, retention_days).await?;
                total_removed += removed;

                // 如果有删除，重新加载该维度
                if removed > 0 {
                    let entries = storage
                        .load_dimension(dimension, Some(self.config.max_entries_per_dimension))
                        .await?;

                    if let Some(dim_storage) = self.dimensions.get(&dimension) {
                        let mut dim_entries = dim_storage.write().await;
                        *dim_entries = entries;
                    }
                }
            }

            Ok(total_removed)
        } else {
            Ok(0)
        }
    }
}

impl Default for BaguaMemoryPalace {
    fn default() -> Self {
        Self::new()
    }
}

/// 维度统计信息
#[derive(Debug, Clone)]
pub struct DimensionStats {
    pub dimension: BaguaDimension,
    pub count: usize,
    pub avg_energy: f64,
    pub avg_relevance: f64,
}

/// 离坎能量平衡
#[derive(Debug, Clone)]
pub struct LiKanBalance {
    pub li_energy: f64,
    pub kan_energy: f64,
    pub li_count: usize,
    pub kan_count: usize,
    pub balance: f64, // > 0 离强，< 0 坎强
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let palace = BaguaMemoryPalace::new();

        let entry = MemoryEntry::new(
            BaguaDimension::Qian,
            MemoryContent::Intent {
                goal: "测试目标".to_string(),
                context: None,
                priority: 0.8,
            },
        );

        palace.store(entry).await.unwrap();

        let retrieved = palace
            .retrieve(BaguaDimension::Qian, Some(10))
            .await
            .unwrap();

        assert_eq!(retrieved.len(), 1);
    }

    #[tokio::test]
    async fn test_dimension_stats() {
        let palace = BaguaMemoryPalace::new();

        // 存储一些条目
        for _ in 0..5 {
            let entry = MemoryEntry::new(
                BaguaDimension::Li,
                MemoryContent::Knowledge {
                    fact: "测试知识".to_string(),
                    source: super::super::entry::KnowledgeSource::SystemObserved,
                    confidence: 0.9,
                },
            );
            palace.store(entry).await.unwrap();
        }

        let stats = palace.dimension_stats(BaguaDimension::Li).await.unwrap();

        assert_eq!(stats.count, 5);
        assert!(stats.avg_energy > 0.0);
    }

    #[tokio::test]
    async fn test_likan_balance() {
        let palace = BaguaMemoryPalace::new();

        // 添加离维度条目
        for _ in 0..3 {
            let entry = MemoryEntry::new(
                BaguaDimension::Li,
                MemoryContent::Knowledge {
                    fact: "知识".to_string(),
                    source: super::super::entry::KnowledgeSource::SystemObserved,
                    confidence: 0.9,
                },
            );
            palace.store(entry).await.unwrap();
        }

        // 添加坎维度条目
        for _ in 0..2 {
            let entry = MemoryEntry::new(
                BaguaDimension::Kan,
                MemoryContent::Pattern {
                    pattern_type: super::super::entry::PatternType::Frequency {
                        command: "test".to_string(),
                        count: 10,
                    },
                    confidence: 0.8,
                    occurrences: 10,
                },
            );
            palace.store(entry).await.unwrap();
        }

        let balance = palace.check_likan_balance().await;

        assert_eq!(balance.li_count, 3);
        assert_eq!(balance.kan_count, 2);
        // 离能量应该高于坎
        assert!(balance.balance > 0.0);
    }

    #[tokio::test]
    async fn test_tracer_integration() {
        use crate::tracer::{UnifiedTracer, Dimension as TracerDimension, EntryType as TracerEntryType};
        use crate::config::ConversationConfig;
        use crate::execution_logger::ExecutionLogger;
        use crate::history::HistoryManager;
        use crate::conversation::context_manager::ContextManager;

        // 创建 Tracer 所需的依赖
        let history = Arc::new(RwLock::new(HistoryManager::new("/tmp/test_bagua_history.json", 100)));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let context = Arc::new(RwLock::new(ContextManager::new(ConversationConfig::default())));
        let tracer = Arc::new(UnifiedTracer::new(history, exec_logger, None, context));

        // 创建带 Tracer 的 Palace
        let mut palace = BaguaMemoryPalace::new();
        palace.set_tracer(tracer.clone());

        assert!(palace.is_tracer_enabled(), "Tracer 应该已启用");

        // 存储几个不同维度的记忆
        let entries = vec![
            MemoryEntry::new(
                BaguaDimension::Qian,
                MemoryContent::Intent {
                    goal: "测试意图".to_string(),
                    context: None,
                    priority: 0.9,
                },
            ),
            MemoryEntry::new(
                BaguaDimension::Dui,
                MemoryContent::Conversation {
                    role: "user".to_string(),
                    message: "你好".to_string(),
                    session_id: None,
                },
            ),
            MemoryEntry::new(
                BaguaDimension::Li,
                MemoryContent::Knowledge {
                    fact: "重要知识点".to_string(),
                    source: super::super::entry::KnowledgeSource::SystemObserved,
                    confidence: 0.95,
                },
            ),
        ];

        for entry in entries {
            palace.store(entry).await.unwrap();
        }

        // 验证 Tracer 记录了事件
        let memory_entries = tracer
            .query_by_dimension(TracerDimension::Memory, 10)
            .await
            .unwrap();

        assert_eq!(
            memory_entries.len(),
            3,
            "应该有 3 条 Memory 维度的记录"
        );

        // 验证事件类型和内容
        for trace_entry in &memory_entries {
            assert_eq!(trace_entry.entry_type, TracerEntryType::SystemEvent);
            assert!(trace_entry.content.contains("记忆存储:"));
            assert!(trace_entry.content.contains("能量="));
        }

        // 验证不同维度的记录都被记录了（不关心顺序，因为是按时间倒序）
        let all_content: String = memory_entries.iter().map(|e| e.content.clone()).collect();
        assert!(all_content.contains("Qian"), "应该有 Qian 维度的记录");
        assert!(all_content.contains("Dui"), "应该有 Dui 维度的记录");
        assert!(all_content.contains("Li"), "应该有 Li 维度的记录");
    }
}
