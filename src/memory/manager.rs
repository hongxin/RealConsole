//! MemoryManager 适配层 (v1.16.0 Phase 3)
//!
//! 提供与原有 Memory API 兼容的接口，底层使用 UnifiedTracer 进行存储
//!
//! # 设计目标
//!
//! - **API 兼容**: 保持原有 Memory 模块的公共 API
//! - **统一存储**: 底层使用 UnifiedTracer 的 Memory 维度
//! - **透明转换**: MemoryEntry ↔ TraceEntry 自动转换
//! - **增强查询**: 支持基于 tags 和 importance 的高级查询

use super::memory_core::{EntryType as MemoryEntryType, Importance as MemoryImportance};
use super::memory_core::{MemoryEntry, MemoryStats};
use crate::tracer::unified_tracer::UnifiedTracer;
use crate::tracer::{Dimension, EntryType, Importance, Status, TraceEntry};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MemoryManager 适配层
///
/// 兼容原有 Memory API，底层使用 UnifiedTracer
pub struct MemoryManager {
    /// 底层统一追踪器
    tracer: Arc<UnifiedTracer>,

    /// 容量限制（用于兼容原有行为）
    capacity: usize,
}

impl MemoryManager {
    /// 使用已存在的 UnifiedTracer 创建 MemoryManager
    ///
    /// # 参数
    /// - `tracer`: 统一追踪器实例
    /// - `capacity`: 最大容量（用于兼容）
    ///
    /// # 示例
    /// ```rust,no_run
    /// use realconsole::memory::manager::MemoryManager;
    /// use realconsole::tracer::UnifiedTracer;
    /// use std::sync::Arc;
    ///
    /// // 假设已有 tracer
    /// let tracer = Arc::new(tracer);
    /// let manager = MemoryManager::new(tracer, 100);
    /// ```
    pub fn new(tracer: Arc<UnifiedTracer>, capacity: usize) -> Self {
        Self { tracer, capacity }
    }

    /// 添加记忆条目
    ///
    /// # 参数
    /// - `content`: 记忆内容
    /// - `entry_type`: 条目类型
    ///
    /// # 示例
    /// ```rust,no_run
    /// use realconsole::memory::manager::MemoryManager;
    /// use realconsole::memory::EntryType;
    ///
    /// # async {
    /// manager.add("Hello, world!".to_string(), EntryType::User).await;
    /// # };
    /// ```
    pub async fn add(&self, content: String, entry_type: MemoryEntryType) {
        let trace_entry = self.memory_entry_to_trace(&MemoryEntry::new(content, entry_type));
        self.tracer.add_entry(trace_entry).await;
    }

    /// 添加带重要性的记忆条目
    pub async fn add_with_importance(
        &self,
        content: String,
        entry_type: MemoryEntryType,
        importance: MemoryImportance,
    ) {
        let memory_entry = MemoryEntry::new_with_importance(content, entry_type, importance);
        let trace_entry = self.memory_entry_to_trace(&memory_entry);
        self.tracer.add_entry(trace_entry).await;
    }

