//! 统一追踪条目定义
//!
//! `TraceEntry` 是四维观测体系的统一数据抽象

use super::types::{Dimension, EntryType, Importance, Status};
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

    // ━━━━━ Memory 维度专属字段 (v1.16.0 Phase 3) ━━━━━
    /// 重要性级别（仅 Memory 维度使用）
    ///
    /// 用于标记记忆条目的重要程度，影响淡忘策略：
    /// - Low: 可以快速淡忘
    /// - Normal: 默认级别
    /// - Important: 需要长期保留
    /// - Critical: 永久保留
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importance: Option<Importance>,

    /// 标签列表（仅 Memory 维度使用）
    ///
    /// 用于分类和检索记忆条目，如：["project:realconsole", "lang:rust"]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// 工作上下文 ID（仅 Memory 维度使用）
    ///
    /// 关联相关的记忆条目，用于追踪对话或任务上下文
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
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
            importance: None,
            tags: Vec::new(),
            context_id: None,
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
            importance: None,
            tags: Vec::new(),
            context_id: None,
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

    // ━━━━━ Memory 维度专属方法 (v1.16.0 Phase 3) ━━━━━

    /// 设置重要性级别（仅 Memory 维度）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status, Importance};
    ///
    /// let mut entry = TraceEntry::new(
    ///     Dimension::Memory,
    ///     EntryType::ContextMessage,
    ///     "重要对话".to_string(),
    ///     Status::Success,
    /// );
    ///
    /// entry.set_importance(Importance::Critical);
    /// ```
    pub fn set_importance(&mut self, importance: Importance) {
        self.importance = Some(importance);
    }

    /// 获取重要性级别
    pub fn get_importance(&self) -> Option<Importance> {
        self.importance
    }

    /// 添加标签
    ///
    /// # 示例
    ///
    /// ```rust
    /// use realconsole::tracer::{TraceEntry, Dimension, EntryType, Status};
    ///
    /// let mut entry = TraceEntry::new(
    ///     Dimension::Memory,
    ///     EntryType::ContextMessage,
    ///     "Rust 学习笔记".to_string(),
    ///     Status::Success,
    /// );
    ///
    /// entry.add_tag("lang:rust".to_string());
    /// entry.add_tag("learning".to_string());
    /// ```
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 批量添加标签
    pub fn add_tags(&mut self, tags: Vec<String>) {
        for tag in tags {
            self.add_tag(tag);
        }
    }

    /// 获取所有标签
    pub fn get_tags(&self) -> &[String] {
        &self.tags
    }

    /// 检查是否包含指定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// 设置上下文 ID
    pub fn set_context_id(&mut self, context_id: String) {
        self.context_id = Some(context_id);
    }

    /// 获取上下文 ID
    pub fn get_context_id(&self) -> Option<&str> {
        self.context_id.as_deref()
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

        // Memory 维度专属字段
        if self.dimension == Dimension::Memory {
            // 重要性
            if let Some(importance) = self.importance {
                lines.push(format!(
                    "   {}: {} {}",
                    "Importance".dimmed(),
                    importance.icon(),
                    format!("{}", importance).yellow()
                ));
            }

            // 标签
            if !self.tags.is_empty() {
                let tags_str = self.tags.join(", ");
                lines.push(format!("   {}: {}", "Tags".dimmed(), tags_str.cyan()));
            }

            // 上下文 ID
            if let Some(context_id) = &self.context_id {
                lines.push(format!(
                    "   {}: {}",
                    "Context".dimmed(),
                    context_id.dimmed()
                ));
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

    // ━━━━━ v1.16.0 Phase 3: Memory 维度专属字段测试 ━━━━━

    #[test]
    fn test_set_and_get_importance() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "重要对话".to_string(),
            Status::Success,
        );

        // 初始状态应该为 None
        assert_eq!(entry.get_importance(), None);

        // 设置重要性
        entry.set_importance(Importance::Critical);
        assert_eq!(entry.get_importance(), Some(Importance::Critical));

        // 修改重要性
        entry.set_importance(Importance::Low);
        assert_eq!(entry.get_importance(), Some(Importance::Low));
    }

    #[test]
    fn test_add_tag() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "学习笔记".to_string(),
            Status::Success,
        );

        // 初始状态应该为空
        assert_eq!(entry.get_tags(), &[] as &[String]);

        // 添加标签
        entry.add_tag("rust".to_string());
        assert_eq!(entry.get_tags(), &["rust"]);
        assert!(entry.has_tag("rust"));
        assert!(!entry.has_tag("python"));

        // 添加重复标签应该被忽略
        entry.add_tag("rust".to_string());
        assert_eq!(entry.get_tags(), &["rust"]);

        // 添加更多标签
        entry.add_tag("learning".to_string());
        assert_eq!(entry.get_tags().len(), 2);
        assert!(entry.has_tag("rust"));
        assert!(entry.has_tag("learning"));
    }

    #[test]
    fn test_add_tags_batch() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "项目讨论".to_string(),
            Status::Success,
        );

        let tags = vec![
            "project:realconsole".to_string(),
            "lang:rust".to_string(),
            "priority:high".to_string(),
        ];

        entry.add_tags(tags);
        assert_eq!(entry.get_tags().len(), 3);
        assert!(entry.has_tag("project:realconsole"));
        assert!(entry.has_tag("lang:rust"));
        assert!(entry.has_tag("priority:high"));
    }

    #[test]
    fn test_set_and_get_context_id() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "对话内容".to_string(),
            Status::Success,
        );

        // 初始状态应该为 None
        assert_eq!(entry.get_context_id(), None);

        // 设置上下文 ID
        entry.set_context_id("ctx_123".to_string());
        assert_eq!(entry.get_context_id(), Some("ctx_123"));

        // 修改上下文 ID
        entry.set_context_id("ctx_456".to_string());
        assert_eq!(entry.get_context_id(), Some("ctx_456"));
    }

    #[test]
    fn test_memory_entry_format() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "测试内容".to_string(),
            Status::Success,
        );

        entry.set_importance(Importance::Important);
        entry.add_tag("test".to_string());
        entry.set_context_id("ctx_test".to_string());

        let formatted = entry.format();

        // 验证包含 Memory 维度标记
        assert!(formatted.contains("💭"));
        assert!(formatted.contains("Memory"));

        // 验证包含内容
        assert!(formatted.contains("测试内容"));

        // 验证包含重要性（通过图标或文本）
        assert!(formatted.contains("Importance") || formatted.contains("●"));

        // 验证包含标签
        assert!(formatted.contains("Tags") || formatted.contains("test"));

        // 验证包含上下文 ID
        assert!(formatted.contains("Context") || formatted.contains("ctx_test"));
    }

    #[test]
    fn test_memory_serialization_with_fields() {
        let mut entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "测试序列化".to_string(),
            Status::Success,
        );

        entry.set_importance(Importance::Critical);
        entry.add_tags(vec!["tag1".to_string(), "tag2".to_string()]);
        entry.set_context_id("ctx_123".to_string());

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: TraceEntry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.id, deserialized.id);
        assert_eq!(entry.dimension, deserialized.dimension);
        assert_eq!(entry.get_importance(), deserialized.get_importance());
        assert_eq!(entry.get_tags(), deserialized.get_tags());
        assert_eq!(entry.get_context_id(), deserialized.get_context_id());
    }

    #[test]
    fn test_memory_serialization_skip_empty_fields() {
        let entry = TraceEntry::new(
            Dimension::Memory,
            EntryType::ContextMessage,
            "空字段测试".to_string(),
            Status::Success,
        );

        let json = serde_json::to_string(&entry).unwrap();

        // 空字段应该被跳过，不包含在 JSON 中
        assert!(!json.contains("\"importance\""));
        assert!(!json.contains("\"context_id\""));
        // tags 使用 default，可能序列化为空数组
    }

    #[test]
    fn test_non_memory_dimension_fields_remain_empty() {
        let entry = TraceEntry::new(
            Dimension::Statistics,
            EntryType::ShellCommand,
            "ls -la".to_string(),
            Status::Success,
        );

        // 非 Memory 维度的条目，Memory 字段应该保持默认值
        assert_eq!(entry.get_importance(), None);
        assert_eq!(entry.get_tags(), &[] as &[String]);
        assert_eq!(entry.get_context_id(), None);
    }
}
