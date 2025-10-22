//! 统一追踪条目定义
//!
//! `TraceEntry` 是四维观测体系的统一数据抽象

use super::types::{Dimension, EntryType, Status};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 统一追踪条目
///
/// 聚合四个维度的记录，提供统一视图
///
/// # 设计理念
///
/// - **统一抽象**：不同数据源映射到同一结构
/// - **元数据灵活**：通过 HashMap 存储维度特定信息
/// - **时间排序**：timestamp 作为主要排序键
/// - **去重支持**：通过 id 和内容哈希实现去重
///
/// # 示例
///
/// ```rust
/// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
///
/// let entry = TraceEntry::new(
///     Dimension::Statistics,
///     EntryType::ShellCommand,
///     "ls -la".to_string(),
///     Status::Success,
/// );
///
/// println!("{}", entry.format());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// 唯一 ID
    pub id: Uuid,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 来源维度
    pub dimension: Dimension,

    /// 条目类型
    pub entry_type: EntryType,

    /// 核心内容
    pub content: String,

    /// 状态
    pub status: Status,

    /// 元数据（维度特定）
    ///
    /// 示例：
    /// - Statistics: {"frequency": 10, "last_used": "2025-10-22"}
    /// - Coordination: {"duration_ms": 1234, "command_type": "shell"}
    /// - BlackBox: {"model": "deepseek-chat", "tokens": 500}
    /// - Memory: {"role": "user", "context_id": "abc123"}
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TraceEntry {
    /// 创建新条目
    ///
    /// # 示例
    ///
    /// ```rust
    /// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
    ///
    /// let entry = TraceEntry::new(
    ///     Dimension::Statistics,
    ///     EntryType::ShellCommand,
    ///     "ls -la".to_string(),
    ///     Status::Success,
    /// );
    /// ```
    pub fn new(
        dimension: Dimension,
        entry_type: EntryType,
        content: String,
        status: Status,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            dimension,
            entry_type,
            content,
            status,
            metadata: HashMap::new(),
        }
    }

    /// 创建带元数据的条目
    ///
    /// # 示例
    ///
    /// ```rust
    /// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// let mut metadata = HashMap::new();
    /// metadata.insert("frequency".to_string(), json!(10));
    ///
    /// let entry = TraceEntry::with_metadata(
    ///     Dimension::Statistics,
    ///     EntryType::ShellCommand,
    ///     "ls".to_string(),
    ///     Status::Success,
    ///     metadata,
    /// );
    /// ```
    pub fn with_metadata(
        dimension: Dimension,
        entry_type: EntryType,
        content: String,
        status: Status,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            dimension,
            entry_type,
            content,
            status,
            metadata,
        }
    }

    /// 添加元数据字段
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据字段
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 格式化输出（彩色，完整信息）
    ///
    /// # 输出格式
    ///
    /// ```text
    /// 📊 ✓ [12:34:56] Statistics ShellCommand
    ///    ls -la
    ///    Metadata: frequency=10
    /// ```
    pub fn format(&self) -> String {
        let dim_icon = self.dimension.icon();
        let status_icon = self.status_colored();
        let time = self.timestamp.format("%H:%M:%S").to_string().dimmed();
        let dimension = format!("{}", self.dimension).cyan();
        let entry_type = format!("{}", self.entry_type).yellow();

        let mut lines = vec![format!(
            "{} {} [{}] {} {}",
            dim_icon, status_icon, time, dimension, entry_type
        )];

        // 内容（缩进）
        let content_lines: Vec<&str> = self.content.lines().collect();
        if content_lines.len() == 1 {
            lines.push(format!("   {}", content_lines[0].dimmed()));
        } else {
            for line in content_lines {
                lines.push(format!("   {}", line.dimmed()));
            }
        }

        // 元数据（如果存在）
        if !self.metadata.is_empty() {
            let metadata_str = self
                .metadata
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            lines.push(format!("   {}: {}", "Metadata".dimmed(), metadata_str.dimmed()));
        }

        lines.join("\n")
    }

    /// 简短预览（单行）
    ///
    /// # 输出格式
    ///
    /// ```text
    /// 📊 ✓ [12:34:56] Statistics: ls -la
    /// ```
    pub fn preview(&self) -> String {
        let dim_icon = self.dimension.icon();
        let status_icon = self.status_colored();
        let time = self.timestamp.format("%H:%M:%S").to_string().dimmed();
        let dimension = format!("{}", self.dimension).cyan();

        // 内容截断（最多 60 字符）
        // 安全的 UTF-8 边界检查，避免切割多字节字符（如中文）
        let content_preview = if self.content.len() > 60 {
            let mut cutoff = 57.min(self.content.len());
            while cutoff > 0 && !self.content.is_char_boundary(cutoff) {
                cutoff -= 1;
            }
            format!("{}...", &self.content[..cutoff])
        } else {
            self.content.clone()
        };

        format!(
            "{} {} [{}] {}: {}",
            dim_icon,
            status_icon,
            time,
            dimension,
            content_preview.dimmed()
        )
    }

    /// 获取彩色状态图标
    fn status_colored(&self) -> colored::ColoredString {
        match &self.status {
            Status::Success => self.status.icon().green(),
            Status::Failed(_) => self.status.icon().red(),
            Status::Running => self.status.icon().yellow(),
            Status::Cancelled => self.status.icon().dimmed(),
        }
    }

    /// 获取维度图标（便捷方法）
    pub fn dimension_icon(&self) -> &'static str {
        self.dimension.icon()
    }

    /// 获取条目类型图标（便捷方法）
    pub fn entry_type_icon(&self) -> &'static str {
        self.entry_type.icon()
    }

    /// 获取状态图标（便捷方法）
    pub fn status_icon(&self) -> &'static str {
        self.status.icon()
    }

    /// 计算内容哈希（用于去重）
    pub fn content_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        hasher.finish()
    }

    /// 获取时间桶（用于去重，10秒精度）
    pub fn time_bucket(&self) -> i64 {
        self.timestamp.timestamp() / 10
    }

    /// 生成去重键
    pub fn dedup_key(&self) -> String {
        format!("{}_{}", self.content_hash(), self.time_bucket())
    }
}