    /// 获取最近的 N 条记忆
    ///
    /// # 参数
    /// - `n`: 返回的条目数量
    ///
    /// # 返回
    /// 最近的 N 条记忆（按时间倒序）
    pub async fn recent(&self, n: usize) -> Result<Vec<MemoryEntry>> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, n)
            .await?;

        Ok(entries
            .into_iter()
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 搜索包含关键词的记忆
    ///
    /// # 参数
    /// - `keyword`: 搜索关键词
    ///
    /// # 返回
    /// 包含关键词的所有记忆条目
    pub async fn search(&self, keyword: &str) -> Result<Vec<MemoryEntry>> {
        let results = self.tracer.search(keyword).await?;

        Ok(results
            .into_iter()
            .filter(|e| e.dimension == Dimension::Memory)
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 获取所有记忆
    pub async fn dump(&self) -> Result<Vec<MemoryEntry>> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        Ok(entries
            .into_iter()
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 获取记忆数量
    pub async fn len(&self) -> usize {
        self.tracer.custom_entries_count().await
    }

    /// 检查记忆是否为空
    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    /// 获取特定类型的记忆
    pub async fn filter_by_type(&self, entry_type: MemoryEntryType) -> Result<Vec<MemoryEntry>> {
        let all_entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        let trace_type = self.memory_type_to_trace_type(entry_type);

        Ok(all_entries
            .into_iter()
            .filter(|e| e.entry_type == trace_type)
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 按重要性过滤记忆
    pub async fn filter_by_importance(
        &self,
        importance: MemoryImportance,
    ) -> Result<Vec<MemoryEntry>> {
        let all_entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        let trace_importance = self.memory_importance_to_trace(importance);

        Ok(all_entries
            .into_iter()
            .filter(|e| e.importance == Some(trace_importance))
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 获取记忆统计信息
    pub async fn stats(&self) -> Result<MemoryStats> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        let mut type_distribution = HashMap::new();

        // 统计各类型数量
        for entry in &entries {
            let memory_type = self.trace_type_to_memory_type(&entry.entry_type);
            *type_distribution.entry(memory_type).or_insert(0) += 1;
        }

        // 获取最早和最新的时间戳
        let earliest_timestamp = entries.iter().map(|e| e.timestamp).min();
        let latest_timestamp = entries.iter().map(|e| e.timestamp).max();

        Ok(MemoryStats {
            total_entries: entries.len(),
            type_distribution,
            earliest_timestamp,
            latest_timestamp,
        })
    }

    // ━━━━━ v1.16.0 Phase 3: 增强查询方法 ━━━━━

    /// 根据标签搜索记忆
    ///
    /// # 参数
    /// - `tag`: 标签名称
    ///
    /// # 返回
    /// 包含指定标签的所有记忆条目
    pub async fn search_by_tag(&self, tag: &str) -> Result<Vec<MemoryEntry>> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        Ok(entries
            .into_iter()
            .filter(|e| e.has_tag(tag))
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 查找重要及以上级别的记忆
    ///
    /// # 返回
    /// 所有重要性为 Important 或 Critical 的记忆
    pub async fn find_important(&self) -> Result<Vec<MemoryEntry>> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        Ok(entries
            .into_iter()
            .filter(|e| {
                matches!(
                    e.importance,
                    Some(Importance::Important) | Some(Importance::Critical)
                )
            })
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    /// 根据上下文 ID 查找相关记忆
    pub async fn find_by_context(&self, context_id: &str) -> Result<Vec<MemoryEntry>> {
        let entries = self
            .tracer
            .query_by_dimension(Dimension::Memory, self.capacity)
            .await?;

        Ok(entries
            .into_iter()
            .filter(|e| e.get_context_id() == Some(context_id))
            .map(|e| self.trace_to_memory_entry(&e))
            .collect())
    }

    // ━━━━━ 内部转换方法 ━━━━━

    /// MemoryEntry 转换为 TraceEntry
    fn memory_entry_to_trace(&self, entry: &MemoryEntry) -> TraceEntry {
        let mut trace_entry = TraceEntry::new(
            Dimension::Memory,
            self.memory_type_to_trace_type(entry.entry_type),
            entry.content.clone(),
            Status::Success,
        );

        // 设置时间戳（保持原有时间）
        trace_entry.timestamp = entry.timestamp;

        // 设置重要性
        let importance = self.memory_importance_to_trace(entry.importance);
        trace_entry.set_importance(importance);

        // 在 metadata 中保存原始 MemoryEntryType（v1.16.0 Phase 4 修复）
        // 避免 ContextMessage 映射时丢失 User/Assistant 区分
        trace_entry.add_metadata(
            "original_memory_type".to_string(),
            serde_json::json!(entry.entry_type.to_string()),
        );

        trace_entry
    }

    /// TraceEntry 转换为 MemoryEntry
    fn trace_to_memory_entry(&self, entry: &TraceEntry) -> MemoryEntry {
        // v1.16.0 Phase 4 修复：优先从 metadata 恢复原始类型
        // 避免 ContextMessage → User 映射时丢失 Assistant 类型
        let memory_type = entry
            .get_metadata("original_memory_type")
            .and_then(|v| v.as_str())
            .and_then(|s| {
                use std::str::FromStr;
                MemoryEntryType::from_str(s).ok()
            })
            .unwrap_or_else(|| self.trace_type_to_memory_type(&entry.entry_type));

        let importance = entry
            .importance
            .map(|i| self.trace_importance_to_memory(i))
            .unwrap_or(MemoryImportance::Normal);

        MemoryEntry {
            timestamp: entry.timestamp,
            entry_type: memory_type,
            content: entry.content.clone(),
            importance,
        }
    }

    /// Memory EntryType 转换为 Trace EntryType
    fn memory_type_to_trace_type(&self, memory_type: MemoryEntryType) -> EntryType {
        match memory_type {
            MemoryEntryType::User => EntryType::ContextMessage,
            MemoryEntryType::Assistant => EntryType::ContextMessage,
            MemoryEntryType::System => EntryType::SystemEvent,
            MemoryEntryType::Shell => EntryType::ShellCommand,
            MemoryEntryType::Tool => EntryType::ToolInvocation,
        }
    }

    /// Trace EntryType 转换为 Memory EntryType
    fn trace_type_to_memory_type(&self, trace_type: &EntryType) -> MemoryEntryType {
        match trace_type {
            EntryType::ContextMessage => MemoryEntryType::User, // 默认映射
            EntryType::SystemEvent => MemoryEntryType::System,
            EntryType::ShellCommand => MemoryEntryType::Shell,
            EntryType::ToolInvocation => MemoryEntryType::Tool,
            _ => MemoryEntryType::System, // 其他类型默认为 System
        }
    }

    /// Memory Importance 转换为 Trace Importance
    fn memory_importance_to_trace(&self, memory_importance: MemoryImportance) -> Importance {
        match memory_importance {
            MemoryImportance::Normal => Importance::Normal,
            MemoryImportance::Important => Importance::Important,
            MemoryImportance::Critical => Importance::Critical,
        }
    }

    /// Trace Importance 转换为 Memory Importance
    fn trace_importance_to_memory(&self, trace_importance: Importance) -> MemoryImportance {
        match trace_importance {
            Importance::Low | Importance::Normal => MemoryImportance::Normal,
            Importance::Important => MemoryImportance::Important,
            Importance::Critical => MemoryImportance::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conversation::context_manager::ContextManager;
    use crate::execution_logger::ExecutionLogger;
    use crate::history::HistoryManager;

    fn create_test_tracer() -> Arc<UnifiedTracer> {
        use crate::config::settings::ConversationConfig;
        use std::path::PathBuf;

        let history = Arc::new(RwLock::new(HistoryManager::new(
            PathBuf::from("/tmp/test_history.jsonl"),
            100,
        )));
        let exec_logger = Arc::new(RwLock::new(ExecutionLogger::new(100)));
        let context = Arc::new(RwLock::new(ContextManager::new(
            ConversationConfig::default(),
        )));

        Arc::new(UnifiedTracer::new(history, exec_logger, None, context))
    }

    #[tokio::test]
    async fn test_memory_manager_creation() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);
        assert_eq!(manager.capacity, 100);
    }

    #[tokio::test]
    async fn test_add_and_recent() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        manager
            .add("Hello".to_string(), MemoryEntryType::User)
            .await;
        manager
            .add("World".to_string(), MemoryEntryType::User)
            .await;

        let recent = manager.recent(2).await.unwrap();
        assert_eq!(recent.len(), 2);
    }

    #[tokio::test]
    async fn test_search() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        manager
            .add("Hello Rust".to_string(), MemoryEntryType::User)
            .await;
        manager
            .add("Hello World".to_string(), MemoryEntryType::User)
            .await;
        manager
            .add("Goodbye".to_string(), MemoryEntryType::User)
            .await;

        let results = manager.search("Hello").await.unwrap();
        assert_eq!(results.len(), 2);

        let results = manager.search("Rust").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_add_with_importance() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        manager
            .add_with_importance(
                "Important message".to_string(),
                MemoryEntryType::User,
                MemoryImportance::Important,
            )
            .await;

        let important = manager
            .filter_by_importance(MemoryImportance::Important)
            .await
            .unwrap();
        assert_eq!(important.len(), 1);
    }

    #[tokio::test]
    async fn test_find_important() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        manager
            .add("Normal".to_string(), MemoryEntryType::User)
            .await;
        manager
            .add_with_importance(
                "Important".to_string(),
                MemoryEntryType::User,
                MemoryImportance::Important,
            )
            .await;
        manager
            .add_with_importance(
                "Critical".to_string(),
                MemoryEntryType::User,
                MemoryImportance::Critical,
            )
            .await;

        let important = manager.find_important().await.unwrap();
        assert_eq!(important.len(), 2);
    }

    // ━━━━━ v1.16.0 Phase 4: Bug 修复测试 ━━━━━

    #[tokio::test]
    async fn test_assistant_message_type_preservation() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        // 添加不同类型的消息
        manager
            .add("用户问题".to_string(), MemoryEntryType::User)
            .await;
        manager
            .add("AI 回复".to_string(), MemoryEntryType::Assistant)
            .await;
        manager
            .add("系统消息".to_string(), MemoryEntryType::System)
            .await;

        // 通过 dump 读回（会经过 TraceEntry 转换）
        let entries = manager.dump().await.unwrap();

        // 验证类型保留正确（dump 返回按时间倒序，最新的在前）
        assert_eq!(entries.len(), 3);

        // 最新的条目（系统消息）
        assert_eq!(entries[0].entry_type, MemoryEntryType::System);
        assert_eq!(entries[0].content, "系统消息");

        // 第二新的条目（AI 回复）
        assert_eq!(entries[1].entry_type, MemoryEntryType::Assistant); // Phase 4 修复：不再变成 User
        assert_eq!(entries[1].content, "AI 回复");

        // 最早的条目（用户问题）
        assert_eq!(entries[2].entry_type, MemoryEntryType::User);
        assert_eq!(entries[2].content, "用户问题");
    }

    #[tokio::test]
    async fn test_all_entry_types_roundtrip() {
        let tracer = create_test_tracer();
        let manager = MemoryManager::new(tracer, 100);

        // 添加所有类型的消息
        let types = vec![
            (MemoryEntryType::User, "用户输入"),
            (MemoryEntryType::Assistant, "助手响应"),
            (MemoryEntryType::System, "系统消息"),
            (MemoryEntryType::Shell, "Shell 命令"),
            (MemoryEntryType::Tool, "工具调用"),
        ];

        for (entry_type, content) in &types {
            manager.add(content.to_string(), *entry_type).await;
        }

        // 读回并验证类型完全保留（返回顺序是倒序）
        let entries = manager.dump().await.unwrap();
        assert_eq!(entries.len(), 5);

        // dump() 返回倒序，所以需要反转预期顺序
        let types_reversed: Vec<_> = types.iter().rev().collect();

        for (i, (expected_type, expected_content)) in types_reversed.iter().enumerate() {
            assert_eq!(
                entries[i].entry_type, *expected_type,
                "位置 {} 的类型 {:?} 未正确保留，实际是 {:?}",
                i,
                expected_type,
                entries[i].entry_type
            );
            assert_eq!(
                entries[i].content, *expected_content,
                "位置 {} 的内容不匹配",
                i
            );
        }
    }
}
