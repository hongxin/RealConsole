//! 多维索引系统
//!
//! v1.56.0: 探路期核心功能 - 验证 10x 查询性能提升
//!
//! ## 设计目标
//!
//! 基于"一分为三"哲学的多维索引：
//! - **时间维度**: 按时间戳快速范围查询
//! - **类型维度**: 按维度/条目类型快速过滤
//! - **语义维度**: 按标签/关键词快速检索
//!
//! ## 索引结构
//!
//! ```text
//! MultiDimensionalIndex
//! ├── by_timestamp (BTreeMap)     - 时间范围查询 O(log n + k)
//! ├── by_dimension (HashMap)      - 维度过滤 O(1)
//! ├── by_entry_type (HashMap)     - 类型过滤 O(1)
//! ├── by_status (HashMap)         - 状态过滤 O(1)
//! ├── by_importance (BTreeMap)    - 重要性范围 O(log n + k)
//! ├── by_tag (HashMap)            - 标签查询 O(1)
//! └── by_content_hash (HashMap)   - 去重检索 O(1)
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::entry::TraceEntry;
use super::types::{Dimension, EntryType, Importance, Status};

/// 条目 ID（使用 UUID）
pub type EntryId = Uuid;

/// 多维索引
///
/// 为 TraceEntry 提供多维度的快速查询能力
#[derive(Debug)]
pub struct MultiDimensionalIndex {
    // ========== 时间维度 ==========
    /// 按时间戳索引（BTreeMap 支持范围查询）
    by_timestamp: BTreeMap<DateTime<Utc>, HashSet<EntryId>>,

    // ========== 类型维度 ==========
    /// 按维度索引
    by_dimension: HashMap<Dimension, HashSet<EntryId>>,

    /// 按条目类型索引
    by_entry_type: HashMap<EntryType, HashSet<EntryId>>,

    /// 按状态索引
    by_status: HashMap<StatusKey, HashSet<EntryId>>,

    // ========== 语义维度 ==========
    /// 按重要性索引（BTreeMap 支持范围查询）
    by_importance: BTreeMap<u8, HashSet<EntryId>>,

    /// 按标签索引（反向索引）
    by_tag: HashMap<String, HashSet<EntryId>>,

    // ========== 辅助索引 ==========
    /// 按内容哈希索引（用于快速去重）
    by_content_hash: HashMap<u64, HashSet<EntryId>>,

    /// 条目存储（主数据）
    entries: HashMap<EntryId, TraceEntry>,

    /// 统计信息
    stats: IndexStats,
}

/// 状态键（简化版，用于 HashMap）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusKey {
    Success,
    Failed,
    Running,
    Cancelled,
}

impl From<&Status> for StatusKey {
    fn from(status: &Status) -> Self {
        match status {
            Status::Success => StatusKey::Success,
            Status::Failed(_) => StatusKey::Failed,
            Status::Running => StatusKey::Running,
            Status::Cancelled => StatusKey::Cancelled,
        }
    }
}

/// 索引统计信息
#[derive(Debug, Default, Clone)]
pub struct IndexStats {
    /// 总条目数
    pub total_entries: usize,
    /// 索引构建耗时（微秒）
    pub build_time_us: u64,
    /// 估算内存使用（字节）
    pub estimated_memory_bytes: usize,
    /// 各维度条目计数
    pub dimension_counts: HashMap<Dimension, usize>,
    /// 各状态条目计数
    pub status_counts: HashMap<StatusKey, usize>,
}

/// 查询结果
#[derive(Debug)]
pub struct QueryResult {
    /// 匹配的条目
    pub entries: Vec<TraceEntry>,
    /// 查询耗时（微秒）
    pub query_time_us: u64,
    /// 扫描的索引条目数
    pub scanned_count: usize,
}

impl MultiDimensionalIndex {
    /// 创建空索引
    pub fn new() -> Self {
        Self {
            by_timestamp: BTreeMap::new(),
            by_dimension: HashMap::new(),
            by_entry_type: HashMap::new(),
            by_status: HashMap::new(),
            by_importance: BTreeMap::new(),
            by_tag: HashMap::new(),
            by_content_hash: HashMap::new(),
            entries: HashMap::new(),
            stats: IndexStats::default(),
        }
    }

    /// 从条目列表构建索引
    pub fn build_from(entries: Vec<TraceEntry>) -> Self {
        let start = std::time::Instant::now();
        let mut index = Self::new();

        for entry in entries {
            index.insert(entry);
        }

        index.stats.build_time_us = start.elapsed().as_micros() as u64;
        index.stats.estimated_memory_bytes = index.estimate_memory();

        index
    }