impl PartialEq for TraceEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TraceEntry {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_trace_entry_new() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        assert_eq!(entry.dimension, Dimension::Statistics);
        assert_eq!(entry.entry_type, EntryType::ShellCommand);
        assert_eq!(entry.content, "ls -la");
        assert!(entry.status.is_success());
        assert!(entry.metadata.is_empty());
    }

    #[test]
    fn test_trace_entry_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("frequency".to_string(), json!(10));
        metadata.insert("last_used".to_string(), json!("2025-10-22"));

        let entry = TraceEntry::with_metadata(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
            metadata,
        );

        assert_eq!(entry.get_metadata("frequency"), Some(&json!(10)));
        assert_eq!(
            entry.get_metadata("last_used"),
            Some(&json!("2025-10-22"))
        );
    }

    #[test]
    fn test_add_metadata() {
        let mut entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
        );

        entry.add_metadata("test_key".to_string(), json!("test_value"));
        assert_eq!(entry.get_metadata("test_key"), Some(&json!("test_value")));
    }

    #[test]
    fn test_format() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let formatted = entry.format();
        assert!(formatted.contains("📊"));
        assert!(formatted.contains("Statistics"));
        assert!(formatted.contains("ShellCommand"));
        assert!(formatted.contains("ls -la"));
    }

    #[test]
    fn test_preview() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let preview = entry.preview();
        assert!(preview.contains("📊"));
        assert!(preview.contains("Statistics"));
        assert!(preview.contains("ls -la"));
    }

    #[test]
    fn test_preview_truncation() {
        let long_content = "a".repeat(100);
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            long_content,
            Status::Success,
        );

        let preview = entry.preview();
        assert!(preview.contains("..."));
        assert!(preview.len() < 150); // 应该被截断
    }

    #[test]
    fn test_content_hash() {
        let entry1 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let entry2 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        // 相同内容应该有相同的哈希
        assert_eq!(entry1.content_hash(), entry2.content_hash());
    }

    #[test]
    fn test_content_hash_different() {
        let entry1 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let entry2 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "pwd".to_string(),
            Status::Success,
        );

        // 不同内容应该有不同的哈希
        assert_ne!(entry1.content_hash(), entry2.content_hash());
    }

    #[test]
    fn test_dedup_key() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let key = entry.dedup_key();
        assert!(key.contains("_")); // 应该包含分隔符
    }

    #[test]
    fn test_dimension_icon() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
        );

        assert_eq!(entry.dimension_icon(), "📊");
    }

    #[test]
    fn test_entry_type_icon() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
        );

        assert_eq!(entry.entry_type_icon(), "🐚");
    }

    #[test]
    fn test_status_icon() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
        );

        assert_eq!(entry.status_icon(), "✓");
    }

    #[test]
    fn test_equality() {
        let entry1 = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls".to_string(),
            Status::Success,
        );

        let entry2 = entry1.clone();

        assert_eq!(entry1, entry2);
    }

    #[test]
    fn test_serialization() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TraceEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.dimension, deserialized.dimension);
        assert_eq!(entry.entry_type, deserialized.entry_type);
        assert_eq!(entry.content, deserialized.content);
    }
}
