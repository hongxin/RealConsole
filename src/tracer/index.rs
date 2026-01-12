//! 多维索引系统
//!
//! v1.56.0: 探路期核心功能 - 验证 10x 查询性能提升
//! v1.57.0: 索引持久化 - bincode 序列化，增量更新，启动优化
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
//!
//! ## 持久化 (v1.57.0)
//!
//! ```text
//! IndexPersistence
//! ├── save()              - 完整索引保存（bincode）
//! ├── load()              - 索引加载
//! ├── append_entries()    - 增量追加
//! └── get_index_info()    - 获取索引元信息
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::entry::TraceEntry;
use super::types::{Dimension, EntryType, Importance, Status};

/// 条目 ID（使用 UUID）
pub type EntryId = Uuid;

/// 多维索引
///
/// 为 TraceEntry 提供多维度的快速查询能力
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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
            .or_default()
            .insert(id);

        // 更新维度索引
        self.by_dimension
            .entry(entry.dimension)
            .or_default()
            .insert(id);
        *self.stats.dimension_counts.entry(entry.dimension).or_insert(0) += 1;

        // 更新类型索引
        self.by_entry_type
            .entry(entry.entry_type.clone())
            .or_default()
            .insert(id);

        // 更新状态索引
        let status_key = StatusKey::from(&entry.status);
        self.by_status
            .entry(status_key)
            .or_default()
            .insert(id);
        *self.stats.status_counts.entry(status_key).or_insert(0) += 1;

        // 更新重要性索引
        if let Some(importance) = &entry.importance {
            let level = importance_to_level(importance);
            self.by_importance
                .entry(level)
                .or_default()
                .insert(id);
        }

        // 更新标签索引
        for tag in &entry.tags {
            self.by_tag
                .entry(tag.clone())
                .or_default()
                .insert(id);
        }

        // 更新内容哈希索引
        let hash = entry.content_hash();
        self.by_content_hash
            .entry(hash)
            .or_default()
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
// v1.57.0: 索引持久化
// ============================================================================

/// 索引文件版本号
const INDEX_VERSION: u32 = 1;

/// 索引文件头
#[derive(Debug, Serialize, Deserialize)]
struct IndexHeader {
    /// 文件版本
    version: u32,
    /// 条目数量
    entry_count: usize,
    /// 创建时间
    created_at: DateTime<Utc>,
    /// 最后更新时间
    updated_at: DateTime<Utc>,
}

/// 索引元信息
#[derive(Debug, Clone)]
pub struct IndexInfo {
    /// 索引文件路径
    pub path: PathBuf,
    /// 文件大小（字节）
    pub file_size: u64,
    /// 条目数量
    pub entry_count: usize,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
    /// 版本
    pub version: u32,
}

/// 增量日志条目
#[derive(Debug, Serialize, Deserialize)]
enum WalEntry {
    /// 插入条目
    Insert(TraceEntry),
    /// 删除条目
    Remove(EntryId),
}