    /// 插入单个条目
    pub fn insert(&mut self, entry: TraceEntry) {
        let id = entry.id;

        // 更新时间索引
        self.by_timestamp
            .entry(entry.timestamp)
            .or_insert_with(HashSet::new)
            .insert(id);

        // 更新维度索引
        self.by_dimension
            .entry(entry.dimension.clone())
            .or_insert_with(HashSet::new)
            .insert(id);
        *self.stats.dimension_counts.entry(entry.dimension.clone()).or_insert(0) += 1;

        // 更新类型索引
        self.by_entry_type
            .entry(entry.entry_type.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        // 更新状态索引
        let status_key = StatusKey::from(&entry.status);
        self.by_status
            .entry(status_key)
            .or_insert_with(HashSet::new)
            .insert(id);
        *self.stats.status_counts.entry(status_key).or_insert(0) += 1;

        // 更新重要性索引
        if let Some(importance) = &entry.importance {
            let level = importance_to_level(importance);
            self.by_importance
                .entry(level)
                .or_insert_with(HashSet::new)
                .insert(id);
        }

        // 更新标签索引
        for tag in &entry.tags {
            self.by_tag
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(id);
        }

        // 更新内容哈希索引
        let hash = entry.content_hash();
        self.by_content_hash
            .entry(hash)
            .or_insert_with(HashSet::new)
            .insert(id);

        // 存储条目
        self.entries.insert(id, entry);
        self.stats.total_entries += 1;
    }

    /// 删除条目
    pub fn remove(&mut self, id: &EntryId) -> Option<TraceEntry> {
        if let Some(entry) = self.entries.remove(id) {
            // 清理所有索引
            if let Some(set) = self.by_timestamp.get_mut(&entry.timestamp) {
                set.remove(id);
            }
            if let Some(set) = self.by_dimension.get_mut(&entry.dimension) {
                set.remove(id);
            }
            if let Some(set) = self.by_entry_type.get_mut(&entry.entry_type) {
                set.remove(id);
            }
            let status_key = StatusKey::from(&entry.status);
            if let Some(set) = self.by_status.get_mut(&status_key) {
                set.remove(id);
            }
            if let Some(importance) = &entry.importance {
                let level = importance_to_level(importance);
                if let Some(set) = self.by_importance.get_mut(&level) {
                    set.remove(id);
                }
            }
            for tag in &entry.tags {
                if let Some(set) = self.by_tag.get_mut(tag) {
                    set.remove(id);
                }
            }
            let hash = entry.content_hash();
            if let Some(set) = self.by_content_hash.get_mut(&hash) {
                set.remove(id);
            }

            self.stats.total_entries = self.stats.total_entries.saturating_sub(1);

            Some(entry)
        } else {
            None
        }
    }

    // ========================================================================
    // 查询方法
    // ========================================================================

    /// 按时间范围查询
    ///
    /// 使用 BTreeMap 的范围查询，复杂度 O(log n + k)
    pub fn query_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> QueryResult {
        let query_start = std::time::Instant::now();
        let mut scanned = 0;

        let ids: Vec<EntryId> = self.by_timestamp
            .range(start..=end)
            .flat_map(|(_, ids)| {
                scanned += ids.len();
                ids.iter().copied()
            })
            .take(limit)
            .collect();

        let entries: Vec<TraceEntry> = ids
            .iter()
            .filter_map(|id| self.entries.get(id).cloned())
            .collect();

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 按维度查询
    ///
    /// 使用 HashMap 直接查找，复杂度 O(1) + O(k)
    pub fn query_by_dimension(&self, dimension: &Dimension, limit: usize) -> QueryResult {
        let query_start = std::time::Instant::now();

        let (entries, scanned) = if let Some(ids) = self.by_dimension.get(dimension) {
            let entries: Vec<TraceEntry> = ids
                .iter()
                .take(limit)
                .filter_map(|id| self.entries.get(id).cloned())
                .collect();
            (entries, ids.len())
        } else {
            (vec![], 0)
        };

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 按条目类型查询
    pub fn query_by_entry_type(&self, entry_type: &EntryType, limit: usize) -> QueryResult {
        let query_start = std::time::Instant::now();

        let (entries, scanned) = if let Some(ids) = self.by_entry_type.get(entry_type) {
            let entries: Vec<TraceEntry> = ids
                .iter()
                .take(limit)
                .filter_map(|id| self.entries.get(id).cloned())
                .collect();
            (entries, ids.len())
        } else {
            (vec![], 0)
        };

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 按状态查询
    pub fn query_by_status(&self, status: StatusKey, limit: usize) -> QueryResult {
        let query_start = std::time::Instant::now();

        let (entries, scanned) = if let Some(ids) = self.by_status.get(&status) {
            let entries: Vec<TraceEntry> = ids
                .iter()
                .take(limit)
                .filter_map(|id| self.entries.get(id).cloned())
                .collect();
            (entries, ids.len())
        } else {
            (vec![], 0)
        };

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 按标签查询（支持多标签 OR）
    pub fn query_by_tags(&self, tags: &[String], limit: usize) -> QueryResult {
        let query_start = std::time::Instant::now();
        let mut matched_ids: HashSet<EntryId> = HashSet::new();
        let mut scanned = 0;

        for tag in tags {
            if let Some(ids) = self.by_tag.get(tag) {
                scanned += ids.len();
                matched_ids.extend(ids);
            }
        }

        let entries: Vec<TraceEntry> = matched_ids
            .iter()
            .take(limit)
            .filter_map(|id| self.entries.get(id).cloned())
            .collect();

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 按重要性范围查询
    pub fn query_by_importance_range(
        &self,
        min_level: u8,
        max_level: u8,
        limit: usize,
    ) -> QueryResult {
        let query_start = std::time::Instant::now();
        let mut scanned = 0;

        let ids: Vec<EntryId> = self.by_importance
            .range(min_level..=max_level)
            .flat_map(|(_, ids)| {
                scanned += ids.len();
                ids.iter().copied()
            })
            .take(limit)
            .collect();

        let entries: Vec<TraceEntry> = ids
            .iter()
            .filter_map(|id| self.entries.get(id).cloned())
            .collect();

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 组合查询（多条件 AND）
    pub fn query_combined(
        &self,
        dimension: Option<&Dimension>,
        entry_type: Option<&EntryType>,
        status: Option<StatusKey>,
        tags: Option<&[String]>,
        limit: usize,
    ) -> QueryResult {
        let query_start = std::time::Instant::now();
        let mut candidate_sets: Vec<&HashSet<EntryId>> = vec![];

        // 收集各条件的候选集
        if let Some(dim) = dimension {
            if let Some(ids) = self.by_dimension.get(dim) {
                candidate_sets.push(ids);
            }
        }
        if let Some(et) = entry_type {
            if let Some(ids) = self.by_entry_type.get(et) {
                candidate_sets.push(ids);
            }
        }
        if let Some(st) = status {
            if let Some(ids) = self.by_status.get(&st) {
                candidate_sets.push(ids);
            }
        }

        // 计算交集
        let result_ids: HashSet<EntryId> = if candidate_sets.is_empty() {
            // 无条件，返回所有
            self.entries.keys().copied().collect()
        } else {
            // 从最小集合开始求交集
            candidate_sets.sort_by_key(|s| s.len());
            let mut result = candidate_sets[0].clone();
            for set in &candidate_sets[1..] {
                result = result.intersection(set).copied().collect();
            }
            result
        };

        // 如果有标签条件，进一步过滤
        let final_ids: HashSet<EntryId> = if let Some(tags) = tags {
            result_ids
                .into_iter()
                .filter(|id| {
                    if let Some(entry) = self.entries.get(id) {
                        tags.iter().any(|t| entry.tags.contains(t))
                    } else {
                        false
                    }
                })
                .collect()
        } else {
            result_ids
        };

        let scanned = final_ids.len();
        let entries: Vec<TraceEntry> = final_ids
            .iter()
            .take(limit)
            .filter_map(|id| self.entries.get(id).cloned())
            .collect();

        QueryResult {
            entries,
            query_time_us: query_start.elapsed().as_micros() as u64,
            scanned_count: scanned,
        }
    }

    /// 检查内容是否已存在（去重）
    pub fn contains_content(&self, content_hash: u64) -> bool {
        self.by_content_hash.contains_key(&content_hash)
    }

    /// 获取索引统计
    pub fn stats(&self) -> &IndexStats {
        &self.stats
    }

    /// 获取条目总数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 估算内存使用（字节）
    fn estimate_memory(&self) -> usize {
        let entry_size = std::mem::size_of::<TraceEntry>() * self.entries.len();
        let index_overhead =
            self.by_timestamp.len() * 32 +
            self.by_dimension.len() * 32 +
            self.by_entry_type.len() * 32 +
            self.by_status.len() * 32 +
            self.by_importance.len() * 32 +
            self.by_tag.len() * 64 +
            self.by_content_hash.len() * 16;

        entry_size + index_overhead
    }
}

impl Default for MultiDimensionalIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// 将 Importance 转换为数值级别
fn importance_to_level(importance: &Importance) -> u8 {
    match importance {
        Importance::Low => 0,
        Importance::Normal => 1,
        Importance::Important => 2,
        Importance::Critical => 3,
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_entry(
        dimension: Dimension,
        entry_type: EntryType,
        content: &str,
        status: Status,
    ) -> TraceEntry {
        TraceEntry {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            dimension,
            entry_type,
            content: content.to_string(),
            status,
            metadata: HashMap::new(),
            importance: Some(Importance::Normal),
            tags: vec!["test".to_string()],
            context_id: None,
        }
    }

    #[test]
    fn test_index_new() {
        let index = MultiDimensionalIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_index_insert() {
        let mut index = MultiDimensionalIndex::new();
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la",
            Status::Success,
        );

        index.insert(entry.clone());

        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_index_remove() {
        let mut index = MultiDimensionalIndex::new();
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la",
            Status::Success,
        );
        let id = entry.id;

        index.insert(entry);
        assert_eq!(index.len(), 1);

        let removed = index.remove(&id);
        assert!(removed.is_some());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_query_by_dimension() {
        let mut index = MultiDimensionalIndex::new();

        // 添加不同维度的条目
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "pwd",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::BlackBox,
            EntryType::LlmRequest,
            "hello",
            Status::Success,
        ));

        let result = index.query_by_dimension(&Dimension::Statistics, 100);
        assert_eq!(result.entries.len(), 2);

        let result = index.query_by_dimension(&Dimension::BlackBox, 100);
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_query_by_status() {
        let mut index = MultiDimensionalIndex::new();

        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "fail",
            Status::Failed("error".to_string()),
        ));

        let result = index.query_by_status(StatusKey::Success, 100);
        assert_eq!(result.entries.len(), 1);

        let result = index.query_by_status(StatusKey::Failed, 100);
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_query_by_tags() {
        let mut index = MultiDimensionalIndex::new();

        let mut entry1 = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls",
            Status::Success,
        );
        entry1.tags = vec!["rust".to_string(), "cli".to_string()];

        let mut entry2 = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "pwd",
            Status::Success,
        );
        entry2.tags = vec!["python".to_string()];

        index.insert(entry1);
        index.insert(entry2);

        let result = index.query_by_tags(&["rust".to_string()], 100);
        assert_eq!(result.entries.len(), 1);

        let result = index.query_by_tags(&["rust".to_string(), "python".to_string()], 100);
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_query_by_time_range() {
        let mut index = MultiDimensionalIndex::new();
        let now = Utc::now();

        let mut entry1 = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "old",
            Status::Success,
        );
        entry1.timestamp = now - Duration::hours(2);

        let mut entry2 = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "new",
            Status::Success,
        );
        entry2.timestamp = now;

        index.insert(entry1);
        index.insert(entry2);

        // 查询最近1小时
        let result = index.query_by_time_range(
            now - Duration::hours(1),
            now + Duration::minutes(1),
            100,
        );
        assert_eq!(result.entries.len(), 1);

        // 查询全部
        let result = index.query_by_time_range(
            now - Duration::hours(3),
            now + Duration::minutes(1),
            100,
        );
        assert_eq!(result.entries.len(), 2);
    }

    #[test]
    fn test_query_combined() {
        let mut index = MultiDimensionalIndex::new();

        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::SystemCommand,
            "help",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::BlackBox,
            EntryType::LlmRequest,
            "hello",
            Status::Success,
        ));

        // 组合查询：Statistics + ShellCommand
        let result = index.query_combined(
            Some(&Dimension::Statistics),
            Some(&EntryType::ShellCommand),
            None,
            None,
            100,
        );
        assert_eq!(result.entries.len(), 1);
    }

    #[test]
    fn test_contains_content() {
        let mut index = MultiDimensionalIndex::new();
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "unique content",
            Status::Success,
        );
        let hash = entry.content_hash();

        assert!(!index.contains_content(hash));
        index.insert(entry);
        assert!(index.contains_content(hash));
    }

    #[test]
    fn test_build_from() {
        let entries = vec![
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "1", Status::Success),
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "2", Status::Success),
            create_test_entry(Dimension::BlackBox, EntryType::LlmRequest, "3", Status::Success),
        ];

        let index = MultiDimensionalIndex::build_from(entries);

        assert_eq!(index.len(), 3);
        assert!(index.stats().build_time_us > 0);
        assert!(index.stats().estimated_memory_bytes > 0);
    }

    #[test]
    fn test_stats() {
        let mut index = MultiDimensionalIndex::new();

        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test",
            Status::Success,
        ));

        let stats = index.stats();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(*stats.dimension_counts.get(&Dimension::Statistics).unwrap(), 1);
        assert_eq!(*stats.status_counts.get(&StatusKey::Success).unwrap(), 1);
    }
}