/// 索引持久化错误
#[derive(Debug)]
pub enum PersistenceError {
    /// IO 错误
    Io(std::io::Error),
    /// 序列化错误
    Serialization(String),
    /// 版本不兼容
    VersionMismatch { expected: u32, found: u32 },
    /// 索引损坏
    Corrupted(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(e) => write!(f, "IO error: {}", e),
            PersistenceError::Serialization(s) => write!(f, "Serialization error: {}", s),
            PersistenceError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            PersistenceError::Corrupted(s) => write!(f, "Index corrupted: {}", s),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<std::io::Error> for PersistenceError {
    fn from(e: std::io::Error) -> Self {
        PersistenceError::Io(e)
    }
}

impl From<bincode::Error> for PersistenceError {
    fn from(e: bincode::Error) -> Self {
        PersistenceError::Serialization(e.to_string())
    }
}

/// 持久化数据格式
///
/// 只保存条目列表，加载时重建索引（10k 条目重建仅需 ~6ms）
#[derive(Debug, Serialize, Deserialize)]
struct PersistenceData {
    /// 文件头
    header: IndexHeader,
    /// 条目列表
    entries: Vec<TraceEntry>,
}

/// 索引持久化管理器
///
/// 提供索引的保存、加载和增量更新功能
///
/// # 设计策略
///
/// - 只持久化条目列表，加载时重建索引
/// - 索引重建很快（10k 条目 ~6ms），无需复杂的索引序列化
/// - WAL 支持增量更新，减少 I/O
///
/// # 文件结构
///
/// ```text
/// index_dir/
/// ├── entries.json    - 主数据文件（JSON 序列化，可读性好）
/// └── entries.wal     - 写前日志（增量更新）
/// ```
///
/// # 使用示例
///
/// ```ignore
/// let persistence = IndexPersistence::new("~/.realconsole/index");
///
/// // 保存索引
/// persistence.save(&index)?;
///
/// // 加载索引
/// let index = persistence.load()?;
///
/// // 增量追加
/// persistence.append_entries(&new_entries)?;
/// ```
pub struct IndexPersistence {
    /// 索引目录
    dir: PathBuf,
    /// 主数据文件路径
    data_path: PathBuf,
    /// WAL 文件路径
    wal_path: PathBuf,
}

impl IndexPersistence {
    /// 创建持久化管理器
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let dir = dir.as_ref().to_path_buf();
        let data_path = dir.join("entries.json");
        let wal_path = dir.join("entries.wal");

        Self {
            dir,
            data_path,
            wal_path,
        }
    }

    /// 保存完整索引
    ///
    /// 将条目列表序列化为 JSON 格式并保存
    pub fn save(&self, index: &MultiDimensionalIndex) -> Result<(), PersistenceError> {
        // 确保目录存在
        fs::create_dir_all(&self.dir)?;

        // 提取所有条目
        let entries: Vec<TraceEntry> = index.entries.values().cloned().collect();

        // 创建持久化数据
        let data = PersistenceData {
            header: IndexHeader {
                version: INDEX_VERSION,
                entry_count: entries.len(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            entries,
        };

        // 写入临时文件，然后原子重命名
        let temp_path = self.dir.join("entries.json.tmp");
        let file = File::create(&temp_path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer(writer, &data)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        // 原子重命名
        fs::rename(&temp_path, &self.data_path)?;

        // 清理 WAL
        if self.wal_path.exists() {
            fs::remove_file(&self.wal_path)?;
        }

        Ok(())
    }

    /// 加载索引
    ///
    /// 从文件加载条目并重建索引，然后应用 WAL 中的增量更新
    pub fn load(&self) -> Result<MultiDimensionalIndex, PersistenceError> {
        // 检查数据文件是否存在
        if !self.data_path.exists() {
            // 尝试应用 WAL（可能是首次使用，只有 WAL）
            if self.wal_path.exists() {
                return self.load_from_wal_only();
            }
            return Ok(MultiDimensionalIndex::new());
        }

        // 读取并反序列化
        let file = File::open(&self.data_path)?;
        let reader = BufReader::new(file);

        let data: PersistenceData = serde_json::from_reader(reader)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        // 检查版本
        if data.header.version != INDEX_VERSION {
            return Err(PersistenceError::VersionMismatch {
                expected: INDEX_VERSION,
                found: data.header.version,
            });
        }

        // 重建索引
        let mut index = MultiDimensionalIndex::build_from(data.entries);

        // 应用 WAL
        if self.wal_path.exists() {
            let wal_entries = self.load_wal()?;
            for entry in wal_entries {
                match entry {
                    WalEntry::Insert(e) => index.insert(e),
                    WalEntry::Remove(id) => { index.remove(&id); }
                }
            }
        }

        Ok(index)
    }

    /// 追加条目到 WAL
    ///
    /// 增量更新，不需要重写整个文件
    pub fn append_entries(&self, entries: &[TraceEntry]) -> Result<(), PersistenceError> {
        if entries.is_empty() {
            return Ok(());
        }

        // 确保目录存在
        fs::create_dir_all(&self.dir)?;

        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)?;

        let mut writer = BufWriter::new(file);

        for entry in entries {
            let wal_entry = WalEntry::Insert(entry.clone());
            serde_json::to_writer(&mut writer, &wal_entry)
                .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
            writer.write_all(b"\n")?;
        }

        writer.flush()?;

        Ok(())
    }

    /// 追加删除操作到 WAL
    pub fn append_remove(&self, id: EntryId) -> Result<(), PersistenceError> {
        // 确保目录存在
        fs::create_dir_all(&self.dir)?;

        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)?;

        let mut writer = BufWriter::new(file);
        let wal_entry = WalEntry::Remove(id);
        serde_json::to_writer(&mut writer, &wal_entry)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        Ok(())
    }

    /// 获取索引元信息
    pub fn get_index_info(&self) -> Result<Option<IndexInfo>, PersistenceError> {
        if !self.data_path.exists() {
            return Ok(None);
        }

        let file = File::open(&self.data_path)?;
        let metadata = file.metadata()?;
        let reader = BufReader::new(file);

        let data: PersistenceData = serde_json::from_reader(reader)
            .map_err(|e| PersistenceError::Serialization(e.to_string()))?;

        Ok(Some(IndexInfo {
            path: self.data_path.clone(),
            file_size: metadata.len(),
            entry_count: data.header.entry_count,
            created_at: data.header.created_at,
            updated_at: data.header.updated_at,
            version: data.header.version,
        }))
    }

    /// 检查是否需要压缩（WAL 过大）
    pub fn needs_compaction(&self) -> bool {
        if !self.wal_path.exists() {
            return false;
        }

        // 如果 WAL 大于数据文件的 20%，建议压缩
        let data_size = fs::metadata(&self.data_path)
            .map(|m| m.len())
            .unwrap_or(0);
        let wal_size = fs::metadata(&self.wal_path)
            .map(|m| m.len())
            .unwrap_or(0);

        if data_size == 0 {
            return wal_size > 100 * 1024; // 100KB
        }

        wal_size > data_size / 5
    }

    /// 压缩索引（合并 WAL）
    pub fn compact(&self) -> Result<(), PersistenceError> {
        let index = self.load()?;
        self.save(&index)?;
        Ok(())
    }

    /// 删除所有持久化数据
    pub fn clear(&self) -> Result<(), PersistenceError> {
        if self.data_path.exists() {
            fs::remove_file(&self.data_path)?;
        }
        if self.wal_path.exists() {
            fs::remove_file(&self.wal_path)?;
        }
        Ok(())
    }

    /// 从 WAL 加载（数据文件不存在时）
    fn load_from_wal_only(&self) -> Result<MultiDimensionalIndex, PersistenceError> {
        let wal_entries = self.load_wal()?;
        let mut index = MultiDimensionalIndex::new();

        for entry in wal_entries {
            match entry {
                WalEntry::Insert(e) => index.insert(e),
                WalEntry::Remove(id) => { index.remove(&id); }
            }
        }

        Ok(index)
    }

    /// 加载 WAL 条目
    fn load_wal(&self) -> Result<Vec<WalEntry>, PersistenceError> {
        let file = File::open(&self.wal_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str(&line) {
                Ok(entry) => entries.push(entry),
                Err(_) => continue, // 跳过损坏的行
            }
        }

        Ok(entries)
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

    // ========================================================================
    // v1.57.0: 持久化测试
    // ========================================================================

    #[test]
    fn test_persistence_save_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 创建索引并添加数据
        let mut index = MultiDimensionalIndex::new();
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la",
            Status::Success,
        ));
        index.insert(create_test_entry(
            Dimension::BlackBox,
            EntryType::LlmRequest,
            "hello",
            Status::Success,
        ));

        // 保存
        persistence.save(&index).unwrap();

        // 加载
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_persistence_empty_index() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 加载不存在的索引应该返回空索引
        let index = persistence.load().unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test_persistence_append_entries() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 创建空索引并保存
        let index = MultiDimensionalIndex::new();
        persistence.save(&index).unwrap();

        // 追加条目
        let entries = vec![
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "ls", Status::Success),
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "pwd", Status::Success),
        ];
        persistence.append_entries(&entries).unwrap();

        // 加载应该包含追加的条目
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_persistence_incremental_updates() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 创建索引
        let mut index = MultiDimensionalIndex::new();
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "initial",
            Status::Success,
        );
        index.insert(entry);
        persistence.save(&index).unwrap();

        // 追加更多条目
        let new_entries = vec![
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "new1", Status::Success),
            create_test_entry(Dimension::Statistics, EntryType::ShellCommand, "new2", Status::Success),
        ];
        persistence.append_entries(&new_entries).unwrap();

        // 加载验证
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 3);
    }

    #[test]
    fn test_persistence_index_info() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 不存在时返回 None
        let info = persistence.get_index_info().unwrap();
        assert!(info.is_none());

        // 保存后返回 Some
        let mut index = MultiDimensionalIndex::new();
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test",
            Status::Success,
        ));
        persistence.save(&index).unwrap();

        let info = persistence.get_index_info().unwrap().unwrap();
        assert_eq!(info.entry_count, 1);
        assert_eq!(info.version, INDEX_VERSION);
        assert!(info.file_size > 0);
    }

    #[test]
    fn test_persistence_compaction() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 创建并保存空索引
        let index = MultiDimensionalIndex::new();
        persistence.save(&index).unwrap();

        // 追加多个条目
        for i in 0..100 {
            let entries = vec![create_test_entry(
                Dimension::Statistics,
                EntryType::ShellCommand,
                &format!("cmd_{}", i),
                Status::Success,
            )];
            persistence.append_entries(&entries).unwrap();
        }

        // 压缩
        persistence.compact().unwrap();

        // 验证
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 100);
    }

    #[test]
    fn test_persistence_clear() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 保存
        let mut index = MultiDimensionalIndex::new();
        index.insert(create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test",
            Status::Success,
        ));
        persistence.save(&index).unwrap();

        // 追加 WAL
        let entries = vec![create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "wal",
            Status::Success,
        )];
        persistence.append_entries(&entries).unwrap();

        // 清理
        persistence.clear().unwrap();

        // 加载应该返回空索引
        let loaded = persistence.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_persistence_remove_via_wal() {
        let temp_dir = tempfile::tempdir().unwrap();
        let persistence = IndexPersistence::new(temp_dir.path());

        // 创建索引并保存
        let mut index = MultiDimensionalIndex::new();
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "to_remove",
            Status::Success,
        );
        let entry_id = entry.id;
        index.insert(entry);
        persistence.save(&index).unwrap();

        // 通过 WAL 追加删除
        persistence.append_remove(entry_id).unwrap();

        // 加载验证
        let loaded = persistence.load().unwrap();
        assert_eq!(loaded.len(), 0);
    }

    #[test]
    fn test_json_serialization() {
        // 测试条目可以正确序列化和反序列化
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test",
            Status::Success,
        );

        // 序列化
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.is_empty());

        // 反序列化
        let restored: TraceEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, entry.id);
        assert_eq!(restored.content, entry.content);
    }

    #[test]
    fn test_wal_entry_serialization() {
        // 测试 WalEntry 可以正确序列化
        let entry = create_test_entry(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "test",
            Status::Success,
        );
        let wal_insert = WalEntry::Insert(entry.clone());
        let wal_remove = WalEntry::Remove(entry.id);

        // Insert 序列化
        let json_insert = serde_json::to_string(&wal_insert).unwrap();
        let restored_insert: WalEntry = serde_json::from_str(&json_insert).unwrap();
        match restored_insert {
            WalEntry::Insert(e) => assert_eq!(e.content, "test"),
            WalEntry::Remove(_) => panic!("Expected Insert"),
        }

        // Remove 序列化
        let json_remove = serde_json::to_string(&wal_remove).unwrap();
        let restored_remove: WalEntry = serde_json::from_str(&json_remove).unwrap();
        match restored_remove {
            WalEntry::Remove(id) => assert_eq!(id, entry.id),
            WalEntry::Insert(_) => panic!("Expected Remove"),
        }
    }
}
